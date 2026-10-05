//! ⭐⭐ **O ESQUELETO como OBJECTO** (A14, plano `docs/Skeleton/05_plano_o_esqueleto_e_um_objecto.md`;
//! a *Armature* do Blender).
//!
//! Marcador VAZIO: o tipo lê-se por presença (`ObjectKind::Skeleton`). Os ossos-raiz são filhos
//! (`ChildOf`) da entidade dele, e o `Transform` dela move o esqueleto inteiro — o mundo de um osso
//! já compõe a cadeia toda. Uma raiz continua a ser *«sem pai, ou pai que não é osso»*.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, SimComponent};
use serde::{Deserialize, Serialize};

/// O marcador do objecto esqueleto.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skeleton;

impl SimComponent for Skeleton {}

/// ⭐ **O esqueleto dono de `e`** — o próprio, se é um esqueleto, ou o primeiro ancestral que o é
/// (um osso sobe a corrente e chega ao objecto). `None` = solto (um osso de antes do A14 que a
/// migração ainda não cobriu, ou qualquer outra coisa).
#[must_use]
pub fn skeleton_of(world: &World, e: Entity) -> Option<Entity> {
    let mut cur = Some(e);
    while let Some(x) = cur {
        if world.get::<Skeleton>(x).is_some() {
            return Some(x);
        }
        cur = world.get::<ChildOf>(x).map(ChildOf::parent);
    }
    None
}

/// ⭐ **Os ossos de um esqueleto** — toda entidade com [`crate::Bone`] cujo dono é `skeleton`,
/// ordenada por bits (determinística dentro de uma sessão).
#[must_use]
pub fn bones_of(world: &World, skeleton: Entity) -> Vec<Entity> {
    let mut v: Vec<Entity> = world
        .iter_entities()
        .filter(|er| er.contains::<crate::Bone>() && skeleton_of(world, er.id()) == Some(skeleton))
        .map(|er| er.id())
        .collect();
    v.sort_by_key(|e| e.to_bits());
    v
}

#[cfg(test)]
#[path = "skeleton_tests.rs"]
mod tests;
