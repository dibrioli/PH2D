//! The bridge folds the optional `GravityScale` component into the sim
//! (ADR-0131 W8).
//!
//! `ph2d-physics` already proves `BodyDesc::gravity_scale` reaches the solver
//! (`gravity_scale.rs` there). This is the ECS half: a body carrying the
//! optional `GravityScale` component must be spawned with that multiplier, and
//! a body without one falls normally. e2e through the bridge (components →
//! spawn → step → readback `Transform`), so a bridge that never reads the
//! component leaves the weightless body falling.
//!
//! Mutation-tested: the bridge's `world.get::<GravityScale>(e)` folded to the
//! neutral `1.0` (ignoring the component) makes the weightless ball fall like
//! the control → RED.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, GravityScale, PhysicsBridge, RigidBody};

#[test]
fn a_gravity_scale_component_is_folded_into_the_sim() {
    let mut sim = SimWorld::new();
    let spawn_y = 5.0_f32;

    let dynamic_ball = |x: f32| {
        (
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, spawn_y)),
        )
    };

    // Control: no GravityScale component → full gravity.
    let control = sim.world_mut().spawn(dynamic_ball(0.0)).id();
    // Weightless: the same body, plus GravityScale(0.0). Spaced 5 m away so the
    // two never touch (there is no floor — this is free fall).
    let weightless = sim
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
            Transform::from_translation(Vec2::new(5.0, spawn_y)),
            GravityScale(0.0),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=60u64 {
        bridge.dispatch(&mut sim, true, tick);
    }

    let control_y = sim.world().get::<Transform>(control).unwrap().translation.y;
    let weightless_y = sim
        .world()
        .get::<Transform>(weightless)
        .unwrap()
        .translation
        .y;

    assert!(
        control_y < spawn_y - 1.0,
        "the control (no GravityScale) should have fallen; y={control_y}"
    );
    assert!(
        (weightless_y - spawn_y).abs() < 0.01,
        "the GravityScale(0.0) body must not fall — the bridge is not reading the \
         component (y={weightless_y}, spawn was {spawn_y})"
    );
}
