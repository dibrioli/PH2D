//! **A seleção.** Marquee (retângulo, elipse), laço, automática por flood, feather, os operadores
//! (add/remove), o offset paramétrico e o Apply/Apply&Keep, a conversão em curva editável e a edição
//! dos seus pontos, os gizmos, o copiar/colar, e o que a seleção faz ao que se pinta dentro dela.

use super::*;

/// Perf harness (RELEASE): recompose N boolean selection shapes while dragging ONE — the per-shape raster
/// cache should reuse the N−1 unchanged shapes (only the dragged one re-rasterizes).
///   cargo test -p ph2d-tool-painter --release perf_selection_boolean -- --ignored --nocapture
#[test]
#[ignore]
fn perf_selection_boolean_recompose_cache_vs_full() {
    use super::selection_shapes::{SelectionEntry, SelectionShape};
    let size = 2048u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    for i in 0..8u32 {
        let cx = 300.0 + i as f32 * 200.0;
        t.paint.selection_shapes.push(SelectionEntry {
            shape: SelectionShape::Ellipse {
                center: [cx, 1024.0],
                u: [1.0, 0.0],
                rx: 160.0,
                ry: 160.0,
            },
            op: if i == 0 { 0 } else { 1 },
        });
    }
    t.recompose_selection_mask(); // warm the per-shape cache
    let bump = |t: &mut PainterTool, k: u32| {
        if let SelectionShape::Ellipse { center, .. } = &mut t.paint.selection_shapes[3].shape {
            center[0] = 900.0 + (k % 10) as f32;
        }
    };
    let iters = 60u32;
    let t0 = std::time::Instant::now();
    for k in 0..iters {
        bump(&mut t, k);
        t.recompose_selection_mask();
    }
    let cached = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    let t1 = std::time::Instant::now();
    for k in 0..iters {
        t.paint.selection_raster_cache.clear(); // simulate the OLD behaviour (re-rasterize all 8)
        bump(&mut t, k);
        t.recompose_selection_mask();
    }
    let full = t1.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    println!(
        "selection recompose 8 boolean shapes, drag 1: cache {cached:.2} ms/move  vs  full {full:.2} ms/move"
    );
}

#[cfg(test)]
/// P4 harness: the per-FRAME cost of the selection machinery at 2048² with 8 boolean shapes — the
/// marching-ants overlay rebuild (animated `phase` ⇒ runs every frame, even idle) and one gizmo-drag
/// Move (recompose). Run:
///   cargo test -p ph2d-tool-painter --release perf_selection_overlay_frame -- --ignored --nocapture
#[test]
#[ignore = "perf measurement — run explicitly in --release"]
fn perf_selection_overlay_frame() {
    use super::selection_shapes::{SelectionEntry, SelectionShape};
    let size = 2048u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    for i in 0..8u32 {
        let cx = 300.0 + i as f32 * 200.0;
        t.paint.selection_shapes.push(SelectionEntry {
            shape: SelectionShape::Ellipse {
                center: [cx, 1024.0],
                u: [1.0, 0.0],
                rx: 160.0,
                ry: 160.0,
            },
            op: if i == 0 { 0 } else { 1 },
        });
    }
    t.recompose_selection_mask();
    t.paint.selection_active = true;
    let iters = 30u32;
    let t0 = std::time::Instant::now();
    for k in 0..iters {
        let _ = t.selection_overlay_rgba(k);
    }
    let overlay = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    println!("selection overlay rebuild (2048², 8 shapes): {overlay:.2} ms/frame");
}

/// Wave 1 DoD (ADR-0103): a committed selection restricts painting to its region, AND the selection edit
/// + the paint stroke live on the ONE interleaved undo queue (undo/redo round-trips both, in order).
#[test]
fn selection_restricts_paint_and_undoes_interleaved_on_one_queue() {
    let mut t = white_canvas(64, 8.0);
    // Select the LEFT half (x < 32). One structural undo entry.
    t.set_rect_selection(0, 0, 32, 64);
    assert!(t.selection_active(), "a selection is now live");
    // Paint one hard black dab straddling the selection border at x=32 (radius 8 → covers x 24..40).
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    // Inside the selection + under the dab → painted black.
    assert_eq!(
        px(&t, 64, 26, 32),
        [0, 0, 0, 255],
        "inside the selection is painted"
    );
    // Outside the selection + under the dab → CLIPPED back to white (the selection gate).
    assert_eq!(
        px(&t, 64, 38, 32),
        [255, 255, 255, 255],
        "the dab is clipped at the selection border"
    );
    // ONE queue: undo the stroke first (selection stays), then undo the selection (it clears).
    t.undo_last();
    assert_eq!(
        px(&t, 64, 26, 32),
        [255, 255, 255, 255],
        "undo removed the stroke"
    );
    assert!(
        t.selection_active(),
        "undoing only the stroke leaves the selection standing"
    );
    t.undo_last();
    assert!(
        !t.selection_active(),
        "undoing again rolled back the selection edit itself"
    );
    // Redo re-applies both, in order.
    t.redo_last();
    assert!(t.selection_active(), "redo re-created the selection");
    t.redo_last();
    assert_eq!(
        px(&t, 64, 26, 32),
        [0, 0, 0, 255],
        "redo re-applied the stroke inside the selection"
    );
}

/// Wave 2: the Rectangle marquee covers the dragged region (Down → Move → Up), and an out-of-rect texel
/// stays unselected.
#[test]
fn selection_rectangle_marquee_covers_the_dragged_region() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Up));
    assert!(t.selection_active(), "a rectangle selection is live");
    assert_eq!(t.selection_coverage_at(20, 25), 255, "inside the rectangle");
    assert_eq!(t.selection_coverage_at(50, 50), 0, "outside the rectangle");
}

/// Wave 2: the Ellipse marquee selects inside the ellipse but not the bbox corners.
#[test]
fn selection_ellipse_marquee_excludes_the_corners() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
    assert_eq!(
        t.selection_coverage_at(20, 20),
        255,
        "the ellipse centre is in"
    );
    assert_eq!(t.selection_coverage_at(1, 1), 0, "the bbox corner is out");
}

/// Wave 2: Add unions a second rectangle into the selection; the operator is the boolean seam.
#[test]
fn selection_add_operator_unions_two_rectangles() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2);
    // First rect (New) top-left.
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    // Second rect with Add, bottom-right — both regions end up selected, the gap stays out.
    t.set_selection_bool_op(1); // Add
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 56.0], PointerPhase::Up));
    assert_eq!(
        t.selection_coverage_at(8, 8),
        255,
        "first rect still selected"
    );
    assert_eq!(t.selection_coverage_at(48, 48), 255, "second rect added");
    assert_eq!(
        t.selection_coverage_at(30, 30),
        0,
        "the gap between is unselected"
    );
}

/// Bbox centre of a converted Freehand selection curve's anchors (test helper).
#[cfg(test)]
fn sel_freehand_center(shape: &super::selection_shapes::SelectionShape) -> [f32; 2] {
    let super::selection_shapes::SelectionShape::Freehand { model, .. } = shape else {
        panic!("expected a Freehand selection curve");
    };
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in &model.points {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}

/// **Merge Curves** (Enio 2026-07-05): two OVERLAPPING selection curves collapse to a SINGLE dense
/// multipoint curve tracing their composed union, and the union coverage is preserved.
#[test]
fn selection_merge_collapses_overlapping_curves_into_one_dense_curve() {
    use super::selection_shapes::SelectionShape;
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    // Base rect (New), then an OVERLAPPING second rect (Add) → their union is one blob.
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([36.0, 36.0], PointerPhase::Up));
    t.set_selection_bool_op(1); // Add
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 52.0], PointerPhase::Up));
    t.selection_convert_to_curve();
    assert!(
        t.paint.selection_shapes.len() >= 2,
        "convert kept one curve per shape, got {}",
        t.paint.selection_shapes.len()
    );
    t.selection_merge_curves();
    assert_eq!(
        t.paint.selection_shapes.len(),
        1,
        "overlapping curves merge into a SINGLE curve"
    );
    let SelectionShape::Freehand { model, .. } = &t.paint.selection_shapes[0].shape else {
        panic!("merged entry is a Freehand curve");
    };
    assert!(
        model.points.len() >= 6,
        "the merged curve is high-precision multipoint, got {} points",
        model.points.len()
    );
    assert_eq!(
        t.paint.selection_shapes[0].op, 0,
        "the sole curve is the base"
    );
    // Union coverage preserved: interior of each original rect stays selected, far outside does not.
    assert_eq!(
        t.selection_coverage_at(14, 14),
        255,
        "first rect interior kept"
    );
    assert_eq!(
        t.selection_coverage_at(46, 46),
        255,
        "second rect interior kept"
    );
    assert_eq!(
        t.selection_coverage_at(60, 4),
        0,
        "outside stays unselected"
    );
}

/// **Simplify Curve** reworked (Enio 2026-07-05): re-fits EVERY converted curve (works on the multi-curve
/// list, before OR after a merge) to far fewer anchors via the Free-Hand Schneider fit.
#[test]
fn selection_simplify_reduces_points_on_all_curves() {
    use super::selection_shapes::SelectionShape;
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([6.0, 6.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([26.0, 26.0], PointerPhase::Up));
    t.set_selection_bool_op(1); // Add (a SEPARATE second region — stays 2 curves)
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([58.0, 58.0], PointerPhase::Up));
    t.selection_convert_to_curve(); // dense multipoint curves
    let dense: usize = t
        .paint
        .selection_shapes
        .iter()
        .filter_map(|e| match &e.shape {
            SelectionShape::Freehand { model, .. } => Some(model.points.len()),
            _ => None,
        })
        .sum();
    assert!(
        dense >= 8,
        "convert produced dense curves, got {dense} points"
    );
    t.selection_simplify_curve();
    let sparse: usize = t
        .paint
        .selection_shapes
        .iter()
        .filter_map(|e| match &e.shape {
            SelectionShape::Freehand { model, .. } => Some(model.points.len()),
            _ => None,
        })
        .sum();
    assert!(
        sparse < dense,
        "simplify cut the anchor count across all curves: {dense} → {sparse}"
    );
    // Both regions still selected after the re-fit.
    assert_eq!(t.selection_coverage_at(14, 14), 255, "first region kept");
    assert_eq!(t.selection_coverage_at(48, 48), 255, "second region kept");
}

/// **Centre-square tap** on a selection gizmo cycles that shape's op Add↔Remove (Enio 2026-07-05), mirroring
/// the stroke gizmo. A tap = Down+Up with no drag; a drag past the slop stays a move (op unchanged).
#[test]
fn selection_centre_square_tap_cycles_add_remove() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    // Base rect (New) big, then a clearly-separated second rect (Add) whose centre is well clear of anchors.
    t.on_canvas_pointer(cp([4.0, 4.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Up));
    t.set_selection_bool_op(1); // Add
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([58.0, 58.0], PointerPhase::Up));
    t.selection_convert_to_curve();
    assert_eq!(
        t.paint.selection_shapes[1].op, 1,
        "second curve starts as Add"
    );
    let c = sel_freehand_center(&t.paint.selection_shapes[1].shape);
    // TAP (no move) on the centre square → Add → Remove.
    t.selection_gizmo_pointer(cp(c, PointerPhase::Down));
    t.selection_gizmo_pointer(cp(c, PointerPhase::Up));
    assert_eq!(
        t.paint.selection_shapes[1].op, 2,
        "a centre-square tap flipped Add → Remove"
    );
    // TAP again → Remove → Add (only the two states cycle).
    t.selection_gizmo_pointer(cp(c, PointerPhase::Down));
    t.selection_gizmo_pointer(cp(c, PointerPhase::Up));
    assert_eq!(
        t.paint.selection_shapes[1].op, 1,
        "a second tap flipped Remove → Add"
    );
    // A DRAG (past the slop) on the centre square is a MOVE, not a tap — op stays put.
    t.selection_gizmo_pointer(cp(c, PointerPhase::Down));
    t.selection_gizmo_pointer(cp([c[0] + 20.0, c[1]], PointerPhase::Move));
    t.selection_gizmo_pointer(cp([c[0] + 20.0, c[1]], PointerPhase::Up));
    assert_eq!(
        t.paint.selection_shapes[1].op, 1,
        "dragging the centre square moves the shape, it does NOT cycle the op"
    );
}

/// Wave 2: Automatic flood-selects the connected same-colour region up to the threshold, joining the undo
/// queue on pen-up.
#[test]
fn selection_automatic_floods_the_connected_region() {
    // Left half red, right half blue; Automatic from the red side selects only the red half.
    let (size, half) = (16u32, 8u32);
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let o = ((y * size + x) * 4) as usize;
            let c = if x < half {
                [200, 0, 0, 255]
            } else {
                [0, 0, 200, 255]
            };
            src[o..o + 4].copy_from_slice(&c);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(0); // Automatic
    t.set_selection_threshold(0.1); // tight — never bridges into blue
    t.on_canvas_pointer(cp([2.0, 2.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([2.0, 2.0], PointerPhase::Up));
    assert_eq!(t.selection_coverage_at(3, 3), 255, "red region selected");
    assert_eq!(
        t.selection_coverage_at(12, 8),
        0,
        "blue region not selected"
    );
    // Undo removes the whole flood-select as ONE queue entry.
    t.undo_last();
    assert!(
        !t.selection_active(),
        "undo cleared the Automatic selection"
    );
}

/// Wave 3 backend: Feather softens the selection border (derived from the crisp accumulator) while the
/// deep interior stays fully selected.
#[test]
fn selection_feather_softens_the_border_not_the_interior() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 64.0], PointerPhase::Up)); // left-half rect (x < 32)
    assert_eq!(
        t.selection_coverage_at(32, 32),
        0,
        "crisp: just outside the border is unselected"
    );
    t.set_selection_feather(0.3);
    let edge = t.selection_coverage_at(32, 32);
    assert!(
        edge > 0 && edge < 255,
        "feather softens the border to a partial value: got {edge}"
    );
    assert!(
        t.selection_coverage_at(5, 32) > 200,
        "the deep interior stays selected after feather"
    );
    // Feather is undoable via the crisp accumulator: dialing it back to 0 restores the crisp edge.
    t.set_selection_feather(0.0);
    assert_eq!(
        t.selection_coverage_at(32, 32),
        0,
        "feather 0 re-derives the crisp (hard) border"
    );
}

/// Wave 4: the on-canvas overlay marks the boundary with opaque marching ants, hatches the deselected
/// area semi-transparently, leaves the interior clear, and is absent with no selection.
#[test]
fn selection_overlay_ants_hatch_and_clear_interior() {
    let mut t = white_canvas(32, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Up)); // left half (x < 16) selected
    let (rgba, w, h) = t.selection_overlay_rgba(0).expect("overlay while active");
    assert_eq!((w, h), (32, 32));
    let a = |x: usize, y: usize| rgba[4 * (y * 32 + x) + 3];
    // Deep interior (selected, non-edge) is fully transparent.
    assert_eq!(a(4, 16), 0, "interior stays clear");
    // Marching ants: opaque texels along the vertical border (x = 15 is the inside edge).
    assert!(
        (0..32).any(|y| a(15, y) == 255),
        "opaque marching ants on the boundary"
    );
    // Hatch: some semi-transparent coverage in the deselected area (right of the border).
    assert!(
        (0..32).any(|y| (20..32).any(|x| {
            let av = a(x, y);
            av > 0 && av < 255
        })),
        "semi-transparent hatch over the deselected area"
    );
    // No selection → no overlay.
    t.clear_selection();
    assert!(
        t.selection_overlay_rgba(0).is_none(),
        "no overlay without a selection"
    );
}

/// Smoke#2 fix A: the marquee previews LIVE during the drag (mask updated on Move, not only pen-up).
#[test]
fn selection_marquee_previews_live_during_the_drag() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move)); // still dragging — NO pen-up yet
    assert!(t.selection_active(), "the selection previews mid-drag");
    assert_eq!(
        t.selection_coverage_at(25, 25),
        255,
        "inside the dragged rect is selected before pen-up"
    );
    assert_eq!(
        t.selection_coverage_at(55, 55),
        0,
        "outside the dragged rect is not selected mid-drag"
    );
}

/// Smoke#2 fix E: Fill (flood) respects the active selection — it fills only inside, clipping the rest
/// back to the pre-fill pixels.
#[test]
fn fill_is_clipped_to_the_active_selection() {
    let mut t = white_canvas(64, 4.0);
    t.set_rect_selection(0, 0, 32, 64); // select the left half (x < 32)
    t.set_paint_tool_mode("fill");
    t.paint.brush.color = [0.0, 0.0, 0.0]; // fill with black
    t.on_canvas_pointer(cp([10.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([10.0, 32.0], PointerPhase::Up));
    t.set_fill_threshold(1.0); // flood the whole (uniform white) canvas — then clip to the selection
    assert_eq!(
        px(&t, 64, 10, 32),
        [0, 0, 0, 255],
        "fill lands inside the selection"
    );
    assert_eq!(
        px(&t, 64, 50, 32),
        [255, 255, 255, 255],
        "fill is clipped outside the selection"
    );
}

/// ADR-0103 Am.2 v2: "Show Selection Gizmos" reveals EVERY editable shape's isolated gizmo at once and
/// leaves the stroke shape editors UNTOUCHED (the contamination bug is fixed).
#[test]
fn selection_gizmos_show_all_shapes_without_touching_the_stroke_editors() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rect (New)
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    t.set_selection_bool_op(1); // Add
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 56.0], PointerPhase::Up));
    assert!(
        t.selection_gizmos().is_empty(),
        "no gizmos until the checkbox is on"
    );
    t.toggle_selection_edit();
    assert!(t.selection_edit_mode(), "gizmos shown");
    assert_eq!(
        t.selection_gizmos().len(),
        2,
        "one gizmo per editable shape, simultaneously"
    );
    // Isolation: selection editing NEVER opens a stroke shape editor.
    assert!(
        t.curve_overlay().is_none()
            && t.ellipse_overlay().is_none()
            && t.polygon_overlay().is_none(),
        "selection gizmos are isolated from the stroke editors"
    );
    t.toggle_selection_edit();
    assert!(t.selection_gizmos().is_empty(), "gizmos hidden again");
}

/// Count fully-selected texels (coverage ≥ 128) across a `size×size` selection — a robust area metric.
fn selected_area(t: &PainterTool, size: u32) -> usize {
    let mut n = 0;
    for y in 0..size {
        for x in 0..size {
            if t.selection_coverage_at(x, y) >= 128 {
                n += 1;
            }
        }
    }
    n
}

/// Dragging an ellipse gizmo handle edits the selection live (through the isolated gizmo pointer path).
#[test]
fn selection_ellipse_gizmo_handle_drag_grows_the_selection() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up)); // centre (32,32), r=16
    t.toggle_selection_edit();
    let before = selected_area(&t, 64);
    // The unified gizmo's scale_handles[4] is the RIGHT edge square (centre + rx); drag it out to grow rx.
    let rh = t.selection_gizmos()[0].scale_handles[4];
    t.on_canvas_pointer(cp(rh, PointerPhase::Down));
    t.on_canvas_pointer(cp([rh[0] + 12.0, rh[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([rh[0] + 12.0, rh[1]], PointerPhase::Up));
    assert!(
        selected_area(&t, 64) > before,
        "dragging the scale handle enlarges the selection"
    );
}

/// Plain-mode Offset (before any Apply & Keep) grows the whole selection when swept outward and shrinks it
/// when swept inward — the mask analogue of the stroke's parallel-curve Offset.
#[test]
fn selection_offset_plain_mode_grows_and_shrinks() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(32, 32, 32, 32); // centred 32×32
    let base = selected_area(&t, 96);
    t.set_selection_offset(0.55); // +20px outward
    assert!(
        selected_area(&t, 96) > base,
        "sweeping the offset outward grows the selection"
    );
    t.set_selection_offset(0.47); // −12px inward
    assert!(
        selected_area(&t, 96) < base,
        "sweeping the offset inward shrinks the selection"
    );
    t.set_selection_offset(0.5); // centred → byte-identical to the un-offset selection
    assert_eq!(
        selected_area(&t, 96),
        base,
        "a centred offset restores the original area exactly"
    );
}

/// **Selection offset keeps CORNERS SHARP** (Enio 2026-07-05, stroke parity): the former SDF dilation
/// rounded a grown rectangle's corners into radius-`d` arcs (Minkowski with a disc); the corner-true
/// contour offset miters them. Grow a centred square by 16px: the true miter corner region — beyond the
/// SDF's rounding arc — must be selected; shrink stays an exact smaller square.
#[test]
fn selection_offset_grows_a_rectangle_with_sharp_miter_corners() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(32, 32, 32, 32); // square [32,64)²
    t.set_selection_offset(0.54); // +16px outward
    // The grown square's sharp corner sits at (16,16). The SDF version excluded the corner-diagonal zone
    // (distance to the old corner = √2·13 ≈ 18.4 > 16 at pixel (19,19)); the miter version includes it.
    assert_eq!(
        t.selection_coverage_at(19, 19),
        255,
        "the grown corner is a sharp miter, not a rounded arc"
    );
    assert_eq!(
        t.selection_coverage_at(48, 17),
        255,
        "the grown top edge is included (sanity)"
    );
    assert_eq!(
        t.selection_coverage_at(13, 13),
        0,
        "beyond the miter corner stays unselected"
    );
    // Shrink: corners stay square (the miter of the inward offset).
    t.set_selection_offset(0.47); // −12px inward → square [44,52)²
    assert_eq!(
        t.selection_coverage_at(45, 45),
        255,
        "the shrunk corner is sharp (inside the smaller square)"
    );
    assert_eq!(
        t.selection_coverage_at(42, 42),
        0,
        "outside the shrunk square is unselected"
    );
}

/// **Selection offset is STROKE-EXACT on parametric shapes** (Enio 2026-07-05 "não ficou preciso como o
/// do stroke"): a marquee rectangle offsets its EXACT geometry (no mask trace round-trip), so every edge
/// of the grown/shrunk rect lands within a pixel of the analytic position; an ellipse marquee stays a
/// perfect ellipse at `r + d`.
#[test]
fn selection_offset_is_parametric_exact_for_rect_and_ellipse() {
    // Rectangle [32,64)² grown +10 → edges at 22/74 on every side, exact.
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(32, 32, 32, 32);
    t.set_selection_offset(0.525); // +10px
    for (inside, outside) in [((48u32, 23u32), (48u32, 21u32)), ((23, 48), (21, 48))] {
        assert_eq!(
            t.selection_coverage_at(inside.0, inside.1),
            255,
            "grown edge includes {inside:?} (exact analytic boundary)"
        );
        assert_eq!(
            t.selection_coverage_at(outside.0, outside.1),
            0,
            "grown edge excludes {outside:?}"
        );
    }
    // Ellipse marquee r=24 grown +8 → a PERFECT circle r≈32: on-axis inside at 31px, outside at 33px,
    // and the diagonal matches too (an SDF/trace wobble would break one of them).
    let mut t2 = white_canvas(96, 4.0);
    t2.set_paint_tool_mode("selection");
    t2.set_selection_mode(3); // Ellipse marquee (corner-dragged bounding box)
    t2.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
    t2.on_canvas_pointer(cp([72.0, 72.0], PointerPhase::Move));
    t2.on_canvas_pointer(cp([72.0, 72.0], PointerPhase::Up)); // centre (48,48), r = 24
    t2.set_selection_offset(0.52); // +8px
    assert_eq!(
        t2.selection_coverage_at(48 + 30, 48),
        255,
        "on-axis inside r+8"
    );
    assert_eq!(
        t2.selection_coverage_at(48 + 33, 48),
        0,
        "on-axis outside r+8"
    );
    // Diagonal at 45°: r·√½ ≈ 22.6 from centre each axis → inside at 22, outside at 24.
    assert_eq!(
        t2.selection_coverage_at(48 + 22, 48 + 22),
        255,
        "diagonal inside"
    );
    assert_eq!(
        t2.selection_coverage_at(48 + 24, 48 + 24),
        0,
        "diagonal outside"
    );
}

/// **Selection offset shrinks HOLES when growing** (the per-contour offset must see hole boundaries — the
/// SDF did implicitly): a donut (rect minus inner rect) grown by 8px expands its outer boundary AND closes
/// in on the hole, with the hole's corners staying sharp.
#[test]
fn selection_offset_grows_a_donut_shrinking_its_hole() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(16, 16, 64, 64); // outer [16,80)²
    t.set_selection_bool_op(2); // Remove
    t.on_canvas_pointer(cp([36.0, 36.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Up)); // hole [36,60)²
    assert_eq!(t.selection_coverage_at(48, 48), 0, "hole centre unselected");
    assert_eq!(t.selection_coverage_at(20, 20), 255, "ring selected");
    t.set_selection_offset(0.52); // +8px outward
    assert_eq!(
        t.selection_coverage_at(10, 48),
        255,
        "the outer boundary grew outward"
    );
    assert_eq!(
        t.selection_coverage_at(40, 40),
        255,
        "the hole SHRANK — its old interior edge is now selected"
    );
    assert_eq!(
        t.selection_coverage_at(48, 48),
        0,
        "the hole centre survives (not swallowed)"
    );
}

/// Apply & Keep switches the Offset into ring mode: the first swept band is PROTECTED (adds no selected
/// area), and after freezing it the next band is PAINT (a new concentric selected ring appears) — the
/// intercalated protected / paint rings the spec describes (ADR-0103 Am.3).
#[test]
fn selection_offset_apply_keep_alternates_protected_and_paint_rings() {
    let mut t = white_canvas(160, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(60, 60, 40, 40); // centred 40×40, wide margins
    let base = selected_area(&t, 160);
    // AK1 with the slider centred bakes the (unchanged) selection as the base + enters ring mode.
    t.selection_offset_apply_keep();
    assert_eq!(
        selected_area(&t, 160),
        base,
        "Apply & Keep with a centred slider keeps the same area"
    );
    // Sweep outward → the first ring band is PROTECTED: it adds nothing to the selected area.
    t.set_selection_offset(0.55); // +20px
    assert_eq!(
        selected_area(&t, 160),
        base,
        "the first outward band is protected — no new selected pixels"
    );
    // Freeze it, then sweep outward again → the next band is PAINT: a concentric selected ring appears.
    t.selection_offset_apply_keep();
    // With the slider re-centred after Apply & Keep, the frozen ring line must STAY drawn (a contour level
    // persists) so the selection line never vanishes in the transition area (Enio 2026-07-03 smoke).
    assert!(
        !t.selection_offset_contour_levels().is_empty(),
        "the frozen offset line stays visible after Apply & Keep re-centres the slider"
    );
    t.set_selection_offset(0.55); // +20px past the frozen boundary
    assert!(
        selected_area(&t, 160) > base,
        "the next band is paint — a new concentric selected ring is added"
    );
}

/// The offset is "engaged" once the slider is touched (so Enter can route to Apply), and Apply bakes the
/// result + disengages while keeping the selection live (the shell's Enter = Apply binding relies on this).
#[test]
fn selection_offset_apply_bakes_and_disengages() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(32, 32, 32, 32);
    assert!(
        !t.selection_offset_engaged(),
        "no offset engaged before touch"
    );
    t.set_selection_offset(0.6); // grow
    assert!(
        t.selection_offset_engaged(),
        "touching the slider engages the offset (Enter → Apply now applies)"
    );
    let grown = selected_area(&t, 96);
    t.selection_offset_apply(); // the Enter = Apply verb
    assert!(!t.selection_offset_engaged(), "Apply leaves offset mode");
    assert!(t.selection_active(), "the baked selection stays live");
    assert_eq!(
        selected_area(&t, 96),
        grown,
        "Apply bakes the grown selection (area preserved)"
    );
}

/// A Rect selection carries the Polygon gizmo (the sides DIAMOND is present; ellipse/freehand have none)
/// and fills a rectangle (its CORNERS are selected — an ellipse's would not be).
#[test]
fn selection_rect_uses_the_polygon_gizmo() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 56.0], PointerPhase::Up));
    assert_eq!(
        t.selection_coverage_at(10, 10),
        255,
        "the rect corner is selected"
    );
    t.toggle_selection_edit();
    let g = &t.selection_gizmos()[0];
    assert_eq!(
        g.scale_handles.len(),
        8,
        "the sprite gizmo has 8 scale squares"
    );
    assert!(
        g.diamond.is_some(),
        "the polygon gizmo's SIDES handle is a distinct diamond"
    );
}

/// Convert-to-Curve → one editable Freehand; Simplify keeps a valid curve; neither touches the stroke
/// editors and both preserve the selected region.
#[test]
fn selection_convert_then_simplify_curve() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t.selection_convert_to_curve();
    assert!(
        t.selection_edit_mode(),
        "still in gizmo mode on the new curve"
    );
    assert!(
        t.curve_overlay().is_none(),
        "convert never opens the stroke curve editor"
    );
    assert_eq!(
        t.selection_coverage_at(48, 48),
        255,
        "region survives convert"
    );
    assert_eq!(t.selection_gizmos().len(), 1, "one merged curve gizmo");
    let ellipse_diamond = t.selection_gizmos()[0].diamond;
    assert!(
        ellipse_diamond.is_none(),
        "a freehand curve has no sides diamond"
    );
    t.selection_simplify_curve();
    assert_eq!(
        t.selection_coverage_at(48, 48),
        255,
        "region survives simplify"
    );
    assert_eq!(
        t.selection_gizmos().len(),
        1,
        "simplify keeps one valid curve gizmo"
    );
}

/// After Convert to Curve the selection shows an editable POINT gizmo (anchors + in/out Bézier handles,
/// like the stroke Curve editor), NOT the transform box — and dragging an anchor edits the selection curve
/// (Enio 2026-07-03 regression fix). A RAW lasso Freehand keeps the transform box (no `edit_curve`).
#[test]
fn converted_selection_curve_is_point_editable() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit();
    // Before Convert: the ellipse shows the transform BOX (no point editor).
    assert!(
        t.selection_gizmos()[0].edit_curve.is_none(),
        "an un-converted shape uses the transform box"
    );
    t.selection_convert_to_curve();
    let giz = t.selection_gizmos();
    let curve = giz[0]
        .edit_curve
        .as_ref()
        .expect("the converted curve exposes an editable point gizmo");
    assert!(curve.anchors.len() >= 3, "anchors are visible for editing");
    assert!(
        curve.selected.is_none(),
        "convert leaves nothing selected (no tangents drawn yet), like the stroke curve"
    );
    // Grab an anchor and drag it — the selection curve follows (the anchor moves with the pointer).
    let anchor = curve.anchors[0];
    t.on_canvas_pointer(cp(anchor, PointerPhase::Down));
    // Pressing an anchor SELECTS it (its tangent handles now show — the shared-model behaviour).
    assert_eq!(
        t.selection_gizmos()[0]
            .edit_curve
            .as_ref()
            .unwrap()
            .selected,
        Some(0),
        "pressing an anchor selects it"
    );
    t.on_canvas_pointer(cp([anchor[0] + 12.0, anchor[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([anchor[0] + 12.0, anchor[1]], PointerPhase::Up));
    let moved = t.selection_gizmos()[0].edit_curve.as_ref().unwrap().anchors[0];
    assert!(
        (moved[0] - anchor[0]).abs() > 6.0,
        "dragging the anchor edited the curve point ({anchor:?} → {moved:?})"
    );
}

/// Convert to Curve FITS a dense lasso outline to a SPARSE handful of anchors (the "muitos pontos" fix,
/// Enio 2026-07-03) — the exact same Schneider fit a stroke Free Hand uses, via the shared `CurveModel`.
#[test]
fn selection_lasso_convert_fits_to_sparse_anchors() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(1); // Freehand lasso
    t.set_selection_stabilizer(0.0);
    // A DENSE square lasso (~60 captured points along the perimeter) — no transcendentals (HR-5-clean test).
    let (lo, hi, step) = (18.0f32, 78.0f32, 4.0f32);
    let mut gesture: Vec<[f32; 2]> = Vec::new();
    let mut x = lo;
    while x < hi {
        gesture.push([x, lo]);
        x += step;
    }
    let mut y = lo;
    while y < hi {
        gesture.push([hi, y]);
        y += step;
    }
    let mut x = hi;
    while x > lo {
        gesture.push([x, hi]);
        x -= step;
    }
    let mut y = hi;
    while y > lo {
        gesture.push([lo, y]);
        y -= step;
    }
    assert!(
        gesture.len() > 40,
        "the raw lasso is dense: {}",
        gesture.len()
    );
    t.on_canvas_pointer(cp(gesture[0], PointerPhase::Down));
    for p in &gesture[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(gesture[0], PointerPhase::Up));
    t.toggle_selection_edit();
    assert!(
        t.selection_gizmos()[0].edit_curve.is_none(),
        "a RAW lasso shows the transform box, not the point editor"
    );
    t.selection_convert_to_curve();
    let giz = t.selection_gizmos();
    let curve = giz[0]
        .edit_curve
        .as_ref()
        .expect("converting exposes the point editor");
    assert!(curve.anchors.len() >= 3, "at least a few anchors");
    assert!(
        curve.anchors.len() < 20,
        "Convert FITS to SPARSE anchors ({} from {} raw lasso points), not a verbatim copy",
        curve.anchors.len(),
        gesture.len()
    );
}

/// The selection curve's right-click **handle-kind menu** and **Delete** are wired through the same tool
/// verbs the shell drives (`set_selection_curve_handle_kind` / `selection_curve_delete_selected_point`) —
/// the seam test for the new capabilities (DIRETIVA §3: drive the real verb, assert the observable effect).
#[test]
fn selection_curve_handle_kind_and_delete_are_wired() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t.selection_convert_to_curve();
    let anchor = t.selection_gizmos()[0].edit_curve.as_ref().unwrap().anchors[0];
    let n = t.selection_gizmos()[0]
        .edit_curve
        .as_ref()
        .unwrap()
        .anchors
        .len();
    // Press the anchor to SELECT it (release without moving = pure select), then set its kind via the menu verb.
    t.on_canvas_pointer(cp(anchor, PointerPhase::Down));
    t.on_canvas_pointer(cp(anchor, PointerPhase::Up));
    assert!(
        t.set_selection_curve_handle_kind(2), // 2 = Vector
        "the handle-kind menu verb applies to the selected anchor"
    );
    assert_eq!(
        t.selection_gizmos()[0]
            .edit_curve
            .as_ref()
            .unwrap()
            .selected_kind,
        Some(2),
        "the chosen kind stuck (observable in the overlay, like the stroke curve)"
    );
    // Delete the selected anchor — one fewer control point.
    assert!(
        t.selection_curve_delete_selected_point(),
        "Delete removes the selected selection-curve anchor"
    );
    assert_eq!(
        t.selection_gizmos()[0]
            .edit_curve
            .as_ref()
            .unwrap()
            .anchors
            .len(),
        n - 1,
        "the anchor count dropped by one"
    );
}

/// The converted curve's transform box is inflated beyond the anchors (Enio 2026-07-05): a RECTANGLE
/// converts to anchors exactly at its corners, and the tight box buried them under the gizmo squares.
/// Every box scale-handle must sit clear of every curve anchor by at least the grab tolerance.
#[test]
fn converted_selection_curve_box_clears_the_anchors() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle — the reported case
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([76.0, 76.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t.selection_convert_to_curve();
    let tol = t.paint.shape_grab_tol_px;
    let g = &t.selection_gizmos()[0];
    let anchors = &g.edit_curve.as_ref().expect("converted curve").anchors;
    assert!(
        anchors.len() >= 8,
        "dense convert: the rectangle has many anchors ({})",
        anchors.len()
    );
    for h in &g.scale_handles {
        for a in anchors {
            let d = ((h[0] - a[0]).powi(2) + (h[1] - a[1]).powi(2)).sqrt();
            assert!(
                d >= tol,
                "box handle {h:?} sits clear of anchor {a:?} (d={d}, tol={tol})"
            );
        }
    }
}

/// Clicking on (or near) the converted selection curve INSERTS a new anchor — the stroke Curve editor's
/// click-to-add, delivered through `on_canvas_pointer` (the real gesture).
#[test]
fn selection_curve_click_inserts_an_anchor() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t.selection_convert_to_curve();
    // The dense convert packs anchors ~16 px apart — shrink the grab tol so a click BETWEEN two
    // adjacent anchors is an insert, not a grab, and aim at the arc midpoint (ON the curve).
    t.set_shape_grab_tol_px(3.0);
    let (n, click) = {
        let g = &t.selection_gizmos()[0];
        let anchors = &g.edit_curve.as_ref().unwrap().anchors;
        let (a, b) = (anchors[0], anchors[1]);
        let c = [48.0f32, 48.0f32]; // the ellipse centre
        let mid = [(a[0] + b[0]) * 0.5 - c[0], (a[1] + b[1]) * 0.5 - c[1]];
        let ra = ((a[0] - c[0]).powi(2) + (a[1] - c[1]).powi(2)).sqrt();
        let ml = (mid[0] * mid[0] + mid[1] * mid[1]).sqrt().max(1e-3);
        let click = [c[0] + mid[0] / ml * ra, c[1] + mid[1] / ml * ra];
        (anchors.len(), click)
    };
    // A press off any anchor/tangent inserts a new anchor at the nearest point ON the curve (shape-preserving).
    t.on_canvas_pointer(cp(click, PointerPhase::Down));
    t.on_canvas_pointer(cp(click, PointerPhase::Up));
    assert_eq!(
        t.selection_gizmos()[0]
            .edit_curve
            .as_ref()
            .unwrap()
            .anchors
            .len(),
        n + 1,
        "clicking the curve inserted an anchor"
    );
}

/// A converted selection curve carries the GLOBAL transform gizmo (move / rotate / scale) IN ADDITION to
/// the per-anchor point editor — dragging a box scale handle transforms the whole curve (Enio 2026-07-04).
#[test]
fn converted_curve_carries_the_transform_gizmo() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t.selection_convert_to_curve();
    let (corner, before) = {
        let giz = t.selection_gizmos();
        let g = &giz[0];
        assert!(g.edit_curve.is_some(), "the point editor is present");
        let c0 = g.box_corners[0];
        let c2 = g.box_corners[2];
        assert!(
            (c0[0] - c2[0]).abs() > 1.0 && (c0[1] - c2[1]).abs() > 1.0,
            "the transform box has real extent on a converted curve"
        );
        (
            g.scale_handles[0], // a corner scale handle (away from the on-curve anchors)
            g.edit_curve.as_ref().unwrap().anchors.clone(),
        )
    };
    // Down on the corner grabs the SCALE handle (not a point), and dragging it transforms the whole curve.
    t.on_canvas_pointer(cp(corner, PointerPhase::Down));
    t.on_canvas_pointer(cp([corner[0] - 18.0, corner[1] - 18.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([corner[0] - 18.0, corner[1] - 18.0], PointerPhase::Up));
    let after = t.selection_gizmos()[0]
        .edit_curve
        .as_ref()
        .unwrap()
        .anchors
        .clone();
    assert!(
        before
            .iter()
            .zip(&after)
            .any(|(a, b)| (a[0] - b[0]).abs() > 1.0 || (a[1] - b[1]).abs() > 1.0),
        "dragging a box handle transformed the whole converted curve"
    );
}

/// Offset **Apply & Keep** auto-unchecks Edit Gizmos (#1); re-checking it materialises the frozen ring
/// boundaries into editable curves (#2) — after one Apply & Keep that is the single base boundary
/// (Enio 2026-07-04).
#[test]
fn edit_gizmos_after_offset_apply_keep_opens_an_editable_curve() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit(); // enter Edit Gizmos — the ellipse gizmo shows
    // Grow the selection, then Apply & Keep → the offset bakes the base + auto-unchecks Edit Gizmos.
    t.set_selection_offset(0.65);
    t.selection_offset_apply_keep();
    assert!(t.selection_active(), "the baked selection is still live");
    assert!(
        !t.selection_gizmos_visible(),
        "Apply & Keep auto-unchecks Edit Gizmos (#1)"
    );
    // Re-check Edit Gizmos → materialise the ring boundary into an editable curve.
    t.toggle_selection_edit(); // on
    let giz = t.selection_gizmos();
    assert_eq!(
        giz.len(),
        1,
        "one ring boundary (the base) after one Apply & Keep"
    );
    assert!(
        giz[0].edit_curve.is_some(),
        "Edit Gizmos opens an editable curve on the baked-offset selection (not nothing)"
    );
}

/// Two Apply & Keep sweeps freeze TWO nested ring boundaries; re-checking Edit Gizmos materialises BOTH as
/// editable curves (not just the last), and they persist (#2, Enio 2026-07-04). The band-parity fill keeps
/// the intercalated selection: the interior stays selected, the protected band deselected.
#[test]
fn offset_apply_keep_rings_materialise_all_as_editable_curves() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([72.0, 72.0], PointerPhase::Up));
    t.toggle_selection_edit();
    // AK#1 bakes the base; AK#2 freezes an outward ring (a protected band between them).
    t.set_selection_offset(0.6);
    t.selection_offset_apply_keep();
    t.set_selection_offset(0.65);
    t.selection_offset_apply_keep();
    // Re-check Edit Gizmos → BOTH boundaries materialise as editable curves.
    t.toggle_selection_edit();
    let giz = t.selection_gizmos();
    assert_eq!(
        giz.len(),
        2,
        "both ring boundaries appear (not just the last)"
    );
    assert!(
        giz.iter().all(|g| g.edit_curve.is_some()),
        "every ring boundary is an editable curve"
    );
    // Interior is still selected (band 0 = paint); the region just outside the base but inside the ring is
    // the protected band (deselected).
    assert_eq!(
        t.selection_coverage_at(48, 48),
        255,
        "interior stays selected"
    );
}

/// A selection-curve point drag is on the GLOBAL painter undo/redo timeline: undo restores the parametric
/// anchor (not just the raster mask, which the next recompose would regenerate), redo re-applies it
/// (Enio 2026-07-03 — `selection_shapes` now rides in the `ModelSnapshot`).
#[test]
fn selection_curve_point_edit_is_undoable_and_redoable() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t.selection_convert_to_curve();
    let anchor = t.selection_gizmos()[0].edit_curve.as_ref().unwrap().anchors[0];
    // Drag anchor 0 by +12 px in x.
    t.on_canvas_pointer(cp(anchor, PointerPhase::Down));
    t.on_canvas_pointer(cp([anchor[0] + 12.0, anchor[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([anchor[0] + 12.0, anchor[1]], PointerPhase::Up));
    let moved = t.selection_gizmos()[0].edit_curve.as_ref().unwrap().anchors[0];
    assert!(
        (moved[0] - anchor[0]).abs() > 6.0,
        "the drag moved the anchor"
    );
    // UNDO → the anchor returns to its original position (parametric shape restored).
    assert!(t.undo_last(), "the point drag is one undo step");
    let undone = t.selection_gizmos()[0].edit_curve.as_ref().unwrap().anchors[0];
    assert!(
        (undone[0] - anchor[0]).abs() < 1.0,
        "undo restored the anchor to {anchor:?}, got {undone:?}"
    );
    // REDO → the anchor moves again.
    assert!(t.redo_last(), "redo re-applies the point edit");
    let redone = t.selection_gizmos()[0].edit_curve.as_ref().unwrap().anchors[0];
    assert!(
        (redone[0] - anchor[0]).abs() > 6.0,
        "redo re-moved the anchor ({anchor:?} → {redone:?})"
    );
}

/// A C&F ColorDrop over ONE of several disjoint selection areas fills ONLY that area (the region the colour
/// was dropped on), not every selected region (Enio 2026-07-04).
#[test]
fn colordrop_fills_only_the_dropped_selection_region() {
    let mut t = white_canvas(48, 4.0); // white canvas
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse marquee
    // Region A (left) — a New selection.
    t.set_selection_bool_op(0);
    t.on_canvas_pointer(cp([4.0, 18.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 30.0], PointerPhase::Up));
    // Region B (right, disjoint) — added.
    t.set_selection_bool_op(1);
    t.on_canvas_pointer(cp([32.0, 18.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 30.0], PointerPhase::Up));
    assert!(t.selection_active(), "two disjoint regions are selected");
    // Drop RED onto region B (right).
    t.set_brush_color_srgb8([255, 0, 0]);
    t.set_paint_tool_mode("fill");
    t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 48, 38, 24),
        [255, 0, 0, 255],
        "the dropped region (B) is filled"
    );
    assert_eq!(
        px(&t, 48, 10, 24),
        [255, 255, 255, 255],
        "the OTHER selected region (A) is untouched"
    );
}

/// ADR-0103 Wave 5: **Color Fill** paints the brush colour only inside the selection.
#[test]
fn selection_color_fill_paints_only_inside() {
    let mut t = white_canvas(32, 4.0); // white canvas, black brush
    t.set_rect_selection(0, 0, 16, 32); // left half
    t.selection_color_fill();
    assert_eq!(
        px(&t, 32, 4, 16),
        [0, 0, 0, 255],
        "inside the selection is filled black"
    );
    assert_eq!(
        px(&t, 32, 24, 16),
        [255, 255, 255, 255],
        "outside the selection is untouched"
    );
}

/// ADR-0103 Wave 5: **Copy** then **Paste** traz os pixels capturados de volta ao lugar de origem.
///
/// ⚠️ **Desde 2026-08-07 o Paste FLUTUA** (a peça transformável, `super::paste_patch`): os pixels
/// aparecem na hora — que é o que este gate mede —, mas só viram tinta no Enter. As duas metades
/// estão aqui de propósito: sem a asserção de que a peça segue VIVA, este gate passaria igual se
/// alguém revertesse o Paste para o composite imediato, e a mudança de comportamento sumiria.
#[test]
fn selection_copy_then_paste_reapplies_the_pixels() {
    let mut t = white_canvas(32, 4.0);
    t.set_rect_selection(0, 0, 16, 32);
    t.selection_color_fill(); // left half → black
    assert_eq!(px(&t, 32, 4, 16), [0, 0, 0, 255]);
    t.selection_copy(); // capture the black left half
    // Wipe the left half back to white by filling with a white brush.
    t.paint.brush.color = [1.0, 1.0, 1.0];
    t.selection_color_fill();
    assert_eq!(
        px(&t, 32, 4, 16),
        [255, 255, 255, 255],
        "left half wiped to white"
    );
    t.selection_paste(); // a peca flutuante aparece sobre a regiao de origem
    assert!(
        t.paste_patch_live(),
        "o Paste ARMA uma peca transformavel; ela so vira tinta no Enter"
    );
    assert_eq!(
        px(&t, 32, 4, 16),
        [0, 0, 0, 255],
        "paste restored the copied region"
    );
    assert_eq!(
        px(&t, 32, 24, 16),
        [255, 255, 255, 255],
        "outside the paste rect is untouched"
    );
}

/// ADR-0103 Wave 5: **Select layer contents** sets the mask from the layer's opaque texels.
#[test]
fn selection_from_layer_contents_selects_opaque_texels() {
    let size = 16u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            if x < 8 {
                let o = ((y * size + x) * 4) as usize;
                src[o..o + 4].copy_from_slice(&[10, 20, 30, 255]); // opaque left half
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("selection");
    t.selection_from_layer_contents();
    assert_eq!(t.selection_coverage_at(3, 3), 255, "opaque texel selected");
    assert_eq!(
        t.selection_coverage_at(12, 3),
        0,
        "transparent texel not selected"
    );
}

/// Leaving the Select tool auto-hides the selection gizmos (Enio 2026-07-03).
#[test]
fn switching_away_from_select_hides_the_gizmos() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(8, 8, 32, 32);
    t.toggle_selection_edit();
    assert!(t.selection_edit_mode(), "gizmos shown while on Select");
    t.set_paint_tool_mode("brush");
    assert!(
        !t.selection_edit_mode(),
        "switching to another tool auto-unchecked Show Selection Gizmos"
    );
}

/// The centroid (mean position) of the selected texels — a shape-agnostic way to test a whole-shape move.
fn selection_centroid(t: &PainterTool, size: u32) -> (f32, f32) {
    let (mut sx, mut sy, mut n) = (0.0f32, 0.0f32, 0.0f32);
    for y in 0..size {
        for x in 0..size {
            if t.selection_coverage_at(x, y) >= 128 {
                sx += x as f32;
                sy += y as f32;
                n += 1.0;
            }
        }
    }
    if n == 0.0 {
        (0.0, 0.0)
    } else {
        (sx / n, sy / n)
    }
}

/// A **Freehand** selection edits via a whole-shape TRANSFORM gizmo (move), NOT editable points: dragging
/// the centre handle translates the whole selection (the centroid shifts by the drag delta).
#[test]
fn selection_freehand_transform_move_shifts_the_selection() {
    let mut t = white_canvas(128, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(1); // Freehand
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    for p in [[40.0, 20.0], [45.0, 35.0], [30.0, 45.0], [18.0, 35.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert!(t.selection_active(), "a freehand selection exists");
    let before = selection_centroid(&t, 128);
    t.toggle_selection_edit();
    // The unified gizmo's centre square is the move handle at the bbox centre.
    let center = t.selection_gizmos()[0].center;
    t.on_canvas_pointer(cp(center, PointerPhase::Down));
    t.on_canvas_pointer(cp([center[0] + 30.0, center[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([center[0] + 30.0, center[1]], PointerPhase::Up));
    let after = selection_centroid(&t, 128);
    assert!(
        (after.0 - before.0 - 30.0).abs() < 3.0,
        "moving the centre handle +30px shifts the whole selection ~+30 in x ({before:?} -> {after:?})"
    );
    assert!(
        (after.1 - before.1).abs() < 3.0,
        "the y position is unchanged by a horizontal move ({before:?} -> {after:?})"
    );
}

/// The Freehand gizmo BOX rotates WITH the selection (its stored `u` follows the rotate handle) — the box
/// corners stop being axis-aligned after a rotation (Enio 2026-07-03 fix).
#[test]
fn selection_freehand_gizmo_box_rotates_with_the_selection() {
    let mut t = white_canvas(160, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(1); // Freehand
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    for p in [[90.0, 42.0], [95.0, 70.0], [50.0, 78.0], [36.0, 66.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
    t.toggle_selection_edit();
    let g = t.selection_gizmos().remove(0);
    let (center, corners) = (g.center, g.box_corners);
    assert!(
        (corners[0][1] - corners[1][1]).abs() < 0.5,
        "the box starts axis-aligned (TL.y == TR.y)"
    );
    // Grab the rotate ring just BEYOND the TR corner (along centre→corner) and swing it ~35°.
    let tr = corners[1];
    let dir = {
        let d = [tr[0] - center[0], tr[1] - center[1]];
        let m = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-3);
        [d[0] / m, d[1] / m]
    };
    let ring = [tr[0] + dir[0] * 10.0, tr[1] + dir[1] * 10.0];
    let (s, c) = 0.6f32.sin_cos(); // ~34°
    let r = [ring[0] - center[0], ring[1] - center[1]];
    let rotated = [
        center[0] + r[0] * c - r[1] * s,
        center[1] + r[0] * s + r[1] * c,
    ];
    t.on_canvas_pointer(cp(ring, PointerPhase::Down));
    t.on_canvas_pointer(cp(rotated, PointerPhase::Move));
    t.on_canvas_pointer(cp(rotated, PointerPhase::Up));
    let after = t.selection_gizmos().remove(0).box_corners;
    assert!(
        (after[0][1] - after[1][1]).abs() > 3.0,
        "after rotating, the box tilts WITH the selection (TL.y != TR.y): {after:?}"
    );
}

/// The Free-selection **Stabilization** slider drives the lasso smoothing: the same gesture yields a
/// different selection at a high stabilizer (the path lags) than at zero.
#[test]
fn free_selection_stabilizer_smooths_the_lasso() {
    let gesture = [
        [10.0, 10.0],
        [50.0, 12.0],
        [52.0, 50.0],
        [12.0, 48.0],
        [10.0, 10.0],
    ];
    let run = |stab: f32| {
        let mut t = white_canvas(64, 4.0);
        t.set_paint_tool_mode("selection");
        t.set_selection_mode(1); // Freehand
        t.set_selection_stabilizer(stab);
        assert!(
            (t.selection_stabilizer() - stab).abs() < 1e-6,
            "stabilizer round-trips"
        );
        t.on_canvas_pointer(cp(gesture[0], PointerPhase::Down));
        for p in &gesture[1..gesture.len() - 1] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(gesture[gesture.len() - 1], PointerPhase::Up));
        selected_area(&t, 64)
    };
    let low = run(0.0);
    let high = run(0.95);
    assert!(low > 0, "the un-stabilized lasso selects a region");
    assert!(
        low != high,
        "the Stabilization slider changes the lasso result (0.0 -> {low}, 0.95 -> {high})"
    );
}

/// Selection centre-square taps coalesce per shape and roll back with one undo.
#[test]
fn selection_op_tap_run_is_one_undoable_step() {
    let mut t = white_canvas(64, 2.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([4.0, 4.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Up));
    t.set_selection_bool_op(1); // Add
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([58.0, 58.0], PointerPhase::Up));
    t.selection_convert_to_curve();
    assert_eq!(t.paint.selection_shapes[1].op, 1, "starts as Add");
    let c = sel_freehand_center(&t.paint.selection_shapes[1].shape);
    let depth_before = t.undo.undo_depth();
    for _ in 0..3 {
        t.selection_gizmo_pointer(cp(c, PointerPhase::Down));
        t.selection_gizmo_pointer(cp(c, PointerPhase::Up));
    }
    assert_eq!(t.paint.selection_shapes[1].op, 2, "3 taps: 1→2→1→2");
    assert_eq!(
        t.undo.undo_depth(),
        depth_before + 1,
        "three taps coalesced into one entry"
    );
    assert!(t.undo_last());
    assert_eq!(
        t.paint.selection_shapes[1].op, 1,
        "one undo restores the pre-run op"
    );
}

#[test]
fn dbg_shrink2() {
    let mut t = white_canvas(96, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_rect_selection(32, 32, 32, 32);
    t.set_selection_offset(0.47); // -12
    for y in [40u32, 44, 45, 48, 51, 52] {
        eprint!("y={y}: ");
        for x in [40u32, 44, 45, 48, 51, 52] {
            eprint!(
                "{} ",
                if t.selection_coverage_at(x, y) > 0 {
                    "X"
                } else {
                    "."
                }
            );
        }
        eprintln!();
    }
    let m = t.selection_offset_mask_at(-12.0);
    let cnt = m.iter().filter(|&&v| v >= 128).count();
    eprintln!("mask_at(-12) area = {cnt}");
}
