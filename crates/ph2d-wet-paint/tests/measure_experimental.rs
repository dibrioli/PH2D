//! Measurement harness for the two EXPERIMENTAL knobs (doc 22 §2.8): pigment
//! mixing (K–M) and glaze layering. Both route colour through the sRGB
//! transfer, whose `libm::pow` is the only transcendental in the model.
//!
//! This file MEASURES; it asserts nothing about wall-clock (the ratio gates
//! live in `perf.rs`). Run:
//!
//!   cargo test -p ph2d-wet-paint --release --test measure_experimental \
//!     -- --ignored --nocapture

mod util;

use std::time::Instant;

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::render::{PigmentVisual, RenderLayer, render_pigment_region_visual};

/// The `perf.rs` flood scene: ~110k wet cells, the ADR-0134 upper bound.
///
/// ⚠️ The colours VARY per cell, unlike `perf.rs`'s single-colour flood. A
/// uniform wash is the one scene where a K–M corner mean is degenerate (four
/// identical corners mix to themselves), so a uniform fixture would flatter
/// any "all corners equal" shortcut and hide the cost of the case the
/// checkbox exists for — colour fronts meeting. Every cell here differs from
/// its neighbours, which is the expensive end.
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
            // A deterministic spread of pigments, different in every cell.
            let h = ((x * 7919) ^ (y * 104_729)) as f32;
            g.susp_rgb[i] = [
                20.0 + (h % 211.0),
                30.0 + ((h / 211.0) % 197.0),
                40.0 + ((h / 41_567.0) % 187.0),
            ];
            g.sett[i] = 120.0;
            g.sett_rgb[i] = [
                200.0 - (h % 173.0),
                180.0 - ((h / 173.0) % 151.0),
                160.0 - ((h / 26_123.0) % 139.0),
            ];
        }
    }
    g.expand_bbox(100, 40, 700, 220);
    e
}

/// A painted sheet for the RENDER side: settled AND suspended everywhere, so
/// both glaze stacking arms run on every pixel (the worst case the product
/// reaches when the artist has laid wet paint over dry).
fn painted_engine() -> Engine {
    let mut e = Engine::new(900, 450);
    let g = e.active_grid_mut();
    let s = g.s;
    for y in 1..=450usize {
        for x in 1..=900usize {
            let i = x + y * s;
            g.sett[i] = 400.0;
            g.sett_rgb[i] = [180.0, 60.0, 40.0];
            g.susp[i] = 200.0;
            g.susp_rgb[i] = [40.0, 80.0, 170.0];
            g.film[i] = 1.5;
            g.wet[i] = 180;
        }
    }
    g.expand_bbox(1, 1, 900, 450);
    e
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// 240 real ticks on the flood, K–M pigment mixing OFF then ON.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_pigment_mixing_cost() {
    let mut off = Vec::new();
    let mut on = Vec::new();
    for km in [false, true] {
        let mut e = flood_engine();
        e.sim.km_mixing = km;
        for _ in 0..12 {
            e.step_simulation();
        }
        let mut by_class: [Vec<f64>; 12] = Default::default();
        let t0 = Instant::now();
        for k in 0..240u64 {
            let t = Instant::now();
            e.step_simulation();
            by_class[(k % 12) as usize].push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let total = t0.elapsed().as_secs_f64() * 1000.0;
        let worst = by_class
            .into_iter()
            .map(median)
            .fold(0.0f64, |a, b| a.max(b));
        if km { &mut on } else { &mut off }.push((total, worst));
    }
    let (t_off, w_off) = off[0];
    let (t_on, w_on) = on[0];
    println!("\n  PIGMENT MIXING (K-M) — flood, 240 ticks");
    println!("    OFF   total {t_off:8.1} ms   worst cadence-class tick {w_off:6.2} ms");
    println!("    ON    total {t_on:8.1} ms   worst cadence-class tick {w_on:6.2} ms");
    println!(
        "    ratio       {:8.2}x                            {:6.2}x",
        t_on / t_off,
        w_on / w_off
    );
}

/// Per-PASS breakdown with K–M mixing OFF vs ON — which passes actually pay
/// for the checkbox, so the fix aims where the milliseconds are.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_km_pass_breakdown() {
    use ph2d_wet_paint::drying::drying_pass;
    use ph2d_wet_paint::solver::{advect, build_flow_field, project, smooth_velocity};
    println!("\n  K-M PASS BREAKDOWN — flood, median of 40 calls (ms)");
    println!("    {:<22} {:>9} {:>9} {:>8}", "pass", "km OFF", "km ON", "ratio");
    let mut off: Vec<(String, f64)> = Vec::new();
    let mut on: Vec<(String, f64)> = Vec::new();
    for km in [false, true] {
        let mut e = flood_engine();
        e.sim.km_mixing = km;
        for _ in 0..12 {
            e.step_simulation();
        }
        let p = e.sim.gather_params(&e.tuning);
        let g = e.active_grid_mut();
        let sink = if km { &mut on } else { &mut off };
        let mut time = |name: &str,
                        g: &mut ph2d_wet_paint::grid::Grid,
                        f: &mut dyn FnMut(&mut ph2d_wet_paint::grid::Grid)| {
            let mut s = Vec::new();
            for _ in 0..40 {
                let t = Instant::now();
                f(g);
                s.push(t.elapsed().as_secs_f64() * 1000.0);
            }
            (name.to_string(), median(s))
        };
        sink.push(time("drying_pass", g, &mut |g| {
            drying_pass(g, &p, 0.00025, 0.000025, false)
        }));
        sink.push(time("build_flow_field", g, &mut |g| {
            build_flow_field(g, &p, 0.0, 0.005, false)
        }));
        sink.push(time("smooth_velocity", g, &mut |g| smooth_velocity(g, &p)));
        sink.push(time("advect", g, &mut |g| {
            advect(g, &p, 0.0, 0.005);
        }));
        sink.push(time("project", g, &mut |g| project(g, &p)));
    }
    for (a, b) in off.iter().zip(on.iter()) {
        println!(
            "    {:<22} {:>9.3} {:>9.3} {:>7.1}x",
            a.0,
            a.1,
            b.1,
            b.1 / a.1.max(1e-9)
        );
    }
}

/// The product composite path (`render_pigment_region_visual`, what the tool
/// calls every dirty frame), glaze OFF vs ON, full 900x450 sheet.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_glaze_layering_cost() {
    let mut e = painted_engine();
    let params = e.sim.gather_params(&e.tuning);
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let mut out = vec![0u8; 900 * 450 * 4];
    println!("\n  GLAZE LAYERING — product composite, full 900x450 sheet");
    for (label, visual) in [
        (
            "paper OFF glaze OFF",
            PigmentVisual {
                paper: false,
                km_glaze: false,
            },
        ),
        (
            "paper ON  glaze OFF",
            PigmentVisual {
                paper: true,
                km_glaze: false,
            },
        ),
        (
            "paper OFF glaze ON ",
            PigmentVisual {
                paper: false,
                km_glaze: true,
            },
        ),
        (
            "paper ON  glaze ON ",
            PigmentVisual {
                paper: true,
                km_glaze: true,
            },
        ),
    ] {
        let mut samples = Vec::new();
        for _ in 0..20 {
            let t = Instant::now();
            render_pigment_region_visual(
                Some(&params),
                &layers,
                visual,
                1,
                1,
                900,
                450,
                &mut out,
            );
            samples.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        println!("    {label}   median {:7.3} ms", median(samples));
    }
}

/// The full-sheet renderer (`render_region`, the reference look / export),
/// glaze OFF vs ON.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_render_region_glaze_cost() {
    use ph2d_wet_paint::render::render_region;
    let mut e = painted_engine();
    let params = e.sim.gather_params(&e.tuning);
    let active: &Grid = &e.layers[0].grid;
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let mut out = vec![0u8; 900 * 450 * 4];
    println!("\n  RENDER_REGION — reference look, full 900x450 sheet");
    for glaze in [false, true] {
        let mut samples = Vec::new();
        for _ in 0..20 {
            let t = Instant::now();
            render_region(
                &params, &layers, active, false, glaze, false, &mut out, 1, 1, 900, 450,
            );
            samples.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let label = if glaze { "ON " } else { "OFF" };
        println!("    km_glaze {label}   median {:7.3} ms", median(samples));
    }
}

/// The REPRESENTATIVE session (the `perf.rs` scripted stroke), K–M OFF vs ON.
/// The flood is the declared upper bound; this is what an artist's stroke
/// actually costs, and it is the number that decides whether the checkbox is
/// usable rather than merely improved.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_representative_session() {
    println!("\n  REPRESENTATIVE SESSION — scripted stroke, 240 ticks");
    for km in [false, true] {
        let mut e = Engine::new(900, 450);
        e.sim.km_mixing = km;
        e.sliders.water = 1.0;
        util::drive_stroke(&mut e, 150.0, 120.0, 700.0, 160.0, 5.0, 0);
        let mut by_class: [Vec<f64>; 12] = Default::default();
        let t0 = Instant::now();
        for k in 0..240u64 {
            let t = Instant::now();
            e.step_simulation();
            by_class[(k % 12) as usize].push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let total = t0.elapsed().as_secs_f64() * 1000.0;
        let worst = by_class
            .into_iter()
            .map(median)
            .fold(0.0f64, |a, b| a.max(b));
        let label = if km { "ON " } else { "OFF" };
        println!("    km_mixing {label}  240 ticks {total:7.1} ms   worst class tick {worst:5.2} ms");
    }
}

/// The primitive under both: how much a single `libm::pow` costs, and how
/// many of them each experimental path spends per cell.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_transfer_primitive() {
    const N: usize = 2_000_000;
    let mut acc = 0.0f64;
    let t = Instant::now();
    for k in 0..N {
        acc += libm::pow((k % 255) as f64 / 255.0 + 0.001, 2.4);
    }
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("\n  TRANSFER PRIMITIVE");
    println!(
        "    libm::pow x{N}  {ms:7.1} ms  ({:.1} ns each)   [sink {acc:.3}]",
        ms * 1e6 / N as f64
    );
    let mut acc2 = 0.0f64;
    let t = Instant::now();
    for k in 0..N {
        acc2 += ((k % 255) as f64 / 255.0 + 0.001).powf(2.4);
    }
    let ms2 = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "    f64::powf  x{N}  {ms2:7.1} ms  ({:.1} ns each)   [sink {acc2:.3}]",
        ms2 * 1e6 / N as f64
    );
}
