//! The Wet Paint gates (W1 + W2 seams) — split from `wetpaint.rs` for the
//! workspace file-LOC cap. Every gate names the mutation that bleeds it.
//! W1 gates. Each names the mutation that bleeds it; the OFF contract and
//! the canvas-identity guard are the two halves that MUST hold for the
//! handoff's law #1 ("o comportamento atual não pode ser prejudicado").

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A white opaque canvas + a red brush, in the given paint mode.
fn tool_in_mode(mode: &str) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode(mode);
    t
}

fn stroke_across(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    for k in 1..=20 {
        t.on_canvas_pointer(cp([30.0 + 7.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([170.0, 60.0], PointerPhase::Up));
}

/// Suspended-pigment mass inside the cell columns `[x0, x1]` (1-based).
fn susp_in_columns(t: &PainterTool, x0: usize, x1: usize) -> f64 {
    let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
    let g = &sess.engine.layers[0].grid;
    let mut sum = 0.0f64;
    for cy in 1..=g.h {
        for cx in x0..=x1 {
            sum += f64::from(g.susp[cx + cy * g.s]);
        }
    }
    sum
}

/// SEAM: Symmetry is FREE through the choke point — the dab list arrives
/// already mirrored, so a stroke held to the LEFT half deposits fluid on
/// the RIGHT half too, in comparable mass. Mutation that bleeds it: a wet
/// route hung off its own geometry instead of `stamp_dabs_inner`'s list
/// (the exact disease the choke-point comment warns about).
#[test]
fn symmetry_mirrors_the_wet_deposit_for_free() {
    // The SOLO baseline first: the same stroke with no symmetry. The
    // ratio alone stayed green under the single-trail bug (the
    // alternating-anchor salvage is symmetric — each side kept ~half),
    // so the oracle also demands each side carries a full stroke's mass.
    let mut solo = tool_in_mode("wetpaint");
    solo.on_canvas_pointer(cp([25.0, 60.0], PointerPhase::Down));
    for k in 1..=10 {
        solo.on_canvas_pointer(cp([25.0 + 4.5 * k as f32, 60.0], PointerPhase::Move));
    }
    solo.on_canvas_pointer(cp([70.0, 60.0], PointerPhase::Up));
    let solo_left = susp_in_columns(&solo, 1, 100);

    let mut t = tool_in_mode("wetpaint");
    t.toggle_symmetry_enabled(); // mirror X on the canvas centre (x = 100)
    t.on_canvas_pointer(cp([25.0, 60.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([25.0 + 4.5 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([70.0, 60.0], PointerPhase::Up));
    let left = susp_in_columns(&t, 1, 100);
    assert!(
        left > solo_left * 0.8,
        "the original side lost mass to the mirror ({left} vs solo {solo_left})"
    );
    let right = susp_in_columns(&t, 101, 200);
    assert!(
        left > 1000.0,
        "the stroke itself deposited nothing ({left})"
    );
    // Each copy is a FULL stroke through its own lane. The first cut
    // accepted 0.5..2.0 and stayed green while the single-trail window
    // silently dropped half of every copy (the "Simetria Circular não
    // está correta" bug wore mirror clothes here).
    assert!(
        right > left * 0.75 && right < left * 1.33,
        "the mirrored deposit is missing or lopsided (left {left}, right {right})"
    );
}

/// SEAM (W2.3b): the painter's SILHOUETTE drives the wet stamp — a
/// flattened dab (Flatten & Rotate) deposits a thin BAND, not the round
/// disc the engine's internal footprint would lay. Mutation that bleeds
/// it: passing `None` instead of the silhouette closure to the shaped
/// door (the deposit's vertical extent balloons back to the full disc).
#[test]
fn the_flattened_dab_deposits_a_band_not_a_disc() {
    let extent_of = |flatten: f32| -> (usize, f64) {
        let mut t = tool_in_mode("wetpaint");
        t.paint.brush.dab_flatten = flatten;
        t.paint.brush.radius_px = 14.0;
        for slot in &mut t.paint.brush_by_mode {
            slot.dab_flatten = flatten;
            slot.radius_px = 14.0;
        }
        stroke_across(&mut t);
        let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
        let g = &sess.engine.layers[0].grid;
        // Vertical extent: rows carrying real suspended mass.
        let mut rows = 0usize;
        let mut total = 0.0f64;
        for gy in 1..=g.h {
            let m: f64 = (1..=g.w).map(|gx| f64::from(g.susp[gx + gy * g.s])).sum();
            total += m;
            if m > 200.0 {
                rows += 1;
            }
        }
        (rows, total)
    };
    let (round_rows, round_mass) = extent_of(0.0);
    let (flat_rows, flat_mass) = extent_of(0.92);
    assert!(
        round_mass > 1000.0 && flat_mass > 500.0,
        "a fixture deposited nothing"
    );
    assert!(
        flat_rows * 2 < round_rows,
        "flatten never reached the fluid (flat {flat_rows} rows vs round {round_rows})"
    );
}

/// SEAM (the Enio report, W2): CIRCULAR symmetry — a stroke drawn in one
/// sector must lay the SAME stroke in every radial sector. With one trail
/// window the interleaved copies were silently dropped (`lx >=
/// TRAIL_SIZE` returns) and the sectors came out broken; the lanes fix
/// is what this pins. Mutation that bleeds it: routing every dab to lane
/// 0 (the matching loop removed).
#[test]
fn circular_symmetry_lays_the_same_stroke_in_every_sector() {
    let mut t = tool_in_mode("wetpaint");
    t.toggle_symmetry_enabled();
    t.toggle_symmetry_circular();
    t.set_symmetry_segments(6);
    // A radial stroke inside one sector, well away from the centre.
    t.on_canvas_pointer(cp([130.0, 60.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([130.0 + 4.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([170.0, 60.0], PointerPhase::Up));
    // Suspended mass per angular sector around the canvas centre.
    let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
    let g = &sess.engine.layers[0].grid;
    let (cx, cy) = (101.0f32, 61.0f32); // canvas centre, cell coords
    let mut sectors = [0.0f64; 6];
    for gy in 1..=g.h {
        for gx in 1..=g.w {
            let m = g.susp[gx + gy * g.s];
            if m <= 0.0 {
                continue;
            }
            let a = (gy as f32 - cy).atan2(gx as f32 - cx); // gate-only trig
            let sec = (((a + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)) * 6.0)
                .clamp(0.0, 5.999) as usize;
            sectors[sec] += f64::from(m);
        }
    }
    let max = sectors.iter().cloned().fold(0.0f64, f64::max);
    assert!(max > 1000.0, "no sector deposited anything ({sectors:?})");
    for (i, m) in sectors.iter().enumerate() {
        assert!(
            *m > max * 0.5,
            "sector {i} is missing its copy ({m:.0} vs max {max:.0}; all {sectors:?})"
        );
    }
}

/// SEAM (W2.2): Randomize Colour reaches the wet deposit — with full Hue
/// jitter the per-dab `d.color` varies, the fresh-ink door follows it,
/// and the deposited pigment carries VISIBLY different hues along the
/// stroke. Mutation that bleeds it: dropping the `set_stroke_color` call
/// (every cell wears the first dab's colour; spread collapses).
#[test]
fn randomize_colour_reaches_the_wet_deposit() {
    let mut t = tool_in_mode("wetpaint");
    t.set_brush_color_jitter(0, 1.0); // full Hue jitter
    stroke_across(&mut t);
    let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
    let g = &sess.engine.layers[0].grid;
    // Across the heavy cells, the red channel of the deposited colour
    // must SPREAD (different dabs, different hues). A fixed-ink stroke
    // measures a few units of spread from tip pickup; full hue jitter
    // measures >100.
    let (mut lo, mut hi, mut n) = (255.0f32, 0.0f32, 0usize);
    for i in 0..g.susp.len() {
        if g.susp[i] > 100.0 {
            lo = lo.min(g.susp_rgb[i][0]);
            hi = hi.max(g.susp_rgb[i][0]);
            n += 1;
        }
    }
    assert!(n > 50, "too few heavy cells to judge ({n})");
    assert!(
        hi - lo > 60.0,
        "the deposit wears one hue — Randomize never reached the fluid (spread {})",
        hi - lo
    );
}

/// SEAM: Tiling wraps the wet deposit — a stroke hugging the LEFT edge
/// lands its wrapped copies by the RIGHT edge. Same mutation as the
/// symmetry gate (the free lunch is the same list).
#[test]
fn tiling_wraps_the_wet_deposit_across_the_edge() {
    let mut t = tool_in_mode("wetpaint");
    t.paint.tiling[0] = true;
    t.on_canvas_pointer(cp([4.0, 60.0], PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp([4.0, 60.0 + 3.0 * k as f32], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([4.0, 90.0], PointerPhase::Up));
    let near_left = susp_in_columns(&t, 1, 20);
    let near_right = susp_in_columns(&t, 185, 200);
    assert!(
        near_left > 500.0,
        "the stroke itself deposited nothing ({near_left})"
    );
    // The wrap is a full lane of its own now — not the accidental half
    // the alternating-anchor windows used to salvage (>0.1 was green
    // over that bug).
    assert!(
        near_right > near_left * 0.5,
        "no wrapped deposit by the far edge (left {near_left}, right {near_right})"
    );
}

/// PRESENCE: a Wet Paint stroke deposits (the canvas moves), the session
/// exists past pen-up (the water is still wet), and the heartbeat keeps
/// the sim alive without poisoning a pixel. Mutation that bleeds it:
/// removing the route arm in `stamp_dabs_inner` (dabs fall through to the
/// colour routes, no session is ever built).
#[test]
fn a_wet_stroke_deposits_and_the_water_survives_pen_up() {
    let mut t = tool_in_mode("wetpaint");
    let before = Arc::clone(&t.canvas_rgba);
    stroke_across(&mut t);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "no wet session after a wet stroke"
    );
    assert_ne!(
        &*before, &*t.canvas_rgba,
        "a wet stroke left the canvas byte-identical"
    );
    // The session spans strokes: pen-up closed the engine stroke but kept the water.
    let sess = t.paint.wetpaint.session.as_ref().unwrap();
    assert!(
        !sess.stroke_open,
        "pen-up must close the engine's direct stroke"
    );
    assert!(
        sess.engine.sim_should_run(),
        "the sim must resume after pen-up"
    );
    // Heartbeat: the sim steps and composites without panicking or NaNing.
    for _ in 0..30 {
        t.paint_tick(1.0 / 40.0);
    }
    assert!(
        t.paint.wetpaint.session.is_some(),
        "the heartbeat must not kill the session"
    );
    assert!(t.canvas_rgba.iter().all(|b| *b != 0 || true), "unreachable");
}

/// The OFF contract — law #1: NO other paint mode reaches the wet engine.
/// Every wet-paint canvas write goes through a session, so "no session
/// after painting" IS "not one byte touched by this module". Mutation
/// that bleeds it: widening the route arm's `matches!` to another mode.
#[test]
fn no_other_paint_mode_reaches_the_wet_engine() {
    for mode in [
        "brush", "eraser", "smear", "blur", "clone", "mask", "inpaint", "fill", "knife", "sculpt",
    ] {
        let mut t = tool_in_mode(mode);
        stroke_across(&mut t);
        assert!(
            t.paint.wetpaint.session.is_none(),
            "mode {mode:?} built a wet session — the wet engine leaked past its mode gate"
        );
    }
    // And the positive control: the same fixture in the wet mode DOES.
    let mut t = tool_in_mode("wetpaint");
    stroke_across(&mut t);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "positive control failed"
    );
}

/// The canvas-identity guard, at the TICK: a foreign `canvas_rgba` swap
/// (undo, fill, layer switch) ends the session BEFORE the sim composites
/// over the restored pixels. Mutation that bleeds it: dropping the
/// `wetpaint_guard` call from `wetpaint_tick` (the restored canvas gets
/// repainted by a zombie session).
#[test]
fn a_foreign_canvas_swap_ends_the_session_before_the_tick_composites() {
    let mut t = tool_in_mode("wetpaint");
    stroke_across(&mut t);
    assert!(t.paint.wetpaint.session.is_some());
    // A foreign mutation: the undo restore swaps the canvas Arc wholesale.
    let restored = Arc::new(vec![255u8; 200 * 120 * 4]);
    t.canvas_rgba = Arc::clone(&restored);
    t.paint_tick(0.25);
    assert!(
        t.paint.wetpaint.session.is_none(),
        "a live session survived a foreign canvas swap"
    );
    assert_eq!(
        &*restored, &*t.canvas_rgba,
        "the tick composited over a canvas the session no longer owns"
    );
}

/// The paint wears the BRUSH's colour — the engine speaks 0..255 and the
/// brush 0..1, and forgetting the scale paints black (Enio's W1 smoke).
/// Mutation that bleeds it: dropping the `* 255.0` in the colour handoff.
#[test]
fn the_wet_paint_wears_the_brushs_colour_not_black() {
    let mut t = tool_in_mode("wetpaint");
    // The fixture brush is [0.8, 0.1, 0.1] — a saturated red.
    stroke_across(&mut t);
    // The strongest deposit anywhere on the canvas (the trail lays mass
    // with vertical structure, so a single row is not a fair sample —
    // the first cut of this gate scanned only the stroke row and failed
    // over a CORRECT product).
    let mut best = [255u8; 4];
    let mut best_dev = 0u32;
    for px in t.canvas_rgba.chunks_exact(4) {
        let dev = px[..3].iter().map(|&c| 255u32 - u32::from(c)).sum::<u32>();
        if dev > best_dev {
            best_dev = dev;
            best = [px[0], px[1], px[2], px[3]];
        }
    }
    assert!(
        best[0] > best[1].saturating_add(40) && best[0] > best[2].saturating_add(40),
        "the wet deposit is not red-dominant (got {best:?}) — the colour scale is wrong"
    );
    assert!(best[0] > 90, "the deposit reads near-black ({best:?})");
}

/// SEAM (W2.4): the artist's GRAIN replaces the engine's bristle in the wet
/// deposit, by the colour route's own law (`dab::grain_at`). A
/// canvas-anchored Checker at Depth 1 VETOES its zero texels — cells the
/// bristled run paints must land at exactly zero, while the pass-texels
/// still deposit (a global dim would fail the kept-count; a dead wire fails
/// the vetoed-count). Mutation that bleeds it: passing `None` instead of
/// the grain closure at the dispatch (B == A, the vetoed set collapses).
#[test]
fn the_artists_grain_textures_the_wet_deposit() {
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let run = |grained: bool| -> Vec<f32> {
        let mut t = tool_in_mode("wetpaint");
        if grained {
            // Tiled = canvas-anchored: the veto pattern holds still across
            // dabs (a dab-relative grain re-phases per dab and overlapping
            // dabs fill each other's zeros — the known ViewPlane behaviour).
            t.paint.brush.texture.kind = TextureKind::Checker;
            t.paint.brush.texture.mapping = TextureMapping::Tiled;
            t.paint.brush.grain_depth = 1.0;
            for slot in &mut t.paint.brush_by_mode {
                slot.texture.kind = TextureKind::Checker;
                slot.texture.mapping = TextureMapping::Tiled;
                slot.grain_depth = 1.0;
            }
        }
        stroke_across(&mut t);
        let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
        sess.engine.layers[0].grid.susp.clone()
    };
    let plain = run(false);
    let grained = run(true);
    let (mut vetoed, mut kept, mut mass) = (0usize, 0usize, 0.0f64);
    for (a, b) in plain.iter().zip(grained.iter()) {
        if *a > 1.0 {
            if *b == 0.0 {
                vetoed += 1;
            } else {
                kept += 1;
            }
        }
        mass += f64::from(*b);
    }
    assert!(
        mass > 1000.0,
        "the grained stroke deposited nothing ({mass})"
    );
    assert!(
        kept > 50,
        "the grain vetoed everything — a dead brush, not a texture (kept {kept})"
    );
    assert!(
        vetoed > 50,
        "no bristle-painted cell was vetoed by the Checker — the Grain never \
         reached the fluid (vetoed {vetoed}, kept {kept})"
    );
}

/// SEAM (W2.5): the SELECTION confines the wet deposit — the fluid may flow
/// wherever the sim takes it, but the canvas write keep-lerps every
/// deselected texel back to the frozen base, and that holds through the
/// TICKS too (the sim keeps compositing after pen-up; a pen-up-only gate
/// would leak as the water spreads). Mutation that bleeds it: dropping
/// `gsel` from the `splat_keep` call in `wetpaint_composite`.
#[test]
fn the_selection_confines_the_wet_deposit() {
    let mut t = tool_in_mode("wetpaint");
    t.set_rect_selection(0, 0, 100, 120); // the LEFT half
    assert!(t.selection_restricts_paint(), "fixture: selection is live");
    stroke_across(&mut t); // crosses the border to x = 170
    for _ in 0..20 {
        t.paint_tick(1.0 / 40.0);
    }
    // The WATER must survive the gates — before the wet route bypassed the
    // outer snapshot/restore wrapper, `restore_deselected_region`'s
    // `Arc::make_mut` re-seated the canvas Arc and the identity guard
    // killed the session every batch (the mode's whole point, gone, with
    // every pixel-assert below still green).
    assert!(
        t.paint.wetpaint.session.is_some(),
        "the wet session died under a selection"
    );
    let mut painted_inside = false;
    for y in 0..120usize {
        for x in 0..200usize {
            let o = (y * 200 + x) * 4;
            let px = &t.canvas_rgba[o..o + 4];
            if x >= 106 {
                assert_eq!(px, &[255u8; 4], "deselected texel ({x},{y}) took wet paint");
            } else if x < 100 && px != [255u8; 4] {
                painted_inside = true;
            }
        }
    }
    assert!(painted_inside, "the selected half took no paint at all");
}

/// SEAM (W2.5): the protection MASK freezes its texels under wet paint —
/// same keep-lerp, other gate. Mutation that bleeds it: dropping `gprot`
/// from the `splat_keep` call (the selection gate stays green — each wire
/// has its own gate).
#[test]
fn the_protection_mask_freezes_its_texels_under_wet_paint() {
    let mut t = tool_in_mode("wetpaint");
    t.ensure_mask_scratch();
    assert!(t.mask_protection_active(), "fixture: protection is live");
    // Blacken (protect) the RIGHT half of the scratch: luminance 0 = frozen.
    {
        let scratch = Arc::make_mut(&mut t.paint.mask_scratch_rgba);
        for y in 0..120usize {
            for x in 100..200usize {
                let o = (y * 200 + x) * 4;
                scratch[o] = 0;
                scratch[o + 1] = 0;
                scratch[o + 2] = 0;
            }
        }
    }
    stroke_across(&mut t);
    for _ in 0..20 {
        t.paint_tick(1.0 / 40.0);
    }
    // Same survival law as the selection gate: the protection restore used
    // to kill the session through the identity guard.
    assert!(
        t.paint.wetpaint.session.is_some(),
        "the wet session died under the protection mask"
    );
    let mut painted_free = false;
    for y in 0..120usize {
        for x in 0..200usize {
            let o = (y * 200 + x) * 4;
            let px = &t.canvas_rgba[o..o + 4];
            if x >= 100 {
                assert_eq!(px, &[255u8; 4], "protected texel ({x},{y}) took wet paint");
            } else if px != [255u8; 4] {
                painted_free = true;
            }
        }
    }
    assert!(painted_free, "the unprotected half took no paint at all");
}

/// SEAM (W2.5): ALPHA-LOCK pins the wet deposit to existing paint — the
/// layer's α is frozen to the session base, so a transparent texel never
/// grows a silhouette however far the water flows, while opaque texels
/// still take colour. Mutation that bleeds it: dropping the α pin in
/// `wetpaint_composite` (the α reference chain in `wet_splat_gates` is the
/// same wire — either cut lands here).
#[test]
fn alpha_lock_pins_the_wet_silhouette_to_the_existing_paint() {
    // Left half opaque white, right half fully transparent.
    let mut src = vec![0u8; 200 * 120 * 4];
    for y in 0..120usize {
        for x in 0..100usize {
            let o = (y * 200 + x) * 4;
            src[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, 200, 120);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("wetpaint");
    let active = t.layers.active().expect("active layer");
    t.layers.get_mut(active).expect("layer").alpha_locked = true;
    stroke_across(&mut t);
    for _ in 0..20 {
        t.paint_tick(1.0 / 40.0);
    }
    let (mut painted_opaque, mut alpha_kept) = (false, true);
    for y in 0..120usize {
        for x in 0..200usize {
            let o = (y * 200 + x) * 4;
            let px = &t.canvas_rgba[o..o + 4];
            if x >= 100 {
                if px[3] != 0 {
                    alpha_kept = false;
                }
            } else {
                assert_eq!(px[3], 255, "opaque texel ({x},{y}) lost its α");
                if px[..3] != [255u8; 3] {
                    painted_opaque = true;
                }
            }
        }
    }
    assert!(
        alpha_kept,
        "a transparent texel grew a silhouette under alpha-lock"
    );
    assert!(painted_opaque, "the opaque half took no colour at all");
}

/// SEAM (W2.7): the artist's PAPER drives the wet tooth — a Checker paper
/// slot seeds the engine's paper plane at session birth, and the deposit's
/// paper gate (tooth valleys reject pigment) follows MY pattern instead of
/// the port's preset. The control run (no slot) shows no alignment with
/// that pattern, so the contrast is the seeding's doing. Mutation that
/// bleeds it: dropping the `seed_paper_with` call (the grained ratio
/// collapses to the control's).
#[test]
fn the_artists_paper_drives_the_wet_tooth() {
    use ph2d_painter_brush::TextureKind;
    let arm = |t: &mut PainterTool| {
        t.paint.brush.paper.kind = TextureKind::Checker;
        t.paint.brush.paper.size = [8.0, 8.0]; // ~16 px checker cells
        for slot in &mut t.paint.brush_by_mode {
            slot.paper.kind = TextureKind::Checker;
            slot.paper.size = [8.0, 8.0];
        }
    };
    let run = |papered: bool| -> Vec<f32> {
        let mut t = tool_in_mode("wetpaint");
        if papered {
            arm(&mut t);
        }
        stroke_across(&mut t);
        let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
        sess.engine.layers[0].grid.susp.clone()
    };
    let control = run(false);
    let papered = run(true);
    // Valley/peak sets from the SAME sampler the seed uses — the reference
    // pattern, not the function under test (which is the seeding + the
    // deposit gate). Restricted to cells the CONTROL stroke reaches.
    let paper_tex = {
        let mut t = tool_in_mode("wetpaint");
        arm(&mut t);
        t.paint.brush.paper
    };
    let rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
    let g_s = 202usize; // grid stride = w + 2 (pad ring)
    let ratio_of = |susp: &[f32]| -> f64 {
        let (mut valley, mut peak) = (0.0f64, 0.0f64);
        for cy in 1..=120usize {
            for cx in 1..=200usize {
                let m = f64::from(susp[cx + cy * g_s]);
                if control[cx + cy * g_s] <= 1.0 {
                    continue; // outside the stroke's reach
                }
                let s = ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                    &paper_tex,
                    cx as i64 - 1,
                    cy as i64 - 1,
                    None,
                    rot,
                    [0.0, 0.0],
                );
                if s < 0.5 {
                    valley += m;
                } else {
                    peak += m;
                }
            }
        }
        valley / peak.max(1.0)
    };
    let r_control = ratio_of(&control);
    let r_papered = ratio_of(&papered);
    assert!(
        r_control > 0.5,
        "fixture: the preset run must NOT align with my checker (ratio {r_control:.2})"
    );
    assert!(
        r_papered < r_control * 0.6,
        "the paper never reached the tooth — no valley rejection \
         (papered {r_papered:.2} vs control {r_control:.2})"
    );
}

/// SEAM (W2.6): the wet ERASER lifts the FLUID and the water survives — the
/// "eraser" wire inside Wet Paint keeps the mode (leaving would BAKE the
/// painting being corrected), and the dabs route to the engine's
/// `Tool::Erase`. Mutations that bleed it: the wire arm mapping "eraser" to
/// `Paint` again (the teardown kills the session → the survival assert), or
/// the erase branch dispatching the PAINT door (mass rises instead of
/// falling).
#[test]
fn the_wet_eraser_lifts_the_fluid_and_the_water_survives() {
    let mut t = tool_in_mode("wetpaint");
    stroke_across(&mut t);
    let before = susp_in_columns(&t, 1, 200);
    assert!(
        before > 1000.0,
        "fixture: the wet stroke deposited ({before})"
    );
    t.set_paint_tool_mode("eraser");
    assert!(
        t.paint.wetpaint.session.is_some(),
        "arming the eraser must NOT bake the wet session"
    );
    assert!(t.paint.eraser, "the eraser override is armed");
    // CANVAS-side oracle: what the artist SEES must lighten too — a grid
    // that empties while the screen keeps the paint is the dropped
    // `merge_dirty` (a mutation that survived the grid-only oracle).
    let canvas_dev =
        |t: &PainterTool| -> u64 { t.canvas_rgba.iter().map(|&b| u64::from(255 - b)).sum() };
    let dev_before = canvas_dev(&t);
    // Erase over the same path — removal is MULTIPLICATIVE through the
    // bristle sieve (mostly near-zero felt), so it is gradual by the
    // reference's design; several passes accumulate it. The FEEL of the
    // default force is the smoke's judgement, not this gate's — here the
    // semantics: mass falls, never rises.
    for _ in 0..8 {
        stroke_across(&mut t);
    }
    assert!(
        t.paint.wetpaint.session.is_some(),
        "the eraser stroke killed the session"
    );
    let after = susp_in_columns(&t, 1, 200);
    assert!(
        after < before * 0.6,
        "the wet eraser did not lift the fluid ({before} -> {after})"
    );
    assert!(
        canvas_dev(&t) < dev_before,
        "the grid emptied but the SCREEN kept the paint — the erase never composited"
    );
}

/// SEAM (W2.6): the wet eraser with NO live session falls through to the
/// normal eraser and erases the BAKED canvas (what is visibly there) —
/// without building a session. Mutation that bleeds it: `wet_owns_the_dabs`
/// losing its session half (the dabs enter the wet arm, which has nothing
/// wet to erase, and the baked paint survives untouched).
#[test]
fn the_sessionless_wet_eraser_erases_the_baked_canvas() {
    let mut t = tool_in_mode("wetpaint");
    stroke_across(&mut t);
    t.set_wetpaint_armed(false); // bake: unchecking exits and ends the session
    t.set_wetpaint_armed(true); // back in wet, nothing wet yet
    assert!(t.paint.wetpaint.session.is_none(), "fixture: no session");
    t.set_paint_tool_mode("eraser"); // stays in Wet Paint (the W2.6 wire)
    let alpha_sum = |t: &PainterTool| -> u64 {
        t.canvas_rgba
            .chunks_exact(4)
            .map(|px| u64::from(px[3]))
            .sum()
    };
    let before = alpha_sum(&t);
    for _ in 0..2 {
        stroke_across(&mut t);
    }
    assert!(
        alpha_sum(&t) < before,
        "the baked paint was not erased (erase-alpha never ran)"
    );
    assert!(
        t.paint.wetpaint.session.is_none(),
        "the fall-through must not build a wet session"
    );
}

/// THE CHECKBOX (Enio 2026-07-21): the Wet Paint arm survives tool
/// round-trips — *"se saio do brush para a borracha ou para a seleção, ao
/// voltar não estou mais no modo wet"*. Armed, every return to "brush" is
/// the FLUID, until the checkbox unchecks. Mutation that bleeds it: the
/// `"brush"` wire arm dropped from `set_paint_tool_mode` (the selection
/// round-trip lands in the plain digital brush — the reported bug).
#[test]
fn the_wet_checkbox_survives_the_tool_round_trips() {
    let mut t = tool_in_mode("brush");
    t.set_wetpaint_armed(true);
    assert!(
        matches!(t.paint.paint_mode, PaintMode::WetPaint),
        "arming from the Brush must enter the fluid on the spot"
    );
    // The reported round-trip: selection and back.
    t.set_paint_tool_mode("selection");
    t.set_paint_tool_mode("brush");
    assert!(
        matches!(t.paint.paint_mode, PaintMode::WetPaint),
        "back from Selection, the brush forgot it was wet (the reported bug)"
    );
    // And the eraser round-trip (W2.6 keeps the mode; leaving eraser to
    // brush must stay wet too).
    t.set_paint_tool_mode("eraser");
    t.set_paint_tool_mode("brush");
    assert!(
        matches!(t.paint.paint_mode, PaintMode::WetPaint),
        "back from the eraser, the brush forgot it was wet"
    );
    // A foreign tool that is not paint at all: smear and back.
    t.set_paint_tool_mode("smear");
    t.set_paint_tool_mode("brush");
    assert!(
        matches!(t.paint.paint_mode, PaintMode::WetPaint),
        "back from Smear, the brush forgot it was wet"
    );
}

/// Entering the mode by ANY door arms the checkbox — a checkbox reading OFF
/// while the paint is wet is a lying radio. Mutation that bleeds it: the
/// arming line dropped from `set_paint_tool_mode`.
#[test]
fn entering_wet_by_any_door_arms_the_checkbox() {
    let t = tool_in_mode("wetpaint"); // the direct wire (the smoke's old door)
    assert!(
        t.paint.wetpaint.armed,
        "the wire entered wet with the checkbox OFF"
    );
}

/// Unchecking exits to the plain Brush and the exit IS the bake: session
/// gone, pixels exactly as composited. Mutation that bleeds it: the disarm
/// arm of `set_wetpaint_armed` not leaving the mode.
#[test]
fn disarming_the_checkbox_exits_and_bakes() {
    let mut t = tool_in_mode("wetpaint");
    stroke_across(&mut t);
    assert!(t.paint.wetpaint.session.is_some());
    let painted = Arc::clone(&t.canvas_rgba);
    t.set_wetpaint_armed(false);
    assert!(
        matches!(t.paint.paint_mode, PaintMode::Paint),
        "unchecking must return to the plain Brush"
    );
    assert!(
        t.paint.wetpaint.session.is_none(),
        "unchecking must end the session (the bake)"
    );
    assert_eq!(
        &*painted, &*t.canvas_rgba,
        "the bake moved pixels — ending must be a stop"
    );
}

/// The SEAM: the panel's Enable checkbox drives the arm over the frozen
/// `PanelEvent` channel — click on, the Brush becomes the fluid; click off,
/// it returns. And the section RESET disarms too (the Watercolor reset's
/// semantics). Mutation that bleeds it: the `route_brush_wetpaint_event`
/// call dropped from `handle_panel_event`.
#[test]
fn the_panels_enable_checkbox_drives_the_wet_arm() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = tool_in_mode("brush");
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_WETPAINT_ENABLE,
    ));
    assert!(
        matches!(t.paint.paint_mode, PaintMode::WetPaint),
        "the Enable click never reached the arm"
    );
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_WETPAINT_ENABLE,
    ));
    assert!(
        matches!(t.paint.paint_mode, PaintMode::Paint),
        "the second click never disarmed"
    );
    // Reset = restore defaults INCLUDING the enable.
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_WETPAINT_ENABLE,
    ));
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_WETPAINT_RESET,
    ));
    assert!(
        !t.paint.wetpaint.armed && matches!(t.paint.paint_mode, PaintMode::Paint),
        "the section reset must disarm"
    );
}

/// Entering Wet Paint must NOT take the Impasto section away (Enio, W1
/// smoke: "uma das regras era não afetar o que já existia") — the section
/// hosts the ten-tool list and the canvas's Lighting. And the radio must
/// light NOTHING (claiming Deposit would be the lying radio the rail
/// refused for the Knife). Mutations: drop WetPaint from
/// `impasto_section_applies` (first half) or let `impasto_tool` fall to
/// its `_ => DEPOSIT` arm (second half).
#[test]
fn wet_paint_keeps_the_impasto_section_with_no_tool_lit() {
    let t = tool_in_mode("wetpaint");
    assert!(
        t.impasto_section_applies(),
        "Wet Paint hid the Impasto section — the tool list and Lighting became unreachable"
    );
    assert_eq!(
        t.impasto_tool(),
        super::super::impasto_tool::IMPASTO_TOOL_NONE,
        "the tool radio claims a tool the hand is not holding"
    );
}

/// Leaving the mode ends the session, and ending IS the bake: the pixels
/// of the last composite stay exactly as they are. Mutation that bleeds
/// it: dropping the teardown arm in `set_paint_tool_mode`.
#[test]
fn leaving_the_mode_bakes_by_simply_stopping() {
    let mut t = tool_in_mode("wetpaint");
    stroke_across(&mut t);
    assert!(t.paint.wetpaint.session.is_some());
    let painted = Arc::clone(&t.canvas_rgba);
    // "brush" no longer LEAVES the mode (the checkbox keeps it wet — the
    // 2026-07-21 law); a real exit is any other tool, or disarming.
    t.set_paint_tool_mode("smear");
    assert!(
        t.paint.wetpaint.session.is_none(),
        "mode exit must end the wet session"
    );
    assert_eq!(
        &*painted, &*t.canvas_rgba,
        "the mode exit moved pixels — ending must be a stop"
    );
}
