//! **Os editores de ELIPSE e de POLÍGONO.** As duas formas paramétricas de alças gêmeas: desenhar
//! do centro, a alça de eixo, a de rotação, a de centro, a de número de lados — e o commit, o cancel, o
//! assado ao trocar de método e o que o undo desfaz. Aqui também moram as duas sequências que só se
//! escrevem sobre uma forma paramétrica ABERTA — o Apply & Keep que dobra o offset no acumulador e a
//! ordem do undo entre a forma e a história de pintura —, porque é a fixtura do círculo que as arma.

use super::*;

/// A `PainterTool` set to the Ellipse method on a 128² white canvas, with a known grab tolerance so
/// the handle positions are predictable.
fn circle_tool() -> PainterTool {
    let mut t = white_canvas(128, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.set_shape_grab_tol_px(6.0); // gap = 6 * 3 = 18 px below the rotate handle
    t
}

/// Draw a circle centre (cx,cy) radius r (centre-out drag) and leave it in edit mode.
fn draw_circle(t: &mut PainterTool, cx: f32, cy: f32, r: f32) {
    t.on_canvas_pointer(cp([cx, cy], PointerPhase::Down));
    t.on_canvas_pointer(cp([cx + r, cy], PointerPhase::Move));
    t.on_canvas_pointer(cp([cx + r, cy], PointerPhase::Up));
}

#[test]
fn interleaved_shape_edits_and_bakes_undo_in_reverse_order() {
    // The headline of the unified timeline (Enio 2026-06-28): create → edit → Apply&Keep → edit → Apply&Keep,
    // then undo walks back bake → edit → bake → edit → creation, ONE sequence regardless of step kind. Track
    // the ellipse's rx (right-handle distance from centre) — deterministic, no pixel flakiness.
    let mut t = circle_tool();
    let rx = |t: &PainterTool| -> f32 {
        let o = t.ellipse_overlay().expect("ring open");
        (o.handles[0][0] - o.handles[5][0]).abs()
    };
    draw_circle(&mut t, 64.0, 64.0, 20.0); // entry 1: creation, rx = 20
    assert!((rx(&t) - 20.0).abs() < 0.5, "created at rx=20");
    // Edit A: drag the right axis handle 84 → 94 (rx 20 → 30).
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Up)); // entry 2: edit rx 20 → 30
    assert!((rx(&t) - 30.0).abs() < 0.5, "edit A grew rx to 30");
    assert!(t.ellipse_commit_keep()); // entry 3: bake (editor kept open)
    // Edit B: drag the right handle 94 → 104 (rx 30 → 40).
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Up)); // entry 4: edit rx 30 → 40
    assert!((rx(&t) - 40.0).abs() < 0.5, "edit B grew rx to 40");
    assert!(t.ellipse_commit_keep()); // entry 5: bake

    assert!(t.undo_last()); // un-bake 5 — editor stays at rx=40
    assert!((rx(&t) - 40.0).abs() < 0.5, "un-bake keeps rx=40");
    assert!(t.undo_last()); // undo edit B
    assert!((rx(&t) - 30.0).abs() < 0.5, "edit B undone → rx=30");
    assert!(t.undo_last()); // un-bake 3
    assert!((rx(&t) - 30.0).abs() < 0.5, "un-bake keeps rx=30");
    assert!(t.undo_last()); // undo edit A
    assert!((rx(&t) - 20.0).abs() < 0.5, "edit A undone → rx=20");
    assert!(t.undo_last()); // undo creation
    assert!(
        t.ellipse_overlay().is_none(),
        "creation undone last → no ring"
    );
}

#[test]
fn circle_draw_creates_an_editable_ellipse_outline() {
    let mut t = circle_tool();
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    // ⚠️ O overlay EXISTE desde o 1º pixel do arrasto (o contorno é a única coisa na tela sob o gesto
    // rascunhado — `shape_draft`); quem diz que as alças ainda não valem é `editing`. A asserção antiga
    // era `is_none()`: a INTENÇÃO era a mesma, o MECANISMO é que mudou.
    assert!(
        !t.ellipse_overlay()
            .expect("o contorno existe ja no Down")
            .editing,
        "no handles while drawing"
    );
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Up));

    let ov = t.ellipse_overlay().expect("editing after release");
    assert!(ov.perimeter.len() >= 16, "perimeter is a dense polyline");
    // right handle at (84,64), centre at (64,64).
    assert!(
        (ov.handles[0][0] - 84.0).abs() < 0.5 && (ov.handles[0][1] - 64.0).abs() < 0.5,
        "right handle at the rim: {:?}",
        ov.handles[0]
    );
    assert_eq!(ov.handles[5], [64.0, 64.0], "centre handle");
    // The OUTLINE is painted (rim black), the centre is empty (it's a ring, not a disc).
    assert_eq!(px(&t, 128, 84, 64), [0, 0, 0, 255], "rim painted");
    assert_eq!(
        px(&t, 128, 64, 64),
        [255, 255, 255, 255],
        "centre empty (outline only)"
    );
}

#[test]
fn circle_axis_handle_resizes_one_axis_into_an_ellipse() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    // Grab the right handle (84,64) and drag it out to rx = 30.
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Up));
    let ov = t.ellipse_overlay().unwrap();
    assert!(
        (ov.handles[0][0] - 94.0).abs() < 0.5,
        "rx grew: {:?}",
        ov.handles[0]
    );
    // The top handle is unchanged (ry stays 20) → it's now an ellipse, not a circle.
    assert!(
        (ov.handles[1][1] - 84.0).abs() < 0.5,
        "ry unchanged: {:?}",
        ov.handles[1]
    );
}

#[test]
fn circle_rotate_handle_spins_the_ellipse() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    let rot = t.ellipse_overlay().unwrap().handles[4];
    // rotate handle sits gap (18) above the top (64, 64+20) → (64, 102).
    assert!(
        (rot[0] - 64.0).abs() < 0.5 && (rot[1] - 102.0).abs() < 0.5,
        "rotate handle above the top: {rot:?}"
    );
    // Drag the rotate handle to the RIGHT of the centre → local up becomes +x, so the x-axis (right
    // handle) rotates to point DOWN: right handle = centre + (0,-1)*rx = (64, 44).
    t.on_canvas_pointer(cp([rot[0], rot[1]], PointerPhase::Down));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Up));
    let ov = t.ellipse_overlay().unwrap();
    assert!(
        (ov.handles[0][0] - 64.0).abs() < 1.0 && (ov.handles[0][1] - 44.0).abs() < 1.0,
        "the ellipse rotated 90°: right handle now below centre: {:?}",
        ov.handles[0]
    );
}

#[test]
fn circle_centre_handle_moves_the_ellipse() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    // Press at the centre (axis handles are 20 px away > tol 6, so the centre is grabbed).
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([70.0, 72.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([70.0, 72.0], PointerPhase::Up));
    assert_eq!(
        t.ellipse_overlay().unwrap().handles[5],
        [70.0, 72.0],
        "centre moved"
    );
}

#[test]
fn ellipse_commit_unbakes_then_undo_removes_the_ring() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.ellipse_commit());
    assert!(t.ellipse_overlay().is_none(), "committed → no session");
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "committed ring survives"
    );
    // Unified timeline: undo of Apply reopens the ring (un-bake); a further undo removes its creation.
    assert!(t.undo_last());
    assert!(
        t.ellipse_overlay().is_some(),
        "undo of Apply reopens the ring (un-bake)"
    );
    while t.undo_last() {}
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "undoing every step reaches the pristine canvas"
    );
    assert!(t.ellipse_overlay().is_none(), "fully undone → no session");
}

#[test]
fn ellipse_cancel_reverts_all_pixels() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert_eq!(px(&t, 128, 84, 64), [0, 0, 0, 255], "ring painted");
    assert!(t.cancel_open_shape(), "a shape was open");
    assert!(t.ellipse_overlay().is_none());
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "cancel reverted the ring"
    );
}

#[test]
fn circle_undo_removes_the_creation_not_applies_it() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.ellipse_overlay().is_some(), "handles visible");
    assert_eq!(px(&t, 128, 84, 64), [0, 0, 0, 255], "ring painted");
    // Undo must NOT apply the circle (Enio 2026-06-28): it undoes the CREATION — the ring is gone, the
    // canvas reverts, the handles close. Folds into the same undo sequence as the paint flow.
    assert!(t.undo_last(), "undo removes the just-created circle");
    assert!(
        t.ellipse_overlay().is_none(),
        "the circle is gone (creation undone)"
    );
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "the painted ring reverted to white (not baked)"
    );
}

#[test]
fn ellipse_baked_when_switching_method_away() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.ellipse_overlay().is_some());
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "the ring painted a preview"
    );
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.ellipse_overlay().is_none(),
        "leaving Ellipse closed the session (baked, not still open)"
    );
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "the shape was APPLIED (baked), not erased"
    );
}

/// A `PainterTool` set to the Polygon method on a 128² white canvas, with a known grab tolerance.
fn polygon_tool() -> PainterTool {
    let mut t = white_canvas(128, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Polygon;
    t.set_shape_grab_tol_px(6.0);
    t
}

/// Draw a polygon centre (cx,cy) radius r (centre-out drag) and leave it in edit mode.
fn draw_polygon(t: &mut PainterTool, cx: f32, cy: f32, r: f32) {
    t.on_canvas_pointer(cp([cx, cy], PointerPhase::Down));
    t.on_canvas_pointer(cp([cx + r, cy], PointerPhase::Move));
    t.on_canvas_pointer(cp([cx + r, cy], PointerPhase::Up));
}

#[test]
fn polygon_draw_creates_an_editable_outline() {
    let mut t = polygon_tool();
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    // Gêmeo do gate da elipse: o contorno já existe, as alças ainda não (ver ali).
    assert!(
        !t.polygon_overlay()
            .expect("o contorno existe ja no Down")
            .editing,
        "no handles while drawing"
    );
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Up));

    let ov = t.polygon_overlay().expect("editing after release");
    assert!(ov.perimeter.len() >= 3, "at least a triangle");
    assert_eq!(ov.sides, 5, "default pentagon");
    assert_eq!(ov.handles[6], [64.0, 64.0], "centre handle");
    // The first vertex (top) of a pentagon at (64, 64+20) is painted (the OUTLINE), centre empty.
    assert_eq!(
        px(&t, 128, 64, 84),
        [0, 0, 0, 255],
        "top vertex of the outline painted"
    );
    assert_eq!(
        px(&t, 128, 64, 64),
        [255, 255, 255, 255],
        "centre empty (outline only)"
    );
}

#[test]
fn polygon_sides_handle_changes_the_side_count() {
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    // sides handle (index 5) for 5 sides sits at x = 64 + 20 + 3*6 + (5-3)*1.5*6 = 64 + 56 = 120.
    let sh = t.polygon_overlay().unwrap().handles[5];
    assert!(
        (sh[0] - 120.0).abs() < 0.5 && (sh[1] - 64.0).abs() < 0.5,
        "sides handle: {sh:?}"
    );

    // Drag it further out → more sides.
    t.on_canvas_pointer(cp([sh[0], sh[1]], PointerPhase::Down));
    t.on_canvas_pointer(cp([140.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([140.0, 64.0], PointerPhase::Up));
    assert!(
        t.polygon_overlay().unwrap().sides > 5,
        "dragging out adds sides"
    );

    // Drag it well in → clamps to the 3-side minimum.
    let sh2 = t.polygon_overlay().unwrap().handles[5];
    t.on_canvas_pointer(cp([sh2[0], sh2[1]], PointerPhase::Down));
    t.on_canvas_pointer(cp([66.0, 64.0], PointerPhase::Move)); // proj ≈ 2 → below the 3-side base
    t.on_canvas_pointer(cp([66.0, 64.0], PointerPhase::Up));
    assert_eq!(
        t.polygon_overlay().unwrap().sides,
        3,
        "clamps to the minimum"
    );
}

#[test]
fn polygon_axis_handle_resizes_one_axis() {
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    // Right axis handle at (84,64); drag out to rx = 30.
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Up));
    let ov = t.polygon_overlay().unwrap();
    assert!(
        (ov.handles[0][0] - 94.0).abs() < 0.5,
        "rx grew: {:?}",
        ov.handles[0]
    );
    assert!(
        (ov.handles[1][1] - 84.0).abs() < 0.5,
        "ry unchanged: {:?}",
        ov.handles[1]
    );
}

#[test]
fn polygon_rotate_handle_spins() {
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    let rot = t.polygon_overlay().unwrap().handles[4]; // (64, 64+20+18) = (64,102)
    t.on_canvas_pointer(cp([rot[0], rot[1]], PointerPhase::Down));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Move)); // drag rotate to the right of centre
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Up));
    let ov = t.polygon_overlay().unwrap();
    // u becomes (0,-1) → right handle = centre + (0,-1)*rx = (64, 44).
    assert!(
        (ov.handles[0][0] - 64.0).abs() < 1.0 && (ov.handles[0][1] - 44.0).abs() < 1.0,
        "rotated 90°: {:?}",
        ov.handles[0]
    );
}

#[test]
fn polygon_centre_handle_moves() {
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down)); // centre (axis handles 20px away)
    t.on_canvas_pointer(cp([70.0, 72.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([70.0, 72.0], PointerPhase::Up));
    assert_eq!(
        t.polygon_overlay().unwrap().handles[6],
        [70.0, 72.0],
        "centre moved"
    );
}

#[test]
fn polygon_commit_cancel_and_undo() {
    // Commit keeps the outline; the unified timeline un-bakes on the first undo, then walks back to pristine.
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert_eq!(px(&t, 128, 64, 84), [0, 0, 0, 255], "outline painted");
    assert!(t.polygon_commit());
    assert!(t.polygon_overlay().is_none());
    assert_eq!(
        px(&t, 128, 64, 84),
        [0, 0, 0, 255],
        "committed outline survives"
    );
    assert!(t.undo_last());
    assert!(
        t.polygon_overlay().is_some(),
        "undo of Apply reopens the polygon (un-bake)"
    );
    while t.undo_last() {}
    assert_eq!(
        px(&t, 128, 64, 84),
        [255, 255, 255, 255],
        "undoing every step reaches pristine"
    );

    // Cancel reverts.
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert!(t.cancel_open_shape());
    assert_eq!(px(&t, 128, 64, 84), [255, 255, 255, 255], "cancel reverted");

    // Undo while authoring removes the CREATION (reverts the canvas) — it must NOT apply the polygon.
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert!(t.undo_last(), "undo removes the just-created polygon");
    assert!(
        t.polygon_overlay().is_none(),
        "the polygon is gone (creation undone)"
    );
    assert_eq!(
        px(&t, 128, 64, 84),
        [255, 255, 255, 255],
        "the painted outline reverted to white (not baked)"
    );
}

/// Switching to a DIFFERENT tool (deactivating the Painter) also BAKES an open shape instead of erasing
/// it — the drawn shape is always applied (Enio 2026-07-03).
#[test]
fn switching_tools_bakes_the_open_shape() {
    use ph2d_editor_core::tool::Tool;
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "the ring painted a preview"
    );
    Tool::on_deactivate(&mut t);
    assert!(
        t.ellipse_overlay().is_none(),
        "deactivating closed the shape session (baked)"
    );
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "switching tools APPLIED the shape, not erased it"
    );
}

#[test]
fn polygon_baked_when_switching_method_away() {
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert!(t.polygon_overlay().is_some());
    assert_eq!(
        px(&t, 128, 64, 84),
        [0, 0, 0, 255],
        "the outline painted a preview"
    );
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.polygon_overlay().is_none(),
        "leaving Polygon closed the session (baked, not still open)"
    );
    assert_eq!(
        px(&t, 128, 64, 84),
        [0, 0, 0, 255],
        "the shape was APPLIED (baked), not erased"
    );
}

#[test]
fn apply_and_keep_folds_offset_into_accumulator_and_continues_outward() {
    // The accumulator in action on the Ellipse (offset = radii): Apply & Keep keeps the displayed ring put,
    // re-centres the slider, and a further offset CONTINUES from the kept position (not the base radius).
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0); // base radius 20
    let disp_r = |t: &PainterTool| -> f32 {
        let o = t.ellipse_overlay().expect("ring open");
        (o.handles[0][0] - o.handles[5][0]).abs()
    };
    t.set_brush_offset(0.55); // +10px → displayed radius 30
    assert!((disp_r(&t) - 30.0).abs() < 0.5, "offset shows radius 30");
    assert!(t.ellipse_commit_keep());
    assert!(
        (disp_r(&t) - 30.0).abs() < 0.5,
        "Apply & Keep did NOT move the shape (still 30)"
    );
    assert!(
        (t.brush_settings().offset - 0.5).abs() < 1e-4,
        "the slider re-centred to 0.5"
    );
    // Undo the Apply & Keep → the slider reverts; the shape stays at 30 the whole time.
    assert!(t.undo_last());
    assert!(
        (t.brush_settings().offset - 0.55).abs() < 1e-4,
        "undo restores the pre-commit slider"
    );
    assert!(
        (disp_r(&t) - 30.0).abs() < 0.5,
        "shape unchanged through undo"
    );
    assert!(t.redo_last());
    assert!(
        (t.brush_settings().offset - 0.5).abs() < 1e-4,
        "redo re-centres again"
    );
    // Continue outward: another +10 adds to the accumulated 10 over base 20 → displayed radius 40.
    t.set_brush_offset(0.55);
    assert!(
        (disp_r(&t) - 40.0).abs() < 0.5,
        "offset continues outward from the kept position (40, not 30)"
    );
}

#[test]
fn undo_sequence_open_shape_before_paint_history() {
    // The undo order is a single sequence: an OPEN shape's creation undoes first, only then the committed
    // paint history (Enio 2026-06-28).
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(
        t.commit_open_shape(),
        "first ring committed (a paint-history entry)"
    );
    assert_eq!(px(&t, 128, 84, 64), [0, 0, 0, 255], "first ring on canvas");
    // A second circle, still authoring.
    draw_circle(&mut t, 64.0, 64.0, 10.0);
    assert!(t.ellipse_overlay().is_some(), "second circle open");
    // Undo #1 removes the OPEN second circle (creation); the committed first ring is untouched.
    assert!(t.undo_last(), "undo the open second circle");
    assert!(t.ellipse_overlay().is_none(), "second circle gone");
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "committed first ring still there"
    );
    // Now the first ring's own steps unwind in order: its Apply un-bakes (reopens the ring), then its
    // creation reverts to pristine — AFTER the open shape, one unified sequence.
    assert!(t.undo_last(), "un-bake the first ring (its Apply)");
    assert!(t.ellipse_overlay().is_some(), "first ring reopened");
    assert!(t.undo_last(), "undo the first ring's creation");
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "first ring undone last"
    );
    assert!(t.ellipse_overlay().is_none(), "first ring fully gone");
}
