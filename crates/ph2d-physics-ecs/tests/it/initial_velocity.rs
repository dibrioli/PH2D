//! The bridge folds the optional `InitialVelocity` component into the sim, and
//! a rewind RE-ARMS it (ADR-0131 W9).
//!
//! `ph2d-physics` proves `BodyDesc::linvel`/`angvel` reach the solver. This is
//! the ECS half: (1) a body carrying the component launches and one without it
//! stays, and (2) after scrubbing the clock back to t=0 and replaying, the launch
//! is reproduced — which is the entire reason initial velocity rides the
//! `BodyDesc` the world rebuilds from, rather than being applied once and lost.
//!
//! Gravity is set to zero so the horizontal launch is the only motion, and the
//! control body staying put is unambiguous.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, PhysicsBridge, PhysicsSettings, RigidBody,
};

fn scene() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let ball = |x: f32| {
        (
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        )
    };
    // Launched right at 4 m/s; the control (no component) starts 5 m away.
    let launched = sim
        .world_mut()
        .spawn((
            ball(0.0),
            InitialVelocity {
                linvel: [4.0, 0.0],
                angvel: 0.0,
            },
        ))
        .id();
    let control = sim.world_mut().spawn(ball(5.0)).id();
    (sim, launched, control)
}

fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

#[test]
fn the_bridge_launches_a_body_with_initial_velocity_and_a_rewind_re_arms_it() {
    let (mut sim, launched, control) = scene();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());

    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let launched_x1 = x_of(&sim, launched);
    let control_x = x_of(&sim, control);

    // Folded: the component launched the body (~4·0.5 = 2 m past its start).
    assert!(
        launched_x1 > 1.5,
        "the body with InitialVelocity did not launch (x={launched_x1}) — the \
         bridge is not folding the component into the sim"
    );
    // The control has no component and did not move from x = 5.
    assert!(
        (control_x - 5.0).abs() < 1e-3,
        "the control (no InitialVelocity) moved to x={control_x}"
    );

    // Scrub back to t=0, then replay forward: the launch must reproduce, which
    // it can only do if the velocity rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let launched_x2 = x_of(&sim, launched);
    assert_eq!(
        launched_x1, launched_x2,
        "after a rewind to t=0 the launch was not re-armed (x {launched_x1} → \
         {launched_x2}) — initial velocity was applied once and lost on scrub"
    );
}
