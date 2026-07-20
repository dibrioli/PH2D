//! Where the W2b slider ceilings COME FROM (CLAUDE.md §0.0: measure, then write
//! the number the measurement gave, with the table beside it).
//!
//! Three of the world knobs bound a real resource and therefore cannot be
//! guessed:
//!
//! * **substeps** and **solver iterations** cost CPU, against HR-4's 1.5 ms;
//! * **contact frequency** bounds solver stability — and the obvious hypothesis
//!   (a Nyquist limit at `1/(2·substep_dt)` = 120 Hz, since the contact spring
//!   is integrated at the sub-step) was **REFUTED** by the probe below: rapier's
//!   soft-constraint formulation drifts exactly zero at 120, 240, 480 and
//!   960 Hz, and only moves at 1920. The ceiling shipped in
//!   `ph2d-physics-ecs::settings` is the measured one, not the derived one.
//!
//! Run with output:
//! ```text
//! cargo test -p ph2d-physics --release --test measure_settings -- --nocapture --ignored
//! ```
//! `--release` is not a preference: `ci-test`/debug builds measure the PROFILE,
//! not the product (the audio line's ADR-0124 pins the same rule).

use ph2d_physics::{BodyDefaults, PhysicsWorld};
use std::time::Instant;

/// The stress scene the HR-4 budget is quoted against: 500 dynamic bodies on a
/// floor — the count the W2a penetration work already used, so these numbers
/// are comparable to the ones in the tracker.
///
/// ⚠️ **Sleep is turned OFF here, and the first run of this harness is why.**
/// With rapier's defaults a settled stack falls asleep, and a sleeping body is
/// not integrated at all — so the jitter probe below read `0.0000 mm` at every
/// frequency including 1920 Hz, and the cost table was timing a stack that had
/// stopped being simulated. Both numbers were guaranteed rather than observed
/// ("zero does not fail unless you make it fail"). Awake is also the honest
/// worst case for a budget.
fn stack_of(n: usize) -> PhysicsWorld {
    let mut w = PhysicsWorld::new();
    w.set_body_defaults(BodyDefaults {
        sleep_linear_threshold: 0.0,
        sleep_angular_threshold: 0.0,
        time_until_sleep: f32::MAX,
        ..BodyDefaults::rapier()
    });
    w.add_static_cuboid(0.0, 0.0, 200.0, 0.1);
    for i in 0..n {
        let row = (i / 25) as f32;
        let col = (i % 25) as f32;
        w.add_dynamic_circle(col * 0.6 - 7.2, 0.5 + row * 0.6, 0.25, 1.0);
    }
    w
}

/// Median wall-clock of one `step()` over `ticks`, after a warm-up.
fn cost_per_step(w: &mut PhysicsWorld, ticks: usize) -> f64 {
    for _ in 0..30 {
        w.step();
    }
    let mut samples: Vec<f64> = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        let t0 = Instant::now();
        w.step();
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn measure_the_cost_of_the_solver_knobs() {
    const BODIES: usize = 500;
    const TICKS: usize = 120;

    println!("\n=== substeps: cost of one tick, {BODIES} bodies (HR-4 budget: 1.5 ms) ===");
    for n in [1u32, 2, 4, 8, 12, 16, 24, 32] {
        let mut w = stack_of(BODIES);
        w.set_substeps(n);
        let ms = cost_per_step(&mut w, TICKS);
        println!(
            "  substeps {n:>3} -> {ms:>7.3} ms/tick   ({:>5.1}% of HR-4)",
            ms / 1.5 * 100.0
        );
    }

    println!("\n=== solver iterations: cost of one tick, {BODIES} bodies ===");
    for n in [1usize, 2, 4, 8, 12, 16, 24, 32] {
        let mut w = stack_of(BODIES);
        w.set_solver_iterations(n);
        let ms = cost_per_step(&mut w, TICKS);
        println!(
            "  iterations {n:>3} -> {ms:>7.3} ms/tick   ({:>5.1}% of HR-4)",
            ms / 1.5 * 100.0
        );
    }
}

/// Damping bounds no resource — it bounds MEANING. rapier scales velocity by
/// `1/(1 + damping·dt)` per sub-step, so the question a ceiling has to answer is
/// "past which value is the knob only choosing between shades of *instantly
/// stopped*?". This prints the fraction of its speed a body keeps after one
/// second at each value.
#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn measure_where_damping_stops_meaning_anything() {
    println!("\n=== linear damping: speed retained after 1 s of falling ===");
    for d in [0.0f32, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0] {
        let mut w = PhysicsWorld::new();
        w.set_body_defaults(BodyDefaults {
            linear_damping: d,
            ..BodyDefaults::rapier()
        });
        let h = w.spawn_body(ph2d_physics::BodyDesc {
            body_type: ph2d_physics::RigidBodyType::Dynamic,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            density: 1.0,
            shape: ph2d_physics::ShapeDesc::Ball { radius: 0.25 },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
        });
        for _ in 0..60 {
            w.step();
        }
        // Terminal speed under gravity with this damping, vs. undamped 9.81 m/s.
        let v = w.bodies().get(h).unwrap().linvel().norm();
        println!(
            "  damping {d:>5.1} -> speed after 1 s {v:>7.4} m/s  ({:>6.2}% of free fall)",
            v / 9.81 * 100.0
        );
    }
}

/// Where contact frequency actually stops being trustworthy.
///
/// The hypothesis going in: the contact spring is integrated at `substep_dt`,
/// so beyond `1/(2·substep_dt)` (120 Hz at our defaults) a higher number cannot
/// describe a stiffer contact — it describes an oscillation the integrator
/// cannot see, which rapier's own docs warn shows up as "jitter due to
/// overshooting".
///
/// **Measured, that is wrong.** A stack forbidden to sleep drifts *exactly*
/// 0.0000 mm at 30, 60, 120, 240, 480 and 960 Hz; the first motion is at
/// 1920 Hz, and it is 0.114 mm (0.011 px at 100 px/m). rapier's soft
/// constraints are stable a long way past Nyquist, so the shipped ceiling is
/// derived from THIS table and not from the formula.
///
/// This prints the residual motion of a stack that should be dead still.
#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn measure_where_contact_frequency_starts_to_jitter() {
    const BODIES: usize = 60;
    let substeps = PhysicsWorld::DEFAULT_SUBSTEPS;
    let nyquist = 1.0 / (2.0 * (PhysicsWorld::DEFAULT_DT / substeps as f32));
    println!(
        "\n=== contact frequency, {BODIES} bodies, substeps {substeps} \
         (substep_dt = {:.5} s, Nyquist = {nyquist:.0} Hz) ===",
        PhysicsWorld::DEFAULT_DT / substeps as f32
    );
    println!("  settled stacks must be motionless; residual motion IS the artifact\n");

    for hz in [30.0f32, 60.0, 120.0, 240.0, 480.0, 960.0, 1920.0] {
        let mut w = stack_of(BODIES);
        w.set_contact_frequency(hz);
        // Let it settle completely.
        for _ in 0..600 {
            w.step();
        }
        let before: Vec<(f32, f32)> = w.body_snapshots().iter().map(|s| (s.x, s.y)).collect();
        // 60 more ticks of a stack that is supposed to be asleep on the floor.
        for _ in 0..60 {
            w.step();
        }
        let after: Vec<(f32, f32)> = w.body_snapshots().iter().map(|s| (s.x, s.y)).collect();
        let drift: f32 = before
            .iter()
            .zip(&after)
            .map(|(a, b)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt())
            .fold(0.0f32, f32::max);
        // In millimetres, and in pixels at the project default of 100 px/m.
        println!(
            "  {hz:>7.0} Hz -> worst drift {:>8.4} mm  ({:>7.4} px @100 px/m){}",
            drift * 1000.0,
            drift * 100.0,
            if hz > nyquist {
                "   [past Nyquist]"
            } else {
                ""
            }
        );
    }
}
