//! Unit tests for [`super::PhysicsWorld`], split out of `world.rs` to keep that
//! file under the workspace LOC cap. A child module (`super::*` reaches the
//! wrapper's private surface exactly as an inline `mod tests` did).

use super::*;

#[test]
fn empty_world_has_zero_bodies() {
    let w = PhysicsWorld::new();
    assert_eq!(w.body_snapshots().len(), 0);
    assert_eq!(w.step_count(), 0);
}

#[test]
fn step_advances_counter() {
    let mut w = PhysicsWorld::new();
    w.step();
    w.step();
    w.step();
    assert_eq!(w.step_count(), 3);
}

#[test]
fn dt_default_is_60hz() {
    let w = PhysicsWorld::new();
    // 1/60 ≈ 0.01666...; allow tiny f32 rounding.
    assert!((w.dt() - 1.0 / 60.0).abs() < 1e-6);
}

#[test]
fn falling_body_hits_floor() {
    let mut w = PhysicsWorld::new();
    // Floor at y=0, half-thickness 0.1.
    w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    // Ball at y=10 (10 m above floor).
    let (ball, _) = w.add_dynamic_circle(0.0, 10.0, 0.5, 1.0);
    // Step long enough to settle (gravity 9.81 m/s²; free-fall
    // from 10m takes ~1.43s; settling takes a few more).
    for _ in 0..600 {
        w.step();
    }
    let pose = w.body_pose(ball).expect("ball still exists");
    // Ball center should be near floor + radius (~ 0.1 + 0.5 = 0.6).
    assert!(
        pose.translation.y >= 0.5 && pose.translation.y <= 1.0,
        "ball settled at y={}, expected ~0.6",
        pose.translation.y
    );
}

#[test]
fn hash_is_stable_across_runs_in_same_process() {
    // Same fixture, same hash. Cross-OS test lives in the bin
    // (tests/spike-style C9 extension) — this is a sanity check
    // that the hashing function itself is deterministic on one
    // OS, not affected by allocation order or HashMap iteration.
    let h1 = run_50_body_fixture();
    let h2 = run_50_body_fixture();
    assert_eq!(h1, h2);
}

fn run_50_body_fixture() -> [u8; 32] {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    for i in 0..50 {
        let row = (i / 10) as f32;
        let col = (i % 10) as f32;
        w.add_dynamic_circle(col * 0.6 - 2.7, 5.0 + row * 0.6, 0.25, 1.0);
    }
    for _ in 0..120 {
        w.step();
    }
    w.deterministic_hash()
}

/// **The sweep behind [`super::joints::SERVO_STIFFNESS`] and its damping
/// sibling** — the arm the `MOTOR_TRACKING` table already uses (0.2 kg, 1 m,
/// hanging straight down from a pin at `(0, 6)`), told to HOLD `+45°` while
/// gravity pulls it the other way.
///
/// `#[ignore]` because it prints a table rather than asserting one: its output
/// is the justification pasted into those two doc comments. Run it with
/// `cargo test -p ph2d-physics servo_gain_sweep -- --ignored --nocapture`.
#[test]
#[ignore = "measurement: prints the table the servo constants are chosen from"]
fn servo_gain_sweep() {
    use super::joints::{JointDesc, JointKind, MotorDesc, MotorMode};
    let target = std::f32::consts::FRAC_PI_4;
    // The sweep's own body maker: `BodyDesc` has no `Default` (every field is a
    // decision the caller has to make), and this file has no fixture helper.
    let desc = |body_type, x: f32, y: f32, rotation: f32, shape| BodyDesc {
        body_type,
        x,
        y,
        rotation,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    };
    // (settled angle deg, worst overshoot deg, seconds to within 1 deg)
    let run = |stiffness: f32, damping: f32| -> (f32, f32, f32) {
        let mut w = PhysicsWorld::new();
        let hook = w.spawn_body(desc(
            RigidBodyType::Fixed,
            0.0,
            6.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
        ));
        // Hanging straight down: gravity pulls it AWAY from the +45 deg target,
        // so what the table reports is a servo holding against a real load.
        let arm = w.spawn_body(desc(
            RigidBodyType::Dynamic,
            0.0,
            5.5,
            -std::f32::consts::FRAC_PI_2,
            ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
        ));
        // ⚠️ `JointDesc`'s anchors are **body-LOCAL** (the caller converts once);
        // passing the world point straight in put the pin 6 m above each body and
        // the first table this printed was of a different rig entirely.
        let (la, lb) = w
            .world_to_local_anchors(hook, arm, [0.0, 6.0], [0.0, 6.0])
            .expect("anchors");
        w.spawn_joint_with_gains(
            hook,
            arm,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: la,
                anchor_b: lb,
                motor: Some(MotorDesc {
                    mode: MotorMode::Position,
                    speed: 0.0,
                    target,
                    // The DEFAULT ceiling, not a generous one: the table has to
                    // describe the servo the artist gets before touching a knob.
                    max_force: 10.0,
                }),
                ..JointDesc::default()
            },
            stiffness,
            damping,
        )
        .expect("joint");
        // ⚠️ **The angle is UNWRAPPED, and the first version of this table was
        // nonsense without it.** `rotation.angle()` wraps at +-pi, so a servo that
        // flies past its target reads as a number on the other side: stiffness
        // 5000 reported "111.91 deg settled" for an arm that had gone round.
        // Accumulate the per-step delta instead and the quantity is the one the
        // constants are about ([[reference_topic_oracle_discipline]]).
        let mut angle = w.body_pose(arm).unwrap().rotation.angle();
        let mut prev = angle;
        let mut overshoot = 0.0f32;
        let mut arrive = f32::NAN;
        for step in 0..300 {
            w.step();
            let a = w.body_pose(arm).unwrap().rotation.angle();
            let mut d = a - prev;
            while d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            }
            while d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            angle += d;
            prev = a;
            overshoot = overshoot.max(angle - target);
            if arrive.is_nan() && (angle - target).abs() < 1.0f32.to_radians() {
                arrive = (step + 1) as f32 / 60.0;
            }
        }
        (angle.to_degrees(), overshoot.to_degrees().max(0.0), arrive)
    };
    println!("target = 45.00 deg  (settled after 5 s | time to within 1 deg)");
    println!("stiffness |  settled  |   droop  | arrive s | overshoot   (damping = 700)");
    for k in [100.0f32, 300.0, 1000.0, 3000.0, 10000.0, 30000.0] {
        let (settled, overshoot, rate) = run(k, 700.0);
        let droop = 45.0 - settled;
        println!("{k:9} | {settled:8.2} | {droop:8.2} | {rate:8.2} | {overshoot:8.2}");
    }
    println!("\ndamping sweep at stiffness = 10000:");
    println!("  damping |  settled  | arrive s | overshoot");
    for d in [50.0f32, 100.0, 200.0, 300.0, 400.0, 500.0, 700.0, 1000.0] {
        let (settled, overshoot, rate) = run(10000.0, d);
        println!("  {d:7} | {settled:8.2} | {rate:8.2} | {overshoot:8.2}");
    }
}

/// **The sweep behind [`super::joints::MOTOR_TRACKING`] on a LINEAR motor** —
/// the case the original table did not have. A velocity motor is a damping
/// term, so working against gravity it settles a fixed `g / tracking` SHORT of
/// the speed it was told: 0.098 m/s at tracking 100, which is a fifth of the
/// 0.5 m/s a new rail is born with.
///
/// `#[ignore]`, same as its angular sibling: it prints the justification.
#[test]
#[ignore = "measurement: linear velocity-motor tracking error"]
fn linear_motor_tracking_sweep() {
    use super::joints::{JointDesc, JointKind, MotorDesc, MotorMode};
    let desc = |body_type, x: f32, y: f32, shape| BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    };
    println!("told 0.5 m/s up a VERTICAL rail, against gravity:");
    println!("  tracking | achieved m/s | shortfall");
    for tracking in [100.0f32, 300.0, 1000.0, 3000.0, 10000.0] {
        let mut w = PhysicsWorld::new();
        let post = desc(
            RigidBodyType::Fixed,
            0.0,
            3.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let post = w.spawn_body(post);
        let car = w.spawn_body(desc(
            RigidBodyType::Dynamic,
            0.0,
            3.0,
            ShapeDesc::Cuboid {
                half_x: 0.2,
                half_y: 0.2,
            },
        ));
        let (la, lb) = w
            .world_to_local_anchors(post, car, [0.0, 3.0], [0.0, 3.0])
            .expect("anchors");
        w.spawn_joint_tuned(
            post,
            car,
            JointDesc {
                kind: JointKind::Slider,
                anchor_a: la,
                anchor_b: lb,
                axis_a: [0.0, 1.0],
                axis_b: [0.0, 1.0],
                motor: Some(MotorDesc {
                    mode: MotorMode::Velocity,
                    speed: 0.5,
                    target: 0.0,
                    max_force: 10.0,
                }),
                ..JointDesc::default()
            },
            super::joints::SERVO_STIFFNESS,
            super::joints::SERVO_DAMPING,
            tracking,
        )
        .expect("joint");
        for _ in 0..60 {
            w.step();
        }
        let a = w.body_pose(car).unwrap().translation.y;
        for _ in 0..60 {
            w.step();
        }
        let b = w.body_pose(car).unwrap().translation.y;
        println!("  {tracking:8} | {:12.4} | {:9.4}", b - a, 0.5 - (b - a));
    }
}
