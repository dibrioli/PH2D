//! Gates for the PRODUCT doors — the surface the painter tool drives
//! (`begin_direct_stroke` / `direct_segment` / `dispatch_pressure_dab` /
//! `end_direct_stroke` and `render_pigment_only_region`). These are OUR
//! product additions, not part of the JS port; the port's own behaviour is
//! pinned by `fingerprint.rs` (both `dispatch_dab` and the full render are
//! thin wrappers over the doors, so a door that drifted would move the pin).

mod util;

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::render::{RenderLayer, render_pigment_only, render_pigment_only_region};
use util::drive_stroke;

fn layers_of(e: &Engine) -> Vec<RenderLayer<'_>> {
    e.layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect()
}

/// The region render writes the full render's bytes INSIDE the rect and not
/// one byte outside it. Mutation that bleeds (off-by-one on `o` or `i`)
/// corrupts the sentinel; mutation that diverges per-cell breaks equality.
#[test]
fn the_region_render_is_the_full_render_where_it_is_asked() {
    let mut e = Engine::new(120, 90);
    drive_stroke(&mut e, 20.0, 30.0, 100.0, 60.0, 4.0, 25);
    let layers = layers_of(&e);
    let (w, h) = (120usize, 90usize);
    let mut full = vec![0u8; w * h * 4];
    render_pigment_only(&layers, &mut full);
    // Sub-rect straddling the stroke; sentinel-filled canvas elsewhere.
    let (x0, y0, x1, y1) = (25usize, 28usize, 95usize, 55usize);
    let mut region = vec![0xABu8; w * h * 4];
    render_pigment_only_region(&layers, x0, y0, x1, y1, &mut region);
    let mut inside = 0usize;
    for cy in 1..=h {
        for cx in 1..=w {
            let o = ((cy - 1) * w + (cx - 1)) * 4;
            let in_rect = (x0..=x1).contains(&cx) && (y0..=y1).contains(&cy);
            if in_rect {
                assert_eq!(
                    &region[o..o + 4],
                    &full[o..o + 4],
                    "cell ({cx},{cy}) diverges"
                );
                inside += 1;
            } else {
                assert_eq!(
                    &region[o..o + 4],
                    &[0xAB; 4],
                    "cell ({cx},{cy}) outside was written"
                );
            }
        }
    }
    // The fixture must contain the phenomenon: pigment inside the rect.
    assert_eq!(inside, (x1 - x0 + 1) * (y1 - y0 + 1));
    assert!(
        full.chunks_exact(4).any(|px| px[3] > 0),
        "stroke deposited nothing"
    );
}

/// Mid-stroke `set_stroke_color` swaps the FRESH INK: the half of the stroke
/// painted after the swap deposits the new hue (the tip self-cleans toward
/// the reservoir), while the first half keeps the old one. Mutation that
/// bleeds it: the door writing only `engine.color` and never the trail's
/// reservoir (the whole stroke stays the first colour).
#[test]
fn a_mid_stroke_ink_swap_recolours_the_rest_of_the_stroke() {
    let mut e = Engine::new(300, 120);
    e.color = [220.0, 20.0, 20.0]; // red ink loaded
    e.begin_direct_stroke(0, 30.0, 60.0);
    let mut prev = 30.0f64;
    for k in 1..=20 {
        let x = 30.0 + 6.0 * k as f64;
        e.direct_segment(0, x - prev);
        e.dispatch_pressure_dab_lane(0, x, 60.0, 5.0, 1.0, 0.0, 8.0, None, None);
        prev = x;
    }
    e.set_stroke_color(0, [20.0, 220.0, 20.0]); // dip in green
    for k in 21..=40 {
        let x = 30.0 + 6.0 * k as f64;
        e.direct_segment(0, x - prev);
        e.dispatch_pressure_dab_lane(0, x, 60.0, 5.0, 1.0, 0.0, 8.0, None, None);
        prev = x;
    }
    e.end_direct_stroke();
    // Mean deposited colour, mass-weighted, in the first vs last stroke third.
    let g = e.active_grid();
    let mean = |x0: usize, x1: usize| {
        let (mut r, mut gg, mut m) = (0.0f64, 0.0f64, 0.0f64);
        for cy in 1..=g.h {
            for cx in x0..=x1 {
                let i = cx + cy * g.s;
                let w = g.susp[i] as f64;
                r += g.susp_rgb[i][0] as f64 * w;
                gg += g.susp_rgb[i][1] as f64 * w;
                m += w;
            }
        }
        (r / m.max(1.0), gg / m.max(1.0), m)
    };
    let (r0, g0, m0) = mean(30, 110);
    let (r1, g1, m1) = mean(190, 270);
    assert!(
        m0 > 1000.0 && m1 > 1000.0,
        "a stroke half deposited nothing ({m0}, {m1})"
    );
    assert!(
        r0 > g0 + 60.0,
        "the first half is not red ({r0:.0} vs {g0:.0})"
    );
    assert!(
        g1 > r1 + 20.0,
        "the second half never turned green ({r1:.0} vs {g1:.0})"
    );
}

/// The direct doors deposit real paint and gate the sim exactly as the
/// engine's own pointer path does: paused while down, running after end.
#[test]
fn a_direct_stroke_deposits_and_gates_the_sim() {
    let mut e = Engine::new(200, 120);
    let mass_before: f64 = {
        let g = e.active_grid();
        g.susp.iter().map(|&v| v as f64).sum()
    };
    e.begin_direct_stroke(0, 40.0, 60.0);
    assert!(
        !e.sim_should_run(),
        "sim must pause while a direct stroke is down"
    );
    let mut prev = (40.0f64, 60.0f64);
    for k in 1..=30 {
        let x = 40.0 + 4.0 * k as f64;
        let y = 60.0;
        let chord = ((x - prev.0).powi(2) + (y - prev.1).powi(2)).sqrt();
        e.direct_segment(0, chord);
        // Real pressure (host units mapped to the §8 range) + real radius.
        e.dispatch_pressure_dab_lane(0, x, y, 5.0, 1.0, 0.0, 9.0, None, None);
        prev = (x, y);
    }
    e.end_direct_stroke();
    assert!(
        e.sim_should_run(),
        "sim must resume after the direct stroke ends"
    );
    let mass_after: f64 = {
        let g = e.active_grid();
        g.susp.iter().map(|&v| v as f64).sum()
    };
    assert!(
        mass_after > mass_before + 1000.0,
        "direct dabs deposited no pigment ({mass_before} -> {mass_after})"
    );
    // And the sim consumes it without poisoning anything.
    for _ in 0..40 {
        e.step_simulation();
    }
    util::sweep_nan(e.active_grid(), "post-direct-stroke sim");
}

/// W2.7's engine half: `seed_paper_with` covers the EXACT index domain
/// `bake_paper` covers — pad ring included — and clamps to the tooth range.
/// The default preset is non-constant (the positive control: seeding really
/// replaced something). Mutation that bleeds it: skipping the pad ring / a
/// row (some cell keeps the preset value ≠ 0.7).
#[test]
fn the_hosts_paper_seed_covers_the_whole_padded_plane() {
    let mut e = Engine::new(120, 90);
    {
        let g = e.active_grid();
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for &v in &g.paper {
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!(
            hi > lo,
            "the preset paper must be non-constant ({lo}..{hi})"
        );
    }
    e.seed_paper_with(&mut |_, _| 0.7);
    let g = e.active_grid();
    assert!(
        g.paper.iter().all(|&v| v == 0.7),
        "a cell kept the preset — the seed missed part of the plane"
    );
}

/// W2.4's engine half: a host `grain` closure REPLACES the bristle sample on
/// the shaped path — it does not multiply with it. Striped grain (odd
/// columns 0) ⇒ odd columns get EXACTLY zero deposit; the bristled control
/// run proves those columns DO deposit without the override, so the absence
/// is the grain's doing, not the bristle's accident. Mutation that bleeds
/// it: sampling the bristle even when `grain` is Some (the veto vanishes).
#[test]
fn the_hosts_grain_replaces_the_bristle() {
    let run = |grained: bool| -> (f64, f64) {
        let mut e = Engine::new(200, 120);
        e.begin_direct_stroke(0, 40.0, 60.0);
        let mut prev = 40.0f64;
        for k in 1..=30 {
            let x = 40.0 + 4.0 * k as f64;
            e.direct_segment(0, x - prev);
            let mut sil = |_: i32, _: i32| 1.0f64;
            let mut gr = |cx: i32, _: i32| if cx % 2 == 0 { 1.0 } else { 0.0 };
            e.dispatch_pressure_dab_lane(
                0,
                x,
                60.0,
                5.0,
                1.0,
                0.0,
                9.0,
                Some(&mut sil),
                grained.then_some(&mut gr as &mut dyn FnMut(i32, i32) -> f64),
            );
            prev = x;
        }
        e.end_direct_stroke();
        let g = e.active_grid();
        let (mut even, mut odd) = (0.0f64, 0.0f64);
        for cy in 1..=g.h {
            for cx in 1..=g.w {
                let v = f64::from(g.susp[cx + cy * g.s]);
                if cx % 2 == 0 {
                    even += v;
                } else {
                    odd += v;
                }
            }
        }
        (even, odd)
    };
    let (ctrl_even, ctrl_odd) = run(false);
    assert!(
        ctrl_even > 1000.0 && ctrl_odd > 1000.0,
        "the bristled control must deposit in BOTH parities ({ctrl_even}, {ctrl_odd})"
    );
    let (even, odd) = run(true);
    assert!(even > 1000.0, "the grained run stopped depositing ({even})");
    assert_eq!(
        odd, 0.0,
        "a vetoed column carries deposit — the bristle spoke over the grain"
    );
}
