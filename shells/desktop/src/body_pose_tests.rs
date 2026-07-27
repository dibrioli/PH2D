//! Gates do gesto de POSE (W-IK).
//!
//! A metade que precisa de janela (o `advance_body_pose`) é coberta pelo
//! arch-gate em `tests/`; aqui fica a decisão PURA — quando o gesto pega.

use super::*;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};

fn rig() -> (ph2d_ecs::SimWorld, PhysicsBridge, Entity) {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, stable_name_id};
    let mut sim = ph2d_ecs::SimWorld::new();
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
    let tip = {
        let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "L2")
            .map(|(e, _)| e)
            .expect("L2")
    };
    (sim, bridge, tip)
}

#[test]
fn the_pose_takes_a_jointed_body_with_the_clock_stopped() {
    let (_sim, mut bridge, tip) = rig();
    assert!(take_pose(&mut bridge, true, tip, false));
    assert!(bridge.is_posing());
}

#[test]
fn the_pose_refuses_while_the_clock_runs() {
    // Tocando, a pose é do SOLVER: autorar aqui seria escrever num `Transform`
    // que o readback sobrescreve no mesmo frame.
    let (_sim, mut bridge, tip) = rig();
    assert!(!take_pose(&mut bridge, true, tip, true));
    assert!(!bridge.is_posing());
}

#[test]
fn the_pose_refuses_when_another_tool_is_in_hand() {
    let (_sim, mut bridge, tip) = rig();
    assert!(!take_pose(&mut bridge, false, tip, false));
    assert!(!bridge.is_posing());
}

#[test]
fn a_body_with_no_rigid_chain_is_not_posable() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Name;
    let mut sim = ph2d_ecs::SimWorld::new();
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
    assert!(!take_pose(&mut bridge, true, e, false));
}

#[test]
fn the_written_pose_is_local_and_keeps_the_scale() {
    // O solver fala MUNDO e o `Transform` é LOCAL: sob um pai deslocado, a
    // conversão é a coisa inteira. E a escala não é da IK.
    use ph2d_core::Vec2;
    let mut sim = ph2d_ecs::SimWorld::new();
    let parent = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(5.0, 0.0)),))
        .id();
    let child = sim
        .world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(0.0, 0.0),
                scale: Vec2::new(3.0, 3.0),
                ..Transform::default()
            },
            ph2d_ecs::ChildOf(parent),
        ))
        .id();
    write_world_pose(&mut sim, child, [7.0, 2.0], 0.5);
    let t = sim.world().get::<Transform>(child).copied().expect("child");
    assert!(
        (t.translation.x - 2.0).abs() < 1e-4,
        "local x = {}",
        t.translation.x
    );
    assert!((t.translation.y - 2.0).abs() < 1e-4);
    assert!((t.rotation - 0.5).abs() < 1e-5);
    assert!(
        (t.scale.x - 3.0).abs() < 1e-6,
        "the IK must not touch the scale"
    );
}
