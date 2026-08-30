//! **O editor de LINHA poligonal.** Vários cliques constroem a polilinha, o último a fecha ou a
//! termina; o arrasto de um ponto, o snap de ângulo e o de coluna, o snap à grade, o filete do canto, o
//! gizmo de transformação da linha inteira e o offset vivo.

use super::*;

#[test]
fn line_polyline_editor_replaces_the_old_single_segment() {
    // The old single-segment Line (Alt/45° drag on the generic paint path) is replaced by the polyline
    // editor: setting the Line method routes canvas events to the editor (a click drops a corner point),
    // NOT the one-shot drag. (The 15° Shift snap is a later increment.)
    let mut t = line_tool();
    t.on_canvas_pointer(cp([8.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 30.0], PointerPhase::Up));
    assert!(
        t.line_overlay().is_some_and(|o| o.points.len() == 1),
        "a Line click opens the polyline editor with one corner point"
    );
}

// ── Line polyline editor (core: multi-click points, end/close, drag, commit/cancel/undo) ──────────

/// A `PainterTool` on a 64² white canvas set to the Line method, with a known grab tolerance.
fn line_tool() -> PainterTool {
    let mut t = white_canvas(64, 5.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_shape_grab_tol_px(8.0);
    t
}

#[test]
fn line_multi_click_builds_a_polyline_then_finishes_on_the_last_point() {
    // Each click drops a corner point; clicking the LAST point ends creation → the editing phase.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 48.0);
    let ov = t.line_overlay().expect("a line session is live");
    assert_eq!(ov.points.len(), 3, "three corner points dropped");
    assert!(!ov.editing, "still in the drawing phase");
    // Click the last point again → end point-creation.
    click(&mut t, 48.0, 48.0);
    let ov = t.line_overlay().expect("session persists into editing");
    assert!(ov.editing, "clicking the last point ended creation");
    assert!(!ov.closed, "an open polyline");
    assert_eq!(
        ov.points.len(),
        3,
        "no extra point added by the finishing click"
    );
}

#[test]
fn line_closes_when_the_last_click_lands_on_the_first_point() {
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 48.0);
    click(&mut t, 16.0, 16.0); // on the first point → close + finish
    let ov = t.line_overlay().expect("session persists");
    assert!(ov.closed, "clicking the first point closed the loop");
    assert!(ov.editing, "and ended creation");
}

#[test]
fn line_commit_paints_the_polyline_and_is_one_undo_step() {
    // Enter/Apply (commit_open_shape aggregator) bakes the line, closes the session, one undo step.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 16.0); // finish on the last point
    assert!(t.commit_open_shape(), "commit baked the open line");
    assert!(t.line_overlay().is_none(), "the session closed on commit");
    // A dab lands at the first vertex → that pixel is painted (black brush).
    assert!(
        px(&t, 64, 16, 16)[0] < 128,
        "the polyline painted the canvas, got {}",
        px(&t, 64, 16, 16)[0]
    );
    // Undo is one step: it reinstates the open editor (the line shows again as a live preview — a further
    // undo would remove the creation), per the unified shape+paint timeline.
    assert!(t.can_undo());
    assert!(t.undo_last());
    assert!(
        t.line_overlay().is_some(),
        "undo reinstated the open line editor"
    );
}

#[test]
fn line_cancel_reverts_the_preview() {
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 16.0); // finish
    assert!(t.cancel_open_shape(), "cancel closed the session");
    assert!(t.line_overlay().is_none());
    assert_eq!(
        px(&t, 64, 16, 16),
        [255, 255, 255, 255],
        "cancel reverted the preview to pristine"
    );
}

#[test]
fn line_edit_phase_drags_a_corner_point() {
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 16.0); // finish → editing
    assert!(t.line_overlay().unwrap().editing);
    // Grab the first point and drag it.
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([20.0, 30.0], PointerPhase::Up));
    let ov = t.line_overlay().expect("still editing");
    assert_eq!(ov.points[0], [20.0, 30.0], "the grabbed corner moved");
    assert_eq!(ov.points[1], [48.0, 16.0], "the other corner stayed put");
}

#[test]
fn line_undo_redo_is_point_by_point_during_creation() {
    // Each click is its own undo step: undo peels one point at a time (not the whole polyline at once).
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0); // point 1
    click(&mut t, 40.0, 16.0); // point 2
    click(&mut t, 40.0, 40.0); // point 3
    assert_eq!(t.line_overlay().unwrap().points.len(), 3);
    // Undo removes the last point, one at a time.
    assert!(t.undo_last());
    assert_eq!(
        t.line_overlay().unwrap().points.len(),
        2,
        "undo peeled ONE point"
    );
    assert!(t.undo_last());
    assert_eq!(
        t.line_overlay().unwrap().points.len(),
        1,
        "undo peeled another point"
    );
    // Undoing the first point removes the editor entirely (back to no shape).
    assert!(t.undo_last());
    assert!(
        t.line_overlay().is_none(),
        "undoing the first point closed the editor"
    );
    // Redo re-adds points one at a time.
    assert!(t.redo_last());
    assert_eq!(
        t.line_overlay().unwrap().points.len(),
        1,
        "redo re-added the first point"
    );
    assert!(t.redo_last());
    assert_eq!(
        t.line_overlay().unwrap().points.len(),
        2,
        "redo re-added the second point"
    );
}

#[test]
fn line_shift_snaps_the_new_segment_to_15_degrees() {
    // With Shift armed, a near-horizontal click snaps to EXACTLY horizontal (0°) from the anchor, keeping
    // the drag distance — the 15°-graduated direction snap (transcendental-free).
    let mut t = line_tool();
    click(&mut t, 20.0, 20.0); // anchor point
    t.set_line_snap(true);
    click(&mut t, 60.0, 24.0); // dx=40, dy=4 → ~5.7° → snaps to 0°
    let p = t.line_overlay().unwrap().points[1];
    let len = (40.0_f32 * 40.0 + 4.0 * 4.0).sqrt();
    assert!(
        (p[1] - 20.0).abs() < 0.01,
        "snapped onto the anchor's row (horizontal), got {p:?}"
    );
    assert!(
        (p[0] - (20.0 + len)).abs() < 0.05,
        "distance preserved along the snapped ray, got {p:?}"
    );
}

#[test]
fn line_dragging_a_point_mid_draw_moves_it_without_adding_a_new_one() {
    // Mid-drawing, grabbing an existing point moves it (no new point); creation then continues in empty.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0); // point 0
    click(&mut t, 48.0, 16.0); // point 1
    assert!(!t.line_overlay().unwrap().editing, "still drawing");
    // Grab point 0 and drag it.
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([10.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([10.0, 40.0], PointerPhase::Up));
    let ov = t.line_overlay().unwrap();
    assert_eq!(ov.points.len(), 2, "dragging created NO new point");
    assert_eq!(ov.points[0], [10.0, 40.0], "point 0 moved");
    assert!(!ov.editing, "still drawing after a mid-draw move");
    // A click in EMPTY space continues creation from the last point.
    click(&mut t, 60.0, 60.0);
    assert_eq!(
        t.line_overlay().unwrap().points.len(),
        3,
        "creation continues in empty space"
    );
}

#[test]
fn line_click_on_an_existing_point_never_adds_a_duplicate() {
    // Points are created only in empty space: a click ON a point selects it (no duplicate, no end).
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 16.0, 16.0); // tap point 0 (n=2 → not close; not last → not end): select only
    let ov = t.line_overlay().unwrap();
    assert_eq!(ov.points.len(), 2, "no duplicate point created");
    assert_eq!(ov.selected, Some(0), "the clicked point is selected");
    assert!(!ov.editing, "a select does not end creation");
}

#[test]
fn line_press_drag_on_empty_creates_the_point_and_drags_it() {
    // A press in EMPTY space creates a corner AND grabs it, so the same held drag positions it live (this
    // is how you set the angle with Shift). Release settles it — no separate rubber-band, no duplicate.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0); // corner 0
    t.on_canvas_pointer(cp([40.0, 16.0], PointerPhase::Down)); // create corner 1 in empty space
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move)); // drag it (same press)
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
    let ov = t.line_overlay().unwrap();
    assert_eq!(ov.points.len(), 2, "one corner created, none duplicated");
    assert_eq!(
        ov.points[1],
        [40.0, 40.0],
        "the created corner followed the drag"
    );
}

#[test]
fn line_dragging_the_last_point_moves_it_not_creates_a_new_one() {
    // The reported bug: trying to drag the last point used to drop a NEW point instead of moving it.
    // A press ON the last point grabs it; the drag moves it (no new point, creation not ended).
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0); // last = corner 1
    t.on_canvas_pointer(cp([48.0, 16.0], PointerPhase::Down)); // grab the last point
    t.on_canvas_pointer(cp([50.0, 40.0], PointerPhase::Move)); // drag it
    t.on_canvas_pointer(cp([50.0, 40.0], PointerPhase::Up));
    let ov = t.line_overlay().unwrap();
    assert_eq!(
        ov.points.len(),
        2,
        "dragging the last point added NO new point"
    );
    assert_eq!(ov.points[1], [50.0, 40.0], "the last point moved");
    assert!(!ov.editing, "a drag does not end creation");
}

#[test]
fn line_tap_within_slop_on_last_point_ends_creation_not_moves() {
    // Tap-vs-drag: a press that stays within the slop (jitter) is a TAP, not a drag — on the last point
    // that ENDS creation and leaves the point exactly where it was (no accidental nudge). tol 8 → slop 3.2.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    t.on_canvas_pointer(cp([48.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([49.0, 17.0], PointerPhase::Move)); // within slop → still a tap
    t.on_canvas_pointer(cp([49.0, 17.0], PointerPhase::Up));
    let ov = t.line_overlay().unwrap();
    assert!(
        ov.editing,
        "a within-slop tap on the last point ended creation"
    );
    assert_eq!(
        ov.points[1],
        [48.0, 16.0],
        "the point did not move (a tap, not a drag)"
    );
}

#[test]
fn line_finish_points_ends_creation_as_one_undo_step() {
    // Right-click (shell) → `line_finish_points`: end point-creation, enter editing, one undo step.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    assert!(!t.line_overlay().unwrap().editing, "still drawing");
    assert!(t.line_finish_points(), "right-click ended point-creation");
    assert!(
        t.line_overlay().unwrap().editing,
        "now in the editing phase"
    );
    assert!(!t.line_finish_points(), "no-op once already editing");
    assert!(t.undo_last());
    assert!(
        !t.line_overlay().unwrap().editing,
        "undo re-entered the drawing phase"
    );
}

#[test]
fn line_transform_gizmo_absent_while_drawing_present_when_editing() {
    // The whole-line transform gizmo is editing-phase chrome only (drawing is still placing points).
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    assert!(
        t.line_overlay().unwrap().transform_gizmo.is_none(),
        "no gizmo while drawing"
    );
    assert!(t.line_finish_points());
    assert!(
        t.line_overlay().unwrap().transform_gizmo.is_some(),
        "the gizmo appears once creation ends"
    );
}

#[test]
fn line_transform_gizmo_moves_the_whole_line() {
    // Grabbing the gizmo CENTRE handle (the inflated bbox centre) translates every corner as one. For the
    // horizontal line (16,16)→(48,16) with tol 8 the centre handle sits at (32,16). Drag it by (+10,+10).
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    assert!(t.line_finish_points(), "end creation → editing");
    t.on_canvas_pointer(cp([32.0, 16.0], PointerPhase::Down)); // centre handle (not a corner point)
    t.on_canvas_pointer(cp([42.0, 26.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([42.0, 26.0], PointerPhase::Up));
    let ov = t.line_overlay().unwrap();
    assert_eq!(
        ov.points[0],
        [26.0, 26.0],
        "corner 0 translated with the line"
    );
    assert_eq!(
        ov.points[1],
        [58.0, 26.0],
        "corner 1 translated with the line"
    );
    // One undo step reverts the whole-line transform.
    assert!(t.undo_last());
    let ov = t.line_overlay().unwrap();
    assert_eq!(ov.points[0], [16.0, 16.0], "undo reverted the transform");
    assert_eq!(ov.points[1], [48.0, 16.0]);
}

#[test]
fn line_dragging_the_fillet_handle_rounds_the_corner() {
    // Full pointer path: an interior corner exposes a Fillet (circle) handle in the editing phase; dragging
    // it out marks that corner filleted (the rendered path then rounds it), and one undo restores it sharp.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0); // corner 1 — the only interior corner of the open 3-point line
    click(&mut t, 48.0, 48.0);
    assert!(t.line_finish_points(), "end creation → editing");
    let ov = t.line_overlay().unwrap();
    assert_eq!(ov.corner_gizmos.len(), 1, "one interior corner has gizmos");
    assert_eq!(ov.corner_gizmos[0].active, 0, "corner starts sharp");
    let fh = ov.corner_gizmos[0].fillet_handle; // grab target from the overlay
    // Drag RIGHT to grow the fillet (right/up increases, left/down shrinks).
    let target = [fh[0] + 30.0, fh[1]];
    t.on_canvas_pointer(cp(fh, PointerPhase::Down));
    t.on_canvas_pointer(cp(target, PointerPhase::Move));
    t.on_canvas_pointer(cp(target, PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().corner_gizmos[0].active,
        1,
        "the corner is now filleted"
    );
    assert!(t.undo_last());
    assert_eq!(
        t.line_overlay().unwrap().corner_gizmos[0].active,
        0,
        "undo restored the sharp corner"
    );
}

#[test]
fn line_fillet_persists_through_undo_redo_snapshot() {
    // The per-corner mod rides the unified undo snapshot (LineState.corner_mods): after filleting, an
    // unrelated edit + undo/redo must preserve the fillet.
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 48.0);
    assert!(t.line_finish_points());
    let fh = t.line_overlay().unwrap().corner_gizmos[0].fillet_handle;
    let target = [fh[0] + 30.0, fh[1]]; // drag right → grow the fillet
    t.on_canvas_pointer(cp(fh, PointerPhase::Down));
    t.on_canvas_pointer(cp(target, PointerPhase::Move));
    t.on_canvas_pointer(cp(target, PointerPhase::Up));
    assert_eq!(t.line_overlay().unwrap().corner_gizmos[0].active, 1);
    // Undo the fillet, then redo it — the snapshot must round-trip the corner mod.
    assert!(t.undo_last());
    assert_eq!(t.line_overlay().unwrap().corner_gizmos[0].active, 0);
    assert!(t.redo_last());
    assert_eq!(
        t.line_overlay().unwrap().corner_gizmos[0].active,
        1,
        "redo reinstated the fillet from the snapshot"
    );
}

#[test]
fn line_corner_drag_grows_right_shrinks_left() {
    // The chamfer handle: a rightward drag GROWS the chamfer, a leftward drag (picking up the current
    // amount) shrinks it back to sharp — the directional mapping (right/up +, left/down −).
    let mut t = line_tool();
    click(&mut t, 16.0, 16.0);
    click(&mut t, 48.0, 16.0);
    click(&mut t, 48.0, 48.0);
    assert!(t.line_finish_points());
    let ch = t.line_overlay().unwrap().corner_gizmos[0].chamfer_handle;
    t.on_canvas_pointer(cp(ch, PointerPhase::Down));
    t.on_canvas_pointer(cp([ch[0] + 30.0, ch[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([ch[0] + 30.0, ch[1]], PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().corner_gizmos[0].active,
        2,
        "a rightward drag grew the chamfer"
    );
    let ch2 = t.line_overlay().unwrap().corner_gizmos[0].chamfer_handle;
    t.on_canvas_pointer(cp(ch2, PointerPhase::Down));
    t.on_canvas_pointer(cp([ch2[0] - 80.0, ch2[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([ch2[0] - 80.0, ch2[1]], PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().corner_gizmos[0].active,
        0,
        "a leftward drag shrank it back to sharp"
    );
}

#[test]
fn line_dragging_a_point_snaps_to_another_points_column() {
    // Auto-snap (Shift off): dragging a point near another point's X aligns it to that column exactly; a
    // far Y is untouched. tol 8 → snap threshold 4.8 px.
    let mut t = line_tool();
    click(&mut t, 20.0, 20.0); // point 0
    click(&mut t, 50.0, 50.0); // point 1
    t.on_canvas_pointer(cp([50.0, 50.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([22.0, 62.0], PointerPhase::Move)); // x≈point0.x (Δ2), y far
    t.on_canvas_pointer(cp([22.0, 62.0], PointerPhase::Up));
    let ov = t.line_overlay().unwrap();
    assert_eq!(ov.points[1][0], 20.0, "x snapped to point 0's column");
    assert!(
        (ov.points[1][1] - 62.0).abs() < 1e-3,
        "y unchanged (no nearby row): {:?}",
        ov.points[1]
    );
}

#[test]
fn line_dragging_a_point_onto_another_snaps_right_on_top() {
    // Both axes within the threshold → the dragged point lands exactly on the other (the "drag one point
    // onto another" case).
    let mut t = line_tool();
    click(&mut t, 20.0, 20.0);
    click(&mut t, 50.0, 50.0);
    t.on_canvas_pointer(cp([50.0, 50.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([23.0, 17.0], PointerPhase::Move)); // near (20,20) on both axes
    t.on_canvas_pointer(cp([23.0, 17.0], PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().points[1],
        [20.0, 20.0],
        "landed on point 0"
    );
}

#[test]
fn line_shift_angle_snap_does_not_inhibit_point_snap() {
    // Bug fix (Enio): angle snap (Shift) must NOT disable point snap — both act together. With Shift ON,
    // dragging a point onto another still point-snaps it right on top (the join wins after the 15° snap).
    let mut t = line_tool();
    click(&mut t, 20.0, 20.0); // point 0
    click(&mut t, 60.0, 60.0); // point 1
    t.set_line_snap(true); // Shift armed — 15° angle snap active
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([22.0, 18.0], PointerPhase::Move)); // near point 0 on both axes
    t.on_canvas_pointer(cp([22.0, 18.0], PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().points[1],
        [20.0, 20.0],
        "point-snapped onto point 0 despite Shift (both snaps act)"
    );
}

#[test]
fn line_grid_snap_places_points_on_the_forwarded_grid_position() {
    // The shell forwards the grid-snapped image position via `set_grid_snap`; a placed/dragged point uses
    // it as the base (drawing-tool grid snap). Here the shell says "grid node (40,40)" for the pointer.
    let mut t = line_tool();
    click(&mut t, 8.0, 8.0); // point 0 (no grid forwarded → raw)
    // The shell resolves the pointer at (47,43) to grid node (40,40) and forwards it.
    t.set_grid_snap(Some([40.0, 40.0]));
    t.on_canvas_pointer(cp([47.0, 43.0], PointerPhase::Down)); // creates point 1 at the grid node
    t.on_canvas_pointer(cp([47.0, 43.0], PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().points[1],
        [40.0, 40.0],
        "point 1 snapped to the forwarded grid node"
    );
    // Grid off (None) → the next point is placed raw (chosen far from other points so point-snap is inert).
    t.set_grid_snap(None);
    t.on_canvas_pointer(cp([20.0, 58.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 58.0], PointerPhase::Up));
    assert_eq!(
        t.line_overlay().unwrap().points[2],
        [20.0, 58.0],
        "grid off → raw placement"
    );
}

#[test]
fn line_offset_slider_shifts_the_open_line_in_real_time() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // The Offset slider shifts the whole Line perpendicular (parallel polyline); changing it must re-fill
    // the open Line live (via refill_open_shape / appearance_sig). Draw a horizontal line along y=32, then
    // nudge Offset to 0.6 (+20px, perpendicular = up) and assert the stroke LEFT y=32 and now paints ~y=12.
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_shape_grab_tol_px(8.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // a horizontal line along y=32
    assert!(
        px(&t, 64, 32, 32)[0] < 200,
        "the line painted on y=32 at zero offset"
    );
    assert_eq!(
        px(&t, 64, 32, 12),
        [255, 255, 255, 255],
        "y=12 is white before offset"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6));
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "the stroke left the original y=32 line (restored white): {:?}",
        px(&t, 64, 32, 32)
    );
    assert!(
        px(&t, 64, 32, 12)[0] < 200,
        "the offset line now paints ~20px up at y=12: {:?}",
        px(&t, 64, 32, 12)
    );
}
