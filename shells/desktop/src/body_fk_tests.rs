//! Gates do gesto de FK (W-FK).
//!
//! A metade que precisa de janela (o `advance_body_fk`) é coberta pelo arch-gate
//! em `tests/`; aqui fica a decisão PURA — quando o gesto pega.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};

fn rig() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, kind: BodyKind| {
        let _ = sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ));
    };
    body("Hook", 0.0, BodyKind::Static);
    body("L1", 0.5, BodyKind::Dynamic);
    body("L2", 1.5, BodyKind::Dynamic);
    for (a, b, x) in [("Hook", "L1", 0.0), ("L1", "L2", 1.0)] {
        let _ = sim.world_mut().spawn((
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ));
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let link = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "L2")
            .map(|(e, _)| e)
            .expect("L2")
    };
    (sim, bridge, link)
}

#[test]
fn the_fk_takes_a_jointed_body_with_the_clock_stopped() {
    let (sim, mut bridge, link) = rig();
    assert!(take_fk(&mut bridge, &sim, true, link, [1.5, 0.0], false));
    assert!(bridge.is_posing_fk());
}

#[test]
fn the_fk_refuses_while_the_clock_runs() {
    // Tocando, a pose é do SOLVER: autorar aqui seria escrever num `Transform`
    // que o readback sobrescreve no mesmo frame.
    let (sim, mut bridge, link) = rig();
    assert!(!take_fk(&mut bridge, &sim, true, link, [1.5, 0.0], true));
    assert!(!bridge.is_posing_fk());
}

#[test]
fn the_fk_refuses_when_another_mode_is_in_hand() {
    let (sim, mut bridge, link) = rig();
    assert!(!take_fk(&mut bridge, &sim, false, link, [1.5, 0.0], false));
    assert!(!bridge.is_posing_fk());
}

/// **A recusa deixa o arrasto normal acontecer**, e é por isso que ela devolve
/// `false` em vez de consumir o press: um corpo sem junta acima ainda é um
/// objeto que o artista quer mover.
#[test]
fn a_body_with_no_joint_above_it_is_not_fk_posable() {
    let mut sim = SimWorld::new();
    let _ = sim.world_mut().spawn((
        Name::new("Lone"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider::default(),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let e = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world()).map(|(e, _)| e).next().expect("body")
    };
    assert!(!take_fk(&mut bridge, &sim, true, e, [0.0, 0.0], false));
}
