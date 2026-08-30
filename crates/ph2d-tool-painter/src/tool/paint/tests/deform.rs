//! **Deform (Liquify): as duas metades.** O kernel de warp inverso — push, freeze pela seleção,
//! reconstruir, um traço = uma entrada de undo — e o gizmo de Transform: mover, distorcer o canto
//! livre, a malha de warp, o levantar da seleção, e o custo por quadro dos dois.

use super::*;

// ============================================================================
// Deform Wave 2 — Transform gizmo (temperament + Uniform/Free affine warp).
// ============================================================================

/// A `size`×`size` canvas: transparent everywhere except an opaque black square `[x0,x1)×[y0,y1)`. The
/// opaque block gives the Transform gizmo a content bbox smaller than the canvas (so its centre-move handle
/// sits at the block's centre, not the canvas centre).
fn deform_square_canvas(size: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> PainterTool {
    let mut t = PainterTool::default();
    let mut buf = vec![0u8; (size * size * 4) as usize];
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * size + x) * 4) as usize;
            buf[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
    t.set_source(buf, size, size);
    t.set_paint_tool_mode("liquify");
    t.set_shape_grab_tol_px(8.0);
    t
}

#[test]
fn deform_transform_identity_is_byte_identical() {
    // The core guarantee: F == F0 ⇒ M = I ⇒ disp = 0 ⇒ pixels untouched. Entering Transform and a no-op
    // grab (Down+Up with no drag) must leave the canvas byte-for-byte identical.
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    let before = t.canvas_rgba.clone();
    t.set_deform_transform_on(true);
    assert_eq!(
        *t.canvas_rgba, *before,
        "entering Transform alters no pixels"
    );
    let c = [32.0, 32.0]; // the square's centre (centre-move handle)
    t.on_canvas_pointer(cp(c, PointerPhase::Down));
    t.on_canvas_pointer(cp(c, PointerPhase::Up));
    assert_eq!(
        *t.canvas_rgba, *before,
        "a no-op gizmo grab is byte-identical"
    );
}

#[test]
fn deform_transform_move_translates_content() {
    // Grab the centre-move handle and drag +12 px in x → the whole block shifts right by 12 (backward-gather
    // samples `pre` at dst−12).
    let mut t = deform_square_canvas(80, 30, 30, 50, 50); // 20×20 block, centre (40,40)
    t.set_deform_transform_on(true);
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 40.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 80, 42, 40),
        [0, 0, 0, 255],
        "block shifted right by ~12"
    );
    assert_eq!(
        px(&t, 80, 31, 40)[3],
        0,
        "the vacated left edge is transparent"
    );
}

#[test]
fn deform_transform_reset_restores_pixels() {
    // After a Transform move, Reset restores the pristine pre-deform pixels (whole session discarded).
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    let before = t.canvas_rgba.clone();
    t.set_deform_transform_on(true);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, *before, "the transform changed pixels");
    t.deform_reset();
    assert_eq!(
        *t.canvas_rgba, *before,
        "Reset restores the pre-deform pixels"
    );
}

#[test]
fn deform_transform_is_confined_to_the_selection() {
    // With a left-half selection, a Transform move warps only the selected texels; an unselected texel on
    // the right stays byte-identical. The gizmo frame also snaps to the selection bbox, so grabbing its
    // centre grabs the selection's centre.
    let mut t = deform_ramp(64);
    t.set_shape_grab_tol_px(8.0);
    t.set_rect_selection(0, 0, 32, 64); // select the left half (x < 32)
    let right_before = px(&t, 64, 48, 32);
    t.set_deform_transform_on(true);
    // The selection bbox centre is (16, 32) — grab the centre-move handle there and drag +6 in x.
    t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([22.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([22.0, 32.0], PointerPhase::Up));
    assert_ne!(
        px(&t, 64, 16, 32),
        [16, 128, 128, 255],
        "a selected texel is transformed"
    );
    assert_eq!(
        px(&t, 64, 48, 32),
        right_before,
        "an unselected texel is left untouched (Transform confined to the selection)"
    );
}

#[test]
fn deform_transform_lifts_the_selection_and_leaves_a_hole() {
    // Procreate model: selecting a region then Transform LIFTS it into a floating patch (the marquee is
    // consumed) and moving it leaves a transparent hole where it was, the pixels reappearing at the new
    // spot. Here an opaque block is fully selected, then moved +20 in x.
    let mut t = deform_square_canvas(80, 20, 20, 40, 40); // opaque block [20,40)²
    t.set_shape_grab_tol_px(8.0);
    t.set_rect_selection(20, 20, 20, 20); // select exactly the block
    t.set_deform_transform_on(true);
    assert!(
        !t.selection_active(),
        "the selection marquee is consumed by the transform"
    );
    // Grab the block/selection centre (30,30) and drag +20 in x.
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([50.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([50.0, 30.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 80, 30, 30)[3],
        0,
        "the original spot is now a transparent hole"
    );
    assert_eq!(
        px(&t, 80, 50, 30),
        [0, 0, 0, 255],
        "the patch reappears at the moved spot"
    );
}

#[test]
fn deform_transform_distort_warps_a_free_corner() {
    // Distort sub-mode: entering it is byte-identical (corners seed the current box), and dragging one
    // corner freely warps the patch (perspective). Whole-layer float (no selection).
    let mut t = deform_square_canvas(80, 20, 20, 60, 60); // opaque block [20,60)²
    t.set_shape_grab_tol_px(10.0);
    t.set_deform_transform_on(true);
    let seeded = t.canvas_rgba.clone();
    t.set_deform_transform_mode(2); // Distort
    assert_eq!(
        *t.canvas_rgba, *seeded,
        "entering Distort is byte-identical"
    );
    // A corner of the content box sits at (20,20); drag it out to (8,8).
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert_ne!(
        *t.canvas_rgba, *seeded,
        "dragging a Distort corner warps the patch"
    );
}

/// Perf harness (RELEASE only — dev/opt-0 lies about perf): time one Transform gizmo move on a moderate
/// selection over a large canvas. The dirty-rect composite touches only the patch's source + destination
/// bbox, so an interactive drag should land well under one 60 Hz frame (16 ms). Ignored by default:
///   cargo test -p ph2d-tool-painter --release deform_transform_perf -- --ignored --nocapture
#[test]
#[ignore]
fn deform_transform_perf_move_is_under_frame_budget() {
    let size = 2048u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for px in src.as_chunks_mut::<4>().0.iter_mut() {
        px.copy_from_slice(&[200, 120, 60, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("liquify");
    t.set_shape_grab_tol_px(12.0);
    // A 512×512 selection near the centre — a typical "move this region" gesture.
    t.set_rect_selection(760, 760, 512, 512);
    t.set_deform_transform_on(true);
    // Warm up, then time a sequence of drags (grab the selection centre, wiggle it).
    let cx = 1016.0;
    let cy = 1016.0;
    t.on_canvas_pointer(cp([cx, cy], PointerPhase::Down));
    let iters = 60;
    let start = std::time::Instant::now();
    for i in 0..iters {
        let dx = ((i % 20) as f32) - 10.0;
        t.on_canvas_pointer(cp([cx + dx, cy + dx], PointerPhase::Move));
    }
    let per = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    t.on_canvas_pointer(cp([cx, cy], PointerPhase::Up));
    println!("deform Transform move: {per:.3} ms/frame (512² selection on {size}² canvas)");
    assert!(
        per < 16.0,
        "Transform move {per:.2} ms exceeds the 16 ms frame budget"
    );
}

/// Perf harness (RELEASE only): time one Warp mesh drag — the Catmull-Rom subdivision + fine-cell raster is
/// heavier than the affine composite, so confirm an interactive drag still fits a 60 Hz frame.
///   cargo test -p ph2d-tool-painter --release deform_warp_perf -- --ignored --nocapture
#[test]
#[ignore]
fn deform_warp_perf_drag_is_under_frame_budget() {
    let size = 2048u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for px in src.as_chunks_mut::<4>().0.iter_mut() {
        px.copy_from_slice(&[200, 120, 60, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("liquify");
    t.set_shape_grab_tol_px(16.0);
    t.set_rect_selection(760, 760, 512, 512);
    t.set_deform_transform_on(true);
    t.set_deform_transform_mode(3); // Warp
    // An interior control point of the 3×3 mesh over the selection [760,1272] sits at ≈(931,931).
    let (cx, cy) = (931.0, 931.0);
    t.on_canvas_pointer(cp([cx, cy], PointerPhase::Down));
    let iters = 60;
    let start = std::time::Instant::now();
    for i in 0..iters {
        let d = ((i % 20) as f32) - 10.0;
        t.on_canvas_pointer(cp([cx + d, cy + d], PointerPhase::Move));
    }
    let per = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    t.on_canvas_pointer(cp([cx, cy], PointerPhase::Up));
    println!("deform Warp drag: {per:.3} ms/frame (512² selection on {size}² canvas)");
    assert!(
        per < 16.0,
        "Warp drag {per:.2} ms exceeds the 16 ms frame budget"
    );
}

/// P1 measurement harness (RELEASE only): the ms/move TABLE for the Transform gizmo — whole-image vs a
/// 512² selection, each with and without the bridge retaining the preview Arc between moves (`hold`).
/// The retained Arc makes the tool's next `Arc::make_mut(canvas_rgba)` deep-copy the whole 16.8 MB
/// canvas EVERY move — invisible to a bench that doesn't hold the Arc (BUGS_painter.md Bug #3, the
/// bench-vs-live gap). The (whole+hold − whole) column split isolates the copy from the gather loop.
///   cargo test -p ph2d-tool-painter --release perf_transform_whole_image_table -- --ignored --nocapture
#[test]
#[ignore]
fn perf_transform_whole_image_table() {
    use std::time::Instant;
    let size = 2048u32;
    let run = |mode: u8, whole: bool, hold: bool, grab: [f32; 2]| -> f64 {
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px in src.as_chunks_mut::<4>().0.iter_mut() {
            px.copy_from_slice(&[200, 120, 60, 255]);
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.set_paint_tool_mode("liquify");
        t.set_shape_grab_tol_px(12.0);
        if !whole {
            t.set_rect_selection(760, 760, 512, 512);
        }
        t.set_deform_transform_on(true);
        t.set_deform_transform_mode(mode);
        let _ = t.take_preview_arc(); // drain the lift frame like the bridge would
        t.on_canvas_pointer(cp(grab, PointerPhase::Down));
        let moves = 20u32;
        let mut held = None;
        let t0 = Instant::now();
        for k in 0..moves {
            let d = ((k % 10) as f32) - 5.0;
            let _ = t.on_canvas_pointer(cp([grab[0] + d, grab[1] + d], PointerPhase::Move));
            if hold && let Some(p) = t.take_preview_arc() {
                held = Some(p); // retain across the next move (bridge behaviour)
            }
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(moves);
        let _ = held;
        t.on_canvas_pointer(cp(grab, PointerPhase::Up));
        ms
    };
    // Grab points: whole-image frame = the full canvas → centre-move (1024,1024), Distort corner (0,0),
    // Warp interior control point ≈(683,683). Selection [760,1272)² → centre (1016,1016), corner
    // (760,760), Warp interior ≈(931,931).
    let cases: [(&str, u8, [f32; 2], [f32; 2]); 4] = [
        (
            "Uniform (centre-move)",
            0,
            [1024.0, 1024.0],
            [1016.0, 1016.0],
        ),
        (
            "Free    (centre-move)",
            1,
            [1024.0, 1024.0],
            [1016.0, 1016.0],
        ),
        ("Distort (corner)", 2, [0.0, 0.0], [760.0, 760.0]),
        ("Warp    (interior pt)", 3, [682.7, 682.7], [930.7, 930.7]),
    ];
    eprintln!("Transform gizmo, {size}² canvas, ms/move (20 moves each, --release):");
    eprintln!(
        "  {:<24} {:>12} {:>12} {:>12} {:>12}",
        "sub-mode", "whole", "whole+hold", "sel512", "sel512+hold"
    );
    for (label, mode, gw, gs) in cases {
        let w = run(mode, true, false, gw);
        let wh = run(mode, true, true, gw);
        let s = run(mode, false, false, gs);
        let sh = run(mode, false, true, gs);
        eprintln!("  {label:<24} {w:>9.2} ms {wh:>9.2} ms {s:>9.2} ms {sh:>9.2} ms");
    }
}

#[test]
fn deform_transform_whole_image_corner_grabs_from_slightly_outside() {
    // P2 (Enio 2026-07-04): a whole-image transform puts the corner squares exactly ON the canvas
    // corner, so most of each square's clickable disc lies OUTSIDE the canvas. The tool grants a grab
    // margin to the shell (`deform_gizmo_grab_margin_px`) and must resolve a Down slightly outside the
    // corner to the DEFORM corner handle — scaling the patch, not silently no-oping.
    let mut t = white_canvas(64, 6.0);
    t.set_paint_tool_mode("liquify");
    t.set_shape_grab_tol_px(8.0);
    assert_eq!(
        t.deform_gizmo_grab_margin_px(),
        0.0,
        "no margin before the gizmo is live"
    );
    t.set_deform_transform_on(true);
    assert!(
        t.deform_gizmo_grab_margin_px() >= 8.0,
        "margin granted while the gizmo is live"
    );
    let before = t.canvas_rgba.clone();
    // The whole-image frame == the full canvas (opaque content bbox) → a corner handle at (0,0).
    // Down 4 px OUTSIDE the canvas, within the grab tol, then drag inward to shrink.
    t.on_canvas_pointer(cp([-4.0, -4.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Up));
    assert_ne!(
        *t.canvas_rgba, *before,
        "the outside-corner Down grabbed the deform corner and scaled the patch"
    );
    // Shrinking a whole-image opaque layer VACATES the border to transparent (a patch sample outside
    // the canvas is transparent, not an edge-clamp smear) while the interior stays opaque white.
    assert_eq!(px(&t, 64, 0, 0)[3], 0, "vacated corner is transparent");
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "interior still opaque white"
    );
    assert!(t.deform_gizmo().is_some(), "the transform stays live");
}

#[test]
fn deform_transform_warp_mesh_moves_a_control_point() {
    // Warp sub-mode: entering it is byte-identical (the mesh seeds on the box); dragging an interior
    // control point warps the patch locally. Whole-layer float (no selection).
    let mut t = deform_square_canvas(90, 15, 15, 75, 75); // opaque block [15,75)²
    t.set_shape_grab_tol_px(12.0);
    t.set_deform_transform_on(true);
    let seeded = t.canvas_rgba.clone();
    t.set_deform_transform_mode(3); // Warp
    assert_eq!(*t.canvas_rgba, *seeded, "entering Warp is byte-identical");
    // The 4×4 mesh over the content box [15,75) has an interior point near (35,35); drag it to (45,45).
    t.on_canvas_pointer(cp([35.0, 35.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([45.0, 45.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([45.0, 45.0], PointerPhase::Up));
    assert_ne!(
        *t.canvas_rgba, *seeded,
        "dragging a mesh point warps the patch"
    );
}

#[test]
fn deform_transform_undo_steps_the_gizmo_back() {
    // Undo while the Transform is LIVE steps the gizmo back one gesture (to the lift pose), and the
    // transform stays live — it does NOT jump to the pre-transform history (Enio 2026-07-04 bug).
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    t.set_deform_transform_on(true);
    let lift_pose = t.canvas_rgba.clone(); // lifted, gizmo at origin (identity == original)
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, *lift_pose, "the gizmo move changed pixels");
    assert!(t.undo_last(), "undo steps the gizmo back");
    assert_eq!(
        *t.canvas_rgba, *lift_pose,
        "the gizmo returns to its previous pose"
    );
    assert!(
        t.deform_gizmo().is_some(),
        "the Transform stays live after a local (gizmo) undo"
    );
    // One more undo un-lifts entirely: the gizmo disappears AND the Transform options close (temperament
    // back to none), so the artist must re-pick a mode to bring it back (Enio 2026-07-04).
    assert!(t.undo_last(), "the final undo un-lifts the transform");
    assert!(
        t.deform_gizmo().is_none(),
        "the gizmo is gone after the un-lift"
    );
    assert_eq!(
        t.paint.deform.temperament, 0,
        "un-lifting closes the Transform options (temperament → none)"
    );
}

#[test]
fn deform_transform_redo_recreates_the_gizmo() {
    // Redo mirrors the live-transform undo: after undoing a move (gizmo back) then un-lifting (gizmo gone),
    // redo re-lifts the gizmo (temperament → Transform) and steps the pose forward again (Enio 2026-07-04).
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    t.set_deform_transform_on(true);
    let lifted = t.canvas_rgba.clone();
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    let moved = t.canvas_rgba.clone();
    assert!(t.undo_last(), "undo the move");
    assert!(t.deform_gizmo().is_some(), "still live at the lift pose");
    assert!(t.undo_last(), "undo the lift (un-lift)");
    assert!(t.deform_gizmo().is_none(), "gizmo gone");
    assert_eq!(t.paint.deform.temperament, 0, "options closed");
    // Redo #1 → re-lift the gizmo.
    assert!(t.redo_last(), "redo re-lifts");
    assert!(t.deform_gizmo().is_some(), "redo recreates the gizmo");
    assert_eq!(
        t.paint.deform.temperament, 2,
        "redo restores the Transform temperament"
    );
    assert_eq!(*t.canvas_rgba, *lifted, "redo restores the lift pose");
    // Redo #2 → re-apply the move.
    assert!(t.redo_last(), "redo re-applies the move");
    assert_eq!(*t.canvas_rgba, *moved, "the gizmo move is back");
}

#[test]
fn deform_transform_relifts_when_repicked_after_leaving_the_panel() {
    // Leaving Deform bakes the transform; re-entering and picking Transform re-lifts a FRESH gizmo
    // (Enio 2026-07-04: the gizmo used to not reappear). That guarantee is what this gate protects.
    //
    // ⚠️ **The MECHANISM under it changed on 2026-08-08 and this gate was asserting the mechanism.** It
    // used to read "re-entering opens the temperament UNSELECTED", which was true because the only wire
    // into the mode was `"deform"` — a lobby. Now each half has its own chip and its own wire, so the
    // entry itself names a half: coming in by `"liquify"` lands in the brush half (no gizmo, asserted
    // below) and picking Transform in the panel lifts one. The rail's own door is gated next door, in
    // `warp/rail_tests::entering_transform_from_another_tool_lifts_a_fresh_gizmo`.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    t.set_deform_transform_on(true);
    assert!(
        t.deform_gizmo().is_some(),
        "gizmo shown when Transform is picked"
    );
    t.set_paint_tool_mode("brush"); // leave Deform (bakes)
    t.set_paint_tool_mode("liquify"); // re-enter
    assert!(
        t.deform_gizmo().is_none(),
        "re-entering by the Liquify wire must leave the brush half in hand — a gizmo here would mean \
         the bake left one floating"
    );
    assert!(t.route_deform_event(&PanelEvent::Click(
        core_ids::PAINTER_DEFORM_TEMPERAMENT_TRANSFORM
    )));
    assert!(
        t.deform_gizmo().is_some(),
        "re-picking Transform re-lifts the gizmo"
    );
}

#[test]
fn deform_transform_undo_rolls_back_the_whole_transform() {
    // The whole Transform commits as ONE undo entry when it ends (Procreate model). After ending it
    // (temperament → Reshape), undo restores the pre-transform pixels.
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    let before = t.canvas_rgba.clone();
    t.set_deform_transform_on(true);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, *before, "the transform changed pixels");
    t.set_deform_transform_on(false); // ends + commits the transform as one undo entry
    assert!(t.undo_last(), "undo the whole transform");
    assert_eq!(
        *t.canvas_rgba, *before,
        "undo restores the pre-transform pixels"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────────
// Deform (Liquify) — Wave 1 kernel + seam tests (DoD). The inverse-warp kernel writes the active raster;
// each stroke = one structural undo entry; Freeze reuses the Selection mask; the panel seam drives the
// real `PanelEvent` → observable deform state.
// ─────────────────────────────────────────────────────────────────────────────────────────────────────

/// A tool with a horizontal red ramp (`red = x`, so a horizontal displacement is detectable) and Deform
/// mode active at a generous radius.
fn deform_ramp(size: u32) -> PainterTool {
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i] = x as u8; // red ramps with x (0..size)
            src[i + 1] = 128;
            src[i + 2] = 128;
            src[i + 3] = 255;
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("liquify");
    t.set_deform_transform_on(false); // pick the Reshape temperament (the panel opens on NONE)
    t.set_deform_size_norm(0.25); // ~33px radius: spans the sampled pixels, corner stays outside the dab
    t
}

#[test]
fn deform_identity_stroke_is_byte_identical() {
    // Push with no motion ⇒ D = 0 everywhere ⇒ the inverse gather resolves each dst to itself. A press +
    // release at the same point must leave the canvas byte-for-byte unchanged (kernel parity).
    let mut t = deform_ramp(64);
    let before = (*t.canvas_rgba).clone();
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    assert_eq!(*t.canvas_rgba, before, "a zero-motion Push is identity");
}

#[test]
fn deform_push_moves_content_along_the_drag() {
    // Pushing +x pulls lower-x (darker-red) content under the dab centre, so the sampled red DROPS.
    let mut t = deform_ramp(64);
    let target = px(&t, 64, 40, 32); // red ≈ 40 before
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move)); // drag +8px in x
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    let after = px(&t, 64, 40, 32);
    assert!(
        after[0] < target[0],
        "Push +x pulls darker (lower-x) content in: red {} → {}",
        target[0],
        after[0]
    );
    // A far corner outside the dab is untouched.
    assert_eq!(px(&t, 64, 0, 0), [0, 128, 128, 255], "corner untouched");
}

#[test]
fn deform_is_confined_to_the_selection() {
    // A left-half selection: pushing a Reshape dab across the boundary moves only the SELECTED (left)
    // texels; the unselected (right) texels stay byte-identical. Deform acts only on the selected area
    // (whole sprite when nothing is selected) — the Freeze toggle is gone (Enio 2026-07-04).
    let mut t = deform_ramp(64);
    t.set_rect_selection(0, 0, 32, 64); // select the left half (x < 32)
    let left_before = px(&t, 64, 16, 32);
    let right_before = px(&t, 64, 48, 32);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    assert_ne!(
        px(&t, 64, 16, 32),
        left_before,
        "a selected texel is warped"
    );
    assert_eq!(
        px(&t, 64, 48, 32),
        right_before,
        "an unselected texel is left untouched (confined to the selection)"
    );
}

#[test]
fn deform_stroke_is_one_undo_entry_and_restores_byte_identical() {
    let mut t = deform_ramp(64);
    let before = (*t.canvas_rgba).clone();
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 34.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 34.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, before, "the deform stroke changed pixels");
    assert!(t.undo_last(), "one deform stroke is one undo entry");
    assert_eq!(
        *t.canvas_rgba, before,
        "undo restores the pre-stroke pixels byte-for-byte"
    );
}

#[test]
fn deform_reset_restores_the_session_pre_pixels() {
    // A multi-stroke session, then Reset returns to the pristine session pixels.
    let mut t = deform_ramp(64);
    let before = (*t.canvas_rgba).clone();
    for _ in 0..2 {
        t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([40.0, 36.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 36.0], PointerPhase::Up));
    }
    assert_ne!(*t.canvas_rgba, before, "two strokes deformed the canvas");
    t.deform_reset();
    assert_eq!(
        *t.canvas_rgba, before,
        "Reset restores the session pre-deform pixels"
    );
}

#[test]
fn deform_panel_seam_mode_slider_and_temperament_drive_the_state() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = deform_ramp(32);
    // Segmented mode pick: Click on the Twist option id → mode 1.
    assert!(t.route_deform_event(&PanelEvent::Click(core_ids::PAINTER_DEFORM_MODE_TWIST)));
    assert_eq!(t.paint.deform.mode, 1, "the Twist segment set mode = Twist");
    // Slider: SetValue on the Strength slider → clamped strength.
    assert!(t.route_deform_event(&PanelEvent::SetValue(
        core_ids::PAINTER_DEFORM_STRENGTH_SLIDER,
        0.9
    )));
    assert!(
        (t.paint.deform.strength - 0.9).abs() < 1e-6,
        "Strength slider drove the value"
    );
    // Temperament segments drive the 3-state temperament (2 = Transform, 1 = Reshape).
    assert!(t.route_deform_event(&PanelEvent::Click(
        core_ids::PAINTER_DEFORM_TEMPERAMENT_TRANSFORM
    )));
    assert_eq!(
        t.paint.deform.temperament, 2,
        "the Transform segment set temperament = Transform"
    );
    assert!(t.route_deform_event(&PanelEvent::Click(
        core_ids::PAINTER_DEFORM_TEMPERAMENT_RESHAPE
    )));
    assert_eq!(
        t.paint.deform.temperament, 1,
        "the Reshape segment set temperament = Reshape"
    );
}

#[test]
fn deform_reconstruct_un_warps_toward_the_original() {
    // Reconstruct must slide pixels BACK to their original positions (reduce the session displacement),
    // not cross-fade the original over the deformed. After enough Reconstruct scrubbing the canvas is far
    // closer to the pre-deform pixels than right after the Push.
    let mut t = deform_ramp(64);
    let before = (*t.canvas_rgba).clone();
    // Deform with a Push drag.
    t.on_canvas_pointer(cp([28.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    let l1 = |a: &[u8], b: &[u8]| -> u64 {
        a.iter()
            .zip(b)
            .map(|(x, y)| (*x as i64 - *y as i64).unsigned_abs())
            .sum()
    };
    let pushed = l1(&t.canvas_rgba, &before);
    assert!(pushed > 0, "the push deformed the canvas");
    // Reconstruct at full pressure, scrubbing the deformed band several times (same session → same disp).
    t.set_deform_mode(5); // Reconstruct
    t.set_deform_pressure(1.0);
    for _ in 0..16 {
        t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Up));
    }
    let reconstructed = l1(&t.canvas_rgba, &before);
    assert!(
        reconstructed * 4 < pushed,
        "Reconstruct un-warped most of the deform: pushed L1 {pushed}, after reconstruct {reconstructed}"
    );
}

#[test]
fn deform_undo_preserves_the_reconstruction_capability() {
    // Regression (Enio 2026-07-04): undoing a deform stroke must roll the displacement back in lock-step
    // with the pixels — NOT drop the session — so Reconstruct can still un-warp what remains.
    let mut t = deform_ramp(64);
    let before = (*t.canvas_rgba).clone();
    let l1 = |a: &[u8], b: &[u8]| -> u64 {
        a.iter()
            .zip(b)
            .map(|(x, y)| (*x as i64 - *y as i64).unsigned_abs())
            .sum()
    };
    // Stroke 1, then stroke 2 (same session accumulates the displacement).
    t.on_canvas_pointer(cp([26.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    let after1 = (*t.canvas_rgba).clone();
    t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([54.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([54.0, 32.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, after1, "stroke 2 deformed further");
    // Undo stroke 2 → pixels roll back to after-stroke-1, and the session (displacement) is preserved.
    assert!(t.undo_last(), "one undo step for stroke 2");
    assert_eq!(*t.canvas_rgba, after1, "undo restored the stroke-1 pixels");
    assert!(t.paint.warp.active, "the deform session survives the undo");
    assert!(
        !t.paint.warp.disp.is_empty(),
        "the displacement survives the undo"
    );
    // And Reconstruct still un-warps toward the pristine session original.
    let pushed = l1(&after1, &before);
    t.set_deform_mode(5); // Reconstruct
    t.set_deform_pressure(1.0);
    for _ in 0..16 {
        t.on_canvas_pointer(cp([22.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([46.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([46.0, 32.0], PointerPhase::Up));
    }
    let reconstructed = l1(&t.canvas_rgba, &before);
    assert!(
        reconstructed * 3 < pushed,
        "Reconstruct still works after undo: pre-undo-warp L1 {pushed}, after reconstruct {reconstructed}"
    );
}
