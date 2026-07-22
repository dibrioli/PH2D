//! Gates of the W-tuning-UI product doors (doc 22 §3 W1): the direct-tool
//! door and the blend lane door match the engine's own dispatch, and the
//! visual pigment render is byte-identical with every term off.
//!
//! These are NEW doors — the pinned session fingerprint never calls them, so
//! their existence cannot move it; these gates are their own floor.

mod util;

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::render::{
    PigmentVisual, RenderLayer, render_pigment_only_region, render_pigment_region_visual,
};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 96;
const H: usize = 96;

/// Seed both engines identically through the PRODUCT lane door (deterministic
/// dab positions — the engine's own synthetic stroke would drift with clock
/// cadence), then settle everything so the tools have paint to act on.
fn seeded_engine(color: [f64; 3]) -> Engine {
    let mut e = Engine::new(W, H);
    e.color = color;
    e.begin_direct_stroke(0, 20.0, 48.0);
    let mut x = 20.0;
    while x < 76.0 {
        x += 4.0;
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane(0, x, 48.0, 5.0, 1.0, 0.0, 10.0, None, None);
    }
    e.end_direct_stroke();
    for _ in 0..8 {
        e.step_simulation();
    }
    e
}

/// Every plane the fingerprint pins, compared at the BIT.
fn assert_grids_identical(a: &Engine, b: &Engine, label: &str) {
    let (ga, gb) = (a.active_grid(), b.active_grid());
    let planes: [(&str, &[f32], &[f32]); 5] = [
        ("film", &ga.film, &gb.film),
        ("susp", &ga.susp, &gb.susp),
        ("sett", &ga.sett, &gb.sett),
        ("vel_x", &ga.vel_x, &gb.vel_x),
        ("vel_y", &ga.vel_y, &gb.vel_y),
    ];
    for (name, pa, pb) in planes {
        let same = pa
            .iter()
            .zip(pb.iter())
            .all(|(x, y)| x.to_bits() == y.to_bits());
        assert!(same, "{label}: plane {name} diverged");
    }
    for (name, pa, pb) in [
        ("susp_rgb", &ga.susp_rgb, &gb.susp_rgb),
        ("sett_rgb", &ga.sett_rgb, &gb.sett_rgb),
    ] {
        let same = pa.iter().zip(pb.iter()).all(|(x, y)| {
            x.iter()
                .zip(y.iter())
                .all(|(c, d)| c.to_bits() == d.to_bits())
        });
        assert!(same, "{label}: plane {name} diverged");
    }
    assert_eq!(ga.wet, gb.wet, "{label}: wet plane diverged");
}

/// The direct-tool door is the engine's own per-tool dispatch, bit for bit —
/// same §9 mapping, same apply, with the prev handed EXPLICITLY (the host
/// tracks it per symmetry lane; the engine tracks it in `prev_dab_*`, and a
/// fresh engine starts with none, exactly like the door's `None`).
#[test]
fn the_tool_door_matches_the_engines_own_dispatch_bit_for_bit() {
    for tool in [Tool::Wet, Tool::Dry, Tool::Blow, Tool::Smear] {
        let mut a = seeded_engine([220.0, 45.0, 40.0]);
        let mut b = seeded_engine([220.0, 45.0, 40.0]);
        let (p1, p2) = ([40.0, 48.0], [46.0, 50.0]);
        a.tool = tool;
        a.dispatch_pressure_dab(p1[0], p1[1], 5.0, 1.0, 0.0, 12.0);
        a.dispatch_pressure_dab(p2[0], p2[1], 5.0, 1.0, 0.0, 12.0);
        b.dispatch_pressure_dab_tool(tool, p1[0], p1[1], 5.0, 1.0, 0.0, 12.0, None);
        b.dispatch_pressure_dab_tool(tool, p2[0], p2[1], 5.0, 1.0, 0.0, 12.0, Some(p1));
        assert_grids_identical(&a, &b, &format!("{tool:?}"));
    }
}

/// The blend lane door REMIXES settled paint (the tool's defining power —
/// t07's oracle on the LANE plumbing): after blending across the boundary of
/// two dried patches, the red patch's settled colour has moved toward blue.
/// The mutation this kills: the door dispatching the PAINT accumulate (which
/// deposits suspension and never touches `sett_rgb`).
#[test]
fn the_blend_lane_door_remixes_settled_paint() {
    let mut e = Engine::new(W, H);
    // Patch A (red) and patch B (blue), side by side, then settled hard.
    for (x0, color) in [(20.0, [220.0f64, 45.0, 40.0]), (48.0, [10.0, 70.0, 150.0])] {
        e.color = color;
        e.begin_direct_stroke(0, x0, 48.0);
        let mut x = x0;
        while x < x0 + 24.0 {
            x += 4.0;
            e.direct_segment(0, 4.0);
            e.dispatch_pressure_dab_lane(0, x, 48.0, 6.0, 1.0, 0.0, 10.0, None, None);
        }
        e.end_direct_stroke();
    }
    e.action_dry_canvas();
    // Oracle = patch A's MASS-WEIGHTED settled mean colour: the blend mask is
    // sparse (only bristle core tips beat the paper gate), so any single cell
    // may sit between the remixed lanes — the regional mean cannot.
    let mean_a = |e: &Engine| -> (f64, [f64; 3]) {
        let g = e.active_grid();
        let (mut mass, mut acc) = (0.0f64, [0.0f64; 3]);
        for cy in 40..=56 {
            for cx in 30..46 {
                let i = cx + 1 + (cy + 1) * g.s;
                let m = g.sett[i] as f64;
                mass += m;
                for c in 0..3 {
                    acc[c] += g.sett_rgb[i][c] as f64 * m;
                }
            }
        }
        if mass > 0.0 {
            for c in &mut acc {
                *c /= mass;
            }
        }
        (mass, acc)
    };
    let (mass_before, before) = mean_a(&e);
    assert!(mass_before > 0.0, "fixture: patch A must hold settled paint");
    // Blend across the boundary through the lane door.
    e.tool = Tool::Blend;
    e.begin_direct_stroke(0, 32.0, 48.0);
    let mut x = 32.0;
    while x < 60.0 {
        x += 4.0;
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane_blend(0, x, 48.0, 6.0, 1.0, 0.0, 12.0);
    }
    e.end_direct_stroke();
    let (_, after) = mean_a(&e);
    assert!(
        after != before,
        "blend must remix DRIED paint (sett_rgb untouched => the door fed the paint trail)"
    );
    assert!(
        after[2] > before[2],
        "patch A's settled colour must move toward the blue neighbour (b {:.3} -> {:.3})",
        before[2],
        after[2]
    );
}

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

/// Visual terms OFF = the historical pigment render, byte for byte — with
/// `p` absent (the wrapper) AND with `p` present (a caller holding live
/// params but no flags): the gating is the FLAGS, never the params.
#[test]
fn the_visual_render_off_is_the_pigment_render_bit_for_bit() {
    let e = seeded_engine([220.0, 45.0, 40.0]);
    let params = e.sim.gather_params(&e.tuning);
    let layers = layers_of(&e);
    let mut base = vec![0u8; W * H * 4];
    let mut off_none = vec![1u8; W * H * 4];
    let mut off_some = vec![2u8; W * H * 4];
    render_pigment_only_region(&layers, 1, 1, W, H, &mut base);
    render_pigment_region_visual(
        None,
        &layers,
        PigmentVisual::default(),
        1,
        1,
        W,
        H,
        &mut off_none,
    );
    render_pigment_region_visual(
        Some(&params),
        &layers,
        PigmentVisual::default(),
        1,
        1,
        W,
        H,
        &mut off_some,
    );
    assert_eq!(base, off_none, "wrapper diverged from the visual body");
    assert_eq!(base, off_some, "params alone must never change the output");
}

/// Paper ON prints the tooth INTO the pigment: colours move where there is
/// paint, alpha never moves anywhere, empty pixels stay empty — and with
/// Visual grain AND Emboss both zeroed the term vanishes exactly (the whole
/// effect is those two knobs, scaled by the master).
#[test]
fn the_paper_visual_prints_the_tooth_into_the_pigment_alpha_intact() {
    let mut e = seeded_engine([220.0, 45.0, 40.0]);
    let params = e.sim.gather_params(&e.tuning);
    let layers = layers_of(&e);
    let paper_on = PigmentVisual {
        paper: true,
        km_glaze: false,
    };
    let mut off = vec![0u8; W * H * 4];
    let mut on = vec![0u8; W * H * 4];
    render_pigment_only_region(&layers, 1, 1, W, H, &mut off);
    render_pigment_region_visual(Some(&params), &layers, paper_on, 1, 1, W, H, &mut on);
    let mut rgb_moved = false;
    for px in 0..W * H {
        let o = px * 4;
        assert_eq!(off[o + 3], on[o + 3], "alpha must be pure pigment coverage");
        if off[o + 3] == 0 {
            assert_eq!(
                &off[o..o + 4],
                &on[o..o + 4],
                "an empty pixel must stay empty"
            );
        } else if off[o..o + 3] != on[o..o + 3] {
            rgb_moved = true;
        }
    }
    assert!(rgb_moved, "default knobs must print visible grain");
    drop(layers);
    // Zero the two amplitude knobs: the term must vanish to the bit.
    e.set_knob(Knob::VisualGrain, 0.0);
    e.set_knob(Knob::Emboss, 0.0);
    let params = e.sim.gather_params(&e.tuning);
    let layers = layers_of(&e);
    let mut on_zeroed = vec![0u8; W * H * 4];
    render_pigment_region_visual(Some(&params), &layers, paper_on, 1, 1, W, H, &mut on_zeroed);
    assert_eq!(off, on_zeroed, "grain 0 + emboss 0 must be an exact no-op");
}

/// Glaze ON stacks the wet film over the dried paint by reflectance — the
/// colour changes exactly where suspension sits OVER settled paint; alpha is
/// untouched everywhere.
#[test]
fn the_glaze_stacks_the_film_over_the_dried_paint() {
    let mut e = seeded_engine([220.0, 45.0, 40.0]);
    e.action_dry_canvas(); // settle the seed
    // A fresh blue wash over the dried red.
    e.color = [10.0, 70.0, 150.0];
    e.begin_direct_stroke(0, 30.0, 48.0);
    let mut x = 30.0;
    while x < 66.0 {
        x += 4.0;
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane(0, x, 48.0, 5.0, 1.0, 0.0, 10.0, None, None);
    }
    e.end_direct_stroke();
    let params = e.sim.gather_params(&e.tuning);
    let layers = layers_of(&e);
    let glaze = PigmentVisual {
        paper: false,
        km_glaze: true,
    };
    let mut off = vec![0u8; W * H * 4];
    let mut on = vec![0u8; W * H * 4];
    render_pigment_only_region(&layers, 1, 1, W, H, &mut off);
    render_pigment_region_visual(Some(&params), &layers, glaze, 1, 1, W, H, &mut on);
    let g = e.active_grid();
    let mut moved = false;
    for cy in 1..=H {
        for cx in 1..=W {
            let i = cx + cy * g.s;
            let o = ((cy - 1) * W + (cx - 1)) * 4;
            assert_eq!(off[o + 3], on[o + 3], "alpha must not move under glaze");
            if g.sett[i] > 0.0 && g.susp[i] > 0.0 && off[o..o + 3] != on[o..o + 3] {
                moved = true;
            }
        }
    }
    assert!(moved, "glaze must change the film-over-dried-paint stacking");
}
