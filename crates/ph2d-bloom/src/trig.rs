//! ⭐ **O SENO SEM TRANSCENDENTAIS** — a aritmética que a anamorfose do halo precisa.
//!
//! # ⚠️ Porque ele vive aqui e não numa `sin` da biblioteca
//!
//! Esta crate é uma **folha de zero dependências** e a lei tem de ser a MESMA nos dois motores; a
//! `f32::sin` do Rust e a `sin` do WGSL não são a mesma função (HR-5). ⇒ a fase é polinomial, e os
//! gates dela **vieram com ela** — *uma lei que se muda de casa sem os testes dela chega à casa
//! nova sem régua.*
//!
//! ⚠️ **Corte por RESPONSABILIDADE, forçado pelo tecto de LOC** e melhor por isso: o irmão responde
//! *«que halo é que este quadro tem»* e isto responde *«em que direcção é que a tenda se estica»*.

// ⭐ O seno SEM TRANSCENDENTAIS (HR-5) que a base da tenda usa — veio com ela.
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

// ⭐ Os gates do seno vieram COM ele — *uma lei que se muda de casa sem os testes dela chega
// à casa nova sem régua.*
#[cfg(test)]
mod trig_tests {
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
