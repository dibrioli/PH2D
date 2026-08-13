//! Transcendental-free `(cos, sin)` for the field's **rotation** — the same
//! corrected parabolic sine the oscillator/orbit/bend use (Capens/devmaster).
//! Angle is in **cycles** (period 1), so a rotation in degrees enters as
//! `deg / 360`. ~0.09% off true trig using only multiply/abs/floor, so the mask
//! is **deterministic** (HR-5) and the CPU↔GPU kernels share the SAME polynomial
//! (parity within ε). Copied per-crate (leaf drop-crate convention — the shared
//! thing is the algorithm, not a symbol). Endpoints are exact: `rotation = 0`
//! gives `(1, 0)` to the bit, so an un-rotated field is byte-identical to before.

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

    #[test]
    fn rotation_zero_is_exactly_identity() {
        // The byte-identity claim: an un-rotated field must be unchanged, so the
        // basis at phase 0 is EXACTLY (1, 0) — not merely close.
        assert_eq!(cos_sin_cycles(0.0), (1.0, 0.0));
    }

    #[test]
    fn stays_near_unit_circle() {
        for k in 0..64 {
            let ph = k as f32 / 64.0;
            let (c, s) = cos_sin_cycles(ph);
            assert!((c * c + s * s - 1.0).abs() < 0.02, "radius² at {ph}");
        }
    }
}
