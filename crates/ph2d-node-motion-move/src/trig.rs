//! Transcendental-free `(cos, sin)` for the LOCAL-space offset — the same
//! corrected parabolic sine the oscillator and the orbit use
//! (Capens/devmaster). Angle is in **cycles** (period 1). ~0.09% off true trig
//! using only multiply/abs, so the offset is **deterministic** (HR-5).
//!
//! ⚠️ **A não-ortonormalidade (`c² + s² ≠ 1`) não acumula aqui, e o motivo é
//! estrutural:** cada quadro roda o `(dx, dy)` AUTORADO pelo `rot` que chega, e
//! escreve o resultado em `P` — não há um estado que a aproximação realimente.
//! O erro é um deslocamento de ~0,09% do comprimento do vetor, uma vez.
//!
//! ⚠️ **Os quatro quartos de volta são EXATOS** (`0.25 → (0, 1)` ao bit, ver o
//! gate `anchors_match_true_trig`), e é isso que faz `rot = 0` devolver
//! `(dx, dy)` sem um épsilon — a igualdade de que o kernel do device depende
//! para casar com o caminho literal da CPU quando a coluna `rot` está ausente.

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
        // 0 cycles → (1, 0); ¼ → (0, 1); ½ → (-1, 0); ¾ → (0, -1).
        let approx = |ph: f32| cos_sin_cycles(ph);
        for (ph, (c, s)) in [
            (0.0, (1.0, 0.0)),
            (0.25, (0.0, 1.0)),
            (0.5, (-1.0, 0.0)),
            (0.75, (0.0, -1.0)),
        ] {
            let (ac, as_) = approx(ph);
            assert!((ac - c).abs() < 1e-6, "cos at {ph}");
            assert!((as_ - s).abs() < 1e-6, "sin at {ph}");
        }
    }

    #[test]
    fn stays_near_unit_circle() {
        // cos²+sin² ≈ 1 (radius stable within the approximation) at many angles.
        for k in 0..64 {
            let ph = k as f32 / 64.0;
            let (c, s) = cos_sin_cycles(ph);
            let r2 = c * c + s * s;
            assert!((r2 - 1.0).abs() < 0.02, "radius² = {r2} at {ph}");
        }
    }

    #[test]
    fn is_deterministic() {
        assert_eq!(cos_sin_cycles(0.37), cos_sin_cycles(0.37));
    }
}
