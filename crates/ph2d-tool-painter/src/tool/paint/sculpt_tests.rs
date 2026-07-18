//! Gates for **Sculpt** (`docs/Painter/18_plano_sculpt_relevo.md`, Wave 1).
//!
//! Every one of these names the MUTATION that must make it bleed. A green gate whose red you have never
//! seen is not a gate — and on this line the lesson arrived twice as *the comment was wrong, not the
//! code*, so where a claim is made about what a test catches, it was checked by breaking the thing.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};
use std::sync::Arc;

/// One canvas pointer sample at full pressure (the sibling `tests` module has its own; both are private).
fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A canvas with a layer that already carries relief — the only ground a sculpt brush has anything to say
/// about. `200` is deliberately NOT a multiple of the memo's 64-px tile: the edge tiles are truncated, and
/// a truncated tile is exactly where the blur's edge-clamp has to coincide with the canvas's.
fn sculpt_canvas(size: u32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    let layer = t.layers.active().expect("a layer");
    // A relief with real high-frequency content: a ridge with a saw-tooth on it, so a blur has something
    // to knock down and a sharpen has something to raise. Deterministic (HR-5: no RNG in a gate).
    let n = (size * size) as usize;
    let relief: Vec<f32> = (0..n)
        .map(|i| {
            let x = (i as u32 % size) as f32;
            let y = (i as u32 / size) as f32;
            let ridge = (1.0 - ((x - 100.0).abs() / 60.0)).max(0.0);
            let saw = if (x as u32 + y as u32).is_multiple_of(3) {
                0.18
            } else {
                0.0
            };
            ridge * 0.5 + saw
        })
        .collect();
    t.heights.insert(layer, Arc::new(relief.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();
    (t, layer, relief)
}

/// Put the tool in Sculpt mode through the REAL router, so the fixture cannot drift from the product.
fn arm_sculpt(t: &mut PainterTool, mode: u8, radius_norm: f32, strength: f32) {
    t.set_paint_tool_mode("sculpt");
    assert!(
        t.is_sculpt_mode(),
        "fixture: the router did not enter Sculpt"
    );
    t.set_sculpt_mode(mode);
    t.set_sculpt_radius(radius_norm);
    // The sculpt has no Strength of its own: the BRUSH's is its strength (one knob, not two).
    let mut b = t.paint.brush;
    b.strength = strength;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;
}

/// Drag a stroke across the ridge.
fn drag(t: &mut PainterTool, path: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
    for p in &path[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(*path.last().unwrap(), PointerPhase::Up));
}

fn heights_of(t: &PainterTool, layer: crate::tool::RtLayerId) -> Vec<f32> {
    t.heights
        .get(&layer)
        .map(|h| (**h).clone())
        .unwrap_or_default()
}

/// The total gradient magnitude of a height field — what "smooth" means, as a number.
fn gradient_sum(h: &[f32], size: u32) -> f64 {
    let w = size as usize;
    let mut s = 0.0f64;
    for y in 1..(size as usize - 1) {
        for x in 1..w - 1 {
            let i = y * w + x;
            let gx = f64::from(h[i + 1] - h[i - 1]);
            let gy = f64::from(h[i + w] - h[i - w]);
            s += (gx * gx + gy * gy).sqrt();
        }
    }
    s
}

// ── §4: the gate that DEFINES the design ────────────────────────────────────────────────────────────

/// **The relief is a function of the SHAPE, not of how the artist dragged it there.**
///
/// The shape editors (Line / Curve / Ellipse / Polygon / Free Hand) re-stamp the WHOLE stroke on every
/// pointer move. A sculpt that blurred `h` in place would melt it a little further on each of those
/// frames, so the finished curve would carry the history of the hand that drew it — deeper where the
/// artist hesitated, and with no way back. The artist would see it and have no word for it.
///
/// This is the gate the whole per-stroke design of §4 exists to pass, and on the naive implementation it
/// is born RED.
///
/// **Mutations that must bleed** (both checked, 2026-07-13):
/// * delete the `restamp_reset_sculpt()` call in `stamp_preview::stamp_drag_preview` — each frame's blur
///   composes onto the last and the relief melts;
/// * make `render_sculpt` read `heights` instead of the frozen `pre` — the same melt, from the other end.
#[test]
fn the_relief_is_a_function_of_the_shape_not_of_how_it_was_dragged_there() {
    let size = 200u32;
    let a = [60.0, 100.0];
    let b = [140.0, 100.0]; // where the line ENDS, in both runs
    let c = [90.0, 60.0]; // …but run 2 gets there the long way round
    let d = [170.0, 150.0];

    // Drive the Line editor the way an artist does. It is a POLYLINE BUILDER: each press places a point,
    // and it stamps NOTHING until there are two of them (`line_refill` bails at `path.len() < 2`). The
    // obvious Down+Move version of this test places one point, drags it, stamps nothing, and passes
    // against a deliberately broken engine — which is what the first draft did. The harness has to
    // reproduce the artist's CONTEXT, not just the mechanism.
    let line_tool = |t: &mut PainterTool| {
        arm_sculpt(t, 0, 1.0, 1.0);
        let mut br = t.paint.brush;
        br.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
        t.paint.brush = br;
        t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = br;
    };
    let place = |t: &mut PainterTool, p: [f32; 2]| {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    };

    // Run 1 — the line A→B, drawn straight.
    let (mut t1, layer1, before) = sculpt_canvas(size);
    line_tool(&mut t1);
    place(&mut t1, a);
    place(&mut t1, b);
    let straight = heights_of(&t1, layer1);
    let sculpted = before
        .iter()
        .zip(&straight)
        .filter(|(x, y)| (*x - *y).abs() > 1e-6)
        .count();
    assert!(
        sculpted > 200,
        "fixture: the Line shape sculpted only {sculpted} texels — the editor never stamped, so the \
         comparison below would be between two no-ops and would pass on ANY engine"
    );

    // Run 2 — the SAME line A→B, but the endpoint is dragged there through a detour. Every frame of that
    // drag re-stamps the whole shape, which is precisely the hazard: a sculpt that blurred `h` in place
    // would melt the relief a little more on each of those frames, and the finished line would carry the
    // history of the artist's hand instead of the shape they actually drew.
    //
    // (The detour does NOT return to the grab's own origin: `line_move` treats a move back inside the
    // slop as a TAP, not a drag, and swallows it — so "drag away and come back" never re-stamps at all,
    // and a gate written that way tests nothing. Ask the tool what it does before asserting what it did.)
    let (mut t2, layer2, _) = sculpt_canvas(size);
    line_tool(&mut t2);
    place(&mut t2, a);
    place(&mut t2, c);
    t2.on_canvas_pointer(cp(c, PointerPhase::Down)); // grab the endpoint…
    t2.on_canvas_pointer(cp(d, PointerPhase::Move)); // …drag it far away…
    t2.on_canvas_pointer(cp(b, PointerPhase::Move)); // …and land it exactly where run 1 put it
    t2.on_canvas_pointer(cp(b, PointerPhase::Up));
    let dragged = heights_of(&t2, layer2);

    let differing = straight
        .iter()
        .zip(&dragged)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "the same line, reached two different ways, left two different reliefs ({differing} texels). \
         The sculpt is carrying the HISTORY of the drag instead of the shape: each re-stamp melted the \
         relief a little further, and there is no way back. A dab must accumulate an INTENSITY and the \
         relief must be re-rendered from the FROZEN `pre` — never blurred in place (doc 18 §4)."
    );
}
/// **A faster mouse must not sculpt deeper.**
///
/// The second reason the per-stroke design exists (doc 18 §4.2), and the one the shape-editor gate above
/// does NOT reach — which is how this one came to be written. The `pre`→`target` mutation stayed GREEN
/// against that gate, and the explanation is worth more than the test: a re-stamp *restores* the relief
/// first, so on that path reading the live plane and reading the frozen one are literally the same read.
///
/// They come apart on a **freehand** stroke, where dabs from different pointer batches land on the same
/// texel and nothing restores anything in between. Read the live relief and each batch lerps from wherever
/// the last one left off:
///
/// ```text
///   frozen source:   h = pre + a·(B − pre)            (a = the touch accumulated so far)
///   live relief:     h = h   + a·(B − h)              → compounds, once per BATCH
/// ```
///
/// Both crawl toward the same `B` (the blur of the frozen source — the memo makes that a constant, which
/// is what already kills the classic diffusion blow-up). The difference is the RAMP: the live version
/// smooths further the more batches arrive. And the number of batches is the POINTER POLLING RATE. So a
/// 1000 Hz mouse would sculpt visibly deeper than a 125 Hz one, for the same gesture, at the same
/// settings — a bug the artist would feel constantly and never once be able to describe.
///
/// So the gate is exactly that sentence: the same geometry, delivered coarsely and delivered finely, must
/// leave the same relief. Byte for byte, because the dab list is identical either way (the stroke engine
/// spaces dabs by DISTANCE — only the batching differs), so anything less than equality is the batching
/// leaking into the result.
///
/// **Mutation that must bleed:** in `render_sculpt`, read `target[i]` (the live relief) instead of
/// `pre[i]` (the frozen source).
#[test]
fn a_faster_mouse_does_not_sculpt_deeper() {
    let size = 200u32;
    let a = [60.0, 100.0];
    let b = [140.0, 100.0];

    let run = |samples: u32| -> (Vec<f32>, Vec<f32>) {
        let (mut t, layer, before) = sculpt_canvas(size);
        // A LIGHT touch, on purpose. The dabs sum, so a firm stroke saturates almost every texel it
        // covers (`amount >= 1`) — and at saturation both the right kernel and the broken one have
        // arrived at the same blur, so the bug hides. The partial regime is where the ramp is visible,
        // and a gate has to live where the difference lives.
        arm_sculpt(&mut t, 0, 1.0, 0.06);
        let mut path = vec![a];
        for k in 1..=samples {
            let f = k as f32 / samples as f32;
            path.push([a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]);
        }
        drag(&mut t, &path);
        (before, heights_of(&t, layer))
    };

    // The SAME straight line, reported by a lazy mouse and by a frantic one.
    let (before, coarse) = run(2);
    let (_, fine) = run(32);

    let sculpted = before
        .iter()
        .zip(&coarse)
        .filter(|(x, y)| (*x - *y).abs() > 1e-6)
        .count();
    assert!(
        sculpted > 200,
        "fixture: the stroke sculpted only {sculpted} texels — two no-ops always match"
    );

    let differing = coarse
        .iter()
        .zip(&fine)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "the same gesture, sampled 2× and sampled 32×, left two different reliefs ({differing} texels). \
         The kernel is reading the relief the previous BATCH wrote, so the depth of the sculpt is set by \
         the artist's polling rate. It has to be a function of the accumulated intensity and the FROZEN \
         source, and of nothing else (doc 18 §4.2)."
    );
}

// ── §5: the sculpt writes the relief, and ONLY the relief ───────────────────────────────────────────

/// **Nothing but `heights` moves.**
///
/// Scraping the BODY of the paint does not take its PIGMENT (§5) — a sculpt that erased colour would just
/// be the eraser, wearing a spatula's name. And it deposits no coverage and no material either: it creates
/// no paint, it reshapes the paint that is there.
///
/// The absent half of this gate is the dangerous half: *"the colour did not change"* is trivially true of
/// a sculpt that does NOTHING. So the relief-moved assert below is not a bonus — it is what stops this
/// whole test from being vacuous ([[feedback_absence_gate_needs_a_presence_sibling]]).
///
/// **Mutation that must bleed:** remove the `return` after `stamp_dabs_sculpt` in `stamp_dabs_inner` —
/// the colour routes run and the canvas fills with brush colour.
#[test]
fn the_sculpt_writes_the_relief_and_nothing_else() {
    let size = 200u32;
    let (mut t, layer, before_h) = sculpt_canvas(size);
    let before_rgba = (*t.canvas_rgba).clone();
    let before_covers = t.covers.get(&layer).map(|c| (**c).clone()).unwrap();
    let before_mats = t.mats.get(&layer).map(|m| (**m).clone());

    arm_sculpt(&mut t, 0, 0.5, 1.0);
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);

    // PRESENCE first — without this the three "unchanged" asserts below are green on a dead tool.
    let after_h = heights_of(&t, layer);
    let relief_moved = before_h
        .iter()
        .zip(&after_h)
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        relief_moved > 500,
        "the sculpt reshaped only {relief_moved} texels — it did essentially nothing, so every \
         'unchanged' assertion below would pass for the wrong reason"
    );

    assert!(
        (*t.canvas_rgba) == before_rgba,
        "the sculpt wrote PIGMENT. It has none to write: scraping the body of the paint does not take \
         its colour (doc 18 §5), and a sculpt that erased colour would just be the eraser."
    );
    assert_eq!(
        t.covers.get(&layer).map(|c| (**c).clone()).unwrap(),
        before_covers,
        "the sculpt wrote COVERAGE — it created paint where it only meant to reshape it"
    );
    assert_eq!(
        t.mats.get(&layer).map(|m| (**m).clone()),
        before_mats,
        "the sculpt wrote MATERIAL — the spatula changed what the paint IS, not just its shape"
    );
}

/// **Sculpting bare paper lights nothing.**
///
/// The light weighs its shading by the coverage (the paint's body), so relief standing over zero coverage
/// does not shade — and a Smooth that reaches past the edge of a stroke DOES push relief out onto bare
/// canvas. That is right, and this pins that it stays right: sculpt across the boundary, and the paper
/// must come out of the composite exactly as it went in.
///
/// Again with its presence sibling: the PAINTED side has to actually change, or "the paper is untouched"
/// is a statement about a tool that does nothing.
#[test]
fn the_sculpt_does_not_light_bare_paper() {
    let size = 200u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 14.0,
        color: [0.9, 0.1, 0.1],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.8,
        impasto_smoothing: 0.0,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    // Paint a stroke down the LEFT half only — the right half stays bare paper.
    drag(&mut t, &[[60.0, 40.0], [60.0, 100.0], [60.0, 160.0]]);
    let layer = t.layers.active().expect("a layer");
    assert!(
        t.heights.contains_key(&layer),
        "fixture: the stroke laid relief"
    );
    t.paint.impasto_show = true;
    t.invalidate_composite();
    let before = {
        let (c, _, _) = t.take_preview_arc().expect("preview");
        (*c).clone()
    };
    let bare_before = (*t.canvas_rgba).clone();

    // Now sculpt ACROSS the stroke's right edge, out onto the paper. A big radius, hard, so the blur
    // genuinely pushes relief past the pigment.
    arm_sculpt(&mut t, 0, 1.0, 1.0);
    let mut sb = t.paint.brush;
    sb.radius_px = 20.0;
    t.paint.brush = sb;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = sb;
    drag(&mut t, &[[74.0, 40.0], [74.0, 100.0], [74.0, 160.0]]);
    t.invalidate_composite();
    let after = {
        let (c, _, _) = t.take_preview_arc().expect("preview");
        (*c).clone()
    };

    // PRESENCE: the painted side moved. (Without this the whole gate is green on a dead sculpt.)
    let painted_moved = (0..before.len())
        .step_by(4)
        .filter(|&i| {
            let x = (i / 4) as u32 % size;
            x < 74 && before[i] != after[i]
        })
        .count();
    assert!(
        painted_moved > 100,
        "the sculpt changed the lit appearance of only {painted_moved} painted texels — it is not \
         reshaping anything, so 'the paper stayed clean' below means nothing"
    );

    // ABSENCE: the paper the brush never painted comes out byte-identical, however hard we sculpt over it.
    let mut lit_paper = 0u32;
    let mut paper = 0u32;
    for i in (0..before.len()).step_by(4) {
        // Bare paper = the pigment never touched it (the canvas is still pure white there).
        if bare_before[i] != 255 || bare_before[i + 1] != 255 || bare_before[i + 2] != 255 {
            continue;
        }
        paper += 1;
        if before[i..i + 3] != after[i..i + 3] {
            lit_paper += 1;
        }
    }
    assert!(paper > 10_000, "sanity: the fixture has a large bare field");
    assert_eq!(
        lit_paper, 0,
        "the sculpt lit {lit_paper} texels of BARE PAPER. It pushed relief out past the pigment (which \
         is correct — a blur does that), but relief over zero coverage must not shade: a sculpt brush \
         creates no paint (doc 18 §5)."
    );
}

// ── The two verbs actually do what their names say ──────────────────────────────────────────────────

/// **Smooth lowers the gradient; Sharpen raises it.** One kernel, one sign — so one gate, two directions.
///
/// The volume assert is what stops "smooth" from being satisfied by a kernel that simply flattens
/// everything to zero (which would ace a gradient test and destroy the painting).
///
/// **Mutation that must bleed:** drop the `sharpen` branch in `render_sculpt` (both verbs then smooth) —
/// the Sharpen half goes red while the Smooth half stays green, which is exactly the asymmetry a single
/// combined assert would have hidden.
#[test]
fn smooth_lowers_the_gradient_and_sharpen_raises_it() {
    let size = 200u32;
    let path = [[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]];

    let run = |mode: u8| -> (f64, f64, f64) {
        let (mut t, layer, before) = sculpt_canvas(size);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        drag(&mut t, &path);
        let after = heights_of(&t, layer);
        (
            gradient_sum(&before, size),
            gradient_sum(&after, size),
            after.iter().map(|v| f64::from(v.abs())).sum::<f64>(),
        )
    };

    let (g0, g_smooth, vol_smooth) = run(0);
    let (_, g_sharp, _) = run(1);
    let vol0: f64 = {
        let (_, _, before) = sculpt_canvas(size);
        before.iter().map(|v| f64::from(v.abs())).sum()
    };

    assert!(
        g_smooth < g0 * 0.97,
        "Smooth did not smooth: the total gradient went {g0:.0} → {g_smooth:.0}. It is supposed to pull \
         the relief toward its own local average."
    );
    assert!(
        g_sharp > g0 * 1.03,
        "Sharpen did not sharpen: the total gradient went {g0:.0} → {g_sharp:.0}. It is the Smooth \
         kernel with the sign flipped; if it also went DOWN, the sign is not being read."
    );
    // …and Smooth did it by AVERAGING, not by deleting. A kernel that zeroed the relief would pass the
    // gradient assert with flying colours and wreck the painting.
    assert!(
        vol_smooth > vol0 * 0.9,
        "Smooth destroyed the relief's volume ({vol0:.0} → {vol_smooth:.0}) — it flattened the paint \
         away instead of averaging it"
    );
}

// ── The selection ───────────────────────────────────────────────────────────────────────────────────

/// **The spatula does not reach outside the selection.**
///
/// The gate is written on the ACCUMULATED intensity rather than on the render, because that is where the
/// mask has to bite: gate only the render, and a later Strength/Radius edit would re-render the stroke and
/// leak straight past the marquee — a bug that appears minutes after the stroke and never gets connected
/// back to the selection.
///
/// **Mutation that must bleed:** delete the `restrict` block in `stamp_dabs_sculpt`.
#[test]
fn the_sculpt_respects_the_selection() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);
    let n = (size * size) as usize;
    // Select the LEFT half, through the REAL selection verb — a hand-built mask would be a fixture that
    // can drift away from what the marquee actually produces.
    t.set_rect_selection(0, 0, 100, size);
    assert!(
        t.selection_restricts_paint(),
        "fixture: the selection must actually be restricting, or this gate is about nothing"
    );

    arm_sculpt(&mut t, 0, 0.5, 1.0);
    // A stroke that crosses the marquee: it starts inside and ends well outside.
    drag(&mut t, &[[70.0, 100.0], [110.0, 100.0], [150.0, 100.0]]);

    let after = heights_of(&t, layer);
    let mut inside = 0u32;
    let mut outside = 0u32;
    for i in 0..n {
        if (before[i] - after[i]).abs() <= 1e-6 {
            continue;
        }
        if (i as u32 % size) < 100 {
            inside += 1;
        } else {
            outside += 1;
        }
    }
    assert!(
        inside > 200,
        "the sculpt changed only {inside} texels INSIDE the selection — it is not working at all"
    );
    assert_eq!(
        outside, 0,
        "the sculpt reshaped {outside} texels OUTSIDE the selection — the marquee is supposed to be \
         what the spatula can reach"
    );
}

// The SESSION's lifecycle (born / parked / dead), its byte budget and the perf kill criterion live in
// the child module: same fixtures, different question. This file gates what the kernel DOES to the
// relief; `session` gates the lifetime of the state that lets it.
mod conserve;
mod filter;
mod inflate;
mod inflate_ball_candidate;
mod inflate_edge;
mod inflate_edge_probes;
mod inflate_junction_probes;
mod inflate_matter;
mod inflate_smooth;
mod inflate_support;
mod memo;
mod plane;
mod session;
mod w3;

/// **The sculpt fires in Sculpt mode, and in no other.**
///
/// It hangs off `stamp_dabs_inner`, which every colour route funnels through — so the mode check is the
/// only thing keeping the spatula off a Brush stroke. And one route re-enters that choke point through a
/// side door: **Mask** swaps `canvas_rgba` for a scratch buffer and calls `stamp_dabs_inner` again
/// (`stamp_route::stamp_dabs_mask`). If the sculpt branch fired there, painting a layer mask would carve
/// the relief underneath it — an edit the artist did not ask for, on a channel they are not even looking
/// at.
///
/// So every mode is swept, not just the suspicious one. (Deform / Fill / Selection have their own pointer
/// lifecycles and never reach here at all; they are in the list anyway, because "never reaches here" is a
/// fact about today's routing.)
///
/// ## The oracle had to be split from the claim (2026-07-18)
///
/// This used to prove its point by asserting that **no relief texel moved** in any non-Sculpt mode — and
/// that was the right assertion only while the Smear's Plow defaulted to `0.0`. Now that a knife carries
/// the paint's body by default, the Smear moves relief legitimately, and the old assertion would have to
/// be either deleted (losing the Mask side-door guard) or the default reverted (losing the fix).
///
/// Neither: *"did the SCULPT fire"* and *"did any relief move"* are different questions that happened to
/// have the same answer. The claim is the first one, so it is asked directly — `sculpt.layer.is_none()`,
/// for every mode, always. The Smear then gets BOTH halves, which is strictly more than the sweep could
/// say before: with the Plow off not one texel may move (so nothing is reaching the relief through that
/// route except the knife), and with the Plow on relief MUST move while the session stays closed (so what
/// moved it was the knife and not the spatula).
///
/// **Mutation that must bleed:** hoist the sculpt call out of the `matches!(…, PaintMode::Sculpt)` guard
/// — or move it up into `stamp_dabs`, above the Mask route's scratch swap.
#[test]
fn no_other_paint_mode_touches_the_relief() {
    /// One mode, dragged across the ridge. Returns how many relief texels moved and whether a SCULPT
    /// session was opened — the two questions this gate keeps apart.
    fn sweep(mode: &str, plow: Option<f32>) -> (usize, bool) {
        let size = 100u32;
        let (mut t, layer, before) = sculpt_canvas(size);
        t.set_paint_tool_mode(mode);
        let mut b = t.paint.brush;
        b.strength = 1.0;
        if let Some(p) = plow {
            b.impasto_plow = p;
        }
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        drag(&mut t, &[[30.0, 50.0], [50.0, 50.0], [70.0, 50.0]]);
        let after = heights_of(&t, layer);
        let moved = before
            .iter()
            .zip(&after)
            .filter(|(x, y)| x.to_bits() != y.to_bits())
            .count();
        (moved, t.paint.sculpt.layer.is_some())
    }

    for mode in [
        "brush",
        "eraser",
        "smear",
        "blur",
        "clone",
        "mask",
        "inpaint",
        "fill",
        "selection",
        "deform",
    ] {
        // The Smear is the one mode with a LEGITIMATE claim on the relief — the Plow, the palette knife.
        // Disarm it and the sweep asks its real question of that route too.
        let (moved, session) = sweep(mode, (mode == "smear").then_some(0.0));
        assert_eq!(
            moved, 0,
            "the `{mode}` tool reshaped {moved} texels of RELIEF. Only Sculpt may (and the Smear's Plow, \
             which is disarmed here) — the Mask route is the one to suspect first: it swaps the canvas \
             for a scratch and re-enters the same choke point the sculpt hangs off."
        );
        assert!(!session, "the `{mode}` tool opened a sculpt session");
    }

    // …and the half the old sweep could not say: with the Plow at its DEFAULT the knife moves relief, and
    // it is still not the sculpt doing it. Without this, reverting the default to `0.0` would leave every
    // assertion above green.
    let (moved, session) = sweep("smear", None);
    assert!(
        moved > 100,
        "with the Plow at its default the knife must carry the paint's BODY — it moved {moved} texels"
    );
    assert!(
        !session,
        "…and it must do that through the Plow, not by opening a sculpt session"
    );
}
