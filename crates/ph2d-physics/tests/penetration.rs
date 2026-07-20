//! **How far does a landing body sink into the floor?**
//!
//! Reported by Enio, 2026-07-18: *"observa-se alguma interpenetração dos
//! objetos dinâmicos com o chão"*. Measuring first separated two things that
//! look alike and are not:
//!
//! - **at rest, ~1.3 mm** — rapier's `normalized_allowed_linear_error`, 1 mm
//!   by design. At the editor's ~100 px/m that is 0.13 px. Not what anyone
//!   saw, and not worth chasing.
//! - **at impact, 83 mm for 9 frames** — a body landing at 9.4 m/s travels
//!   157 mm per 60 Hz tick, so the tick it first touches it is *already*
//!   deep inside. ~8 px on screen for 0.15 s. That is the report.
//!
//! And the depth is **not a solver failure**: contact damping, the
//! corrective-velocity ceiling, extra solver iterations and CCD were each
//! measured and every one left it at exactly 83.2 mm. It is `velocity × dt`,
//! so the only lever is a smaller `dt` (sub-steps), and the only lever on how
//! long it lasts is the contact spring frequency.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

const FLOOR_TOP: f32 = -0.8;
const HALF: f32 = 0.28;

/// Roughly one screen pixel at the editor's default zoom (~100 px/m). The
/// bar is what the ARTIST can see, not a number that flatters the solver.
const VISIBLE_M: f32 = 0.01;

/// Drop one body from `drop_y` onto a floor whose top is at [`FLOOR_TOP`].
/// Returns `(worst penetration, frames it stayed visible, resting depth)`.
fn drop_probe(world: &mut PhysicsWorld, drop_y: f32) -> (f32, u32, f32) {
    world.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: -1.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 4.0,
            half_y: 0.2,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
    });
    let h = world.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: drop_y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: HALF,
            half_y: HALF,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
    });
    let (mut worst, mut frames) = (0.0f32, 0u32);
    for _ in 0..400 {
        world.step();
        let pen = FLOOR_TOP - (world.body_pose(h).unwrap().translation.y - HALF);
        if pen > worst {
            worst = pen;
        }
        if pen > VISIBLE_M {
            frames += 1;
        }
    }
    let rest = FLOOR_TOP - (world.body_pose(h).unwrap().translation.y - HALF);
    (worst, frames, rest)
}

/// **The gate.** At every drop height the smoke scenes actually use, a body
/// may spend at most ONE frame visibly inside the floor.
///
/// Mutation-tested: dropping `DEFAULT_SUBSTEPS` back to 1 takes the depth to
/// 83 mm, and dropping `DEFAULT_CONTACT_HZ` back to rapier's 30 stretches the
/// recovery to 6+ frames. Either one makes this red — which is the point:
/// the two constants fix two different halves of the same artifact, so each
/// needs to be caught on its own ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn a_landing_body_is_never_visibly_inside_the_floor_for_more_than_a_frame() {
    for drop_y in [0.0f32, 1.6, 2.7, 4.0] {
        let mut w = PhysicsWorld::new();
        let (worst, frames, rest) = drop_probe(&mut w, drop_y);
        assert!(
            frames <= 1,
            "dropped from y={drop_y}: the body was visibly inside the floor for {frames} frames \
             ({:.1} mm at worst) — the artist sees it sink and pop back out",
            worst * 1000.0
        );
        assert!(
            worst < 0.035,
            "dropped from y={drop_y}: sank {:.1} mm into the floor. Sub-stepping is the only \
             lever on this depth (it is velocity x dt, not a solver failure)",
            worst * 1000.0
        );
        assert!(
            rest < 0.003,
            "dropped from y={drop_y}: resting {:.2} mm inside the floor; rapier's designed slop \
             is 1 mm",
            rest * 1000.0
        );
    }
}

/// The fix must not be bought with jitter — a settled pile that trembles is
/// a worse artifact than one that sinks. Measured at zero before and after.
#[test]
fn a_settled_pile_is_completely_still() {
    let mut w = PhysicsWorld::new();
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: -1.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 50.0,
            half_y: 0.2,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
    });
    for i in 0..30 {
        w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: (i % 6) as f32 * 0.6 - 1.5,
            y: (i / 6) as f32 * 0.6,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Ball { radius: 0.25 },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
        });
    }
    for _ in 0..600 {
        w.step();
    }
    let mut prev: Vec<f32> = w.body_snapshots().iter().map(|s| s.y).collect();
    let mut worst = 0.0f32;
    for _ in 0..90 {
        w.step();
        let now: Vec<f32> = w.body_snapshots().iter().map(|s| s.y).collect();
        for (a, b) in prev.iter().zip(&now) {
            worst = worst.max((a - b).abs());
        }
        prev = now;
    }
    assert!(
        worst < 1e-4,
        "a settled pile is still moving {:.5} mm/tick — the stiffer contacts bought the \
         penetration back as jitter",
        worst * 1000.0
    );
}

/// Sub-stepping must stay inside HR-4's 1.5 ms physics frame at a body count
/// a 2D game actually ships. A RATIO against the single-step cost, not a
/// wall-clock bar: `ci-test` builds at `opt-level=1` and a stopwatch there
/// measures the profile, not the code.
#[test]
fn sub_stepping_costs_what_it_says_it_costs() {
    let build = |substeps: u32| {
        let mut w = PhysicsWorld::new();
        w.set_substeps(substeps);
        w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Fixed,
            x: 0.0,
            y: -1.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Cuboid {
                half_x: 50.0,
                half_y: 0.2,
            },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
        });
        for i in 0..200 {
            w.spawn_body(BodyDesc {
                body_type: RigidBodyType::Dynamic,
                x: (i % 20) as f32 * 0.6 - 5.7,
                y: (i / 20) as f32 * 0.6,
                rotation: 0.0,
                density: 1.0,
                shape: ShapeDesc::Ball { radius: 0.25 },
                restitution: 0.0,
                friction: 0.5,
                layer: 0,
                is_sensor: false,
                gravity_scale: 1.0,
                linvel: [0.0, 0.0],
                angvel: 0.0,
                ccd: false,
                lock_rotation: false,
            });
        }
        w
    };
    let time = |substeps: u32| {
        let mut w = build(substeps);
        for _ in 0..120 {
            w.step();
        }
        let t = std::time::Instant::now();
        for _ in 0..100 {
            w.step();
        }
        t.elapsed().as_nanos() as f64 / 100.0
    };
    let one = time(1);
    let four = time(PhysicsWorld::DEFAULT_SUBSTEPS);
    let ratio = four / one;
    println!(
        "200 bodies: 1 substep {:.1} us, 4 substeps {:.1} us ({ratio:.2}x)",
        one / 1000.0,
        four / 1000.0
    );
    assert!(
        ratio < 6.0,
        "4 sub-steps cost {ratio:.2}x a single step — more than the ~4x they are, so something \
         beyond the integration is being repeated per sub-step"
    );
}
