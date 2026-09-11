//! **The rest pose is the AUTHORED pose at tick 0.**
//!
//! A body's rest state is what Reset returns it to and what a cache miss
//! replays from. It was captured once, at spawn, and never looked at again —
//! so moving an object in the viewport and pressing Reset threw the artist's
//! placement away and jumped the object back to wherever it first appeared.
//!
//! The rule that fixes it is one line of meaning: at tick 0 the body simply
//! *is* its authored description, re-read every frame rather than remembered.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const SPAWN: Vec2 = Vec2::new(0.0, 5.0);
/// Where the artist drags the ball to — far from the spawn on BOTH axes, so
/// a partial fix cannot pass by accident.
const PLACED: Vec2 = Vec2::new(2.5, 3.0);

fn scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(SPAWN),
        ))
        .id();
    (sim, ball)
}

fn pos(sim: &SimWorld, e: Entity) -> Vec2 {
    sim.world().get::<Transform>(e).unwrap().translation
}

/// Drag the object where the gizmo would: write its `Transform`.
fn place(sim: &mut SimWorld, e: Entity, at: Vec2) {
    sim.world_mut().get_mut::<Transform>(e).unwrap().translation = at;
}

/// The reported bug, as a gate. Mutation-tested: dropping the tick-0
/// re-describe from `reconcile_structure` makes the final assert red with the
/// ball back at its spawn.
#[test]
fn moving_an_object_and_resetting_keeps_the_new_placement() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();

    // Watch it fall once, then rewind — the ordinary thing an artist does
    // before deciding they want it somewhere else.
    for tick in 1..=200u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(pos(&sim, ball), SPAWN, "Reset must return to the rest pose");

    // The artist drags it. Paused, at tick 0 — this IS authoring.
    place(&mut sim, ball, PLACED);
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        pos(&sim, ball),
        PLACED,
        "a paused body must stay where it was put"
    );

    // Play: it falls from the NEW place, not the old one.
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let falling = pos(&sim, ball);
    assert!(
        (falling.x - PLACED.x).abs() < 1e-4 && falling.y < PLACED.y,
        "the ball should fall from the placed position, but it is at {falling:?}"
    );

    // And Reset returns to where the artist PUT it.
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        pos(&sim, ball),
        PLACED,
        "Reset threw away the artist's placement and jumped the object back to its original \
         spawn — the rest pose was remembered from spawn instead of read from the scene"
    );
}

/// The rule must not fire while the clock is running: past tick 0 the
/// `Transform` is the sim's OUTPUT, so re-describing from it would feed the
/// simulation its own result and rebuild the body every single frame.
///
/// This is the sibling that gives the fix its edges — without it, "always
/// re-describe" would pass the gate above and destroy the simulation
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn the_rest_pose_is_not_re_read_while_the_clock_is_past_zero() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();

    for tick in 1..=120u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let mid = pos(&sim, ball);
    assert!(
        mid.y < SPAWN.y - 1.0,
        "fixture is not mid-fall (y={})",
        mid.y
    );

    // Rewinding must go back to the SPAWN, not to wherever the readback last
    // wrote — proof the sim's own output never became the rest pose.
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        pos(&sim, ball),
        SPAWN,
        "the simulated pose leaked into the rest state"
    );

    // And the body is not being torn down and rebuilt every frame: the count
    // stays put, and the fall keeps ACCELERATING. Velocity is the sharp test —
    // a body respawned each frame is re-created at rest, so it would still
    // descend (gravity restarts too) but at a constant crawl instead of
    // speeding up.
    //
    // Sampled at tick 30, deliberately: free fall from y=5 reaches the floor
    // at ~58 ticks, so a later sample would be measuring a ball at REST and
    // the gate would prove nothing about falling.
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(bridge.body_count(), 2, "bodies were rebuilt mid-flight");
    let a = pos(&sim, ball);
    bridge.dispatch(&mut sim, true, 31);
    let first = a.y - pos(&sim, ball).y;
    let b = pos(&sim, ball);
    bridge.dispatch(&mut sim, true, 32);
    let second = b.y - pos(&sim, ball).y;
    assert!(
        first > 0.0 && second > first,
        "the fall is not accelerating ({first} then {second} per tick) — a body respawned \
         mid-flight loses its velocity every frame"
    );
}
