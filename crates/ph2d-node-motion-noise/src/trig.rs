//! Transcendental-free `(cos, sin)` para a ROTAÇÃO DO ESPAÇO deste campo — a mesma
//! senoide parabólica corrigida (Capens/devmaster) que o `motion.oscillator` usa e que
//! o `motion.orbit` já tinha. Ângulo em **ciclos** (período 1). ~0,09% da trig
//! verdadeira usando só multiply/abs, logo o campo é **determinístico** (HR-5).
//!
//! ⚠️ **Cópia por drop-crate, como o `falloff_at` e o `channel.rs`** — o vocabulário
//! partilhado desta casa é o comportamento, não um símbolo importado de um vizinho.
//! O que impede as cópias de divergirem **não é a disciplina**: é o gate de paridade
//! CPU×GPU, que compara este arquivo com o port WGSL linha-a-linha no `kernel.rs`.
//!
//! ⚠️ E a rotação aqui **não acumula**: cada quadro roda o ponto de amostragem
//! pristino por um ângulo ABSOLUTO, nunca o resultado do quadro anterior.

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
