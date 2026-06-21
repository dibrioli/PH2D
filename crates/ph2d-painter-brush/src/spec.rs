//! `BrushSpec` — the brush parameters.
//!
//! Clean-room model of the relevant fields of Blender's `Brush` (`makesdna/DNA_brush_types.h`):
//! radius, strength, the per-dab build-up (`flow`/`alpha`), spacing, blend mode, the distance
//! falloff curve, jitter, and the paint colour. Fields the texture painter does not need yet
//! (texture slots, stencil masks, projection options) are deliberately omitted — see
//! `docs/Painter/02_plano_de_implementacao.md` for what each phase adds.

use crate::blend::BrushBlend;
use crate::falloff::Falloff;
use crate::falloff_curve::FalloffCurve;
use crate::stroke_method::{JitterUnit, StrokeMethod};

/// Largest brush radius the engine will allocate a dab for, in pixels. Derived from the editor
/// overlay budget (HR-4): a 4096-px-radius dab would be an 8k² bbox — far past interactive. This
/// caps the bbox, not the artist's intent (the value is clamped, not rejected).
pub const MAX_BRUSH_RADIUS_PX: f32 = 4096.0;

/// Parameters of a single brush. Cheap to copy; the stroke engine reads it per dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSpec {
    /// Dab radius in image pixels (UI label "Radius"). Clamped to `[0.5, MAX_BRUSH_RADIUS_PX]`.
    pub radius_px: f32,
    /// Plateau before the falloff begins, `0..1`. `0` = pure falloff curve; `1` = hard disk.
    /// Mirrors Blender's brush `hardness` as a distance remap applied before [`Self::falloff`].
    pub hardness: f32,
    /// Maximum opacity a single stroke can build to, `0..1` (Blender "Strength"). Enforced as a
    /// per-stroke accumulation cap by the stroke engine (later phase), not by a single dab.
    pub strength: f32,
    /// Per-dab build-up, `0..1` (Blender "Alpha"/flow). Scales every dab's coverage.
    pub flow: f32,
    /// Distance between dabs as a fraction of the diameter (Blender "Spacing"; default `0.10`).
    pub spacing: f32,
    /// How the dab colour combines with the layer.
    pub blend: BrushBlend,
    /// Radial intensity profile.
    pub falloff: Falloff,
    /// Random per-dab position offset when [`Self::jitter_unit`] is [`JitterUnit::Brush`]: a
    /// fraction of the diameter, `0..1` (Blender `Brush.jitter`, default `0.0`,
    /// `DNA_brush_types.h:221`). Max radial offset ≈ `jitter × diameter`.
    pub jitter: f32,
    /// Paint colour, straight RGB in `[0, 1]` in the layer's native space.
    pub color: [f32; 3],
    /// The editable profile used when [`Self::falloff`] is [`Falloff::Custom`]
    /// (ignored otherwise). Kept inline so the spec stays `Copy`/alloc-free.
    pub custom_falloff: FalloffCurve,

    // ── Stroke panel (Blender `paint_stroke.cc` / `DNA_brush_types.h`) ──────────────
    /// How the pointer path becomes dabs (Blender `stroke_method`, default `Space`,
    /// `DNA_brush_types.h:213`).
    pub stroke_method: StrokeMethod,
    /// "Adjust Strength for Spacing" — normalise total deposited opacity so a densely-spaced
    /// stroke doesn't pile up to full opacity (Blender `BRUSH_SPACE_ATTEN`, default ON,
    /// `DNA_brush_types.h:206`). Only applies for spacing < 100% (see [`Self::space_overlap_factor`]).
    pub space_attenuation: bool,
    /// Dash "on" fraction of each dash period, `0..1` (Blender `dash_ratio`, default `1.0` = solid,
    /// `DNA_brush_types.h:275`).
    pub dash_ratio: f32,
    /// Dash period length in dab-slots (Blender `dash_samples`, default `20`,
    /// `DNA_brush_types.h:276`). `0` is treated as "no dash" (solid).
    pub dash_samples: u32,
    /// Unit for [`Self::jitter`] / [`Self::jitter_absolute_px`] (Blender `BRUSH_ABSOLUTE_JITTER`).
    pub jitter_unit: JitterUnit,
    /// Random per-dab offset in pixels when [`Self::jitter_unit`] is [`JitterUnit::View`]
    /// (Blender `jitter_absolute`, default `0`, `DNA_brush_types.h:223`). Max radial offset
    /// ≈ `2 × jitter_absolute_px`, independent of brush size.
    pub jitter_absolute_px: f32,
    /// Number of most-recent raw input samples box-averaged before processing, `>= 1`
    /// (Blender `input_samples`, default `1`, `DNA_brush_types.h:216`).
    pub input_samples: u32,
    /// Airbrush emission period in seconds (Blender `rate`, default `0.1` = 10 Hz,
    /// `DNA_brush_types.h:232`). Used only by [`StrokeMethod::Airbrush`]; the tool's tick drives it.
    pub airbrush_rate_s: f32,
}

impl Default for BrushSpec {
    /// A soft round black brush, matching Blender's default "TexDraw": smooth falloff, full
    /// strength/flow, 10% spacing.
    fn default() -> Self {
        Self {
            radius_px: 25.0,
            hardness: 0.0,
            strength: 1.0,
            flow: 1.0,
            spacing: 0.10,
            blend: BrushBlend::Mix,
            falloff: Falloff::Smooth,
            jitter: 0.0,
            color: [0.0, 0.0, 0.0],
            custom_falloff: FalloffCurve::default(),
            stroke_method: StrokeMethod::Space,
            space_attenuation: true,
            dash_ratio: 1.0,
            dash_samples: 20,
            jitter_unit: JitterUnit::Brush,
            jitter_absolute_px: 0.0,
            input_samples: 1,
            airbrush_rate_s: 0.1,
        }
    }
}

impl BrushSpec {
    /// Effective dab radius after clamping to the allocation cap.
    #[must_use]
    pub fn clamped_radius(&self) -> f32 {
        self.radius_px.clamp(0.5, MAX_BRUSH_RADIUS_PX)
    }

    /// Distance between dab centres in pixels, derived from spacing × diameter.
    /// At least one pixel so a stroke always advances.
    #[must_use]
    pub fn dab_spacing_px(&self) -> f32 {
        (self.spacing.max(0.01) * 2.0 * self.clamped_radius()).max(1.0)
    }

    /// Whether the dab at dash-slot `slot` is painted, given the dash pattern.
    ///
    /// Behavioural reference: `paint_stroke.cc::add_step` (`dash = (slot % dash_samples) /
    /// dash_samples`; skip when `dash > dash_ratio`). `dash_samples == 0` ⇒ no dash (always on).
    /// With the default `dash_ratio = 1.0` every slot is on (solid).
    #[must_use]
    pub fn dash_on(&self, slot: u32) -> bool {
        if self.dash_samples == 0 {
            return true;
        }
        let dash = (slot % self.dash_samples) as f32 / self.dash_samples as f32;
        dash <= self.dash_ratio.clamp(0.0, 1.0)
    }

    /// "Adjust Strength for Spacing" multiplier applied to each dab's coverage, in `(0, 1]`.
    ///
    /// Behavioural reference (clean-room): `paint_stroke.cc::paint_stroke_integrate_overlap` +
    /// `paint_stroke_overlapped_curve`. Models how many neighbouring dab falloff kernels stack at a
    /// given phase and returns `1 / max_phase(Σ kernels)` so a densely-spaced stroke is normalised to
    /// unit opacity instead of piling up. Returns `1.0` (no attenuation) when the flag is off or
    /// spacing ≥ 100% (Blender's exact gate). Uses this brush's own falloff, like Blender.
    #[must_use]
    pub fn space_overlap_factor(&self) -> f32 {
        // Blender stores spacing as an integer percent; this engine stores a 0..1 fraction.
        let spacing_pct = (self.spacing * 100.0).max(0.0);
        if !(self.space_attenuation && spacing_pct < 100.0) {
            return 1.0;
        }
        // Sample the overlap sum at M phases across one period; the factor cancels the peak.
        const M: usize = 10;
        let g = 1.0 / M as f32;
        let mut max = 0.0_f32;
        for i in 0..M {
            let o = self.overlapped_curve(i as f32 * g, spacing_pct).abs();
            if o > max {
                max = o;
            }
        }
        if max == 0.0 { 1.0 } else { 1.0 / max }
    }

    /// Sum of overlapping dab falloff kernels at phase `x` for a given `spacing_pct`
    /// (`paint_stroke_overlapped_curve`): `n = floor(100 / spacing_pct)` kernels spaced
    /// `h = spacing_pct / 50` apart, each evaluated through this brush's falloff.
    fn overlapped_curve(&self, x: f32, spacing_pct: f32) -> f32 {
        let clamped = spacing_pct.max(0.1);
        let n = (100.0 / clamped) as i32;
        let h = clamped / 50.0;
        let x0 = x - 1.0;
        let mut sum = 0.0;
        for i in 0..n {
            let xx = (x0 + i as f32 * h).abs();
            if xx < 1.0 {
                sum += self.falloff_weight(xx);
            }
        }
        sum
    }

    /// Falloff weight remapped by [`Self::hardness`]. `t = distance / radius`.
    ///
    /// Hardness pushes the falloff outward: for `t < hardness` the weight is full; the curve then
    /// runs over `[hardness, 1]`. `hardness >= 1` yields a hard disk. [`Falloff::Custom`] reads the
    /// editable [`Self::custom_falloff`] profile; every other preset uses its formula.
    #[must_use]
    pub fn falloff_weight(&self, t: f32) -> f32 {
        let h = self.hardness.clamp(0.0, 1.0);
        if h >= 1.0 {
            return if t < 1.0 { 1.0 } else { 0.0 };
        }
        let remapped = ((t - h) / (1.0 - h)).clamp(0.0, 1.0);
        match self.falloff {
            Falloff::Custom => self.custom_falloff.weight(remapped),
            preset => preset.weight(remapped),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let b = BrushSpec::default();
        assert_eq!(b.blend, BrushBlend::Mix);
        assert_eq!(b.falloff, Falloff::Smooth);
        assert!(b.dab_spacing_px() >= 1.0);
    }

    #[test]
    fn radius_clamped() {
        let b = BrushSpec {
            radius_px: 999_999.0,
            ..Default::default()
        };
        assert_eq!(b.clamped_radius(), MAX_BRUSH_RADIUS_PX);
        let b = BrushSpec {
            radius_px: 0.0,
            ..Default::default()
        };
        assert_eq!(b.clamped_radius(), 0.5);
    }

    #[test]
    fn hardness_full_is_hard_disk() {
        let b = BrushSpec {
            hardness: 1.0,
            ..Default::default()
        };
        assert_eq!(b.falloff_weight(0.0), 1.0);
        assert_eq!(b.falloff_weight(0.99), 1.0);
        assert_eq!(b.falloff_weight(1.0), 0.0);
    }

    #[test]
    fn hardness_plateau_then_falls() {
        let b = BrushSpec {
            hardness: 0.5,
            falloff: Falloff::Linear,
            ..Default::default()
        };
        assert_eq!(b.falloff_weight(0.5), 1.0); // inside plateau
        // At t=0.75, remapped = (0.75-0.5)/0.5 = 0.5 → linear weight 0.5.
        assert!((b.falloff_weight(0.75) - 0.5).abs() < 1e-6);
    }
}
