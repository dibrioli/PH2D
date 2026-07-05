//! [`SmoothGain`] — the anti-zipper primitive.
//!
//! A raw gain change per block is a step discontinuity → an audible click
//! ("zipper noise"). `SmoothGain` ramps the applied gain toward its target one
//! sample at a time (one-pole approach), so control changes never click.

/// Samples over which a gain change nominally settles (~5 ms @ 48 kHz).
const DEFAULT_RAMP_SAMPLES: f32 = 256.0;

/// A gain value that glides toward its target instead of jumping. Linear ramp
/// with a fixed per-sample step, so it settles in `ramp_samples` samples.
#[derive(Copy, Clone, Debug)]
pub struct SmoothGain {
    current: f32,
    target: f32,
    step: f32,
    ramp_samples: f32,
}

impl SmoothGain {
    /// Start settled at `value` (current == target, no ramp pending).
    pub fn immediate(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            step: 0.0,
            ramp_samples: DEFAULT_RAMP_SAMPLES,
        }
    }

    /// Alias for [`SmoothGain::immediate`].
    pub fn new(initial: f32) -> Self {
        Self::immediate(initial)
    }

    /// Glide toward `target` over the ramp window from the current value.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
        self.step = (self.target - self.current) / self.ramp_samples;
    }

    /// Snap to `value` now (no ramp) — e.g. when (re)starting a voice.
    pub fn set_immediate(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
    }

    /// The current (possibly still-ramping) gain.
    pub fn current(&self) -> f32 {
        self.current
    }

    /// True once the ramp has reached its target.
    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() <= f32::EPSILON
    }

    /// Advance one sample and return the gain to apply.
    #[inline]
    pub fn tick(&mut self) -> f32 {
        let diff = self.target - self.current;
        if diff.abs() > f32::EPSILON {
            self.current += self.step;
            // Within one fixed step of the target → snap so it settles exactly.
            if (self.target - self.current).abs() <= self.step.abs() {
                self.current = self.target;
            }
        }
        self.current
    }
}

impl Default for SmoothGain {
    fn default() -> Self {
        Self::immediate(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immediate_does_not_ramp() {
        let mut g = SmoothGain::immediate(0.5);
        assert!(g.is_settled());
        assert_eq!(g.tick(), 0.5);
    }

    #[test]
    fn ramp_is_monotonic_and_settles() {
        let mut g = SmoothGain::immediate(0.0);
        g.set_target(1.0);
        let mut prev = g.current();
        for _ in 0..4096 {
            let v = g.tick();
            assert!(v >= prev - 1e-6, "gain ramp must be monotonic up");
            assert!(v <= 1.0 + 1e-6, "gain must not overshoot target");
            prev = v;
        }
        assert!(g.is_settled(), "ramp should settle within the window");
        assert!((g.current() - 1.0).abs() < 1e-4);
    }
}
