use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A `PainterTool` sourced with a white opaque `size`×`size` canvas (one
/// active raster layer) and a small hard black brush for crisp assertions.
fn white_canvas(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: radius,
        hardness: 1.0, // hard disk → deterministic centre
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        // These tests assert FULL-coverage pixels to verify painting mechanics
        // (alpha-lock / undo / blend). The Blender-default "Adjust Strength for
        // Spacing" attenuates a lone dab below full opacity, so opt out here — the
        // attenuation behaviour has its own dedicated engine test.
        space_attenuation: false,
        ..Default::default()
    };
    t
}

fn px(t: &PainterTool, size: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * size + x) * 4) as usize;
    [
        t.canvas_rgba[i],
        t.canvas_rgba[i + 1],
        t.canvas_rgba[i + 2],
        t.canvas_rgba[i + 3],
    ]
}

#[test]
fn down_paints_into_active_raster_and_marks_dirty() {
    let mut t = white_canvas(64, 6.0);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert_eq!(px(&t, 64, 32, 32), [0, 0, 0, 255], "centre painted black");
    assert!(t.preview_dirty, "preview flagged dirty");
    assert!(t.dirty_rect.is_some(), "dirty rect accumulated");
    // A far corner is untouched.
    assert_eq!(px(&t, 64, 0, 0), [255, 255, 255, 255]);
}

#[test]
fn trivial_stack_stroke_uploads_only_the_dab_bbox_not_the_whole_canvas() {
    // Regression: a single-layer (trivial) stroke must hand the bridge the dab's
    // dirty bbox so it patches only that sub-rect. Forcing `None` here made every
    // painted frame a full clone + premul + full GPU texture upload, O(W×H)
    // regardless of the 10px brush — the 300→150 fps drop.
    let mut t = white_canvas(64, 4.0);
    assert!(
        t.is_trivial_stack(),
        "single opaque Normal raster is trivial"
    );

    // First drain is the source-push seed (no paint yet) → `None` → the bridge
    // does one full upload to seed the GPU texture.
    assert!(t.take_preview_arc().is_some(), "source-push marks dirty");
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "seed frame uploads the full canvas (no dab yet)"
    );

    // Now paint one dab and drain again — the bbox must be present and strictly
    // smaller than the canvas (the dab footprint), not the whole 64×64.
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert!(
        t.take_preview_arc().is_some(),
        "the dab re-dirtied the preview"
    );
    let (bx, by, bw, bh) = t
        .take_preview_upload_bbox()
        .expect("a trivial-stack stroke must carry its dab bbox, not None");
    assert!(bw > 0 && bh > 0, "bbox is non-empty");
    assert!(
        bw < 64 && bh < 64,
        "partial upload, not the full canvas: got {bw}×{bh}"
    );
    assert!(
        bx <= 32 && by <= 32 && bx + bw >= 32 && by + bh >= 32,
        "bbox contains the dab centre (32,32): ({bx},{by},{bw},{bh})"
    );
}

#[test]
fn hover_never_paints() {
    let mut t = white_canvas(32, 4.0);
    let _ = t.take_preview_dirty(); // clear the dirty flag `set_source` raised
    assert!(!t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Hover)));
    assert_eq!(
        px(&t, 32, 16, 16),
        [255, 255, 255, 255],
        "hover left canvas untouched"
    );
    assert!(!t.preview_dirty, "hover did not re-dirty the preview");
}

#[test]
fn drag_dot_follows_cursor_leaving_no_trail() {
    // Blender Drag Dot: one dab follows the cursor (no trail) and only the dab at the release point
    // is committed. The tool restores the pixels under the previous position before re-stamping.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::DragDot;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down)); // dot appears at the press point
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Move)); // dot moves — previous erased
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move)); // dot moves again
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // commit at the release point
    assert_eq!(
        px(&t, 64, 56, 32),
        [0, 0, 0, 255],
        "the dot is committed at the release point"
    );
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "no trail left at the press point"
    );
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "no trail left at the intermediate point"
    );
    assert!(t.paint.stroke.is_none());
    assert!(
        t.paint.drag_preview.is_none(),
        "the restore record is cleared once the dot is committed"
    );
}

#[test]
fn stroke_down_move_up_paints_a_line() {
    let mut t = white_canvas(64, 3.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    // Spacing emits many dabs along the horizontal segment → the midpoint is
    // painted, while a point well off the line stays white.
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "midpoint of the stroke painted"
    );
    assert_eq!(
        px(&t, 64, 32, 10),
        [255, 255, 255, 255],
        "off-line pixel untouched"
    );
    // Stroke ended → no stroke in progress.
    assert!(t.paint.stroke.is_none());
}

#[test]
fn move_without_down_is_ignored() {
    let mut t = white_canvas(32, 4.0);
    assert!(
        !t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Move)),
        "stray move"
    );
    assert_eq!(px(&t, 32, 16, 16), [255, 255, 255, 255]);
}

#[test]
fn alpha_lock_blocks_paint_on_transparency() {
    // Canvas: left half opaque white, right half transparent.
    let size = 16u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 3.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false, // full coverage for the alpha-lock assertion
        ..Default::default()
    };
    // Enable alpha lock on the active layer.
    let active = t.layers.active().expect("active layer");
    t.layers.get_mut(active).expect("layer").alpha_locked = true;

    // Paint on the transparent side → blocked (no alpha created).
    t.on_canvas_pointer(cp([12.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 12, 8)[3],
        0,
        "alpha-lock blocked paint on transparency"
    );

    // Paint on the opaque side → recoloured, alpha preserved.
    t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 3, 8),
        [0, 0, 0, 255],
        "recoloured the opaque side"
    );
}

#[test]
fn brush_size_norm_round_trips_through_settings() {
    let mut t = PainterTool::default();
    t.set_brush_size_norm(0.5);
    let s = t.brush_settings();
    // Squared track: 0.5 → 1 + 0.25·(512−1) px, and the snapshot maps back.
    assert!((s.size_px - 128.75).abs() < 0.01, "size_px = {}", s.size_px);
    assert!(
        (s.size_norm - 0.5).abs() < 1e-4,
        "size_norm = {}",
        s.size_norm
    );
    // Clamps at the ends.
    t.set_brush_size_norm(2.0);
    assert!((t.brush_settings().size_px - BRUSH_SIZE_MAX_PX).abs() < 0.01);
    t.set_brush_size_norm(-1.0);
    assert!((t.brush_settings().size_px - BRUSH_SIZE_MIN_PX).abs() < 0.01);
}

#[test]
fn nudge_grows_and_shrinks_and_clamps() {
    let mut t = PainterTool::default();
    let start = t.brush_settings().size_px;
    let up = t.nudge_brush_size(1);
    assert!(up > start, "`]` grows ({start} → {up})");
    let down = t.nudge_brush_size(-1);
    assert!(down < up, "`[` shrinks ({up} → {down})");
    // Bracket-down never goes below the floor.
    for _ in 0..200 {
        t.nudge_brush_size(-1);
    }
    assert!((t.brush_settings().size_px - BRUSH_SIZE_MIN_PX).abs() < 0.01);
}

#[test]
fn brush_color_channels_set_and_clamp() {
    let mut t = PainterTool::default();
    t.set_brush_color_channel(0, 0.5);
    t.set_brush_color_channel(1, 2.0); // over → 1
    t.set_brush_color_channel(2, -1.0); // under → 0
    t.set_brush_color_channel(9, 0.7); // out-of-range channel → ignored
    assert_eq!(t.brush_settings().color, [0.5, 1.0, 0.0]);
}

#[test]
fn panel_events_drive_brush_size_colour_blend() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};

    let mut t = PainterTool::default();
    // Size slider drag (0..1 track).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_SIZE_SLIDER,
        0.5,
    ));
    assert!((t.brush_settings().size_px - 128.75).abs() < 0.01);
    // Colour from the shared Blender picker read-back ("r,g,b", 8-bit native).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_COLOR_THUMB,
        "255,64,0".to_string(),
    ));
    let c = t.brush_settings().color;
    assert!((c[0] - 1.0).abs() < 1e-6 && (c[1] - 64.0 / 255.0).abs() < 1e-6 && c[2] == 0.0);
    // Blend dropdown pick (wire u8 → Multiply == 3).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_BLEND,
        "3".to_string(),
    ));
    assert_eq!(t.brush_settings().blend, 3);
    // The chosen brush colour (255,64,0) + Multiply blend actually drive the
    // next stroke: a hard dab over white → white·colour = the colour itself at
    // full coverage.
    let size = 16u32;
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(4.0);
    t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false; // full coverage for the pixel assertion
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 8, 8),
        [255, 64, 0, 255],
        "Multiply brush colour over white painted the colour"
    );
}

#[test]
fn panel_events_drive_strength_falloff_and_eraser() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::Falloff;

    let mut t = PainterTool::default();
    // Strength slider.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STRENGTH_SLIDER,
        0.75,
    ));
    // Falloff preset pick (wire u8 → Constant == 8 = hard disk).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_FALLOFF,
        Falloff::Constant.to_u8().to_string(),
    ));
    let s = t.brush_settings();
    assert!((s.strength - 0.75).abs() < 1e-6, "strength {}", s.strength);
    assert_eq!(
        s.falloff,
        Falloff::Constant.to_u8(),
        "falloff preset applied"
    );
    assert!(!s.eraser);
    // Eraser toggle via the panel button.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ERASER));
    assert!(t.brush_settings().eraser, "eraser toggled on");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ERASER));
    assert!(!t.brush_settings().eraser, "eraser toggled off");
}

#[test]
fn panel_events_drive_custom_falloff_curve() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};

    let mut t = PainterTool::default();
    // Pick the editable Custom preset (wire u8 = 9).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_FALLOFF,
        Falloff::Custom.to_u8().to_string(),
    ));
    let s = t.brush_settings();
    assert_eq!(s.falloff, Falloff::Custom.to_u8(), "Custom preset selected");
    assert_eq!(s.falloff_len, 2, "default Custom curve = 2 endpoints");

    // "+" button → a third control point (profile unchanged until dragged).
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_FALLOFF_ADD));
    let s = t.brush_settings();
    assert_eq!(s.falloff_len, 3, "added a control point");
    // The new middle point (x≈0.5) — drive it by its STABLE id, not a position.
    let mid = s.falloff_points[..3]
        .iter()
        .find(|p| (p.x - 0.5).abs() < 0.05)
        .expect("middle point")
        .id;

    // 2-D drag of the middle point (by id) to (distance 0.5, strength 0.9).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_FALLOFF_EDIT,
        format!("{mid}:0.5:0.9"),
    ));
    let s = t.brush_settings();
    let p = s.falloff_points[..3].iter().find(|p| p.id == mid).unwrap();
    assert!((p.x - 0.5).abs() < 1e-6, "x moved");
    assert!((p.y - 0.9).abs() < 1e-6, "y moved");
    // The dab now evaluates the custom curve: mid-distance strength is lifted,
    // and the panel preview reads the SAME value the engine stamps.
    let w = brush_falloff_weight_at(&s, 0.5);
    assert!(w > 0.8, "custom curve lifted the mid strength, got {w}");

    // "−" button (payload = the stable id) drops the point; back to 2 endpoints.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_FALLOFF_REMOVE,
        mid.to_string(),
    ));
    assert_eq!(
        t.brush_settings().falloff_len,
        2,
        "removed the control point"
    );
}

#[test]
fn falloff_point_drags_past_neighbour_and_handle_sets() {
    use ph2d_painter_brush::HandleType;

    let mut t = PainterTool::default();
    t.set_brush_falloff(Falloff::Custom.to_u8());
    let mid = t.add_brush_falloff_point_at(0.5, 0.5).expect("added");
    // Drag the middle point PAST the right endpoint — the curve re-sorts and the
    // id stays valid (the handle keeps its grab).
    t.set_brush_falloff_point(mid, 1.0, 0.3);
    let s = t.brush_settings();
    let xs: Vec<f32> = s.falloff_points[..s.falloff_len as usize]
        .iter()
        .map(|p| p.x)
        .collect();
    for w in xs.windows(2) {
        assert!(w[0] <= w[1] + 1e-6, "points stay ascending after re-sort");
    }
    assert!(
        s.falloff_points[..s.falloff_len as usize]
            .iter()
            .any(|p| p.id == mid),
        "dragged id survives the re-sort"
    );
    // Vector handle (the right-click menu choice) sticks on the point.
    t.set_brush_falloff_point_handle(mid, HandleType::Vector.to_u8());
    let s = t.brush_settings();
    assert_eq!(
        s.falloff_points[..s.falloff_len as usize]
            .iter()
            .find(|p| p.id == mid)
            .unwrap()
            .handle,
        HandleType::Vector
    );
}

/// The user's REAL workflow end-to-end through the tool's public API: select
/// Custom, click-add a point (collinear, as the click-add does), drag it OFF the
/// line (so a Vector corner is geometrically visible), then set the Vector
/// handle. Assert `brush_falloff_weight_at` shows a SLOPE DISCONTINUITY — the
/// sharp corner the right-click menu promises. This is the step-5→7 contract the
/// shell drain depends on (the drain just calls `set_brush_falloff_point_handle`).
#[test]
fn vector_handle_on_dragged_off_line_point_makes_a_corner() {
    use ph2d_painter_brush::HandleType;

    let mut t = PainterTool::default();
    t.set_brush_falloff(Falloff::Custom.to_u8());
    // Click-add ON the line at x=0.5 (default curve passes through (0.5, 0.5)).
    let mid = t.add_brush_falloff_point_at(0.5, 0.5).expect("added");
    // Drag it OFF the line — up to (0.5, 0.9), non-collinear.
    t.set_brush_falloff_point(mid, 0.5, 0.9);

    let slopes = |t: &PainterTool| {
        let s = t.brush_settings();
        let l = (brush_falloff_weight_at(&s, 0.5) - brush_falloff_weight_at(&s, 0.49)) / 0.01;
        let r = (brush_falloff_weight_at(&s, 0.51) - brush_falloff_weight_at(&s, 0.5)) / 0.01;
        (l, r)
    };

    // Auto (default) → smooth, C1 across the point (no corner).
    let (al, ar) = slopes(&t);
    assert!((al - ar).abs() < 0.3, "Auto must be smooth: {al} vs {ar}");

    // The right-click menu choice: Vector handle on the off-line point.
    t.set_brush_falloff_point_handle(mid, HandleType::Vector.to_u8());
    let (vl, vr) = slopes(&t);
    assert!(
        (vl - vr).abs() > 1.0,
        "Vector must make a sharp corner (slope discontinuity): {vl} vs {vr}"
    );
}

#[test]
fn custom_falloff_curve_changes_the_dab() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};

    // Two identical hard-ish brushes, one Custom-lifted in the mid-band, paint a
    // dab; the lifted curve must darken a mid-radius pixel more than the default.
    let dab_mid = |custom: bool| -> u8 {
        let size = 40u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_size_px(14.0);
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_FALLOFF,
            Falloff::Custom.to_u8().to_string(),
        ));
        if custom {
            // Lift the whole interior toward full strength (steep shoulder): add a
            // point and drag it (by its stable id) up near the rim.
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_FALLOFF_ADD));
            let mid = t.brush_settings().falloff_points[..3]
                .iter()
                .find(|p| (p.x - 0.5).abs() < 0.05)
                .expect("middle point")
                .id;
            t.handle_panel_event(PanelEvent::SelectOption(
                core_ids::PAINTER_BRUSH_FALLOFF_EDIT,
                format!("{mid}:0.8:0.95"),
            ));
        }
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
        // A pixel ~9 px out from the centre (mid-band of a 14 px-radius dab).
        px(&t, size, 29, 20)[0]
    };
    assert!(
        dab_mid(true) < dab_mid(false),
        "the lifted Custom curve paints the mid-band darker"
    );
}

#[test]
fn eraser_removes_alpha_from_opaque_pixels() {
    // Opaque white canvas, hard brush; eraser on → a dab clears alpha.
    let mut t = white_canvas(32, 6.0);
    t.toggle_brush_eraser();
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 32, 16, 16)[3],
        0,
        "eraser cleared alpha at the centre"
    );
    // A far corner is untouched (still opaque).
    assert_eq!(px(&t, 32, 0, 0)[3], 255);
}

#[test]
fn dock_defaults_to_layers_then_toggles() {
    let mut t = PainterTool::default();
    assert!(
        t.dock_shows_layers(),
        "dock opens on the Layers/Effects view"
    );
    t.toggle_dock();
    assert!(
        !t.dock_shows_layers(),
        "header toggle flips to the Brush view"
    );
    t.toggle_dock();
    assert!(t.dock_shows_layers(), "toggling back returns to Layers");
}

#[test]
fn stroke_is_one_undo_step_and_redoable() {
    let mut t = white_canvas(64, 6.0);
    let pristine = Vec::clone(&t.canvas_rgba); // white, pre-stroke
    assert!(!t.can_undo(), "fresh source has nothing to undo");

    // One stroke (down → up).
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, pristine, "stroke changed pixels");
    assert!(t.can_undo(), "stroke pushed exactly one undo step");

    // Undo restores the pre-stroke pixels byte-for-byte.
    assert!(t.undo_last());
    assert_eq!(*t.canvas_rgba, pristine, "undo restored the canvas");
    assert!(!t.can_undo(), "one stroke == one undo step");

    // Redo repaints.
    assert!(t.redo_last());
    assert_ne!(*t.canvas_rgba, pristine, "redo repainted the stroke");
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "stroke start back to black"
    );
}

#[test]
fn stroke_section_panel_events_route_to_brush_settings() {
    // Behavioural seam (tool layer): a real `PanelEvent` from the Stroke section reaches the
    // matching `set_brush_*` setter and is reflected in the next `brush_settings()` snapshot,
    // including the clamps and the jitter-unit conditional routing.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();

    // Method dropdown (DragDot = wire 4).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        "4".into(),
    ));
    assert_eq!(t.brush_settings().stroke_method, 4);

    // Spacing slider (fraction-of-diameter track).
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_SPACING, 0.25));
    assert!((t.brush_settings().spacing - 0.25).abs() < 1e-6);

    // "Adjust Strength for Spacing" toggles from the default ON.
    assert!(t.brush_settings().space_attenuation);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SPACE_ATTEN));
    assert!(!t.brush_settings().space_attenuation);

    // Input samples: track 1.0 → max window; 0.0 → 1.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        1.0,
    ));
    assert_eq!(t.brush_settings().input_samples, BRUSH_COUNT_SLIDER_MAX);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        0.0,
    ));
    assert_eq!(t.brush_settings().input_samples, 1);

    // Stabilizer intensity slider: the 0..1 track lands verbatim on `stabilizer`.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_STABILIZE, 0.8));
    assert!((t.brush_settings().stabilizer - 0.8).abs() < 1e-6);

    // Rate slider: 0..1 track maps linearly onto [MIN, MAX] s; 0 → MIN, 1 → MAX.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_RATE, 0.0));
    assert!((t.brush_settings().airbrush_rate_s - BRUSH_AIRBRUSH_RATE_MIN_S).abs() < 1e-6);
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_RATE, 1.0));
    assert!((t.brush_settings().airbrush_rate_s - BRUSH_AIRBRUSH_RATE_MAX_S).abs() < 1e-6);

    // Edge to Edge toggles from the default OFF (Anchored only, but routing is method-agnostic).
    assert!(!t.brush_settings().edge_to_edge);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_EDGE_TO_EDGE));
    assert!(t.brush_settings().edge_to_edge);

    // Jitter unit routing: View → the Jitter slider drives absolute px; Brush → relative 0..1.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        "1".into(),
    ));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_JITTER, 1.0));
    assert!((t.brush_settings().jitter_absolute_px - BRUSH_JITTER_ABS_MAX_PX).abs() < 1e-3);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        "0".into(),
    ));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_JITTER, 0.3));
    assert!((t.brush_settings().jitter - 0.3).abs() < 1e-6);
}

#[test]
fn airbrush_deposits_on_the_tick_at_the_tracked_cursor_not_on_a_bare_move() {
    // End-to-end (tool layer): the airbrush is a timer method — a bare move lays NO dab; the
    // per-frame `on_tick` fires the timer and deposits at the cursor the moves tracked to. Wire
    // path: `on_canvas_pointer(Down/Move)` tracks position, `on_tick(dt_ms)` → `paint_tick` →
    // `Stroke::tick` → `stamp_dabs`. This is the behaviour the §2.1 handoff deferred until `on_tick`
    // drove the timer (it now does).
    use ph2d_editor_core::tool::Tool;
    let mut t = white_canvas(48, 4.0);
    t.paint.brush.stroke_method = StrokeMethod::Airbrush;
    t.paint.brush.airbrush_rate_s = 0.1; // 10 Hz
    t.paint.brush.stabilizer = 0.0; // raw, so the tick lands exactly at the moved-to point
    t.paint.brush.space_attenuation = false; // full coverage for the pixel assertion

    // Down at A: the begin dab paints A (airbrush `emits_on_begin`).
    t.on_canvas_pointer(cp([8.0, 24.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 48, 8, 24),
        [0, 0, 0, 255],
        "down paints the first dab"
    );

    // Move to B with NO tick: the airbrush must not paint on the bare move (timer-only).
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Move));
    assert_eq!(
        px(&t, 48, 40, 24),
        [255, 255, 255, 255],
        "a bare move left a dab — airbrush must deposit only on the timer"
    );

    // One frame of 100 ms = one rate period → the timer deposits one dab at the tracked cursor B.
    t.on_tick(100.0);
    assert_eq!(
        px(&t, 48, 40, 24),
        [0, 0, 0, 255],
        "the tick deposited the airbrush dab at the tracked cursor"
    );

    // Closing the stroke stops the spray: a later tick paints nothing new.
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Up));
    t.on_tick(100.0);
    assert_eq!(
        px(&t, 48, 24, 24),
        [255, 255, 255, 255],
        "no spray after pointer-up"
    );
}

#[test]
fn anchored_stamps_a_drag_sized_disc_centred_on_the_press_point() {
    // Anchored end-to-end (tool layer): press anchors (no paint), the drag sizes a single disc
    // centred on the press point (restore+re-stamp preview — no trail), pen-up commits it.
    let mut t = white_canvas(48, 4.0);
    t.paint.brush.stroke_method = StrokeMethod::Anchored;
    t.paint.brush.edge_to_edge = false;
    t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    // Press at the anchor — nothing painted yet (interactive).
    t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 48, 10, 24),
        [255, 255, 255, 255],
        "the press alone paints nothing"
    );

    // An intermediate small drag then a larger one: the preview restores between moves, so only the
    // final disc survives — proving the resize leaves no trail.
    t.on_canvas_pointer(cp([16.0, 24.0], PointerPhase::Move)); // small (r≈6)
    t.on_canvas_pointer(cp([26.0, 24.0], PointerPhase::Move)); // grow (r≈16)
    t.on_canvas_pointer(cp([26.0, 24.0], PointerPhase::Up)); // commit

    // Committed disc: centre = anchor (10,24), radius = final drag distance 16.
    assert_eq!(px(&t, 48, 10, 24), [0, 0, 0, 255], "anchor painted");
    assert_eq!(
        px(&t, 48, 22, 24),
        [0, 0, 0, 255],
        "12 px from the anchor is inside the disc"
    );
    assert_eq!(
        px(&t, 48, 0, 0),
        [255, 255, 255, 255],
        "a far corner is outside the disc"
    );
}

#[test]
fn line_paints_a_straight_committed_line_with_no_trail() {
    // Line end-to-end (tool layer): press anchors (no paint), each move previews the straight
    // anchor→cursor line (restore + re-stamp — no trail), pen-up commits the last. A wrong
    // intermediate drag must leave no trace.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 64, 8, 8),
        [255, 255, 255, 255],
        "the press alone paints nothing"
    );

    // Drag to a WRONG spot (a vertical line), then to the final spot (a horizontal line), release.
    t.on_canvas_pointer(cp([8.0, 56.0], PointerPhase::Move)); // wrong: vertical
    t.on_canvas_pointer(cp([56.0, 8.0], PointerPhase::Move)); // final: horizontal
    t.on_canvas_pointer(cp([56.0, 8.0], PointerPhase::Up)); // commit

    // The committed line is horizontal at y=8 from the anchor (8,8) to the release (56,8).
    assert_eq!(px(&t, 64, 8, 8), [0, 0, 0, 255], "anchor end painted");
    assert_eq!(
        px(&t, 64, 32, 8),
        [0, 0, 0, 255],
        "midpoint of the committed line painted"
    );
    assert_eq!(px(&t, 64, 56, 8), [0, 0, 0, 255], "release end painted");
    // The discarded vertical drag left no trail (restored before the horizontal re-stamp).
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "the discarded vertical drag left no trail"
    );
}

#[test]
fn snap_to_45_projects_onto_the_eight_rays() {
    let a = [0.0, 0.0];
    assert_eq!(
        snap_to_45(a, [10.0, 1.0]),
        [10.0, 0.0],
        "near-horizontal → flat"
    );
    assert_eq!(
        snap_to_45(a, [1.0, 10.0]),
        [0.0, 10.0],
        "near-vertical → vertical"
    );
    assert_eq!(
        snap_to_45(a, [-1.0, 10.0]),
        [0.0, 10.0],
        "sign of the cursor picks the ray"
    );
    let d = snap_to_45(a, [10.0, 9.0]); // near-diagonal
    assert!(
        (d[0] - d[1]).abs() < 1e-4,
        "snapped onto the y=x diagonal: {d:?}"
    );
    assert!(d[0] > 0.0);
}

#[test]
fn line_alt_constrain_snaps_to_45_degrees() {
    // Alt-drag constrains the Line to 45° increments around the anchor: a near-horizontal drag
    // (small vertical drift) snaps flat onto the anchor's row.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;
    t.set_line_constrain(true);

    t.on_canvas_pointer(cp([8.0, 30.0], PointerPhase::Down));
    // Drag to (56, 36): 48 across, 6 down → ~7° → snaps to horizontal at y=30, projected to x=56.
    t.on_canvas_pointer(cp([56.0, 36.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 36.0], PointerPhase::Up));

    assert_eq!(
        px(&t, 64, 32, 30),
        [0, 0, 0, 255],
        "the snapped line is horizontal at the anchor row"
    );
    assert_eq!(
        px(&t, 64, 56, 30),
        [0, 0, 0, 255],
        "snapped endpoint painted at (56,30)"
    );
    assert_eq!(
        px(&t, 64, 56, 36),
        [255, 255, 255, 255],
        "the un-snapped endpoint (56,36) is NOT on the constrained line"
    );
}

/// Curve: a press-drag-release of the initial line yields a 3-point editable curve (overlay shows
/// start / midpoint / end), the midpoint pre-selected — and the line paints along it (no trail).
#[test]
fn curve_draw_creates_three_points_and_paints_the_line() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    assert!(t.curve_overlay().is_none(), "no chrome before drawing");
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    assert!(
        t.curve_overlay().is_none(),
        "still drawing — chrome appears on release"
    );
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));

    let ov = t
        .curve_overlay()
        .expect("editing after the line is released");
    assert_eq!(ov.points.len(), 3, "start + midpoint + end");
    assert_eq!(ov.points[0], [8.0, 32.0]);
    assert_eq!(ov.points[2], [56.0, 32.0]);
    assert_eq!(
        ov.selected,
        Some(1),
        "midpoint pre-selected (ready to bend)"
    );
    // The straight line is painted along y=32 (preview is live, not committed).
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "the line paints through the midpoint"
    );
}

/// Dragging the selected midpoint bends the curve OFF the chord — pixels appear above the original
/// straight line. Esc then reverts every painted pixel (nothing was committed).
#[test]
fn curve_bend_then_cancel_reverts_all_pixels() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3 points now
    // Click a 4th point in open space (added + grabbed + selected).
    t.on_canvas_pointer(cp([40.0, 50.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 50.0], PointerPhase::Up));
    let ov = t.curve_overlay().unwrap();
    assert_eq!(
        ov.points.len(),
        4,
        "a point was added on the empty-space click"
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
        px(&t, 64, 32, 32) != [255, 255, 255, 255],
        "something is painted pre-commit"
    );
    assert!(t.curve_commit());
    assert!(t.curve_overlay().is_none(), "committed → no session");
    let painted = px(&t, 64, 8, 32);
    assert_eq!(painted, [0, 0, 0, 255], "committed dabs survive");
    // One undo step: undo restores the pristine canvas.
    assert!(t.undo_last());
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "the whole curve undoes as one step"
    );
}

/// Switching the stroke method away from Curve mid-session discards it (reverts the preview).
#[test]
fn curve_discarded_when_switching_method_away() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    t.on_canvas_pointer(cp([8.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 20.0], PointerPhase::Up));
    assert!(t.curve_overlay().is_some());
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.curve_overlay().is_none(),
        "leaving Curve discarded the session"
    );
    assert_eq!(
        px(&t, 64, 32, 20),
        [255, 255, 255, 255],
        "the preview was reverted"
    );
}

/// The grab tolerance is honoured: a Down within the forwarded radius grabs the nearest point; a
/// Down well outside it adds a new point instead.
#[test]
fn curve_grab_tolerance_grabs_near_and_adds_far() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.set_shape_grab_tol_px(5.0);

    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // points: 8 / 32 / 56 at y=32
    assert_eq!(t.curve_overlay().unwrap().points.len(), 3);

    // Down 3 px from the midpoint (within tol 5) → grabs it, no new point.
    t.on_canvas_pointer(cp([32.0, 35.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 35.0], PointerPhase::Up));
    assert_eq!(
        t.curve_overlay().unwrap().points.len(),
        3,
        "near press grabbed, didn't add"
    );

    // Down far from every point (> tol) → adds a 4th.
    t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Up));
    assert_eq!(
        t.curve_overlay().unwrap().points.len(),
        4,
        "far press added a point"
    );
}

/// Undo while the curve is being authored (points visible) COMMITS it first (applies the curve,
/// clears the points); the NEXT undo removes the committed stroke. Regression for "the drawing
/// vanished but the control points stayed" — undo must not strand the points over a reverted canvas.
#[test]
fn curve_undo_commits_first_then_undoes() {
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3 points, painted, editing
    assert!(
        t.curve_overlay().is_some(),
        "points visible while authoring"
    );
    assert_eq!(px(&t, 64, 8, 32), [0, 0, 0, 255], "curve painted");

    // Undo #1: applies the curve — points cleared, the painted curve survives (no orphan state).
    assert!(t.undo_last());
    assert!(
        t.curve_overlay().is_none(),
        "first undo applied the curve (points cleared)"
    );
    assert_eq!(
        px(&t, 64, 8, 32),
        [0, 0, 0, 255],
        "the painted curve survives the commit"
    );

    // Undo #2: now undoes the committed stroke — back to the pristine canvas.
    assert!(t.undo_last());
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "second undo removes the committed curve"
    );
}

/// A `PainterTool` set to the Circle method on a 128² white canvas, with a known grab tolerance so
/// the handle positions are predictable.
fn circle_tool() -> PainterTool {
    let mut t = white_canvas(128, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Circle;
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
fn circle_draw_creates_an_editable_ellipse_outline() {
    let mut t = circle_tool();
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    assert!(t.circle_overlay().is_none(), "no handles while drawing");
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Up));

    let ov = t.circle_overlay().expect("editing after release");
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
    let ov = t.circle_overlay().unwrap();
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
    let rot = t.circle_overlay().unwrap().handles[4];
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
    let ov = t.circle_overlay().unwrap();
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
        t.circle_overlay().unwrap().handles[5],
        [70.0, 72.0],
        "centre moved"
    );
}

#[test]
fn circle_commit_keeps_the_ring_and_is_one_undo_step() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.circle_commit());
    assert!(t.circle_overlay().is_none(), "committed → no session");
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "committed ring survives"
    );
    assert!(t.undo_last());
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "one undo removes the whole ring"
    );
}

#[test]
fn circle_cancel_reverts_all_pixels() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert_eq!(px(&t, 128, 84, 64), [0, 0, 0, 255], "ring painted");
    assert!(t.cancel_open_shape(), "a shape was open");
    assert!(t.circle_overlay().is_none());
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "cancel reverted the ring"
    );
}

#[test]
fn circle_undo_commits_first_then_undoes() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.circle_overlay().is_some(), "handles visible");
    // Undo #1 applies the circle (handles gone, ring survives).
    assert!(t.undo_last());
    assert!(
        t.circle_overlay().is_none(),
        "first undo applied the circle"
    );
    assert_eq!(
        px(&t, 128, 84, 64),
        [0, 0, 0, 255],
        "ring survives the commit"
    );
    // Undo #2 removes the committed ring.
    assert!(t.undo_last());
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "second undo removes the ring"
    );
}

#[test]
fn circle_discarded_when_switching_method_away() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.circle_overlay().is_some());
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.circle_overlay().is_none(),
        "leaving Circle discarded the session"
    );
    assert_eq!(
        px(&t, 128, 84, 64),
        [255, 255, 255, 255],
        "the preview was reverted"
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
    assert!(t.polygon_overlay().is_none(), "no handles while drawing");
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
    // Commit keeps the outline + is one undo step.
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
    assert_eq!(
        px(&t, 128, 64, 84),
        [255, 255, 255, 255],
        "one undo removes it"
    );

    // Cancel reverts.
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert!(t.cancel_open_shape());
    assert_eq!(px(&t, 128, 64, 84), [255, 255, 255, 255], "cancel reverted");

    // Undo while authoring commits first.
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert!(t.undo_last());
    assert!(
        t.polygon_overlay().is_none(),
        "first undo applied the polygon"
    );
    assert_eq!(
        px(&t, 128, 64, 84),
        [0, 0, 0, 255],
        "outline survives the commit"
    );
    assert!(t.undo_last());
    assert_eq!(
        px(&t, 128, 64, 84),
        [255, 255, 255, 255],
        "second undo removes it"
    );
}

#[test]
fn polygon_discarded_when_switching_method_away() {
    let mut t = polygon_tool();
    draw_polygon(&mut t, 64.0, 64.0, 20.0);
    assert!(t.polygon_overlay().is_some());
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.polygon_overlay().is_none(),
        "leaving Polygon discarded it"
    );
    assert_eq!(
        px(&t, 128, 64, 84),
        [255, 255, 255, 255],
        "the preview was reverted"
    );
}

#[test]
fn texture_setters_clamp_and_new_assigns_noise() {
    use ph2d_painter_brush::{TEX_OFFSET_MAX, TEX_SIZE_MAX, TextureKind, TextureMapping};
    let mut t = PainterTool::default();
    // No texture by default; "New" assigns the default procedural (Noise).
    assert_eq!(t.brush_settings().texture_kind, TextureKind::None.to_u8());
    t.new_brush_texture();
    assert_eq!(t.brush_settings().texture_kind, TextureKind::Noise.to_u8());
    // Kind + mapping round-trip through the wire setters.
    t.set_brush_texture_kind(TextureKind::Checker.to_u8());
    assert_eq!(
        t.brush_settings().texture_kind,
        TextureKind::Checker.to_u8()
    );
    t.set_brush_texture_mapping(TextureMapping::Tiled.to_u8());
    assert_eq!(
        t.brush_settings().texture_mapping,
        TextureMapping::Tiled.to_u8()
    );
    // Angle: 0..1 track → 0..=360°, clamped.
    t.set_brush_texture_angle_norm(0.5);
    assert_eq!(t.brush_settings().texture_angle_deg, 180);
    t.set_brush_texture_angle_norm(2.0);
    assert_eq!(t.brush_settings().texture_angle_deg, 360);
    // Offset: track 0.5 → 0 (centre of the symmetric range); track 1 → MAX.
    t.set_brush_texture_offset_norm(0, 0.5);
    assert!(t.brush_settings().texture_offset[0].abs() < 1e-6);
    t.set_brush_texture_offset_norm(1, 1.0);
    assert!((t.brush_settings().texture_offset[1] - TEX_OFFSET_MAX).abs() < 1e-6);
    // Size: track 1 → MAX; a bad axis index is a no-op (Y stays at the default 1.0).
    t.set_brush_texture_size_norm(0, 1.0);
    assert!((t.brush_settings().texture_size[0] - TEX_SIZE_MAX).abs() < 1e-6);
    t.set_brush_texture_size_norm(9, 0.0);
    assert!((t.brush_settings().texture_size[1] - 1.0).abs() < 1e-6);
    // Rake / Random toggles flip.
    t.toggle_brush_texture_rake();
    t.toggle_brush_texture_random();
    assert!(t.brush_settings().texture_rake && t.brush_settings().texture_random);
}

#[test]
fn textured_dab_masks_part_of_the_footprint() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 14.0);
    // Big checker tiles so each cell spans several pixels across the footprint; the 0-cells
    // deposit no paint, so a textured hard dab leaves a MIX of black + untouched white pixels —
    // proving the texture reaches the canvas through the tool's stamp_dabs → stamp_dab_textured.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::ViewPlane,
        size: [0.25, 0.25],
        ..Default::default()
    };
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    let (mut black, mut white) = (0, 0);
    for y in 18..46 {
        for x in 18..46 {
            match px(&t, 64, x, y) {
                [0, 0, 0, 255] => black += 1,
                [255, 255, 255, 255] => white += 1,
                _ => {}
            }
        }
    }
    assert!(black > 0, "the texture let some paint through");
    assert!(
        white > 0,
        "the texture masked part of the footprint (checker 0-cells)"
    );
}
