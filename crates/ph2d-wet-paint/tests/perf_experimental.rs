//! Regression guard for the two EXPERIMENTAL knobs (pigment mixing / glaze
//! layering), sibling of `perf.rs` — that file owns the DEFAULT path's kill
//! criterion, this one owns the price of turning a checkbox on.
//!
//! The assertion is a RATIO of two measurements taken in the same run on the
//! same machine (checkbox OFF vs ON), never wall-clock: `ci-test` builds this
//! crate at opt-level 1 while `libm` — a dependency — gets opt-level 3, so a
//! wall-clock bar here would measure the profile, and a ratio measured under
//! that handicap only ever UNDER-states the table's win.
//!
//! What it defends: both knobs used to route every colour through
//! `libm::pow`, 9-15 times per cell, which cost 12.5x on the sim tick and 21x
//! on the composite — measured, and unusable. Reverting the tabulated
//! transfer (`colorops::transfer`) puts those multiples straight back and
//! bleeds here.

mod util;

use std::time::Instant;

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::render::{PigmentVisual, RenderLayer, render_pigment_region_visual};

const W: usize = 300;
const H: usize = 200;

/// A wet, MULTI-COLOURED region. The colours must vary: four identical
/// corners make the advection's K–M mean degenerate, so a uniform fixture
/// would measure the one case the checkbox is cheapest on.
fn scene() -> Engine {
    let mut e = Engine::new(W, H);
    let g = e.active_grid_mut();
    let s = g.s;
    for y in 20..=(H - 20) {
        for x in 20..=(W - 20) {
            let i = x + y * s;
            g.film[i] = 6.0;
            g.susp[i] = 500.0;
            g.wet[i] = 200;
            let h = ((x * 7919) ^ (y * 104_729)) as f32;
            g.susp_rgb[i] = [
                20.0 + (h % 211.0),
                30.0 + ((h / 211.0) % 197.0),
                40.0 + (h % 187.0),
            ];
            g.sett[i] = 120.0;
            g.sett_rgb[i] = [
                200.0 - (h % 173.0),
                180.0 - ((h / 173.0) % 151.0),
                160.0 - (h % 139.0),
            ];
        }
    }
    g.expand_bbox(20, 20, (W - 20) as i32, (H - 20) as i32);
    e
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Median cost of one sim tick with the pigment-mixing checkbox in `km`.
fn tick_ms(km: bool) -> f64 {
    let mut e = scene();
    e.sim.km_mixing = km;
    for _ in 0..12 {
        e.step_simulation();
    }
    // Sample whole cadence cycles so both sides see the same pass mix.
    let mut s = Vec::new();
    for _ in 0..5 {
        let t = Instant::now();
        for _ in 0..12 {
            e.step_simulation();
        }
        s.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    median(s)
}

#[test]
fn pigment_mixing_costs_a_ratio_not_an_order_of_magnitude() {
    let off = tick_ms(false);
    let on = tick_ms(true);
    let ratio = on / off.max(1e-9);
    println!("pigment mixing: OFF {off:.3} ms / 12 ticks, ON {on:.3} ms, ratio {ratio:.2}x");
    // Measured on THIS scene: 2.68x release, 2.58x ci-test (the profile
    // handicap turned out not to matter). The pre-table implementation
    // measured 12.5x on the flood; the bar sits far below that and far above
    // the noise.
    assert!(
        ratio < 6.0,
        "pigment mixing costs {ratio:.2}x a plain tick (bar 6x) — the sRGB \
         transfer is back on the per-call `libm::pow` path"
    );
}

#[test]
fn glaze_layering_costs_a_ratio_not_an_order_of_magnitude() {
    let e = scene();
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
    let mut out = vec![0u8; W * H * 4];
    let mut run = |glaze: bool| {
        let visual = PigmentVisual {
            paper: false,
            km_glaze: glaze,
        };
        let mut s = Vec::new();
        for _ in 0..9 {
            let t = Instant::now();
            render_pigment_region_visual(Some(&params), &layers, visual, 1, 1, W, H, &mut out);
            s.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        median(s)
    };
    let off = run(false);
    let on = run(true);
    let ratio = on / off.max(1e-9);
    println!("glaze layering: OFF {off:.3} ms, ON {on:.3} ms, ratio {ratio:.2}x");
    // Measured 2.84x release, 2.60x ci-test. The pre-table implementation
    // measured 20.7x on the full sheet.
    assert!(
        ratio < 8.0,
        "glaze layering costs {ratio:.2}x a plain composite (bar 8x) — the \
         sRGB transfer is back on the per-call `libm::pow` path"
    );
}

/// Bare paper must render IDENTICALLY with the glaze checkbox on and off.
///
/// The stacking law is `a=0 -> backdrop`, so a pixel no layer paints is the
/// backdrop by definition — but the implementation used to prove that by
/// converting the sheet into reflectance and straight back, an identity round
/// trip that both cost the whole checkbox on a bare sheet AND landed an ulp
/// away from where it started. Entering reflectance lazily makes the answer
/// exact, and "exact" is assertable where "close" was not: this compares
/// BYTES over a sheet with a painted island, so it covers the untouched
/// pixels and the boundary at once.
#[test]
fn the_glaze_checkbox_leaves_unpainted_paper_byte_identical() {
    let mut e = Engine::new(W, H);
    {
        let g = e.active_grid_mut();
        let s = g.s;
        // A small island of paint; the rest of the sheet stays bare.
        for y in 60..=90usize {
            for x in 60..=120usize {
                let i = x + y * s;
                g.sett[i] = 400.0;
                g.sett_rgb[i] = [180.0, 60.0, 40.0];
                g.susp[i] = 200.0;
                g.susp_rgb[i] = [40.0, 80.0, 170.0];
            }
        }
        g.expand_bbox(60, 60, 120, 90);
    }
    let params = e.sim.gather_params(&e.tuning);
    let active_grid = &e.layers[0].grid;
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let mut off = vec![0u8; W * H * 4];
    let mut on = vec![0u8; W * H * 4];
    let paint = |glaze: bool, out: &mut [u8]| {
        ph2d_wet_paint::render::render_region(
            &params,
            &layers,
            active_grid,
            false,
            glaze,
            false,
            out,
            1,
            1,
            W as i32,
            H as i32,
        );
    };
    paint(false, &mut off);
    paint(true, &mut on);
    let mut bare_differ = 0usize;
    let mut painted_differ = 0usize;
    for cy in 1..=H {
        for cx in 1..=W {
            let i = cx + cy * active_grid.s;
            let o = ((cy - 1) * W + (cx - 1)) * 4;
            let painted = active_grid.sett[i] > 0.0 || active_grid.susp[i] > 0.0;
            if off[o..o + 4] != on[o..o + 4] {
                if painted {
                    painted_differ += 1;
                } else {
                    bare_differ += 1;
                }
            }
        }
    }
    assert_eq!(
        bare_differ, 0,
        "the glaze checkbox moved {bare_differ} bare-paper pixels"
    );
    // ...and the control: it MUST move the painted ones, or the comparison
    // above would be satisfied by a glaze that does nothing at all.
    assert!(
        painted_differ > 0,
        "the glaze changed nothing anywhere — this gate would pass on a no-op"
    );
}

/// Where there is no paint, the glaze checkbox must cost nothing.
///
/// This is the gate for the LAZY entry into reflectance, and it has to be a
/// perf gate: the eager form converted the sheet in and straight back on
/// every pixel, and that round trip is accurate enough that `clamp_u8` rounds
/// to the SAME bytes — so no comparison of output can see it. What it costs
/// is time, and only on the pixels it was pointless for: measured on a bare
/// sheet, eager 1.264 ms against lazy 0.323 ms.
#[test]
fn the_glaze_checkbox_is_free_where_there_is_no_paint() {
    let e = Engine::new(W, H); // bare: no pigment anywhere
    let params = e.sim.gather_params(&e.tuning);
    let active_grid = &e.layers[0].grid;
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let mut out = vec![0u8; W * H * 4];
    let mut run = |glaze: bool| {
        let mut s = Vec::new();
        for _ in 0..15 {
            let t = Instant::now();
            ph2d_wet_paint::render::render_region(
                &params,
                &layers,
                active_grid,
                false,
                glaze,
                false,
                &mut out,
                1,
                1,
                W as i32,
                H as i32,
            );
            s.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        median(s)
    };
    let off = run(false);
    let on = run(true);
    let ratio = on / off.max(1e-9);
    println!("bare sheet: glaze OFF {off:.4} ms, ON {on:.4} ms, ratio {ratio:.2}x");
    // Measured 1.08x lazy against 4.2x eager.
    assert!(
        ratio < 1.8,
        "the glaze costs {ratio:.2}x on a sheet with nothing to glaze (bar \
         1.8x) — reflectance is being entered before a layer asks for it"
    );
}

/// The other half of the guard, and the one a ratio cannot express: turning
/// a checkbox ON must not change what the OTHER path renders. Both knobs OFF
/// has to stay byte-identical to the historical composite.
#[test]
fn the_knobs_off_path_is_untouched_by_the_table() {
    let e = scene();
    let layers: Vec<RenderLayer<'_>> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let mut a = vec![0u8; W * H * 4];
    let mut b = vec![0u8; W * H * 4];
    ph2d_wet_paint::render::render_pigment_only_region(&layers, 1, 1, W, H, &mut a);
    let params = e.sim.gather_params(&e.tuning);
    render_pigment_region_visual(
        Some(&params),
        &layers,
        PigmentVisual::default(),
        1,
        1,
        W,
        H,
        &mut b,
    );
    assert_eq!(
        a, b,
        "the visual-terms-off body stopped matching the wrapper"
    );
}
