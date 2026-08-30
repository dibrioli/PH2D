//! **O editor de CURVA (e o Free Hand que deixa uma).** Âncoras e alças (simétrica, livre, vetorial),
//! o refill em tempo real quando um param do pincel muda, o offset vivo, o Apply/Apply&Keep/Delete, a
//! conversão de círculo e polígono em curva editável, o Simplify, e o que tudo isso vale em undo.

use super::*;

#[test]
fn free_hand_stabilizer_smooths_the_capture() {
    use ph2d_painter_brush::StrokeMethod;
    // Stabilize is ACTIVE for Free Hand: the lazy-mouse filter lags the cursor, so a high stabilizer
    // yields different (smoothed) control points than no stabilization on the SAME jittery path.
    let jitter = [
        [24.0, 36.0],
        [28.0, 28.0],
        [32.0, 38.0],
        [36.0, 27.0],
        [40.0, 37.0],
    ];
    let capture = |stab: f32| {
        let mut t = white_canvas(64, 6.0);
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.stabilizer = stab;
        t.on_canvas_pointer(cp([18.0, 32.0], PointerPhase::Down));
        for &p in &jitter {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
        t.curve_overlay().map(|o| o.points)
    };
    let raw = capture(0.0).expect("raw capture");
    let smoothed = capture(1.0).expect("stabilized capture");
    assert_ne!(
        raw, smoothed,
        "the stabilizer changes (smooths) the Free Hand capture"
    );
}

#[test]
fn apply_buttons_route_through_panel_click() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // The panel's Apply / Apply & Keep buttons forward as PanelEvent::Click — this exercises the FULL
    // wiring (handle_panel_event → route_brush_dab_event → commit), not just the verbs (Enio 2026-06-27).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(t.curve_overlay().is_some(), "a curve editor is open");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP));
    assert!(
        t.curve_overlay().is_some(),
        "Apply & Keep via Click bakes but keeps the curve"
    );
    assert!(
        px(&t, 64, 32, 26)[0] < 200,
        "the stroke was baked by the Click (probe on the arc's apex — the Arc bows up)"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_APPLY));
    assert!(
        t.curve_overlay().is_none(),
        "plain Apply via Click discards the curve"
    );
}

#[test]
fn brush_param_change_refills_open_curve_in_real_time() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // While a Curve editor is open, changing a brush param (here Size) must re-fill the pending stroke
    // immediately — not only when a gizmo handle is nudged (Enio 2026-06-27). Draw a thin horizontal
    // curve, then grow the brush via the panel Size slider and assert a pixel ABOVE the thin line (white
    // before) is now painted by the wider stroke.
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3-point curve along y=32
    assert!(t.curve_overlay().is_some(), "a curve editor is open");
    assert_eq!(
        px(&t, 64, 32, 17),
        [255, 255, 255, 255],
        "~8px above the arc's apex (y=24.8) is white before growing the brush"
    );
    // Grow the brush — routed in the match arm, which re-fills the open shape.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_SIZE_SLIDER,
        0.6,
    ));
    assert_ne!(
        px(&t, 64, 32, 17),
        [255, 255, 255, 255],
        "the wider brush re-filled the curve in real time (no gizmo nudge needed)"
    );
}

#[test]
fn reducing_strength_with_an_open_curve_does_not_erase_it() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // Regression (Enio 2026-06-27): with Accumulate off, Strength<1 caps each pixel via the per-stroke
    // `stroke_mask`. A fill (Curve) re-stamps the WHOLE stroke each re-fill, so the mask MUST reset each
    // time — else the 2nd re-fill sees the mask already at the cap and paints nothing, so reducing Strength
    // (which re-fills) erased the stroke. Drag the Strength slider down a few times with a curve open and
    // assert it stays painted.
    let mut t = white_canvas(64, 4.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // curve along y=32, full strength
    assert!(
        px(&t, 64, 32, 25)[0] < 200,
        "the curve painted at full strength (probe on the arc's apex)"
    );
    // Drag the slider down (each event re-fills). At 0.7 a fresh fill is clearly dark (~130); the stale-
    // mask bug instead left it white (~255, "erased"). Assert it stays clearly painted.
    for v in [0.9_f64, 0.7] {
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_BRUSH_STRENGTH_SLIDER,
            v,
        ));
    }
    assert!(
        px(&t, 64, 32, 25)[0] < 180,
        "reducing Strength must keep the curve painted (not erase to white): {:?}",
        px(&t, 64, 32, 25)
    );
}

#[test]
fn dragging_a_tangent_handle_reshapes_the_curve_and_mirrors_the_opposite() {
    use ph2d_painter_brush::StrokeMethod;
    // Gizmo (Enio 2026-06-27): the selected anchor exposes draggable Bézier tangent handles. Grabbing the
    // OUT handle (off the point) and pulling it must move that handle (not the anchor) and swing the IN
    // handle to stay aligned (collinear through the anchor) — the standard smooth-handle behaviour.
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.set_shape_grab_tol_px(4.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3-pt curve, midpoint (idx 1) selected
    let ov = t.curve_overlay().expect("curve open");
    assert_eq!(ov.selected, Some(1));
    let tan = ov
        .tangents
        .expect("the selected interior anchor exposes tangents");
    let out = tan.out_handle.expect("out handle present");
    let anchor = tan.anchor;
    assert!(
        (anchor[1] - 24.8).abs() < 0.05,
        "midpoint bows up to y=24.8 (the Arc's initial curvature)"
    );
    // Grab the out handle and pull it straight up.
    t.on_canvas_pointer(cp(out, PointerPhase::Down));
    let target = [out[0], out[1] - 12.0];
    t.on_canvas_pointer(cp(target, PointerPhase::Move));
    t.on_canvas_pointer(cp(target, PointerPhase::Up));
    let tan2 = t
        .curve_overlay()
        .unwrap()
        .tangents
        .expect("tangents still shown");
    let out2 = tan2.out_handle.unwrap();
    assert!(
        (out2[0] - target[0]).abs() < 0.6 && (out2[1] - target[1]).abs() < 0.6,
        "the out handle followed the drag: {out2:?}"
    );
    // Aligned mirror: the out pull went UP (−y from anchor) ⇒ the in handle swings DOWN (+y).
    let in2 = tan2.in_handle.unwrap();
    assert!(
        in2[1] - tan2.anchor[1] > 0.5,
        "the in handle mirrored downward: {in2:?}"
    );
    // The anchor itself did not move (we grabbed the handle, not the point).
    assert!(
        (tan2.anchor[0] - anchor[0]).abs() < 1e-3 && (tan2.anchor[1] - anchor[1]).abs() < 1e-3,
        "the anchor stayed put"
    );
}

#[test]
fn a_hand_edited_tangent_is_pinned_through_a_later_anchor_move() {
    use ph2d_painter_brush::StrokeMethod;
    // Once a tangent is hand-edited the curve is PINNED (no auto-resmooth), so a later anchor drag
    // rigid-translates the handles instead of recomputing flat chordal tangents — the artist's sculpted
    // curvature survives. Pull a tangent up, then nudge the anchor, and assert the vertical pull persists.
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.set_shape_grab_tol_px(4.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    let out = t
        .curve_overlay()
        .unwrap()
        .tangents
        .unwrap()
        .out_handle
        .unwrap();
    // Hand-edit the out tangent (pull up) → pins the handles.
    t.on_canvas_pointer(cp(out, PointerPhase::Down));
    let pulled = [out[0], out[1] - 12.0];
    t.on_canvas_pointer(cp(pulled, PointerPhase::Move));
    t.on_canvas_pointer(cp(pulled, PointerPhase::Up));
    // Now grab the midpoint anchor and nudge it; pinned ⇒ the out handle keeps its vertical offset.
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)); // hits anchor 1
    t.on_canvas_pointer(cp([32.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([32.0, 30.0], PointerPhase::Up));
    let tan = t.curve_overlay().unwrap().tangents.expect("still selected");
    let out_after = tan.out_handle.unwrap();
    // If the curve had auto-resmoothed (NOT pinned), the out handle would be flat (≈ anchor.y); pinned, it
    // keeps a clear vertical pull above the (now-moved) anchor.
    assert!(
        tan.anchor[1] - out_after[1] > 5.0,
        "the sculpted vertical tangent survived the anchor move (pinned): out={out_after:?} anchor={:?}",
        tan.anchor
    );
}

/// Open a 3-point Curve (along y=32) in edit mode with the midpoint (index 1) selected, grab tol set.
fn open_curve_midpoint_selected() -> PainterTool {
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.set_shape_grab_tol_px(4.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    t
}

/// **A shape is editable from its wrapped Tiling copy (Enio 2026-07-11).** With seamless Tiling the shape's
/// wash + editable overlay show in the neighbour tiles; grabbing a control point on a COPY must edit the
/// ORIGINAL. The shape-editor pointer folds into the sprite space on each tiled axis, so a grab in the
/// right tile (`x + w`) hits the in-sprite anchor and the drag moves it. Off-tiling ⇒ the raw coord.
#[test]
fn shape_editable_from_a_wrapped_tile_under_tiling() {
    let mut t = open_curve_midpoint_selected(); // endpoints (8,32) & (56,32), now editing
    t.paint.tiling = [true, false]; // wrap on X

    let has = |t: &PainterTool, x: f32, y: f32| {
        t.curve_overlay()
            .unwrap()
            .points
            .iter()
            .any(|p| (p[0] - x).abs() < 2.0 && (p[1] - y).abs() < 2.0)
    };
    assert!(has(&t, 56.0, 32.0), "the (56,32) endpoint exists");

    // Grab the (56,32) endpoint via its RIGHT-tile ghost at (56+64, 32) = (120,32) and drag +5 → the wrap
    // folds 120→56 (grab) and 125→61 (drag), so the ORIGINAL anchor moves to (61,32).
    t.on_canvas_pointer(cp([120.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([125.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([125.0, 32.0], PointerPhase::Up));

    assert!(
        has(&t, 61.0, 32.0),
        "grabbing the wrapped-tile ghost moved the original anchor to (61,32): {:?}",
        t.curve_overlay().unwrap().points
    );
    assert!(has(&t, 8.0, 32.0), "the other endpoint stayed put");
}

/// **An in-sprite grab drags CONTINUOUSLY past the seam — no wrap jump (Enio 2026-07-11).** The edit-in-tile
/// offset is fixed at the grab Down, so a handle grabbed inside the sprite keeps offset `0` and follows the
/// pointer BEYOND the sprite — instead of the per-sample `rem_euclid` that snapped it to the opposite edge
/// mid-drag (the Ellipse/Polygon "size jump" bug). The anchor lands at the raw target, not its wrapped twin.
#[test]
fn shape_in_sprite_grab_drags_past_the_seam_without_wrapping() {
    let mut t = open_curve_midpoint_selected(); // endpoints (8,32) & (56,32)
    t.paint.tiling = [true, false]; // wrap on X

    // Grab the (56,32) endpoint INSIDE the sprite (offset stays 0), then drag PAST the right edge to x=80.
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 32.0], PointerPhase::Up));

    let pts = t.curve_overlay().unwrap().points;
    assert!(
        pts.iter()
            .any(|p| (p[0] - 80.0).abs() < 3.0 && (p[1] - 32.0).abs() < 3.0),
        "the anchor followed the pointer beyond the sprite (x≈80): {pts:?}"
    );
    assert!(
        !pts.iter().any(|p| (p[0] - 16.0).abs() < 3.0),
        "the anchor must NOT wrap to x=16 (80 mod 64) — the old per-sample rem_euclid jump: {pts:?}"
    );
}

#[test]
fn handle_kind_menu_pick_updates_the_selected_point() {
    // The right-click menu's wire u8 (0=Free 1=Aligned 2=Vector 3=Auto) routes to the selected point and
    // the overlay reports the active kind (Enio 2026-06-27). Authored points start Auto (3).
    let mut t = open_curve_midpoint_selected();
    assert_eq!(
        t.curve_overlay().unwrap().selected_kind,
        Some(3),
        "authored point starts Auto"
    );
    for wire in [2u8, 0, 1, 3, 4] {
        assert!(t.set_curve_handle_kind(wire), "kind {wire} applied");
        assert_eq!(t.curve_overlay().unwrap().selected_kind, Some(wire));
    }
    assert!(
        !t.set_curve_handle_kind(9),
        "an out-of-range wire is rejected"
    );
}

#[test]
fn symmetric_handle_mirrors_the_opposite_with_equal_length() {
    // Symmetric (wire 4): dragging one tangent reflects the other EXACTLY (collinear + equal length),
    // unlike Aligned which keeps the opposite's own length.
    let mut t = open_curve_midpoint_selected();
    assert!(t.set_curve_handle_kind(4)); // Symmetric
    let out = t
        .curve_overlay()
        .unwrap()
        .tangents
        .expect("seeded tangents")
        .out_handle
        .expect("out present");
    let target = [out[0] + 6.0, out[1] - 14.0]; // pull out up + sideways
    t.on_canvas_pointer(cp(out, PointerPhase::Down));
    t.on_canvas_pointer(cp(target, PointerPhase::Move));
    t.on_canvas_pointer(cp(target, PointerPhase::Up));
    let tan2 = t.curve_overlay().unwrap().tangents.unwrap();
    let (out2, in2, a) = (
        tan2.out_handle.unwrap(),
        tan2.in_handle.unwrap(),
        tan2.anchor,
    );
    // in must be the exact reflection of out through the anchor: in = 2*anchor − out.
    assert!(
        (in2[0] - (2.0 * a[0] - out2[0])).abs() < 0.5
            && (in2[1] - (2.0 * a[1] - out2[1])).abs() < 0.5,
        "Symmetric: in {in2:?} is the exact mirror of out {out2:?} about {a:?}"
    );
}

#[test]
fn free_handle_does_not_mirror_the_opposite() {
    // A Free point's tangents are independent: dragging one must NOT swing the other (contrast the Aligned
    // mirror). Switch the midpoint to Free (seeds smooth handles), then pull its out handle up.
    let mut t = open_curve_midpoint_selected();
    assert!(t.set_curve_handle_kind(0)); // Free
    let tan = t
        .curve_overlay()
        .unwrap()
        .tangents
        .expect("seeded tangents");
    let out = tan.out_handle.expect("out present");
    let in_before = tan.in_handle.expect("in present");
    t.on_canvas_pointer(cp(out, PointerPhase::Down));
    t.on_canvas_pointer(cp([out[0], out[1] - 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([out[0], out[1] - 12.0], PointerPhase::Up));
    let tan2 = t.curve_overlay().unwrap().tangents.unwrap();
    assert!(
        (tan2.in_handle.unwrap()[1] - in_before[1]).abs() < 1e-3,
        "Free: the in handle stayed put (no mirror): {:?} vs {:?}",
        tan2.in_handle,
        in_before
    );
    // And it stayed Free (a Free tangent edit does not convert to Aligned).
    assert_eq!(t.curve_overlay().unwrap().selected_kind, Some(0));
}

#[test]
fn vector_handles_point_at_the_neighbours_after_a_move() {
    // Move the midpoint off-axis, then make it Vector: its tangents must point 1/3 toward the two
    // neighbour anchors (a polyline-like joint), regardless of the prior smooth handles.
    let mut t = open_curve_midpoint_selected();
    t.on_canvas_pointer(cp([32.0, 24.8], PointerPhase::Down)); // the bowed midpoint (Arc)
    t.on_canvas_pointer(cp([32.0, 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([32.0, 12.0], PointerPhase::Up));
    assert!(t.set_curve_handle_kind(2)); // Vector
    let tan = t
        .curve_overlay()
        .unwrap()
        .tangents
        .expect("vector tangents shown");
    // out toward (56,32): (32,12)+((56-32)/3,(32-12)/3) = (40, 18.667); in toward (8,32): (24, 18.667).
    let out = tan.out_handle.unwrap();
    let in_h = tan.in_handle.unwrap();
    assert!(
        (out[0] - 40.0).abs() < 0.5 && (out[1] - 18.667).abs() < 0.5,
        "out={out:?}"
    );
    assert!(
        (in_h[0] - 24.0).abs() < 0.5 && (in_h[1] - 18.667).abs() < 0.5,
        "in={in_h:?}"
    );
}

#[test]
fn offset_slider_shifts_the_open_curve_in_real_time() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // The Offset slider shifts the whole curve perpendicular (control geometry); changing it must re-fill
    // the open shape live (folded into appearance_sig). Draw a horizontal curve along y=32, then nudge
    // Offset to 0.6 (+20px, perpendicular = up) and assert the stroke LEFT y=32 and now paints at y≈12.
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // curve along y=32
    assert!(
        px(&t, 64, 32, 25)[0] < 200,
        "the arc painted through its apex (32,24.8) at zero offset"
    );
    assert_eq!(
        px(&t, 64, 32, 5),
        [255, 255, 255, 255],
        "y=5 is white before offset"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6)); // +20px perpendicular
    assert_eq!(
        px(&t, 64, 32, 25),
        [255, 255, 255, 255],
        "the stroke left the original apex (restored white): {:?}",
        px(&t, 64, 32, 25)
    );
    assert!(
        px(&t, 64, 32, 5)[0] < 200,
        "the offset stroke now paints ~20px up at the apex (32,5): {:?}",
        px(&t, 64, 32, 5)
    );
}

#[test]
fn offset_apply_keep_absorbs_the_offset_keeping_the_drawing_put() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // Under DRAWING-ONLY offset (Enio 2026-07-05): the guide LINE stays on the pristine curve (y=32) while the
    // painted DRAWING is offset up (apex ~y=5). Apply & Keep folds the slider into the accumulator (slider →
    // 0.5) WITHOUT moving the geometry — so the painted drawing stays up and the guide stays pristine (no jump).
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6)); // +20px up
    // The guide LINE stays on the pristine curve at y≈32; only the DRAWING moved up.
    let guide_y = t.curve_overlay().unwrap().spine[0][1];
    assert!(
        (guide_y - 32.0).abs() < 2.0,
        "the guide line stays on the pristine curve: {guide_y}"
    );
    assert!(
        px(&t, 64, 32, 5)[0] < 200,
        "the painted drawing is offset up (apex ~y=5): {:?}",
        px(&t, 64, 32, 5)
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP));
    assert!(
        (t.brush_settings().offset - 0.5).abs() < 1e-4,
        "the Offset slider reset to centre"
    );
    // The guide is still pristine and the drawing is still up — Apply & Keep absorbed the offset, no jump.
    let kept_guide_y = t.curve_overlay().expect("kept open").spine[0][1];
    assert!(
        (kept_guide_y - 32.0).abs() < 2.0,
        "the guide stayed pristine after Apply & Keep: {kept_guide_y}"
    );
    assert!(
        px(&t, 64, 32, 5)[0] < 200,
        "the painted drawing stayed up after Apply & Keep: {:?}",
        px(&t, 64, 32, 5)
    );
}

#[test]
fn color_ramp_edits_change_the_appearance_signature() {
    // Regression (Enio 2026-06-27): the real-time re-fill trigger compared only the BrushSpec, but the
    // Colour-Ramp enable / B&W / stop edits live OUTSIDE it (in PaintState) — so toggling the ramp didn't
    // re-fill the open curve until a point moved. `appearance_sig` now folds the ramp/texture/shape state
    // in, so any of these changes it → the handler re-fills. Assert the sig actually moves.
    let mut t = white_canvas(64, 4.0);
    let s0 = t.appearance_sig();
    t.toggle_texture_ramp_enabled();
    assert!(
        t.appearance_sig() != s0,
        "enabling the Color Ramp must change the appearance sig"
    );
    let s1 = t.appearance_sig();
    t.ramp_add_stop();
    assert!(
        t.appearance_sig() != s1,
        "adding a ramp stop must change the appearance sig"
    );
    let s2 = t.appearance_sig();
    t.toggle_texture_ramp_bw();
    assert!(
        t.appearance_sig() != s2,
        "the ramp B&W toggle must change the appearance sig"
    );
}

#[test]
fn edit_button_converts_circle_into_an_editable_curve() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // The Edit (E) button turns an open Ellipse into an editable Bézier curve: the circle editor closes, a
    // curve editor opens (with the closing anchor so it reads closed), and the method switches to Curve so
    // pointers route to the curve editor (Enio 2026-06-27).
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(t.ellipse_overlay().is_some(), "a circle editor is open");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_EDIT));
    assert!(t.ellipse_overlay().is_none(), "the circle editor closed");
    let ov = t.curve_overlay().expect("a curve editor opened");
    assert_eq!(
        t.brush_settings().stroke_method,
        StrokeMethod::Arc.to_u8(),
        "method is now Curve"
    );
    // Convert-to-Curve is the DENSE direction (Enio 2026-07-05): the exact circle densifies to many
    // manipulation anchors (~perimeter / 16 px), every one still ON the circle.
    assert!(
        ov.points.len() >= 8,
        "circle converts to MANY anchors (multipoint): {}",
        ov.points.len()
    );
    for p in &ov.points {
        let r = ((p[0] - 32.0).powi(2) + (p[1] - 32.0).powi(2)).sqrt();
        assert!((r - 20.0).abs() < 0.5, "anchor stays ON the circle: r={r}");
    }
    assert_ne!(
        *ov.points.first().unwrap(),
        *ov.points.last().unwrap(),
        "no duplicate seam anchor — the anchors are distinct"
    );
    // The closed loop shows in the SPINE: it returns to (near) the start.
    let (s0, sl) = (*ov.spine.first().unwrap(), *ov.spine.last().unwrap());
    assert!(
        (s0[0] - sl[0]).abs() < 0.5 && (s0[1] - sl[1]).abs() < 0.5,
        "the spine closes the loop back to the start: {s0:?} vs {sl:?}"
    );
}

#[test]
fn edit_with_a_live_offset_converts_the_pristine_circle_keeping_the_offset() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // Drawing-only (Enio 2026-07-05): Edit (convert) with a live Offset produces the PRISTINE circle as a
    // curve (anchors at radius 20, NOT the offset radius) and the offset PERSISTS as a drawing transform (the
    // slider is NOT reset). So no offset artifact can land in the control points, and there is no double
    // offset (supersedes the old bake-the-offset behavior).
    let mut t = white_canvas(96, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([68.0, 48.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([68.0, 48.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6)); // +20px
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_EDIT));
    assert!(
        (t.brush_settings().offset - 0.6).abs() < 1e-4,
        "the offset PERSISTS through convert (drawing-only) — slider not reset: {}",
        t.brush_settings().offset
    );
    let ov = t.curve_overlay().expect("a curve opened");
    assert!(
        ov.points.len() >= 8,
        "dense convert: {} anchors",
        ov.points.len()
    );
    // Every anchor sits at the PRISTINE radius ~20 (the offset was NOT baked in) — exactly on the circle.
    for p in &ov.points {
        let r = ((p[0] - 48.0).powi(2) + (p[1] - 48.0).powi(2)).sqrt();
        assert!(
            (r - 20.0).abs() < 2.0,
            "anchor sits at the pristine radius ~20, not the offset radius: r={r}"
        );
    }
}

#[test]
fn edit_button_converts_polygon_into_a_sharp_editable_curve() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Polygon;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(t.polygon_overlay().is_some(), "a polygon editor is open");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_EDIT));
    assert!(t.polygon_overlay().is_none(), "the polygon editor closed");
    let ov = t.curve_overlay().expect("a curve editor opened");
    assert_ne!(
        *ov.points.first().unwrap(),
        *ov.points.last().unwrap(),
        "no duplicate seam vertex — the anchors are distinct"
    );
    assert!(ov.points.len() >= 3, "polygon vertices became anchors");
    let (s0, sl) = (*ov.spine.first().unwrap(), *ov.spine.last().unwrap());
    assert!(
        (s0[0] - sl[0]).abs() < 0.5 && (s0[1] - sl[1]).abs() < 0.5,
        "the spine closes the loop back to the start: {s0:?} vs {sl:?}"
    );
}

#[test]
fn delete_button_drops_the_open_shape_without_baking() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // The trash button cancels the open shape editor WITHOUT baking it — the canvas stays pristine.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(t.curve_overlay().is_some(), "a curve editor is open");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_DELETE));
    assert!(t.curve_overlay().is_none(), "Delete drops the editor");
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "Delete did NOT bake — canvas pristine"
    );
}

#[test]
fn apply_keep_bakes_but_keeps_the_editable_curve() {
    use ph2d_painter_brush::StrokeMethod;
    // "Apply & Keep" bakes the pending stroke yet keeps the editable curve (for re-apply / reshape);
    // plain "Apply" bakes and discards it.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up)); // released → edit mode, control points
    assert!(t.curve_overlay().is_some(), "a curve editor is open");
    assert!(t.commit_open_shape_keep(), "Apply & Keep ran");
    assert!(
        t.curve_overlay().is_some(),
        "the editable curve persists after Apply & Keep"
    );
    assert!(px(&t, 64, 32, 26)[0] < 200, "the stroke was baked");
    assert!(t.commit_open_shape(), "Apply ran");
    assert!(
        t.curve_overlay().is_none(),
        "the curve is discarded after plain Apply"
    );
}

#[test]
fn free_hand_paints_and_leaves_an_editable_curve() {
    use ph2d_painter_brush::StrokeMethod;
    // Free Hand: a freehand drag paints the stroke AND, on release, leaves an editable curve (control
    // points + spine) reusing the Curve editor. The captured path simplifies to >= 2 control points.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::FreeHand;
    t.on_canvas_pointer(cp([10.0, 32.0], PointerPhase::Down));
    for &p in &[
        [18.0, 32.0],
        [26.0, 32.0],
        [34.0, 34.0],
        [42.0, 38.0],
        [50.0, 42.0],
    ] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([54.0, 44.0], PointerPhase::Up));
    let ov = t
        .curve_overlay()
        .expect("Free Hand leaves an editable curve overlay on release");
    assert!(
        ov.points.len() >= 2,
        "the captured path simplified to control points: {}",
        ov.points.len()
    );
    assert!(!ov.spine.is_empty(), "the editable curve has a spine");
    assert!(
        px(&t, 64, 18, 32)[0] < 200,
        "the freehand stroke painted along the path: {:?}",
        px(&t, 64, 18, 32)
    );
}

/// Curve: a press-drag-release of the initial line yields a 3-point editable curve (overlay shows
/// start / midpoint / end), the midpoint pre-selected — and the line paints along it (no trail).
#[test]
fn curve_draw_creates_three_points_and_paints_the_line() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    assert!(t.curve_overlay().is_none(), "no chrome before drawing");
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    // Durante o arrasto a curva publica SÓ o spine: as âncoras e o gizmo aparecem na soltura (nenhum
    // Down as alcança antes disso). Era `is_none()` — mesma intenção, mecanismo novo.
    {
        let ov = t.curve_overlay().expect("o spine existe durante o arrasto");
        assert!(
            ov.points.is_empty() && ov.transform_gizmo.is_none(),
            "still drawing — chrome appears on release"
        );
        assert!(ov.spine.len() >= 2, "…e a linha puxada e desenhada");
    }
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));

    let ov = t
        .curve_overlay()
        .expect("editing after the line is released");
    assert_eq!(ov.points.len(), 3, "start + midpoint + end");
    assert_eq!(ov.points[0], [8.0, 32.0]);
    assert_eq!(ov.points[2], [56.0, 32.0]);
    // The Arc's initial bow: the midpoint sits ABOVE the chord by ARC_BOW × chord (48·0.15 = 7.2), so a
    // fresh Arc reads as an arc, not a straight line (Enio 2026-07-04).
    assert!(
        (ov.points[1][0] - 32.0).abs() < 1e-3 && (ov.points[1][1] - 24.8).abs() < 0.05,
        "midpoint bows perpendicular to the chord: {:?}",
        ov.points[1]
    );
    assert_eq!(
        ov.selected,
        Some(1),
        "midpoint pre-selected (ready to bend)"
    );
    // The arc paints through its bowed apex; the chord midpoint (32,32) stays white.
    assert_eq!(
        px(&t, 64, 32, 25),
        [0, 0, 0, 255],
        "the arc paints through the bowed midpoint"
    );
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "the chord midpoint is NOT painted — the shape is an arc, not a line"
    );
}

/// Dragging the selected midpoint bends the curve OFF the chord — pixels appear above the original
/// straight line. Esc then reverts every painted pixel (nothing was committed).
#[test]
fn curve_bend_then_cancel_reverts_all_pixels() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    // Draw the base line at y=40, then grab the midpoint (~[32,40]) and drag it up to y=12.
    t.on_canvas_pointer(cp([8.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 40.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([32.0, 40.0], PointerPhase::Down)); // grab midpoint
    t.on_canvas_pointer(cp([32.0, 12.0], PointerPhase::Move)); // bend up
    t.on_canvas_pointer(cp([32.0, 12.0], PointerPhase::Up));

    assert_eq!(
        px(&t, 64, 32, 12),
        [0, 0, 0, 255],
        "the curve bows up to the dragged midpoint"
    );
    // Esc reverts the whole preview to the pristine white canvas.
    assert!(t.curve_cancel(), "a session was open");
    assert!(t.curve_overlay().is_none(), "session gone");
    for (x, y) in [(8u32, 40u32), (32, 40), (56, 40), (32, 12)] {
        assert_eq!(
            px(&t, 64, x, y),
            [255, 255, 255, 255],
            "cancel reverted ({x},{y})"
        );
    }
}

/// Clicking empty space adds a control point (and grabs it); Delete removes the selected point but
/// never drops below two; Enter commits (the painted curve survives + is one undo step).
#[test]
fn curve_add_delete_and_commit() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3 points now
    // Click a 4th point ON the curve (empty space never adds a point — Enio 2026-07-04): take a spine
    // sample away from every anchor and click it.
    let spine_click = {
        let ov = t.curve_overlay().unwrap();
        ov.spine[ov.spine.len() * 3 / 4]
    };
    t.on_canvas_pointer(cp(spine_click, PointerPhase::Down));
    t.on_canvas_pointer(cp(spine_click, PointerPhase::Up));
    let ov = t.curve_overlay().unwrap();
    assert_eq!(
        ov.points.len(),
        4,
        "a point was added by the on-curve click"
    );
    let sel = ov.selected.unwrap();

    // Delete the selected point → back to 3.
    assert!(t.curve_delete_selected());
    assert_eq!(t.curve_overlay().unwrap().points.len(), 3);
    assert_eq!(t.curve_overlay().unwrap().selected, Some(sel.min(2)));

    // Floor at 2: select an endpoint, delete twice — the second is refused.
    // (selected is some valid index; delete down to 2 then refuse.)
    assert!(t.curve_delete_selected(), "3 → 2 allowed");
    assert!(!t.curve_delete_selected(), "2 is the floor — refused");
    assert_eq!(t.curve_overlay().unwrap().points.len(), 2);

    // Enter commits: the painted curve stays + the session closes + it is one undo step.
    assert!(
        px(&t, 64, 8, 32) != [255, 255, 255, 255],
        "something is painted pre-commit (the endpoint anchor stays put)"
    );
    assert!(t.curve_commit());
    assert!(t.curve_overlay().is_none(), "committed → no session");
    let painted = px(&t, 64, 8, 32);
    assert_eq!(painted, [0, 0, 0, 255], "committed dabs survive");
    // Unified timeline: the FIRST undo un-bakes (reopens the shape over the pre-bake pixels); undoing every
    // step then walks back through the edits + creation to the pristine canvas.
    assert!(t.undo_last());
    assert!(
        t.curve_overlay().is_some(),
        "undo of Apply reopens the curve (un-bake)"
    );
    while t.undo_last() {}
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "undoing every step reaches the pristine canvas"
    );
    assert!(t.curve_overlay().is_none(), "fully undone → no session");
}

/// Switching the stroke method away from Curve mid-session BAKES it (applies the shape), never erases it
/// (Enio 2026-07-03): the session closes but the painted preview stays on the canvas.
#[test]
fn curve_baked_when_switching_method_away() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    t.on_canvas_pointer(cp([8.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 20.0], PointerPhase::Up));
    assert!(t.curve_overlay().is_some());
    let baked = px(&t, 64, 32, 13); // the live preview pixel on the arc's apex (bow: 20−7.2)
    assert_ne!(baked, [255, 255, 255, 255], "the curve painted a preview");
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.curve_overlay().is_none(),
        "leaving Curve closed the session (baked, not still open)"
    );
    assert_eq!(
        px(&t, 64, 32, 13),
        baked,
        "the shape was APPLIED (baked), not erased"
    );
}

/// The grab tolerance is honoured: a Down within the forwarded radius grabs the nearest point; a Down
/// ON the curve (away from anchors) inserts; a Down in EMPTY SPACE never creates a point — it parks the
/// curve and starts a new shape (Enio 2026-07-04).
#[test]
fn curve_grab_tolerance_grabs_near_inserts_on_curve_never_in_space() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.set_shape_grab_tol_px(5.0);

    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // anchors: (8,32) / (32,24.8) / (56,32)
    assert_eq!(t.curve_overlay().unwrap().points.len(), 3);

    // Down 3 px from the (bowed) midpoint (within tol 5) → grabs it, no new point.
    t.on_canvas_pointer(cp([32.0, 27.8], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 27.8], PointerPhase::Up));
    assert_eq!(
        t.curve_overlay().unwrap().points.len(),
        3,
        "near press grabbed, didn't add"
    );

    // Down ON the spine, away from every anchor → inserts a 4th (subdivides on the curve).
    let spine_click = {
        let ov = t.curve_overlay().unwrap();
        ov.spine[ov.spine.len() * 3 / 4]
    };
    t.on_canvas_pointer(cp(spine_click, PointerPhase::Down));
    t.on_canvas_pointer(cp(spine_click, PointerPhase::Up));
    assert_eq!(
        t.curve_overlay().unwrap().points.len(),
        4,
        "an on-curve press added a point"
    );

    // Down in EMPTY space (far from the spine) → NO point is created; the multi-shape router parks this
    // curve and begins a fresh draw (no overlay until its release).
    t.on_canvas_pointer(cp([20.0, 52.0], PointerPhase::Down));
    // A curva anterior foi PARQUEADA e um desenho novo começou: o overlay agora é o da figura NOVA
    // (fase de desenho ⇒ sem âncoras), e não os 4 pontos da que ficou para trás.
    assert!(
        t.curve_overlay()
            .is_none_or(|ov| ov.points.is_empty() && ov.transform_gizmo.is_none()),
        "empty-space press starts a NEW shape draw (nothing added to the parked curve)"
    );
    t.on_canvas_pointer(cp([20.0, 52.0], PointerPhase::Up));
}

/// Undo while the curve is being authored (points visible) COMMITS it first (applies the curve,
/// clears the points); the NEXT undo removes the committed stroke. Regression for "the drawing
/// vanished but the control points stayed" — undo must not strand the points over a reverted canvas.
#[test]
fn curve_undo_walks_edits_then_undoes_the_creation() {
    // Undo must NOT Apply an open curve (Enio 2026-06-27): while authoring it walks the per-session edit
    // history, and once that's exhausted the next undo removes the curve (undoes the creation) — never bakes.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Arc;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;
    t.set_shape_grab_tol_px(4.0);

    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3 points, painted, editing

    // Move the midpoint, then undo — the edit reverts and the curve STAYS open (never applied).
    let before = t.curve_overlay().unwrap().points;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([32.0, 12.0], PointerPhase::Up));
    assert_ne!(
        t.curve_overlay().unwrap().points,
        before,
        "the midpoint moved"
    );
    assert!(t.undo_last(), "the move is undone");
    assert_eq!(
        t.curve_overlay().unwrap().points,
        before,
        "reverted to pre-move"
    );
    assert!(
        t.curve_overlay().is_some(),
        "still open — undo never applied it"
    );

    // No edits left → the next undo undoes the CREATION: the curve is gone, the canvas reverts to white.
    assert!(t.undo_last(), "undo removes the just-created curve");
    assert!(
        t.curve_overlay().is_none(),
        "creation undone — the curve is gone (not applied)"
    );
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "the painted preview reverted to the white canvas"
    );
}

#[test]
fn a_curve_point_move_is_undoable_and_redoable() {
    // Editing a curve weaves into the paint Ctrl+Z: grab the selected midpoint, drag it, then undo/redo the
    // move step-by-step (Enio 2026-06-27).
    let mut t = open_curve_midpoint_selected();
    let before = t.curve_overlay().unwrap().points;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 10.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([32.0, 10.0], PointerPhase::Up));
    let moved = t.curve_overlay().unwrap().points;
    assert_ne!(moved, before, "the midpoint moved");
    assert!(t.can_undo(), "the curve edit is undoable");
    assert!(t.undo_last(), "undo the move");
    assert_eq!(
        t.curve_overlay().unwrap().points,
        before,
        "reverted to pre-move"
    );
    assert!(t.redo_last(), "redo the move");
    assert_eq!(
        t.curve_overlay().unwrap().points,
        moved,
        "re-applied the move"
    );
}

#[test]
fn selecting_a_curve_point_is_not_an_undo_step() {
    // A pure selection click (down + up, no drag) must not push an undo entry. The drawn curve already has
    // ONE entry (its creation), so the proof is: a single undo jumps straight to the creation (the shape
    // closes) rather than first reverting a phantom select step.
    let mut t = open_curve_midpoint_selected();
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    assert!(t.undo_last());
    assert!(
        t.curve_overlay().is_none(),
        "one undo removed the creation — the select was not its own step"
    );
    assert!(!t.can_undo(), "nothing remains after the creation undo");
}

#[test]
fn simplify_is_hidden_until_a_point_is_added_on_a_plain_curve() {
    // A freshly drawn Curve exposes no Simplify button until the user inserts a point (Enio 2026-06-27);
    // adding one (click on the curve away from existing points) unlocks it.
    let mut t = open_curve_midpoint_selected();
    assert!(
        !t.brush_settings().can_simplify,
        "no Simplify before a point is added"
    );
    // Select an endpoint first so the midpoint's tangent handles don't intercept the insert click.
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    let n0 = t.curve_overlay().unwrap().points.len();
    let spine_click = {
        let ov = t.curve_overlay().unwrap();
        ov.spine[ov.spine.len() * 3 / 4] // ON the curve, away from any anchor
    };
    t.on_canvas_pointer(cp(spine_click, PointerPhase::Down));
    t.on_canvas_pointer(cp(spine_click, PointerPhase::Up));
    assert_eq!(
        t.curve_overlay().unwrap().points.len(),
        n0 + 1,
        "a point was inserted"
    );
    assert!(
        t.brush_settings().can_simplify,
        "adding a point unlocks Simplify"
    );
}

/// A CONVERTED curve (Ellipse/Polygon → dense closed Bézier) exposes **Simplify** immediately — it is closed
/// and dense, exactly what Simplify reduces; the button used to stay hidden until a point was added (Enio
/// 2026-07-05: "onde está o botão simplify curve?").
#[test]
fn simplify_is_available_on_a_converted_curve() {
    let mut t = white_canvas(96, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Up));
    assert!(t.convert_open_shape_to_curve(), "converted to a curve");
    assert!(
        t.brush_settings().can_simplify,
        "Simplify is available on a converted (closed dense) curve, no point-add needed"
    );
}

#[test]
fn curve_apply_and_keep_keeps_the_exact_curve_and_recentres_the_slider() {
    // Apply & Keep must NOT move or alter the curve (Enio 2026-06-28): the live offset is folded into the
    // accumulator and the slider re-centred, so the displayed curve is byte-identical and only the slider
    // changes — letting the user keep offsetting in the same direction.
    let mut t = open_curve_midpoint_selected();
    t.set_brush_offset(0.6); // +20px offset
    let before = t.curve_overlay().unwrap().points;
    assert!(t.curve_commit_keep());
    let after = t.curve_overlay().unwrap().points;
    assert_eq!(
        before, after,
        "Apply & Keep did not move/alter the displayed curve"
    );
    assert!(
        (t.brush_settings().offset - 0.5).abs() < 1e-4,
        "the slider re-centred to 0.5"
    );
}

#[test]
fn an_offset_change_is_undoable_on_an_open_curve() {
    // The Offset slider is woven into curve undo (Enio 2026-06-27): changing it then undoing reverts it.
    let mut t = open_curve_midpoint_selected();
    let off0 = t.brush_settings().offset;
    t.set_brush_offset(0.8);
    assert!(
        (t.brush_settings().offset - 0.8).abs() < 1e-4,
        "offset applied"
    );
    assert!(t.undo_last(), "the offset change is undone");
    assert!(
        (t.brush_settings().offset - off0).abs() < 1e-4,
        "offset reverted by undo"
    );
}
