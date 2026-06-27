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
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAKE));
    assert!(t.brush_settings().shape_rake, "shape rake toggled on");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RANDOM));
    assert!(t.brush_settings().shape_random, "shape random toggled on");
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
    assert!(
        !s.shape_rake && !s.shape_random,
        "reset cleared rake/random"
    );
    assert_eq!(s.dab_flatten, 0.0, "reset cleared the dab flatten");
    assert_eq!(s.dab_angle_deg, 0, "reset cleared the dab angle");
}

#[test]
fn shape_source_dropdown_requests_image_and_clears_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::TextureKind;

    let mut t = PainterTool::default();
    // Picking "Image" in the Shape source dropdown requests a file load (the shell polls it); the engine
    // does no I/O, so the silhouette stays the falloff until pixels arrive. Mirrors the Grain Kind flow.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::Image.to_u8().to_string(),
    ));
    assert!(
        t.take_brush_shape_image_request(),
        "picking Image requests a Shape file load"
    );
    assert!(
        !t.take_brush_shape_image_request(),
        "the Shape request is consumed once"
    );
    assert!(
        !t.brush_settings().shape_has_image,
        "no pixels yet ⇒ silhouette is still the falloff"
    );

    // The shell delivers the pixels ⇒ shape_has_image flips (the dropdown then reads "Image").
    t.set_brush_shape_image(vec![255u8; 16], 4, 4);
    assert!(t.brush_settings().shape_has_image, "shape image assigned");

    // Picking "None" clears the image (→ falloff), the same as the section reset.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::None.to_u8().to_string(),
    ));
    assert!(
        !t.brush_settings().shape_has_image,
        "picking None cleared the shape image"
    );

    // Picking a PROCEDURAL kind installs that pattern (no pixels) — the panel's "Texture" picker.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::Checker.to_u8().to_string(),
    ));
    assert_eq!(
        t.brush_settings().shape_kind,
        TextureKind::Checker.to_u8(),
        "procedural Shape kind installed"
    );
    assert!(
        !t.brush_settings().shape_has_image,
        "a procedural Shape never holds pixels"
    );

    // The procedural Shape exposes the kind's per-pattern params (like the Grain): a SetValue on a
    // PAINTER_SHAPE_PARAMS slider tunes only the Shape pattern.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_PARAMS[0], 0.9));
    assert!(
        (t.brush_settings().shape_params[0] - 0.9).abs() < 1e-6,
        "Shape per-pattern param routed to the Shape slot"
    );
}

#[test]
fn procedural_shape_is_masked_by_the_falloff_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::TextureKind;

    // A soft round falloff so the envelope actually attenuates toward the dab edge.
    let mut a = white_canvas(64, 24.0);
    a.paint.brush.falloff = Falloff::Smooth;
    let _ = a.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));

    // Same brush + a procedural Checker Shape, selected via the panel "Texture" picker.
    let mut b = white_canvas(64, 24.0);
    b.paint.brush.falloff = Falloff::Smooth;
    b.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::Checker.to_u8().to_string(),
    ));
    let _ = b.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));

    // The procedural silhouette is `falloff × pattern ≤ falloff`, so the Checker dab deposits strictly
    // LESS total ink than the bare-falloff dab (the pattern carves out ~half), yet still paints. Total
    // coverage is the robust invariant (a per-pixel bound is foiled by the cached stamp's bilinear blit
    // of the sharp checker edge). Proves the masking end-to-end (panel "Texture" pick → engine).
    let ink = |t: &PainterTool| -> u64 {
        let mut s = 0u64;
        for yy in 0..64 {
            for xx in 0..64 {
                s += 255 - u64::from(px(t, 64, xx, yy)[0]); // darkness on white = deposited ink
            }
        }
        s
    };
    let (ink_falloff, ink_checker) = (ink(&a), ink(&b));
    assert!(ink_checker > 0, "the Checker Shape must still paint");
    assert!(
        ink_checker < ink_falloff * 9 / 10,
        "the falloff must MASK the Checker (less ink than the bare falloff): {ink_checker} vs {ink_falloff}"
    );
}

#[test]
fn shape_value_ramp_remaps_the_silhouette_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::TextureKind;

    let ink = |t: &PainterTool| -> u64 {
        let mut s = 0u64;
        for yy in 0..64 {
            for xx in 0..64 {
                s += 255 - u64::from(px(t, 64, xx, yy)[0]); // darkness on white = deposited ink
            }
        }
        s
    };

    // The Shape ramp acts as the B&W **tone** remap when its B&W filter is on — which auto-enables
    // when a Grain is assigned (Enio 2026-06-26) — so assign a Noise Grain in each case below.
    // White tip (silhouette 1) + identity ramp (luma(v)=v) ⇒ the tip paints under the Grain.
    let mut t2 = white_canvas(64, 12.0);
    t2.set_brush_shape_image(vec![255u8; 16], 4, 4);
    t2.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t2.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAMP_ENABLE));
    t2.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let ink_identity = ink(&t2);
    assert!(
        ink_identity > 0,
        "identity tone ramp still paints (with a Grain)"
    );

    // INVERT the ramp (white→black) ⇒ the value-1 tip maps to 0 BEFORE the Grain multiply ⇒ the tip is
    // zeroed: the centre stays pure white, and far less ink overall.
    let mut t3 = white_canvas(64, 12.0);
    t3.set_brush_shape_image(vec![255u8; 16], 4, 4);
    t3.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t3.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAMP_ENABLE));
    t3.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAMP_INVERT));
    t3.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    assert_eq!(
        px(&t3, 64, 32, 32),
        [255, 255, 255, 255],
        "the inverted tone ramp zeroes the white tip (before the Grain multiply)"
    );
    assert!(
        ink(&t3) < ink_identity / 2,
        "inverted tone ramp deposits far less ink: {} vs {ink_identity}",
        ink(&t3)
    );
}

#[test]
fn shape_colour_ramp_colourises_the_silhouette_when_grain_is_none() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};

    // No Grain ⇒ the SHAPE colour ramp OWNS the painted colour (B&W off): the silhouette coverage
    // indexes it (Enio 2026-06-26). A solid-red ramp over a BLUE brush base ⇒ a RED dab — proving the
    // Shape ramp colourises (without it the centre would be the brush's blue).
    let mut t = white_canvas(64, 12.0);
    t.set_brush_color_channel(2, 1.0); // brush base = blue [0,0,1]
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_ramp_enabled(true); // B&W stays off ⇒ the ramp colourises
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let c = px(&t, 64, 32, 32);
    assert!(
        c[0] > 200 && c[1] < 80 && c[2] < 80,
        "grain-None Shape colour ramp paints red (not the brush's blue): {c:?}"
    );
}

#[test]
fn resetting_the_shape_clears_the_per_layer_color_state() {
    // Reset OR removing the Shape image (dropdown → None) must drop the captured layers + the Per-Layer
    // Color mode, so the panel rows disappear AND a now-None Shape never routes into the coloured path
    // (which left it un-paintable). Both the section Reset and the kind→None dropdown are covered.
    for clear_via_kind in [false, true] {
        let mut t = white_canvas(64, 6.0);
        t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
        t.toggle_brush_shape_per_layer_color();
        let on = t.brush_settings();
        assert!(
            on.shape_layer_count == 2 && on.shape_per_layer_color,
            "armed"
        );
        if clear_via_kind {
            t.set_brush_shape_kind(0); // TextureKind::None — "remove from the slot"
        } else {
            t.reset_brush_shape(); // the Shape section Reset button
        }
        let off = t.brush_settings();
        assert_eq!(off.shape_layer_count, 0, "captured layers dropped");
        assert!(!off.shape_per_layer_color, "Per-Layer Color mode dropped");
        // And painting still works — a plain dab lands (no stale coloured-path routing).
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        let c = px(&t, 64, 32, 32);
        assert!(c[3] > 0, "a normal dab still paints after the reset: {c:?}");
    }
}

#[test]
fn per_layer_color_top_layer_paints_above_all_lower_painting_across_the_stroke() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // 2-layer Shape: layer 0 (bottom) = a full square, layer 1 (top) = its RIGHT half only. Colours red
    // bottom, green top. Two overlapping dabs (B to the right of A) emitted in SEPARATE `stamp_dabs`
    // calls — exactly how a real freehand stroke arrives (one batch per pointer move). At a pixel inside
    // A's right-half (green) that B's left-half (red bottom, no green) re-covers, a direct per-dab
    // composite lets B's later red bury A's green (only the tip's highlight survives, worse the slower
    // the stroke). The per-stroke accumulate + recomposite keeps it GREEN across batches (Enio 2026-06-26).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space; // incremental → accumulate across batches
    let full = vec![255u8; 64]; // 8×8 full coverage (the body)
    let mut right = vec![0u8; 64]; // 8×8, right half = 255 (the highlight)
    for row in 0..8 {
        for col in 4..8 {
            right[row * 8 + col] = 255;
        }
    }
    t.set_brush_shape_layers(vec![(full, 8, 8), (right, 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // bottom = red
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]); // top = green
    let dab = |cx: f32| Dab {
        center: [cx, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab(20.0)]); // batch 1 (dab A)
    t.stamp_dabs(&[dab(26.0)]); // batch 2 (dab B overlapping A's right half from the left)
    let [r, g, b, _] = px(&t, 64, 22, 32); // inside A's right-half-green, re-covered by B's left-half
    assert!(
        g > 200 && r < 80,
        "the top (green) layer survives the lower (red) layer across batches: {:?}",
        [r, g, b]
    );
}

#[test]
fn per_layer_color_respects_brush_blend_mode() {
    use ph2d_painter_brush::{BrushBlend, Dab, StrokeMethod};
    // The per-layer-colour tip must blend onto the canvas via the **Brush blend mode** (applied to the
    // whole composite, once). On a 50% grey canvas a solid RED tip with Multiply yields ~half-red
    // (grey×red), NOT the pure red that Normal gives — the old per-layer `blend_over` mis-applied it.
    let mut t = white_canvas(64, 6.0);
    t.set_source(vec![128u8; 64 * 64 * 4], 64, 64); // 50% grey
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.blend = BrushBlend::Multiply;
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [1.0, 0.0, 0.0]); // solid red composite
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab]);
    let [r, g, b, _] = px(&t, 64, 32, 32);
    assert!(
        (100..=150).contains(&r) && g < 30 && b < 30,
        "Multiply grey×red is ~half-red, not pure red: {:?}",
        [r, g, b]
    );
}

#[test]
fn per_layer_color_dynamic_randomize_color_tints_per_dab() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Randomize Color on → the DYNAMIC per-layer path, which tints by each dab's own `d.color`. Two
    // non-overlapping dabs carrying red / blue paint red / blue (the static cached path baked one colour
    // for the whole stroke and would ignore `d.color`).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color_jitter_hue = 0.5; // Randomize Color active (amount > 0) → routes to the dynamic path
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color(); // layers un-coloured → they take the per-dab base colour
    let dab = |cx: f32, col: [f32; 3]| Dab {
        center: [cx, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: col,
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab(16.0, [1.0, 0.0, 0.0])]); // red dab
    t.stamp_dabs(&[dab(48.0, [0.0, 0.0, 1.0])]); // blue dab
    let red = px(&t, 64, 16, 32);
    let blue = px(&t, 64, 48, 32);
    assert!(red[0] > 200 && red[2] < 80, "first dab is red: {red:?}");
    assert!(
        blue[2] > 200 && blue[0] < 80,
        "second dab is blue: {blue:?}"
    );
}

#[test]
fn per_layer_color_dynamic_shape_random_angle_paints() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Shape Random Angle + per-layer-colour routes to the dynamic path (per-dab rotation). Guard that it
    // runs and paints (the cached path silently ignored the rotation).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.shape.random_angle = true; // routes to the dynamic path
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.2, 0.4, 0.6],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab]);
    assert!(
        px(&t, 64, 32, 32)[3] > 0,
        "a random-angle per-layer-colour dab paints"
    );
}

#[test]
fn per_layer_color_grain_random_angle_routes_dynamic_and_paints() {
    use ph2d_painter_brush::{Dab, StrokeMethod, TextureKind};
    // Grain Rake / Random Angle must work in per-layer-colour — the route used to check only Grain
    // Jitter-Rotate, so Grain Rake/Random fell to the constant-orientation cached path. With Grain Random
    // on, the dynamic path (per-dab Grain basis) runs and paints.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.texture.kind = TextureKind::Checker; // an active Grain
    t.paint.brush.texture.random_angle = true; // Grain Random Angle
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.2, 0.4, 0.6],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab]);
    assert!(
        px(&t, 64, 32, 32)[3] > 0,
        "a Grain-random-angle per-layer-colour dab paints"
    );
}

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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
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
        px(&t, 64, 32, 32)[0] < 200,
        "the stroke was baked by the Click"
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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // 3-point curve along y=32
    assert!(t.curve_overlay().is_some(), "a curve editor is open");
    assert_eq!(
        px(&t, 64, 32, 25),
        [255, 255, 255, 255],
        "9px above the thin line is white before growing the brush"
    );
    // Grow the brush — routed in the match arm, which re-fills the open shape.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_SIZE_SLIDER,
        0.6,
    ));
    assert_ne!(
        px(&t, 64, 32, 25),
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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // curve along y=32, full strength
    assert!(
        px(&t, 64, 32, 32)[0] < 200,
        "the curve painted at full strength"
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
        px(&t, 64, 32, 32)[0] < 180,
        "reducing Strength must keep the curve painted (not erase to white): {:?}",
        px(&t, 64, 32, 32)
    );
}

#[test]
fn dragging_a_tangent_handle_reshapes_the_curve_and_mirrors_the_opposite() {
    use ph2d_painter_brush::StrokeMethod;
    // Gizmo (Enio 2026-06-27): the selected anchor exposes draggable Bézier tangent handles. Grabbing the
    // OUT handle (off the point) and pulling it must move that handle (not the anchor) and swing the IN
    // handle to stay aligned (collinear through the anchor) — the standard smooth-handle behaviour.
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
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
    assert!((anchor[1] - 32.0).abs() < 1e-3, "midpoint sits on y=32");
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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
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
    // The Edit (E) button turns an open Circle into an editable Bézier curve: the circle editor closes, a
    // curve editor opens (with the closing anchor so it reads closed), and the method switches to Curve so
    // pointers route to the curve editor (Enio 2026-06-27).
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Circle;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(t.circle_overlay().is_some(), "a circle editor is open");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_EDIT));
    assert!(t.circle_overlay().is_none(), "the circle editor closed");
    let ov = t.curve_overlay().expect("a curve editor opened");
    assert_eq!(
        t.brush_settings().stroke_method,
        StrokeMethod::Curve.to_u8(),
        "method is now Curve"
    );
    assert!(
        ov.points.len() >= 4,
        "circle → at least the 4 cardinal anchors"
    );
    assert_eq!(
        *ov.points.first().unwrap(),
        *ov.points.last().unwrap(),
        "closing anchor = a closed loop"
    );
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
    assert_eq!(
        *ov.points.first().unwrap(),
        *ov.points.last().unwrap(),
        "closed loop"
    );
    assert!(ov.points.len() >= 4, "polygon vertices became anchors");
}

#[test]
fn delete_button_drops_the_open_shape_without_baking() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // The trash button cancels the open shape editor WITHOUT baking it — the canvas stays pristine.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up)); // released → edit mode, control points
    assert!(t.curve_overlay().is_some(), "a curve editor is open");
    assert!(t.commit_open_shape_keep(), "Apply & Keep ran");
    assert!(
        t.curve_overlay().is_some(),
        "the editable curve persists after Apply & Keep"
    );
    assert!(px(&t, 64, 32, 32)[0] < 200, "the stroke was baked");
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

#[test]
fn per_layer_color_fill_method_uses_canvas_base_and_self_clears() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Fill methods (Line/Curve/Circle/Polygon) take the no-snapshot / self-clearing per-layer path: the
    // canvas is the recomposite base (the drag preview restores it to the pre-shape each move) and the
    // maps self-clear, so there's no per-move full-canvas clone + N-map re-allocation (the FPS fix). Two
    // full layers (red bottom, green top) → green on top; re-stamping the identical fill onto the same
    // restored canvas must be STABLE — proving the maps self-cleared and the canvas-base didn't double-
    // composite (a stale-map or double-composite bug would change the second result).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Line; // a fill method → the non-incremental path
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // bottom red
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]); // top green
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    let pristine = (*t.canvas_rgba).clone(); // the pre-shape the drag preview restores to
    t.stamp_dabs(&[dab]); // first fill
    let a = px(&t, 64, 32, 32);
    *std::sync::Arc::make_mut(&mut t.canvas_rgba) = pristine; // emulate the drag-preview restore
    t.stamp_dabs(&[dab]); // re-fill the identical shape onto the restored canvas
    let b = px(&t, 64, 32, 32);
    assert!(
        a[1] > 200 && a[0] < 80,
        "the top (green) layer wins on the fill: {a:?}"
    );
    assert_eq!(
        a, b,
        "re-filling the restored canvas is stable (maps self-clear, canvas as base)"
    );
}

#[test]
fn dab_bbox_covers_the_paint_write_bounds() {
    // Regression (Enio 2026-06-27): `dab_bbox` is the drag-preview SAVE/RESTORE + dirty-upload region for
    // the fill methods. It MUST be a superset of every paint path's write bounds — `floor(c−r)..ceil(c+r)+1`
    // (the blit/accumulate loop) — or an edge row can paint outside the saved region and never get restored
    // (a CPU trail) / never get re-uploaded (a stale row on the upscaled GPU texture: the thin horizontal
    // lines). The old `round(c)±(ceil(r)+1)` box violated this by 1px for fractional centres (e.g. c=0.4,
    // r=1.7). This pins the invariant directly.
    let t = white_canvas(64, 3.0);
    for &c in &[0.1f32, 0.4, 0.5, 0.6, 0.9, 12.3, 31.5, 47.7] {
        for &r in &[1.0f32, 1.7, 2.5, 3.0, 5.5, 8.2] {
            let want_x0 = (c - r).floor().max(0.0) as i64;
            let want_x1 = ((c + r).ceil() as i64 + 1).min(64);
            let bb = t.dab_bbox([c, c], r).expect("dab in-canvas has a bbox");
            assert!(
                (bb.x as i64) <= want_x0 && (bb.x + bb.w) as i64 >= want_x1,
                "dab_bbox x [{},{}) must cover paint bounds [{want_x0},{want_x1}) for c={c} r={r}",
                bb.x,
                bb.x + bb.w
            );
        }
    }
}

#[test]
fn line_per_layer_color_moving_endpoint_leaves_no_trail() {
    use ph2d_painter_brush::StrokeMethod;
    // Draw a Line (press at A, drag the endpoint around) with Per-Layer Color on. Each move re-stamps the
    // whole line via the drag-preview restore; an earlier endpoint position must be fully restored — no
    // thin trail survives along where the line used to be (Enio 2026-06-27). Drives the real pointer path.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8)]); // full square silhouette → coverage to the edge
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    let a = [10.0, 31.0];
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    // Sweep the endpoint to several positions (the line pivots around A), then settle near A.
    for b in [[52.0, 12.0], [52.0, 50.0], [52.0, 31.0], [16.0, 31.0]] {
        t.on_canvas_pointer(cp(b, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([16.0, 31.0], PointerPhase::Up));
    // The final line is the short A=(10,31)→(16,31) segment (y≈31, x≈7..19). Any painted pixel well away
    // from it (e.g. y<24 or y>38, or x>26) is a trail from an earlier endpoint the move failed to restore.
    let mut trail = Vec::new();
    for y in 0..64u32 {
        for x in 0..64u32 {
            let far = y < 24 || y > 38 || x > 26;
            if far && px(&t, 64, x, y) != [255, 255, 255, 255] {
                trail.push((x, y));
            }
        }
    }
    assert!(
        trail.is_empty(),
        "moving the Line endpoint left a trail at {} pixels, e.g. {:?}",
        trail.len(),
        &trail[..trail.len().min(8)]
    );
}

#[test]
fn per_layer_color_randomize_jitters_custom_layer_colours() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Randomize Color must jitter the per-layer CUSTOM colours too (the artist's case), not only the
    // un-coloured layers. Brush base grey; both layers a custom green. Two dabs with different `d.color`
    // shift the green by different HSV offsets → the two locations differ (the path used to ignore the
    // custom colours, so Randomize Color had no effect).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color = [0.5, 0.5, 0.5];
    t.paint.brush.color_jitter_hue = 0.5; // Randomize Color active (amount > 0)
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [0.0, 1.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    let dab = |cx: f32, col: [f32; 3]| Dab {
        center: [cx, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: col,
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab(16.0, [1.0, 0.0, 0.0])]);
    t.stamp_dabs(&[dab(48.0, [0.0, 0.0, 1.0])]);
    let a = px(&t, 64, 16, 32);
    let b = px(&t, 64, 48, 32);
    assert_ne!(a, b, "custom layer colours jitter per dab: {a:?} vs {b:?}");
}

#[test]
fn editing_the_shape_source_re_captures_and_keeps_colours() {
    use ph2d_painter_brush::StrokeMethod;
    // Capture a multi-layer sprite as the Shape + colour layer 0; painting on that SAME sprite (the Shape
    // source) auto-re-captures the Shape at pointer-up WITHOUT wiping the colours (no manual re-assign).
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
    t.layers.add_raster("L2", 64, 64); // make it multi-layer
    t.capture_layers_as_brush_shape();
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // red on layer 0
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up));
    t.refresh_shape_source_if_changed(); // the bridge calls this each frame; the paint changed the source
    let s = t.brush_settings();
    assert!(
        s.shape_layer_color_on[0],
        "layer 0 stays coloured after the auto re-capture"
    );
    assert_eq!(
        s.shape_layer_color[0],
        [1.0, 0.0, 0.0],
        "the per-layer red is preserved across the re-capture"
    );
    assert!(
        s.shape_per_layer_color,
        "per-layer-colour mode survives the re-capture"
    );
}

#[test]
fn changing_source_layer_opacity_re_captures_the_shape() {
    // Editing the reference sprite WITHOUT painting — here a layer's opacity — must still update the brush
    // Shape. The per-frame revision poll catches opacity / visibility / undo, not only paint strokes.
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
    let l2 = t.layers.add_raster("L2", 64, 64).expect("add layer");
    t.capture_layers_as_brush_shape();
    let ver0 = t.brush_shape_image_version();
    t.set_layer_opacity(l2, 0.5); // edit the source, no painting
    t.refresh_shape_source_if_changed();
    assert_ne!(
        t.brush_shape_image_version(),
        ver0,
        "an opacity change on the source re-captures the Shape"
    );
}

#[test]
fn shape_ramp_swatch_select_option_sets_the_stop_colour() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    // The Shape ramp swatch picker forwards `"id,r,g,b,a"` (sRGB bytes) to PAINTER_SHAPE_RAMP_SWATCH;
    // the tool sets THAT stop's colour. Stop id 0 defaults to black → drive it to pure red.
    let mut t = PainterTool::default();
    let id0 = t.shape_color_ramp().stops()[0].id;
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_RAMP_SWATCH,
        format!("{id0},255,0,0,255"),
    ));
    let s0 = *t
        .shape_color_ramp()
        .stops()
        .iter()
        .find(|s| s.id == id0)
        .unwrap();
    assert!(
        s0.color[0] > 0.9 && s0.color[1] < 0.1 && s0.color[2] < 0.1,
        "swatch set stop {id0} to red, got {:?}",
        s0.color
    );
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
fn shape_image_paints_the_silhouette_end_to_end() {
    // A full-white 4×4 Shape image makes the dab a SQUARE silhouette: a footprint corner that the round
    // falloff disc would leave blank gets painted. Proves the tool routes the Shape slot end-to-end
    // (set image → stamp route → engine composition), not just the unit sampler.
    let size = 16u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(6.0);
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.falloff = Falloff::Smooth; // a SOFT disc — the corner is far below the rim

    // Control: no Shape image ⇒ the corner (3,3) stays white (round disc).
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 3, 3),
        [255, 255, 255, 255],
        "round falloff leaves the corner blank"
    );

    // Assign the square Shape image and paint again ⇒ the corner is now painted (square silhouette).
    let mut t2 = PainterTool::default();
    t2.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t2.set_brush_size_px(6.0);
    t2.paint.brush.color = [0.0, 0.0, 0.0];
    t2.paint.brush.falloff = Falloff::Smooth;
    t2.set_brush_shape_image(vec![255u8; 16], 4, 4); // 4×4 all-white → full-square silhouette
    t2.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t2.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    let corner = px(&t2, size, 3, 3);
    assert!(
        corner[0] < 80,
        "square shape paints the footprint corner (got {corner:?})"
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
fn unbaked_edits_tracked_and_deactivate_defers_the_bake() {
    // Persistence (Enio 2026-06-24): the painter flags unbaked edits so the shell auto-persists them
    // on leave/deactivate. A fresh bind has none; an edit sets the flag; deactivating with edits KEEPS
    // the canvas + defers the bake (so the shell can write it back before teardown).
    use ph2d_editor_core::tool::{RasterEditTool, Tool};
    let mut t = PainterTool::default();
    (&mut t as &mut dyn RasterEditTool).set_source(vec![0u8; 4 * 4 * 4], 4, 4);
    assert!(!t.has_unbaked_edits(), "fresh bind has no unbaked edits");

    // A structural edit (add a layer) marks the canvas unbaked.
    t.add_raster_layer("Layer 2");
    assert!(t.has_unbaked_edits(), "an edit flags unbaked work");

    // Deactivating with unbaked edits defers the bake + keeps the canvas for the shell.
    t.on_deactivate();
    assert!(
        t.take_deferred_bake(),
        "deactivate defers the bake when edited"
    );
    assert!(
        t.has_unbaked_edits(),
        "canvas kept until the shell bakes it"
    );

    // The shell signals the bake landed.
    t.mark_baked();
    assert!(!t.has_unbaked_edits());
}

#[test]
fn deactivate_without_edits_tears_down_immediately() {
    use ph2d_editor_core::tool::{RasterEditTool, Tool};
    let mut t = PainterTool::default();
    (&mut t as &mut dyn RasterEditTool).set_source(vec![0u8; 4 * 4 * 4], 4, 4);
    t.on_deactivate();
    assert!(!t.take_deferred_bake(), "no edits → no deferred bake");
    assert!(!t.has_unbaked_edits());
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
        brush_settings::snap_to_45(a, [10.0, 1.0]),
        [10.0, 0.0],
        "near-horizontal → flat"
    );
    assert_eq!(
        brush_settings::snap_to_45(a, [1.0, 10.0]),
        [0.0, 10.0],
        "near-vertical → vertical"
    );
    assert_eq!(
        brush_settings::snap_to_45(a, [-1.0, 10.0]),
        [0.0, 10.0],
        "sign of the cursor picks the ray"
    );
    let d = brush_settings::snap_to_45(a, [10.0, 9.0]); // near-diagonal
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

/// Timing: the FULL per-move tool cost of an Anchored size-drag (restore + save + stamp), plain vs
/// textured, on a large canvas. Tells us where the per-move CPU goes. Run:
/// `cargo test -p ph2d-tool-painter --release perf_anchored -- --ignored --nocapture`.
#[test]
#[ignore]
fn perf_anchored_drag_per_move_cost() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    use std::time::Instant;
    // `hold_preview` simulates the shell bridge retaining the preview Arc across frames (it drains
    // `take_preview_arc` each frame and keeps it). With it held, the tool's next mutation hits
    // refcount=2 → `Arc::make_mut` deep-clones the whole 16.8MB canvas EVERY move. That clone is
    // invisible to a bench that doesn't hold the Arc — the bench-vs-live gap.
    let run = |label: &str, kind: TextureKind, mapping: TextureMapping, hold_preview: bool| {
        let mut t = white_canvas(2048, 10.0);
        t.paint.brush.texture = TextureSettings {
            kind,
            mapping,
            ..Default::default()
        };
        t.set_brush_stroke_method(StrokeMethod::Anchored.to_u8());
        let _ = t.on_canvas_pointer(cp([1024.0, 1024.0], PointerPhase::Down));
        let moves = 20u32;
        let mut held = None;
        let t0 = Instant::now();
        for k in 1..=moves {
            let r = 60.0 + k as f32 * 45.0; // radius grows to ~960 px
            let _ = t.on_canvas_pointer(cp([1024.0, 1024.0 + r], PointerPhase::Move));
            if hold_preview {
                held = t.take_preview_arc(); // retain across the next move (bridge behaviour)
            }
        }
        let _ = held;
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(moves);
        eprintln!("  anchored {label:<22} {ms:6.2} ms/move");
    };
    eprintln!(
        "perf: Anchored size-drag on 2048², radius→960 px, per-move tool cost (preview held):"
    );
    run("plain", TextureKind::None, TextureMapping::ViewPlane, true);
    run(
        "voronoi View (cached)",
        TextureKind::Voronoi,
        TextureMapping::ViewPlane,
        true,
    );
    run(
        "voronoi Tiled (cached)",
        TextureKind::Voronoi,
        TextureMapping::Tiled,
        true,
    );
    run(
        "noise Tiled (cached)",
        TextureKind::Noise,
        TextureMapping::Tiled,
        true,
    );
}

#[test]
fn anchored_textured_stroke_commits_a_textured_result() {
    // Perf fix: the interactive Anchored preview stamps texture-FREE (fast), then re-applies the
    // texture once on pen-up. Assert the COMMITTED result is still textured — a hard Checker dab
    // leaves a mix of painted (black) and masked (white) pixels in its footprint.
    use ph2d_painter_brush::{StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(96, 6.0);
    t.set_brush_texture_kind(TextureKind::Checker.to_u8());
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::ViewPlane,
        size: [0.2, 0.2], // big cells across the anchored footprint
        ..Default::default()
    };
    t.set_brush_stroke_method(StrokeMethod::Anchored.to_u8());
    // Anchored: press at the centre, drag out (radius = drag distance), release.
    let _ = t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    let _ = t.on_canvas_pointer(cp([48.0, 78.0], PointerPhase::Move)); // radius ≈ 30
    let _ = t.on_canvas_pointer(cp([48.0, 78.0], PointerPhase::Up));
    // Scan the footprint for both fully-painted (black) and masked (white) pixels.
    let (mut black, mut white) = (0, 0);
    for y in 20..76 {
        for x in 20..76 {
            match px(&t, 96, x, y) {
                [0, 0, 0, 255] => black += 1,
                [255, 255, 255, 255] => white += 1,
                _ => {}
            }
        }
    }
    assert!(black > 0, "the committed Anchored dab painted");
    assert!(
        white > 0,
        "the committed Anchored dab is textured (checker masked some texels)"
    );
}

#[test]
fn stencil_overlay_outlines_the_rect_only_for_stencil() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 10.0);
    // No texture → no overlay.
    assert!(t.stencil_overlay().is_none());
    // A texture but a non-Stencil mapping → still no overlay.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::ViewPlane,
        ..Default::default()
    };
    assert!(t.stencil_overlay().is_none());
    // Stencil, centred, full-canvas size (stencil_size 1), no rotation → corners at the canvas corners.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0],
        ..Default::default()
    };
    let o = t.stencil_overlay().expect("stencil overlay present");
    assert_eq!(
        o.corners,
        [[0.0, 0.0], [64.0, 0.0], [64.0, 64.0], [0.0, 64.0]],
        "centred full-canvas stencil outlines the whole canvas"
    );
    // The DEFAULT stencil (stencil_size 0.5) is a centred rect at 50 % of the sprite.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        ..Default::default()
    };
    assert_eq!(
        t.stencil_overlay().expect("overlay").corners,
        [[16.0, 16.0], [48.0, 16.0], [48.0, 48.0], [16.0, 48.0]],
        "the default stencil rect is 50% of the sprite"
    );
}

#[test]
fn stencil_dab_paints_only_inside_the_rect() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    // A hard black dab whose Stencil rect covers only the centre: a corner well outside the rect
    // stays white (masked), proving the engine mask reaches the canvas via stamp_dabs.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.4, 0.4], // a central rect ≈ [19.2 .. 44.8]
        ..Default::default()
    };
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert_eq!(
        px(&t, 64, 2, 2),
        [255, 255, 255, 255],
        "a corner outside the stencil rect is untouched"
    );
}

/// A `PainterTool` with a centred full-canvas Stencil texture (handles at the canvas corners +
/// centre), ready for the drag-gesture tests.
fn stencil_tool() -> PainterTool {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0], // stencil_offset 0, stencil_size 1 → rect = whole canvas
        ..Default::default()
    };
    t
}

#[test]
fn stencil_centre_handle_drag_moves_the_rect() {
    let mut t = stencil_tool();
    let center = t.stencil_overlay().expect("overlay").center; // (32, 32)
    assert!(
        t.on_canvas_pointer(cp(center, PointerPhase::Down)),
        "grab the centre handle"
    );
    let _ = t.on_canvas_pointer(cp([40.0, 36.0], PointerPhase::Move));
    let _ = t.on_canvas_pointer(cp([40.0, 36.0], PointerPhase::Up));
    let s = t.brush_settings();
    // new centre (40,36) → stencil_offset = (px/64*2 − 1) = (0.25, 0.125). The gizmo writes the
    // dedicated stencil field, NOT the texture offset.
    assert!(
        (s.stencil_offset[0] - 0.25).abs() < 1e-3,
        "x {}",
        s.stencil_offset[0]
    );
    assert!(
        (s.stencil_offset[1] - 0.125).abs() < 1e-3,
        "y {}",
        s.stencil_offset[1]
    );
    assert_eq!(
        s.texture_offset,
        [0.0, 0.0],
        "texture offset untouched by the gizmo"
    );
}

#[test]
fn stencil_corner_handle_drag_resizes_the_rect() {
    let mut t = stencil_tool();
    // Grab the bottom-right corner (64, 64) and drag in to (48, 48).
    assert!(
        t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down)),
        "grab a corner handle"
    );
    let _ = t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Move));
    let _ = t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    let s = t.brush_settings();
    // half = |(48,48) − centre(32,32)| = 16 each → stencil_size = 2·16/64 = 0.5.
    assert!(
        (s.stencil_size[0] - 0.5).abs() < 1e-3,
        "x {}",
        s.stencil_size[0]
    );
    assert!(
        (s.stencil_size[1] - 0.5).abs() < 1e-3,
        "y {}",
        s.stencil_size[1]
    );
    assert_eq!(
        s.texture_size,
        [1.0, 1.0],
        "texture size untouched by the gizmo"
    );
}

#[test]
fn stencil_corner_ring_drag_rotates_the_rect() {
    let mut t = stencil_tool();
    t.set_shape_grab_tol_px(5.0); // scale ≤ 5 px; the rotate ring is 5..13 px past a corner
    // A point just OUTSIDE the bottom-right corner (64, 64): in the rotate ring, not the scale disc.
    let down = [70.0, 70.0]; // dist from the corner ≈ 8.5 px
    assert!(
        t.on_canvas_pointer(cp(down, PointerPhase::Down)),
        "grab the rotate ring just outside a corner"
    );
    assert!(
        t.stencil_overlay().expect("overlay").rotating,
        "the active grab is a rotation (square→circle cue)"
    );
    // Drag from 45° to 135° about the centre (32, 32) → +90°.
    let _ = t.on_canvas_pointer(cp([-6.0, 70.0], PointerPhase::Move));
    let deg = i32::from(t.brush_settings().stencil_angle_deg);
    assert!((deg - 90).abs() <= 2, "stencil rotated ~90°, got {deg}");
    let _ = t.on_canvas_pointer(cp([-6.0, 70.0], PointerPhase::Up));
    assert_eq!(
        t.brush_settings().texture_angle_deg,
        0,
        "the texture's own angle is untouched by the gizmo"
    );
}

#[test]
fn stencil_preview_shows_during_transform_and_fades_when_idle() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = stencil_tool();
    assert!(t.stencil_preview().is_none(), "no preview when idle");
    // A panel param change (Stencil card) arms the transient in-gizmo preview.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_ANGLE,
        30.0,
    ));
    assert!(
        t.stencil_preview().is_some(),
        "preview shows after a stencil param change"
    );
    // It fades after the hold window (decayed by the per-frame tick).
    t.paint_tick(1.0); // 1 s ≫ the hold
    assert!(
        t.stencil_preview().is_none(),
        "preview fades once the user stops changing params"
    );
    // A handle drag shows it live; releasing hides it crisply.
    let center = t.stencil_overlay().expect("overlay").center;
    let _ = t.on_canvas_pointer(cp(center, PointerPhase::Down));
    assert!(
        t.stencil_preview().is_some(),
        "preview shows during a handle drag"
    );
    let _ = t.on_canvas_pointer(cp(center, PointerPhase::Up));
    assert!(
        t.stencil_preview().is_none(),
        "preview hides the moment the drag ends"
    );
}

#[test]
fn shape_colour_ramp_paints_cached_and_colourises_the_silhouette() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    // The no-Grain Shape Colour Ramp blits the cached coverage mask applying `ramp[coverage]` (the
    // fast path) — this proves it colourises correctly. With a Shape silhouette + Strength 1 + no Grain
    // + no per-dab rotation + B&W off, the router takes `stamp_dabs_cached_ramped`.
    let mut t = white_canvas(64, 16.0);
    t.set_brush_shape_image(vec![255u8; 16], 4, 4); // full-coverage silhouette ⇒ cacheable
    t.set_brush_strength(1.0); // no Accumulate cap ⇒ keeps the cacheable path
    // Shape colour ramp ON (B&W off), red high stop (full coverage → top colour).
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_ramp_enabled(true);

    // Paint a dab; the full-coverage silhouette centre takes the top ramp colour (red).
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let c = px(&t, 64, 32, 32);
    assert!(
        c[0] > 150 && c[1] < 90 && c[2] < 90,
        "Shape colour-ramp centre is ramp red via the cached blit, got {c:?}"
    );
}

#[test]
fn grain_assign_auto_enables_shape_bw_and_resets_the_grain_ramp() {
    use ph2d_painter_brush::TextureKind;
    let mut t = white_canvas(64, 10.0);
    // A coloured Shape ramp (no Grain yet): enable it, B&W off, so it owns colour.
    t.set_shape_ramp_enabled(true);
    assert!(
        !t.shape_ramp_bw(),
        "Shape ramp starts as a colour ramp (B&W off)"
    );
    // A coloured GRAIN ramp too, enabled — so we can observe it reset.
    t.toggle_texture_ramp_enabled();
    assert!(t.brush_settings().texture_ramp_enabled);

    // Assign a Grain texture → the Shape ramp's B&W auto-enables (it becomes the tone), and the (now
    // Grain-owned) colour ramp resets to its default off state (Enio 2026-06-26).
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    let b = t.brush_settings();
    assert!(
        b.shape_color_ramp_bw,
        "assigning a Grain auto-enabled the Shape ramp's B&W (tone) filter"
    );
    assert!(
        b.shape_color_ramp_enabled,
        "the Shape ramp stays enabled (now as tone)"
    );
    assert!(
        !b.texture_ramp_enabled && !b.texture_ramp_bw,
        "assigning a Grain reset the Grain colour ramp to defaults"
    );
}

#[test]
fn texture_image_request_then_modulates_the_dab() {
    use ph2d_painter_brush::TextureKind;
    let mut t = white_canvas(64, 12.0);
    // Picking the Image kind requests a file load (the shell polls this); consumed once.
    t.set_brush_texture_kind(TextureKind::Image.to_u8());
    assert!(
        t.take_brush_texture_image_request(),
        "picking Image requests a file load"
    );
    assert!(
        !t.take_brush_texture_image_request(),
        "the request is consumed once"
    );
    // All-black luminance → mask 0 → the dab paints nothing.
    t.set_brush_texture_image(vec![0u8; 16], 4, 4);
    assert_eq!(t.brush_settings().texture_kind, TextureKind::Image.to_u8());
    let _ = t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "an all-black image mask paints nothing"
    );
    // All-white luminance → mask 1 → paints fully (hard brush → black centre).
    t.set_brush_texture_image(vec![255u8; 16], 4, 4);
    let _ = t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 64, 20, 20),
        [0, 0, 0, 255],
        "an all-white image mask paints fully"
    );
}

#[test]
fn stencil_down_away_from_handles_paints_not_edits() {
    let mut t = stencil_tool();
    let before = t.brush_settings();
    // (16, 8) is well clear of every handle (corners + centre) → no grab → it paints.
    let _ = t.on_canvas_pointer(cp([16.0, 8.0], PointerPhase::Down));
    let after = t.brush_settings();
    assert_eq!(
        before.stencil_offset, after.stencil_offset,
        "a Down away from handles must not move the stencil"
    );
    assert_eq!(
        before.stencil_size, after.stencil_size,
        "a Down away from handles must not resize the stencil"
    );
    assert!(t.dirty_rect.is_some(), "it painted instead of editing");
}

#[test]
fn stencil_card_panel_events_drive_the_stencil_frame_not_the_texture() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::{TEX_SIZE_MAX, TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        ..Default::default()
    };
    // The Stencil card's number boxes write the REAL value to the dedicated stencil_* fields.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_X,
        0.3,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_OFFSET_Y,
        -0.5,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_ANGLE,
        90.0,
    ));
    let s = t.brush_settings();
    assert!(
        (s.stencil_size[0] - 0.3).abs() < 1e-6,
        "{}",
        s.stencil_size[0]
    );
    assert!(
        (s.stencil_offset[1] + 0.5).abs() < 1e-6,
        "{}",
        s.stencil_offset[1]
    );
    assert_eq!(s.stencil_angle_deg, 90);
    // The texture tiling is independent state — the card leaves it alone.
    assert_eq!(s.texture_size, [1.0, 1.0], "texture size untouched");
    assert_eq!(s.texture_offset, [0.0, 0.0], "texture offset untouched");
    assert_eq!(s.texture_angle_deg, 0, "texture angle untouched");
    // Real-value clamp to the size bound (not a 0..1 remap).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_X,
        999.0,
    ));
    assert!((t.brush_settings().stencil_size[0] - TEX_SIZE_MAX).abs() < 1e-6);
}

// ── Texture layers (LayerKind::Texture) — end-to-end through the panel-event path ──

/// `true` when the RGBA buffer is not a flat fill (the texture produced spatial variation).
fn buf_varies(b: &[u8]) -> bool {
    b.chunks_exact(4).any(|p| p != &b[0..4])
}

#[test]
fn texture_layer_renders_composites_and_edits_live_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::TextureKind;

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32); // opaque white base

    // Add a Texture layer: it becomes active, with its rendered pixels in `canvas_rgba`.
    let id = t.add_texture_layer().expect("texture layer added");
    assert!(
        matches!(
            t.layers().get(id).map(|l| &l.kind),
            Some(LayerKind::Texture(_))
        ),
        "the new layer is a Texture layer"
    );
    assert_eq!(t.layers().active(), Some(id), "the texture layer is active");
    let buf_default = t.canvas_rgba.as_ref().clone();
    assert_eq!(buf_default.len(), 32 * 32 * 4);
    assert!(
        buf_varies(&buf_default),
        "the default texture fills with variation"
    );

    // It composites like a raster (non-trivial stack → the texture covers the white base).
    let (composite, _, _) = t.run_full();
    assert!(
        buf_varies(&composite),
        "the composite shows the texture over the base"
    );

    // Live edit through the FROZEN panel-event channel — change the kind. The active layer is a
    // texture layer, so the tool routes the texture widget to it (not the brush).
    let brush_kind_before = t.brush_settings().texture_kind;
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        TextureKind::Checker.to_u8().to_string(),
    ));
    let buf_checker = t.canvas_rgba.as_ref().clone();
    assert_ne!(
        buf_default, buf_checker,
        "changing the kind re-rendered the layer live"
    );
    match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => assert_eq!(tex.kind, TextureKind::Checker.to_u8()),
        _ => panic!("layer should still be a Texture layer"),
    }
    assert_eq!(
        t.brush_settings().texture_kind,
        brush_kind_before,
        "the edit routed to the LAYER, leaving the brush texture untouched"
    );

    // A per-pattern param edit also re-renders live (Checker defaults to hard Softness 0.0; push it
    // fully soft so the edge pixels change).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[2],
        1.0,
    ));
    let buf_soft = t.canvas_rgba.as_ref().clone();
    assert_ne!(
        buf_checker, buf_soft,
        "editing a per-pattern param re-rendered the layer"
    );

    // A standard layer feature works on a Texture layer: hiding it drops it from the composite,
    // leaving only the opaque white base.
    t.set_layer_visible(id, true);
    t.set_layer_visible(id, false);
    let (hidden, _, _) = t.run_full();
    assert!(
        hidden
            .chunks_exact(4)
            .all(|p| p[0] == 255 && p[1] == 255 && p[2] == 255 && p[3] == 255),
        "hiding the texture layer reveals the white base"
    );
    t.set_layer_visible(id, true);
}

#[test]
fn texture_layer_size_and_offset_panel_events_are_real_valued_and_clamp() {
    // Regression (Enio 2026-06-25): the Layers texture-layer editor uses the SAME drag-scrub number
    // fields as the Brush panel — which emit the REAL value — but routed Size/Offset through
    // normalized (`0..1`) setters. So Size 1.0 mapped to TEX_SIZE_MAX (10.0) and any value < 1 to
    // `0.1 + v*9.9` (e.g. 0.1 → 1.09). The layer must store the real value, clamped to the real range,
    // exactly like the brush's `set_brush_texture_size` / `set_brush_texture_offset`.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::{TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN};

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    let id = t.add_texture_layer().expect("texture layer added");
    let size = |t: &PainterTool, axis: usize| match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => tex.size[axis],
        _ => panic!("texture layer"),
    };
    let offset = |t: &PainterTool, axis: usize| match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => tex.offset[axis],
        _ => panic!("texture layer"),
    };

    // Size: the headline bug — 1.0 must stay 1.0 (used to jump to 10.0), and a sub-1 value stays
    // itself (used to become `0.1 + v*9.9`).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        1.0,
    ));
    assert!(
        (size(&t, 0) - 1.0).abs() < 1e-6,
        "Size 1.0 stays 1.0, got {}",
        size(&t, 0)
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        0.5,
    ));
    assert!(
        (size(&t, 1) - 0.5).abs() < 1e-6,
        "Size 0.5 stays 0.5, got {}",
        size(&t, 1)
    );
    // Size clamps to the real bounds (not the normalized track).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        999.0,
    ));
    assert!((size(&t, 0) - TEX_SIZE_MAX).abs() < 1e-6);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        -5.0,
    ));
    assert!((size(&t, 0) - TEX_SIZE_MIN).abs() < 1e-6);

    // Offset: real-valued + clamps to ±1 the same way.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        -0.5,
    ));
    assert!(
        (offset(&t, 0) + 0.5).abs() < 1e-6,
        "Offset -0.5 stays -0.5, got {}",
        offset(&t, 0)
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        5.0,
    ));
    assert!((offset(&t, 1) - TEX_OFFSET_MAX).abs() < 1e-6);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        -5.0,
    ));
    assert!((offset(&t, 0) - TEX_OFFSET_MIN).abs() < 1e-6);
}

#[test]
fn texture_layer_compatible_with_duplicate_and_mask() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    let id = t.add_texture_layer().expect("texture layer added");

    // Duplicate (audit fix): a Texture layer duplicates like a raster.
    let dup = t.duplicate_layer(id).expect("texture layer duplicates");
    assert_ne!(dup, id);
    assert!(matches!(
        t.layers().get(dup).map(|l| &l.kind),
        Some(LayerKind::Texture(_))
    ));

    // Mask (audit fix): a Texture layer can take a grayscale mask (the dup is active after duplicate).
    let mask = t.add_mask_to_active().expect("texture layer takes a mask");
    assert_eq!(
        t.layers().get(dup).and_then(|l| l.mask),
        Some(mask),
        "the mask is attached to the texture layer"
    );
}

#[test]
fn brush_texture_section_not_hijacked_when_dock_shows_brush() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::TextureKind;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 16 * 16 * 4], 16, 16);
    let id = t.add_texture_layer().expect("texture layer added"); // active, dock shows Layers
    // Switch the dock to the Brush view; the texture layer stays active.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LAYERS_TOGGLE_DOCK));
    // A Kind change in the Brush view must hit the BRUSH, not the active texture layer.
    let layer_kind_before = match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => tex.kind,
        _ => panic!("expected a texture layer"),
    };
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        TextureKind::Voronoi.to_u8().to_string(),
    ));
    assert_eq!(
        t.brush_settings().texture_kind,
        TextureKind::Voronoi.to_u8(),
        "the Brush view's Kind edit reaches the brush"
    );
    match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => assert_eq!(
            tex.kind, layer_kind_before,
            "the texture layer is untouched while the Brush view is showing"
        ),
        _ => panic!("expected a texture layer"),
    }
}

// ── Per-dab randomize seam (Jitter Scale / Rotate + Randomize Color) ─────────────────────────
// These prove the panel controls are WIRED end-to-end: the generic PanelEvent reaches the brush
// state (not silently dropped — the dead-control class) AND the per-dab jitter actually alters the
// painted pixels. Unit-green ≠ product-green; only this e2e drive catches a missing register/route.

#[test]
fn randomize_controls_reach_the_brush_and_snapshot() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    let mut t = PainterTool::default();
    // Enable toggle (Click) + the five 0..1 sliders (SetValue) — exactly what the panel emits.
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
        0.3,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
        0.2,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
        0.1,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_SCALE,
        0.7,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_ROTATE,
        0.4,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_SPACING,
        0.6,
    ));
    // (a) the events reached the brush model (would all be 0/false if dropped).
    assert!(t.paint.brush.color_jitter_enabled, "enable toggle wired");
    assert_eq!(t.paint.brush.color_jitter_hue, 0.3, "Hue slider wired");
    assert_eq!(t.paint.brush.color_jitter_sat, 0.2, "Sat slider wired");
    assert_eq!(t.paint.brush.color_jitter_val, 0.1, "Value slider wired");
    assert_eq!(t.paint.brush.jitter_scale, 0.7, "Jitter Scale slider wired");
    assert_eq!(
        t.paint.brush.jitter_rotate, 0.4,
        "Jitter Rotate slider wired"
    );
    assert_eq!(
        t.paint.brush.jitter_spacing, 0.6,
        "Jitter Spacing slider wired"
    );
    // (b) the published snapshot the panel reads back mirrors them (slider positions).
    let s = t.brush_settings();
    assert!(s.color_jitter_enabled);
    assert_eq!(s.color_jitter, [0.3, 0.2, 0.1]);
    assert_eq!(s.jitter_scale, 0.7);
    assert_eq!(s.jitter_rotate, 0.4);
    assert_eq!(s.jitter_spacing, 0.6);
    // A second enable Click toggles it back off.
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
    ));
    assert!(!t.paint.brush.color_jitter_enabled, "enable toggle flips");
}

#[test]
fn randomize_color_varies_the_painted_pixels_e2e() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    // Mid-grey base + hard disk so each dab fully replaces its footprint with its (jittered) colour.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.color = [0.5, 0.5, 0.5];
    // Drive Randomize Color ON with a strong Value amount via the PANEL events (the wiring proof).
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
        1.0,
    ));
    // Paint a multi-dab horizontal stroke; per-dab Value jitter must yield >1 painted shade.
    t.on_canvas_pointer(cp([6.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Up));
    let shades: std::collections::BTreeSet<u8> = (6..58).map(|x| px(&t, 64, x, 32)[0]).collect();
    assert!(
        shades.len() > 1,
        "Randomize Color must vary the painted shades end-to-end, got {shades:?}"
    );

    // Control: with Randomize Color OFF the same stroke paints a single uniform shade.
    let mut t0 = white_canvas(64, 3.0);
    t0.paint.brush.color = [0.5, 0.5, 0.5];
    t0.on_canvas_pointer(cp([6.0, 32.0], PointerPhase::Down));
    t0.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Move));
    t0.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Up));
    let base: std::collections::BTreeSet<u8> = (6..58).map(|x| px(&t0, 64, x, 32)[0]).collect();
    assert_eq!(base.len(), 1, "no jitter ⟹ one uniform shade, got {base:?}");
}

// ── Seamless Tiling (wrap-around painting) ───────────────────────────────────────────────────

#[test]
fn tiling_x_wraps_paint_across_the_sprite_edge_e2e() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    // Enable Tiling X via the panel (the wiring proof — a dropped Click would leave it off).
    let mut t = white_canvas(64, 6.0);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_TILING_X));
    assert_eq!(
        t.brush_tiling(),
        [true, false],
        "Tiling X toggle reached the tool"
    );
    // A single dab at the RIGHT edge (x=63). With Tiling X it also paints the wrapped copy that
    // crosses onto the LEFT edge (x=0) — so a stroke over the border is seamless when tiled.
    t.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 64, 63, 32),
        [0, 0, 0, 255],
        "the dab painted the right edge"
    );
    assert_eq!(
        px(&t, 64, 0, 32),
        [0, 0, 0, 255],
        "Tiling X wrapped it onto the left edge"
    );
    // Only X tiles: the top-left corner stays white (no vertical wrap).
    assert_eq!(
        px(&t, 64, 0, 0),
        [255, 255, 255, 255],
        "Tiling Y off ⟹ no vertical wrap"
    );

    // Control: without tiling the same edge dab does NOT appear on the opposite edge.
    let mut t0 = white_canvas(64, 6.0);
    t0.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Down));
    t0.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Up));
    assert_eq!(
        px(&t0, 64, 0, 32),
        [255, 255, 255, 255],
        "no Tiling ⟹ the left edge is untouched"
    );
}

// ── Layer mask painting (bug: a selected mask couldn't be painted — the event fell through to
//    the move tool and dragged the sprite instead) ────────────────────────────────────────────

#[test]
fn a_selected_mask_is_paintable_e2e() {
    let mut t = white_canvas(64, 6.0);
    // Add a mask to the active raster layer; it becomes active with a white (fully-visible) buffer.
    let _mask = t
        .add_mask_to_active()
        .expect("mask added to the active raster layer");
    assert!(t.active_is_mask(), "the new mask is the active layer");
    // The bug: `paint_target_ready` rejected masks, so `on_canvas_pointer` returned `false` and the
    // event fell through to the move tool (dragging the sprite). It must now CONSUME the event...
    let consumed = t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    assert!(
        consumed,
        "painting a selected mask must consume the canvas event (not fall through to move/drag)"
    );
    // ...and paint the mask's coverage: the default black brush conceals (luma → 0) at the centre.
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "the mask was painted (black = conceal)"
    );
    // An unpainted corner stays white (fully visible).
    assert_eq!(
        px(&t, 64, 0, 0),
        [255, 255, 255, 255],
        "unpainted mask area stays white"
    );
}

#[test]
fn repeat_image_toggle_reaches_the_tool_e2e() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    let mut t = PainterTool::default();
    assert!(!t.repeat_image(), "off by default");
    // Toggle Repeat Image via the panel (wiring proof — a dropped Click would leave it off).
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_REPEAT_IMAGE));
    assert!(t.repeat_image(), "Repeat Image toggle reached the tool");
    assert!(
        t.brush_settings().repeat_image,
        "snapshot mirrors it for the panel"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_REPEAT_IMAGE));
    assert!(!t.repeat_image(), "toggles back off");
}

#[test]
fn sample_composite_at_uv_reads_the_painted_colour() {
    // The colour-picker eyedropper samples this (not the transparent Vello overlay). A painted
    // pixel must read its colour; an unpainted pixel reads the canvas background — never transparent.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.color = [1.0, 0.0, 0.0]; // red
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let centre = t
        .sample_composite_at_uv(0.5, 0.5)
        .expect("composite sample at centre");
    assert_eq!(
        [centre[0], centre[1], centre[2], centre[3]],
        [255, 0, 0, 255],
        "eyedropper reads the painted (opaque) colour, not transparent"
    );
    let corner = t
        .sample_composite_at_uv(0.0, 0.0)
        .expect("composite sample at corner");
    assert_eq!(
        [corner[0], corner[1], corner[2], corner[3]],
        [255, 255, 255, 255],
        "an unpainted pixel reads the opaque white canvas, not transparent"
    );
}
