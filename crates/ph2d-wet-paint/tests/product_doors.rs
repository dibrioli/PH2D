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
        .map(|l| RenderLayer { grid: &l.grid, opacity: l.opacity, visible: l.visible })
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
                assert_eq!(&region[o..o + 4], &full[o..o + 4], "cell ({cx},{cy}) diverges");
                inside += 1;
            } else {
                assert_eq!(&region[o..o + 4], &[0xAB; 4], "cell ({cx},{cy}) outside was written");
            }
        }
    }
    // The fixture must contain the phenomenon: pigment inside the rect.
    assert_eq!(inside, (x1 - x0 + 1) * (y1 - y0 + 1));
    assert!(full.chunks_exact(4).any(|px| px[3] > 0), "stroke deposited nothing");
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
    e.begin_direct_stroke(40.0, 60.0);
    assert!(!e.sim_should_run(), "sim must pause while a direct stroke is down");
    let mut prev = (40.0f64, 60.0f64);
    for k in 1..=30 {
        let x = 40.0 + 4.0 * k as f64;
        let y = 60.0;
        let chord = ((x - prev.0).powi(2) + (y - prev.1).powi(2)).sqrt();
        e.direct_segment(chord);
        // Real pressure (host units mapped to the §8 range) + real radius.
        e.dispatch_pressure_dab(x, y, 5.0, 1.0, 0.0, 9.0);
        prev = (x, y);
    }
    e.end_direct_stroke();
    assert!(e.sim_should_run(), "sim must resume after the direct stroke ends");
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
