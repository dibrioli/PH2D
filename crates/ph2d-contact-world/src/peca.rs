//! **O que o stream declara de UMA peça**, e o corpo do rapier que a representa (doc 121 §9.20,
//! ponto 4 do desenho).

use ph2d_contact::{Forma, Material};
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, SIZE_IDENTITY, Stream,
};
use rapier2d::prelude::*;

/// Tudo o que o mundo precisa para construir a peça — e, comparado por igualdade, o que diz que
/// ela MUDOU (o `size` animado, o cartão editado em Play).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Spec {
    /// A forma no referencial da peça (o `rot` é a rotação do CORPO, nunca da forma).
    pub forma: Forma,
    /// O centro do colisor no referencial da peça (o `Offset` do cartão, já escalado pelo `size`).
    pub desvio: [f32; 2],
    pub massa_inv: f32,
    pub inercia_inv: f32,
    pub material: Option<Material>,
}

fn escalares<'a>(s: &'a Stream, nome: &str) -> Option<&'a [f32]> {
    match s.get(nome) {
        Some(Column::Scalar(v)) if v.len() == s.count() => Some(v),
        _ => None,
    }
}

fn pares<'a>(s: &'a Stream, nome: &str) -> Option<&'a [[f32; 2]]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) if v.len() == s.count() => Some(v),
        _ => None,
    }
}

/// A [`Spec`] de cada linha — `None` quando ninguém declara colisor (o passo é o de sempre), e
/// `Some(None)` numa linha cujo colisor é inválido (ela não entra no mundo e atravessa as outras).
///
/// ⭐ As portas são as MESMAS de antes do mundo: [`ph2d_contact::declarado`] (com o ângulo `0`: a
/// forma no referencial da peça), [`ph2d_contact::inv_inercias`] (o `Lock Rotation` escreve
/// `inv_inertia = 0`) e [`ph2d_contact::materiais`].
pub(crate) fn specs(s: &Stream, pesos: &[f32]) -> Option<Vec<Option<Spec>>> {
    let (raio, caixa) = (escalares(s, COLLIDER_COLUMN), pares(s, COLLIDER_BOX_COLUMN));
    if raio.is_none() && caixa.is_none() {
        return None;
    }
    let n = s.count();
    let (desvio, size) = (pares(s, COLLIDER_OFFSET_COLUMN), pares(s, "size"));
    let locais: Vec<Option<ph2d_contact::Colisor>> = (0..n)
        .map(|i| {
            ph2d_contact::declarado(
                raio.map(|v| v[i]),
                caixa.map(|v| v[i]),
                desvio.map_or([0.0, 0.0], |v| v[i]),
                size.map_or(SIZE_IDENTITY, |v| v[i]),
                0.0,
            )
        })
        .collect();
    let inercia = ph2d_contact::inv_inercias(s, &locais, pesos);
    let material = ph2d_contact::materiais(s);
    Some(
        locais
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let c = (*c)?;
                let w = pesos.get(i).copied().unwrap_or(1.0);
                (w.is_finite() && w >= 0.0).then(|| Spec {
                    forma: c.forma,
                    desvio: c.desvio,
                    massa_inv: w,
                    inercia_inv: inercia[i],
                    material: material.as_ref().map(|m| m[i]),
                })
            })
            .collect(),
    )
}

/// **As regras de combinação** (doc 109 §7): atrito `√(a·b)` e salto `max` — as do Box2D, que o
/// rapier `0.35` tem (`GeometricMean`, `Max`).
///
/// ⚠️ **Uma peça SEM material declarado sofre o atrito do obstáculo INTEIRO** (a distinção
/// `None`/`Some(LISO)` da [`ph2d_contact::materiais`]): ela entra com `0` e a regra `Max`, e o
/// obstáculo com a regra `Average` — o rapier aplica a regra de MAIOR prioridade das duas, logo
/// `max(0, μ)` contra o obstáculo e `max(0, 0) = 0` contra outra peça lisa, a lei de antes ao bit.
pub(crate) fn colisor(spec: &Spec, etiqueta: u128) -> Collider {
    let b = match spec.forma {
        Forma::Disco(r) => ColliderBuilder::ball(r),
        Forma::Caixa { meia, .. } => ColliderBuilder::cuboid(meia[0], meia[1]),
    };
    let (atrito, salto, regra) = match spec.material {
        Some(m) => (m.atrito, m.salto, CoefficientCombineRule::GeometricMean),
        None => (0.0, 0.0, CoefficientCombineRule::Max),
    };
    b.translation(Vector::new(spec.desvio[0], spec.desvio[1]))
        .density(0.0)
        .friction(atrito)
        .friction_combine_rule(regra)
        .restitution(salto)
        .restitution_combine_rule(CoefficientCombineRule::Max)
        .user_data(etiqueta)
        .build()
}

/// A massa e a inércia EXPLÍCITAS (o colisor tem densidade `0`): `1/inv_mass` e `1/inv_inertia`,
/// pela mesma porta de antes — um pino (`inv_mass = 0`) é um corpo CINEMÁTICO que o stream conduz.
pub(crate) fn tipo_e_massa(spec: &Spec) -> (RigidBodyType, MassProperties, bool) {
    if spec.massa_inv <= 0.0 {
        return (
            RigidBodyType::KinematicPositionBased,
            MassProperties::default(),
            true,
        );
    }
    let travada = spec.inercia_inv <= 0.0;
    let inercia = if travada { 0.0 } else { 1.0 / spec.inercia_inv };
    (
        RigidBodyType::Dynamic,
        MassProperties::new(Vector::ZERO, 1.0 / spec.massa_inv, inercia),
        travada,
    )
}
