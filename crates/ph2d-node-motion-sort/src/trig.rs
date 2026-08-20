//! `(cos, sin)` **sem transcendental** para a direção do eixo de ordenação — a mesma senoide
//! parabólica corrigida (Capens/devmaster) que o oscillator/orbit/clone usam. O ângulo é em
//! **ciclos** (período 1).
//!
//! ⚠️ **Cópia local por convenção de drop-crate**, como nas outras dez folhas que a têm: o
//! vocabulário partilhado desta casa é a PORTA, não um símbolo comum, e uma folha que
//! dependesse de outra por causa de um seno deixaria de ser drop-in.
//!
//! ⚠️ **E ela existe por HR-5, não por velocidade.** O `f32::sin_cos` da `std` é a libm da
//! PLATAFORMA: dois SOs podem devolver ulps diferentes, e uma ordenação é
//! justamente onde um ulp vira uma PERMUTAÇÃO diferente — o replay deixaria de bater. ~0,09%
//! de erro contra o seno verdadeiro é irrelevante para escolher uma direção, e é
//! determinístico em toda parte.
//!
//! ⚠️ **`cos_sin_cycles(0)` é exactamente `(1, 0)`**, e isso é aritmética e não sorte:
//! `sin_cycles(0)` cai no ramo `f < 0.5` com `u = 0` ⇒ `p = 0` ⇒ `Q·(0−0)+0 = 0`, e
//! `sin_cycles(0.25)` dá `u = 0.5` ⇒ `p = 1` ⇒ `Q·(1−1)+1 = 1`.

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
    fn is_deterministic() {
        assert_eq!(cos_sin_cycles(0.37), cos_sin_cycles(0.37));
    }
}
