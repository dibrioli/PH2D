//! Axes as graph inputs (ADR-0066 §2.4): a [`VariableFontAxisCurve`] drives one
//! font axis from an [`AnimationCurveSampler`], exposing it as an
//! [`AttributeEvaluator`] the animation/motion graph samples. So
//! `motion-wave → variable-font.weight` or `expr "sin(t)*50+400" → weight` work
//! with no per-frame recompile (axis change is a UBO update, §2.3).

use ph2d_vector_traits::{AnimValue, AnimationCurveSampler, AttributeEvaluator};

use crate::axis::AxisTag;

/// Drives a single font axis from an animation curve, clamped to the axis's
/// `[min, max]` range. Implements [`AttributeEvaluator`] so it plugs into the
/// graph exactly like any other animated attribute.
pub struct VariableFontAxisCurve {
    /// The axis this curve drives.
    pub tag: AxisTag,
    min: f32,
    max: f32,
    curve: Box<dyn AnimationCurveSampler>,
}

impl VariableFontAxisCurve {
    /// Drive `tag` (clamped to `[min, max]`) from `curve`.
    pub fn new(tag: AxisTag, min: f32, max: f32, curve: Box<dyn AnimationCurveSampler>) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            tag,
            min,
            max,
            curve,
        }
    }

    /// The clamped axis value at time `t`. A non-`Float` curve sample (a
    /// misconfigured graph) falls back to the axis minimum rather than panicking.
    pub fn axis_value(&self, t: f64) -> f32 {
        let raw = match self.curve.at(t) {
            AnimValue::Float(v) => v,
            _ => self.min,
        };
        raw.clamp(self.min, self.max)
    }
}

impl AttributeEvaluator for VariableFontAxisCurve {
    fn sample(&self, t: f64) -> AnimValue {
        AnimValue::Float(self.axis_value(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A linear ramp `value = lo + (hi-lo)·t` over `t ∈ [0,1]`, extrapolating
    /// outside — used to exercise both the smooth band and the clamp.
    struct LinearRamp {
        lo: f32,
        hi: f32,
    }
    impl AnimationCurveSampler for LinearRamp {
        fn at(&self, t: f64) -> AnimValue {
            AnimValue::Float(self.lo + (self.hi - self.lo) * t as f32)
        }
    }

    fn weight_curve(lo: f32, hi: f32) -> VariableFontAxisCurve {
        VariableFontAxisCurve::new(
            AxisTag::WEIGHT,
            100.0,
            900.0,
            Box::new(LinearRamp { lo, hi }),
        )
    }

    #[test]
    fn interpolation_is_smooth_and_in_range() {
        // Gate `variable_font_axis_interpolation_smooth`: across the timeline the
        // axis value is continuous (small Δt → small Δvalue), monotone for a
        // monotone curve, and never leaves [min, max].
        let c = weight_curve(100.0, 900.0);
        let n = 256;
        let mut prev = c.axis_value(0.0);
        let lipschitz = (900.0 - 100.0) / n as f32 * 1.5; // per-step bound + slack
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let v = c.axis_value(t);
            assert!((100.0..=900.0).contains(&v), "t={t}: {v} out of range");
            assert!(v >= prev - 1e-4, "t={t}: not monotone ({v} < {prev})");
            assert!((v - prev).abs() <= lipschitz, "t={t}: discontinuous jump");
            prev = v;
        }
        assert!((c.axis_value(0.0) - 100.0).abs() < 1e-3);
        assert!((c.axis_value(1.0) - 900.0).abs() < 1e-3);
    }

    #[test]
    fn out_of_range_curve_is_clamped() {
        // A curve overshooting the axis range pins, never errors (animation must
        // not fail because a driver pushed past the design limits).
        let c = weight_curve(-500.0, 2000.0);
        assert_eq!(c.axis_value(0.0), 100.0, "undershoot pins to min");
        assert_eq!(c.axis_value(1.0), 900.0, "overshoot pins to max");
    }

    #[test]
    fn samples_as_float_attr() {
        let c = weight_curve(100.0, 900.0);
        assert_eq!(c.sample(0.5), AnimValue::Float(500.0));
    }
}
