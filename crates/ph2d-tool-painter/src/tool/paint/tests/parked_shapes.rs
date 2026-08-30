//! **Várias formas ao mesmo tempo: as PARQUEADAS e o que se faz com elas.** Estacionar a forma ativa
//! e começar outra, reativar a que se clica, o modo de operação booleana de cada uma, o offset que age
//! sobre todas, o Convert to Curve por forma, o Merge que funde o resultado booleano e o Simplify/Refit
//! que reduz a curva sem a degenerar.

use super::*;

/// Multi-shape SEAM (Enio 2026-07-04): the canvas pixels are a DERIVED recompose of the parked-shape set.
/// A parked ellipse (geometry only, no live editor) still stamps its outline when `restamp_shapes_preview`
/// runs — proving the parked shapes paint independently of the single active editor.
#[test]
fn recompose_stamps_parked_shapes_with_no_active_editor() {
    let mut t = white_canvas(64, 3.0);
    let before_black = (0..64 * 64).filter(|&i| t.canvas_rgba[i * 4] == 0).count();
    assert_eq!(before_black, 0, "canvas starts white");
    t.paint.parked_shapes.push(stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: [20.0, 20.0],
            u: [1.0, 0.0],
            rx: 8.0,
            ry: 8.0,
            editing: true,
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Overlay,
    });
    t.restamp_shapes_preview(&[]); // no active shape → recompose the parked-only set
    let after_black = (0..64 * 64).filter(|&i| t.canvas_rgba[i * 4] == 0).count();
    assert!(after_black > 0, "the parked ellipse outline is stamped");
    // The paint sits around the ellipse (centred at 20,20 r=8), not in a far corner.
    assert_eq!(
        px(&t, 64, 0, 0),
        [255, 255, 255, 255],
        "far corner untouched"
    );
}

/// The parked-shape set round-trips through a `ModelSnapshot` (undo capture → restore), incl. the wire op —
/// so a structural undo/redo reinstates every simultaneously-editable shape, not just the active one.
#[test]
fn parked_shapes_round_trip_through_a_snapshot() {
    let mut t = white_canvas(32, 3.0);
    t.paint.parked_shapes.push(stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: [10.0, 10.0],
            u: [1.0, 0.0],
            rx: 4.0,
            ry: 4.0,
            editing: true,
            seed: 7,
        }),
        op: stroke_multi::StrokeOp::Remove,
    });
    let snap = t.capture_shape_model();
    assert_eq!(snap.parked_shapes.len(), 1);
    assert_eq!(snap.parked_shapes[0].op, 2, "Remove op captured as wire 2");
    // Clear then restore.
    t.paint.parked_shapes.clear();
    t.restore_parked_shapes(snap.parked_shapes);
    assert_eq!(t.paint.parked_shapes.len(), 1);
    assert_eq!(
        t.paint.parked_shapes[0].op,
        stroke_multi::StrokeOp::Remove,
        "op restored from the wire value"
    );
}

/// Multi-shape GESTURE (Enio 2026-07-04): drawing a second ellipse via a Down in empty space PARKS the
/// first (keeps it painted + editable) and starts a fresh one — both outlines are stamped simultaneously,
/// and Apply bakes the whole set + clears it "todas de uma vez".
#[test]
fn empty_space_down_parks_the_active_shape_and_starts_a_new_one() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // Ellipse 1 around (16,16), radius 8 → right-edge outline at (24,16).
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([24.0, 16.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([24.0, 16.0], PointerPhase::Up));
    assert!(t.paint.ellipse.is_some(), "ellipse 1 active");
    assert!(!t.has_parked_shapes(), "only one shape so far");
    // A Down far from ellipse 1 (empty space) → park it + begin ellipse 2 around (44,44).
    t.on_canvas_pointer(cp([44.0, 44.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 44.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 44.0], PointerPhase::Up));
    assert_eq!(t.paint.parked_shapes.len(), 1, "ellipse 1 got parked");
    assert!(t.paint.ellipse.is_some(), "ellipse 2 is the active editor");
    // BOTH ellipses are stamped at once (their right-edge outline pixels are black).
    fn near_black(t: &PainterTool, x: u32, y: u32) -> bool {
        (x.saturating_sub(2)..=x + 2)
            .flat_map(|xx| (y.saturating_sub(2)..=y + 2).map(move |yy| (xx, yy)))
            .any(|(xx, yy)| px(t, 64, xx, yy) == [0, 0, 0, 255])
    }
    assert!(near_black(&t, 24, 16), "parked ellipse 1 still painted");
    assert!(near_black(&t, 52, 44), "active ellipse 2 painted");
    // Apply bakes the whole set + clears every shape "de uma vez".
    assert!(t.commit_open_shape(), "Apply committed the set");
    assert!(t.paint.ellipse.is_none(), "no active editor after Apply");
    assert!(!t.has_parked_shapes(), "parked set cleared by Apply");
    assert!(t.paint.drag_preview.is_none(), "preview baked");
    assert!(
        near_black(&t, 24, 16) && near_black(&t, 52, 44),
        "both shapes stay baked"
    );
    // Undo the Apply → the whole editable set returns.
    assert!(t.undo_last(), "undo the Apply");
    assert!(t.paint.ellipse.is_some(), "active editor restored");
    assert_eq!(t.paint.parked_shapes.len(), 1, "parked shape restored");
}

/// Clicking on a PARKED shape re-activates it (parking whatever was active) — the switch swaps which shape
/// is the live editor without dropping either.
#[test]
fn clicking_a_parked_shape_reactivates_it() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // Shape 1 around (16,16).
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([22.0, 16.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([22.0, 16.0], PointerPhase::Up));
    // Shape 2 around (46,46) (empty space → parks shape 1).
    t.on_canvas_pointer(cp([46.0, 46.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 46.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 46.0], PointerPhase::Up));
    assert_eq!(t.paint.parked_shapes.len(), 1);
    // Click back on shape 1's region → it re-activates (shape 2 parks).
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert_eq!(
        t.paint.parked_shapes.len(),
        1,
        "still exactly two shapes total"
    );
    assert!(
        t.paint.ellipse.is_some(),
        "an ellipse is active after the switch"
    );
}

/// The Stroke OPERATION mode (multi-shape) round-trips + a NEW shape adopts it as its op (Enio 2026-07-04).
#[test]
fn stroke_operation_mode_sets_the_new_shapes_op() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // Panel selects "Add" (wire 1).
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_STROKE_OP_ADD));
    assert_eq!(t.stroke_op_mode(), 1, "Operation mode set to Add");
    // Draw a shape → it is created with the Add op.
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([28.0, 20.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([28.0, 20.0], PointerPhase::Up));
    // Park it (empty-space click) and inspect its stored op.
    t.on_canvas_pointer(cp([50.0, 50.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([50.0, 50.0], PointerPhase::Up));
    assert_eq!(t.paint.parked_shapes.len(), 1);
    assert_eq!(
        t.paint.parked_shapes[0].op,
        stroke_multi::StrokeOp::Add,
        "the shape carried the Add op it was drawn with"
    );
}

/// A quick TAP on the active shape's centre square cycles its Operation (+ → − → o); a drag from the centre
/// does NOT cycle (it moves). Enio 2026-07-04.
#[test]
fn centre_square_tap_cycles_the_op_but_a_drag_does_not() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([38.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([38.0, 30.0], PointerPhase::Up));
    assert_eq!(
        t.paint.active_op,
        stroke_multi::StrokeOp::Overlay,
        "starts Overlay"
    );
    // Tap the centre (no drag) → cycles Overlay → Add.
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Up));
    assert_eq!(
        t.paint.active_op,
        stroke_multi::StrokeOp::Add,
        "centre tap cycled to Add"
    );
    // A drag from the centre must NOT cycle (moves instead).
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 30.0], PointerPhase::Up));
    assert_eq!(
        t.paint.active_op,
        stroke_multi::StrokeOp::Add,
        "a drag did not cycle the op"
    );
}

/// Multi-shape (Enio 2026-07-04): the ONE Offset slider acts on EVERY shape at once + in real time, not
/// just the active one — moving it expands a PARKED shape's outline too.
#[test]
fn offset_slider_acts_on_all_shapes_including_parked() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // Ellipse A around (16,16) r=6 → base outline near (22,16).
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([22.0, 16.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([22.0, 16.0], PointerPhase::Up));
    // Ellipse B far away (empty-space Down parks A).
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 48.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 48.0], PointerPhase::Up));
    assert_eq!(t.paint.parked_shapes.len(), 1, "A parked, B active");
    let scan = |t: &PainterTool, x: u32, y: u32| {
        (x.saturating_sub(2)..=x + 2)
            .flat_map(|xx| (y.saturating_sub(2)..=y + 2).map(move |yy| (xx, yy)))
            .any(|(xx, yy)| px(t, 64, xx, yy) == [0, 0, 0, 255])
    };
    assert!(
        !scan(&t, 40, 16),
        "A's expanded outline is NOT painted at offset 0"
    );
    // Push the global Offset slider out (~+20px) → A (parked) AND B expand together.
    t.set_brush_offset(0.6);
    t.refill_open_shape();
    assert!(
        scan(&t, 40, 16),
        "the parked ellipse A grew with the global Offset slider"
    );
}

/// Selection Convert-to-Curve (Enio 2026-07-04): multiple SEPARATE selections (created with Add) each
/// become their own editable curve — none disappears (regression: the old composed-contour path kept only
/// the first region).
#[test]
fn convert_to_curve_preserves_every_separate_selection() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    // First rect (New), top-left.
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    // Second rect (Add), bottom-right — SEPARATE from the first.
    t.set_selection_bool_op(1);
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 56.0], PointerPhase::Up));
    assert_eq!(t.paint.selection_shapes.len(), 2, "two separate selections");
    // Convert → BOTH become editable curves; neither region vanishes.
    t.selection_convert_to_curve();
    assert_eq!(
        t.paint.selection_shapes.len(),
        2,
        "both separate selections survive Convert (not collapsed to one)"
    );
    assert_eq!(t.selection_coverage_at(8, 8), 255, "first region kept");
    assert_eq!(t.selection_coverage_at(48, 48), 255, "second region kept");
}

/// Multi-shape (Enio 2026-07-04): switching BETWEEN dynamic shape methods ACCUMULATES (parks the open
/// shape) instead of baking — so a mixed-type composition builds up. Apply fires only on leaving the shape
/// system (a non-shape method / the panel button / Enter / another tool).
#[test]
fn switching_between_shape_methods_parks_not_bakes() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([22.0, 16.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([22.0, 16.0], PointerPhase::Up));
    assert!(t.paint.ellipse.is_some());
    // Ellipse → FreeHand (both dynamic): the ellipse is PARKED, not baked.
    t.set_brush_stroke_method(StrokeMethod::FreeHand.to_u8());
    assert!(
        t.paint.ellipse.is_none(),
        "ellipse editor closed on the switch"
    );
    assert_eq!(
        t.paint.parked_shapes.len(),
        1,
        "the ellipse accumulated (parked), not baked"
    );
    assert!(
        t.paint.drag_preview.is_some(),
        "still a live (un-baked) preview after the shape→shape switch"
    );
    // Leaving the shape system (→ Space, a non-shape method) BAKES the whole set.
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        !t.has_parked_shapes(),
        "non-shape switch baked + cleared the set"
    );
    assert!(t.paint.drag_preview.is_none(), "preview committed");
}

/// Multi-curve selection editing (Enio 2026-07-04): with several converted curves, a click near curve 0's
/// line targets CURVE 0 — it must NOT drop a point on the last-drawn curve. Regression for "só o gizmo da
/// última seleção funciona; clicar nos outros cria pontos na última".
#[test]
fn multi_curve_click_targets_the_nearest_curve_not_the_last() {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    // Rect 0 top-left, Rect 1 (Add) bottom-right — two SEPARATE selections.
    t.on_canvas_pointer(cp([0.0, 0.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    t.set_selection_bool_op(1);
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 56.0], PointerPhase::Up));
    t.selection_convert_to_curve(); // 2 editable curves + gizmos, edit mode on
    assert_eq!(t.paint.selection_shapes.len(), 2);
    // A Down on/near CURVE 0's outline (top-left) must grab/insert on curve 0 — never the last (curve 1).
    t.on_canvas_pointer(cp([8.0, 0.0], PointerPhase::Down));
    let grab = t
        .paint
        .selection_grab
        .as_ref()
        .expect("a grab resulted near curve 0");
    assert_eq!(
        grab.shape, 0,
        "the click targeted the NEAREST curve (0), not the last-drawn (1)"
    );
}

/// Stroke boolean (Enio 2026-07-04): two OVERLAPPING Add ellipses union — the combined region's OUTER
/// contour is stroked, and the inner arcs (where the two circles cross, now inside the union) VANISH. With
/// Overlay both full outlines would paint; Add must remove the overlap arcs.
#[test]
fn two_overlapping_add_ellipses_union_their_outline() {
    let mut t = white_canvas(64, 2.0);
    let add_ellipse = |c: [f32; 2], r: f32| stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: c,
            u: [1.0, 0.0],
            rx: r,
            ry: r,
            editing: true,
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Add,
    };
    // Two OVERLAPPING Add ellipses (centres 20 apart, r=12 each → overlap around x=34).
    t.paint.parked_shapes.push(add_ellipse([24.0, 32.0], 12.0));
    t.paint.parked_shapes.push(add_ellipse([44.0, 32.0], 12.0));
    t.restamp_shapes_preview(&[]); // recompose → boolean union → stroked contour
    let scan = |t: &PainterTool, x: u32, y: u32| {
        (x.saturating_sub(2)..=x + 2)
            .flat_map(|xx| (y.saturating_sub(2)..=y + 2).map(move |yy| (xx, yy)))
            .any(|(xx, yy)| px(t, 64, xx, yy) == [0, 0, 0, 255])
    };
    assert!(
        scan(&t, 12, 32),
        "the union's OUTER boundary (A's left edge) is stroked"
    );
    assert!(
        !scan(&t, 34, 32),
        "the overlap centre is INSIDE the union → the inner arcs vanished (boolean, not overlay)"
    );
}

/// Stroke boolean also works for a CLOSED Line (Enio 2026-07-04: "add e remove devem funcionar para
/// qualquer linha fechada"). Two overlapping closed-line squares with Add union their outline.
#[test]
fn two_overlapping_closed_line_squares_union() {
    let mut t = white_canvas(64, 2.0);
    let add_square = |c: [f32; 2], r: f32| stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Line(crate::undo::LineState {
            points: vec![
                [c[0] - r, c[1] - r],
                [c[0] + r, c[1] - r],
                [c[0] + r, c[1] + r],
                [c[0] - r, c[1] + r],
            ],
            closed: true,
            editing: true,
            corner_mods: vec![(0, 0.0); 4],
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Add,
    };
    t.paint.parked_shapes.push(add_square([22.0, 32.0], 12.0)); // x 10..34
    t.paint.parked_shapes.push(add_square([42.0, 32.0], 12.0)); // x 30..54 — overlaps 30..34
    t.restamp_shapes_preview(&[]);
    let scan = |t: &PainterTool, x: u32, y: u32| {
        (x.saturating_sub(2)..=x + 2)
            .flat_map(|xx| (y.saturating_sub(2)..=y + 2).map(move |yy| (xx, yy)))
            .any(|(xx, yy)| px(t, 64, xx, yy) == [0, 0, 0, 255])
    };
    assert!(scan(&t, 10, 32), "the union's outer-left edge is stroked");
    assert!(
        !scan(&t, 32, 32),
        "the overlap centre is inside the union → the shared vertical edges vanished"
    );
}

/// Regression (Enio 2026-07-04): a brush stroke, UNDO everything, then a selection — the selection must
/// still draw (the two systems must not interfere).
#[test]
fn selection_works_after_stroking_and_undoing_everything() {
    let mut t = white_canvas(64, 3.0);
    // A brush stroke.
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
    assert_ne!(px(&t, 64, 20, 20), [255, 255, 255, 255], "stroke painted");
    // Undo until nothing is left.
    let mut guard = 0;
    while t.undo_last() && guard < 50 {
        guard += 1;
    }
    // Now use the selection system (Rectangle).
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2);
    t.on_canvas_pointer(cp([10.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Up));
    // The SHELL draws the selection overlay only when `selection_active()` AND `selection_overlay_rgba` is
    // Some — assert BOTH, not just the raw mask, so a stuck `selection_active` is caught.
    assert!(
        t.selection_active(),
        "selection_active set after stroke+undo+select"
    );
    assert!(
        t.selection_overlay_rgba(0).is_some(),
        "the shell selection overlay has pixels to draw"
    );
    assert!(
        t.selection_coverage_at(20, 20) > 0,
        "the selection is applied after stroke+undo (systems must not interfere)"
    );
}

/// Same regression but with a SHAPE stroke (ellipse via the multi-shape system) — draw, Apply, undo all,
/// then a selection must still draw (drag_preview / parked-shape state must not leak into selection).
#[test]
fn selection_works_after_shape_stroke_and_undo() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Up));
    t.commit_open_shape(); // Apply (bake)
    let mut guard = 0;
    while t.undo_last() && guard < 50 {
        guard += 1;
    }
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2);
    t.on_canvas_pointer(cp([10.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 55.0], PointerPhase::Up));
    assert!(
        t.selection_coverage_at(20, 47) > 0,
        "selection draws after a shape stroke + undo"
    );
    assert!(!t.has_parked_shapes(), "no parked shapes leaked past undo");
    assert!(
        t.paint.drag_preview.is_none(),
        "no stale stroke preview leaked into selection"
    );
}

/// **Stroke Simplify** on a CLOSED curve uses the robust reducer (Enio 2026-07-05): the old Schneider
/// `simplify_curve` degenerated a converted circle to 2 identical points; `simplify_closed_smooth` keeps it
/// closed with far fewer, non-degenerate anchors.
#[test]
fn stroke_simplify_reduces_a_dense_closed_curve_without_degenerating() {
    let mut t = white_canvas(64, 2.0);
    // Draw a Polygon (square), then Convert → a DENSE closed curve: the straight edges densify to many
    // COLLINEAR anchors, which Simplify removes (the corners stay). A circle would already sit at its
    // fidelity minimum (no reduction) — the polygon proves the reducer actually sheds points.
    t.paint.brush.stroke_method = StrokeMethod::Polygon;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    assert!(t.paint.polygon.is_some(), "polygon drawn");
    assert!(
        t.convert_open_shape_to_curve(),
        "converted to a dense curve"
    );
    let dense = t
        .paint
        .curve
        .as_ref()
        .expect("curve open after convert")
        .model
        .points
        .len();
    assert!(dense >= 8, "the converted curve is dense: {dense}");
    // Simplify the CLOSED curve — must reduce, stay closed, and NOT degenerate (the old Schneider fit
    // collapsed a start==end loop to 2 identical points).
    assert!(t.curve_simplify(), "simplify applied");
    let ed = t.paint.curve.as_ref().expect("curve still open");
    assert!(ed.model.closed, "the curve stays closed");
    assert!(
        ed.model.points.len() > 2,
        "NOT degenerate, got {} points",
        ed.model.points.len()
    );
    assert!(
        ed.model.points.len() < dense,
        "reduced from the dense {dense} anchors to {}",
        ed.model.points.len()
    );
}

/// **Simplify = least-squares REFIT** (Enio 2026-07-05, the Inkscape/paper.js pipeline): a dense converted
/// circle collapses to VERY FEW anchors whose fitted curve still hugs the true circle to sub-pixel; smooth
/// joins are **Aligned**, corners **Free** — the two kinds a least-squares fit produces. Pressing again
/// keeps reducing (never increases) down to a floor.
#[test]
fn simplify_refits_a_circle_to_few_faithful_anchors() {
    use super::curve_handle::HandleKind;
    let mut t = white_canvas(96, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([88.0, 48.0], PointerPhase::Move)); // radius 40 → dense convert
    t.on_canvas_pointer(cp([88.0, 48.0], PointerPhase::Up));
    assert!(t.convert_open_shape_to_curve());
    let dense = t.paint.curve.as_ref().unwrap().model.points.len();
    assert!(dense >= 12, "dense start: {dense}");
    assert!(t.curve_simplify(), "first press applied");
    let ed = t.paint.curve.as_ref().unwrap();
    let n1 = ed.model.points.len();
    // A circle is smooth → NO corners: every anchor is a fitted smooth join (Aligned), and the fit needs
    // only a handful of anchors ("números de pontos bem reduzidos").
    assert!(
        n1 <= dense / 2,
        "big reduction on the first press: {dense} → {n1}"
    );
    for k in &ed.model.kinds {
        assert!(
            matches!(k, HandleKind::Aligned),
            "a smooth circle has only Aligned fitted joins, got {k:?}"
        );
    }
    // FIDELITY: the fitted curve still hugs the true circle (centre 48,48 r=40) to ~1px.
    let mut spine = Vec::new();
    super::curve_geom::flatten_spine(&ed.model.points, &ed.model.handles, true, &mut spine);
    for p in &spine {
        let r = ((p[0] - 48.0).powi(2) + (p[1] - 48.0).powi(2)).sqrt();
        assert!(
            (r - 40.0).abs() < 1.5,
            "fitted curve stays on the circle: r={r:.2}"
        );
    }
    // Repeated presses never increase the count and respect the floor.
    let mut prev = n1;
    for _ in 0..4 {
        if !t.curve_simplify() {
            break; // nothing left to shed — a valid stop
        }
        let n = t.paint.curve.as_ref().unwrap().model.points.len();
        assert!(n < prev, "each accepted press reduces: {prev} → {n}");
        assert!(n >= 4, "never collapses below a real ring: {n}");
        prev = n;
    }
}

/// **Simplify preserves CORNERS as Free anchors** (Enio 2026-07-05): a converted regular polygon refits to
/// exactly its vertices — tagged `Free` (independent fitted arms), never smoothed away.
#[test]
fn simplify_refits_a_polygon_to_exactly_its_free_corners() {
    use super::curve_handle::HandleKind;
    let mut t = white_canvas(96, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Polygon;
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Move)); // radius 32
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Up));
    let sides = t
        .paint
        .polygon
        .as_ref()
        .expect("polygon open")
        .to_state()
        .sides as usize;
    assert!(t.convert_open_shape_to_curve());
    let dense = t.paint.curve.as_ref().unwrap().model.points.len();
    assert!(dense >= 8, "dense start: {dense}");
    assert!(t.curve_simplify(), "simplify applied");
    let ed = t.paint.curve.as_ref().unwrap();
    let corners = ed
        .model
        .kinds
        .iter()
        .filter(|k| matches!(k, HandleKind::Free))
        .count();
    assert_eq!(
        corners, sides,
        "every polygon vertex survives as a Free corner anchor"
    );
    for k in &ed.model.kinds {
        assert!(
            matches!(k, HandleKind::Free | HandleKind::Aligned),
            "only Free corners + Aligned smooth joins, got {k:?}"
        );
    }
    assert!(
        ed.model.points.len() <= sides * 2,
        "a straight-edged polygon needs little beyond its corners: {}",
        ed.model.points.len()
    );
}

/// **Drawing-only Offset** (Enio 2026-07-05, the Selection model): offsetting a CONVERTED curve leaves the
/// whole EDITOR — control anchors AND the guide line (spine) — on the PRISTINE curve; ONLY the painted
/// drawing (pixels) shifts. Nothing in the editor moves or bunches ("ponto e linha ficassem parados e apenas
/// o desenho sofresse o offset").
#[test]
fn offset_moves_only_the_painted_drawing_not_the_editor() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(128, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Move)); // radius 40
    t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Up));
    assert!(
        t.convert_open_shape_to_curve(),
        "converted to a dense curve"
    );
    let ov0 = t.curve_overlay().expect("curve open");
    let pristine = ov0.points.clone();
    let pristine_spine = ov0.spine.clone();
    let max_r = |ps: &[[f32; 2]]| {
        ps.iter()
            .map(|p| ((p[0] - 64.0).powi(2) + (p[1] - 64.0).powi(2)).sqrt())
            .fold(0.0f32, f32::max)
    };
    // Topmost painted (black) row — the extent of the DRAWING.
    let top_black = |t: &PainterTool| -> u32 {
        for y in 0..128 {
            for x in 0..128 {
                if px(t, 128, x, y) == [0, 0, 0, 255] {
                    return y;
                }
            }
        }
        128
    };
    let base_top = top_black(&t);
    // Big inward offset — via the panel event, so the preview re-fills like the real app.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.35)); // −30px
    let ov = t.curve_overlay().expect("curve still open");
    // The EDITOR is untouched: control points AND the guide line both stay on the pristine curve.
    assert_eq!(
        ov.points.len(),
        pristine.len(),
        "no control points added/removed"
    );
    for (a, b) in ov.points.iter().zip(&pristine) {
        assert!(
            (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3,
            "control point stayed pristine: {a:?} vs {b:?}"
        );
    }
    assert!(
        (max_r(&ov.spine) - max_r(&pristine_spine)).abs() < 1.0,
        "the guide LINE stayed on the pristine curve (radius {:.1} vs {:.1})",
        max_r(&ov.spine),
        max_r(&pristine_spine)
    );
    // But the DRAWING (painted pixels) moved inward — its topmost black row dropped by ~the offset.
    let off_top = top_black(&t);
    assert!(
        off_top > base_top + 15,
        "the painted drawing shifted inward: top black row {base_top} → {off_top}"
    );
}

/// A parked Add ellipse `StrokeShape` fixture for the Convert/Merge tests.
fn add_ellipse_shape(c: [f32; 2], r: f32) -> stroke_multi::StrokeShape {
    stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: c,
            u: [1.0, 0.0],
            rx: r,
            ry: r,
            editing: true,
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Add,
    }
}

/// **Convert to Curve** is now PER-SHAPE (Enio 2026-07-05): each shape becomes its OWN editable dense curve,
/// preserving its Operation — it NEVER merges (that is the separate Merge button). Two Add ellipses → two
/// curves (one active + one parked), both still Add.
#[test]
fn convert_to_curve_is_per_shape_not_merged() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.paint
        .parked_shapes
        .push(add_ellipse_shape([24.0, 32.0], 12.0));
    t.paint
        .parked_shapes
        .push(add_ellipse_shape([44.0, 32.0], 12.0));
    assert!(t.convert_open_shape_to_curve(), "converted per-shape");
    assert!(
        t.paint.curve.is_some(),
        "the first shape is now the live editable Curve"
    );
    assert_eq!(
        t.paint.parked_shapes.len(),
        1,
        "the OTHER ellipse became its OWN parked curve — NOT merged"
    );
    assert!(
        matches!(
            t.paint.parked_shapes[0].state,
            crate::undo::ShapeEditState::Curve(_)
        ),
        "the parked shape is now a Curve, not still an Ellipse"
    );
    assert_eq!(
        t.paint.active_op,
        stroke_multi::StrokeOp::Add,
        "the active curve keeps its Add op"
    );
    assert_eq!(
        t.paint.parked_shapes[0].op,
        stroke_multi::StrokeOp::Add,
        "the parked curve keeps its Add op"
    );
}

/// **Merge Curves** folds interacting Add shapes into ONE editable curve tracing the union — the behavior
/// Convert used to auto-do, now its own button (Enio 2026-07-05).
#[test]
fn merge_curves_folds_the_boolean_result_into_one_curve() {
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // Two OVERLAPPING Add ellipses → one boolean region.
    t.paint
        .parked_shapes
        .push(add_ellipse_shape([24.0, 32.0], 12.0));
    t.paint
        .parked_shapes
        .push(add_ellipse_shape([40.0, 32.0], 12.0));
    assert!(t.merge_open_shapes_to_curves(), "merged the fill composite");
    assert!(
        t.paint.curve.is_some(),
        "the merged result is the live editable Curve"
    );
    assert_eq!(
        t.paint.parked_shapes.len(),
        0,
        "the two ellipses folded into the single curve (one region → one curve)"
    );
    // The curve traces the union boundary — its outer-left edge is painted, the overlap centre is not.
    let scan = |t: &PainterTool, x: u32, y: u32| {
        (x.saturating_sub(2)..=x + 2)
            .flat_map(|xx| (y.saturating_sub(2)..=y + 2).map(move |yy| (xx, yy)))
            .any(|(xx, yy)| px(t, 64, xx, yy) == [0, 0, 0, 255])
    };
    assert!(
        scan(&t, 12, 32),
        "the merged curve strokes the union outer boundary"
    );
}

/// **Corner offset stays SHARP** (Enio 2026-07-05: "arredonda as quinas no offset"): a refit corner must be
/// re-anchored on the TRUE edge-line intersection past the trace-smoothing's rounded tip — because an offset
/// AMPLIFIES tip rounding by |d|. A smoothing-rounded square ring refits to razor corners at the exact square
/// vertices, and its outward offset reaches each true miter apex instead of rounding it off.
#[test]
fn refit_corner_offset_reaches_the_sharp_miter_apex() {
    use super::curve_handle::HandleKind;
    // A 100×100 square ring sampled at ~0.8px, then smoothed like the mask trace (2× 3-point moving
    // average) — the corners arrive ROUNDED (~2px tip), exactly what a merged contour looks like.
    let (lo, hi) = (50.0f32, 150.0f32);
    let mut ring: Vec<[f32; 2]> = Vec::new();
    let steps = 125; // 0.8px per side sample
    for i in 0..steps {
        let t = lo + (hi - lo) * (i as f32 / steps as f32);
        ring.push([t, lo]);
    }
    for i in 0..steps {
        let t = lo + (hi - lo) * (i as f32 / steps as f32);
        ring.push([hi, t]);
    }
    for i in 0..steps {
        let t = hi - (hi - lo) * (i as f32 / steps as f32);
        ring.push([t, hi]);
    }
    for i in 0..steps {
        let t = hi - (hi - lo) * (i as f32 / steps as f32);
        ring.push([lo, t]);
    }
    let n = ring.len();
    for _ in 0..2 {
        let prev = ring.clone();
        for i in 0..n {
            let a = prev[(i + n - 1) % n];
            let b = prev[i];
            let c = prev[(i + 1) % n];
            ring[i] = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
        }
    }
    let r = super::curve_refit::refit_closed_spine(&ring, 1.0).expect("refit succeeds");
    // 4 Free corners, each re-anchored within ~1px of the TRUE square vertex (the smoothing pulled the
    // traced tip ~1px inward; the edge-line intersection restores it).
    let true_corners = [[lo, lo], [hi, lo], [hi, hi], [lo, hi]];
    let mut found = 0;
    for (p, k) in r.points.iter().zip(&r.kinds) {
        if matches!(k, HandleKind::Free) {
            let d = true_corners
                .iter()
                .map(|c| ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sqrt())
                .fold(f32::INFINITY, f32::min);
            assert!(
                d < 1.2,
                "corner anchor sits on the true vertex: off by {d:.2}px"
            );
            found += 1;
        }
    }
    assert_eq!(found, 4, "all four square corners survive as Free anchors");
    // OFFSET: expand by 12px. The outward offset of a square corner miters to the apex (corner ± 12 on both
    // axes). A rounded corner would fall ~12·(√2−1) ≈ 5px short of the apex — assert we get within 1.5px.
    for d in [12.0f32, -12.0] {
        let (op, oh, _) = super::curve_offset::offset_curve_refined(&r.points, &r.handles, d, true);
        let mut spine = Vec::new();
        super::curve_geom::flatten_spine(&op, &oh, true, &mut spine);
        let grew = spine.iter().any(|p| p[0] < lo - 6.0);
        if !grew {
            continue; // this sign offsets inward — the apex test is for the outward one
        }
        for c in &true_corners {
            let apex = [
                c[0] + 12.0 * (c[0] - 100.0).signum(),
                c[1] + 12.0 * (c[1] - 100.0).signum(),
            ];
            let best = spine
                .iter()
                .map(|p| ((p[0] - apex[0]).powi(2) + (p[1] - apex[1]).powi(2)).sqrt())
                .fold(f32::INFINITY, f32::min);
            assert!(
                best < 1.5,
                "the offset reaches the sharp miter apex {apex:?}: nearest {best:.2}px"
            );
        }
        return; // outward direction found + asserted
    }
    panic!("neither offset direction grew the square outward");
}

/// **Merge produces a CLEAN, reduced-point curve** (Enio 2026-07-05): a sharp-waist peanut (two barely-
/// overlapping Add ellipses) used to fold into a DENSE curve whose concave waist spiked into a self-crossing
/// needle. Merge now runs the robust `simplify_closed_smooth` reducer — fewer anchors, no self-crossing.
#[test]
fn merge_produces_a_clean_low_point_curve_at_a_sharp_waist() {
    let mut t = white_canvas(200, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // Barely-overlapping ellipses → a pinched waist (the concave region that spiked).
    t.paint
        .parked_shapes
        .push(add_ellipse_shape([80.0, 100.0], 45.0));
    t.paint
        .parked_shapes
        .push(add_ellipse_shape([140.0, 100.0], 45.0));
    assert!(t.merge_open_shapes_to_curves(), "merged");
    let ov = t.curve_overlay().expect("merged curve open");
    let pts = &ov.points;
    let n = pts.len();
    assert!(n >= 6, "still a real multipoint curve: {n}");
    assert!(
        n <= 64,
        "REDUCED point count (not the dense-fit hundreds): {n}"
    );
    // No self-crossing of the control polygon — the concave waist is a clean corner, not a needle spike.
    let cross = |a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]| {
        let o = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
            (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
        };
        (o(a, b, c) > 0.0) != (o(a, b, d) > 0.0) && (o(c, d, a) > 0.0) != (o(c, d, b) > 0.0)
    };
    let mut crossings = 0;
    for i in 0..n {
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            if cross(pts[i], pts[(i + 1) % n], pts[j], pts[(j + 1) % n]) {
                crossings += 1;
            }
        }
    }
    assert_eq!(
        crossings, 0,
        "the merged control polygon does not self-cross"
    );
}

/// Polygon boolean uses the SAME perimeter as the gizmo (`polygon_perimeter`, first vertex at top) — a
/// sides=4 Add polygon composites as a DIAMOND (top/left/bottom/right vertices), NOT a corner-seeded
/// axis-aligned square. Regression for the gizmo-vs-drawing rotation divergence (Enio 2026-07-04).
#[test]
fn polygon_boolean_matches_the_gizmo_phase_not_the_selection_corner_seed() {
    let mut t = white_canvas(64, 2.0);
    t.paint.parked_shapes.push(stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Polygon(crate::undo::PolygonState {
            center: [32.0, 32.0],
            u: [1.0, 0.0],
            rx: 12.0,
            ry: 12.0,
            sides: 4,
            editing: true,
            seed: 1,
        }),
        op: stroke_multi::StrokeOp::Add,
    });
    t.restamp_shapes_preview(&[]);
    let scan = |t: &PainterTool, x: u32, y: u32| {
        (x.saturating_sub(2)..=x + 2)
            .flat_map(|xx| (y.saturating_sub(2)..=y + 2).map(move |yy| (xx, yy)))
            .any(|(xx, yy)| px(t, 64, xx, yy) == [0, 0, 0, 255])
    };
    assert!(
        scan(&t, 32, 44),
        "the diamond TOP vertex is stroked (gizmo phase)"
    );
    assert!(
        !scan(&t, 44, 44),
        "the box CORNER is empty — not the selection's corner-seeded square (no 45° divergence)"
    );
}

/// **Coalesced Simplify undo** (Enio 2026-07-05: "cada mínima ação entra na sequência undo/redo"): a run of
/// progressive Simplify presses records ONE undo entry, and a single Ctrl+Z restores the pre-run curve.
#[test]
fn simplify_run_is_one_undo_step_back_to_the_dense_curve() {
    let mut t = white_canvas(96, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Polygon;
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Up));
    assert!(t.convert_open_shape_to_curve());
    let dense = t.paint.curve.as_ref().unwrap().model.points.len();
    let depth_before = t.undo.undo_depth();
    // Three progressive presses…
    let mut presses = 0;
    for _ in 0..3 {
        if t.curve_simplify() {
            presses += 1;
        }
    }
    assert!(presses >= 1, "at least one press reduced");
    assert_eq!(
        t.undo.undo_depth(),
        depth_before + 1,
        "{presses} Simplify presses coalesced into ONE undo entry"
    );
    // …and ONE undo restores the pre-run dense curve.
    assert!(t.undo_last());
    assert_eq!(
        t.paint.curve.as_ref().unwrap().model.points.len(),
        dense,
        "one Ctrl+Z returns to the curve before the FIRST press"
    );
}

/// **Op-cycle taps are undoable and coalesced**: the stroke centre-square tap previously recorded NO undo
/// (`active_op` wasn't captured); now a run of taps is ONE entry and undo restores the pre-run Operation.
#[test]
fn stroke_op_tap_run_is_one_undoable_step() {
    use super::stroke_multi::StrokeOp;
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Up));
    assert_eq!(t.paint.active_op, StrokeOp::Overlay);
    let depth_before = t.undo.undo_depth();
    t.cycle_active_op(); // Overlay → Add
    t.cycle_active_op(); // Add → Remove
    assert_eq!(t.paint.active_op, StrokeOp::Remove);
    assert_eq!(
        t.undo.undo_depth(),
        depth_before + 1,
        "two taps coalesced into one entry"
    );
    assert!(t.undo_last());
    assert_eq!(
        t.paint.active_op,
        StrokeOp::Overlay,
        "one undo restores the pre-run Operation"
    );
}
