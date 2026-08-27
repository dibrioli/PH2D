//! Transcendental-free `(cos, sin)` para o **eixo local** de um elemento — a mesma senoide
//! parabólica corrigida (Capens/devmaster) que o vento, a órbita e o oscilador usam. O ângulo
//! vai em **ciclos** (período 1); ~0,09% longe da trigonometria real usando só multiplicação e
//! `abs`, então a direcção é **determinística** (HR-5, *o `sin` do WGSL não tem garantia
//! cross-vendor*). Self-contained por crate-nó (isolamento de drop-crate).
//!
//! ⚠️ **É por isto que o espaço do elemento não chama `f32::sin_cos`:** a CPU e o device têm
//! de concordar, e a única forma de concordarem é as duas correrem a MESMA aproximação — o
//! gémeo em WGSL vive no `DRIVE_LIB` do `lib.rs`, e há gate a cruzar os dois.
//!
//! ⭐ Os quatro quartos de volta batem com a trigonometria real a `1e-6`, então `rot = 0` dá
//! `(1, 0)` e o espaço do elemento reduz ao do mundo **ao bit** numa cena que não gire nada.

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
