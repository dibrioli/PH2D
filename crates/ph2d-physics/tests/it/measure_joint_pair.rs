//! **What a joint's PAIR is worth** (W-J8) — the two questions the *Higiene do
//! par* wave has to answer with numbers rather than with an opinion:
//!
//! 1. **Collide Connected** — how much does it actually change? (It has been
//!    hardcoded `false` since W3 on the strength of one measurement; exposing it
//!    as a knob means the ON side has to be a thing someone would want.)
//! 2. **Swap A↔B** — which of a joint's quantities are measured *between* the
//!    two bodies, and therefore change meaning when the pair is exchanged?
//!
//! The second decides the wave's design: if swapping silently reverses a motor
//! and mirrors a limit range, then a swap either has to compensate or has to be
//! documented as a thing that re-aims the joint. Guessing would ship one of those
//! two as a surprise.
//!
//! ## What it measured
//!
//! **Collide Connected.** A hub pinned inside the plank it drives, told 4 rad/s:
//! OFF reads `−3.883 / 0.117` (relative 4.000, the motor keeping its word), ON
//! reads `0.392 / 0.392` — a relative speed of **zero**, the motor completely
//! defeated by the interpenetration it is pinned into. And the case the ON side
//! exists FOR: a crate roped to a static block rests **on** it at `y = 0.899`
//! with contacts on, and falls straight **through** it to the rope's full 4 m
//! (`y = −4.000`) with them off. Both sides are a thing someone wants; the
//! default stays OFF because the chain link is the common case.
//!
//! **Swap A↔B.** The same rig, pair exchanged, and the third column negates
//! every signed quantity measured between the bodies:
//!
//! | quantity | authored | bare swap | compensated |
//! |---|---|---|---|
//! | pin: load y | −1.0000 | −1.0000 | −1.0000 |
//! | rope: load y | −2.0000 | −2.0000 | −2.0000 |
//! | **motor: wheel ω** | 4.0000 | **−4.0000** | 4.0000 |
//! | **servo: wheel rot** | 44.9998° | **−44.9998°** | 44.9998° |
//! | **limit: plank rot** | −11.4592° | **−34.3775°** | −11.4592° |
//! | **slider: carriage y** | −0.3000 | **−1.2000** | −0.3000 |
//!
//! So a BARE swap reverses the motor, the servo target and mirrors the limit
//! range (`[min, max] → [−max, −min]`; the plank's `[−0.2, 0.6]` rad becomes
//! `[−0.6, 0.2]` and it settles at the other end). The compensation reproduces
//! the authored column **to four decimals in every row**, which is what decided
//! the design: *a swap changes which end is called A, and nothing else.*
//!
//! `#[ignore]`d — a harness, not a gate. Run with
//! `cargo test -p ph2d-physics --test measure_joint_pair -- --ignored --nocapture
//! --test-threads=1` (the three tables interleave otherwise).

use ph2d_physics::{BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, ShapeDesc};

type Handle = ph2d_physics::RigidBodyHandle;

fn body(
    world: &mut PhysicsWorld,
    kind: ph2d_physics::RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
) -> Handle {
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
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

fn dynamic(world: &mut PhysicsWorld, x: f32, y: f32, shape: ShapeDesc) -> Handle {
    body(world, ph2d_physics::RigidBodyType::Dynamic, x, y, shape)
}

fn fixed(world: &mut PhysicsWorld, x: f32, y: f32) -> Handle {
    fixed_shape(world, x, y, ShapeDesc::Ball { radius: 0.05 })
}

fn fixed_shape(world: &mut PhysicsWorld, x: f32, y: f32, shape: ShapeDesc) -> Handle {
    body(world, ph2d_physics::RigidBodyType::Fixed, x, y, shape)
}

/// Angular velocity of one body, read out of the snapshot list. There is no
/// direct accessor, and the RATE is what a spin has to be measured by — a
/// rotation angle wraps at +-pi and stops meaning anything past one turn.
fn angvel(world: &PhysicsWorld, h: Handle) -> f32 {
    world
        .body_snapshots()
        .into_iter()
        .find(|s| s.handle_index == h.into_raw_parts().0)
        .map(|s| s.angvel)
        .unwrap_or(0.0)
}

/// Spawn `desc` between `a` and `b`, or the **swapped** pair — which means
/// exchanging the bodies AND everything the descriptor states per-body (the two
/// anchors, the two axes). Nothing else is touched, so what this measures is
/// exactly *what a bare swap does*.
/// `0` = as authored · `1` = BARE swap (pair + the per-body fields, nothing
/// else) · `2` = swap with every signed between-the-bodies quantity negated.
fn spawn_mode(world: &mut PhysicsWorld, a: Handle, b: Handle, desc: JointDesc, mode: u8) {
    if mode == 0 {
        world.spawn_joint(a, b, desc).expect("joint");
        return;
    }
    let mut d = JointDesc {
        anchor_a: desc.anchor_b,
        anchor_b: desc.anchor_a,
        axis_a: desc.axis_b,
        axis_b: desc.axis_a,
        ..desc
    };
    if mode == 2 {
        // The free degree of freedom is measured FROM A TO B, so exchanging the
        // pair negates it: a range mirrors and a motor reverses.
        d.limits = d.limits.map(|[lo, hi]| [-hi, -lo]);
        d.motor = d.motor.map(|m| MotorDesc {
            speed: -m.speed,
            target: -m.target,
            ..m
        });
    }
    world.spawn_joint(b, a, d).expect("joint");
}

fn steps(world: &mut PhysicsWorld, n: usize) {
    for _ in 0..n {
        world.step();
    }
}

/// **A hub pinned INSIDE a plank, told to spin.** The rig that found the
/// `contacts_enabled = false` default in W3; re-run here so the ON side of the
/// new knob has a number of its own.
#[test]
#[ignore = "measurement harness"]
fn collide_connected_costs_this_much() {
    println!("\n== Collide Connected: a hub pinned inside the plank it drives ==");
    println!("{:>10} | {:>12} | {:>12}", "contacts", "hub w", "plank w");
    for contacts in [false, true] {
        let mut world = PhysicsWorld::new();
        world.set_gravity(0.0, 0.0);
        let hub = dynamic(&mut world, 0.0, 0.0, ShapeDesc::Ball { radius: 0.25 });
        let plank = dynamic(
            &mut world,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 0.15,
            },
        );
        spawn_mode(
            &mut world,
            hub,
            plank,
            JointDesc {
                kind: JointKind::Pin,
                motor: Some(MotorDesc {
                    mode: MotorMode::Velocity,
                    speed: 4.0,
                    target: 0.0,
                    max_force: 100.0,
                }),
                contacts_enabled: contacts,
                ..Default::default()
            },
            0,
        );
        steps(&mut world, 120);
        println!(
            "{contacts:>10} | {:>12.3} | {:>12.3}",
            angvel(&world, hub),
            angvel(&world, plank)
        );
    }
    println!("(told 4 rad/s; the ON row is the interpenetration the solver fights)");
}

/// **Two bodies pinned side by side, one falling onto the other.** The case the
/// ON side exists FOR: a pair that is jointed and still has to bump.
#[test]
#[ignore = "measurement harness"]
fn collide_connected_is_what_makes_a_jointed_pair_bump() {
    println!("\n== Collide Connected: a rope-hung crate resting ON its anchor body ==");
    println!("{:>10} | {:>12}", "contacts", "crate y");
    for contacts in [false, true] {
        let mut world = PhysicsWorld::new();
        let base = fixed_shape(
            &mut world,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 0.5,
            },
        );
        let crated = dynamic(
            &mut world,
            0.0,
            2.0,
            ShapeDesc::Cuboid {
                half_x: 0.4,
                half_y: 0.4,
            },
        );
        spawn_mode(
            &mut world,
            base,
            crated,
            JointDesc {
                kind: JointKind::Rope,
                max_length: 4.0,
                contacts_enabled: contacts,
                ..Default::default()
            },
            0,
        );
        steps(&mut world, 240);
        println!(
            "{contacts:>10} | {:>12.3}",
            world
                .body_pose(crated)
                .map(|p| p.translation.y)
                .unwrap_or(0.0)
        );
    }
    println!("(OFF: the crate falls THROUGH the block it is roped to, down to the rope's 4 m)");
}

/// **The whole swap question, one row per quantity.** Same rig twice, the second
/// with the pair exchanged; anything that differs is measured *between* the
/// bodies and therefore changes meaning with the pair.
#[test]
#[ignore = "measurement harness"]
fn what_a_bare_swap_changes() {
    println!("\n== Swap A<->B: same rig, pair exchanged ==");
    println!(
        "{:>22} | {:>12} | {:>12} | {:>12}",
        "quantity", "A=post", "bare swap", "compensated"
    );

    let row = |name: &str, f: &dyn Fn(u8) -> f32| {
        println!(
            "{name:>22} | {:>12.4} | {:>12.4} | {:>12.4}",
            f(0),
            f(1),
            f(2)
        );
    };

    row("pin: load y", &|mode| {
        let mut world = PhysicsWorld::new();
        let post = fixed(&mut world, 0.0, 0.0);
        let load = dynamic(&mut world, 0.0, -1.0, ShapeDesc::Ball { radius: 0.2 });
        spawn_mode(
            &mut world,
            post,
            load,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: [0.0, 0.0],
                anchor_b: [0.0, 1.0],
                ..Default::default()
            },
            mode,
        );
        steps(&mut world, 120);
        world
            .body_pose(load)
            .map(|p| p.translation.y)
            .unwrap_or(0.0)
    });

    row("rope: load y", &|mode| {
        let mut world = PhysicsWorld::new();
        let post = fixed(&mut world, 0.0, 0.0);
        let load = dynamic(&mut world, 0.0, -1.0, ShapeDesc::Ball { radius: 0.2 });
        spawn_mode(
            &mut world,
            post,
            load,
            JointDesc {
                kind: JointKind::Rope,
                max_length: 2.0,
                ..Default::default()
            },
            mode,
        );
        steps(&mut world, 240);
        world
            .body_pose(load)
            .map(|p| p.translation.y)
            .unwrap_or(0.0)
    });

    // 2. A motorised wheel: a disc pinned at its centre to a static hub.
    row("motor: wheel w", &|mode| {
        let mut world = PhysicsWorld::new();
        world.set_gravity(0.0, 0.0);
        let hub = fixed(&mut world, 0.0, 0.0);
        let wheel = dynamic(&mut world, 0.0, 0.0, ShapeDesc::Ball { radius: 0.5 });
        spawn_mode(
            &mut world,
            hub,
            wheel,
            JointDesc {
                kind: JointKind::Pin,
                motor: Some(MotorDesc {
                    mode: MotorMode::Velocity,
                    speed: 4.0,
                    target: 0.0,
                    max_force: 100.0,
                }),
                ..Default::default()
            },
            mode,
        );
        steps(&mut world, 120);
        angvel(&world, wheel)
    });

    // 3. A servo told to hold +45 degrees.
    row("servo: wheel rot", &|mode| {
        let mut world = PhysicsWorld::new();
        world.set_gravity(0.0, 0.0);
        let hub = fixed(&mut world, 0.0, 0.0);
        let wheel = dynamic(&mut world, 0.0, 0.0, ShapeDesc::Ball { radius: 0.5 });
        spawn_mode(
            &mut world,
            hub,
            wheel,
            JointDesc {
                kind: JointKind::Pin,
                motor: Some(MotorDesc {
                    mode: MotorMode::Position,
                    speed: 0.0,
                    target: std::f32::consts::FRAC_PI_4,
                    max_force: 100.0,
                }),
                ..Default::default()
            },
            mode,
        );
        steps(&mut world, 240);
        world
            .body_pose(wheel)
            .map(|p| p.rotation.angle().to_degrees())
            .unwrap_or(0.0)
    });

    // 4. An asymmetric limit range: a plank pinned at one end, free to swing
    //    only ONE way. Gravity decides which way it tries.
    row("limit: plank rot", &|mode| {
        let mut world = PhysicsWorld::new();
        let post = fixed(&mut world, 0.0, 0.0);
        let plank = dynamic(
            &mut world,
            1.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 0.1,
            },
        );
        spawn_mode(
            &mut world,
            post,
            plank,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: [0.0, 0.0],
                anchor_b: [-1.0, 0.0],
                limits: Some([-0.2, 0.6]),
                ..Default::default()
            },
            mode,
        );
        steps(&mut world, 240);
        world
            .body_pose(plank)
            .map(|p| p.rotation.angle().to_degrees())
            .unwrap_or(0.0)
    });

    // 5. A slider with an ASYMMETRIC stroke on a vertical rail: gravity pulls
    //    the carriage to whichever end the range allows.
    row("slider: carriage y", &|mode| {
        let mut world = PhysicsWorld::new();
        let post = fixed(&mut world, 0.0, 0.0);
        let car = dynamic(
            &mut world,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
        );
        spawn_mode(
            &mut world,
            post,
            car,
            JointDesc {
                kind: JointKind::Slider,
                axis_a: [0.0, 1.0],
                axis_b: [0.0, 1.0],
                limits: Some([-0.3, 1.2]),
                ..Default::default()
            },
            mode,
        );
        steps(&mut world, 240);
        world.body_pose(car).map(|p| p.translation.y).unwrap_or(0.0)
    });

    println!("(a row whose two columns differ is measured BETWEEN the bodies)");
}
