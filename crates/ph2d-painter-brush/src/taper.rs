//! **Taper** — the stroke thins at its ends because it is near an end, not because a pen pressed
//! lightly (Procreate's *Touch Taper*; Enio 2026-08-08: *"como Procreate faz com sua ferramenta de
//! Taper (sem pressão da caneta)"*).
//!
//! ## What it is
//!
//! Procreate's Brush Studio splits the idea in two: **Pressure Taper** (lengthen the taper a pen's
//! pressure already produces) and **Touch Taper** (the same shaping for a finger, which reports no
//! pressure). On a mouse the first does not exist — every sample arrives at pressure `1.0` — so the
//! second *is* the taper, and it is the only way a mouse draws a calligraphic line at all.
//!
//! The Procreate model, and what this ports:
//!
//! * two lengths, one per end, authored on a stroke-shaped widget with a handle on each side;
//! * a **Tip** size, "sharp" at the low end and "blunt" at the high end — how thin the very tip gets;
//! * an **Opacity** amount, so the taper can fade as well as narrow.
//!
//! ## The law
//!
//! A dab knows two distances: `s`, the arc travelled from the pen-down, and `e`, the arc still to
//! come. Each end contributes a width multiplier that is `tip` at the very tip, `1` once past the
//! taper length, and a **smoothstep** in between — C¹ at both ends, so the taper joins the full-width
//! body with no crease and the tip lands with no corner. No transcendental (HR-5).
//!
//! The two contributions combine by **`min`, never by product**. On a stroke shorter than
//! `start + end` the two windows overlap, and a product would thin it *twice* — a short flick would
//! very nearly vanish, which reads as the brush being broken rather than as a taper. `min` says the
//! honest thing: a dab is as wide as the *tightest* end allows.
//!
//! ## Why the length is in DIAMETERS
//!
//! A taper measured in pixels reads completely differently at two brush sizes — 60 px of taper is a
//! flourish on a 6 px liner and invisible on a 120 px wash. Brush-relative is what makes one number
//! mean one thing at every size, and it is what this crate already does for
//! [`crate::spec::BrushSpec::spacing`], for [`crate::spec::BrushSpec::jitter`] and for the heading's
//! smoothing length.
//!
//! ⚠️ **A fraction of the STROKE would have been the other obvious choice, and it cannot work here:**
//! a percentage needs the total length, and a freehand stroke does not have one until the pen lifts —
//! it would make even the *start* taper unknowable while the artist draws. See [`crate::stroke`] for
//! how the END is resolved for each family of stroke method.

/// Taper lengths are authored in **diameters**; this is the top of the slider.
///
/// Eight is where a 40 px brush tapers over 320 px — already longer than anything that reads as "the
/// brush", and far past any taper in the reference app. It costs no latency at any value: nothing is
/// ever held back ([`mod@crate::stroke::ends`]).
pub const MAX_TAPER_DIAMETERS: f32 = 8.0;

/// The stroke-end shaping of a brush (Procreate *Touch Taper*). All-zero is off, and an off taper is
/// byte-identical to no taper at all — the multipliers are exactly `1.0` and the dab is untouched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Taper {
    /// Taper length at the **start** of the stroke, in diameters (`0` = off).
    pub start: f32,
    /// Taper length at the **end** of the stroke, in diameters (`0` = off).
    pub end: f32,
    /// Width at the very **start** tip as a fraction of full width: `0` = a sharp point, `1` = blunt
    /// (no narrowing at all, so the taper is only whatever [`Self::opacity`] fades). Procreate's
    /// "Tip", whose own slider runs sharp → blunt.
    pub tip_start: f32,
    /// Width at the very **end** tip. Ignored while [`Self::link_tips`] is set — then both ends wear
    /// [`Self::tip_start`].
    pub tip_end: f32,
    /// **Link tip sizes**: one tip size drives both ends. On by default — two ends of one stroke
    /// wanting different points is the special case, not the rule.
    pub link_tips: bool,
    /// How much the taper also **fades**, `0..1`. `0` = the taper is purely a width (the tip is thin
    /// but as opaque as the body); `1` = coverage narrows exactly like the width, so the tip
    /// dissolves. Procreate's taper Opacity.
    pub opacity: f32,
}

impl Taper {
    /// The **neutral** taper: off, and byte-identical to a brush that never heard of one.
    ///
    /// A `const` because the panel's pre-publish fallback brush is a `const` too, and a second neutral
    /// spelled out there would be the version that quietly stops matching this one.
    pub const NEUTRAL: Self = Self {
        start: 0.0,
        end: 0.0,
        // A tapered end is a POINT unless the artist asks otherwise: this is the sharp end of
        // Procreate's own Tip slider, and the reason the feature is worth having on a mouse.
        tip_start: 0.0,
        tip_end: 0.0,
        link_tips: true,
        opacity: 0.0,
    };
}

impl Default for Taper {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl Taper {
    /// Whether anything about this taper can change a dab. A brush whose taper answers `false` never
    /// enters the taper path at all, which is what keeps a default brush byte-identical.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start > 0.0 || self.end > 0.0
    }

    /// The tip width of the END, honouring [`Self::link_tips`] — the one door for the question, asked
    /// by the width law here and by the panel that draws the preview.
    #[must_use]
    pub fn effective_tip_end(&self) -> f32 {
        if self.link_tips {
            self.tip_start
        } else {
            self.tip_end
        }
    }

    /// Taper length at the start in **pixels**, for a brush of `diameter_px`.
    #[must_use]
    pub fn start_px(&self, diameter_px: f32) -> f32 {
        self.start.clamp(0.0, MAX_TAPER_DIAMETERS) * diameter_px.max(0.0)
    }

    /// Taper length at the end in **pixels**, for a brush of `diameter_px`.
    #[must_use]
    pub fn end_px(&self, diameter_px: f32) -> f32 {
        self.end.clamp(0.0, MAX_TAPER_DIAMETERS) * diameter_px.max(0.0)
    }

    /// The **width multiplier** for a dab that is `from_start` px along the stroke and `to_end` px from
    /// its far end, on a brush of `diameter_px`.
    ///
    /// Pass `f32::INFINITY` for `to_end` when the end is not yet known — the end term is then exactly
    /// `1.0` and only the start taper applies, which is the correct answer for a stroke still being
    /// drawn (see [`crate::stroke`]).
    #[must_use]
    pub fn width(&self, from_start: f32, to_end: f32, diameter_px: f32) -> f32 {
        let a = ramp(from_start, self.start_px(diameter_px), self.tip_start);
        let b = ramp(to_end, self.end_px(diameter_px), self.effective_tip_end());
        // `min`, not `a * b` — see the module docs: on a short stroke the two windows overlap and a
        // product thins it twice.
        a.min(b)
    }

    /// The **coverage multiplier** that goes with a width multiplier `w`. `opacity = 0` leaves the dab
    /// as opaque as the body; `opacity = 1` fades it exactly as fast as it narrows.
    #[must_use]
    pub fn coverage_mult(&self, w: f32) -> f32 {
        let k = self.opacity.clamp(0.0, 1.0);
        (1.0 + k * (w - 1.0)).clamp(0.0, 1.0)
    }
}

/// One end's width multiplier: `tip` at distance `0`, `1.0` from `len` on, smoothstep between.
///
/// `len <= 0` (that end has no taper) short-circuits to exactly `1.0`, so an off end costs one compare
/// and cannot perturb a single bit.
fn ramp(d: f32, len: f32, tip: f32) -> f32 {
    // Written as a POSITIVE pair of comparisons on purpose: everything that is not definitely inside a
    // real taper window — no window (`len <= 0`), past it, an infinite `to_end` (the far end is not known
    // yet), a NaN — falls through to full width. Negating the comparisons instead would send NaN down the
    // ramp, and the one thing worse than a taper that ignores a bad number is a taper that trusts it.
    if len > 0.0 && d < len {
        let t = (d.max(0.0) / len).clamp(0.0, 1.0);
        let s = t * t * (3.0 - 2.0 * t); // smoothstep: C¹ at both ends, no transcendental (HR-5)
        let tip = tip.clamp(0.0, 1.0);
        tip + (1.0 - tip) * s
    } else {
        1.0
    }
}

/// Scale a dab in place by a taper width `w`, keeping the radius inside the engine's own bounds.
///
/// The single place a taper ever touches a [`crate::Dab`], and it is applied at EMISSION — the instant
/// the dab is made, at the position the pointer put it. Nothing is buffered waiting to learn its
/// distance-to-the-end; what the far end costs is answered by geometry instead
/// ([`mod@crate::stroke::ends`]).
///
/// ⚠️ [`crate::Dab::stroke_radius_px`] is deliberately NOT scaled. It is the stroke's *nominal* radius
/// and it is stroke-constant by contract — the Shape **Flow** mapping divides its along-the-stroke
/// coordinate by it, so a per-dab divisor re-phases the whole accumulated pattern (the bug that cost
/// `0.42` of a tile when it rode the live radius). A taper is exactly the kind of per-dab breathing
/// that contract exists to keep out.
pub fn scale_dab(dab: &mut crate::Dab, taper: &Taper, w: f32) {
    if w >= 1.0 {
        return;
    }
    dab.radius_px = (dab.radius_px * w.max(0.0)).clamp(0.5, crate::spec::MAX_BRUSH_RADIUS_PX);
    dab.coverage = (dab.coverage * taper.coverage_mult(w)).clamp(0.0, 1.0);
}
