//! **The instruments** for the Blob's edge — the `#[ignore]`d probes that MEASURE, beside the gates that
//! assert ([`super::inflate_edge`]). Split from them for the file-LOC cap, along a real seam: a gate states
//! a law and must stay green; a probe answers *"what is actually happening?"* and prints.
//!
//! They are written to HUNT, and that is the lesson they carry. Enio's second smoke of the border
//! (2026-07-16) cost a whole round to a probe pointed at the capsule's SPINE — where there is no defect at
//! all; the artefact lives on the rim. So these scan the whole canvas for the biggest neighbour jump and
//! report WHERE it is, instead of trusting the author to pick the texel.
//!
//! What they measured, 2026-07-16 (Depth 1, Filter Layer):
//!
//! ```text
//! the deposit            max neighbour jump 0.38 load     0 texels over 1 load
//! the Blob, taper ON     max neighbour jump 1.39-1.69     188-444 texels over 1 load
//! the Blob, taper OFF    max neighbour jump 4.91          cover cliff back to 255
//! ```
//!
//! The light reads the DERIVATIVE, so a 1.4-load step between two texels IS a wall — that is the staircase,
//! and those are the white gashes in the photograph. Two things follow, and both are counter-intuitive
//! enough to be worth writing down:
//!
//! * **It is not the paint's thickness.** A blob from 4 to 40 loads ramps identically (edge step 62-88).
//! * **It is not the taper.** Disable it and the jump TRIPLES (4.91) and the cover cliffs back to 255 — the
//!   reach cap becomes a cliff with nothing to soften it. The taper is the MITIGATION.
//!
//! The residual is the argmax's **seam**: across it the envelope is continuous (that is what a lower
//! envelope IS), but the DISTANCE to the winning source is not — and the taper is a function of that
//! distance. A lone form facets too (444 texels), so it needs no junction to appear.

use super::inflate_edge::{
    CX, CY, SIZE, covers_of, inflate_the_whole_layer, max_step, radial_cover,
};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};

const INFLATE: u8 = 7;

/// DIAGNOSTIC (2026-07-16, Enio's 2nd smoke) — his arrangement: a vertical capsule crossed by a row of
/// blobs, then Filter Layer + Inflate. Prints the cover and the height down the capsule's spine, hunting
/// for the white gashes and the stepped silhouette.
#[test]
#[ignore = "diagnostic"]
fn diag_enio_capsule_and_blobs() {
    const S: u32 = 400;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    let b = BrushSpec {
        radius_px: 23.0,
        hardness: 0.3,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("a layer");

    t.on_canvas_pointer(cp([200.0, 90.0], PointerPhase::Down));
    let mut y = 100.0;
    while y <= 306.0 {
        t.on_canvas_pointer(cp([200.0, y], PointerPhase::Move));
        y += 8.0;
    }
    t.on_canvas_pointer(cp([200.0, 306.0], PointerPhase::Up));
    for cx in [140.0f32, 200.0, 260.0] {
        t.on_canvas_pointer(cp([cx, 200.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([cx, 200.0], PointerPhase::Up));
    }

    let c0 = covers_of(&t, layer);
    let h0 = heights_of(&t, layer);
    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    t.set_sculpt_depth(1.0);
    assert!(t.filter_sculpt_layer(FilterScope::Layer), "filter ran");
    let c1 = covers_of(&t, layer);
    let h1 = heights_of(&t, layer);

    // Let the MEASUREMENT find the artefact: the biggest neighbour jumps in the height, anywhere.
    let jump = |h: &[f32]| -> (f32, Vec<(u32, u32, f32)>) {
        let mut worst = 0.0f32;
        let mut hot: Vec<(u32, u32, f32)> = Vec::new();
        for y in 81..319u32 {
            for x in 81..319u32 {
                let i = (y * S + x) as usize;
                let dx = (h[i + 1] - h[i]).abs();
                let dy = (h[i + S as usize] - h[i]).abs();
                let d = dx.max(dy);
                worst = worst.max(d);
                if d > 1.0 {
                    hot.push((x, y, d));
                }
            }
        }
        hot.sort_by(|a, b| b.2.total_cmp(&a.2));
        (worst, hot)
    };
    let (w0, hot0) = jump(&h0);
    let (w1, hot1) = jump(&h1);
    println!("height max neighbour jump: BEFORE={w0:.2}  AFTER={w1:.2}");
    println!(
        "texels jumping >1 load:    BEFORE={}  AFTER={}",
        hot0.len(),
        hot1.len()
    );
    println!("worst AFTER (x,y,jump): {:?}", &hot1[..hot1.len().min(8)]);
    // At the worst spot, what does the MATTER say?
    if let Some((x, y, d)) = hot1.first().copied() {
        let i = (y * S + x) as usize;
        println!(
            "at ({x},{y}) jump={d:.2}: cover before={} after={} | height before={:.2} after={:.2}",
            c0[i], c1[i], h0[i], h1[i]
        );
        let row: Vec<u8> = (x - 6..x + 7).map(|xx| c1[(y * S + xx) as usize]).collect();
        let rowh: Vec<i32> = (x - 6..x + 7)
            .map(|xx| h1[(y * S + xx) as usize].round() as i32)
            .collect();
        println!("  cover  across it: {row:?}");
        println!("  height across it: {rowh:?}");
    }
    let lost = (0..c0.len())
        .filter(|i| c0[*i] > 200 && c1[*i] < 200)
        .count();
    println!("cover LOST (was solid, now not): {lost} texels");
}

/// DIAGNOSTIC — is the edge's hardness a function of the paint's THICKNESS? If the self-floor
/// (`own = |Depth|·amount`, a full load laid on EVERY texel by the filter) is what truncates the ball's
/// taper, then thin paint must cliff and thick paint must ramp, on the same code.
#[test]
#[ignore = "diagnostic"]
fn diag_edge_hardness_vs_paint_thickness() {
    for taps in [1usize, 2, 3, 6, 10] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        let b = BrushSpec {
            radius_px: 40.0,
            hardness: 0.3,
            falloff: Falloff::Smooth,
            strength: 1.0,
            color: [0.1, 0.2, 0.3],
            space_attenuation: false,
            impasto: true,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.set_paint_tool_mode("brush");
        t.set_brush_impasto_depth(1.0);
        let layer = t.layers.active().expect("a layer");
        for _ in 0..taps {
            t.on_canvas_pointer(cp([CX, CY], PointerPhase::Down));
            t.on_canvas_pointer(cp([CX, CY], PointerPhase::Up));
        }
        let peak = heights_of(&t, layer)
            .iter()
            .copied()
            .fold(f32::MIN, f32::max);
        inflate_the_whole_layer(&mut t);
        let prof = radial_cover(&covers_of(&t, layer));
        let step = max_step(&prof);
        let tail: Vec<u8> = prof
            .iter()
            .copied()
            .skip_while(|c| *c == 255)
            .take(8)
            .collect();
        println!(
            "taps={taps:2} peak={peak:5.2} loads -> edge max step {step:3}  tail after solid: {tail:?}"
        );
    }
}

/// DIAGNOSTIC — ONE form vs TWO forms competing. If the artefact is the argmax's seam (where two sources
/// contend and the winner flips), a lone blob must be clean and two blobs 60 px apart must crack open
/// exactly between them.
#[test]
#[ignore = "diagnostic"]
fn diag_one_form_vs_two_competing() {
    let build = |centres: &[[f32; 2]]| -> (f32, usize, u8) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        let b = BrushSpec {
            radius_px: 30.0,
            hardness: 0.3,
            falloff: Falloff::Smooth,
            strength: 1.0,
            color: [0.1, 0.2, 0.3],
            space_attenuation: false,
            impasto: true,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.set_paint_tool_mode("brush");
        t.set_brush_impasto_depth(1.0);
        let layer = t.layers.active().expect("a layer");
        for c in centres {
            for _ in 0..2 {
                t.on_canvas_pointer(cp(*c, PointerPhase::Down));
                t.on_canvas_pointer(cp(*c, PointerPhase::Up));
            }
        }
        inflate_the_whole_layer(&mut t);
        let h = heights_of(&t, layer);
        let c = covers_of(&t, layer);
        let (mut worst, mut hot) = (0.0f32, 0usize);
        for y in 5..SIZE - 5 {
            for x in 5..SIZE - 5 {
                let i = (y * SIZE + x) as usize;
                let d = (h[i + 1] - h[i])
                    .abs()
                    .max((h[i + SIZE as usize] - h[i]).abs());
                worst = worst.max(d);
                if d > 1.0 {
                    hot += 1;
                }
            }
        }
        // The coverage cliff anywhere on the mid row.
        let row: Vec<u8> = (5..SIZE - 5)
            .map(|x| c[((SIZE / 2) * SIZE + x) as usize])
            .collect();
        (worst, hot, max_step(&row))
    };
    let (w1, h1, s1) = build(&[[110.0, 110.0]]);
    println!("ONE form   : worst height jump {w1:.2}, texels>1 = {h1:3}, cover max step {s1}");
    let (w2, h2, s2) = build(&[[80.0, 110.0], [140.0, 110.0]]);
    println!("TWO forms  : worst height jump {w2:.2}, texels>1 = {h2:3}, cover max step {s2}");
    let (w3, h3, s3) = build(&[[70.0, 110.0], [150.0, 110.0]]);
    println!("TWO far    : worst height jump {w3:.2}, texels>1 = {h3:3}, cover max step {s3}");
}
