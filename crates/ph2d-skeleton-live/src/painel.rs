//! O que o painel Bones pergunta de UM osso pelos bits: se ele vive, como se chama e quantos metros
//! de mundo vale uma unidade do comprimento dele.

use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_skeleton_ecs::Bone;

/// O eixo do osso é o `x` local: `|eixo x do afim de mundo|`, exacto sob rotação, escala
/// não-uniforme e skew da cadeia.
#[must_use]
pub fn metros_por_unidade(sim: &SimWorld, bits: u64) -> f64 {
    let x = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, Entity::from_bits(bits)),
    );
    let [a, b, ..] = x.0;
    a.hypot(b)
}

/// `bits` ainda é um osso vivo (o cartão preso a ele só lhe manda pedidos enquanto existir).
#[must_use]
pub fn live_bone(sim: &SimWorld, bits: u64) -> Option<u64> {
    Entity::try_from_bits(bits)
        .filter(|e| sim.world().get::<Bone>(*e).is_some())
        .map(|_| bits)
}

/// O osso e o nome dele — o subtítulo do cartão solto («Bone 3»).
#[must_use]
pub fn bone_subject(sim: &SimWorld, bits: u64) -> Option<(u64, String)> {
    let e = Entity::try_from_bits(live_bone(sim, bits)?)?;
    let nome = sim
        .world()
        .get::<Name>(e)
        .map_or_else(String::new, |n| n.as_str().to_owned());
    Some((bits, nome))
}
