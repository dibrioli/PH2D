//! W1 e2e gate: a `RigidBody{Dynamic}` sprite dropped above a
//! `Collider{Static}` floor **falls and settles** — driven entirely through
//! the ECS bridge (spawn from components → step at the tick → readback into
//! `Transform`), not the raw wrapper. This is the "integration, not unit"
//! gate (DIRETIVA §1): a dead bridge leaves the Transform at its spawn
//! height and the assert bleeds.
//!
//! Mutation-tested: commenting out the step loop OR the `readback` in
//! `PhysicsBridge::dispatch` leaves `y` at 5.0 → RED.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

#[test]
fn a_dynamic_sprite_falls_onto_a_static_floor_and_settles() {
    let mut sim = SimWorld::new();

    // Floor: static cuboid, top surface at y = 0.1.
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

    // Ball: dynamic, radius 0.25, dropped from y = 5.0 → should rest at
    // y ≈ floor_top(0.1) + radius(0.25) = 0.35.
    let spawn_y = 5.0_f32;
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
            Transform::from_translation(Vec2::new(0.0, spawn_y)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();

    // Before play, the bridge should not have moved anything.
    bridge.dispatch(&mut sim, false, 0);
    let y0 = sim.world().get::<Transform>(ball).unwrap().translation.y;
    assert!(
        (y0 - spawn_y).abs() < 1e-4,
        "paused bridge moved the ball: y={y0}, expected {spawn_y}"
    );

    // Play: 300 fixed ticks (5 s @ 60 Hz) — free-fall from 5 m is ~1 s,
    // plus settling.
    for tick in 1..=300u64 {
        bridge.dispatch(&mut sim, true, tick);
    }

    let y = sim.world().get::<Transform>(ball).unwrap().translation.y;
    assert!(
        y < spawn_y - 1.0,
        "the ball did not fall (y={y}, spawn was {spawn_y}) — the bridge is dead"
    );
    assert!(
        (0.25..=0.6).contains(&y),
        "the ball did not settle near floor+radius (0.35): y={y}"
    );

    // 51... no: 2 bodies here (floor + ball).
    assert_eq!(bridge.body_count(), 2);
}
