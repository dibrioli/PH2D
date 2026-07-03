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
    // Seed every per-mode slot with this hard-disk fixture brush so a mode switch (e.g. into Mask) keeps
    // it instead of loading that tool's independent default (the "Sync with other tools" model). Tests
    // that exercise the independent/linked behaviour itself set their slots explicitly.
    let seed = t.paint.brush;
    for slot in &mut t.paint.brush_by_mode {
        *slot = seed;
    }
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
fn smear_mode_drags_pixels_along_the_stroke() {
    // DoD seam test: select Smear via the frozen PanelEvent channel (exactly what the left-rail
    // dispatch pushes), then drive a real stroke and assert the canvas content is dragged — the
    // Blender/Krita "Smearing" behaviour end-to-end through the tool.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 48u32;
    let mut t = PainterTool::default();
    // Left half black, right half white (both opaque).
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        space_attenuation: false,
        ..Default::default()
    };
    // Rail selects Smear → `SelectOption(PAINTER_PAINT_MODE, "smear")` reaches handle_panel_event.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "smear".to_string(),
    ));
    let boundary = size / 2; // x = 24
    let probe_x = boundary + 4; // x = 28, starts white
    let mid = (size / 2) as f32;
    assert_eq!(
        px(&t, size, probe_x, size / 2),
        [255, 255, 255, 255],
        "probe starts white"
    );
    // Stroke rightward starting inside the black region, crossing the boundary.
    t.on_canvas_pointer(cp([(boundary - 6) as f32, mid], PointerPhase::Down));
    t.on_canvas_pointer(cp([(boundary + 8) as f32, mid], PointerPhase::Move));
    t.on_canvas_pointer(cp([(boundary + 8) as f32, mid], PointerPhase::Up));
    let after = px(&t, size, probe_x, size / 2);
    assert!(
        after[0] < 255,
        "smear dragged darker (black) pixels rightward into the white area: {after:?}"
    );
    // Selecting Brush again exits Smear (no stuck mode).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Paint));
}

#[test]
fn blur_mode_softens_a_hard_edge_along_the_stroke() {
    // DoD seam test: select Blur via the frozen PanelEvent channel (exactly what the left-rail
    // dispatch pushes), then stroke along a hard black|white edge and assert the seam softens — the
    // Blender Soften behaviour end-to-end through the tool. Blur is stationary per dab, so the stroke
    // runs ALONG the boundary (each dab's footprint straddles both sides).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 48u32;
    let mut t = PainterTool::default();
    // Left half black, right half white (both opaque).
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "blur".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Blur));
    let boundary = size / 2; // x = 24, first white column
    let far_white = boundary + 15; // x = 39, outside the radius-8 footprint at x=24
    assert_eq!(
        px(&t, size, boundary, size / 2),
        [255, 255, 255, 255],
        "seam starts white"
    );
    // Stroke vertically ALONG the seam (x = boundary), top to bottom.
    let bx = boundary as f32;
    t.on_canvas_pointer(cp([bx, 6.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([bx, (size / 2) as f32], PointerPhase::Move));
    t.on_canvas_pointer(cp([bx, (size - 6) as f32], PointerPhase::Move));
    t.on_canvas_pointer(cp([bx, (size - 6) as f32], PointerPhase::Up));
    let seam = px(&t, size, boundary, size / 2);
    assert!(
        seam[0] < 255 && seam[0] > 0,
        "the seam softened toward grey (black averaged into the white edge): {seam:?}"
    );
    assert_eq!(seam[3], 255, "opaque canvas stays opaque");
    assert_eq!(
        px(&t, size, far_white, size / 2),
        [255, 255, 255, 255],
        "a pixel outside the footprint is untouched"
    );
    // Selecting Brush again exits Blur (no stuck mode).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Paint));
}

#[test]
fn inpaint_mode_heals_a_marked_defect_on_pen_up() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    // Mostly-white canvas with a small red "defect" square near the centre.
    let size = 48u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 21..27 {
        for x in 21..27 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 9.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    // Select the Inpaint heal mode exactly as the left-rail button forwards it.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "inpaint".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Inpaint));
    // Paint over the defect and release → the marked region is reconstructed.
    assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up));
    // Every defect pixel now reads ~white (rebuilt from the surrounding white), not red;
    // and no red tint survives the heal.
    for y in 21..27 {
        for x in 21..27 {
            let p = px(&t, size, x, y);
            assert!(
                p[0] > 225 && p[1] > 225 && p[2] > 225,
                "defect not healed at ({x},{y}): {p:?}"
            );
        }
    }
    // The heal mask is cleared for the next stroke.
    assert!(t.paint.inpaint_mask.iter().all(|&m| m < 128));
}

#[cfg(feature = "gpu")]
#[test]
#[ignore = "needs a GPU adapter; run with --features gpu -- --ignored"]
fn inpaint_heal_runs_on_the_gpu_and_reconstructs() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    // Confirm the heal actually reaches the GPU on this machine (else the test would silently prove
    // only the CPU fallback — the "unit-green ≠ works" trap this whole check guards against).
    if !super::inpaint::gpu_heal_available() {
        eprintln!("no GPU adapter — skipping GPU heal check");
        return;
    }
    // A 256² canvas with a large red defect, brushed with a big radius → the crop (defect + margin)
    // exceeds the 128² GPU threshold, so `run_inpaint` takes the GPU branch.
    let size = 256u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 96..160 {
        for x in 96..160 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 40.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "inpaint".to_string(),
    ));
    // Cover the whole defect with a few dabs, then release → GPU heal.
    for c in [
        [128.0, 128.0],
        [112.0, 112.0],
        [144.0, 144.0],
        [112.0, 144.0],
        [144.0, 112.0],
    ] {
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
    }
    t.on_canvas_pointer(cp([128.0, 128.0], PointerPhase::Up));
    // Every masked pixel is rebuilt to ~white (from the surrounding white) — the GPU compute produced a
    // valid reconstruction, not the red defect.
    for y in 96..160 {
        for x in 96..160 {
            if t.paint.inpaint_mask[(y * size + x) as usize] < 128 {
                continue;
            }
            let p = px(&t, size, x, y);
            assert!(
                p[0] > 210 && p[1] > 210 && p[2] > 210,
                "GPU heal left a defect at ({x},{y}): {p:?}"
            );
        }
    }
}

#[test]
fn inpaint_stroke_is_one_undo_step_back_to_the_defect() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 48u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 21..27 {
        for x in 21..27 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 9.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "inpaint".to_string(),
    ));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up));
    assert!(px(&t, size, 24, 24)[0] > 225, "defect healed");
    // One undo restores the original defect (heal ran before close_stroke).
    assert!(t.undo_last(), "there is an undo step to pop");
    assert_eq!(
        px(&t, size, 24, 24),
        [220, 20, 20, 255],
        "a single undo brings the defect back"
    );
}

#[test]
fn inpaint_param_sliders_route_into_the_heal() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "inpaint".to_string(),
    ));
    // Defaults reproduce today's behaviour (patch 3 / iters 6 / margin ≈ hole/2).
    assert_eq!(t.paint.inpaint_patch_norm, 0.25);
    assert_eq!(t.paint.inpaint_quality_norm, 0.3333);
    assert_eq!(t.paint.inpaint_search_norm, 0.2);
    // Each slider's `SetValue` lands on the matching norm (clamped `0..1`).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_INPAINT_PATCH_SLIDER,
        1.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_INPAINT_QUALITY_SLIDER,
        0.75,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_INPAINT_SEARCH_SLIDER,
        0.5,
    ));
    assert_eq!(t.paint.inpaint_patch_norm, 1.0);
    assert_eq!(t.paint.inpaint_quality_norm, 0.75);
    assert_eq!(t.paint.inpaint_search_norm, 0.5);
}

/// A white canvas with a red square at `[12,20)×[12,20)`, in Fill mode with a green brush.
fn fill_fixture(size: u32) -> PainterTool {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 12..20 {
        for x in 12..20 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "fill".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Fill));
    t.paint.brush.color = [0.0, 1.0, 0.0]; // green
    t
}

#[test]
fn fill_colordrop_fills_the_connected_region_and_undoes_in_one_step() {
    let size = 32u32;
    let mut t = fill_fixture(size);
    t.paint.fill_threshold = 0.05; // tight — only the red square
    // Drag the colour onto the red square and release (the ColorDrop).
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    // The whole connected red square is now green; the white surround is untouched.
    assert_eq!(
        px(&t, size, 16, 16),
        [0, 255, 0, 255],
        "drop filled the square"
    );
    assert_eq!(
        px(&t, size, 13, 18),
        [0, 255, 0, 255],
        "connected red all filled"
    );
    assert_eq!(
        px(&t, size, 2, 2),
        [255, 255, 255, 255],
        "surround untouched"
    );
    // Leaving Fill commits the drop; one undo restores the original red.
    t.set_paint_tool_mode("brush");
    assert!(t.undo_last(), "the fill is one undo step");
    assert_eq!(
        px(&t, size, 16, 16),
        [220, 20, 20, 255],
        "undo restored the defect"
    );
}

#[test]
fn fill_modal_threshold_slider_refills_live_and_cancel_reverts() {
    let size = 32u32;
    let mut t = fill_fixture(size);
    // Drop tight (only the red square). The drop leaves a pending fill for the modal to tune.
    t.set_fill_threshold(0.05);
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert!(
        t.has_active_fill(),
        "the drop leaves a pending fill for the modal"
    );
    assert_eq!(
        px(&t, size, 16, 16),
        [0, 255, 0, 255],
        "the drop filled the square"
    );
    // The modal slider drives set_fill_threshold live — a large threshold overflows into the surround.
    t.set_fill_threshold(1.0);
    assert_eq!(
        px(&t, size, 2, 2),
        [0, 255, 0, 255],
        "slider up overflows into the surround"
    );
    // Cancel reverts the layer to exactly before the drop — no undo entry.
    t.fill_cancel();
    assert!(!t.has_active_fill());
    assert_eq!(
        px(&t, size, 16, 16),
        [220, 20, 20, 255],
        "cancel restored the red square"
    );
    assert_eq!(
        px(&t, size, 2, 2),
        [255, 255, 255, 255],
        "cancel restored the surround"
    );
    assert!(!t.undo_last(), "cancel leaves NO undo step");
}

#[test]
fn route_fill_event_drives_the_modal_slider_done_and_cancel() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 32u32;

    // ── SetValue on the modal slider routes to set_fill_threshold + re-fills live. ──
    let mut t = fill_fixture(size);
    t.set_fill_threshold(0.05);
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert!(t.has_active_fill());
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_FILL_MODAL_SLIDER,
        1.0,
    ));
    assert_eq!(
        t.fill_threshold(),
        1.0,
        "SetValue routed to set_fill_threshold"
    );
    assert_eq!(
        px(&t, size, 2, 2),
        [0, 255, 0, 255],
        "the slider overflowed the fill into the surround"
    );

    // ── Click(DONE) commits: clears the pending fill + keeps it as one undo step. ──
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_FILL_MODAL_DONE));
    assert!(!t.has_active_fill(), "Done cleared the pending fill");
    assert!(t.undo_last(), "Done kept the fill as one undo step");

    // ── Click(CANCEL) reverts: clears the pending fill + leaves NO undo step. ──
    let mut t2 = fill_fixture(size);
    t2.set_fill_threshold(0.05);
    t2.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t2.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert!(t2.has_active_fill());
    t2.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_FILL_MODAL_CANCEL));
    assert!(!t2.has_active_fill(), "Cancel cleared the pending fill");
    assert_eq!(
        px(&t2, size, 16, 16),
        [220, 20, 20, 255],
        "Cancel restored the red square"
    );
    assert!(!t2.undo_last(), "Cancel leaves NO undo step");
}

#[test]
fn fill_shrinking_repaints_the_vacated_overflow() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    // A horizontal grey gradient (step 4/col): a high threshold bridges the whole row (overflow); a low
    // one only a few left columns — so shrinking must repaint (dirty + restore) the vacated right side.
    // Regression guard for "reducing the threshold didn't erase the overflow".
    let size = 32u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let v = (100 + x * 4) as u8;
            let o = ((y * size + x) * 4) as usize;
            src[o..o + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "fill".to_string(),
    ));
    t.paint.brush.color = [1.0, 0.0, 0.0]; // red fill
    // Drop at the left column with a high threshold → the whole row fills (overflow).
    t.paint.fill_threshold = 0.9;
    t.on_canvas_pointer(cp([0.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([0.0, 16.0], PointerPhase::Up));
    let overflow = t.dirty_rect.expect("the drop dirtied a region");
    assert!(overflow.w >= size - 2, "the drop overflowed across the row");
    // Lower the threshold via the modal slider → the fill shrinks to the left columns.
    t.dirty_rect = None;
    t.set_fill_threshold(0.0);
    let after = t.dirty_rect.expect("the shrink dirtied a region");
    // The vacated right side must be marked dirty (else it ghosts on the GPU) — the dirty rect still
    // spans the original overflow width, not just the small new fill.
    assert!(
        after.x + after.w >= overflow.x + overflow.w,
        "shrink must dirty the vacated overflow: {after:?} must cover {overflow:?}"
    );
    // And the buffer itself is restored there — a far-right column is back to its gradient grey, not red.
    let rx = size - 2;
    let v = (100 + rx * 4) as u8;
    assert_eq!(
        px(&t, size, rx, 16),
        [v, v, v, 255],
        "the vacated pixel is restored to the original gradient"
    );
}

#[test]
fn composite_brush_runs_an_isolated_layer_and_reorders() {
    // Composite is a Brush-tool upgrade wired over the frozen PanelEvent channel. Prove: (1) the enable
    // checkbox toggles it, (2) a layer isolated by Strength actually runs inside the stack (Blur softens
    // a hard edge; the Brush/Smear layers are zeroed so no colour is painted), (3) the reorder buttons
    // move the tool between the fixed positions.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 48u32;
    let mut t = PainterTool::default();
    let mut src = vec![255u8; (size * size * 4) as usize]; // right half white
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[0, 0, 0, 255]); // left half black
        }
    }
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        color: [1.0, 0.0, 0.0], // red — would show if the Brush layer painted
        space_attenuation: false,
        ..Default::default()
    };
    // Enable the Composite Brush.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE));
    assert!(t.composite_enabled(), "checkbox enabled composite");
    // Isolate the Blur layer (default positions: 0 Brush · 1 Smear · 2 Blur) by zeroing Brush + Smear.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[0],
        0.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[1],
        0.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[2],
        1.0,
    ));
    let boundary = size / 2; // x = 24, first white column
    let bx = boundary as f32;
    t.on_canvas_pointer(cp([bx, 6.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([bx, (size / 2) as f32], PointerPhase::Move));
    t.on_canvas_pointer(cp([bx, (size - 6) as f32], PointerPhase::Move));
    t.on_canvas_pointer(cp([bx, (size - 6) as f32], PointerPhase::Up));
    let seam = px(&t, size, boundary, size / 2);
    assert!(
        seam[0] > 0 && seam[0] < 255,
        "the Blur layer softened the seam inside the composite: {seam:?}"
    );
    assert!(
        seam[0] == seam[1] && seam[1] == seam[2],
        "grey (no colour) — the zeroed Brush layer painted nothing: {seam:?}"
    );
    // Reorder: move the Blur layer (position 2) up to position 0.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_COMPOSITE_UP[2]));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_COMPOSITE_UP[1]));
    assert_eq!(
        t.paint.composite[0].op,
        crate::tool::paint::CompositeOp::Blur,
        "reorder moved Blur to the top position"
    );
}

#[test]
fn composite_runs_layers_under_the_interactive_preview_methods() {
    // The interactive-preview methods (Drag Dot / Anchored / Line) restore + re-stamp each frame; they
    // must run the WHOLE composite stack, not just the Brush layer. Prove it with a Drag Dot + a
    // Blur-only composite: a single click must soften the hard edge under the dab (the Blur layer ran
    // through the preview path), with no colour painted (Brush + Smear layers zeroed).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 48u32;
    let mut t = PainterTool::default();
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        color: [1.0, 0.0, 0.0], // red — must NOT appear (Brush layer zeroed)
        stroke_method: ph2d_painter_brush::StrokeMethod::DragDot,
        space_attenuation: false,
        ..Default::default()
    };
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE));
    // Blur-only: Brush(0) + Smear(1) off, Blur(2) on.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[0],
        0.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[1],
        0.0,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[2],
        1.0,
    ));
    let boundary = size / 2; // x = 24
    // A single Drag-Dot click centred on the seam.
    t.on_canvas_pointer(cp([boundary as f32, (size / 2) as f32], PointerPhase::Down));
    t.on_canvas_pointer(cp([boundary as f32, (size / 2) as f32], PointerPhase::Up));
    let seam = px(&t, size, boundary, size / 2);
    assert!(
        seam[0] > 0 && seam[0] < 255,
        "the Blur layer softened the seam via the Drag-Dot preview path: {seam:?}"
    );
    assert!(
        seam[0] == seam[1] && seam[1] == seam[2],
        "grey (no colour) — the zeroed Brush layer painted nothing in the preview: {seam:?}"
    );
}

#[test]
fn clone_mode_copies_from_the_sampled_source() {
    // DoD seam test: select Clone via the frozen PanelEvent channel, sample a source with the "Set
    // Source" pick mode (a canvas click that must NOT paint), then paint elsewhere and assert the
    // source pixels are cloned in at the fixed offset — the clone stamp end-to-end through the tool.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 48u32;
    let mut t = PainterTool::default();
    // Left half red, right half blue (opaque).
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            let c = if x < size / 2 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            };
            src[i..i + 4].copy_from_slice(&c);
        }
    }
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 6.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        strength: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "clone".to_string(),
    ));
    assert!(t.is_clone_mode());
    let mid = (size / 2) as f32;
    // Arm the pick, then click a RED source point (x=8). The click samples the source, not paints.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_CLONE_SET_SOURCE));
    assert!(t.clone_sample_armed());
    t.on_canvas_pointer(cp([8.0, mid], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, mid], PointerPhase::Up));
    assert_eq!(t.paint.clone_source, Some([8.0, mid]), "source sampled");
    assert!(!t.clone_sample_armed(), "pick disarmed after sampling");
    // Paint at a BLUE point (x=30): offset = 8 − 30 = −22 → the dab clones from x≈8 (red).
    let probe = 30;
    assert_eq!(
        px(&t, size, probe, size / 2),
        [0, 0, 255, 255],
        "probe starts blue"
    );
    t.on_canvas_pointer(cp([probe as f32, mid], PointerPhase::Down));
    t.on_canvas_pointer(cp([probe as f32, mid], PointerPhase::Up));
    let p = px(&t, size, probe, size / 2);
    assert!(
        p[0] > 0 && p[2] < 255,
        "red cloned from the source into the blue half: {p:?}"
    );
    // Exit Clone → back to normal paint.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Paint));
}

#[test]
fn mask_brush_protects_and_keeps_the_layer_fully_visible() {
    // The Mask BRUSH paints a TEMPORARY PROTECTION scratch (Blender Sculpt-mask style). It must NOT create
    // a stack layer, keep the current layer active, leave the layer's pixels untouched (non-destructive),
    // and — critically — NEVER make anything invisible: the layer stays fully opaque; the overlay only
    // TINTS the protected region so you can see it.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 6.0);
    let raster = t.layers.active().expect("a raster is active");
    let n_before = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    assert!(t.is_mask_mode());
    assert_eq!(t.mask_brush(), 0, "default sub-brush is Paint (protect)");
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down)); // protect the centre (into the scratch)
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    // No layer created; still on the raster; a transient scratch is live.
    assert_eq!(
        t.layers.all_ids().count(),
        n_before,
        "the mask brush creates NO layer"
    );
    assert_eq!(t.layers.active(), Some(raster), "still on the raster");
    assert!(t.mask_scratch_active(), "a transient scratch is live");
    // Non-destructive: the layer's own pixels (canvas_rgba) are untouched (still white).
    assert_eq!(
        px(&t, 24, 12, 12),
        [255, 255, 255, 255],
        "the layer pixels are untouched (non-destructive)"
    );
    // NOTHING is invisible: the protected centre stays FULLY OPAQUE (a = 255) — the opposite of a
    // visibility mask. The overlay only tints the RGB so you can see it; an unprotected corner is pristine.
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let c = ((12 * w + 12) * 4) as usize;
    let corner = ((2 * w + 2) * 4) as usize;
    assert_eq!(
        buf[c + 3],
        255,
        "the protected pixel is NOT hidden — still fully opaque"
    );
    assert!(
        buf[c] < 255,
        "the overlay tints the protected region, got {}",
        buf[c]
    );
    assert_eq!(
        [buf[corner], buf[corner + 3]],
        [255, 255],
        "an unprotected corner keeps the pristine image"
    );
}

#[test]
fn mask_stroke_undoes_and_redoes_with_the_global_timeline() {
    // A mask stroke mutates only the transient scratch (the layer's own pixels stay put), so the undo
    // model must capture that scratch — else the stroke produces a no-op undo entry and can't be rolled
    // back. The reported bug. Paint a mask dab, then undo/redo and check the scratch flips
    // concealed↔cleared in lock-step with the global painter timeline.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 24u32;
    let mut t = white_canvas(size, 6.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    let center = ((12 * size + 12) * 4) as usize; // R channel of the scratch centre
    // Paint a mask dab → the scratch is created + the centre concealed (R drops from white).
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    let painted = t.paint.mask_scratch_rgba[center];
    assert!(
        painted < 200,
        "the mask dab concealed the centre (R={painted})"
    );
    // Undo → the scratch rolls back to white (the stroke IS undoable with the global timeline).
    assert!(t.undo_last(), "the mask stroke is an undo step");
    assert_eq!(
        t.paint.mask_scratch_rgba[center], 255,
        "undo cleared the scratch back to white"
    );
    // Redo → the conceal comes back, identically.
    assert!(t.redo_last(), "redo re-applies the mask stroke");
    assert_eq!(
        t.paint.mask_scratch_rgba[center], painted,
        "redo restored the concealed scratch"
    );
}

#[test]
fn mask_canvas_op_is_undoable() {
    // A whole-canvas mask Modifier (Clear / Invert / …) mutates the scratch and must be undoable too.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 24u32;
    let mut t = white_canvas(size, 6.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    let center = ((12 * size + 12) * 4) as usize;
    // A mask dab conceals the centre.
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    let concealed = t.paint.mask_scratch_rgba[center];
    assert!(concealed < 200, "the dab concealed the centre");
    // Clear (op 5) whitens the whole scratch.
    t.mask_canvas_op(5);
    assert_eq!(
        t.paint.mask_scratch_rgba[center], 255,
        "Clear whitened the scratch"
    );
    // Undo rolls the Clear back to the concealed dab (the canvas op is its own undo step).
    assert!(t.undo_last(), "the canvas op is an undo step");
    assert_eq!(
        t.paint.mask_scratch_rgba[center], concealed,
        "undo restored the concealed dab"
    );
}

#[test]
fn mask_brush_freezes_pixels_against_the_paint_brush() {
    // The CORE of the protection mask: a scratch-protected region is FROZEN — the paint Brush (and every
    // other paint tool) cannot alter it. Protect the centre, switch to the Brush, then paint over both the
    // protected centre and an unprotected corner: the centre keeps its pixel, the corner paints normally.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(32, 4.0); // black brush on white
    // Protect the centre.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Switch to the normal Brush (protection persists) and stroke the protected centre — it must not move.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 32, 16, 16),
        [255, 255, 255, 255],
        "the protected centre is FROZEN — the brush could not paint it"
    );
    // An unprotected corner paints normally (black).
    t.on_canvas_pointer(cp([28.0, 28.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([28.0, 28.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 32, 28, 28),
        [0, 0, 0, 255],
        "an unprotected pixel paints normally"
    );
}

#[test]
fn clear_then_repaint_makes_a_fresh_protection() {
    // Bug: after Clear the app could no longer create new temporary masks (the mask leaked the user's
    // brush colour, so a light colour painted an invisible/weak mask). Clear must leave the scratch able
    // to take a NEW full-strength protection regardless of the brush colour.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 5.0);
    t.paint.brush.color = [0.9, 0.9, 0.9]; // a light brush colour must not weaken the fresh mask
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Clear the mask.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[5]));
    // Paint a NEW protection.
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(
        t.mask_scratch_active(),
        "a scratch is live after Clear + repaint"
    );
    // The fresh protection must freeze the brush at the centre.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 24, 12, 12),
        [255, 255, 255, 255],
        "the freshly-painted protection freezes the brush after Clear"
    );
}

// ── Audit: layer-system mask via Apply (Photoshop-style visibility mask) ──────────────────────────

#[test]
fn apply_mask_is_one_undo_step_and_redoable() {
    // Apply is a single structural undo step: undo removes the created Mask layer + restores the parent's
    // full visibility (snapshot_model captures both layers AND images); redo re-creates it with its pixels.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let target = t.layers.active().unwrap();
    let n_before = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    assert!(t.layers.get(target).and_then(|l| l.mask).is_some());
    assert_eq!(t.layers.all_ids().count(), n_before + 1);
    // Undo → the mask layer is gone and the parent is unmasked again.
    assert!(t.can_undo());
    assert!(t.undo_last());
    assert!(
        t.layers.get(target).and_then(|l| l.mask).is_none(),
        "undo removed the layer mask"
    );
    assert_eq!(
        t.layers.all_ids().count(),
        n_before,
        "the Mask layer is gone after undo"
    );
    // Redo → the mask is back and conceals the centre again.
    assert!(t.redo_last());
    assert!(
        t.layers.get(target).and_then(|l| l.mask).is_some(),
        "redo restored the layer mask"
    );
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i + 3] < 128,
        "redo restored the mask concealment, got a = {}",
        buf[i + 3]
    );
}

#[test]
fn apply_copies_the_scratch_into_the_mask_faithfully() {
    // Apply must copy the scratch coverage 1:1 into the layer mask — the mask pixel at a point equals the
    // scratch coverage there (no re-threshold / re-colour). Verified at a protected centre + a clear corner.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let target = t.layers.active().unwrap();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    let idx_c = (8 * 16 + 8) as usize;
    let idx_corner = (16 + 1) as usize; // (1,1)
    let sc_c = crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx_c);
    let sc_corner = crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx_corner);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    let mask = t.layers.get(target).and_then(|l| l.mask).unwrap();
    let img = t.images.get(&mask).expect("the mask pixels live in images");
    assert!(
        (crate::compositor::mask_value(&img.rgba8, idx_c) - sc_c).abs() < 0.004,
        "mask centre coverage matches the scratch (faithful copy)"
    );
    assert!(
        (crate::compositor::mask_value(&img.rgba8, idx_corner) - sc_corner).abs() < 0.004,
        "mask corner coverage matches the scratch (faithful copy)"
    );
}

#[test]
fn apply_twice_merges_into_the_existing_mask() {
    // The merge branch: once a layer has a mask, painting a NEW protection + Apply again multiplies the
    // scratch INTO the existing mask (NO second Mask layer; the same mask id refines its coverage).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 5.0);
    let target = t.layers.active().unwrap();
    let n0 = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    // First Apply: protect + apply at spot A.
    t.on_canvas_pointer(cp([6.0, 6.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([6.0, 6.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    let mask = t.layers.get(target).and_then(|l| l.mask).unwrap();
    assert_eq!(t.layers.all_ids().count(), n0 + 1);
    // Second protection at spot B + Apply again → merge in place (no new layer, same mask id).
    t.on_canvas_pointer(cp([18.0, 18.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([18.0, 18.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    assert_eq!(
        t.layers.all_ids().count(),
        n0 + 1,
        "merge: no second Mask layer created"
    );
    assert_eq!(
        t.layers.get(target).and_then(|l| l.mask),
        Some(mask),
        "the same mask id refined in place"
    );
    // Both spots A and B are hidden (black) in the merged mask.
    let img = t.images.get(&mask).unwrap();
    assert!(
        crate::compositor::mask_value(&img.rgba8, (6 * 24 + 6) as usize) < 0.5,
        "spot A stays hidden after the merge"
    );
    assert!(
        crate::compositor::mask_value(&img.rgba8, (18 * 24 + 18) as usize) < 0.5,
        "spot B is hidden after the merge"
    );
}

#[test]
fn erase_mask_sub_brush_removes_protection() {
    // Bug: the Erase sub-brush was broken (the stamp reads each dab's OWN colour, baked from the user's
    // brush colour, so the white override was a no-op → Erase painted the wrong coverage). Erase (white)
    // over a protected area must UNPROTECT it, so the paint brush can then modify those pixels again.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 6.0);
    let idx = (12 * 24 + 12) as usize;
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    // Paint (protect) the centre → scratch fully black there.
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(
        crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx) < 0.01,
        "Paint protected the centre (scratch black)"
    );
    // Erase sub-brush over the same spot → scratch back to white (unprotected).
    t.set_mask_brush(1);
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(
        crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx) > 0.99,
        "Erase unprotected the centre (scratch white again)"
    );
    // End-to-end: the centre is now unprotected → the paint brush CAN modify it.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 24, 12, 12),
        [0, 0, 0, 255],
        "Erase removed the protection → the brush paints the centre again"
    );
}

#[test]
fn mask_paint_ignores_the_brush_colour() {
    // Root cause of the "mask is much lighter than normal" + Clear/Erase bugs: the mask must paint a PURE
    // coverage (black = protect), ignoring the user's brush colour. A light/coloured brush used to leak
    // its luma into the scratch → partial protection → a faint overlay. Paint with a light colour and a
    // Screen blend and assert FULL protection (scratch black, brush frozen).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 6.0);
    t.paint.brush.color = [1.0, 0.85, 0.2]; // a light yellow — must NOT weaken the mask
    t.paint.brush.blend = ph2d_painter_brush::BrushBlend::Screen; // a non-Mix blend must not break it
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    let idx = (12 * 24 + 12) as usize;
    assert!(
        crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx) < 0.01,
        "the mask painted FULL black protection regardless of the light brush colour + blend"
    );
    // And it freezes the brush at the centre (full protection, not partial).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 24, 12, 12),
        [255, 255, 255, 255],
        "full protection freezes the brush"
    );
}

#[test]
fn mask_apply_creates_a_layer_mask_from_the_scratch() {
    // Apply promotes the transient scratch to a REAL layer-system mask attached to the current layer:
    // a Mask layer appears (count up), `target.mask` points at it, the scratch clears, the parent's OWN
    // pixels are UNTOUCHED (non-destructive — the mask lives in the stack, not baked into the alpha), and
    // the composite still conceals through the new mask.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let target = t.layers.active().expect("a raster is active");
    let n_before = t.layers.all_ids().count();
    assert!(
        t.layers.get(target).and_then(|l| l.mask).is_none(),
        "the raster starts with no layer mask"
    );
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down)); // conceal centre (scratch only)
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Apply → a real layer mask is created from the scratch.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    assert!(!t.mask_scratch_active(), "Apply cleared the scratch");
    assert_eq!(
        t.layers.all_ids().count(),
        n_before + 1,
        "Apply added exactly one Mask layer"
    );
    let mask = t
        .layers
        .get(target)
        .and_then(|l| l.mask)
        .expect("the target now owns a layer mask");
    assert!(
        matches!(
            t.layers.get(mask).map(|l| &l.kind),
            Some(LayerKind::Mask(_))
        ),
        "the attached layer is a Mask"
    );
    assert_eq!(
        t.layers.active(),
        Some(target),
        "the parent raster stays the active edit layer (not the mask)"
    );
    // Non-destructive: the parent's OWN pixels are untouched (still opaque white — the mask is separate).
    assert_eq!(
        px(&t, 16, 8, 8),
        [255, 255, 255, 255],
        "Apply did NOT bake into the layer alpha (the mask is a separate stack layer)"
    );
    // The composite still conceals the masked centre through the new layer mask (scratch cleared → no
    // overlay film now, so the alpha drops to ~0).
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i + 3] < 128,
        "the layer mask conceals the centre, got a = {}",
        buf[i + 3]
    );
}

#[test]
fn mask_scratch_persists_across_a_tool_switch() {
    // The scratch is PERSISTENT (correção #1): switching the rail tool does NOT discard it. After painting
    // the scratch and switching to the Brush, it stays live (its target layer is still active) and keeps
    // PROTECTING the region — so you can paint freely around the frozen area with the Brush. (Switching
    // LAYERS is the only thing that makes it go dormant.)
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down)); // protect centre (scratch only)
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Switch to the Brush — the scratch must NOT be discarded.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    assert!(
        t.mask_scratch_active(),
        "switching tools keeps the scratch alive (its target layer is still active)"
    );
    // The composite still TINTS the protected centre while an unprotected corner keeps the pristine image.
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let c = ((8 * w + 8) * 4) as usize;
    let corner = ((w + 1) * 4) as usize;
    assert!(
        buf[c] < 128,
        "the scratch still marks the protected centre after the tool switch, got {}",
        buf[c]
    );
    assert_eq!(buf[corner], 255, "the unprotected corner keeps the image");
}

#[test]
fn mask_canvas_op_clear_then_invert() {
    // The whole-canvas Modifiers edit the transient SCRATCH (no layer). Clear → nothing protected (no
    // overlay tint → pristine layer); Invert → everything protected (fully tinted). Verified via composite.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = white_canvas(16, 4.0);
    let n_before = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[5])); // Clear → scratch white
    assert_eq!(
        t.layers.all_ids().count(),
        n_before,
        "canvas ops create NO layer"
    );
    assert!(
        t.mask_scratch_active(),
        "a scratch is live after a Modifier"
    );
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert_eq!(
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]],
        [255, 255, 255, 255],
        "Clear → nothing protected → pristine (no overlay tint)"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[4])); // Invert → scratch black
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i] < 128,
        "Invert → all protected, composite tinted dark, got {}",
        buf[i]
    );
}

#[test]
fn layer_mask_paintable_by_brush_and_grayscale_view_eye() {
    // A LAYER-SYSTEM mask (Layers "Mask" button) is paintable by the NORMAL brush (any tool), and its
    // grayscale-view eye toggles the canvas between the masked effect (closed) and the mask channel (open).
    let mut t = white_canvas(16, 4.0);
    let mask = t.add_mask_to_active().expect("layer mask created + active");
    // Normal Paint stroke (black default) on the active mask → conceal centre.
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert!(
        px(&t, 16, 8, 8)[0] < 128,
        "the brush painted the layer mask"
    );
    // Eye closed (default): composite shows the EFFECT — concealed centre hidden (low alpha).
    assert_eq!(t.mask_view_grayscale(), None);
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i + 3] < 128,
        "effect view hides the concealed centre, got a = {}",
        buf[i + 3]
    );
    // Eye open: composite shows the mask GRAYSCALE — concealed centre opaque black.
    t.toggle_mask_view_grayscale(mask);
    assert_eq!(t.mask_view_grayscale(), Some(mask.0));
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert_eq!(
        [buf[i], buf[i + 3]],
        [0, 255],
        "grayscale view shows the mask channel (opaque black centre)"
    );
}

#[test]
fn mask_overlay_tints_the_protected_composite() {
    // The overlay is a quick-mask film over the PROTECTED region: an all-unprotected (white) mask shows
    // nothing, so Clear→Invert (all protected / black) + the fluorescent-yellow overlay pulls the
    // composite's blue down (yellow = low blue), proving the film renders on the frozen area.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = white_canvas(16, 4.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_COLOR[1])); // fluorescent yellow
    assert_eq!(t.mask_overlay_color(), 1);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[5])); // Clear → white (unprotected)
    // An all-unprotected mask must NOT tint (no flood).
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert_eq!(
        [buf[i], buf[i + 1], buf[i + 2]],
        [255, 255, 255],
        "an all-unprotected mask shows NO overlay flood"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[4])); // Invert → black (protected)
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    let (r, g, b) = (buf[i], buf[i + 1], buf[i + 2]);
    assert!(
        b < r && b < g,
        "yellow overlay tints the protected area, pulling blue below red/green: ({r}, {g}, {b})"
    );
}

#[test]
fn eyedropper_samples_the_pixel_into_the_brush_colour() {
    // The rail arms the on-canvas Eyedropper (mode "eyedropper"); the next canvas Down samples the pixel
    // under the cursor into the brush colour, consumes the click (no paint / no sprite move), and disarms
    // back to Brush. Sampling a WHITE canvas pixel flips the black default brush to white.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 3.0); // brush colour is black [0,0,0]
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "eyedropper".to_string(),
    ));
    assert!(t.eyedropper_armed(), "the pick is armed");
    let consumed = t.on_canvas_pointer(cp([2.0, 2.0], PointerPhase::Down));
    assert!(
        consumed,
        "the Eyedropper consumes the click (no fall-through to move)"
    );
    assert!(!t.eyedropper_armed(), "one-shot — disarms after sampling");
    assert_eq!(
        t.brush_settings().color,
        [1.0, 1.0, 1.0],
        "sampled the white canvas pixel into the brush colour"
    );
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

/// Open a 3-point Curve (along y=32) in edit mode with the midpoint (index 1) selected, grab tol set.
fn open_curve_midpoint_selected() -> PainterTool {
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.set_shape_grab_tol_px(4.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    t
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
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
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
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // curve along y=32
    assert!(
        px(&t, 64, 32, 32)[0] < 200,
        "the curve painted on y=32 at zero offset"
    );
    assert_eq!(
        px(&t, 64, 32, 12),
        [255, 255, 255, 255],
        "y=12 is white before offset"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6)); // +20px perpendicular
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "the stroke left the original y=32 line (restored white): {:?}",
        px(&t, 64, 32, 32)
    );
    assert!(
        px(&t, 64, 32, 12)[0] < 200,
        "the offset stroke now paints ~20px up at y=12: {:?}",
        px(&t, 64, 32, 12)
    );
}

#[test]
fn offset_bakes_into_the_geometry_on_apply_keep_and_resets_the_slider() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // Apply & Keep bakes the live Offset into the kept curve (its position is now offset 0) and resets the
    // slider to centre (0.5). Draw a curve, offset it, Apply & Keep, then assert the slider snapped back to
    // 0.5 AND the kept overlay sits at the offset position (so it does not jump when the offset clears).
    let mut t = white_canvas(64, 2.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6)); // +20px up
    let off_spine_y = t.curve_overlay().unwrap().spine[0][1];
    assert!(
        off_spine_y < 25.0,
        "the offset overlay sits up near y=12: {off_spine_y}"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP));
    assert!(
        (t.brush_settings().offset - 0.5).abs() < 1e-4,
        "the Offset slider reset to centre"
    );
    let kept_spine_y = t.curve_overlay().expect("kept open").spine[0][1];
    assert!(
        (kept_spine_y - off_spine_y).abs() < 1.0,
        "the kept curve stayed at the offset position (no jump): {kept_spine_y} vs {off_spine_y}"
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
        StrokeMethod::Curve.to_u8(),
        "method is now Curve"
    );
    assert_eq!(
        ov.points.len(),
        4,
        "circle → the 4 cardinal anchors (no duplicate seam)"
    );
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
fn edit_after_offset_bakes_first_so_the_converted_circle_is_not_deformed() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::StrokeMethod;
    // Regression (Enio 2026-06-28): E (Edit) with a live Offset converted the BASE circle and then re-applied
    // the offset on top (double offset → deformed curve). Now convert bakes the offset first: the slider
    // resets to 0.5 and the converted anchors sit at the OFFSET (effective) radius, evenly around the centre.
    let mut t = white_canvas(96, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([68.0, 48.0], PointerPhase::Move)); // radius 20
    t.on_canvas_pointer(cp([68.0, 48.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_OFFSET, 0.6)); // +20px → radius ~40
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_STROKE_EDIT));
    assert!(
        (t.brush_settings().offset - 0.5).abs() < 1e-4,
        "Edit baked the offset → slider reset to 0.5"
    );
    let ov = t.curve_overlay().expect("a curve opened");
    assert_eq!(ov.points.len(), 4, "4 cardinal anchors");
    // Every anchor is ~40px from the centre (the effective radius), so the circle is NOT deformed.
    for p in &ov.points {
        let r = ((p[0] - 48.0).powi(2) + (p[1] - 48.0).powi(2)).sqrt();
        assert!(
            (r - 40.0).abs() < 2.0,
            "anchor sits at the offset radius ~40: r={r}"
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
    // Fill methods (Line/Curve/Ellipse/Polygon) take the no-snapshot / self-clearing per-layer path: the
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
    // Polyline model: click corner A, then press corner B and sweep it around (the line pivots on A),
    // settling near A. Each move re-stamps the whole line via the drag-preview restore.
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    t.on_canvas_pointer(cp(a, PointerPhase::Up)); // corner A
    t.on_canvas_pointer(cp([52.0, 12.0], PointerPhase::Down)); // create corner B
    for b in [[52.0, 50.0], [52.0, 31.0], [16.0, 31.0]] {
        t.on_canvas_pointer(cp(b, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([16.0, 31.0], PointerPhase::Up));
    // The final line is the short A=(10,31)→(16,31) segment (y≈31, x≈7..19). Any painted pixel well away
    // from it (e.g. y<24 or y>38, or x>26) is a trail from an earlier endpoint the move failed to restore.
    let mut trail = Vec::new();
    for y in 0..64u32 {
        for x in 0..64u32 {
            let far = !(24..=38).contains(&y) || x > 26;
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
fn shape_layer_opacity_edits_its_source_document_layer_two_way() {
    // The per-layer opacity box is TWO-WAY with the Shape SOURCE layer's opacity slider: editing the box
    // edits exactly that source layer's opacity (Enio 2026-06-29). Uses `white_canvas` → `set_source`, so
    // the painter is NOT document-bound (`bound_doc == None`) — the real case where the box wasn't updating
    // the sprite layer; the guard is `bound_doc == shape_source_doc`, which holds for the unbound doc too.
    let mut t = white_canvas(64, 6.0);
    t.layers.add_raster("L2", 64, 64).expect("add layer");
    t.capture_layers_as_brush_shape(); // shape source == the (unbound) painted document
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_opacity(0, 0.5);
    let s = t.brush_settings();
    assert!(
        (s.shape_layer_opacity[0] - 0.5).abs() < 1e-3,
        "the box value mirrors into the brush snapshot: {}",
        s.shape_layer_opacity[0]
    );
    let changed = t
        .layers
        .root()
        .iter()
        .filter(|&&id| {
            t.layers
                .get(id)
                .is_some_and(|l| (l.opacity - 0.5).abs() < 1e-3)
        })
        .count();
    assert_eq!(
        changed, 1,
        "exactly the ONE source document layer's opacity changed (two-way), not the others"
    );
}

#[test]
#[allow(non_snake_case)]
fn shape_opacity_and_blend_remote_control_the_STASHED_source_sprite() {
    use ph2d_painter_effects::BlendMode;
    // The shape source sprite is used to paint OTHER sprites. After capturing sprite 1 as the Shape and
    // switching to paint sprite 2, sprite 1 is STASHED — yet editing opacity/blend in the brush must still
    // update sprite 1's layer (Enio 2026-06-29: "a sprite usada como shape, que agora não está mais
    // selecionada, deve ser atualizada"), never sprite 2's. Verified by switching back to sprite 1.
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
    t.layers.add_raster("L2", 64, 64).expect("add"); // multi-layer ⇒ sprite 1 gets stashed on switch
    t.capture_layers_as_brush_shape(); // shape source = sprite 1
    t.toggle_brush_shape_per_layer_color();
    t.bind_document(2, vec![0u8; 32 * 32 * 4], 32, 32); // paint sprite 2 — sprite 1 is now stashed
    let two_layers = t.brush_settings().shape_layer_count >= 2;
    t.set_brush_shape_layer_opacity(0, 0.4);
    if two_layers {
        t.set_brush_shape_layer_blend(1, BlendMode::Multiply.to_u8());
    }
    // Sprite 2's layers must be UNTOUCHED (we edited the stashed sprite 1, not the painted sprite 2).
    assert!(
        t.layers.root().iter().all(|&id| t
            .layers
            .get(id)
            .is_some_and(|l| (l.opacity - 0.4).abs() > 1e-3)),
        "the painted sprite 2 must NOT have its opacity changed"
    );
    // Switch BACK to sprite 1 — its stashed stack (restored) must carry the brush edits.
    t.bind_document(1, vec![0u8; 4], 1, 1);
    let op_changed = t
        .layers
        .root()
        .iter()
        .filter(|&&id| {
            t.layers
                .get(id)
                .is_some_and(|l| (l.opacity - 0.4).abs() < 1e-3)
        })
        .count();
    assert_eq!(
        op_changed, 1,
        "the stashed shape-source sprite's layer opacity was remote-controlled"
    );
    if two_layers {
        let blend_changed = t
            .layers
            .root()
            .iter()
            .filter(|&&id| {
                t.layers
                    .get(id)
                    .is_some_and(|l| l.blend_mode == BlendMode::Multiply)
            })
            .count();
        assert_eq!(
            blend_changed, 1,
            "the stashed shape-source sprite's layer blend was remote-controlled"
        );
    }
}

#[test]
fn shape_layer_blend_edits_its_source_document_layer_two_way() {
    use ph2d_painter_effects::BlendMode;
    // The blend dropdown is a REMOTE CONTROL of the source layer's blend mode (Enio 2026-06-29). Editing it
    // edits that source layer's `blend_mode` (and the Layers panel shows it). Layer index 1 is a non-base
    // layer (the base, index 0, has no blend).
    let mut t = white_canvas(64, 6.0);
    t.layers.add_raster("L2", 64, 64).expect("add layer");
    t.capture_layers_as_brush_shape();
    t.toggle_brush_shape_per_layer_color();
    let s = t.brush_settings();
    if s.shape_layer_count < 2 {
        return; // capture grabbed a single layer — the 2-layer blend path isn't exercised here
    }
    t.set_brush_shape_layer_blend(1, BlendMode::Multiply.to_u8());
    let changed = t
        .layers
        .root()
        .iter()
        .filter(|&&id| {
            t.layers
                .get(id)
                .is_some_and(|l| l.blend_mode == BlendMode::Multiply)
        })
        .count();
    assert_eq!(
        changed, 1,
        "exactly the ONE source layer's blend mode changed (remote control), not the others"
    );
    assert_eq!(
        t.brush_settings().shape_layer_blend[1],
        BlendMode::Multiply.to_u8(),
        "the brush snapshot mirrors the picked blend"
    );
}

#[test]
fn manual_blend_and_opacity_reflect_in_the_snapshot_and_paint() {
    use ph2d_painter_brush::StrokeMethod;
    // The "B" blend pick + the per-layer opacity box land in the snapshot the panel reads, and the
    // stroke paints. (Texture Color is the default — `color_on` off — so a custom colour is opt-in.)
    let mut t = white_canvas(64, 6.0);
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_blend(1, 1); // Multiply on the top layer
    t.set_brush_shape_layer_opacity(0, 0.4); // brush-only opacity on the bottom layer
    let s = t.brush_settings();
    assert_eq!(
        s.shape_layer_blend[1], 1,
        "manual blend reflects in the snapshot"
    );
    assert!(
        (s.shape_layer_opacity[0] - 0.4).abs() < 1e-3,
        "per-layer opacity reflects in the snapshot: {}",
        s.shape_layer_opacity[0]
    );
    // The stroke still lands (no dead no-op).
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let c = px(&t, 64, 32, 32);
    assert!(c[3] > 0, "a per-layer-colour dab paints something: {c:?}");
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
    // Line end-to-end (tool layer), polyline model: click the first corner (a lone point paints nothing),
    // then PRESS the second corner and drag it to a WRONG spot then the final spot (each move previews the
    // straight line, restore + re-stamp → no trail), release, Enter bakes. The wrong drag leaves no trace.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    // First corner: a lone point paints nothing (< 2 points).
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 64, 8, 8),
        [255, 255, 255, 255],
        "one corner paints nothing"
    );

    // Second corner: press in empty space (creates it), drag to a WRONG spot (vertical) then the final
    // spot (horizontal), release. Enter bakes the line.
    t.on_canvas_pointer(cp([8.0, 56.0], PointerPhase::Down)); // create corner 1 (wrong: vertical)
    t.on_canvas_pointer(cp([56.0, 8.0], PointerPhase::Move)); // drag to final (horizontal)
    t.on_canvas_pointer(cp([56.0, 8.0], PointerPhase::Up));
    assert!(t.commit_open_shape(), "Enter baked the open line");

    // The committed line is horizontal at y=8 from (8,8) to (56,8).
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
fn curve_undo_walks_edits_then_undoes_the_creation() {
    // Undo must NOT Apply an open curve (Enio 2026-06-27): while authoring it walks the per-session edit
    // history, and once that's exhausted the next undo removes the curve (undoes the creation) — never bakes.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Curve;
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
    assert!(t.ellipse_overlay().is_none(), "no handles while drawing");
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
fn ellipse_discarded_when_switching_method_away() {
    let mut t = circle_tool();
    draw_circle(&mut t, 64.0, 64.0, 20.0);
    assert!(t.ellipse_overlay().is_some());
    t.set_brush_stroke_method(StrokeMethod::Space.to_u8());
    assert!(
        t.ellipse_overlay().is_none(),
        "leaving Ellipse discarded the session"
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

#[test]
fn per_layer_color_grain_stencil_masks_to_the_rect_not_the_whole_dab() {
    use ph2d_painter_brush::{Dab, StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    // Regression (Enio 2026-06-27): with Per-Layer Color ON, a Grain mapped Stencil was baked into the
    // dab-LOCAL cached coloured stamp, which can't represent the canvas-fixed rect → the colour leaked
    // OUTSIDE the Stencil (worst on a big Anchored dab). Canvas-fixed Grain now routes to the per-pixel
    // dynamic path, which masks the rect. A pixel inside the dab but outside the rect must stay white.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.3, 0.3], // central rect ≈ [22.4 .. 41.6]
        ..Default::default()
    };
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 30.0, // covers the canvas; the silhouette (full square) reaches well past the rect
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab]);
    // (32,10): inside the dab radius (dy = 22 < 30) but ABOVE the rect (y < 22.4) → masked = white.
    assert_eq!(
        px(&t, 64, 32, 10),
        [255, 255, 255, 255],
        "Per-Layer Color + Grain Stencil masks to the rect — inside the dab but outside the rect stays white"
    );
}

// Diagnostics (Enio 2026-06-28): does each per-dab rotation actually reach the painted result? Two
// strokes with DIFFERENT seeds must differ — the seed only feeds the rotation here, so identical = the
// rotation was DROPPED (the cached-vs-per-pixel path confound is avoided: both runs use the SAME path).
fn directional_bar() -> Vec<u8> {
    let mut bar = vec![0u8; 64]; // 8×8, top 3 rows white = directional under rotation
    for px in bar.iter_mut().take(3 * 8) {
        *px = 255;
    }
    bar
}

#[test]
fn jitter_rotate_reaches_curve_fill() {
    use ph2d_painter_brush::StrokeMethod;
    let bar = directional_bar();
    let run = |seed: u64| {
        let mut t = white_canvas(64, 8.0);
        t.set_brush_shape_image(bar.clone(), 8, 8);
        t.paint.brush.stroke_method = StrokeMethod::Curve;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([54.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([54.0, 32.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate must vary the Curve fill with the seed"
    );
}

#[test]
fn jitter_rotate_spins_a_flattened_falloff_with_no_texture() {
    use ph2d_painter_brush::StrokeMethod;
    // Enio 2026-06-28: Jitter Rotate spins the brush FOOTPRINT (the flatten + rotation circle), so a
    // flattened round brush with NO Texture and NO Shape still rotates per dab (the `Texture: None` case).
    let run = |seed: u64| {
        let mut t = white_canvas(64, 10.0);
        t.paint.brush.dab_flatten = 0.5; // elliptical footprint (anisotropic under rotation)
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([14.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate spins the flattened footprint even with no Texture / Shape"
    );
}

#[test]
fn jitter_rotate_reaches_the_paint() {
    use ph2d_painter_brush::StrokeMethod;
    let bar = directional_bar();
    let run = |seed: u64| {
        let mut t = white_canvas(48, 8.0);
        t.set_brush_shape_image(bar.clone(), 8, 8);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate must vary with the seed (rotation reaches the paint)"
    );
}

#[test]
fn shape_random_angle_reaches_the_paint() {
    use ph2d_painter_brush::StrokeMethod;
    let bar = directional_bar();
    let run = |seed: u64| {
        let mut t = white_canvas(48, 8.0);
        t.set_brush_shape_image(bar.clone(), 8, 8);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.toggle_brush_shape_random();
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Shape Random Angle must vary with the seed (rotation reaches the paint)"
    );
}

#[test]
fn grain_random_angle_reaches_the_paint() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind};
    let run = |seed: u64| {
        let mut t = white_canvas(48, 8.0);
        t.paint.brush.texture.kind = TextureKind::Stripes; // a directional grain
        t.paint.brush.texture.size = [0.5, 0.5];
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.toggle_brush_texture_random();
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Grain Random Angle must vary with the seed (rotation reaches the paint)"
    );
}

#[test]
fn jitter_rotate_panel_event_sets_the_field() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(48, 8.0);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_ROTATE,
        0.7,
    ));
    assert!(
        (t.brush_settings().jitter_rotate - 0.7).abs() < 1e-4,
        "the panel slider event must set jitter_rotate: {}",
        t.brush_settings().jitter_rotate
    );
}

#[test]
fn grain_ramp_stencil_does_not_paint_outside_the_rect() {
    use ph2d_painter_brush::texture::{TextureKind, TextureMapping};
    // Regression (Enio 2026-06-28): a Grain **Color Ramp** indexed by the grain value painted `ramp[0]`
    // OUTSIDE the Stencil rect — `sample()` returns 0 there, which the ramp read as a colour (not "no
    // paint"). The rect must mask the ramp path too. A central rect; a dab covers the canvas; a pixel
    // inside the dab but outside the rect must stay white.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.texture.kind = TextureKind::Noise;
    t.paint.brush.texture.mapping = TextureMapping::Stencil;
    t.paint.brush.texture.stencil_size = [0.3, 0.3]; // central rect ≈ [22.4 .. 41.6]
    t.set_texture_ramp_enabled(true); // grain value → ramp colour
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)); // radius 30 dab over the canvas
    assert_eq!(
        px(&t, 64, 32, 8),
        [255, 255, 255, 255],
        "the Grain ramp is masked by the Stencil — inside the dab but outside the rect stays white"
    );
}

#[test]
fn loading_a_grain_image_fits_the_aspect_for_each_mapping() {
    use ph2d_painter_brush::TextureMapping;
    // Enio 2026-06-28: a Grain Image is never squashed. STENCIL shapes the rect to the image (Size 1:1);
    // the other mappings put the aspect in the Grain Size (`sx:sy = h:w`).
    let mut t = white_canvas(64, 8.0);

    // Stencil: a 2:1 image → stencil_size aspect 2:1 (wider axis at the 0.5 box); Size stays 1:1.
    t.set_brush_texture_mapping(TextureMapping::Stencil.to_u8());
    t.set_brush_texture_image(vec![128u8; 32 * 16], 32, 16); // 2:1
    let b = t.brush_settings();
    assert!(
        (b.stencil_size[0] - 0.5).abs() < 1e-4 && (b.stencil_size[1] - 0.25).abs() < 1e-4,
        "2:1 image → stencil_size [0.5, 0.25]: {:?}",
        b.stencil_size
    );
    assert_eq!(
        b.texture_size,
        [1.0, 1.0],
        "Stencil image fills the rect once (Size 1:1)"
    );

    // View Plane: the aspect goes into the Grain Size — a 2:1 image → [0.5, 1.0] (h:w), so it's not squashed.
    t.set_brush_texture_mapping(TextureMapping::ViewPlane.to_u8());
    let s = t.brush_settings().texture_size;
    assert!(
        (s[0] - 0.5).abs() < 1e-4 && (s[1] - 1.0).abs() < 1e-4,
        "2:1 image (View) → Size [0.5, 1.0]: {s:?}"
    );
    // A tall 1:2 image flips it.
    t.set_brush_texture_image(vec![128u8; 16 * 32], 16, 32); // 1:2
    let s = t.brush_settings().texture_size;
    assert!(
        (s[0] - 1.0).abs() < 1e-4 && (s[1] - 0.5).abs() < 1e-4,
        "1:2 image (View) → Size [1.0, 0.5]: {s:?}"
    );
}

#[test]
fn stencil_corner_drag_with_shift_scales_uniformly() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    // Shift held → a Stencil corner scale preserves the grab-time aspect ratio (the Sprite gizmo's
    // aspect-lock): dragging mostly along X grows BOTH axes by the X factor, keeping the 2:1 rect 2:1.
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.4, 0.2], // a 2:1 rect (centred)
        ..Default::default()
    };
    let corner = t.stencil_overlay().unwrap().corners[2]; // [++] bottom-right
    t.set_uniform_scale(true);
    assert!(
        t.on_canvas_pointer(cp(corner, PointerPhase::Down)),
        "grab the corner"
    );
    t.on_canvas_pointer(cp([60.0, 34.0], PointerPhase::Move)); // grow X a lot, Y barely
    let s = t.brush_settings();
    let aspect = s.stencil_size[0] / s.stencil_size[1];
    assert!(
        (aspect - 2.0).abs() < 0.05,
        "uniform scale keeps the 2:1 aspect: {aspect} ({:?})",
        s.stencil_size
    );
}

#[test]
fn anchored_stencil_does_not_leak_outside_the_rect_during_the_drag() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    // Regression (Enio 2026-06-27): an Anchored stroke with a Grain mapped Stencil leaked colour OUTSIDE
    // the rect while dragging (the interactive preview stamped texture-free for speed → no stencil mask).
    // A small central rect; the Anchored anchor sits in a corner well outside it; after the size-drag the
    // anchor pixel (dab centre, falloff = 1) must stay white — the stencil masks the live preview too.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.3, 0.3], // central rect ≈ [22.4 .. 41.6]
        ..Default::default()
    };
    t.set_brush_stroke_method(StrokeMethod::Anchored.to_u8());
    assert!(t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down))); // anchor in the corner (outside rect)
    let _ = t.on_canvas_pointer(cp([8.0, 50.0], PointerPhase::Move)); // grow radius ≈ 42 (covers the canvas)
    assert_eq!(
        px(&t, 64, 8, 8),
        [255, 255, 255, 255],
        "the Anchored preview is masked by the stencil — the corner outside the rect stays white"
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
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Down)); // click on the curve, away from any anchor
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
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

// ============================================================================
// FASE A — Per-Layer Color perf-measurement harness.
// Tracker: docs/HANDOFF_per_layer_color_perf_artifacts.md §1 (owed numbers).
// Ignored by default. Run in RELEASE — dev (opt-0) lies about perf
// (project_painter_composite_perf):
//   cargo test -p ph2d-tool-painter --release per_layer_perf -- --ignored --nocapture
//
// Design: drive the real pointer API (no GUI needed for timing). A Curve in
// DRAW mode re-fills the whole anchor->cursor line every Move, so K identical
// Moves at a fixed cursor = pure per-move cost at a fixed geometry. Comparing a
// DIAGONAL vs a HORIZONTAL line of the SAME length (same dab count D) isolates
// the bbox-bound cost (save/restore memcpy + O(bbox.N) recomposite + the
// composite_region in take_preview_arc) from the dab-count-bound cost (the
// whole-shape re-stamp + the O(D.N.S) accumulate). N-scaling (2 vs 16 shape
// layers) isolates the x N loops from the N-independent memcpy.
// ============================================================================
#[cfg(test)]
mod per_layer_perf {
    use super::*;
    use ph2d_painter_brush::{Falloff, StrokeMethod};
    use std::time::Instant;

    /// White `size`x`size` canvas, `n_shape` full 16x16 Shape layers (distinct colours), Per-Layer
    /// Color ON, Curve method, hard disk `radius`. `doc_extra` extra raster doc layers make the
    /// document stack non-trivial (exercises `take_preview_arc`'s `composite_region` lane).
    fn setup(size: u32, n_shape: usize, doc_extra: usize, radius: f32) -> PainterTool {
        let mut t = white_canvas(size, radius);
        t.paint.brush.stroke_method = StrokeMethod::Curve;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.falloff = Falloff::Constant;
        t.paint.brush.space_attenuation = false;
        let layers: Vec<(Vec<u8>, u32, u32)> = (0..n_shape)
            .map(|_| (vec![255u8; 16 * 16], 16, 16))
            .collect();
        t.set_brush_shape_layers(layers);
        t.toggle_brush_shape_per_layer_color();
        for i in 0..n_shape {
            let f = i as f32;
            t.set_brush_shape_layer_color(
                i,
                [(f * 0.13) % 1.0, (f * 0.27) % 1.0, (f * 0.41) % 1.0],
            );
        }
        for k in 0..doc_extra {
            t.add_raster_layer(format!("L{k}"));
        }
        assert!(
            t.paint.shape_layers.is_color_mode(),
            "harness must be in per-layer-colour mode"
        );
        t
    }

    /// Time K identical full-line re-fills (the Curve draw branch re-fills anchor->p1 every Move) and
    /// the matching `take_preview_arc` drains. Returns `(avg_move_us, avg_preview_us)`.
    fn measure(t: &mut PainterTool, p0: [f32; 2], p1: [f32; 2], k: usize) -> (f64, f64) {
        t.on_canvas_pointer(cp(p0, PointerPhase::Down));
        t.on_canvas_pointer(cp(p1, PointerPhase::Move)); // establish the full line
        let _ = t.take_preview_arc(); // drain the establish frame
        let mut move_ns = 0u128;
        let mut prev_ns = 0u128;
        for _ in 0..k {
            let a = Instant::now();
            t.on_canvas_pointer(cp(p1, PointerPhase::Move));
            move_ns += a.elapsed().as_nanos();
            let b = Instant::now();
            let _ = t.take_preview_arc();
            prev_ns += b.elapsed().as_nanos();
        }
        let kf = k as f64;
        (move_ns as f64 / kf / 1000.0, prev_ns as f64 / kf / 1000.0)
    }

    /// Diagonal vs horizontal endpoints of EQUAL length `len`, inset `m` from the canvas origin.
    fn endpoints(size: u32, m: f32, len: f32, diagonal: bool) -> ([f32; 2], [f32; 2]) {
        if diagonal {
            let d = len / std::f32::consts::SQRT_2;
            ([m, m], [m + d, m + d])
        } else {
            ([m, size as f32 * 0.5], [m + len, size as f32 * 0.5])
        }
    }

    #[test]
    #[ignore = "perf measurement — run explicitly in --release"]
    fn per_layer_perf_sweep() {
        println!(
            "\n{:>6} {:>5} {:>4} {:>4} | {:>10} {:>10} | {:>10} {:>10} | {:>7}",
            "size", "r", "N", "doc", "D.move", "D.prev", "H.move", "H.prev", "D/H"
        );
        for &size in &[256u32, 1024u32] {
            let m = size as f32 * 0.1;
            let len = size as f32 * 0.6;
            for &radius in &[8.0_f32, 40.0, 100.0] {
                for &n in &[2usize, 16usize] {
                    for &doc in &[0usize, 1usize] {
                        let mut td = setup(size, n, doc, radius);
                        let (d0, d1) = (
                            endpoints(size, m, len, true).0,
                            endpoints(size, m, len, true).1,
                        );
                        let (dm, dp) = measure(&mut td, d0, d1, 30);
                        let mut th = setup(size, n, doc, radius);
                        let (h0, h1) = (
                            endpoints(size, m, len, false).0,
                            endpoints(size, m, len, false).1,
                        );
                        let (hm, hp) = measure(&mut th, h0, h1, 30);
                        println!(
                            "{size:>6} {radius:>5.0} {n:>4} {doc:>4} | {dm:>10.1} {dp:>10.1} | {hm:>10.1} {hp:>10.1} | {:>6.1}x",
                            if hm > 0.0 { dm / hm } else { 0.0 }
                        );
                    }
                }
            }
        }
        println!(
            "\nus per Move. move=curve_fill+save/restore+accumulate+recomposite; prev=take_preview_arc."
        );
        println!(
            "D/H = diagonal/horizontal at EQUAL dab count: >>1 => bbox-bound; ~1 => D.S.N-bound."
        );
        println!("time ~prop N => the xN per-layer loops dominate.\n");
    }

    /// Worst observed config (1024 r100 N16, diagonal) in isolation so `PH2D_PAINT_PROF=1` prints a
    /// clean accumulate-vs-recomposite split:
    ///   PH2D_PAINT_PROF=1 cargo test -p ph2d-tool-painter --release per_layer_perf_worst -- --ignored --nocapture
    #[test]
    #[ignore = "perf measurement — run explicitly in --release with PH2D_PAINT_PROF=1"]
    fn per_layer_perf_worst() {
        let size = 1024u32;
        let m = size as f32 * 0.1;
        let len = size as f32 * 0.6;
        let mut t = setup(size, 16, 0, 100.0);
        let (p0, p1) = (
            endpoints(size, m, len, true).0,
            endpoints(size, m, len, true).1,
        );
        let (mv, pv) = measure(&mut t, p0, p1, 8);
        println!("worst: move_us={mv:.1} prev_us={pv:.1}");
    }
}

/// `coalesces_canvas_motion` gates per-frame pointer coalescing in the shell. It must be true EXACTLY for
/// the restore + whole-shape re-stamp fill methods (latest-position-only, so coalescing is byte-identical)
/// and false for the incremental / capture methods (Space/Dots/Airbrush/Free Hand) that need every event.
/// Guards the FPS-drop fix (`HANDOFF_per_layer_color_perf_artifacts` §1.R) against a method slipping into
/// the wrong bucket (e.g. coalescing Free Hand would drop captured path points).
#[test]
fn coalesces_canvas_motion_is_true_only_for_restore_based_fill_methods() {
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(8, 2.0);
    let cases = [
        (StrokeMethod::Curve, true),
        (StrokeMethod::Ellipse, true),
        (StrokeMethod::Polygon, true),
        (StrokeMethod::Line, true),
        (StrokeMethod::Anchored, true),
        (StrokeMethod::DragDot, true),
        (StrokeMethod::Space, false),
        (StrokeMethod::Dots, false),
        (StrokeMethod::Airbrush, false),
        (StrokeMethod::FreeHand, false),
    ];
    for (method, want) in cases {
        t.paint.brush.stroke_method = method;
        assert_eq!(
            t.coalesces_canvas_motion(),
            want,
            "{method:?} coalesce bucket"
        );
    }
}

// ── Rail Shapes ⟷ Stroke:Method wiring (the tool half of the seam) ────────────────────────────────

#[test]
fn stroke_method_channel_sets_shapes_and_the_brush_sentinel_restores_the_last_non_shape() {
    // The tool rail drives the SAME PAINTER_BRUSH_STROKE_METHOD channel as the Method dropdown: a shape's
    // wire u8 selects it; the sentinel "brush" (the rail Brush button) restores the last NON-shape method.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let sm = |t: &PainterTool| t.paint.brush.stroke_method;
    let set = |t: &mut PainterTool, v: &str| {
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            v.to_string(),
        ));
    };
    // Choose a non-shape method (Dots = 0) — becomes the remembered "resting" method.
    set(&mut t, "0");
    assert_eq!(sm(&t), StrokeMethod::Dots);
    // Pick a shape (Ellipse = 7) — the method switches, but the non-shape memory is untouched.
    set(&mut t, "7");
    assert_eq!(sm(&t), StrokeMethod::Ellipse);
    // The Brush button (sentinel "brush") restores the last non-shape method (Dots), NOT the default.
    set(&mut t, "brush");
    assert_eq!(
        sm(&t),
        StrokeMethod::Dots,
        "Brush restored the last non-shape method"
    );
    // Another shape, then Brush again → still Dots (the memory persists across shape excursions).
    set(&mut t, "9"); // FreeHand
    assert_eq!(sm(&t), StrokeMethod::FreeHand);
    set(&mut t, "brush");
    assert_eq!(sm(&t), StrokeMethod::Dots);
}

#[test]
fn brush_sentinel_restores_space_when_no_non_shape_was_chosen() {
    // Fresh tool → a shape → Brush restores the default resting method (Space), never a shape.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        "8".to_string(), // Polygon
    ));
    assert_eq!(t.paint.brush.stroke_method, StrokeMethod::Polygon);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        "brush".to_string(),
    ));
    assert_eq!(t.paint.brush.stroke_method, StrokeMethod::Space);
}

// ── Line polyline editor (core: multi-click points, end/close, drag, commit/cancel/undo) ──────────

/// A `PainterTool` on a 64² white canvas set to the Line method, with a known grab tolerance.
fn line_tool() -> PainterTool {
    let mut t = white_canvas(64, 5.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_shape_grab_tol_px(8.0);
    t
}

fn click(t: &mut PainterTool, x: f32, y: f32) {
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
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

#[test]
fn tools_keep_independent_brush_settings_by_default() {
    // Default model: each paint tool has its OWN BrushSpec; a mode switch swaps slots, so editing one
    // tool never bleeds into another. (`white_canvas` seeds every slot to the fixture 6.0, so the split
    // below is created by the test's own edits, not the fixture.)
    let mut t = white_canvas(32, 6.0);
    assert!(
        !t.link_shared_settings(),
        "independent (unlinked) by default"
    );
    t.paint.brush.radius_px = 20.0; // Brush (Paint) size
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 6.0,
        "Mask uses its own size (fixture 6), not the Brush's 20"
    );
    t.paint.brush.radius_px = 3.0; // edit Mask only
    t.set_paint_tool_mode("brush");
    assert_eq!(
        t.paint.brush.radius_px, 20.0,
        "the Brush size survived the Mask detour"
    );
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 3.0,
        "Mask kept its own edited size"
    );
}

#[test]
fn syncing_shares_settings_and_seeds_from_the_checked_panel() {
    let mut t = white_canvas(32, 6.0);
    // Give Brush and Mask independent sizes.
    t.paint.brush.radius_px = 20.0; // Brush
    t.set_paint_tool_mode("mask");
    t.paint.brush.radius_px = 3.0; // Mask
    t.set_paint_tool_mode("brush");
    assert_eq!(t.paint.brush.radius_px, 20.0);
    // Check "Sync with other tools" on the Brush panel → it configures the others.
    t.toggle_link_shared_settings();
    assert!(t.link_shared_settings());
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 20.0,
        "linked: Mask now shows the checked (Brush) panel's size, not its old 3"
    );
    // While linked, editing any tool changes the shared value seen by all.
    t.paint.brush.radius_px = 12.0;
    t.set_paint_tool_mode("brush");
    assert_eq!(
        t.paint.brush.radius_px, 12.0,
        "linked: editing Mask also changed the Brush"
    );
    // Uncheck → every tool keeps the current shared value, then diverges.
    t.toggle_link_shared_settings();
    assert!(!t.link_shared_settings());
    t.paint.brush.radius_px = 7.0; // edit Brush only
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 12.0,
        "unlinked: Mask kept the last shared value (12), not the Brush's new 7"
    );
}

#[test]
fn sync_checkbox_click_routes_to_the_link_toggle() {
    // Guards the panel→tool wiring: a Click on PAINTER_BRUSH_SYNC reaches toggle_link_shared_settings
    // through route_brush_dab_event.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 4.0);
    assert!(!t.link_shared_settings());
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYNC));
    assert!(
        t.link_shared_settings(),
        "the Sync checkbox click toggled the link on"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYNC));
    assert!(!t.link_shared_settings(), "clicking again toggled it off");
}

#[test]
fn rebinding_a_sprite_abandons_a_pending_fill_and_disarms_the_eyedropper() {
    // The Enio 2026-07-02 lifecycle bug: deleting a sprite (Painter active) then selecting another used
    // to carry a pending Fill ColorDrop + an armed Eyedropper onto the new sprite — the Fill flooded it
    // BLACK and the pick swallowed the next Down ("can't paint"). Binding a new document must clear both.
    let mut t = white_canvas(16, 4.0); // black brush, white canvas
    t.set_paint_tool_mode("fill");
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down)); // arm a ColorDrop (fill_begin_drop)
    assert!(
        t.has_active_fill(),
        "a ColorDrop is pending on the old sprite"
    );
    t.paint.eyedropper_armed = true; // also arm the Eyedropper
    // Model delete-then-select-another by binding a fresh mid-grey document.
    <PainterTool as RasterEditTool>::set_source(&mut t, vec![128u8; 16 * 16 * 4], 16, 16);
    assert!(
        !t.has_active_fill(),
        "the stale ColorDrop was abandoned on rebind"
    );
    assert!(
        !t.paint.eyedropper_armed,
        "the Eyedropper was disarmed on rebind"
    );
    // A stray Fill modal slider drag can no longer flood the newly-bound sprite (fill_seed is gone).
    t.set_fill_threshold(0.9);
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 128),
        "the new sprite is intact — not flooded black by the leaked fill"
    );
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
    // The ellipse's RIGHT axis handle sits at centre + rx = (48,32); drag it out to grow rx.
    let rh = t.selection_gizmos()[0].square_handles[0];
    t.on_canvas_pointer(cp(rh, PointerPhase::Down));
    t.on_canvas_pointer(cp([rh[0] + 12.0, rh[1]], PointerPhase::Move));
    t.on_canvas_pointer(cp([rh[0] + 12.0, rh[1]], PointerPhase::Up));
    assert!(
        selected_area(&t, 64) > before,
        "dragging the axis handle enlarges the selection"
    );
}

/// A Rect selection carries the Polygon gizmo (rotate + sides circle handles) and fills a rectangle
/// (its CORNERS are selected — an ellipse's would not be).
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
        g.circle_handles.len(),
        1,
        "the polygon gizmo has a round rotate handle"
    );
    assert_eq!(
        g.diamond_handles.len(),
        1,
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
    t.selection_simplify_curve();
    assert_eq!(
        t.selection_coverage_at(48, 48),
        255,
        "region survives simplify"
    );
    assert!(
        t.selection_gizmos()[0].square_handles.len() >= 3,
        "simplify keeps a valid closed curve"
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

/// ADR-0103 Wave 5: **Copy** then **Paste** re-applies the captured pixels at their source location.
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
    t.selection_paste(); // re-apply the copied black pixels
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
