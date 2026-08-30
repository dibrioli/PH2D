//! **Os controles do pincel e a rota dos eventos de painel.** Tamanho, cor, blend, strength, falloff
//! (incluindo a curva autorada e o arrasto dos seus pontos), os resets por seção, os campos de valor
//! real, e os parâmetros de textura, papel e rampa — com o que cada um faz ao dab.

use super::*;

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
fn panel_events_drive_shape_and_grain_depth() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};

    let mut t = PainterTool::default();
    // Grain Depth slider (Grain section).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_GRAIN_DEPTH,
        0.4,
    ));
    assert!(
        (t.brush_settings().grain_depth - 0.4).abs() < 1e-6,
        "grain depth set"
    );

    // Shape rotation controls (tracked on the spec even before an image is assigned). The number field
    // forwards the REAL degrees now (not a 0..1 track), Enio 2026-06-25.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_ANGLE, 180.0));
    assert_eq!(t.brush_settings().shape_angle_deg, 180, "shape angle set");
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_FOLLOW,
        "1".to_string(), // Rake
    ));
    assert_eq!(t.brush_settings().shape_follow, 1, "shape follow → Rake");
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_SIZE_X, 0.0)); // → TEX_SIZE_MIN
    assert!(
        (t.brush_settings().shape_size[0] - 0.1).abs() < 1e-4,
        "shape size X → min"
    );

    // No image yet ⇒ the silhouette is the falloff.
    assert!(!t.brush_settings().shape_has_image, "no shape image yet");

    // Dab flatten/rotate gizmo (Shape section): non-default before reset.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_DAB_FLATTEN,
        0.5,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_DAB_ANGLE,
        90.0,
    ));
    assert!(
        (t.brush_settings().dab_flatten - 0.5).abs() < 1e-6,
        "dab flatten set"
    );
    assert_eq!(t.brush_settings().dab_angle_deg, 90, "dab angle set");

    // Assign a Shape image ⇒ shape_has_image flips; the section reset clears it (→ falloff) + rotation
    // + the dab flatten/rotate gizmo.
    t.set_brush_shape_image(vec![255u8; 16], 4, 4);
    assert!(t.brush_settings().shape_has_image, "shape image assigned");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RESET));
    let s = t.brush_settings();
    assert!(!s.shape_has_image, "reset cleared the shape image");
    assert_eq!(s.shape_angle_deg, 0, "reset cleared the shape angle");
    assert_eq!(s.shape_follow, 0, "reset cleared follow → Off");
    assert_eq!(s.dab_flatten, 0.0, "reset cleared the dab flatten");
    assert_eq!(s.dab_angle_deg, 0, "reset cleared the dab angle");
}

#[test]
fn texture_and_shape_number_fields_set_real_values_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    // The Grain/Shape param fields are NumberInputs forwarding the REAL value (degrees / tile-fraction /
    // scale), not a 0..1 track — the tool's real-value setters clamp it (Enio 2026-06-25).
    let mut t = PainterTool::default();
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        5.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        -0.5,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_ANGLE,
        90.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_SIZE_X, 3.0));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_ANGLE, 45.0));
    let b = t.brush_settings();
    assert!(
        (b.texture_size[0] - 5.0).abs() < 1e-4,
        "Grain Size X real: {}",
        b.texture_size[0]
    );
    assert!(
        (b.texture_offset[0] + 0.5).abs() < 1e-4,
        "Grain Offset X real: {}",
        b.texture_offset[0]
    );
    assert_eq!(b.texture_angle_deg, 90, "Grain Angle real degrees");
    assert!(
        (b.shape_size[0] - 3.0).abs() < 1e-4,
        "Shape Size X real: {}",
        b.shape_size[0]
    );
    assert_eq!(b.shape_angle_deg, 45, "Shape Angle real degrees");
}

#[test]
fn accumulate_off_caps_the_stroke_even_with_a_colour_ramp() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};

    // Color ramp ON (so the ramped stamp path is taken) + Strength 0.5: Accumulate OFF must CAP the
    // overlapping back-and-forth stroke at Strength; ON builds past it. Regression for the cap being
    // dropped on the Color-Ramp path (Enio 2026-06-25).
    let make = |accumulate: bool| -> PainterTool {
        let mut t = white_canvas(64, 10.0);
        t.set_brush_strength(0.5);
        if accumulate {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ACCUMULATE));
        }
        t.set_shape_color_ramp(ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
                RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        ));
        t.set_shape_ramp_enabled(true); // no Grain ⇒ the Shape ramp owns colour (the ramped path)
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        for _ in 0..5 {
            t.on_canvas_pointer(cp([38.0, 32.0], PointerPhase::Move));
            t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
        t
    };
    // Red over white: green+blue at the centre measures the white still showing — HIGHER = less opaque.
    let whiteness = |t: &PainterTool| {
        let p = px(t, 64, 32, 32);
        u32::from(p[1]) + u32::from(p[2])
    };
    assert!(
        whiteness(&make(false)) > whiteness(&make(true)) + 30,
        "Accumulate OFF caps the colour-ramp stroke (lighter than ON): off={} on={}",
        whiteness(&make(false)),
        whiteness(&make(true))
    );
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
fn section_reset_buttons_restore_section_defaults() {
    // Each section's reset icon (forwarded as a Click) restores that section's brush fields to
    // defaults while leaving the OTHER sections untouched (Enio 2026-06-24).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();

    // Dirty several sections.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
        0.5,
    ));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_SPACING, 0.42));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SPACE_ATTEN)); // Adjust Strength → on
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_TILING_X)); // Tiling X → on
    t.new_brush_texture(); // assign a procedural texture (Noise)
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
    )); // ramp → on

    let s = t.brush_settings();
    assert!(s.color_jitter[0] > 0.0);
    assert!(s.tiling[0]);
    assert_ne!(s.texture_kind, 0);
    assert!(s.texture_ramp_enabled);

    // Randomize reset → hue back to 0; nothing else touched.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_RANDOMIZE_RESET));
    assert_eq!(t.brush_settings().color_jitter[0], 0.0);
    assert!(
        t.brush_settings().tiling[0],
        "randomize reset spared tiling"
    );

    // Stroke reset → spacing + Adjust-Strength back to defaults; tiling untouched.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_RESET));
    let s = t.brush_settings();
    assert!((s.spacing - 0.10).abs() < 1e-6);
    assert!(!s.space_attenuation);
    assert!(s.tiling[0], "stroke reset must not touch tiling");

    // Color Ramp reset → ramp off, but the texture stays assigned (finer than the Texture reset).
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_COLOR_RAMP_RESET));
    assert!(!t.brush_settings().texture_ramp_enabled);
    assert_ne!(
        t.brush_settings().texture_kind,
        0,
        "ramp reset must not clear the texture"
    );

    // Texture reset → texture cleared to None.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_TEXTURE_RESET));
    assert_eq!(t.brush_settings().texture_kind, 0);

    // Tiling reset → tiling off.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_TILING_RESET));
    assert!(!t.brush_settings().tiling[0]);
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

    // "Adjust Strength for Spacing" toggles from the default OFF (Enio 2026-06-24).
    assert!(!t.brush_settings().space_attenuation);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SPACE_ATTEN));
    assert!(t.brush_settings().space_attenuation);

    // "Accumulate" toggles from the default OFF (Blender default; off caps a stroke at Strength).
    assert!(!t.brush_settings().accumulate);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ACCUMULATE));
    assert!(t.brush_settings().accumulate);

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
    // Rake toggle flips (per-slot Random Angle was retired 2026-07-19 — Jitter Rotate covers a random spin).
    t.toggle_brush_texture_rake();
    assert!(t.brush_settings().texture_rake);
}

#[test]
fn texture_params_reset_on_kind_change_and_set_per_slot() {
    use ph2d_painter_brush::{TextureKind, param_specs};
    let mut t = white_canvas(32, 8.0);
    // Selecting a kind resets params to that kind's spec defaults (Grid: …/…/Thickness/Frequency).
    t.set_brush_texture_kind(TextureKind::Grid.to_u8());
    let specs = param_specs(TextureKind::Grid);
    assert_eq!(
        specs.len(),
        4,
        "Grid exposes Contrast/Brightness/Thickness/Frequency"
    );
    assert!(
        (t.brush_settings().texture_params[2] - specs[2].default).abs() < 1e-6,
        "slot 2 reset to Grid's Thickness default ({})",
        specs[2].default
    );
    // A param setter stores the normalized track; an out-of-range slot is a no-op.
    t.set_brush_texture_param_norm(0, 0.9);
    assert!((t.brush_settings().texture_params[0] - 0.9).abs() < 1e-6);
    t.set_brush_texture_param_norm(9, 0.0); // bad slot ignored, no panic
    // Switching kinds re-resets every slot to the new kind's spec defaults, neutral 0.5 past them.
    t.set_brush_texture_kind(TextureKind::Diamonds.to_u8());
    let dspecs = param_specs(TextureKind::Diamonds);
    assert!(
        (t.brush_settings().texture_params[0] - 0.5).abs() < 1e-6,
        "Contrast reset to default on kind change"
    );
    assert!(
        (t.brush_settings().texture_params[2] - dspecs[2].default).abs() < 1e-6,
        "slot 2 reset to Diamonds' Softness default on kind change"
    );
    assert!(
        (t.brush_settings().texture_params[dspecs.len()] - 0.5).abs() < 1e-6,
        "a slot past the kind's specs resets to neutral 0.5"
    );
}

/// **The Paper slot resets its params on a kind change, exactly like the Grain slot (Enio 2026-07-11).**
/// Picking Voronoi in Paper used to keep the neutral `[0.5; 6]` params, so it rendered with Randomness
/// `0.5` + Metric `0.5` (Chebyshev square cells) instead of Voronoi's own defaults (Randomness `1.0` +
/// Metric `0.0` = organic Euclidean) — looking nothing like the SAME kind in the Grain slot. Now
/// `set_brush_paper_kind` resets to the kind's `param_specs` defaults, so Paper == Grain for a kind.
#[test]
fn paper_kind_change_resets_params_to_kind_defaults_matching_grain() {
    use ph2d_painter_brush::{TextureKind, param_specs};
    let mut t = white_canvas(32, 8.0);
    let wire = TextureKind::Voronoi.to_u8();
    t.set_brush_paper_kind(wire);
    t.set_brush_texture_kind(wire);
    let paper = t.brush_settings().paper_params;
    let grain = t.brush_settings().texture_params;
    // Same kind ⇒ Paper and Grain share the kind defaults (the bug left Paper at the neutral 0.5).
    assert_eq!(
        paper, grain,
        "Paper and Grain of the same kind must reset to the SAME param defaults"
    );
    // And specifically the VORONOI defaults, not the neutral 0.5 (`param_specs`: Randomness 1.0, Metric 0.0).
    let specs = param_specs(TextureKind::Voronoi);
    assert!(
        (paper[2] - specs[2].default).abs() < 1e-6 && (paper[2] - 1.0).abs() < 1e-6,
        "Paper Voronoi Randomness reset to 1.0 (was the neutral 0.5): {}",
        paper[2]
    );
    assert!(
        (paper[4] - specs[4].default).abs() < 1e-6 && paper[4].abs() < 1e-6,
        "Paper Voronoi Metric reset to 0.0 = Euclidean (was 0.5 = Chebyshev square): {}",
        paper[4]
    );
}

/// **A procedural Paper kind defaults to a FINE tooth Size; presets/Image stay at 1 (Enio 2026-07-11).**
/// The paper is canvas-Tiled (`rel = px·size/256`), so a procedural at Size 1 shows 256-px "giant blobs".
/// Picking a procedural defaults the Size to `PAPER_PROCEDURAL_DEFAULT_SIZE`; a baked preset / Image is one
/// full tile per 256 px ⇒ Size 1. The default only re-applies when the SCALE CLASS changes (procedural ↔
/// bitmap), so a Size the user tuned survives a switch between two procedural kinds.
#[test]
fn procedural_paper_defaults_to_a_fine_size_presets_stay_at_one() {
    use super::watercolor_settings::PAPER_PROCEDURAL_DEFAULT_SIZE;
    use ph2d_painter_brush::TextureKind;
    let mut t = white_canvas(32, 8.0);
    let fine = PAPER_PROCEDURAL_DEFAULT_SIZE;
    assert!(
        fine > 4.0,
        "the fine default must be meaningfully finer than Size 1"
    );

    // None → procedural: class changes ⇒ fine default.
    t.set_brush_paper_kind(TextureKind::Voronoi.to_u8());
    assert_eq!(
        t.brush_settings().paper_size,
        [fine, fine],
        "Voronoi paper gets the fine tooth default"
    );

    // Procedural → procedural: SAME class ⇒ a user-tuned Size survives the kind switch.
    t.set_brush_paper_size(0, 30.0);
    t.set_brush_paper_size(1, 30.0);
    t.set_brush_paper_kind(TextureKind::Noise.to_u8());
    assert_eq!(
        t.brush_settings().paper_size,
        [30.0, 30.0],
        "tuned Size preserved within the procedural class"
    );

    // Procedural → baked preset: class changes ⇒ back to one full tile (Size 1).
    t.set_brush_paper_kind(TextureKind::PaperCold.to_u8());
    assert_eq!(
        t.brush_settings().paper_size,
        [1.0, 1.0],
        "a baked preset resets to Size 1 (one 256² tile)"
    );

    // Preset → procedural again ⇒ the fine default returns.
    t.set_brush_paper_kind(TextureKind::Checker.to_u8());
    assert_eq!(
        t.brush_settings().paper_size,
        [fine, fine],
        "back to the fine default for a procedural"
    );
}

/// **Comprehensive guard: EVERY kind renders the same in Paper and Grain (Enio 2026-07-11).** The whole
/// bug class the smoke surfaced is "a slot doesn't reset its params to the kind defaults, so the same kind
/// looks different per slot". This sweeps all `TextureKind`s: after selecting a kind in BOTH slots, their
/// params must be equal (both = the kind's `param_specs` defaults). Catches any future slot-setter that
/// forgets the reset, for any kind — not just the reported Voronoi.
#[test]
fn every_kind_resets_paper_params_to_match_grain() {
    use ph2d_painter_brush::TextureKind;
    let mut t = white_canvas(32, 8.0);
    let mut seen = std::collections::BTreeSet::new();
    for k in 0u8..40 {
        let wire = TextureKind::from_u8(k).to_u8(); // canonical wire (unknown → None), dedup below
        if !seen.insert(wire) {
            continue;
        }
        t.set_brush_paper_kind(wire);
        t.set_brush_texture_kind(wire);
        assert_eq!(
            t.brush_settings().paper_params,
            t.brush_settings().texture_params,
            "kind {:?}: Paper and Grain params must match after a kind change",
            TextureKind::from_u8(wire)
        );
    }
    assert!(
        seen.len() > 15,
        "the sweep must cover the full kind set, not just a few: {}",
        seen.len()
    );
}

#[test]
fn ramp_move_stop_can_cross_a_neighbour_by_id() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    let mut t = white_canvas(32, 8.0);
    t.set_texture_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 0.0, 0.0, 1.0]),
            RampStop::new(0.4, [1.0, 0.0, 0.0, 1.0]), // RED, the middle stop
            RampStop::new(0.8, [1.0, 1.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    let mid_id = t.texture_ramp().stops()[1].id; // the RED stop's stable id
    // Drag it PAST the 0.8 stop to 0.9 — tracked by id, it crosses + keeps its colour.
    t.ramp_move_stop(mid_id, 0.9);
    let stops = t.texture_ramp().stops();
    assert_eq!(
        stops[2].id, mid_id,
        "the dragged stop crossed to the last position, same id"
    );
    assert!((stops[2].pos - 0.9).abs() < 1e-6, "at its new position");
    assert_eq!(
        stops[2].color,
        [1.0, 0.0, 0.0, 1.0],
        "kept its red colour through the cross"
    );
}

#[test]
fn ramp_set_stop_color_applies_alpha() {
    let mut t = white_canvas(32, 8.0);
    // Default 2 stops with ids 0,1; recolour id 0 to a half-transparent red (sRGB bytes).
    t.ramp_set_stop_color(0, [255, 0, 0, 128]);
    let s = *t
        .texture_ramp()
        .stops()
        .iter()
        .find(|s| s.id == 0)
        .expect("stop id 0");
    assert!(
        (s.color[3] - 128.0 / 255.0).abs() < 1e-6,
        "alpha applied straight (was preserved-only before): {}",
        s.color[3]
    );
    assert!(s.color[0] > 0.9, "red channel high (linear of sRGB 255)");
}

/// End-to-end via the SAME dispatch the panel sends: enable ramp + a translucent stop + pick the
/// alpha mode through `handle_panel_event`, then paint a real stroke. Reproduces "I select a mode but
/// painting does not change" — proves the tool+engine path so a UI-side break is isolated.
#[test]
fn ramp_alpha_mode_dispatch_changes_the_painted_result() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::RampAlphaMode;
    use ph2d_painter_brush::texture::{TextureKind, TextureMapping};
    let make = |mode: &str| {
        let mut t = white_canvas(48, 16.0);
        // A checker texture so the ramped path engages (`texture_ramp_enabled && texture.is_active()`).
        t.paint.brush.texture.kind = TextureKind::Checker;
        t.paint.brush.texture.mapping = TextureMapping::ViewPlane;
        t.paint.brush.texture.size = [0.25, 0.25];
        t.set_texture_ramp_enabled(true);
        // Make the s=1 stop (id 1) fully transparent via the real swatch dispatch ("id,r,g,b,a").
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
            "1,255,255,255,0".into(),
        ));
        // Select the alpha action through the dropdown dispatch.
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
            mode.into(),
        ));
        // Paint one stroke across the middle.
        t.on_canvas_pointer(cp([8.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Up));
        t
    };
    let transparent = |t: &PainterTool| {
        (0..48 * 48)
            .filter(|&i| t.canvas_rgba[i * 4 + 3] < 30)
            .count()
    };

    // "2" = Sprite Alpha: the transparent ramp cells must punch the white sprite see-through.
    let sprite = make("2");
    assert_eq!(
        sprite.paint.texture_ramp_alpha_mode,
        RampAlphaMode::TextureAlpha,
        "the dropdown dispatch set the mode"
    );
    assert!(
        transparent(&sprite) > 0,
        "Sprite Alpha must make part of the sprite transparent"
    );
    // "0" = Off over the same setup leaves the sprite fully opaque (alpha ignored).
    assert_eq!(
        transparent(&make("0")),
        0,
        "Off ignores the ramp alpha — nothing is punched transparent"
    );
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
    t.paint.brush.texture.params[2] = 0.0; // hard checker (crisp 0/1 cells) — Softness slot
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

#[test]
fn enabled_color_ramp_paints_the_ramp_colours_through_the_tool() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 14.0);
    // Checker so some texels read 0 and some 1 across the footprint.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::ViewPlane,
        size: [0.25, 0.25],
        ..Default::default()
    };
    t.paint.brush.texture.params[2] = 0.0; // hard checker (crisp 0/1 cells) — Softness slot
    // Brush colour GREEN — it must NOT appear once the ramp drives the colour.
    t.set_brush_color_channel(0, 0.0);
    t.set_brush_color_channel(1, 1.0);
    t.set_brush_color_channel(2, 0.0);
    // Ramp: red at the 0-cells → blue at the 1-cells (linear stops; the tool bakes linear→sRGB).
    t.set_texture_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Constant,
    ));
    t.set_texture_ramp_enabled(true);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    let (mut red, mut blue, mut green) = (0, 0, 0);
    for y in 18..46 {
        for x in 18..46 {
            let [r, g, b, _] = px(&t, 64, x, y);
            if r > 200 && g < 60 && b < 60 {
                red += 1;
            } else if b > 200 && r < 60 && g < 60 {
                blue += 1;
            } else if g > 200 && r < 60 && b < 60 {
                green += 1;
            }
        }
    }
    assert!(
        red > 0 && blue > 0,
        "ramp paints red (checker 0) + blue (checker 1): red={red} blue={blue}"
    );
    assert_eq!(
        green, 0,
        "the brush's own green must not appear — the ramp drives the colour"
    );
}
