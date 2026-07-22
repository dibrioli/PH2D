//! **Does capturing the impact PEAK inside the sub-step loop cost enough to
//! gate?** (ADR-0131 W-ImpactForce, the front-B cure the W-Contacts /
//! W-ContactEvents notes name.)
//!
//! `contact_reports().impulse` is the LOAD a pair carries at tick-end, not the
//! peak of the hit: `step` returns after the solver has already stopped the
//! body, so the impact lives *between* the sub-steps and is gone. The cure is a
//! per-sub-step `max`, and the standing law (CLAUDE.md §0.0) is **measure the
//! cost before you commit to it**, because it is paid by every scene for a
//! reading only some scenes want.
//!
//! This harness measures the cost WITHOUT writing the capture yet, from an
//! upper bound built out of APIs that already exist: the added work per tick is
//! `substeps` scans of the contact graph computing an impulse and a `max` into a
//! map — and `contact_reports()` already does **strictly more** per pair than
//! that (it also allocates, sorts, and maps handles to a plain struct). So
//! `substeps × contact_reports()` over-estimates the capture, and if THAT is a
//! small fraction of `step()` the capture is cheap and belongs always-on.
//!
//! Run with output:
//! ```text
//! cargo test -p ph2d-physics --release --test measure_impact -- --nocapture --ignored
//! ```
//! `--release` is not a preference (ADR-0124): a `ci-test`/debug build measures
//! the profile, not the product.

use ph2d_physics::{BodyDefaults, BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};
use std::time::Instant;

fn ball_desc(x: f32, y: f32, r: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: r },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A settled PILE — the worst case for contact density, which is exactly where
/// the per-sub-step scan costs the most. 500 balls dropped into a heap over a
/// floor, sleep OFF so every body stays integrated (the same correction the
/// `measure_settings` harness had to make: a sleeping stack reads a guaranteed
/// zero, not an observed one).
fn pile_of(n: usize) -> PhysicsWorld {
    let mut w = PhysicsWorld::new();
    w.set_body_defaults(BodyDefaults {
        sleep_linear_threshold: 0.0,
        sleep_angular_threshold: 0.0,
        time_until_sleep: f32::MAX,
        ..BodyDefaults::rapier()
    });
    w.add_static_cuboid(0.0, 0.0, 6.0, 0.1);
    // Pack them into a narrow column so they settle into a many-contact heap
    // rather than a flat one-deep carpet.
    for i in 0..n {
        let row = (i / 12) as f32;
        let col = (i % 12) as f32;
        w.add_dynamic_circle(col * 0.55 - 3.0, 0.5 + row * 0.55, 0.25, 1.0);
    }
    w
}

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

fn cost_per_step(w: &mut PhysicsWorld, ticks: usize) -> f64 {
    for _ in 0..30 {
        w.step();
    }
    let mut samples = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        let t0 = Instant::now();
        w.step();
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    median(samples)
}

fn cost_per_scan(w: &PhysicsWorld, calls: usize) -> (f64, usize) {
    // Warm the settled contact graph.
    let pairs = w.contact_reports().len();
    let mut samples = Vec::with_capacity(calls);
    for _ in 0..calls {
        let t0 = Instant::now();
        let r = w.contact_reports();
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
        std::hint::black_box(&r);
    }
    (median(samples), pairs)
}

#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn measure_the_cost_of_capturing_the_impact_peak() {
    const BODIES: usize = 500;
    const TICKS: usize = 120;
    const SUBSTEPS: u32 = PhysicsWorld::DEFAULT_SUBSTEPS;

    // Settle the pile first so the contact graph is dense (that is the case that
    // costs), then measure both halves.
    let mut w = pile_of(BODIES);
    for _ in 0..240 {
        w.step();
    }

    let step_ms = cost_per_step(&mut w, TICKS);
    let (scan_ms, pairs) = cost_per_scan(&w, TICKS);
    // The capture is bounded above by one scan per sub-step, per tick.
    let capture_ub_ms = scan_ms * SUBSTEPS as f64;

    println!("\n=== impact-peak capture cost, {BODIES} bodies (settled pile) ===");
    println!("  active contact pairs .......... {pairs}");
    println!(
        "  step()          ............... {step_ms:>8.4} ms/tick   ({:>5.1}% of HR-4 1.5 ms)",
        step_ms / 1.5 * 100.0
    );
    println!("  contact_reports() (one scan) .. {scan_ms:>8.4} ms/call");
    println!("  capture upper bound (x{SUBSTEPS}) .. {capture_ub_ms:>8.4} ms/tick");
    println!(
        "  => overhead <= {:>5.1}% of step() ({:>5.2}% of HR-4)",
        capture_ub_ms / step_ms * 100.0,
        capture_ub_ms / 1.5 * 100.0
    );
    println!("  (contact_reports does MORE per pair than the capture: alloc + sort + handle map,");
    println!("   so the real capture cost is under this bound.)");
}

/// Drop one ball from a height and report both numbers on the tick it lands: the
/// settled LOAD (`impulse`) and the captured IMPACT peak (`impact`). This is what
/// picks the overlay's impact ruler and scene 29's numbers — and it proves the
/// capture does what the load meter cannot: grow with how hard the hit was.
fn land_one_ball(drop_y: f32) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 4.0, 0.1);
    w.add_dynamic_circle(0.0, drop_y, 0.3, 1.0);
    // Step until it first touches, then take the peak/load on that landing tick.
    let mut best_impact = 0.0f32;
    let mut best_load = 0.0f32;
    for _ in 0..600 {
        w.step();
        // The landing tick is the first with a real load; capture its peak.
        if let Some(r) = w.contact_reports().first()
            && r.impact > best_impact
        {
            best_impact = r.impact;
            best_load = r.impulse;
        }
    }
    (best_load, best_impact)
}

/// **Why the gates read the peak from a GRAZING tick and not a landing tick.** The
/// hard finding of this wave: at the world level `impulse` is the load at tick-end, and
/// for a pair still in contact then it EQUALS the sub-step peak — the body is caught
/// hardest at the boundary. So on a landing tick `impact` and `impulse` coincide, and a
/// gate that reads one landing tick's endpoint cannot tell the max-over-sub-steps from
/// the last sub-step. The gap shows only on a grazing tick, which needs the body to lift
/// off before the last sub-step — a phase a THIN floor produces inside `contact_reports`.
///
/// This probe prints all three cases side by side so the fixture choice is legible.
#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn debug_the_peak_shows_only_on_a_grazing_tick() {
    // (1) A dead ball settling: the last sub-step is the WEIGHT, so a settled tick has
    // load << peak — but only AFTER the landing tick, not on it.
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 4.0, 0.1);
    w.add_dynamic_circle(0.0, 6.0, 0.3, 1.0);
    println!("\n=== dead ball, 6 m: the landing tick then the settle ===");
    let mut n = 0;
    for tick in 0..300 {
        w.step();
        if let Some(r) = w.contact_reports().first()
            && n < 3
        {
            println!(
                "  tick {tick:>3}  load {:.6}  impact {:.6}",
                r.impulse, r.impact
            );
            n += 1;
        }
    }

    // (2) The desc path (what the ECS bridge uses) on a thick floor: the landing tick's
    // last sub-step IS the peak, so load == impact — the coincidence a gate must avoid.
    let mut w2 = PhysicsWorld::new();
    w2.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        shape: ShapeDesc::Cuboid {
            half_x: 10.0,
            half_y: 0.5,
        },
        ..ball_desc(0.0, -0.5, 0.0)
    });
    w2.spawn_body(ball_desc(0.0, 6.0, 0.3));
    println!("\n=== desc path, 6 m: landing tick load == impact (the trap) ===");
    for tick in 0..120 {
        w2.step();
        if let Some(r) = w2.contact_reports().first() {
            println!(
                "  tick {tick:>3}  load {:.6}  impact {:.6}",
                r.impulse, r.impact
            );
            break;
        }
    }

    // (3) A bouncy ball on a THIN floor (top at y=0): the grazing ticks land inside
    // `contact_reports` with endpoint ~0 and peak large — what the wrapper gate reads.
    let mut w3 = PhysicsWorld::new();
    w3.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        shape: ShapeDesc::Cuboid {
            half_x: 4.0,
            half_y: 0.2,
        },
        ..ball_desc(0.0, -0.2, 0.0)
    });
    w3.spawn_body(BodyDesc {
        restitution: 0.75,
        ..ball_desc(0.0, 1.2, 0.3)
    });
    println!("\n=== bouncy, thin floor: the grazing ticks (peak >> endpoint) ===");
    for tick in 0..200 {
        w3.step();
        if let Some(r) = w3.contact_reports().first()
            && r.impact > 0.1
            && r.impulse < r.impact * 0.5
        {
            println!(
                "  tick {tick:>3}  load {:.6}  impact {:.6}  <- grazing",
                r.impulse, r.impact
            );
        }
    }
}

#[test]
#[ignore = "measurement harness — run explicitly with --release --nocapture"]
fn measure_the_impact_peak_grows_with_drop_height() {
    println!("\n=== impact peak vs settled load, ball r=0.3 dropped onto a floor ===");
    println!("  the LOAD is what a load meter reads; the IMPACT is what a hit sound wants");
    println!(
        "  {:>8}   {:>12}   {:>12}   {:>7}",
        "drop m", "load N.s", "impact N.s", "x load"
    );
    for drop_y in [0.6f32, 1.2, 2.0, 3.4, 6.0, 10.0] {
        let (load, impact) = land_one_ball(drop_y);
        let ratio = if load > 1e-9 { impact / load } else { f32::NAN };
        println!("  {drop_y:>8.1}   {load:>12.6}   {impact:>12.6}   {ratio:>6.1}x");
    }
}
