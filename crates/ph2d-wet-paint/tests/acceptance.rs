//! SPEC §18 acceptance tests 1–10, ported from the reference `test/smoke.mjs`
//! — same scripted inputs, same numbers, same tolerances. This IS the parity
//! suite of ADR-0134: the JS app passes these checks; the port must pass the
//! identical ones.

mod util;

// `Arc<Mutex<_>>` e não `Rc<RefCell<_>>`: o `on_dab` é `+ Send` porque o
// `Engine` VIAJA para a thread da sim (`wetpaint/offthread.rs`), e um `Rc` não
// atravessa thread. O hook é sonda de teste — nenhum caminho de produto o
// preenche —, então o bound não custa nada além desta troca.
use std::sync::{Arc, Mutex};

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::paper::{PaperKnobs, PaperPreset, generate_paper_tile, measure_tile_stats};
use ph2d_wet_paint::solver::advect;
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob, Tuning};
use util::{drive_stroke, sweep_nan};

// ---------------------------------------------------------------------------
// §18.1 + §18.2: pressure fade-in and porous deposit (shared 260-px stroke)
// ---------------------------------------------------------------------------

#[test]
fn t01_02_pressure_fade_in_and_porous_deposit() {
    let mut e = Engine::new(900, 450);
    let pressures: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));
    {
        let sink = pressures.clone();
        e.on_dab = Some(Box::new(move |b, _dab| {
            sink.lock().expect("o mutex da sonda").push(b);
        }));
    }
    drive_stroke(&mut e, 100.0, 200.0, 360.0, 200.0, 4.0, 0);
    let p = pressures.lock().expect("o mutex da sonda");
    assert!(p.len() > 30, "expected many dabs, got {}", p.len());
    assert_eq!(p[0], 0.0, "first dab pressure = {}, expected 0", p[0]);
    let n = p.len();
    let steady_slice = &p[n - 20..n - 5]; // last 15 minus tail 5
    let steady = steady_slice.iter().sum::<f64>() / steady_slice.len() as f64;
    let early = &p[1..6];
    let early_mean = early.iter().sum::<f64>() / early.len() as f64;
    assert!(
        early_mean < 0.7 * steady,
        "early mean {early_mean:.2} !< 70% of steady {steady:.2}"
    );
    assert!(p.iter().all(|v| !v.is_nan()), "NaN pressure recorded");
    let max_p = p.iter().cloned().fold(f64::MIN, f64::max);
    assert!(max_p <= 7.34, "max pressure {max_p:.3} > 7.34");

    // §18.2 — porous deposit on the same stroke's band (radius 12).
    let g = e.active_grid();
    let mut band = 0u32;
    let mut holding = 0u32;
    let mut total = 0.0f64;
    for y in (200 - 12)..=(200 + 12) {
        for x in 100..=360usize {
            let i = x + y * g.s;
            band += 1;
            let m = g.susp[i] as f64 + g.sett[i] as f64;
            if m > 0.0 {
                holding += 1;
            }
            total += m;
        }
    }
    assert!(total > 0.0, "no pigment deposited at all");
    let frac = holding as f64 / band as f64;
    assert!(
        frac < 0.85,
        "{:.1}% of band cells hold pigment (must be < 85%)",
        frac * 100.0
    );
    sweep_nan(g, "post-stroke");
}

// ---------------------------------------------------------------------------
// §18.3: wet & dry tools
// ---------------------------------------------------------------------------

#[test]
fn t03_wet_and_dry_tools() {
    let mut e = Engine::new(900, 450);
    let s = e.active_grid().s;
    let center = 150 + 150 * s;
    e.tool = Tool::Wet;
    drive_stroke(&mut e, 100.0, 150.0, 200.0, 150.0, 4.0, 0);
    let first = e.active_grid().film[center] as f64;
    assert!(
        first > 0.0 && first < 0.2,
        "one wet pass film = {first} (need 0 < f < 0.2)"
    );
    assert!(
        e.active_grid().wet[center] > 0,
        "wetness byte at line center is 0 after wet pass"
    );
    for _ in 0..30 {
        drive_stroke(&mut e, 100.0, 150.0, 200.0, 150.0, 4.0, 0);
    }
    let after = e.active_grid().film[center] as f64;
    assert!(
        after > 5.0 * first,
        "31 wet passes film {after} !> 5x first {first}"
    );
    e.tool = Tool::Dry;
    drive_stroke(&mut e, 100.0, 150.0, 200.0, 150.0, 4.0, 0);
    let g = e.active_grid();
    assert_eq!(
        g.wet[center], 0,
        "dry pass left wetness {} (must seal to 0)",
        g.wet[center]
    );
    assert!(
        g.film[center] >= 0.001,
        "dry pass film {} < 0.001 floor",
        g.film[center]
    );
}

// ---------------------------------------------------------------------------
// §18.4: blow
// ---------------------------------------------------------------------------

#[test]
fn t04_blow() {
    let mut e = Engine::new(900, 450);
    {
        let g = e.active_grid_mut();
        let s = g.s;
        for y in 140..=180usize {
            for x in 140..=180usize {
                let i = x + y * s;
                g.film[i] = 3.0;
                g.wet[i] = (0.8f64 * 255.0).round() as u8;
            }
        }
        g.expand_bbox(140, 140, 180, 180);
    }
    e.tool = Tool::Blow;
    drive_stroke(&mut e, 130.0, 160.0, 190.0, 160.0, 3.0, 0); // sim runs live during blow
    let g = e.active_grid();
    let mut max_vx = 0.0f64;
    for y in 2..=(g.h - 1) {
        for x in 2..=(g.w - 1) {
            let v = (g.vel_x[x + y * g.s] as f64).abs();
            if v > max_vx {
                max_vx = v;
            }
        }
    }
    assert!(
        max_vx > 0.0 && max_vx < 0.31,
        "blow max |vx| = {max_vx:.4} (need 0 < v < 0.31)"
    );
    sweep_nan(g, "post-blow");
}

// ---------------------------------------------------------------------------
// §18.5: advection semantics
// ---------------------------------------------------------------------------

#[test]
fn t05_advection_semantics() {
    // (a) color replacement: heavy cell receiving from a light colored
    // neighbour REPLACES its suspended color after one advect.
    let mut e = Engine::new(60, 60);
    let p = e.sim.gather_params(&e.tuning);
    let g = e.active_grid_mut();
    let s = g.s;
    let dst = 20 + 20 * s;
    let src = 19 + 20 * s;
    g.susp[dst] = 2500.0;
    g.susp_rgb[dst] = [255.0, 0.0, 0.0];
    g.susp[src] = 300.0;
    g.susp_rgb[src] = [0.0, 0.0, 255.0];
    g.active[dst] = 1;
    g.active[src] = 1;
    g.expand_bbox(18, 18, 22, 22);
    g.flow_x[dst] = 1.0; // back-trace lands exactly on the neighbour
    advect(g, &p, 0.0, 0.0);
    assert!(
        g.susp_rgb[dst][2] == 255.0 && g.susp_rgb[dst][0] == 0.0,
        "destination color not replaced: rgb({:?})",
        g.susp_rgb[dst]
    );
    assert!(g.susp[dst] > 2500.0, "destination mass did not grow");

    // (b) no water cap during advection.
    let mut e2 = Engine::new(60, 60);
    let p2 = e2.sim.gather_params(&e2.tuning);
    let g2 = e2.active_grid_mut();
    let s2 = g2.s;
    let d2 = 30 + 30 * s2;
    let s2i = 29 + 30 * s2;
    g2.film[d2] = 8.0;
    g2.film[s2i] = 8.0;
    g2.active[d2] = 1;
    g2.active[s2i] = 1;
    g2.expand_bbox(28, 28, 32, 32);
    g2.flow_x[d2] = 1.0;
    advect(g2, &p2, 0.0, 0.0);
    assert!(
        g2.film[d2] > 8.0,
        "destination film {} did not exceed the cap 8",
        g2.film[d2]
    );
}

// ---------------------------------------------------------------------------
// §18.6: paper statistics
// ---------------------------------------------------------------------------

#[test]
fn t06_paper_statistics() {
    for preset in [PaperPreset::Cold, PaperPreset::Rough, PaperPreset::Hot] {
        let p = preset.params();
        let tile = generate_paper_tile(preset, 0, PaperKnobs::default());
        let s = measure_tile_stats(&tile);
        assert!(
            (s.mean - p.mean).abs() < 0.02,
            "{preset:?} mean {:.3} vs {}",
            s.mean,
            p.mean
        );
        assert!(
            (s.std - p.std).abs() < 0.04,
            "{preset:?} std {:.3} vs {}",
            s.std,
            p.std
        );
        assert!(
            (s.grad - p.grad).abs() / p.grad < 0.18,
            "{preset:?} grad {:.4} vs {}",
            s.grad,
            p.grad
        );
    }
}

// ---------------------------------------------------------------------------
// §18.7: blend re-mixes dry paint
// ---------------------------------------------------------------------------

#[test]
fn t07_blend_remixes_dry_paint() {
    let mut e = Engine::new(900, 450);
    {
        let g = e.active_grid_mut();
        let s = g.s;
        for y in 100..=140usize {
            for x in 100..=140usize {
                let i = x + y * s;
                g.sett[i] = 900.0;
                g.sett_rgb[i] = [200.0, 40.0, 30.0];
            }
        }
    }
    let sum_outside = |e: &Engine| {
        let g = e.active_grid();
        let mut sum = 0.0f64;
        for y in 90..=150usize {
            for x in 141..=200usize {
                sum += g.sett[x + y * g.s] as f64;
            }
        }
        sum
    };
    let before = sum_outside(&e);
    e.tool = Tool::Blend;
    drive_stroke(&mut e, 120.0, 120.0, 170.0, 120.0, 4.0, 0); // across the patch's right edge
    let grown = sum_outside(&e) - before;
    assert!(
        grown > 100.0,
        "settled mass beyond the patch grew by {grown:.1} (need > 100)"
    );
    sweep_nan(e.active_grid(), "post-blend");
}

// ---------------------------------------------------------------------------
// §18.8: tuning registry
// ---------------------------------------------------------------------------

#[test]
fn t08_tuning_registry() {
    assert!(KNOB_DEFS.len() >= 39, "only {} knobs", KNOB_DEFS.len());
    let mut t = Tuning::default();
    t.set_by_key("leveling", 1.7);
    t.set_by_key("gravity", 0.03);
    t.set_by_key("brake", 0.1);
    let changed = t.reset_group(ph2d_wet_paint::tuning::KnobGroup::Physics);
    assert_eq!(
        changed.len(),
        3,
        "3 knobs were off-default; each must report"
    );
    assert!(
        t.get(Knob::Leveling) == 0.5 && t.get(Knob::Gravity) == 0.005 && t.get(Knob::Brake) == 1.5,
        "group reset did not restore defaults"
    );
    t.set_by_key("waterCap", 999.0); // free numeric input beyond the slider range
    assert_eq!(t.get(Knob::WaterCap), 999.0, "out-of-range value rejected");
    t.set_by_key("waterCap", f64::NAN);
    assert_eq!(
        t.get(Knob::WaterCap),
        8.0,
        "NaN input fell back to {}, expected default 8",
        t.get(Knob::WaterCap)
    );
}

// ---------------------------------------------------------------------------
// §18.9: drip + stability
// ---------------------------------------------------------------------------

fn drip_setup() -> Engine {
    let mut e = Engine::new(80, 300);
    let g = e.active_grid_mut();
    let s = g.s;
    for y in 20..=39usize {
        for x in 30..=49usize {
            let i = x + y * s;
            g.film[i] = 8.0;
            g.susp[i] = 800.0;
            g.wet[i] = 255;
            g.susp_rgb[i] = [30.0, 60.0, 150.0];
        }
    }
    g.expand_bbox(30, 20, 49, 39);
    e
}

fn wet_front_max_y(e: &Engine) -> usize {
    let g = e.active_grid();
    let mut max_y = 0;
    for y in 1..=g.h {
        for x in 1..=g.w {
            // The JS compares the widened f32 against f64 0.1 — a cell at
            // exactly 0.1f32 (0.10000000149...) counts as wet.
            if g.film[x + y * g.s] as f64 > 0.1 {
                max_y = y;
                break;
            }
        }
    }
    max_y
}

#[test]
fn t09_drip_and_stability() {
    let mut e = drip_setup();
    e.sim.gravity_override = Some([0.0, 1.5]);
    for _ in 0..240 {
        e.step_simulation();
    }
    let descent = wet_front_max_y(&e) as i64 - 39;
    assert!(
        descent >= 40,
        "gravity drip descended {descent} rows (need >= 40)"
    );
    let mut e0 = drip_setup();
    e0.sim.gravity_override = Some([0.0, 0.0]);
    for _ in 0..240 {
        e0.step_simulation();
    }
    let spread = wet_front_max_y(&e0) as i64 - 39;
    assert!(
        spread <= 8,
        "zero-gravity front moved {spread} rows (need <= 8)"
    );
    for _ in 0..120 {
        e.step_simulation();
    }
    sweep_nan(e.active_grid(), "post-drip");
}

// ---------------------------------------------------------------------------
// §18.10: extension neutrality — the §17 code paths compiled-in vs
// force-skipped produce IDENTICAL field state at neutral knobs.
// ---------------------------------------------------------------------------

fn neutrality_run(bypass: bool) -> Engine {
    let mut e = Engine::new(300, 200);
    e.sim.ext_bypass = bypass;
    drive_stroke(&mut e, 60.0, 100.0, 240.0, 100.0, 4.0, 0);
    for _ in 0..60 {
        e.step_simulation();
    }
    e
}

#[test]
fn t10_extension_neutrality() {
    let e_on = neutrality_run(false); // extension code paths run (neutral knobs)
    let e_off = neutrality_run(true); // extension code paths force-skipped
    let g_on = e_on.active_grid();
    let g_off = e_off.active_grid();
    let pairs: [(&str, &[f32], &[f32]); 5] = [
        ("film", &g_on.film, &g_off.film),
        ("susp", &g_on.susp, &g_off.susp),
        ("sett", &g_on.sett, &g_off.sett),
        ("vel_x", &g_on.vel_x, &g_off.vel_x),
        ("vel_y", &g_on.vel_y, &g_off.vel_y),
    ];
    for (name, a, b) in pairs {
        for i in 0..a.len() {
            assert!(
                a[i].to_bits() == b[i].to_bits(),
                "{name} differs at index {i}: {} vs {}",
                a[i],
                b[i]
            );
        }
    }
}
