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
    /// Random per-dab position offset as a fraction of the radius, `0..1` (Blender "Jitter").
    pub jitter: f32,
    /// Paint colour, straight RGB in `[0, 1]` in the layer's native space.
    pub color: [f32; 3],
    /// The editable profile used when [`Self::falloff`] is [`Falloff::Custom`]
    /// (ignored otherwise). Kept inline so the spec stays `Copy`/alloc-free.
    pub custom_falloff: FalloffCurve,
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
