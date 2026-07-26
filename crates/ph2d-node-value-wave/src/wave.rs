//! The **transcendental-free** waveform bank (HR-5), a leaf-local mirror of
//! `motion.oscillator`'s / `value.lfo`'s wave core (kept copied per drop-crate —
//! the shared vocabulary is the *shape*, not a shared symbol; this is the third
//! copy, and the `time → wave == lfo` device gate pins it to lfo's byte-for-byte).
//! `phase` is measured in *cycles* (unit period); the shapes are piecewise
//! polynomial. The "Sine" wave is a parabolic approximation with a 2nd-order
//! correction (Capens/devmaster) — ~0.09% off a true sine using only multiply +
//! abs — because a real `sin` is non-deterministic across platforms (HR-5).

/// The fractional part of `p` in `[0,1)` — IEEE `floor` is correctly-rounded and
/// deterministic (HR-5-safe, unlike `sin`).
fn frac(p: f32) -> f32 {
    p - p.floor()
}

/// A periodic waveform at `phase` (in cycles, period 1) — bipolar `[-1,1]` except
/// **Spike** (a unipolar `[0,1]` pulse). All shapes are piecewise polynomial →
/// transcendental-free (HR-5). Unknown / `0` is the parabolic sine-approximation.
/// Byte-for-byte identical to `value.lfo`'s copy (the shaper-dual relationship).
pub(crate) fn waveform(kind: i32, phase: f32) -> f32 {
    let f = frac(phase);
    match kind {
        1 => {
            // Triangle: 0 at 0, +1 at ¼, 0 at ½, −1 at ¾.
            if f < 0.25 {
                4.0 * f
            } else if f < 0.75 {
                2.0 - 4.0 * f
            } else {
                4.0 * f - 4.0
            }
        }
        2 => {
            // Square: +1 first half, −1 second.
            if f < 0.5 { 1.0 } else { -1.0 }
        }
        3 => 2.0 * f - 1.0, // Saw: −1 → +1 rising.
        4 => {
            // Spike: a narrow unipolar pulse at the cycle start (a periodic kick).
            const SPIKE_WIDTH: f32 = 0.08;
            if f < SPIKE_WIDTH { 1.0 } else { 0.0 }
        }
        _ => {
            // Parabolic sine-approximation: a +hump over [0,½), a −hump over
            // [½,1), each `±4u(1−u)` — continuous, 0 at 0/½, ±1 at ¼/¾.
            let p = if f < 0.5 {
                let u = f * 2.0;
                4.0 * u * (1.0 - u)
            } else {
                let u = (f - 0.5) * 2.0;
                -4.0 * u * (1.0 - u)
            };
            // 2nd-order correction (Capens/devmaster): the bare parabola is ~5.6%
            // off a true sine; `0.225·(p·|p|−p)+p` drops that to ~0.09% using only
            // multiply + abs (transcendental-free, HR-5). Endpoint/range-preserving.
            const Q: f32 = 0.225;
            Q * (p * p.abs() - p) + p
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The anchor points of every shape (the same pins `value.lfo`'s copy holds —
    /// this is how the leaf-local copy is kept honest without a shared symbol).
    #[test]
    fn every_shape_stays_in_range_and_repeats_per_cycle() {
        for kind in 0..=4 {
            for step in 0..40 {
                let p = step as f32 * 0.1;
                let v = waveform(kind, p);
                assert!((-1.0..=1.0).contains(&v), "wave {kind} at {p} = {v}");
                assert!(
                    (waveform(kind, p) - waveform(kind, p + 1.0)).abs() < 1e-5,
                    "wave {kind} periodic at {p}"
                );
            }
        }
        // Anchor points of the corrected sine approximation (preserved).
        assert_eq!(waveform(0, 0.0), 0.0);
        assert_eq!(waveform(0, 0.25), 1.0);
        assert_eq!(waveform(0, 0.75), -1.0);
        // Triangle / Square / Saw anchors.
        assert_eq!(waveform(1, 0.25), 1.0);
        assert_eq!(waveform(2, 0.0), 1.0);
        assert_eq!(waveform(2, 0.5), -1.0);
        assert_eq!(waveform(3, 0.0), -1.0);
        assert_eq!(waveform(3, 0.5), 0.0);
        // Spike: a narrow unipolar pulse — 1 at the cycle start, 0 through most.
        assert_eq!(waveform(4, 0.0), 1.0);
        assert_eq!(waveform(4, 0.5), 0.0);
    }
}
