//! [`Compressor`] — a stereo-linked feed-forward compressor for per-bus dynamics.
//!
//! Standard design: a peak detector (linked across L/R), a gain computer that
//! turns the over-threshold excess down by `ratio`, and an attack/release
//! envelope so the gain change is smooth. HR-5: [`Compressor::process`] (the
//! audio-thread hot path) is mul/add/compare/divide only — the attack/release
//! **time → coefficient** `exp` is done control-side in the engine, and the
//! coefficients are passed in via [`Compressor::set_params`].

/// A stereo-linked feed-forward compressor. Bypassed when `off`.
#[derive(Clone, Copy)]
pub struct Compressor {
    on: bool,
    /// Level (linear, 0..1) above which gain reduction starts.
    threshold: f32,
    /// Compression ratio (`>= 1`; `1` = no compression).
    ratio: f32,
    /// Per-sample attack coefficient (gain moving *down* toward the target).
    attack: f32,
    /// Per-sample release coefficient (gain moving *up* toward unity).
    release: f32,
    /// Current smoothed gain (`1.0` = no reduction), carried across blocks.
    gain: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            on: false,
            threshold: 1.0,
            ratio: 1.0,
            attack: 0.5,
            release: 0.05,
            gain: 1.0,
        }
    }
}

impl Compressor {
    /// Configure: `on`, `threshold` (linear 0..1), `ratio` (>=1), and the
    /// pre-computed per-sample `attack` / `release` coefficients (0..1).
    pub fn set_params(&mut self, on: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        self.on = on;
        self.threshold = threshold.clamp(0.001, 1.0);
        self.ratio = ratio.max(1.0);
        self.attack = attack.clamp(0.0, 1.0);
        self.release = release.clamp(0.0, 1.0);
    }

    /// Gain computer: bring the over-threshold excess down by `ratio`. Below the
    /// threshold the target is unity (no reduction).
    #[inline]
    fn target_gain(&self, level: f32) -> f32 {
        if level > self.threshold {
            (self.threshold + (level - self.threshold) / self.ratio) / level
        } else {
            1.0
        }
    }

    /// Settle the envelope on `(l, r)` as if the compressor had already been
    /// running at that level. **Offline** callers prime on the first frame of a
    /// region: `gain` otherwise starts at unity, so the first sample escapes
    /// uncompressed — a startup transient that clicks at a selection edge and
    /// defeats peak-preserving make-up (the output peak would still equal the
    /// input's, leaving no reduction to hand back).
    pub fn prime(&mut self, l: f32, r: f32) {
        if self.on {
            self.gain = self.target_gain(l.abs().max(r.abs()));
        }
    }

    /// Compress one stereo sample (linked detection). Passes through when off.
    #[inline]
    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        if !self.on {
            return (l, r);
        }
        let level = l.abs().max(r.abs());
        let target = self.target_gain(level);
        // Attack when clamping down (target below current gain), release when
        // easing back up — the classic asymmetric envelope.
        let coeff = if target < self.gain {
            self.attack
        } else {
            self.release
        };
        self.gain += (target - self.gain) * coeff;
        (l * self.gain, r * self.gain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_passes_through() {
        let mut c = Compressor::default();
        c.set_params(true, 0.5, 4.0, 0.5, 0.05);
        // A steady 0.2 signal (below the 0.5 threshold) is untouched.
        for _ in 0..1000 {
            let (l, r) = c.process(0.2, 0.2);
            assert!((l - 0.2).abs() < 1e-6 && (r - 0.2).abs() < 1e-6);
        }
    }

    #[test]
    fn above_threshold_reduces_gain_toward_the_computed_target() {
        let mut c = Compressor::default();
        // Threshold 0.5, ratio 4, fast attack so it settles within the loop.
        c.set_params(true, 0.5, 4.0, 0.5, 0.05);
        let mut out = 1.0f32;
        for _ in 0..5000 {
            out = c.process(0.9, 0.9).0;
        }
        // Settled output ≈ threshold + (0.9 - 0.5)/4 = 0.6, well under the input.
        assert!(
            (out - 0.6).abs() < 0.02,
            "a 0.9 signal at 4:1 over 0.5 must settle near 0.6, got {out}"
        );
        assert!(out < 0.9, "compression must reduce the loud level");
    }

    #[test]
    fn off_is_a_true_bypass() {
        let mut c = Compressor::default();
        c.set_params(false, 0.1, 8.0, 0.5, 0.05); // heavy settings but off
        for &x in &[0.2, 0.9, -0.7, 1.0] {
            let (l, r) = c.process(x, x);
            assert_eq!((l, r), (x, x), "an off compressor must pass through");
        }
    }
}
