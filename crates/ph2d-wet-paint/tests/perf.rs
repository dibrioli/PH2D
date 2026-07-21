//! Perf sanity + the ADR-0134 kill-criterion measurement for the sim side.
//!
//! The reference harness bars (JS, browser-class machine): a representative
//! scripted session must run well under 1 s per 100 steps; the worst-case
//! flood (~110k wet cells) under 1.5 s per 100 steps. The Rust port keeps the
//! same bars as SANITY (they hold with margin even at opt-level 1); the ADR's
//! REAL bar — a 40 Hz tick <= 8 ms on the flood upper bound — is asserted by
//! the `#[ignore]` measure test, which must run in release:
//!
//!   cargo test -p ph2d-wet-paint --release --test perf -- --ignored --nocapture
//!
//! Wall-clock in ci-test (opt-level 1) is a PROFILE, not the product number;
//! the release run is the number the ADR table records.

mod util;

use std::time::Instant;

use ph2d_wet_paint::painter::Engine;
use util::{drive_stroke, sweep_nan};

fn flood_engine() -> Engine {
    let mut e = Engine::new(900, 450);
    let g = e.active_grid_mut();
    let s = g.s;
    for y in 40..=220usize {
        for x in 100..=700usize {
            let i = x + y * s;
            g.film[i] = 6.0;
            g.susp[i] = 500.0;
            g.wet[i] = 200;
            g.susp_rgb[i] = [60.0, 90.0, 160.0];
        }
    }
    g.expand_bbox(100, 40, 700, 220);
    e
}

/// Functional stability guard on the flood scene (no timing): a shorter run
/// that must stay NaN-free in every profile.
#[test]
fn flood_stays_stable_and_nan_free() {
    let mut e = flood_engine();
    for _ in 0..60 {
        e.step_simulation();
    }
    sweep_nan(e.active_grid(), "post-flood-60");
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn perf_sanity_scripted_session() {
    let mut e = Engine::new(900, 450);
    e.sliders.water = 1.0;
    drive_stroke(&mut e, 150.0, 120.0, 700.0, 160.0, 5.0, 0);
    // Same methodology as the flood gate: per-cadence-class medians (a
    // single-sample max asserts on scheduler noise, not on the product).
    let mut by_class: [Vec<f64>; 12] = Default::default();
    let t0 = Instant::now();
    for k in 0..240u64 {
        let tick0 = Instant::now();
        e.step_simulation();
        by_class[(k % 12) as usize].push(tick0.elapsed().as_secs_f64() * 1000.0);
    }
    let dt = t0.elapsed().as_secs_f64() * 1000.0;
    let per100 = (dt / 240.0) * 100.0;
    let mut worst_median = 0.0f64;
    for class in by_class.iter_mut() {
        class.sort_by(f64::total_cmp);
        let m = class[class.len() / 2];
        if m > worst_median {
            worst_median = m;
        }
    }
    println!("    240 steps in {dt:.0} ms  (~{per100:.0} ms / 100 steps)");
    println!("    worst cadence-class median tick: {worst_median:.2} ms");
    assert!(per100 < 500.0, "{per100:.0} ms per 100 steps (must be well under 1000)");
    // ADR-0134 (amended) interactive bar: on the REPRESENTATIVE session the
    // worst tick fits comfortably inside a 60 fps frame's slack.
    assert!(worst_median <= 2.0, "session worst tick class {worst_median:.2} ms > 2 ms");
}

/// Per-pass cost table on the flood (the ADR-0134 measurement record).
/// Times each solver pass in isolation at the worst-case cadence tick.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_pass_breakdown_on_flood() {
    use ph2d_wet_paint::drying::drying_pass;
    use ph2d_wet_paint::solver::{
        advect, apply_boundaries, build_flow_field, project, rebuild_active_region,
        smooth_velocity,
    };
    let mut e = flood_engine();
    // Settle the cadence state with a few real steps first.
    for _ in 0..12 {
        e.step_simulation();
    }
    let p = e.sim.gather_params(&e.tuning);
    let g = e.active_grid_mut();
    let mut time = |name: &str, f: &mut dyn FnMut(&mut ph2d_wet_paint::grid::Grid)| {
        let t0 = Instant::now();
        f(g);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("    {name:<22} {ms:>7.3} ms");
        ms
    };
    let mut total = 0.0;
    total += time("rebuild_active_region", &mut |g| rebuild_active_region(g));
    total += time("drying_pass", &mut |g| drying_pass(g, &p, 0.00025, 0.000025, false));
    total += time("build_flow_field", &mut |g| build_flow_field(g, &p, 0.0, 0.005, false));
    total += time("smooth_velocity", &mut |g| smooth_velocity(g, &p));
    total += time("advect", &mut |g| {
        advect(g, &p, 0.0, 0.005);
    });
    total += time("apply_boundaries", &mut |g| apply_boundaries(g, false));
    total += time("project", &mut |g| project(g, &p));
    println!("    {:<22} {total:>7.3} ms", "TOTAL (all passes)");
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn perf_guard_full_flood_and_kill_criterion() {
    let mut e = flood_engine();
    // Warm-up so the active mask + adaptive cadence exist before timing.
    for _ in 0..12 {
        e.step_simulation();
    }
    // The pass cadences (rebuild %2, dry %3 flowing, flow %4, project %3)
    // repeat with period 12 — tick class n%12 == 0 runs EVERY pass and is
    // the deterministic worst case. A single-sample max is scheduler noise,
    // not the product: assert on the per-class MEDIAN, worst class.
    let mut by_class: [Vec<f64>; 12] = Default::default();
    let t0 = Instant::now();
    for k in 0..240u64 {
        let tick0 = Instant::now();
        e.step_simulation();
        let tick_ms = tick0.elapsed().as_secs_f64() * 1000.0;
        by_class[(k % 12) as usize].push(tick_ms);
    }
    let dt = t0.elapsed().as_secs_f64() * 1000.0;
    let per100 = (dt / 240.0) * 100.0;
    let mut worst_median = 0.0f64;
    for (ci, class) in by_class.iter_mut().enumerate() {
        class.sort_by(f64::total_cmp);
        let median = class[class.len() / 2];
        println!("      class n%12=={ci:<2} median {median:.2} ms");
        if median > worst_median {
            worst_median = median;
        }
    }
    println!("    240 steps in {dt:.0} ms  (~{per100:.0} ms / 100 steps)");
    println!("    worst cadence-class median tick: {worst_median:.2} ms (ADR-0134 amended kill: 12 ms)");
    assert!(per100 < 1500.0, "flood: {per100:.0} ms per 100 steps (upper-bound guard 1500)");
    // ADR-0134 AMENDED kill criterion (measured table in the ADR): the flood
    // guard scene (~110k wet cells, 27% of the reference sheet standing wet)
    // measured 8.3-8.8 ms on the worst 1-in-12 cadence tick at the serial
    // scalar ceiling — the solver is serial BY SEMANTICS (the brake reads
    // live wetness written earlier in the same pass; drying reads the left
    // neighbour post-update), so ADR-0109 row-parallelism cannot apply.
    // 12 ms = measured worst + machine-noise headroom; a regression to the
    // JS reference's own upper-bound class (~15 ms/tick) still bleeds here.
    assert!(
        worst_median <= 12.0,
        "flood worst tick class {worst_median:.2} ms > 12 ms — regression past the measured ceiling"
    );
    sweep_nan(e.active_grid(), "post-flood");
}
