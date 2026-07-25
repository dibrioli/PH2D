//! The **per-stroke coverage buffer** and the LAW by which a stroke's dabs accumulate into it.
//!
//! A stroke lays its paint through many overlapping dabs, so "how much of this stroke's paint is at
//! this texel?" has to be answered by a buffer that outlives the dab. This module owns that answer —
//! one place, two laws, and the arithmetic written once so the per-pixel stamp and the ramped stamp
//! cannot disagree about it.
//!
//! ## Why there are two laws, and why the second one exists
//!
//! [`StrokeCoverLaw::BuildUp`] is what the brush has always done: each dab moves the texel a
//! *fraction* of the way to a ceiling (`m += w·(cap − m)`), so the dab profile acts as a **rate**.
//! Over `n` overlapping dabs the texel reaches `cap·(1 − Π(1 − w_k))` — it converges to the ceiling
//! from below, which is exactly the "paint over it again to deepen it" that pigment wants, and it is
//! the law GIMP ships for its non-incremental paint core (`gimp_gegl_combine_mask_weird`'s
//! non-stipple branch: `if (opacity > dest) dest += (opacity − dest) * mask * opacity`).
//!
//! [`StrokeCoverLaw::Envelope`] is **Krita's Wash mode** (its internal *Alpha Darken*): the dab
//! profile is the **target**, not a rate, and the stroke keeps the largest one it has seen
//! (`m = max(m, w·g·coverage)`). Krita's own description of why it exists — *"ensures the line
//! doesn't get darker when you cross it again and again"*, without *"the circular pattern you can see
//! in Build-Up"* — is precisely the defect this law fixes here.
//!
//! ## The consequence that decides which one a stroke wants
//!
//! Under `BuildUp` the feather is a function of **how many dabs crossed the texel**, and near a
//! stroke's rim that count falls off with distance, so the soft shoulder converges toward the
//! geometric rim: the transition band narrows with every pass (measured on the mask brush:
//! **3.53 px after one pass → 1.38 px after fifteen**), and once it is thinner than a texel the edge
//! is a hard, *scalloped* boundary — the individual dab discs, binarised (the sawtooth of the 50 %
//! contour tripled over the same fifteen passes: 0.035 px → 0.106 px).
//!
//! Under `Envelope` the stroke's coverage is the **envelope of its dab profiles** — a pure function
//! of the path and the brush, independent of the dab spacing and of the polling rate — so no amount
//! of scrubbing sharpens it. That is what a *coverage channel* (a mask) needs, and it is why the
//! Mask brush asks for this law while pigment keeps `BuildUp`. Note this is a within-STROKE law only:
//! consecutive strokes still build up on each other, because the buffer starts fresh per stroke and
//! the deposit composites over what the previous stroke left (see the `add` algebra below) — so
//! neighbouring strokes still SUM in the valley between them, which a cross-stroke envelope (`min`)
//! would leave lighter than the paint around it (a visible seam; tried and rejected, doc 25 §13.8).
//!
//! ## The `add` algebra — why the deposit lands on the pre-stroke pixels
//!
//! The caller blends the dab at `a = add / (1 − m)`. That is not a fudge: writing `A_n` for the
//! total fraction of the stroke's paint laid after `n` dabs, source-over gives
//! `1 − A_n = Π(1 − a_k)`, and substituting `a_k = (m_k − m_{k−1}) / (1 − m_{k−1})` telescopes to
//! `1 − A_n = 1 − m_n` **exactly** (the same telescoping the impasto bite pays for its own
//! spacing-independence). So the canvas always holds `pre_stroke·(1 − m) + colour·m`: the stroke is
//! applied to the state it started from, which is the other half of GIMP's model (it applies the
//! constant-mode paint from the undo snapshot, never from the live drawable) — and we get it without
//! keeping a copy of the canvas.

/// How a stroke's dabs accumulate into its per-stroke coverage buffer. There is no `Default`: every
/// caller states which law its medium wants, because the two disagree on the one thing that matters
/// (whether re-crossing a texel deepens it) and a forgotten default would silently pick pigment's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrokeCoverLaw {
    /// Pigment: the dab profile is a RATE toward the ceiling — crossing again deepens (GIMP).
    BuildUp,
    /// Coverage channel: the dab profile is the TARGET, kept by `max` — crossing again is inert,
    /// so the feather is a fact of the path and never hardens (Krita Wash / Alpha Darken).
    Envelope,
}

/// A stroke's coverage buffer (canvas-sized, 1 byte per pixel, `0` = nothing of this stroke laid yet)
/// plus the law it accumulates by. Borrowed for the duration of one dab.
pub struct StrokeCover<'a> {
    pub buf: &'a mut [u8],
    pub law: StrokeCoverLaw,
}

impl<'a> StrokeCover<'a> {
    /// The pigment buffer (today's Accumulate-OFF cap).
    #[must_use]
    pub fn build_up(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            law: StrokeCoverLaw::BuildUp,
        }
    }

    /// The coverage-channel buffer (the Mask brush).
    #[must_use]
    pub fn envelope(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            law: StrokeCoverLaw::Envelope,
        }
    }

    /// Re-borrow for the next dab (the stamp takes the cover by value per dab).
    pub fn reborrow(&mut self) -> StrokeCover<'_> {
        StrokeCover {
            buf: self.buf,
            law: self.law,
        }
    }
}

/// How much of the stroke's paint this dab adds at one texel, given the texel's coverage so far `m`,
/// the dab's silhouette weight `w`, its Grain weight `g`, the dab's opacity `coverage`
/// (stroke coverage × Flow × Strength) and whether the film's screen-space AA is live.
/// `None` = this dab adds nothing here (the caller skips the texel).
///
/// `BuildUp` is the arithmetic that shipped, moved here unchanged — including the two branches and
/// the `1e-4` guards — so the pigment path stays byte-identical.
#[must_use]
pub(crate) fn cover_add(
    law: StrokeCoverLaw,
    m: f32,
    w: f32,
    g: f32,
    coverage: f32,
    film_aa: bool,
) -> Option<f32> {
    match law {
        StrokeCoverLaw::BuildUp if film_aa => {
            // The film's AA rim caps the texel at its fractional AREA while the per-dab opacity
            // still builds WITHIN it (BUGS #16).
            let cap = (w * coverage).min(1.0);
            if m >= cap {
                return None; // the film's area is fully laid here
            }
            Some((w * g * coverage) * (1.0 - m / cap.max(1e-4)))
        }
        StrokeCoverLaw::BuildUp => {
            let cap = (g * coverage).min(1.0);
            if m >= cap {
                return None; // already at this texel's weighted cap
            }
            Some(w * (cap - m))
        }
        StrokeCoverLaw::Envelope => {
            // Krita Wash: the dab's own weight is the target; keep the deepest one the stroke has
            // laid here. Idempotent under re-crossing ⇒ the shoulder cannot converge to the rim.
            // (The film AA has no branch of its own: the AA is an impasto-body concern and the film
            // is Paint-mode-only, while this law is asked for by the Mask brush, which lays no body.
            // Should a body-laying medium ever want the envelope, the AA rim's area cap belongs in
            // `w` before it gets here — never as a second opinion about the target.)
            let target = (w * g * coverage).min(1.0);
            if target <= m {
                return None; // a previous dab already laid at least this much
            }
            Some(target - m)
        }
    }
}

#[cfg(test)]
#[path = "stroke_cover_tests.rs"]
mod tests;
