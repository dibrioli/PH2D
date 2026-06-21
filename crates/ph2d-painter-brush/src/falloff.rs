//! Radial distance falloff for the brush dab.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `blenkernel/intern/brush.cc::BKE_brush_curve_strength` (the `eBrushCurvePreset` shapes) and
//! `editors/sculpt_paint/mesh/paint_image_2d_curve_mask.cc` (how the curve is sampled into a
//! per-pixel mask). Blender evaluates each preset on `p = 1 - distance/radius` (so `p = 1` at the
//! dab centre, `p = 0` at the rim); this port keeps that convention.

/// Falloff profile. Shapes mirror Blender's `eBrushCurvePreset` (the single reference for this
/// engine). `Custom` (an editable curve) is deferred to the UI phase and not represented here yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Falloff {
    /// `3p² − 2p³` (smoothstep). Blender default soft brush.
    #[default]
    Smooth = 0,
    /// `p³(p(6p − 15) + 10)` (smootherstep). Softer than [`Falloff::Smooth`].
    Smoother = 1,
    /// `√(2p − p²)` — rounded shoulder, like a hemisphere.
    Sphere = 2,
    /// `√p` — soft rim, full centre.
    Root = 3,
    /// `p²` — concentrated toward the centre.
    Sharp = 4,
    /// `p` — linear cone.
    Linear = 5,
    /// `p⁴` — very concentrated (Blender `POW4`).
    Pow4 = 6,
    /// `p(2 − p)` — inverse-square shoulder.
    InvSquare = 7,
    /// `1` everywhere inside the radius — hard disk.
    Constant = 8,
}

/// Number of falloff presets (matches Blender's `eBrushCurvePreset`, minus the
/// editable `Custom` curve which is a later UI phase).
pub const MAX_FALLOFF: u8 = 9;

impl Falloff {
    /// Every preset, in Blender's "Falloff Curve Preset" display order. The index
    /// is the stable `u8` discriminant.
    pub const ALL: [Falloff; MAX_FALLOFF as usize] = [
        Self::Smooth,
        Self::Smoother,
        Self::Sphere,
        Self::Root,
        Self::Sharp,
        Self::Linear,
        Self::Pow4,
        Self::InvSquare,
        Self::Constant,
    ];

    /// The stable wire discriminant (`0..MAX_FALLOFF`).
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Preset for a wire discriminant; out-of-range falls back to [`Self::Smooth`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        Self::ALL.get(v as usize).copied().unwrap_or(Self::Smooth)
    }

    /// Display name for the Falloff section dropdown (Blender's preset labels;
    /// `Pow4` is shown as Blender's "Sharper"). English UI per HR-15.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Smoother => "Smoother",
            Self::Sphere => "Sphere",
            Self::Root => "Root",
            Self::Sharp => "Sharp",
            Self::Linear => "Linear",
            Self::Pow4 => "Sharper",
            Self::InvSquare => "Inverse Square",
            Self::Constant => "Constant",
        }
    }

    /// Weight at normalized distance `t = distance / radius`.
    ///
    /// Returns `1.0` at the centre (`t = 0`), decreases monotonically, and is exactly `0.0` at or
    /// beyond the rim (`t >= 1`). Negative `t` is clamped to `0`.
    #[must_use]
    pub fn weight(self, t: f32) -> f32 {
        if t >= 1.0 {
            return 0.0;
        }
        // Blender convention: evaluate the preset on `p = 1 - t`.
        let p = 1.0 - t.max(0.0);
        match self {
            Falloff::Sharp => p * p,
            Falloff::Smooth => 3.0 * p * p - 2.0 * p * p * p,
            Falloff::Smoother => {
                let p3 = p * p * p;
                p3 * (p * (p * 6.0 - 15.0) + 10.0)
            }
            Falloff::Root => p.sqrt(),
            Falloff::Linear => p,
            Falloff::Constant => 1.0,
            Falloff::Sphere => (2.0 * p - p * p).sqrt(),
            Falloff::Pow4 => p * p * p * p,
            Falloff::InvSquare => p * (2.0 - p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Falloff, MAX_FALLOFF};

    const ALL: [Falloff; 9] = Falloff::ALL;

    #[test]
    fn wire_round_trips_and_names_unique() {
        assert_eq!(Falloff::ALL.len(), MAX_FALLOFF as usize);
        let mut names = Vec::new();
        for (i, &f) in Falloff::ALL.iter().enumerate() {
            assert_eq!(f.to_u8() as usize, i, "{f:?} discriminant != index");
            assert_eq!(Falloff::from_u8(i as u8), f);
            assert!(!f.name().is_empty());
            names.push(f.name());
        }
        assert_eq!(
            Falloff::from_u8(MAX_FALLOFF),
            Falloff::Smooth,
            "OOR → Smooth"
        );
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate falloff name");
    }

    #[test]
    fn endpoints_centre_full_rim_zero() {
        for f in ALL {
            assert!((f.weight(0.0) - 1.0).abs() < 1e-6, "{f:?} centre != 1");
            assert_eq!(f.weight(1.0), 0.0, "{f:?} rim != 0");
            assert_eq!(f.weight(1.5), 0.0, "{f:?} past rim != 0");
        }
    }

    #[test]
    fn monotonic_non_increasing_inside() {
        for f in ALL {
            let mut prev = f.weight(0.0);
            let mut t = 0.0;
            while t <= 1.0 {
                let w = f.weight(t);
                assert!(
                    w <= prev + 1e-6,
                    "{f:?} not monotonic at t={t}: {w} > {prev}"
                );
                assert!(
                    (0.0..=1.0).contains(&w),
                    "{f:?} weight {w} out of [0,1] at t={t}"
                );
                prev = w;
                t += 0.01;
            }
        }
    }

    #[test]
    fn known_values_match_blender_shapes() {
        // p = 1 - t. At t = 0.5 → p = 0.5.
        assert!((Falloff::Linear.weight(0.5) - 0.5).abs() < 1e-6);
        assert!((Falloff::Sharp.weight(0.5) - 0.25).abs() < 1e-6); // p²
        assert!((Falloff::Pow4.weight(0.5) - 0.0625).abs() < 1e-6); // p⁴
        assert!((Falloff::Constant.weight(0.99) - 1.0).abs() < 1e-6);
        // Smooth at t=0.5 is the smoothstep midpoint → 0.5.
        assert!((Falloff::Smooth.weight(0.5) - 0.5).abs() < 1e-6);
        // Sphere: √(1 - t²) at t=0.5 → √0.75.
        assert!((Falloff::Sphere.weight(0.5) - 0.75_f32.sqrt()).abs() < 1e-6);
    }
}
