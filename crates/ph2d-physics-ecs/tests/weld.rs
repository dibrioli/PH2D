//! **Weld (FixedJoint)** — ADR-0131 W3 carry-over. A weld locks two bodies
//! rigidly at the anchor: no relative translation OR rotation. This drives the
//! real authoring path (a `PhysicsJoint{kind: Weld}` component → the bridge
//! derives the shared-point anchors → rapier's `FixedJoint`) and contrasts it
//! with a Pin, which shares the same point but lets the body swing.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// A plank joined to a STATIC hook at the plank's LEFT end. The plank's centre
/// is 0.5 m right of the anchor, so gravity has a lever to rotate it about the
/// anchor — which a Pin allows and a Weld forbids. Returns the plank's absolute
/// rotation after it has settled.
fn plank_rotation_after_settling(kind: JointKind) -> f32 {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Plank"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.5, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Joint"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Plank"),
            kind,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=180u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
    let plank = q
        .iter(sim.world())
        .find(|(_, n)| n.as_str() == "Plank")
        .map(|(e, _)| e)
        .expect("plank");
    sim.world()
        .get::<Transform>(plank)
        .expect("transform")
        .rotation
        .abs()
}

/// **A Weld holds the body rigid where a Pin would swing.** Same scene, same
/// shared-point anchor — only the kind differs. The Weld keeps the plank at its
/// authored rotation (≈ 0); the Pin lets it hang.
///
/// The Pin is the CONTROL that gives the Weld assertion meaning: a weld that
/// silently mapped to a revolute joint would pass "rotation ≈ 0" only if the
/// whole scene were frozen, so the test also proves the Pin is NOT frozen.
///
/// Mutation-tested: mapping `JointKind::Weld` to a `RevoluteJoint` in
/// `spawn_joint`, or keying the anchor on `is_hinge()` instead of
/// `shares_a_point()`, lets the welded plank swing and this goes red.
#[test]
fn a_weld_holds_the_body_rigid_where_a_pin_would_swing() {
    let weld = plank_rotation_after_settling(JointKind::Weld);
    let pin = plank_rotation_after_settling(JointKind::Pin);

    assert!(
        weld < 0.05,
        "a Weld should hold the plank at its authored rotation (≈ 0), but it turned {weld:.3} rad"
    );
    assert!(
        pin > 0.5,
        "the Pin control should let the plank swing, but it only turned {pin:.3} rad — the scene \
         is frozen and the Weld assertion means nothing"
    );
    assert!(
        pin > weld * 5.0,
        "the Weld ({weld:.3}) is not visibly more rigid than the Pin ({pin:.3})"
    );
}
