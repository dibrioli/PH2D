//! Transcendental-free `(cos, sin)` for the wave surface — the same corrected parabolic
//! sine the orbit/oscillator/wind use (Capens/devmaster). Phase is in **cycles**
//! (period 1); ~0.09% off true trig using only multiply/abs, so the sea is
//! **deterministic** (HR-5). Self-contained per node crate (drop-crate isolation).

fn frac(p: f32) -> f32 {
    p - p.floor()
}

/// The corrected parabolic sine at `phase` cycles, in `[-1, 1]`.
fn sin_cycles(phase: f32) -> f32 {
    let f = frac(phase);
    let p = if f < 0.5 {
        let u = f * 2.0;
        4.0 * u * (1.0 - u)
    } else {
        let u = (f - 0.5) * 2.0;
        -4.0 * u * (1.0 - u)
    };
    const Q: f32 = 0.225;
    Q * (p * p.abs() - p) + p
}

/// `(cos, sin)` of `phase` cycles. `cos(x) = sin(x + ¼ cycle)`.
pub(crate) fn cos_sin_cycles(phase: f32) -> (f32, f32) {
    (sin_cycles(phase + 0.25), sin_cycles(phase))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_match_true_trig() {
        for (ph, (c, s)) in [
            (0.0, (1.0, 0.0)),
            (0.25, (0.0, 1.0)),
            (0.5, (-1.0, 0.0)),
            (0.75, (0.0, -1.0)),
        ] {
            let (ac, as_) = cos_sin_cycles(ph);
            assert!((ac - c).abs() < 1e-6, "cos at {ph}");
            assert!((as_ - s).abs() < 1e-6, "sin at {ph}");
        }
    }

    /// The pair is a *derivative* pair: `cos` is where `sin` is steepest and flat where
    /// `sin` peaks. The wave's surface slope is read from the `cos`, so a swapped return
    /// would tilt every float the wrong way — and only this relation catches that.
    ///
    /// The bound is **measured, not slack**: the parabolic sine is ~0.09% off in VALUE but
    /// its *derivative* is looser — the worst point of the cycle sits `0.0812` away from
    /// the true `2π·cos`, i.e. **1.29%** of the peak slope. That is the approximation's
    /// error, so that is what the gate allows (`0.085`) — and a swapped or sign-flipped
    /// pair misses by `2π`, seventy times more than this admits.
    #[test]
    fn cos_is_the_slope_of_sin() {
        let mut worst = 0.0f32;
        for i in 0..400 {
            let ph = i as f32 / 400.0;
            let h = 1e-3;
            let (c, _) = cos_sin_cycles(ph);
            let numeric = (cos_sin_cycles(ph + h).1 - cos_sin_cycles(ph - h).1) / (2.0 * h);
            // d/dphase sin(2π·phase) = 2π·cos(2π·phase)
            let expected = std::f32::consts::TAU * c;
            worst = worst.max((numeric - expected).abs());
        }
        assert!(
            worst < 0.085,
            "the parabolic sine's slope drifted {worst} from its own cosine (measured \
             ceiling: 0.0812)"
        );
    }
}
