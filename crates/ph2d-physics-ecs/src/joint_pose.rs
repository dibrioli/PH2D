//! **A pose de MUNDO de uma junta** — a porta única para ler e escrever o pivô e o eixo dela.
//!
//! A junta mora dentro do corpo A (ou da montagem, num pino de cenário): o `Transform` dela é
//! LOCAL ao pai, e o pivô e o eixo são pontos do MUNDO. Quem lê ou escreve o `Transform` de uma
//! junta cru volta ao defeito #2 da física (`docs/Physics/BUGS_physics.md`): numa junta-raiz os dois
//! coincidem e todo teste de raiz passa.

use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, Entity, Transform};

use crate::{JointAssembly, PhysicsJoint};

/// O pivô e o eixo da junta no MUNDO.
#[must_use]
pub fn joint_world_pose(world: &World, joint: Entity) -> Option<Transform> {
    ph2d_ecs::world_transform(world, joint)
}

/// Põe o pivô (e, com `rotation`, o eixo) da junta num ponto do MUNDO. Escreve só o que mudou no
/// LOCAL, então repetir a mesma pose não fabrica diff para o desfazer. `false` sem `Transform` ou
/// com a cadeia dos pais não invertível (nada é escrito).
pub fn set_joint_world_pose(
    world: &mut World,
    joint: Entity,
    xy: [f32; 2],
    rotation: Option<f32>,
) -> bool {
    let parent = ph2d_ecs::parent_world_transform(world, joint);
    let Some(local) = world.get::<Transform>(joint).copied() else {
        return false;
    };
    let now = Transform::compose(parent, local);
    let target = Transform {
        translation: ph2d_core::Vec2::new(xy[0], xy[1]),
        rotation: rotation.unwrap_or(now.rotation),
        ..now
    };
    let Some(next) = Transform::inverse_compose(parent, target) else {
        return false;
    };
    if next.translation != local.translation || next.rotation != local.rotation {
        let Some(mut t) = world.get_mut::<Transform>(joint) else {
            return false;
        };
        t.translation = next.translation;
        t.rotation = next.rotation;
    }
    true
}

/// Gira o eixo da junta para `rotation` (radianos, no MUNDO), sem mexer no pivô.
pub fn set_joint_world_axis(world: &mut World, joint: Entity, rotation: f32) -> bool {
    let Some(at) = joint_world_pose(world, joint) else {
        return false;
    };
    set_joint_world_pose(
        world,
        joint,
        [at.translation.x, at.translation.y],
        Some(rotation),
    )
}

/// Onde a junta MORA na Hierarquia: dentro do corpo A — o pivô é um ponto de A, então segue-o sem
/// reescrita —, ou, num pino de CENÁRIO (o prego não pode andar com o corpo), dentro da montagem;
/// sem montagem, na raiz.
#[must_use]
pub fn joint_home(world: &World, joint: Entity) -> Option<Entity> {
    let j = *world.get::<PhysicsJoint>(joint)?;
    let world_pin = j.body_b == 0 || world.get::<crate::JointWorldAnchor>(joint).is_some();
    let wanted = if world_pin { j.assembly } else { j.body_a };
    if wanted == 0 {
        return None;
    }
    let e = ph2d_ecs::entity_with_stable_id(world, ph2d_ecs::StableId(wanted))?;
    (world_pin || world.get::<JointAssembly>(e).is_none()).then_some(e)
}

/// Pendura `child` em `parent` (ou na raiz, com `None`) sem o mover no mundo. `false` quando a
/// pose não volta (pai degenerado) ou seria um ciclo — e aí nada muda.
pub fn adopt_keeping_world(world: &mut World, child: Entity, parent: Option<Entity>) -> bool {
    if world.get::<ChildOf>(child).map(ChildOf::parent) == parent {
        return true;
    }
    let Some(at) = ph2d_ecs::world_transform(world, child) else {
        return false;
    };
    let Some(p) = parent else {
        world.entity_mut(child).remove::<ChildOf>().insert(at);
        return true;
    };
    let mut up = Some(p);
    while let Some(a) = up {
        if a == child {
            return false;
        }
        up = world.get::<ChildOf>(a).map(ChildOf::parent);
    }
    let pw = ph2d_ecs::world_transform(world, p).unwrap_or(Transform::IDENTITY);
    let Some(local) = Transform::inverse_compose(pw, at) else {
        return false;
    };
    world.entity_mut(child).insert((local, ChildOf(p)));
    true
}

/// Leva a junta para a casa dela ([`joint_home`]) sem a mover.
pub fn rehome_joint(world: &mut World, joint: Entity) -> bool {
    let home = joint_home(world, joint);
    adopt_keeping_world(world, joint, home)
}
