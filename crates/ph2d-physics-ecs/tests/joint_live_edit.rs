//! **W-JointParams — a joint parameter edit reaches the solver while the clock
//! is running.**
//!
//! The report (Enio, 2026-07-25): *"os parâmetros de Spring não mudam em nada o
//! comportamento da mola."* Measured airtight (`ph2d-physics/tests/joints.rs`
//! proves every knob varies the force at a FRESH spawn — so the engine seam is
//! fine): the edit reached the component but not the solver, because
//! `reconcile_joints` re-described a joint only `if at_rest` (`last_stepped == 0`).
//! Tuning a spring while watching it bounce — the whole point of a spring — did
//! nothing until a Reset.
//!
//! The `at_rest` gate was written in W3 to stop an ANCHOR being re-derived
//! mid-swing (which would bake in the swing offset). W-AnchorFollow made the
//! anchor authored BODY-LOCAL, seeded from the **rest** pose, so re-describing
//! mid-play no longer touches the anchor's frame — the protection moved and the
//! gate was left needlessly blocking every PARAMETER edit too. The fix drops
//! `at_rest` from the re-describe condition; the anchor protection is proven
//! intact by the anchor gates in `joints.rs`, which still pass.
//!
//! Every gate here is RED with the `at_rest &&` reinstated on
//! `bridge/joints.rs`'s re-describe condition — the mutation the fix removes.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

fn entity(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

fn ball_y(sim: &mut SimWorld) -> f32 {
    let e = entity(sim, "Ball");
    sim.world().get::<Transform>(e).expect("t").translation.y
}

/// Hook at (0,10), ball hanging at (0,9) on a rest-length-1 spring, stiffness 30.
fn spring_scene() -> SimWorld {
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
        Transform::from_translation(Vec2::new(0.0, 10.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Ball"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 9.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Spring"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Ball"),
            kind: JointKind::Spring,
            rest_length: 1.0,
            stiffness: 30.0,
            damping: 0.5,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 10.0)),
    ));
    sim
}

/// **The reported bug: tuning a spring WHILE it is running changes the spring.**
///
/// The settled sag is `m·g / k`: at k=30 the ball hangs ~0.065 m past the
/// rest length, at k=300 ~0.0065 m (measured in the wrapper sweep). The artist
/// plays the spring, watches it settle, then stiffens it — and it must tighten
/// up, not sit there until a Reset.
#[test]
fn stiffening_a_spring_mid_play_tightens_it() {
    let mut sim = spring_scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=400 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let soft_sag = (10.0 - ball_y(&mut sim)) - 1.0;
    assert!(
        soft_sag > 0.04,
        "the fixture is not settled soft: sag is {soft_sag:.4}, expected ~0.065 at k=30"
    );

    // The artist stiffens the spring 30 -> 300 while the sim is live.
    let spring = entity(&mut sim, "Spring");
    sim.world_mut()
        .get_mut::<PhysicsJoint>(spring)
        .expect("joint")
        .stiffness = 300.0;

    for tick in 401..=800 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let stiff_sag = (10.0 - ball_y(&mut sim)) - 1.0;
    assert!(
        stiff_sag < 0.02,
        "the spring was stiffened to k=300 mid-play (sag should fall to ~0.0065) \
         but the ball still hangs at sag {stiff_sag:.4} — the edit never reached \
         the solver, exactly the reported bug"
    );
}

/// **The same for a Pin's MOTOR — the bug is not spring-specific.**
///
/// A wheel (a disc pinned at its own centre, where gravity exerts no torque —
/// the measurement decision the wrapper's motor gate documents) is spun by a
/// motor. The artist changes the target speed while it turns, and it must
/// re-speed. Proves the fix covers every parameter, not just the spring.
#[test]
fn re_speeding_a_motor_mid_play_changes_the_spin() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hub"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Disc"),
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
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Motor"),
        PhysicsJoint {
            body_a: stable_name_id("Hub"),
            body_b: stable_name_id("Disc"),
            kind: JointKind::Pin,
            motor_enabled: true,
            motor_speed: 2.0,
            motor_max_force: 100.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=200 {
        bridge.dispatch(&mut sim, true, tick);
    }

    // Speed it up 2 -> 6 rad/s while it is turning.
    let motor = entity(&mut sim, "Motor");
    sim.world_mut()
        .get_mut::<PhysicsJoint>(motor)
        .expect("joint")
        .motor_speed = 6.0;

    for tick in 201..=400 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // Read the spin as the angle turned over one tick at the end.
    let disc = entity(&mut sim, "Disc");
    let a0 = sim.world().get::<Transform>(disc).expect("t").rotation;
    bridge.dispatch(&mut sim, true, 401);
    let a1 = sim.world().get::<Transform>(disc).expect("t").rotation;
    let mut d = a1 - a0;
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    let spin = d / (1.0 / 60.0); // rad/s at 60 fps
    assert!(
        spin > 4.0,
        "the motor was re-speeded to 6 rad/s mid-play but is still turning at \
         {spin:.2} rad/s — the parameter edit never reached the solver"
    );
}

/// **The guard the fix must not break: an un-edited joint does not churn.**
///
/// Dropping `at_rest` means a re-describe can now fire mid-play — it MUST fire
/// only on a genuine parameter/anchor diff, never every frame, or every frame
/// would clear the checkpoint ring and W1.5's scrub would die (the same failure
/// `a_dormant_joint_does_not_destroy_the_scrub_cache` guards on the other axis).
///
/// Play a spring forward WITHOUT editing it, then scrub back. If the ring were
/// being cleared by a spurious per-frame re-describe, the scrub would replay
/// every step from rest instead of restoring a checkpoint.
#[test]
fn an_unedited_joint_does_not_churn_the_scrub_cache() {
    let mut sim = spring_scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=200 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let before = bridge.steps_taken();
    bridge.dispatch(&mut sim, false, 150);
    let replayed = bridge.steps_taken() - before;
    assert!(
        replayed <= 10,
        "a scrub back to tick 150 replayed {replayed} steps on a joint nobody \
         edited — removing the at_rest gate is re-describing every frame and \
         clearing the ring, and W1.5 is dead"
    );
}
