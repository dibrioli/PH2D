//! Transcendental-free `(cos, sin)` para a direção do ESTOURO — a mesma seno-parábola
//! corrigida que o `motion.emitter` usa para a direção de lançamento (Capens/devmaster).
//! Ângulo em **ciclos** (período 1); ~0,09% do trig verdadeiro usando só multiplicação e
//! `abs`, logo a direção é **determinística** (HR-5).
//!
//! ⚠️ **Cópia verbatim, e é o padrão deste repo:** dez e tantas crates-folha carregam o
//! próprio `trig.rs` (emitter, orbit, field.box, force.wind, fibonacci…), porque uma
//! crate-nó é drop-in e não tem para onde importar isto sem virar dependência. O que ela
//! NÃO pode é divergir — e o gate `the_burst_direction_matches_the_emitters` compara as
//! duas, ponto a ponto.

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
}
