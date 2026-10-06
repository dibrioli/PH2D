//! **Os obstáculos do `sim.collide` como colisores FIXOS do mundo** (doc 121 §9.20, ponto 5).

use ph2d_contact::obstaculo::FormaFixa;
use ph2d_motion_kit::trig;
use rapier2d::prelude::*;

/// A flecha máxima da polilinha que faz a TAÇA (o rapier não tem «dentro de um círculo»): a
/// distância entre a corda e o arco, em unidades do mundo. As peças da `=114` medem `0,22`; `10⁻³`
/// é `0,5 %` delas, abaixo do erro de contacto que o próprio solver admite (`0,005`).
const FLECHA: f32 = 1e-3;
/// Os lados da polilinha nunca descem daqui (uma taça minúscula continua redonda) nem passam dali
/// (o custo da fase larga cresce com eles; `4 096` lados dão a flecha a uma taça de raio `~3 400`).
const LADOS_MIN: usize = 32;
const LADOS_MAX: usize = 4096;

/// Quantos lados a polilinha de uma taça de `raio` precisa para a flecha não passar da [`FLECHA`]:
/// `r · (1 − cos(π/N)) ≤ f` ⇔ `N ≥ π / acos(1 − f/r)`, e `acos(1 − x) ≈ √(2x)` para `x` pequeno.
fn lados(raio: f32) -> usize {
    if !(raio.is_finite() && raio > FLECHA) {
        return LADOS_MIN;
    }
    let meio_angulo = (2.0 * FLECHA / raio).sqrt();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "uma contagem positiva, limitada logo a seguir"
    )]
    let n = (std::f32::consts::PI / meio_angulo).ceil() as usize;
    n.clamp(LADOS_MIN, LADOS_MAX)
}

/// Os colisores fixos de um obstáculo declarado. ⚠️ A regra do atrito é `Average` de propósito: a
/// da peça decide (ver [`crate::peca::colisor`]).
///
/// ⭐ **A TAÇA é um colisor POR LADO, não uma polilinha** (doc 121 §9.20): a polilinha tem a caixa
/// do tamanho da taça, e cada peça lá dentro faz par com ela — o 1.º passo de um mundo novo de
/// `16 384` peças custava `213` ms (o recomeço de cada `Loop`). Lado a lado só as peças junto da
/// parede fazem par: `8` ms, e o passo de regime `3,29 → 2,91` ms, a mesma pilha.
pub(crate) fn colisores(
    forma: FormaFixa,
    atrito: f32,
    salto: f32,
    etiqueta: u128,
) -> Vec<Collider> {
    let acaba = |b: ColliderBuilder| {
        b.friction(atrito)
            .friction_combine_rule(CoefficientCombineRule::Average)
            .restitution(salto)
            .restitution_combine_rule(CoefficientCombineRule::Max)
            .user_data(etiqueta)
            .build()
    };
    let b = match forma {
        FormaFixa::Plano { normal, altura } => {
            let n = Vector::new(normal[0], normal[1]);
            ColliderBuilder::halfspace(rapier2d::na::Unit::new_unchecked(n)).translation(n * altura)
        }
        FormaFixa::Disco { centro, raio } => {
            ColliderBuilder::ball(raio).translation(Vector::new(centro[0], centro[1]))
        }
        FormaFixa::Taca { centro, raio } => {
            let n = lados(raio);
            #[expect(clippy::cast_precision_loss, reason = "um índice de lado, <= 4096")]
            let pontos: Vec<Vector> = (0..=n)
                .map(|i| {
                    let (c, s) = trig::cos_sin_cycles((i % n) as f32 / n as f32);
                    Vector::new(centro[0] + raio * c, centro[1] + raio * s)
                })
                .collect();
            return pontos
                .windows(2)
                .map(|w| acaba(ColliderBuilder::segment(w[0], w[1])))
                .collect();
        }
        FormaFixa::Caixa { centro, meia, eixo } => ColliderBuilder::cuboid(meia[0], meia[1])
            .position(Pose::from_parts(
                Vector::new(centro[0], centro[1]),
                Rotation {
                    re: eixo[0],
                    im: eixo[1],
                },
            )),
    };
    vec![acaba(b)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_taca_tem_a_flecha_que_promete() {
        for raio in [0.5_f32, 1.8, 10.0, 33.0] {
            let n = lados(raio);
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de lados")]
            let flecha = raio * (1.0 - (std::f32::consts::PI / n as f32).cos());
            assert!(
                flecha <= FLECHA * 1.01,
                "raio {raio}: {n} lados, flecha {flecha}"
            );
        }
    }
}
