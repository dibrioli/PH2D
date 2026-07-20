//! Authored initial velocity reaches the solver (ADR-0131 W9).
//!
//! `BodyDesc::linvel`/`angvel` are applied at spawn. Gravity is set to zero so
//! the ONLY thing that can move a body is its launch — a trajectory oracle where
//! "did it move / did it spin" answers exactly whether the field was applied.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

fn ball(x: f32, y: f32, linvel: [f32; 2], angvel: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel,
        angvel,
    }
}

/// A launched body travels, a spun body rotates, and a body with neither stays
/// put — under zero gravity, so the launch is the only cause.
///
/// Mutation-tested: `spawn_body` dropping `desc.linvel`/`angvel` (rapier's
/// default rest) leaves the launched body where it started and the spinner at
/// angle 0 → both asserts go RED.
#[test]
fn initial_velocity_launches_and_spins() {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);

    // Spaced far apart on y so nothing collides — this is pure ballistics.
    let launched = w.spawn_body(ball(0.0, 0.0, [4.0, 0.0], 0.0));
    let at_rest = w.spawn_body(ball(0.0, 10.0, [0.0, 0.0], 0.0));
    let spinner = w.spawn_body(ball(0.0, -10.0, [0.0, 0.0], 3.0));

    for _ in 0..30 {
        w.step(); // 0.5 s @ 60 Hz
    }
    let pose = |h| w.body_pose(h).expect("body exists");

    // Launched: x ≈ v·t = 4·0.5 = 2 m (no gravity, no drag).
    let x = pose(launched).translation.vector.x;
    assert!(
        (x - 2.0).abs() < 0.05,
        "launched body travelled to x={x}, expected ~2.0 (v·t) — the initial \
         linear velocity did not reach the solver"
    );

    // At rest: did not move.
    let rest_x = pose(at_rest).translation.vector.x;
    assert!(
        rest_x.abs() < 1e-3,
        "the body with no initial velocity moved to x={rest_x}"
    );

    // Spinner: angle ≈ ω·t = 3·0.5 = 1.5 rad.
    let angle = pose(spinner).rotation.angle();
    assert!(
        (angle - 1.5).abs() < 0.05,
        "spinner rotated to {angle} rad, expected ~1.5 (ω·t) — the initial \
         angular velocity did not reach the solver"
    );
}
