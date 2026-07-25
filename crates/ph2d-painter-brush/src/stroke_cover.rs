//! The **per-stroke coverage buffer** — the Accumulate-OFF cap — with its arithmetic written ONCE.
//!
//! A stroke lays its paint through many overlapping dabs, so *"how much of this stroke's paint is at this
//! texel?"* has to be answered by a buffer that outlives the dab. The per-pixel stamp and the ramped
//! stamp both consult it, and they must not hold two opinions about the rule — hence this module.
//!
//! ## The law (and why it is the ONLY one)
//!
//! Each dab moves the texel a *fraction* of the way to a ceiling (`m += w·(cap − m)`), so the dab profile
//! acts as a **RATE**: over `n` overlapping dabs the texel reaches `cap·(1 − Π(1 − w_k))`. That is the
//! *"paint over it again to deepen it"* pigment wants, and it is the law GIMP ships for its
//! non-incremental paint core (`gimp_gegl_combine_mask_weird`'s non-stipple branch:
//! `if (opacity > dest) dest += (opacity − dest) * mask * opacity`).
//!
//! ⚠️ **A second law was built and REPROVED on screen (2026-07-25).** Krita's Wash mode / *Alpha Darken*
//! makes the dab profile the **target**, kept by `max` — which stops a scrubbed stroke from ever
//! hardening its feather, and was implemented here for the Mask brush. The measurement that killed it:
//! without the product's saturation the **per-dab modulation of the profile stays visible**, and a stroke
//! comes out in BEADS along its shoulder (rendered; doc 25 §13.10). The Enio's standing order after that
//! smoke is *"a máscara deve pintar exatamente como o brush digital normal"* — one law, this one, for
//! every medium that deposits through the dab pipeline. Do not re-introduce a per-medium law without a
//! render-and-look that shows the beads are gone.
//!
//! ## The `add` algebra — why the deposit lands on the pre-stroke pixels
//!
//! The caller blends the dab at `a = add / (1 − m)`. That is not a fudge: writing `A_n` for the total
//! fraction of the stroke's paint laid after `n` dabs, source-over gives `1 − A_n = Π(1 − a_k)`, and
//! substituting `a_k = (m_k − m_{k−1}) / (1 − m_{k−1})` telescopes to `1 − A_n = 1 − m_n` **exactly**. So
//! the canvas always holds `pre_stroke·(1 − m) + colour·m`: the stroke is applied to the state it started
//! from, which is the other half of GIMP's model (it applies constant-mode paint from the undo snapshot,
//! never from the live drawable) — and we get it without keeping a copy of the canvas.

/// How much of the stroke's paint this dab adds at one texel, given the texel's coverage so far `m`, the
/// dab's silhouette weight `w`, its Grain weight `g`, the dab's opacity `coverage` (stroke coverage ×
/// Flow × Strength) and whether the film's screen-space AA is live. `None` = this dab adds nothing here
/// (the caller skips the texel).
///
/// Moved here from `bands.rs` unchanged — including both branches and the `1e-4` guards — so there is one
/// copy of the rule; the gate next door compares it term by term against the expressions that shipped.
#[must_use]
pub(crate) fn cover_add(m: f32, w: f32, g: f32, coverage: f32, film_aa: bool) -> Option<f32> {
    if film_aa {
        // The film's AA rim caps the texel at its fractional AREA while the per-dab opacity still
        // builds WITHIN it (BUGS #16).
        let cap = (w * coverage).min(1.0);
        if m >= cap {
            return None; // the film's area is fully laid here
        }
        Some((w * g * coverage) * (1.0 - m / cap.max(1e-4)))
    } else {
        let cap = (g * coverage).min(1.0);
        if m >= cap {
            return None; // already at this texel's weighted cap
        }
        Some(w * (cap - m))
    }
}

#[cfg(test)]
#[path = "stroke_cover_tests.rs"]
mod tests;
