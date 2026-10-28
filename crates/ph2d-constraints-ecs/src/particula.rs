//! O alvo de um ponto de simulação: a partícula `id` do objecto Motion, no mundo de agora
//! (`docs/BlenderConstraints/07_W6_pontas.md` §5). Quem enche o recurso é o Motion.

use bevy_ecs::entity::Entity;
use ph2d_constraints::Pose2;
use ph2d_ecs::motion_generator::Particulas;
use ph2d_ecs::{SimWorld, StableId};

/// A partícula `id` do objecto `alvo` no mundo, pelo afim do objecto NA HORA (como o ponto de
/// forma). `None` se ela não está viva no último cozimento — o item fica sem alvo, nunca na origem.
pub(crate) fn no_mundo(sim: &SimWorld, alvo: Entity, s: StableId, id: u32) -> Option<Pose2> {
    let local = *sim
        .world()
        .get_resource::<Particulas>()?
        .0
        .get(&s)?
        .get(&id)?;
    let x = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, alvo),
    );
    let [px, py] = x.apply(local);
    Some(Pose2 {
        x: px,
        y: py,
        ..Pose2::IDENTITY
    })
}
