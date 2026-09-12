//! A sonda da cena 78 + os gates que mantêm a mensagem dela honesta
//! (W-JointAnim).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena E as tracks, e devolve o par que o produto usa.
fn staged() -> (SimWorld, TimelineDoc, [Entity; 4], PhysicsBridge) {
    let mut sim = SimWorld::new();
    let joints = build_joint_anim_scene(sim.world_mut());
    // A costura NOME → IDENTIDADE (ADR-0164 F1) — o roteador das cenas fá-la no produto.
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut doc = TimelineDoc::new();
    author_joint_anim_tracks(&mut doc, joints);
    (sim, doc, joints, PhysicsBridge::new())
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 78 nao montou '{name}'"))
}

/// **A cena monta as cinco máquinas que a mensagem nomeia.**
#[test]
fn the_scene_builds_the_machines_it_names() {
    let (mut sim, _, joints, _) = staged();
    for n in [
        "ServoArm",
        "CtrlArm",
        "WinchLoad",
        "MuscleWeight",
        "SpinBlade",
    ] {
        let _ = named(&mut sim, n);
    }
    assert_eq!(joints.len(), 4);
}

/// **Cada canal novo tem uma track**, e é a track que a mensagem promete.
///
/// ⚠️ Sem isto a cena poderia montar quatro máquinas paradas e a mensagem
/// continuaria dizendo que elas são animadas.
#[test]
fn every_new_channel_is_actually_keyed_in_this_scene() {
    let (_, doc, joints, _) = staged();
    for (i, prop) in [
        PropKind::JointMotorTarget,
        PropKind::JointMaxLength,
        PropKind::JointRestLength,
        PropKind::JointMotorSpeed,
    ]
    .into_iter()
    .enumerate()
    {
        let bound = doc
            .bindings()
            .iter()
            .any(|b| b.entity == joints[i].to_bits() && b.prop == prop);
        assert!(bound, "{prop:?} tem de estar bound na cena 78");
    }
}
