//! **Baking a JOINTED rig — one link is the whole rig.**
//!
//! Split off `physics_bake_tests.rs` under the shell's 600-LOC cap. The seam is
//! real: that file proves a lone body's baked curve is the simulation; this one
//! proves the selection is expanded to its jointed connected component first, so
//! a bake of one link cannot leave its coupled neighbours frozen.

use ph2d_core::Vec2;
use ph2d_ecs::scene::EditorCommandQueue;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

use super::tests::{BAKE_SECONDS, DT, registry};
use super::{BakeChannels, bake_selection};

/// A hook (static) with two dynamic links pinned in a line, and the L2 entity.
/// The links start off-axis so the whole thing swings and both move — a channel
/// that never moves writes no track, so a still link would not prove the flip.
fn chain() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let spawn_body = |sim: &mut SimWorld, name: &str, kind: BodyKind, x: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    };
    spawn_body(&mut sim, "Hook", BodyKind::Static, 0.0);
    spawn_body(&mut sim, "L1", BodyKind::Dynamic, 1.0);
    spawn_body(&mut sim, "L2", BodyKind::Dynamic, 2.0);
    let pin = |sim: &mut SimWorld, name: &str, a: &str, b: &str, x: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    };
    pin(&mut sim, "J0", "Hook", "L1", 0.5);
    pin(&mut sim, "J1", "L1", "L2", 1.5);
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    let l2 = q
        .iter(sim.world())
        .find(|(_, n)| n.as_str() == "L2")
        .map(|(e, _)| e)
        .expect("L2");
    (sim, l2)
}

fn kind_of(sim: &mut SimWorld, name: &str) -> BodyKind {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    let e = q
        .iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity");
    sim.world().get::<RigidBody>(e).expect("rb").kind
}

/// **Baking one link of a jointed chain bakes the WHOLE rig.**
///
/// Select ONLY the last link and bake: `jointed_group` must pull in the other
/// dynamic link too, so BOTH end up `Kinematic` (curve-driven). Without the
/// expansion the un-selected link stays `Dynamic` — and with the Physics toggle
/// off it would freeze at rest while the baked link plays, the exact half-baked
/// state the expansion exists to prevent. The static Hook is a boundary: it is
/// never baked (constant trajectory) and stays `Static`.
///
/// Mutation-tested: remove the `jointed_group` expansion in `bake_selection`
/// and only L2 is baked (`outcome.bodies == 1`, L1 stays `Dynamic`) — red.
#[test]
fn baking_one_link_of_a_chain_bakes_the_whole_rig() {
    let (mut sim, l2) = chain();
    let mut bridge = PhysicsBridge::new();
    let mut timeline = ph2d_timeline::TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();
    // Select ONLY the last link.
    let outcome = bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[l2],
        0.0,
        BAKE_SECONDS,
        DT,
        BakeChannels::All,
        &queue,
        &reg,
    );
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &queue, &reg).expect("apply");

    assert_eq!(
        outcome.bodies, 2,
        "baking one link baked {} bodies — the whole 2-link rig should have been \
         pulled in through the joint",
        outcome.bodies
    );
    assert_eq!(
        kind_of(&mut sim, "L1"),
        BodyKind::Kinematic,
        "the un-selected link stayed Dynamic — a partial bake that freezes when \
         Physics is off"
    );
    assert_eq!(kind_of(&mut sim, "L2"), BodyKind::Kinematic);
    assert_eq!(
        kind_of(&mut sim, "Hook"),
        BodyKind::Static,
        "the static hook was baked — it should be a boundary, not a body"
    );
}
