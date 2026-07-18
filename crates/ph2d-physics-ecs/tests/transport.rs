//! The transport gates: the sim must obey the clock the way the timeline
//! does. Reset puts the body back at its rest pose (rapier cannot rewind, so
//! the bridge replays from the spawn description) and Pause **freezes** —
//! it must not quietly zero the velocity and restart the fall.
//!
//! Both were REAL defects: with only a forward path, Reset left the ball on
//! the floor and the physics looked dead to the TopBar buttons (Enio,
//! 2026-07-18).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const SPAWN_Y: f32 = 5.0;

/// Floor at y=0 (top 0.1) + a ball dropped from `SPAWN_Y`.
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
            },
            Transform::from_translation(Vec2::new(0.0, SPAWN_Y)),
        ))
        .id();
    (sim, ball)
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.y
}

/// Reset = the clock goes back to 0, and the ball must be back at the top,
/// ready to fall again. Mutation-tested: deleting the backwards (`Less`)
/// branch of `dispatch` leaves the ball on the floor → RED.
#[test]
fn resetting_the_clock_returns_the_body_to_its_rest_pose() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();

    for tick in 1..=300u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let landed = y_of(&sim, ball);
    assert!(landed < SPAWN_Y - 1.0, "the ball never fell (y={landed})");

    // Reset: clock rewound to 0 and paused (what the TopBar button does).
    bridge.dispatch(&mut sim, false, 0);
    let after = y_of(&sim, ball);
    assert!(
        (after - SPAWN_Y).abs() < 1e-4,
        "Reset did not return the ball to its rest pose: y={after}, expected {SPAWN_Y}"
    );
    assert_eq!(
        bridge.last_stepped(),
        0,
        "Reset left the tick counter ahead"
    );

    // …and playing again re-drops it (the replay left a usable world, not a
    // frozen one).
    for tick in 1..=300u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert!(
        y_of(&sim, ball) < SPAWN_Y - 1.0,
        "the ball did not fall again after Reset"
    );
}

/// Pause must FREEZE, not reset the motion. A run that is paused for 60
/// frames mid-fall has to land on exactly the same trajectory as one that was
/// never paused. Mutation-tested: making `settle` teleport unconditionally
/// (its old behaviour) zeroes the velocity every paused frame → the paused
/// run falls short → RED.
#[test]
fn pausing_mid_fall_does_not_change_the_trajectory() {
    // Reference: 90 ticks straight through.
    let (mut sim_a, ball_a) = scene();
    let mut bridge_a = PhysicsBridge::new();
    for tick in 1..=90u64 {
        bridge_a.dispatch(&mut sim_a, true, tick);
    }
    let straight = y_of(&sim_a, ball_a);

    // Same 90 ticks, but paused for 60 frames at tick 30 (mid-fall).
    let (mut sim_b, ball_b) = scene();
    let mut bridge_b = PhysicsBridge::new();
    for tick in 1..=30u64 {
        bridge_b.dispatch(&mut sim_b, true, tick);
    }
    let at_pause = y_of(&sim_b, ball_b);
    assert!(
        at_pause < SPAWN_Y && at_pause > 1.0,
        "fixture is not mid-fall (y={at_pause}) — the gate would prove nothing"
    );
    for _ in 0..60 {
        bridge_b.dispatch(&mut sim_b, false, 30);
    }
    assert_eq!(
        y_of(&sim_b, ball_b),
        at_pause,
        "a paused frame moved the ball"
    );
    for tick in 31..=90u64 {
        bridge_b.dispatch(&mut sim_b, true, tick);
    }

    assert_eq!(
        y_of(&sim_b, ball_b),
        straight,
        "pausing changed the trajectory — Pause is zeroing the velocity \
         instead of freezing"
    );
}
