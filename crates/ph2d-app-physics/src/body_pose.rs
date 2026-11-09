//! **POSAR ARRASTANDO A PONTA** (W-IK) — o gesto de cinemática inversa.
//!
//! História e porquê: `git log` deste ficheiro.

use ph2d_ecs::{Entity, SimWorld, Transform, parent_world_transform};
use ph2d_physics_ecs::PhysicsBridge;

/// **A porta única do press.** `true` = a pose pegou, e o chamador NÃO deve
/// abrir arrasto de gizmo nem mexer na seleção.
///
/// As condições 1 e 2 chegam como argumento (é o chamador que vê o relógio e a
/// ferramenta), a 3 é da ponte. A divisão é a mesma do `body_grab::take_hold`,
/// e pelo mesmo motivo: assim a decisão inteira é testável sem janela.
pub fn take_pose(physics: &mut PhysicsBridge, is_ik: bool, entity: Entity, playing: bool) -> bool {
    if playing || !is_ik {
        return false;
    }
    physics.ik_begin(entity)
}

/// **A cadeia segue o cursor.** No-op sem sessão, então roda em todo Move ao lado dos
/// outros `advance_*`. **Devolve `true` se AUTOROU.**
///
/// ⚠️ Devolve «autorou» ao contrário do `advance_body_grab` — e a diferença é
/// exatamente a que o cabeçalho descreve: aquele cutuca o solver e não deixa nada;
/// este AUTORA a cena, então o diff de undo tem de ver o quadro em que a pose mudou.
///
/// ⚠️ Era um `impl App` (W2/L2 Fase B). As `opts` entram por ARGUMENTO porque vivem no
/// `PhysicsState`, que é campo da `App` — e é o chamador que o tem.
pub fn advance_body_pose(
    physics: &mut PhysicsBridge,
    sim: &mut SimWorld,
    world: [f32; 2],
    opts: ph2d_physics_ecs::IkOptions,
) -> bool {
    if !physics.is_posing() {
        return false;
    }
    // O ângulo alvo é o que a ponta JÁ tem: com `Match` ligado, arrastar mantém a
    // atitude em vez de girar a ponta para um número que o mouse não tem. Lido do
    // `Transform` autorado, que é onde a pose vive.
    let tip_angle = physics
        .posing_tip()
        .and_then(|e| sim.world().get::<Transform>(e))
        .map_or(0.0, |t| t.rotation);
    let poses = physics.ik_move(world, tip_angle, opts);
    for (e, translation, rotation) in poses {
        write_world_pose(sim, e, translation, rotation);
    }
    true
}

/// Encerra o gesto. Idempotente — soltar sem ter posado é o caso de quase todo release
/// do app, e é por isso que ele mora no topo do handler junto do irmão da mão.
pub fn release_body_pose(physics: &mut PhysicsBridge) {
    physics.ik_end();
}

pub fn write_world_pose(
    sim: &mut ph2d_ecs::SimWorld,
    e: Entity,
    translation: [f32; 2],
    rotation: f32,
) {
    let parent = parent_world_transform(sim.world(), e);
    let Some(current) = sim.world().get::<Transform>(e).copied() else {
        return;
    };
    let mut world = current;
    world.translation.x = translation[0];
    world.translation.y = translation[1];
    world.rotation = rotation;
    let Some(local) = Transform::inverse_compose(parent, world) else {
        return;
    };
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        // Só o que a IK de fato resolveu. A escala e o skew ficam onde o artista
        // os deixou.
        t.translation = local.translation;
        t.rotation = local.rotation;
    }
}

#[cfg(test)]
#[path = "body_pose_tests.rs"]
mod tests;
