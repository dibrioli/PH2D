//! **What does a joint actually carry, and what does breaking cost?**
//! (ADR-0131 W-J7 — the measurement the defaults and the always-on decision come
//! from.)
//!
//! Four questions, and none of them can be answered by reading the source:
//!
//! 1. **Is the reading a FORCE?** rapier publishes `ImpulseJoint::impulses`, an
//!    impulse over the solver's own small step. Dividing by that step should give
//!    newtons — and the way to know is to hang a known weight and see whether the
//!    joint reads `m·g`.
//! 2. **Does it survive a change of sub-step count?** This is the whole reason the
//!    threshold is written in newtons rather than in newton-seconds (the
//!    timestep-dependence complaint the research collected against Godot). If the
//!    number moves when `set_substeps` does, the unit is a lie.
//! 3. **What is the SCALE?** A default threshold has to be a number an artist can
//!    reason about — big enough that ordinary rigs hold, small enough that a real
//!    yank breaks. That needs a table of what ordinary rigs read.
//! 4. **What does the per-sub-step scan cost?** Always-on is only honest if it is
//!    cheap (the W-ImpactForce precedent: measure first, then commit).
//!
//! Run with output:
//! ```text
//! cargo test -p ph2d-physics --release --test measure_joint_break -- --nocapture --ignored
//! ```
//! `--release` is not a preference (ADR-0124): a debug build measures the profile.

use ph2d_physics::{BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyType, ShapeDesc};
use std::time::Instant;

fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    mass: Option<f32>,
) -> ph2d_physics::RigidBodyHandle {
    world.spawn_body(BodyDesc {
        body_type: kind,
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
        mass_override: mass,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

/// The peak load a hanging rig settles to, after `ticks` of settling.
fn settled_load(kind: JointKind, mass: f32, drop: f32, substeps: u32, ticks: usize) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    w.set_substeps(substeps);
    let hub = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Ball { radius: 0.05 },
        None,
    );
    let load = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        -drop,
        ShapeDesc::Ball { radius: 0.2 },
        Some(mass),
    );
    let (la, lb) = w
        .world_to_local_anchors(hub, load, [0.0, 0.0], [0.0, -drop])
        .expect("anchors");
    let h = w
        .spawn_joint(
            hub,
            load,
            JointDesc {
                kind,
                anchor_a: la,
                anchor_b: lb,
                max_length: drop,
                rest_length: drop,
                ..JointDesc::default()
            },
        )
        .expect("joint");
    for _ in 0..ticks {
        w.step();
    }
    let l = w.joint_load(h).expect("load");
    (l.force, l.torque)
}

#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn measure_what_a_joint_carries() {
    println!("\n=== 1. CALIBRATION: does a hanging weight read m*g? ===");
    println!("  (a Pin, settled 120 ticks, default 4 sub-steps)\n");
    println!(
        "  {:>8}  {:>12}  {:>12}  {:>8}",
        "mass kg", "m*g N", "read N", "ratio"
    );
    for mass in [0.5_f32, 1.0, 2.0, 5.0, 10.0] {
        let (f, _) = settled_load(JointKind::Pin, mass, 1.0, 4, 120);
        let mg = mass * 9.81;
        println!("  {mass:>8.2}  {mg:>12.4}  {f:>12.4}  {:>8.4}", f / mg);
    }

    println!("\n=== 2. SUB-STEP INVARIANCE: the same rig, more sub-steps ===");
    println!("  (1 kg on a Pin — m*g = 9.81 N)\n");
    println!(
        "  {:>9}  {:>12}  {:>12}",
        "sub-steps", "force N", "torque N.m"
    );
    for subs in [1_u32, 2, 4, 8, 16] {
        let (f, t) = settled_load(JointKind::Pin, 1.0, 1.0, subs, 120);
        println!("  {subs:>9}  {f:>12.4}  {t:>12.6}");
    }
    println!("\n  -- and the OTHER divisor: solver iterations (sub-steps fixed at 4)\n");
    println!("  {:>9}  {:>12}", "iters", "force N");
    for iters in [1_usize, 2, 4, 8, 16] {
        let mut w = PhysicsWorld::new();
        w.set_solver_iterations(iters);
        let hub = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            None,
        );
        let load = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            -1.0,
            ShapeDesc::Ball { radius: 0.2 },
            Some(1.0),
        );
        let (la, lb) = w
            .world_to_local_anchors(hub, load, [0.0, 0.0], [0.0, -1.0])
            .expect("anchors");
        let h = w
            .spawn_joint(
                hub,
                load,
                JointDesc {
                    kind: JointKind::Pin,
                    anchor_a: la,
                    anchor_b: lb,
                    ..JointDesc::default()
                },
            )
            .expect("joint");
        for _ in 0..120 {
            w.step();
        }
        println!(
            "  {iters:>9}  {:>12.4}",
            w.joint_load(h).expect("load").force
        );
    }

    println!("\n=== 3. THE SCALE: what ordinary rigs read ===\n");
    println!("  {:>28}  {:>12}  {:>12}", "rig", "force N", "torque N.m");
    for (name, kind, mass, drop) in [
        ("Pin, 1 kg hanging", JointKind::Pin, 1.0_f32, 1.0_f32),
        ("Pin, 10 kg hanging", JointKind::Pin, 10.0, 1.0),
        ("Rope, 1 kg hanging", JointKind::Rope, 1.0, 1.0),
        ("Spring, 1 kg hanging", JointKind::Spring, 1.0, 1.0),
        ("Weld, 1 kg hanging", JointKind::Weld, 1.0, 1.0),
    ] {
        let (f, t) = settled_load(kind, mass, drop, 4, 180);
        println!("  {name:>28}  {f:>12.4}  {t:>12.6}");
    }
    // TORQUE needs a lever: a load hanging straight down from a pin at its own
    // centre exerts none, which is why every row above reads 0.000000.
    for (name, half_len, mass) in [
        ("Weld cantilever 1 m, 1 kg", 0.5_f32, 1.0_f32),
        ("Weld cantilever 1 m, 5 kg", 0.5, 5.0),
        ("Weld cantilever 4 m, 1 kg", 2.0, 1.0),
    ] {
        let mut w = PhysicsWorld::new();
        let wall = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            None,
        );
        let plank = body(
            &mut w,
            RigidBodyType::Dynamic,
            half_len,
            0.0,
            ShapeDesc::Cuboid {
                half_x: half_len,
                half_y: 0.05,
            },
            Some(mass),
        );
        let (la, lb) = w
            .world_to_local_anchors(wall, plank, [0.0, 0.0], [0.0, 0.0])
            .expect("anchors");
        let h = w
            .spawn_joint(
                wall,
                plank,
                JointDesc {
                    kind: JointKind::Weld,
                    anchor_a: la,
                    anchor_b: lb,
                    ..JointDesc::default()
                },
            )
            .expect("joint");
        for _ in 0..180 {
            w.step();
        }
        let l = w.joint_load(h).expect("load");
        let expect = mass * 9.81 * half_len;
        println!(
            "  {name:>28}  {:>12.4}  {:>12.6}   (m*g*r = {expect:.3} -- NOT reported)",
            l.force, l.torque
        );
    }

    println!(
        "\n  -- WHERE DOES THE ANGULAR REACTION LIVE? (two-body weld, and a hinge at its stop)\n"
    );
    for (name, wall_kind, kind, limits) in [
        (
            "Weld to STATIC wall",
            RigidBodyType::Fixed,
            JointKind::Weld,
            None,
        ),
        (
            "Weld to HEAVY dynamic wall (asleep)",
            RigidBodyType::Dynamic,
            JointKind::Weld,
            None,
        ),
        (
            "Pin at its LIMIT, static hub",
            RigidBodyType::Fixed,
            JointKind::Pin,
            Some([-0.001_f32, 0.001]),
        ),
    ] {
        let mut w = PhysicsWorld::new();
        let wall = body(
            &mut w,
            wall_kind,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.2,
                half_y: 0.2,
            },
            Some(10_000.0),
        );
        let plank = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.5,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 0.05,
            },
            Some(1.0),
        );
        let (la, lb) = w
            .world_to_local_anchors(wall, plank, [0.0, 0.0], [0.0, 0.0])
            .expect("anchors");
        let h = w
            .spawn_joint(
                wall,
                plank,
                JointDesc {
                    kind,
                    anchor_a: la,
                    anchor_b: lb,
                    limits,
                    ..JointDesc::default()
                },
            )
            .expect("joint");
        for _ in 0..180 {
            w.step();
        }
        let l = w.joint_load(h).expect("load");
        println!(
            "  {name:>30}  force {:>9.3}  torque {:>9.4}",
            l.force, l.torque
        );
    }

    // And the motor: a servo holding an arm out horizontally.
    {
        let mut w = PhysicsWorld::new();
        let hub = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            None,
        );
        let arm = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.5,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 0.05,
            },
            Some(1.0),
        );
        let (la, lb) = w
            .world_to_local_anchors(hub, arm, [0.0, 0.0], [0.0, 0.0])
            .expect("anchors");
        let h = w
            .spawn_joint(
                hub,
                arm,
                JointDesc {
                    kind: JointKind::Pin,
                    anchor_a: la,
                    anchor_b: lb,
                    motor: Some(ph2d_physics::MotorDesc {
                        mode: ph2d_physics::MotorMode::Position,
                        speed: 0.0,
                        target: 0.0,
                        max_force: 100.0,
                    }),
                    ..JointDesc::default()
                },
            )
            .expect("joint");
        for _ in 0..180 {
            w.step();
        }
        let l = w.joint_load(h).expect("load");
        println!(
            "  {:>30}  force {:>9.3}  torque {:>9.4}   (m*g*r = 4.905)",
            "SERVO holding the arm out", l.force, l.torque
        );
    }

    println!("\n  -- a YANK: a 1 kg load on a 3 m rope, free-falling N metres before it catches");
    println!("  (it starts DIRECTLY BELOW the hub, so the separation only grows -- passing");
    println!("   through the anchor makes the rope's direction degenerate and the catch is");
    println!("   absorbed silently, which is what the first version of this table measured)\n");
    println!(
        "  {:>12}  {:>12}  {:>14}  {:>12}",
        "fall m", "v at catch", "peak force N", "settled N"
    );
    for (fall, subs) in [
        (0.1_f32, 4_u32),
        (0.5, 4),
        (2.0, 4),
        (2.0, 8),
        (2.0, 16),
        (2.0, 32),
        (2.0, 64),
    ] {
        const L: f32 = 3.0;
        let mut w = PhysicsWorld::new();
        w.set_substeps(subs);
        let hub = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            None,
        );
        let y0 = -(L - fall);
        let load = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            y0,
            ShapeDesc::Ball { radius: 0.2 },
            Some(1.0),
        );
        let (la, lb) = w
            .world_to_local_anchors(hub, load, [0.0, 0.0], [0.0, y0])
            .expect("anchors");
        let h = w
            .spawn_joint(
                hub,
                load,
                JointDesc {
                    kind: JointKind::Rope,
                    anchor_a: la,
                    anchor_b: lb,
                    max_length: L,
                    ..JointDesc::default()
                },
            )
            .expect("joint");
        let mut peak = 0.0_f32;
        let mut max_sep = 0.0_f32;
        for _ in 0..240 {
            w.step();
            peak = peak.max(w.joint_load(h).expect("load").force);
            let y = w.body_pose(load).expect("pose").translation.y;
            max_sep = max_sep.max(-y);
        }
        let settled = w.joint_load(h).expect("load").force;
        let v = (2.0 * 9.81 * fall).sqrt();
        println!(
            "  {fall:>12.2}  {v:>12.2}  {peak:>14.1}  {settled:>12.4}   sub-steps {subs:>2}, max sep {max_sep:.4} (rope {L})"
        );
    }

    println!("\n=== 4. COST: the per-sub-step joint scan, at N joints ===\n");
    println!(
        "  {:>8}  {:>14}  {:>14}  {:>14}",
        "joints", "ms/step", "scan bound ms", "scan % HR-4"
    );
    for n in [0_usize, 50, 200, 500] {
        let mut w = PhysicsWorld::new();
        let mut handles = Vec::with_capacity(n);
        for i in 0..n {
            let x = (i % 25) as f32 * 0.7 - 8.0;
            let y = (i / 25) as f32 * 0.7 + 2.0;
            let hub = body(
                &mut w,
                RigidBodyType::Fixed,
                x,
                y,
                ShapeDesc::Ball { radius: 0.05 },
                None,
            );
            let load = body(
                &mut w,
                RigidBodyType::Dynamic,
                x,
                y - 0.5,
                ShapeDesc::Ball { radius: 0.15 },
                None,
            );
            let (la, lb) = w
                .world_to_local_anchors(hub, load, [x, y], [x, y - 0.5])
                .expect("anchors");
            if let Some(h) = w.spawn_joint(
                hub,
                load,
                JointDesc {
                    kind: JointKind::Pin,
                    anchor_a: la,
                    anchor_b: lb,
                    ..JointDesc::default()
                },
            ) {
                handles.push(h);
            }
        }
        for _ in 0..30 {
            w.step();
        }
        let mut samples = Vec::with_capacity(120);
        for _ in 0..120 {
            let t0 = Instant::now();
            w.step();
            samples.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        let ms = samples[samples.len() / 2];
        // An UPPER BOUND on the added work, built from an API that already
        // exists (the W-ImpactForce trick): `joint_load` does a map lookup AND a
        // joint lookup, which is strictly more per joint than the fold does, and
        // the fold runs `substeps` times per tick.
        let mut scan = Vec::with_capacity(120);
        for _ in 0..120 {
            let t0 = Instant::now();
            for _ in 0..PhysicsWorld::DEFAULT_SUBSTEPS {
                for hh in &handles {
                    std::hint::black_box(w.joint_load(*hh));
                }
            }
            scan.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        scan.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        let scan_ms = scan[scan.len() / 2];
        println!(
            "  {n:>8}  {ms:>14.4}  {scan_ms:>14.4}  {:>13.2}%",
            scan_ms / 1.5 * 100.0
        );
    }
    println!();
}
