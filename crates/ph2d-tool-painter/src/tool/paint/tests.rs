use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::{DepthSource, DrawTo, Falloff};

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

/// A momentary ColorDrop (the shell's C&F drag) enters Fill remembering the prior tool, then RETURNS to it
/// when the fill finalizes (modal Done / dwell release) — so a drag-fill doesn't strand the user in Fill.
#[test]
fn colordrop_fill_returns_to_the_prior_shape_tool() {
    let size = 32u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 12..20 {
        for x in 12..20 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("brush");
    assert_eq!(t.active_paint_mode_id(), "brush");
    t.begin_colordrop_fill("brush"); // the drag reached the canvas → momentary Fill
    assert_eq!(
        t.active_paint_mode_id(),
        "fill",
        "the ColorDrop drag activates Fill"
    );
    t.paint.fill_threshold = 0.05;
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down)); // flood
    assert!(t.has_active_fill());
    t.fill_commit(); // modal Done / dwell release finalizes the fill
    assert_eq!(
        t.active_paint_mode_id(),
        "brush",
        "the finalized ColorDrop returns to the shape/brush tool"
    );
}

/// A C&F drag that entered Fill but never reached the canvas (released off-sprite → the picker path) still
/// restores the prior tool, so it isn't stranded in Fill.
#[test]
fn colordrop_missed_canvas_restores_the_prior_tool() {
    let mut t = white_canvas(32, 4.0);
    t.set_paint_tool_mode("brush");
    t.begin_colordrop_fill("brush");
    assert_eq!(t.active_paint_mode_id(), "fill");
    t.restore_after_colordrop(); // the shell's picker-path restore for a missed drag
    assert_eq!(t.active_paint_mode_id(), "brush");
    // Idempotent: a plain click (no return recorded) is a no-op.
    t.restore_after_colordrop();
    assert_eq!(t.active_paint_mode_id(), "brush");
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

/// Build the reported context: a live Per-Layer Colour freehand stroke on the CACHED route (two layers
/// with custom colours ⇒ 1 B/px coverage maps, no per-dab dynamics yet).
#[cfg(test)]
fn per_layer_live_stroke() -> PainterTool {
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space; // incremental freehand
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // bottom red
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]); // top green
    t
}

#[cfg(test)]
fn live_dab(x: f32) -> ph2d_painter_brush::Dab {
    ph2d_painter_brush::Dab {
        center: [x, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
    }
}

#[test]
fn per_layer_color_route_flip_mid_stroke_reshapes_the_maps() {
    // Enio's PANIC (2026-07-12), painting a live freehand stroke in Per-Layer Colour and pressing Shape
    // **Rake**: `range end index 3911680 out of range for slice of length 1048576`
    // (`accumulate_batch.rs`) → SIGSEGV. Rake flips the route from the cached path (maps = 1 B/px coverage)
    // to the per-dab dynamic path (maps = 4 B/px premul RGBA), and the reuse guard only asked
    // "initialised?" (`pre.is_empty()`) / "layer count changed?" (`cov.len() != n`) — never the ELEMENT
    // SIZE. The dynamic route then sliced the previous route's `w*h` maps as if they were `w*h*4`: an
    // out-of-bounds slice, not a wrong pixel. (3911680 = row 955 × stride 4096; 1048576 = 1024².)
    // RED without the fix: the second `stamp_dabs` PANICS here.
    let mut t = per_layer_live_stroke();
    t.stamp_dabs(&[live_dab(24.0)]); // batch 1 — the cached route allocates the 1 B/px maps
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64,
        "the cached route's maps are 1 B/px coverage"
    );
    t.paint.brush.shape.rake = true; // the user presses Rake with the stroke STILL LIVE → the route flips
    t.stamp_dabs(&[live_dab(40.0)]); // batch 2 — the dynamic route: this is where it blew up
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64 * 4,
        "the flipped-to dynamic route re-shaped the maps to 4 B/px premul RGBA"
    );
    assert!(
        px(&t, 64, 40, 32)[3] > 0,
        "the dab painted after the mid-stroke route flip"
    );
}

#[test]
fn switching_sprite_while_the_paint_is_still_wet_does_not_index_the_old_moisture_map() {
    // Sweep finding (2026-07-12), same family as Bug #12: `canvas_wet` is the ONE canvas-sized buffer that
    // SURVIVES pen-up (the moisture map dries on the heartbeat, over ~10 s). `dry_canvas_wet` guards it
    // with `is_empty()` — "does it exist?" — and then indexes it with the CURRENT sprite's stride (`fw`)
    // and a `canvas_wet_rect` recorded in the OLD sprite's coordinates. Bind a BIGGER sprite inside the
    // drying window and the very next tick slices past the end of the old buffer.
    // RED without the fix: `paint_tick` PANICS (`range end index … out of range for slice of length 4096`)
    // — the same signature class as Enio's Rake crash, from the same root: a guard that asks "exists?"
    // instead of "does the SHAPE match?".
    let mut t = white_canvas(64, 8.0);
    t.paint.brush.watercolor = true;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Up));
    assert_eq!(
        t.paint.canvas_wet.len(),
        64 * 64,
        "the wet stroke left a moisture map sized for the 64² sprite"
    );
    assert!(
        t.paint.canvas_wet_rect.is_some(),
        "and a wet rect in the 64² sprite's coordinates"
    );
    // The user clicks a BIGGER sprite while the paint is still drying — one click, nothing else.
    t.bind_document(2, vec![255u8; 512 * 512 * 4], 512, 512);
    t.paint_tick(0.1); // the heartbeat the shell runs every frame → dry_canvas_wet
    assert!(
        t.paint.canvas_wet.is_empty() || t.paint.canvas_wet.len() == 512 * 512,
        "the new sprite must not inherit a moisture map shaped for the old one"
    );
}

#[test]
fn editing_the_paper_re_renders_the_wet_wash_with_the_new_paper() {
    // Sweep finding (2026-07-12), found INDEPENDENTLY by two lenses. The live-editable wash (2026-07-11)
    // re-renders the committed pool when a Grain/Paper param moves while the paper is still wet —
    // `rerender_editable_wash`'s own doc says it reconstructs "with the CURRENT brush texture". But the
    // paper-tooth memo (`wet_substrate`) is only NaN-reset at PEN-DOWN, and `fill_substrate_cache` fills
    // only the NaN misses — so every pixel of the pool keeps the paper height computed for the OLD paper.
    // The field's doc-comment asserts "the paper cannot change mid-stroke, so there is no in-stroke
    // invalidation to get wrong" — the live-edit feature made that premise false, and defeated ITSELF for
    // the Paper slot (the Grain works, which is why the smoke passed).
    // RED without the fix: the canvas is byte-identical after moving Paper Size.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        granulation: 0.9, // the substrate has to WEIGH on the bake, else nothing is observable
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::Voronoi; // a lattice paper: Size genuinely changes the tooth
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    t.paint.brush.paper_depth = 1.0;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let memo_before: Vec<f32> = t.paint.wet_substrate.clone();
    let memoised = memo_before.iter().filter(|v| !v.is_nan()).count();
    assert!(
        memoised > 0,
        "the wash memoised the paper tooth under its footprint"
    );
    // The paper is still wet. The user drags Paper Size — the live-edit feature's whole reason to exist.
    t.set_brush_paper_size(0, 24.0);
    t.set_brush_paper_size(1, 24.0);
    t.paint_tick(0.016); // the heartbeat → rerender_editable_wash
    // THE ORACLE IS THE MEMO, not the pixels. A pixel-level `assert_ne!` goes GREEN for the wrong reason:
    // the re-render forces a bigger dirty region, so freshly-filled (NaN) pixels move bytes around even
    // while every ALREADY-memoised pixel keeps the old paper. Compare only the pixels that were already
    // memoised — those are the ones the staleness hides in.
    let stale = t
        .paint
        .wet_substrate
        .iter()
        .zip(memo_before.iter())
        .filter(|(now, was)| !was.is_nan() && now.to_bits() == was.to_bits())
        .count();
    assert_eq!(
        stale, 0,
        "every memoised paper-tooth sample must be rebuilt for the new Paper          ({stale}/{memoised} still hold the OLD paper's tooth)"
    );
}

#[test]
fn jitter_rotate_reaches_smear_on_a_flattened_untextured_dab() {
    // Sweep (2026-07-12): `has_per_dab_rotation()` demanded `texture.is_active()`, so a FLATTENED dab with
    // no Shape and no Grain looked "constant" to the guard — and Smear/Blur/Clone served it the cached,
    // constant-orientation StampMask. Every dab smeared with the SAME ellipse angle: Jitter Rotate did
    // nothing there. (The paint path never had it — with both slots off it has no cache to serve, so
    // `jitter_rotate_spins_a_flattened_falloff_with_no_texture` has always passed. This is its Smear twin.)
    // Jitter Rotate spins the whole FOOTPRINT, so an anisotropic footprint alone makes it visible.
    // RED without the fix: the two canvases are byte-identical.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::StrokeMethod;
    let size = 48u32;
    let smear = |seed: u64| -> Vec<u8> {
        let mut t = PainterTool::default();
        // Left half black, right half white — the SAME fixture the working Smear gate uses: the dab has to
        // straddle a BOUNDARY, because smearing inside a uniform region is a no-op at any angle.
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
            stroke_method: StrokeMethod::Space, // allows_jitter
            dab_flatten: 0.6,                   // anisotropic ⇒ a per-dab rotation is VISIBLE
            jitter_rotate: 1.0,
            ..Default::default()
        };
        // The tool keeps a brush PER MODE — seed every slot, or selecting Smear swaps the settings out.
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        t.paint.seed = seed; // the jitter draw
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_PAINT_MODE,
            "smear".to_string(),
        ));
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([(size / 2 - 6) as f32, mid], PointerPhase::Down));
        t.on_canvas_pointer(cp([(size / 2 + 8) as f32, mid], PointerPhase::Move));
        t.on_canvas_pointer(cp([(size / 2 + 8) as f32, mid], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    // Guard against a no-op fixture proving nothing: the smear must actually have dragged the boundary.
    let mut pristine = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            pristine[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
    let a = smear(1);
    assert_ne!(a, pristine, "the smear actually dragged pixels");
    assert_ne!(
        a,
        smear(999),
        "Jitter Rotate must reach the Smear of a flattened, untextured dab"
    );
}

#[test]
fn flipping_per_layer_color_mid_stroke_keeps_what_was_already_painted() {
    // Sweep (2026-07-12). Same seam as Bug #12 — the panel is live while the canvas is — but a different
    // failure: not the maps' SHAPE, the stroke's CONTINUITY. `pre` is the PRE-stroke canvas snapshot; a dab
    // painted while Per-Layer Color was OFF went straight to `canvas_rgba`, so it is in neither `pre` nor
    // the coverage maps. Turning the mode back ON, the next batch recomposites its bbox from `pre` — and
    // the off-interval dab EVAPORATES.
    // RED without the fix: the pixel painted with the mode off is white again.
    use ph2d_painter_brush::{Dab, StrokeMethod};
    let mut t = white_canvas(64, 8.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color = [0.0, 0.0, 1.0]; // blue — the plain-route dab
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    let dab = |cx: f32, coverage: f32| Dab {
        center: [cx, 32.0],
        radius_px: 8.0,
        coverage,
        color: [0.0, 0.0, 1.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
    };
    t.stamp_dabs(&[dab(16.0, 1.0)]); // per-layer ON — this seeds `pre` (the canvas BEFORE it: all white)
    t.toggle_brush_shape_per_layer_color(); // OFF, stroke still live
    t.stamp_dabs(&[dab(32.0, 1.0)]); // opaque BLUE, straight to the canvas — in neither `pre` nor the maps
    assert!(
        px(&t, 64, 32, 32)[2] > 200 && px(&t, 64, 32, 32)[0] < 60,
        "the mode-off dab painted blue"
    );
    t.toggle_brush_shape_per_layer_color(); // back ON
    // A SEMI-TRANSPARENT dab over the same spot. Its recomposite rebuilds the region as `pre ⊕ layers` —
    // and where the layers are only 30% opaque, 70% of what shows through is the BASE. If the base is the
    // stale pre-stroke snapshot, that 70% is WHITE and the blue dab is gone. (An opaque dab would hide the
    // bug: it overwrites what is under it anyway, which is just normal painting.)
    t.stamp_dabs(&[dab(32.0, 0.3)]);
    let p = px(&t, 64, 32, 32);
    assert!(
        p[0] < 60,
        "the dab painted with the mode OFF must survive the flip back ON — its blue must still be the \
         base under the translucent dab, not rebuilt away from a stale `pre` (red channel {}, so the base \
         went back to WHITE)",
        p[0]
    );
}

#[test]
fn appearance_signature_tracks_tiling_and_ramp_alpha() {
    // Sweep (2026-07-12): `AppearanceSig` is the change detector that re-fills an OPEN shape editor's
    // preview (`on_panel_event` compares it and calls `refill_open_shape`). It missed the two ramps'
    // **Alpha Mode** and **Tiling** — both because their setters deliberately say "no re-bake needed, it
    // only affects future stamps". True of the LUT and of the canvas; NOT true of an open shape, whose
    // preview IS a stamp that has not landed yet. Toggling Tiling (or Alpha Mode) with a Curve on screen
    // changed nothing until some unrelated knob moved.
    //
    // The gate is on the SIGNATURE, which is precisely the defect: if it does not move, the refill is never
    // even called. (Driving the repaint end-to-end needs an open-shape state this harness does not
    // reproduce, and a test whose behaviour I cannot explain is worth less than one that pins the cause.)
    // RED without the fix: the signature is unchanged and the preview stays stale.
    use ph2d_painter_brush::RampAlphaMode;
    let mut t = white_canvas(64, 8.0);
    let sig = t.appearance_sig();
    t.toggle_brush_tiling(0);
    assert!(
        sig != t.appearance_sig(),
        "Tiling wraps an open shape's stamp — it MUST be in the appearance signature"
    );
    let sig = t.appearance_sig();
    t.set_texture_ramp_alpha_mode(RampAlphaMode::Strength.to_u8());
    assert!(
        sig != t.appearance_sig(),
        "the Grain ramp's Alpha Mode is applied at STAMP time — it MUST be in the appearance signature"
    );
    let sig = t.appearance_sig();
    t.set_shape_ramp_alpha_mode(RampAlphaMode::Strength.to_u8());
    assert!(
        sig != t.appearance_sig(),
        "the Shape ramp's Alpha Mode is applied at STAMP time — it MUST be in the appearance signature"
    );
}

#[test]
fn paper_depth_and_granulation_re_render_the_wet_wash() {
    // Sweep (2026-07-12): the live-editable wash's change detector was `(Grain, Paper)` `TextureSettings`
    // only. But `apply_watercolor` also reads **Paper Depth** and **Granulation**, which live on
    // `BrushSpec`, NOT inside `TextureSettings`. So dragging Paper *Size* re-rendered the wet pool and
    // dragging Paper *Depth* — the slider right next to it — did nothing: the same gesture, two different
    // behaviours, side by side. (Swapping the Paper/Grain IMAGE while keeping `kind: Image` was invisible
    // too: no setting changes, only the pixel version.)
    // RED without the fix: the canvas is byte-identical after moving Paper Depth.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        granulation: 0.9,
        paper_depth: 1.0,
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::Voronoi;
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let before = (*t.canvas_rgba).clone();
    // The paper is still wet. The user drags Paper Depth — nothing else moves.
    t.set_brush_paper_depth(0.0);
    t.paint_tick(0.016); // the heartbeat → rerender_editable_wash
    assert_ne!(
        before,
        (*t.canvas_rgba).clone(),
        "Paper Depth is read by the composite — it must re-render the wet pool like Paper Size does"
    );
}

#[test]
fn grain_rake_and_random_are_inert_under_the_wash() {
    // Enio asked to close the sweep's open findings. This one is the Grain twin of the Paper Rake removal.
    // With Watercolor on, the Grain slot IS the granulation map — a CANVAS-ANCHORED substrate saying where
    // pigment settles — not a stamp the dab carries. The composite samples it through
    // `angle_basis(texture.angle_deg)`: no `d.dir`, no rng. So "Rake" (follow the stroke) and "Random
    // Angle" (fresh angle per dab) have nothing to rotate, and the same checkbox meant two different things
    // depending on the Watercolor tick — one of them being "nothing".
    // This pins the deadness that justifies hiding them (a hidden knob must be PROVABLY inert), and it
    // fails the day someone wires per-dab Grain frames into the wash — at which point the panel must
    // show them again.
    use ph2d_painter_brush::{TextureKind, TextureSettings};
    let wash = |rake: bool, random: bool| -> Vec<u8> {
        let mut t = white_canvas(64, 9.0);
        t.paint.brush = BrushSpec {
            radius_px: 9.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            depth: 1.5,
            granulation: 0.9, // the Grain has to WEIGH on the bake, else the test proves nothing
            texture: TextureSettings {
                kind: TextureKind::Noise,
                size: [6.0, 6.0],
                rake,
                random_angle: random,
                ..Default::default()
            },
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down));
        for i in 1..10u16 {
            t.on_canvas_pointer(cp([16.0 + 4.0 * f32::from(i), 32.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let plain = wash(false, false);
    assert!(
        plain.chunks_exact(4).any(|p| p[0] != 255 || p[1] != 255),
        "the wash actually painted (guard against a fixture that proves nothing)"
    );
    assert_eq!(
        plain,
        wash(true, false),
        "Grain Rake is inert under the wash"
    );
    assert_eq!(
        plain,
        wash(false, true),
        "Grain Random Angle is inert under the wash"
    );
}

#[test]
fn shape_layer_opacity_reaches_the_flattened_silhouette() {
    // Sweep (2026-07-12): `ShapeLayers::flatten()` scales each layer by `opacity[i]` — but `set_layers()`
    // RESETS the opacities to 1.0, and the capture only installs the real ones afterwards
    // (`set_layers_meta`). So the flatten always baked against all-1.0. And `set_opacity` never re-flattened
    // at all: the per-layer **Opacity** box was DEAD everywhere except Per-Layer Color mode (which applies
    // opacity at recomposite time and so bypasses the flatten entirely — which is exactly why nobody
    // noticed). The `op` term inside `flatten()` was unreachable code.
    // RED without the fix: the flattened silhouette is byte-identical after zeroing a layer's opacity.
    let mut t = white_canvas(64, 9.0);
    // Two DISJOINT halves. (A full bottom layer would over-composite to saturation and the top layer's
    // opacity could not change the result at ALL — the fixture would prove nothing.)
    let mut left = vec![0u8; 64];
    let mut right = vec![0u8; 64];
    for y in 0..8 {
        for x in 0..4 {
            left[y * 8 + x] = 255;
            right[y * 8 + x + 4] = 255;
        }
    }
    t.set_brush_shape_layers(vec![(left, 8, 8), (right, 8, 8)]);
    let before = t
        .brush_shape_image()
        .expect("a shape image was flattened")
        .0
        .to_vec();
    // Drop the top layer to fully transparent — it must vanish from the silhouette.
    t.set_brush_shape_layer_opacity(1, 0.0);
    let after = t
        .brush_shape_image()
        .expect("a shape image was flattened")
        .0
        .to_vec();
    assert_ne!(
        before, after,
        "layer Opacity scales the flattened silhouette — the box must re-bake it"
    );
}

#[test]
fn granulation_re_bakes_the_coloured_stamp() {
    // Sweep (2026-07-12): `render_color_stamp_mask` folds `effective_granulation()` into the baked Grain
    // coverage — so Granulation is an INPUT of the bake — but `ColorStampKey` did not carry it. The
    // grayscale `StampKey` always has (with a comment saying exactly why); the coloured twin was written
    // without it. Dragging Granulation left the coloured stamp STALE until some other field moved the key.
    //
    // ★ The test MUST reuse ONE tool. Baking on two fresh tools proves nothing — each starts with a cold
    // cache, so it re-bakes either way and the test goes green with the bug alive (it did, for me, until I
    // ran the RED). A cache-key gate has to exercise the CACHE HIT.
    // RED without the fix: the second bake is byte-identical — the key matched and the stale stamp was reused.
    use ph2d_painter_brush::{TextureKind, TextureSettings};
    let mut t = white_canvas(64, 9.0);
    t.paint.brush.watercolor = true; // `effective_granulation()` is 0 unless watercolor is on
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        size: [6.0, 6.0],
        ..t.paint.brush.texture
    };
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    let stamp_bytes = |t: &PainterTool| -> Vec<u8> {
        t.paint
            .color_stamp_cache
            .as_ref()
            .expect("stamp baked")
            .0
            .iter()
            .flat_map(|s| s.data().to_vec())
            .collect()
    };
    t.paint.brush.granulation = 0.0;
    let brush = t.paint.brush;
    t.ensure_color_stamp_cache(&brush, 64);
    let before = stamp_bytes(&t);
    // The user drags Granulation — nothing else moves.
    t.paint.brush.granulation = 0.9;
    let brush = t.paint.brush;
    t.ensure_color_stamp_cache(&brush, 64); // must MISS the cache and re-bake
    assert_ne!(
        before,
        stamp_bytes(&t),
        "Granulation is folded into the baked Grain coverage — it MUST be in the stamp's key"
    );
}

#[test]
fn per_layer_color_grain_random_offset_takes_the_per_dab_route() {
    // Sweep (2026-07-12): the per-layer route hand-rolled its "can the constant coloured stamp express this
    // Grain?" test as `Rake || Random-Angle || canvas-fixed`. Grain **Mapping = Random Offset** randomises
    // the per-dab OFFSET, not the angle — it matched none of those clauses, so the CONSTANT stamp was baked
    // once and blitted for every dab: the texture "sticks" to the dab instead of jittering. The canonical
    // predicate is `!texture.is_cacheable()`, which the grayscale routes had always used and which covers
    // Rake, Random-Angle, Random-Offset, Tiled and Stencil in one place.
    //
    // ORACLE = the ROUTE, read from real state, not a re-derived predicate: the per-layer maps are 4 B/px
    // premul RGBA on the per-dab dynamic route and 1 B/px coverage on the constant cached one (the very
    // asymmetry behind Bug #12). Asserting the canvas instead would be nicer, but a full-opacity two-layer
    // silhouette swamps the grain's contribution in this fixture — and a test whose green I cannot explain
    // is worse than one that pins exactly the defect.
    // RED without the fix: the maps come back 1 B/px — the constant stamp served a per-dab Grain.
    use ph2d_painter_brush::{Dab, StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 9.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Random, // randomises the OFFSET per dab — no angle involved
        size: [6.0, 6.0],
        ..t.paint.brush.texture
    };
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    let dab = |cx: f32| Dab {
        center: [cx, 32.0],
        radius_px: 9.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
    };
    t.stamp_dabs(&[dab(18.0), dab(46.0)]);
    assert_eq!(
        t.paint
            .per_layer_stroke
            .cov
            .first()
            .map_or(0, std::vec::Vec::len),
        64 * 64 * 4,
        "Random Offset must route to the per-dab dynamic path (4 B/px maps), not the constant cached stamp"
    );
}

#[test]
fn under_the_wash_accumulate_is_inert_but_strength_is_not() {
    // Enio (2026-07-12): "no modo aquarela, faz sentido ter Strength e Accumulate no painel?"
    // The answer differs for the two, and this pins BOTH — a hidden knob must be provably dead, and a
    // kept knob must be provably alive, or the panel is lying either way.
    //
    // ACCUMULATE is dead: it is read ONLY by `accumulate_cap` inside the stamp routing, and the wash
    // short-circuits before that (`stamp_dabs`). It is also redundant by construction — the wash's
    // coverage is MAX-blended (an envelope), which already IS "no build-up within one stroke".
    // STRENGTH is alive: the stroke engine bakes it into `Dab.coverage`, and the wash reads it as the
    // deposit peak (`coverage × (1 − Dilution)`).
    //
    // This also settles that EVERY stroke method washes: the shape editors run the optics through
    // `stamp_drag_preview_watercolor` (doc 13 #3), so there is no method where Accumulate comes back.
    let wash = |strength: f32, accumulate: bool| -> Vec<u8> {
        let mut t = white_canvas(64, 10.0);
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            depth: 1.5,
            strength,
            accumulate,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([36.0, 36.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([36.0, 36.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_eq!(
        wash(0.6, true),
        wash(0.6, false),
        "Accumulate is INERT under the wash — hiding the checkbox removes nothing"
    );
    assert_ne!(
        wash(0.35, true),
        wash(0.9, true),
        "Strength is ALIVE under the wash (it is the deposit peak) — the slider must STAY"
    );
}

#[test]
fn tiling_wrapped_copies_share_the_dabs_random_frame() {
    // Sweep finding (2026-07-12). Under Tiling the canvas is a TORUS: a dab crossing an edge is drawn as
    // two `Dab`s (`tiling::tiled_dabs` replicates the list), but they are the SAME dab seen from both
    // sides — so they must share one random frame. They did not: the paint routes iterate the already-
    // wrapped list and draw from `tex_rng` PER COPY, so Shape/Grain Random-Angle (and Randomize Color)
    // gave each side of the seam a different draw. The seam stops matching — which breaks the whole
    // promise of seamless Tiling. Smear/Blur/Clone already do it right and say so out loud:
    // "Computed ONCE per dab, so the wrapped Tiling copies share the same random frame."
    //
    // ORACLE: paint the SAME dab twice from a fresh tool — once centred (no wrap) as the reference, once
    // straddling the left edge with Tiling on. On a torus the tiled canvas must be the reference disc,
    // translated. The wrapped band is where the second (spurious) rng draw shows up.
    // RED without the fix: the wrapped band differs from the reference.
    use ph2d_painter_brush::{Dab, TextureKind, TextureMapping, TextureSettings};
    let brush_with_random_grain = |t: &mut PainterTool| {
        t.paint.brush.texture = TextureSettings {
            kind: TextureKind::Noise,
            mapping: TextureMapping::ViewPlane, // dab-local ⇒ the grain rotates WITH the dab
            random_angle: true,                 // ← the per-dab draw the copies must SHARE
            ..t.paint.brush.texture
        };
    };
    let dab_at = |cx: f32| Dab {
        center: [cx, 32.0],
        radius_px: 12.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
    };
    // Reference: the same dab, centred, no tiling → the disc's own random frame.
    let mut r = white_canvas(64, 12.0);
    brush_with_random_grain(&mut r);
    r.stamp_dabs(&[dab_at(32.0)]);
    // Under test: the dab straddles the LEFT edge, Tiling X on → a wrapped copy lands on the right edge.
    let mut t = white_canvas(64, 12.0);
    brush_with_random_grain(&mut t);
    t.paint.tiling = [true, false];
    t.stamp_dabs(&[dab_at(2.0)]);
    // Map canvas → dab-local, carefully (an off-by-one here fails on ANY implementation and would be a
    // false RED): the dab straddling the left edge sits at x=2, so its wrapped copy is centred at 2+64=66.
    // A canvas pixel x in the wrapped band is dab-local `x-66`, which the centred reference holds at
    // `32+(x-66) = x-34`. The central band is dab-local `x-2`, held by the reference at `x+30`.
    let mut mismatch = 0;
    for y in 24..40u32 {
        for x in 54..64u32 {
            if px(&t, 64, x, y) != px(&r, 64, x - 34, y) {
                mismatch += 1; // the WRAPPED copy — where the spurious second rng draw lands
            }
        }
        for x in 0..14u32 {
            if px(&t, 64, x, y) != px(&r, 64, x + 30, y) {
                mismatch += 1; // the central copy — must equal the reference too (same first draw)
            }
        }
    }
    assert_eq!(
        mismatch, 0,
        "the wrapped copy must use the SAME random frame as its central self \
         ({mismatch} texels of the seam band disagree with the reference disc)"
    );
}

#[test]
fn switching_sprite_drops_the_compositor_cut_cache() {
    // Sweep finding (2026-07-12) — the THIRD instance of the Bug #12 family, and the nastiest, because the
    // same-size case does not crash: it corrupts in silence.
    //
    // The compositor caches a "cut point" per Adjustment layer: the composited accumulator BELOW it, a
    // `Vec<[f32;4]>` sized for THAT document's canvas. `set_source` (bind another sprite) builds a fresh
    // `LayerStack` — and `LayerStack::new()` restarts `next_id` at 1, so the new document's layer ids
    // COLLIDE with the old one's by construction. The cut cache was never cleared, and its guard only asks
    // "is there a cut for this id?", never "does that cut have the shape of THIS canvas?".
    //
    // The asymmetry is the tell: `restore_doc` clears compositor_cache / adjustment_cache_pending /
    // dirty_rect / preview_upload_bbox — `set_source` cleared none of them. Same seam, two doors, one
    // locked. Bigger sprite ⇒ the accumulator is indexed past its end (panic). Same size ⇒ the new sprite's
    // Adjustment composites over the OLD sprite's cached layers-below: a silently wrong preview.
    // RED without the fix: `cuts` is non-empty after the rebind (and the 1024² step panics).
    use ph2d_painter_effects::adjustments::AdjustmentKind;
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 256 * 256 * 4], 256, 256);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::BrightnessContrast)
        .expect("adjustment added");
    t.set_adjustment_param(adj, 0, 0.8);
    let _ = t.take_preview_arc(); // drains the composite → seeds the cut cache for sprite 1
    assert!(
        !t.compositor_cache.cuts.is_empty(),
        "sprite 1 seeded a cut-point cache sized for its 256\u{b2} canvas"
    );
    // The user clicks a BIGGER sprite.
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    assert!(
        t.compositor_cache.cuts.is_empty(),
        "the cut cache is DOCUMENT-scoped — binding another sprite must drop it"
    );
    // And the new document must composite without reading the old canvas's accumulator.
    let adj2 = t
        .add_adjustment_layer(AdjustmentKind::BrightnessContrast)
        .expect("adjustment added on the new sprite");
    t.set_adjustment_param(adj2, 0, 0.8); // same recycled LayerId as sprite 1's adjustment
    let _ = t.take_preview_arc(); // panicked here before the fix (index past the 256² accumulator)
}

#[test]
fn switching_sprite_does_not_carry_the_old_sprites_selection() {
    // Sweep finding (2026-07-12): the pixel Selection is TOOL-global — it is not in `StashedDoc` (which
    // stashes the LAYER selection) and was never registered in `reset_transient_edit_state`. And
    // `selection_restricts_paint()` asks only "is the mask non-empty?", never "does it belong to THIS
    // sprite?". So the new sprite silently inherited the old one's selection and every stroke outside it
    // was reverted: the "it just doesn't paint and I don't know why" class.
    // RED without the fix: the dab at (48,48) is restored to white by `restore_deselected_region`.
    let mut t = white_canvas(64, 6.0);
    t.set_rect_selection(0, 0, 16, 16); // select a corner of sprite 1
    assert!(t.paint.selection_active);
    t.bind_document(2, vec![255u8; 64 * 64 * 4], 64, 64); // click another sprite (same size = the silent case)
    assert!(
        !t.paint.selection_active,
        "the new sprite starts unselected — the old sprite's selection must not gate its paint"
    );
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down)); // far OUTSIDE sprite 1's selection
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    assert_ne!(
        px(&t, 64, 48, 48),
        [255, 255, 255, 255],
        "the stroke paints — it is not gated by a selection that belongs to another sprite"
    );
}

#[test]
fn per_layer_color_grain_rake_flip_mid_stroke_reshapes_the_maps() {
    // Enio (2026-07-12): "temos outro rake em grain e paper". The **Grain** Rake reaches the SAME route
    // predicate as the Shape Rake — `grain_has_per_dab_rotation()` is one of `per_dab_dynamic`'s disjuncts
    // (`stamp_route.rs`) — so it flips a live Per-Layer Colour stroke from the cached (1 B/px) route to the
    // dynamic (4 B/px) one exactly like Shape Rake did, and panicked the same way. The shape guard is
    // route-agnostic, so it covers this too; this test PINS that, because "it's the same code path" is a
    // claim, and a claim is not a gate.
    let mut t = per_layer_live_stroke();
    t.paint.brush.texture = ph2d_painter_brush::TextureSettings {
        kind: ph2d_painter_brush::TextureKind::Noise, // an active Grain, so Rake is meaningful
        ..t.paint.brush.texture
    };
    t.stamp_dabs(&[live_dab(24.0)]); // batch 1 — cached route (1 B/px maps)
    let before = t.paint.per_layer_stroke.cov[0].len();
    t.paint.brush.texture.rake = true; // GRAIN Rake, stroke still live → the route flips
    t.stamp_dabs(&[live_dab(40.0)]); // batch 2 — panicked here before the fix
    assert_eq!(
        before,
        64 * 64,
        "the cached route's maps start at 1 B/px coverage"
    );
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64 * 4,
        "Grain Rake re-shaped the maps to the dynamic route's 4 B/px premul RGBA"
    );
}

#[test]
fn per_layer_color_route_flip_back_reshapes_the_maps_too() {
    // The REVERSE flip (Rake turned back off: dynamic → cached) never panicked — the 4 B/px maps are big
    // enough to index at 1 B/px — it CORRUPTED in silence: the cached recomposite read the leftover
    // premul-RGBA bytes as coverage. Same root cause (the guard ignored the element size), so the same
    // guard has to catch this direction too, or the fix would only have moved the bug.
    let mut t = per_layer_live_stroke();
    t.paint.brush.shape.rake = true; // start on the dynamic route (4 B/px maps)
    t.stamp_dabs(&[live_dab(24.0)]);
    assert_eq!(t.paint.per_layer_stroke.cov[0].len(), 64 * 64 * 4);
    t.paint.brush.shape.rake = false; // Rake back off, stroke still live → back to the cached route
    t.stamp_dabs(&[live_dab(40.0)]);
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64,
        "the flipped-back cached route re-shaped the maps to 1 B/px coverage"
    );
    let p = px(&t, 64, 40, 32);
    assert!(
        p[1] > 200 && p[0] < 80,
        "the post-flip dab paints the TOP layer's green — not RGBA bytes read as coverage: {p:?}"
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
fn dock_defaults_to_brush_then_toggles() {
    let mut t = PainterTool::default();
    assert!(
        !t.dock_shows_layers(),
        "dock opens on the Brush-properties view (Enio 2026-07-04)"
    );
    t.toggle_dock();
    assert!(
        t.dock_shows_layers(),
        "header toggle flips to the Layers/Effects view"
    );
    t.toggle_dock();
    assert!(!t.dock_shows_layers(), "toggling back returns to Brush");
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
    t.paint.brush.stroke_method = StrokeMethod::Arc;
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
    assert!(
        t.curve_overlay().is_none(),
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
        t.paint.brush.stroke_method = StrokeMethod::Arc;
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
    t.toggle_dock(); // texture-layer editing lives in the Layers view (dock now opens on Brush)

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
    t.toggle_dock(); // texture-layer editing lives in the Layers view (dock now opens on Brush)
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
    let id = t.add_texture_layer().expect("texture layer added"); // active; dock now opens on Brush
    // The dock already shows the Brush view (default) with the texture layer active.
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
    t.set_paint_tool_mode("deform");
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
    for px in src.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 120, 60, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("deform");
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
    for px in src.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 120, 60, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.set_paint_tool_mode("deform");
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
        for px in src.chunks_exact_mut(4) {
            px.copy_from_slice(&[200, 120, 60, 255]);
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.set_paint_tool_mode("deform");
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
    t.set_paint_tool_mode("deform");
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
    // Leaving Deform bakes the transform; re-entering opens the temperament UNSELECTED, so re-picking
    // Transform re-lifts a fresh gizmo (Enio 2026-07-04: the gizmo used to not reappear).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = deform_square_canvas(64, 20, 20, 44, 44);
    t.set_deform_transform_on(true);
    assert!(
        t.deform_gizmo().is_some(),
        "gizmo shown when Transform is picked"
    );
    t.set_paint_tool_mode("brush"); // leave Deform (bakes)
    t.set_paint_tool_mode("deform"); // re-enter
    assert_eq!(
        t.paint.deform.temperament, 0,
        "temperament reopens unselected"
    );
    assert!(
        t.deform_gizmo().is_none(),
        "no gizmo until a mode is picked"
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

#[test]
fn per_layer_texture_color_paints_each_layers_own_rgb() {
    // The capture DEFAULT: a layer without a custom pick paints its OWN captured RGB (Texture Color).
    // With a constant orientation this routes through the cached RGBA path (baked premul stamps) — the
    // per-dab dynamic path only runs for Rake/Random/Jitter/Randomize/canvas-fixed Grain. Two layers:
    // bottom all-RED full mask, top all-GREEN on the LEFT half only → the painted tip is green on the
    // left (top over bottom) and red on the right (bottom alone).
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;
    let s = 16u32;
    let full = vec![255u8; (s * s) as usize];
    let mut left = vec![0u8; (s * s) as usize];
    for y in 0..s {
        for x in 0..s / 2 {
            left[(y * s + x) as usize] = 255;
        }
    }
    t.set_brush_shape_layers(vec![(full, s, s), (left, s, s)]);
    let red = vec![[255u8, 0, 0]; (s * s) as usize]
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>();
    let green = vec![[0u8, 255, 0]; (s * s) as usize]
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>();
    t.paint
        .shape_layers
        .set_layers_meta(vec![red, green], vec![1.0; 2], vec![0; 2], vec![1, 2]);
    t.toggle_brush_shape_per_layer_color();
    assert!(t.paint.shape_layers.is_color_mode());
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let l = px(&t, 64, 27, 32); // left of centre — the GREEN top layer covers here
    let r = px(&t, 64, 37, 32); // right of centre — only the RED bottom layer covers
    assert!(
        l[1] > 180 && l[0] < 80,
        "left half paints the TOP layer's green rgb: {l:?}"
    );
    assert!(
        r[0] > 180 && r[1] < 80,
        "right half paints the BOTTOM layer's red rgb: {r:?}"
    );
}

mod per_layer_perf {
    use super::*;
    use ph2d_painter_brush::{Falloff, StrokeMethod};
    use std::time::Instant;

    /// White `size`x`size` canvas, `n_shape` full 16x16 Shape layers (distinct colours), Per-Layer
    /// Color ON, Curve method, hard disk `radius`. `doc_extra` extra raster doc layers make the
    /// document stack non-trivial (exercises `take_preview_arc`'s `composite_region` lane).
    fn setup(size: u32, n_shape: usize, doc_extra: usize, radius: f32) -> PainterTool {
        let mut t = white_canvas(size, radius);
        t.paint.brush.stroke_method = StrokeMethod::Arc;
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

    /// The LIVE config (Enio 2026-07-04: "FPS 60→10 com Line/Arc/Ellipse/Polygon/Freehand"): captured
    /// layers WITHOUT a custom colour pick default to **Texture Color** → the route takes the per-pixel
    /// DYNAMIC path (`stamp_dabs_per_layer_dynamic`), not the cached one the batch kernel accelerated.
    /// Times the EDIT phase (drag the Arc's mid control point — a whole-shape re-stamp per move) at
    /// 2048², dynamic (texture-colour default) vs cached (all colours picked), N 3/16:
    ///   cargo test -p ph2d-tool-painter --release per_layer_perf_live -- --ignored --nocapture
    #[test]
    #[ignore = "perf measurement — run explicitly in --release"]
    fn per_layer_perf_live() {
        let size = 2048u32;
        let radius = 100.0f32;
        for &n in &[3usize, 16] {
            for &custom_colors in &[false, true] {
                let mut t = white_canvas(size, radius);
                t.paint.brush.stroke_method = StrokeMethod::Arc;
                t.paint.brush.hardness = 1.0;
                t.paint.brush.falloff = Falloff::Constant;
                t.paint.brush.space_attenuation = false;
                // 128² soft-disc layer masks (a real captured-layer silhouette, not a flat square).
                let layers: Vec<(Vec<u8>, u32, u32)> = (0..n)
                    .map(|_| {
                        let s = 128u32;
                        let mut m = vec![0u8; (s * s) as usize];
                        for y in 0..s {
                            for x in 0..s {
                                let dx = x as f32 - 64.0;
                                let dy = y as f32 - 64.0;
                                let d = (dx * dx + dy * dy).sqrt() / 64.0;
                                m[(y * s + x) as usize] = ((1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
                            }
                        }
                        (m, s, s)
                    })
                    .collect();
                t.set_brush_shape_layers(layers);
                // Real captured layers carry per-pixel RGB (`w·h·3`) — WITHOUT it `any_texture_color()`
                // is false and the route silently falls back to the cached path, hiding the dynamic
                // kernel (the live default) from the measurement.
                let rgb: Vec<Vec<u8>> = (0..n)
                    .map(|i| {
                        let s = 128usize;
                        let mut v = vec![0u8; s * s * 3];
                        for p in 0..s * s {
                            v[p * 3] = ((p * 7 + i * 31) % 256) as u8;
                            v[p * 3 + 1] = ((p * 13 + i * 17) % 256) as u8;
                            v[p * 3 + 2] = ((p * 3 + i * 53) % 256) as u8;
                        }
                        v
                    })
                    .collect();
                t.paint.shape_layers.set_layers_meta(
                    rgb,
                    vec![1.0; n],
                    vec![0; n],
                    (0..n as u64).collect(),
                );
                t.toggle_brush_shape_per_layer_color();
                if custom_colors {
                    for i in 0..n {
                        let f = i as f32;
                        t.set_brush_shape_layer_color(
                            i,
                            [(f * 0.13) % 1.0, (f * 0.27) % 1.0, (f * 0.41) % 1.0],
                        );
                    }
                }
                assert!(t.paint.shape_layers.is_color_mode());
                t.add_raster_layer("doc"); // non-trivial doc stack → real preview lane
                // Create the Arc: a long horizontal drag (chord 1800 px → mid bows to y≈754).
                t.on_canvas_pointer(cp([124.0, 1024.0], PointerPhase::Down));
                t.on_canvas_pointer(cp([1924.0, 1024.0], PointerPhase::Move));
                t.on_canvas_pointer(cp([1924.0, 1024.0], PointerPhase::Up));
                let _ = t.take_preview_arc();
                // EDIT phase: grab the (bowed) mid anchor and wiggle it — one whole-shape re-stamp per move.
                let mid = t.curve_overlay().expect("arc open").points[1];
                t.on_canvas_pointer(cp(mid, PointerPhase::Down));
                let moves = 10u32;
                let mut held = None;
                let mut move_ns = 0u128;
                let mut prev_ns = 0u128;
                for k in 0..moves {
                    let d = ((k % 5) as f32) - 2.0;
                    let a = std::time::Instant::now();
                    let _ = t.on_canvas_pointer(cp([mid[0] + d, mid[1] + d], PointerPhase::Move));
                    move_ns += a.elapsed().as_nanos();
                    let b = std::time::Instant::now();
                    if let Some(p) = t.take_preview_arc() {
                        held = Some(p); // bridge retainer
                    }
                    prev_ns += b.elapsed().as_nanos();
                }
                let _ = held;
                t.on_canvas_pointer(cp(mid, PointerPhase::Up));
                let kf = f64::from(moves);
                eprintln!(
                    "  live 2048² r100 N{n:<2} {}  move {:>9.1} us   prev {:>9.1} us",
                    if custom_colors {
                        "cached (colours picked)"
                    } else {
                        "texture-colour (default)"
                    },
                    move_ns as f64 / kf / 1000.0,
                    prev_ns as f64 / kf / 1000.0,
                );
            }
        }
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

    /// REGRESSION (Enio 2026-07-11 smoke, clean `--release`): editing a per-layer-colour shape left "lines
    /// at the edges of rectangles, in the brush's own colours" — and `PH2D_PAINT_FULL_UPLOAD=1` did NOT
    /// clear them, so it is NOT the partial GPU upload; it is upstream (the canvas restore/recomposite, or
    /// the `composite_region`/`blit_region` cache lane). The trigger is a shape that MOVES so the new
    /// footprint does NOT cover the old (the screenshot's off-centre ghost) — the [[watercolor Drag-Dot
    /// "moving preview restores the old position"]] class, unguarded for the per-layer-colour shape path.
    /// A line whose endpoint SWEEPS across the canvas must leave the SAME preview as the final line drawn
    /// directly; any pixel that differs is residue the sweep failed to revert (the stale rectangle). The
    /// diff bbox tells whether it is an axis-aligned rectangle (the Bug #9 signature). Cached path (colours
    /// picked) AND the per-dab dynamic path (Randomize Colour — Enio's 3D-look brushes) are both checked.
    #[test]
    fn per_layer_moving_shape_leaves_no_stale_rectangle() {
        let size = 256u32;
        let a = [40.0f32, 40.0]; // fixed line start
        // Endpoint sweeps a wide arc → successive lines barely overlap (the moving-preview case).
        let sweep = [
            [220.0f32, 60.0],
            [210.0, 200.0],
            [70.0, 215.0],
            [200.0, 130.0],
        ];
        let run = |doc: usize, randomize: bool| -> (usize, (u32, u32, u32, u32)) {
            let mk = || {
                let mut t = setup(size, 3, doc, 12.0);
                t.paint.brush.stroke_method = StrokeMethod::Line;
                if randomize {
                    // Route through the per-dab DYNAMIC path (`stamp_dabs_per_layer_dynamic`): a Hue jitter
                    // makes `has_colour_jitter_amount()` true — the gate the 3D-look brushes trip.
                    t.paint.brush.color_jitter_hue = 0.5;
                }
                t
            };
            let last = *sweep.last().unwrap();
            // TRUTH: the final line, drawn directly.
            let mut truth = mk();
            truth.on_canvas_pointer(cp(a, PointerPhase::Down));
            truth.on_canvas_pointer(cp(last, PointerPhase::Move));
            let (tb, w, h) = truth.take_preview_arc().expect("truth preview");
            // ACTUAL: the endpoint sweeps through every point, ending at the SAME final line.
            let mut actual = mk();
            actual.on_canvas_pointer(cp(a, PointerPhase::Down));
            let mut ab_opt = None;
            for p in sweep {
                actual.on_canvas_pointer(cp(p, PointerPhase::Move));
                if let Some(v) = actual.take_preview_arc() {
                    ab_opt = Some(v); // drain each frame like the bridge; keep the last non-empty
                }
            }
            let (ab, _, _) = ab_opt.expect("actual preview");
            let mut n = 0usize;
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    if tb[i..i + 4] != ab[i..i + 4] {
                        n += 1;
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            (n, (x0, y0, x1, y1))
        };
        let (n0, b0) = run(0, false);
        let (n1, b1) = run(1, false);
        let (nd, bd) = run(0, true);
        assert_eq!(
            n0, 0,
            "cached, trivial stack: {n0} residue px on the CANVAS, bbox={b0:?}"
        );
        assert_eq!(
            n1, 0,
            "cached, doc stack: {n1} residue px in the COMPOSITE cache lane, bbox={b1:?}"
        );
        assert_eq!(
            nd, 0,
            "DYNAMIC (Randomize Colour): {nd} residue px, bbox={bd:?}"
        );
    }

    /// The REAL context (Enio's screenshot): a PARKED shape (drawn earlier) plus an ACTIVE shape being
    /// EDITED — the `restamp_shapes_preview` multi-shape path (active + every parked re-stamped onto one
    /// baseline each frame), NOT the single `stamp_drag_preview` a plain stroke uses. As the active shape's
    /// handle sweeps, the union footprint shifts; any pixel that ends up different from the same final
    /// two-shape scene built directly is residue (the off-centre ghost). Trivial + doc stack.
    #[test]
    fn per_layer_multishape_edit_leaves_no_stale_rectangle() {
        let size = 256u32;
        // Shape 1 (parked): a small ellipse in the top-left. Shape 2 (active): drawn bottom-right, then its
        // right handle is dragged around. The sweep + the final resting handle position.
        let c1 = [70.0f32, 70.0];
        let c2 = [165.0f32, 165.0];
        let sweep = [
            [235.0f32, 165.0],
            [165.0, 235.0],
            [120.0, 120.0],
            [210.0, 150.0],
        ];
        let run = |doc: usize| -> (usize, (u32, u32, u32, u32)) {
            // Returns the LIVE preview captured mid-drag (no pen-up — the artifact is a live-preview residue
            // the final commit would otherwise hide).
            let build = |edits: &[[f32; 2]]| -> (Arc<Vec<u8>>, u32, u32) {
                let mut t = setup(size, 3, doc, 12.0);
                t.paint.brush.stroke_method = StrokeMethod::Ellipse;
                // Shape 1 → parked once shape 2 begins.
                t.on_canvas_pointer(cp(c1, PointerPhase::Down));
                t.on_canvas_pointer(cp([c1[0] + 25.0, c1[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp([c1[0] + 25.0, c1[1]], PointerPhase::Up));
                // Shape 2 (empty Down parks shape 1) → radius 40, then editable.
                t.on_canvas_pointer(cp(c2, PointerPhase::Down));
                t.on_canvas_pointer(cp([c2[0] + 40.0, c2[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp([c2[0] + 40.0, c2[1]], PointerPhase::Up));
                // Edit shape 2: grab the right handle (at centre + rx) and drag it through `edits` — NO Up.
                let h = [c2[0] + 40.0, c2[1]];
                t.on_canvas_pointer(cp(h, PointerPhase::Down));
                let mut prev = None;
                for &e in edits {
                    t.on_canvas_pointer(cp(e, PointerPhase::Move));
                    if let Some(v) = t.take_preview_arc() {
                        prev = Some(v);
                    }
                }
                prev.or_else(|| {
                    t.preview_dirty = true;
                    t.take_preview_arc()
                })
                .expect("a live preview mid-edit")
            };
            let last = *sweep.last().unwrap();
            let (tb, w, h) = build(&[last]); // edit straight to the final handle position
            let (ab, _, _) = build(&sweep); // sweep through every intermediate position
            let mut n = 0usize;
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    if tb[i..i + 4] != ab[i..i + 4] {
                        n += 1;
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            (n, (x0, y0, x1, y1))
        };
        let (n0, b0) = run(0);
        let (n1, b1) = run(1);
        assert_eq!(
            n0, 0,
            "multi-shape, trivial stack: {n0} residue px, bbox={b0:?}"
        );
        assert_eq!(
            n1, 0,
            "multi-shape, doc stack: {n1} residue px, bbox={b1:?}"
        );
    }

    /// The DECISIVE oracle: gesture-vs-gesture cancels a bug that is geometry-dependent (both go through
    /// the partial `composite_region`/`blit_region` cache lane). Compare the incrementally-blitted
    /// `composited` CACHE against a FULL recompose of the SAME final state — that is the exact difference
    /// `PH2D_PAINT_FULL_UPLOAD` cannot fix (it uploads the stale cache). Edits sweep a shape toward the
    /// canvas EDGE so the dirty bbox can clamp (`composite_region` returns `rw < bbox.w` while `blit_region`
    /// strides by `bbox.w` — the §3-B shear). Non-trivial doc stack (the composite lane only runs there).
    #[test]
    fn per_layer_composite_cache_matches_full_recompose_during_shape_edit() {
        let size = 128u32;
        let mut t = setup(size, 3, 1, 10.0);
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        // Draw an ellipse near the right edge, then drag its handle across the boundary and back.
        let c = [96.0f32, 64.0];
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        t.on_canvas_pointer(cp([c[0] + 24.0, c[1]], PointerPhase::Move));
        t.on_canvas_pointer(cp([c[0] + 24.0, c[1]], PointerPhase::Up));
        let handle = [c[0] + 24.0, c[1]];
        t.on_canvas_pointer(cp(handle, PointerPhase::Down));
        let mut partial = None;
        for &e in &[[124.0f32, 64.0], [110.0, 30.0], [70.0, 64.0], [118.0, 90.0]] {
            t.on_canvas_pointer(cp(e, PointerPhase::Move));
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial.expect("a partial-lane composited preview mid-edit");
        // FULL recompose of the SAME final state (drop the incremental cache).
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "partial composite cache != full recompose: {n} stale px, bbox=({x0},{y0})..({x1},{y1}) \
             — the incremental blit lane left a stale rectangle (FULL_UPLOAD can't fix this)"
        );
    }

    /// The screenshot's actual shape: a self-overlapping FREE HAND stroke (the un-incrementalised
    /// whole-path re-fill) with per-layer colour + Randomize Colour (the dynamic recomposite path) +
    /// a doc stack. Mid-draw, the partial composite CACHE must equal a FULL recompose of the same state.
    #[test]
    fn per_layer_freehand_selfoverlap_cache_matches_full_recompose() {
        let size = 160u32;
        let mut t = setup(size, 3, 1, 8.0);
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.color_jitter_hue = 0.5; // → the dynamic per-dab recomposite path
        // A figure-8 that crosses itself (the pretzel), captured point by point.
        let path = [
            [40.0f32, 80.0],
            [70.0, 40.0],
            [110.0, 40.0],
            [120.0, 80.0],
            [90.0, 120.0],
            [60.0, 120.0],
            [40.0, 80.0],
            [70.0, 60.0],
            [110.0, 100.0],
        ];
        t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
        let mut partial = None;
        for &p in &path[1..] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial
            .or_else(|| {
                t.preview_dirty = true;
                t.take_preview_arc()
            })
            .expect("a partial-lane preview mid free-hand");
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "free-hand: {n} stale px, bbox=({x0},{y0})..({x1},{y1})"
        );
    }

    /// The MISSING context (Enio 2026-07-11: "esqueci de colocar na freehand"): FreeHand is the ONLY shape
    /// method that does NOT coalesce (`coalesces_canvas_motion == false`) — the real app processes SEVERAL
    /// pointer Moves per frame, then ONE `take_preview_arc`. Every prior harness drained once per Move, so
    /// the multi-Move-per-frame accumulation of the growing whole-path re-fill was never exercised. Compare
    /// the partial composite CACHE built that way against a FULL recompose of the same final state.
    #[test]
    fn per_layer_freehand_multimove_per_frame_matches_full_recompose() {
        let size = 200u32;
        let mut t = setup(size, 3, 1, 6.0);
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.color_jitter_hue = 0.5; // dynamic per-dab path (Enio's 3D brushes)
        // A growing, self-overlapping scribble captured point by point (spaced > min-capture).
        let pts: Vec<[f32; 2]> = (0..40)
            .map(|i| {
                let f = i as f32;
                // A lissajous-ish curve that crosses itself, no RNG, deterministic.
                let x = 100.0 + 70.0 * ((f * 0.5).sin());
                let y = 100.0 + 60.0 * ((f * 0.31).sin());
                [x, y]
            })
            .collect();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        let mut partial = None;
        // 4 Moves per "frame", ONE drain per frame (the un-coalesced FreeHand cadence).
        for frame in pts[1..].chunks(4) {
            for &p in frame {
                t.on_canvas_pointer(cp(p, PointerPhase::Move));
            }
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial
            .or_else(|| {
                t.preview_dirty = true;
                t.take_preview_arc()
            })
            .expect("a partial-lane preview");
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "free-hand multi-move/frame: {n} stale px, bbox=({x0},{y0})..({x1},{y1}) \
             — the partial cache diverged from a full recompose"
        );
    }

    /// Enio's ACTUAL screenshot scene: a FREE HAND scribble already drawn (→ PARKED) with an ELLIPSE editor
    /// active ON TOP of it, being edited. `restamp_shapes_preview` then re-stamps the parked free-hand's
    /// LONG dab list + the active ellipse onto one baseline EVERY move — the heaviest multi-shape path, and
    /// the one the screenshot shows. Partial composite cache vs FULL recompose of the same final state.
    #[test]
    fn per_layer_parked_freehand_plus_active_ellipse_matches_full_recompose() {
        let size = 220u32;
        let mut t = setup(size, 3, 1, 7.0);
        // 1) Draw a self-overlapping FREE HAND scribble, pen-up → it parks when the next shape starts.
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.color_jitter_hue = 0.5;
        let pts: Vec<[f32; 2]> = (0..32)
            .map(|i| {
                let f = i as f32;
                [
                    70.0 + 55.0 * (f * 0.5).sin(),
                    80.0 + 45.0 * (f * 0.31).sin(),
                ]
            })
            .collect();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for frame in pts[1..].chunks(4) {
            for &p in frame {
                t.on_canvas_pointer(cp(p, PointerPhase::Move));
            }
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        let _ = t.take_preview_arc();
        // 2) An ELLIPSE on top (empty Down parks the free-hand), then EDIT its handle across a sweep.
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        let c = [150.0f32, 150.0];
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        t.on_canvas_pointer(cp([c[0] + 45.0, c[1]], PointerPhase::Move));
        t.on_canvas_pointer(cp([c[0] + 45.0, c[1]], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let handle = [c[0] + 45.0, c[1]];
        t.on_canvas_pointer(cp(handle, PointerPhase::Down));
        let mut partial = None;
        for &e in &[
            [205.0f32, 150.0],
            [150.0, 205.0],
            [95.0, 120.0],
            [190.0, 175.0],
        ] {
            t.on_canvas_pointer(cp(e, PointerPhase::Move));
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial
            .or_else(|| {
                t.preview_dirty = true;
                t.take_preview_arc()
            })
            .expect("a partial-lane preview mid-edit");
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "parked free-hand + active ellipse: {n} stale px, bbox=({x0},{y0})..({x1},{y1})"
        );
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
        (StrokeMethod::Arc, true),
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
    // Selection mode: gizmo drags / Rectangle / Ellipse / Automatic act on the latest position only →
    // coalesce (each raw Move paid a full boolean recompose — the P4 storm, Enio 2026-07-04). The
    // Freehand lasso (mode 1) CAPTURES the path → every event, regardless of the brush method.
    t.paint.brush.stroke_method = StrokeMethod::Space; // would NOT coalesce as a stroke
    t.set_paint_tool_mode("selection");
    for (mode, want) in [(0u8, true), (1, false), (2, true), (3, true)] {
        t.set_selection_mode(mode);
        assert_eq!(
            t.coalesces_canvas_motion(),
            want,
            "selection mode {mode} coalesce bucket"
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

/// The paint COLOUR is one shared foreground colour across every paint mode (Photoshop/Procreate model):
/// a colour set in one mode survives a mode switch, so the C&F ColorDrop (which switches to Fill mode) and
/// switching tools no longer revert it to the previous / default black (Enio 2026-07-04).
#[test]
fn paint_colour_is_shared_across_modes() {
    let mut t = white_canvas(32, 4.0);
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([200, 50, 20]);
    assert_eq!(t.brush_color_srgb8(), [200, 50, 20]);
    // Switching to Fill mode (what the ColorDrop does) must keep the colour, not swap in Fill's black slot.
    t.set_paint_tool_mode("fill");
    assert_eq!(
        t.brush_color_srgb8(),
        [200, 50, 20],
        "the colour survives the switch to Fill mode (ColorDrop)"
    );
    // And through Selection + back to Brush.
    t.set_paint_tool_mode("selection");
    assert_eq!(t.brush_color_srgb8(), [200, 50, 20]);
    t.set_paint_tool_mode("brush");
    assert_eq!(
        t.brush_color_srgb8(),
        [200, 50, 20],
        "colour is shared, not per-mode"
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
    t.set_paint_tool_mode("deform");
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
    assert!(
        t.paint.deform.active,
        "the deform session survives the undo"
    );
    assert!(
        !t.paint.deform.disp.is_empty(),
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

/// Watercolor render-path #1 — the **edge** term pools pigment at the receding boundary: a rim band
/// (just inside the wash) reads DARKER than the deep interior when Edge is on, and NOT darker when Edge
/// is off. Granulation + Warp are zeroed to isolate the edge term (the paper-noise + boundary-warp
/// fields would otherwise perturb the sampled pixels). Drives the real optical composite end-to-end
/// through `paint_end` (the "efeito perceptual" DIRETIVA §4 asserts). See `super::watercolor_render`.
#[test]
fn watercolor_edge_darkens_the_rim_not_the_interior() {
    fn wet_brush(radius: f32, edge_gain: f32) -> BrushSpec {
        BrushSpec {
            radius_px: radius,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.24, 0.39, 0.63], // mid blue → darkening is measurable in every channel
            space_attenuation: false,
            watercolor: true,
            edge_gain,
            edge_spread: 7.0,
            granulation: 0.0, // isolate the edge term from the paper granulation
            warp: 0.0,        // and from the organic-boundary displacement
            ..Default::default()
        }
    }
    fn paint_dab(brush: BrushSpec, size: u32, center: [f32; 2]) -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = brush;
        for slot in &mut t.paint.brush_by_mode {
            *slot = brush;
        }
        assert!(t.on_canvas_pointer(cp(center, PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp(center, PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let center = [48.0f32, 48.0];
    let lum = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    // The rim band sits a few px inside the 20 px disc boundary (where cover is high but blur(cover) has
    // fallen — the edge term peaks); the interior is the disc centre (cover ≈ 1, blur ≈ 1 → edge ≈ 0).
    let rim_y = 32; // 16 px above centre

    // Edge ON: the rim band is darker than the interior (pigment pooled at the receding front).
    let t = paint_dab(wet_brush(20.0, 3.0), size, center);
    let interior = px(&t, size, 48, 48);
    let rim = px(&t, size, 48, rim_y);
    assert!(
        lum(rim) < lum(interior),
        "edge darkening must pool pigment at the rim: rim {rim:?} not darker than interior {interior:?}"
    );

    // Edge OFF (gain 0): no rim pooling — the boundary only has LESS coverage, so it is never darker than
    // the interior (density there is `cover·fill`, and `cover ≤ 1`).
    let t0 = paint_dab(wet_brush(20.0, 0.0), size, center);
    assert!(
        lum(px(&t0, size, 48, rim_y)) >= lum(px(&t0, size, 48, 48)),
        "with Edge off there is no rim pooling (the boundary is lighter, never darker, than the interior)"
    );
}

/// Watercolor render-path #2 — **granulation** textures the wash: the paper-tooth field modulates the
/// optical density (`gran = 1 + (paperHeight − 0.5)·2·granAmt`), so turning Granulation up raises the
/// spatial VARIANCE of the interior (mottled tooth) versus a flat wash at Granulation 0. Symmetric
/// around the mean (wet_edges), so it redistributes pigment, not a net wipe. Real optical composite,
/// Edge off. DIRETIVA §4.
#[test]
fn watercolor_granulation_textures_the_wash() {
    fn interior_variance(granulation: f32) -> f64 {
        let size = 64u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 26.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.0, 0.0, 0.0], // black on white → deposit reads as darkening
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate granulation from the edge term
            warp: 0.0,      // sample the true tooth, un-displaced
            fill: 0.6,      // a solid wash so the tooth variation is well above quantisation
            granulation,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up)));
        // Variance of the R channel over a deep-interior window (well inside the 26 px disc → cover ≈ 1,
        // so only the tooth varies the value).
        let vals: Vec<f64> = (24..40)
            .flat_map(|y| (24..40).map(move |x| (x, y)))
            .map(|(x, y)| f64::from(t.canvas_rgba[((y * size + x) * 4) as usize]))
            .collect();
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64
    }
    let flat = interior_variance(0.0);
    let granulated = interior_variance(0.8);
    assert!(
        flat < 1.0,
        "Granulation 0 is a flat wash (near-zero interior variance): got {flat}"
    );
    assert!(
        granulated > flat + 4.0,
        "Granulation must texture the wash (raise interior variance): granulated {granulated} vs flat {flat}"
    );
}

/// Watercolor render-path #3 — **Pigment** mixes wet-on-wet subtractively: painting yellow over an
/// opaque blue base with Pigment on lifts GREEN in the overlap (RYB: blue + yellow → green), where the
/// plain optical composite (Pigment off) stays a muddy Beer–Lambert blend. A dense wash (high Fill/Depth)
/// so the pigment film is opaque enough to mix. Real composite. DIRETIVA §4.
#[test]
fn watercolor_pigment_mixes_wet_on_wet_toward_green() {
    fn center_pixel(pigment_mix: f32) -> [u8; 4] {
        let size = 48u32;
        // A solid opaque blue base already on the canvas (the "previous wash" to mix into).
        let mut src = vec![0u8; (size * size * 4) as usize];
        for p in src.chunks_exact_mut(4) {
            p.copy_from_slice(&[30, 55, 195, 255]);
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 14.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.90, 0.80, 0.10], // yellow
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate pigment from the edge term
            granulation: 0.0,
            warp: 0.0,
            fill: 0.85, // dense wash → an opaque pigment film that mixes strongly
            depth: 2.0,
            pigment: pigment_mix > 0.0,
            pigment_mix,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up)));
        px(&t, size, 24, 24)
    }
    let off = center_pixel(0.0); // Pigment off → plain Beer–Lambert blend (a muddy YELLOW-green: high red)
    let on = center_pixel(1.0); // Pigment on → subtractive RYB → a true GREEN (green pulls ahead of red)
    // "Toward green" = green dominates red more strongly with Pigment on. (The green *channel* alone is
    // higher in the yellow-green off-state — yellow carries green too — so the signature is green−red.)
    let green_lead = |p: [u8; 4]| i32::from(p[1]) - i32::from(p[0]);
    assert!(
        green_lead(on) > green_lead(off),
        "wet-on-wet pigment must swing toward green (green leading red) vs the plain blend: on {on:?} vs off {off:?}"
    );
}

/// Watercolor render-path #4 — **Opacity (pigment body)** lets a LIGHT-valued pigment deposit at its hue
/// (doc 13 #17: "azul e amarelo quase não aparecem"). Pure Beer–Lambert (`opacity = 0`) can only subtract
/// light, so a light yellow over white paper leaves its bright channels at `Tᵢ ≈ 1` and barely darkens —
/// the reported bug. Turning Opacity up lays the pigment's own colour (scattering / hiding power), so the
/// SAME wash darkens substantially MORE and keeps its yellow character (blue absorbed hardest). The
/// `opacity = 0` render is byte-identical to the old path by construction: `body_cov = 0` ⇒ the fold term
/// `(s2l[pig] − optical)·0.0` is exactly `0.0` and `max(1−t_min, 0)` is unchanged. Real composite,
/// Edge/Warp/Granulation off to isolate the body term. DIRETIVA §4 (verified RED by neutering the fold).
#[test]
fn watercolor_opacity_gives_light_pigments_body() {
    fn light_yellow_center(opacity: f32) -> [u8; 4] {
        let size = 64u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 20.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.95, 0.85, 0.20], // light yellow → bright R,G (near-transparent under Beer–Lambert)
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the body term from the rim
            granulation: 0.0,
            warp: 0.0,
            fill: 0.15, // a thin default-ish wash — where the light-pigment invisibility bites hardest
            depth: 1.2,
            opacity,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up)));
        px(&t, size, 32, 32)
    }
    // Total darkening away from white paper — the "how much did the wash actually deposit" meter.
    let deposit =
        |p: [u8; 4]| (255 - u32::from(p[0])) + (255 - u32::from(p[1])) + (255 - u32::from(p[2]));
    let transparent = light_yellow_center(0.0); // pure Beer–Lambert → the faint, near-invisible wash
    let bodied = light_yellow_center(0.8); //       body on → the pigment shows at its hue
    assert!(
        deposit(bodied) > deposit(transparent) + 30,
        "Opacity must give a light pigment body (deposit far more): bodied {bodied:?} (Δ{}) vs transparent {transparent:?} (Δ{})",
        deposit(bodied),
        deposit(transparent),
    );
    // The character stays YELLOW: blue is the hardest-absorbed channel in both (body lays the pigment's
    // OWN colour, it does not gray the wash toward paper).
    assert!(
        bodied[2] < bodied[0] && bodied[2] < bodied[1],
        "body must preserve the pigment hue (blue absorbed hardest): {bodied:?}"
    );
}

/// Watercolor render-path is **LIVE** — the wash appears *during* the stroke (each frame recomposited
/// from the growing coverage over the frozen base), not as a jump on release. Paint a horizontal band
/// and, WITHOUT releasing, assert (a) the interior already differs from the white base and (b) the rim
/// is already darker than the centreline. (Fix for the "não pinta em tempo real / escurece no final"
/// feedback; the pen-up bake is covered by `watercolor_edge_darkens_the_rim_not_the_interior`.)
#[test]
fn watercolor_wash_is_live_before_pen_up() {
    let size = 96u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.25, 0.40, 0.62],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 2.0,
        edge_spread: 5.0,
        granulation: 0.0,
        warp: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    let lum = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    // Paint a horizontal band and STOP without releasing (no Up event).
    assert!(t.on_canvas_pointer(cp([24.0, 48.0], PointerPhase::Down)));
    for x in [32.0, 40.0, 48.0, 56.0, 64.0] {
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    // Pointer still down: the wash is ALREADY on the canvas (interior differs from the white base) and
    // its rim is ALREADY darker than the centreline.
    let interior = px(&t, size, 44, 48);
    assert!(
        lum(interior) < 3 * 255,
        "the wash is live mid-stroke (interior no longer white)"
    );
    let rim = px(&t, size, 44, 40); // 8 px above centre → top rim of the radius-10 band
    assert!(
        lum(rim) < lum(interior),
        "edge must be LIVE mid-stroke: rim {rim:?} not darker than interior {interior:?}"
    );
}

/// Watercolor is **inert when off**: a `watercolor = false` stroke is byte-identical to a plain brush
/// (the render-path skips deposit AND composite — the skip-deposit gate must not leak into a normal
/// stroke). Paints the same dab with the flag off and confirms real pigment landed on the canvas.
#[test]
fn watercolor_off_is_a_plain_deposit() {
    let size = 48u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.10, 0.20, 0.80],
        space_attenuation: false,
        watercolor: false, // OFF → the plain deposit path, no optical composite
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down)));
    assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up)));
    // The dab deposited the (opaque) brush colour straight — a plain Mix over white.
    let c = px(&t, size, 24, 24);
    assert_eq!(c[3], 255, "opaque deposit");
    assert!(
        c[2] > c[0] && c[2] > c[1],
        "the plain blue brush colour landed (blue dominant): {c:?}"
    );
}

/// Watercolor **TRUE SMEAR** — Smudge > 0 physically DRAGS the already-painted paint along the stroke
/// ("Smearing", not a colour tint): crossing a red band with Pickup 0 (pure brush-colour wash, so the
/// smear is isolated) must (a) drag red PAST the band's far edge, and (b) drag white INTO the band's
/// entry edge — displacement of the base, which the reservoir-only model never did ("levanta mas não
/// borra a já pintada", Enio 2026-07-06).
#[test]
fn watercolor_smudge_true_smears_the_painted_paint() {
    fn run(smudge: f32) -> PainterTool {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (40..70).contains(&x) {
                    [217u8, 13, 13, 255] // red band mid-canvas
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        // NB: the engine's default (soft) falloff — a Constant falloff at full strength degenerates the
        // smear into a rigid translation (the disc's initial content overwrites everything it crosses).
        t.paint.brush = BrushSpec {
            radius_px: 6.0,
            color: [0.1, 0.2, 0.85],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.3, // a light wash so the (smeared) base reads through
            depth: 1.0,
            wet_smudge: smudge,
            wet_rewet: 0.0, // isolate the physical smear from the wet-on-wet rewet
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        // Left-to-right stroke crossing the band and exiting into white.
        assert!(t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 96.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up)));
        t
    }
    let size = 128u32;
    let plain = run(0.0);
    let smeared = run(0.9);
    // (a) Past the band's far edge: the smear dragged red out of the band → markedly redder (lower G/B
    // vs R) than the plain wash over white.
    let (ex, ey) = (74u32, 64u32);
    let p = px(&plain, size, ex, ey);
    let s = px(&smeared, size, ex, ey);
    let redness = |c: [u8; 4]| i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2;
    assert!(
        redness(s) > redness(p) + 40,
        "smear must drag red past the band edge: smeared {s:?} vs plain {p:?}"
    );
    // (b) At the band's entry edge: the smear dragged white INTO the band → lighter than the plain
    // wash over pristine red.
    let (bx, by) = (43u32, 64u32);
    let p = px(&plain, size, bx, by);
    let s = px(&smeared, size, bx, by);
    let lum = |c: [u8; 4]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
    assert!(
        lum(s) > lum(p) + 60,
        "smear must drag white into the band's entry edge: smeared {s:?} vs plain {p:?}"
    );
}

/// **Watercolor Smudge wraps across the Tiling seam (doc 13 #2, follow-up a — Enio 2026-07-11).** With
/// Tiling on, the coverage/color wash already wraps (`tiled_dabs`); the TRUE SMEAR must wrap too, or the
/// far edge's wash composites over an UN-smeared base — a visible smudge seam. A rightward smear crossing
/// the RIGHT edge lifts the right-edge paint toroidally and stamps it onto the wrapped LEFT edge, so a red
/// right-edge band gets dragged onto the left edge (unreachable without the wrap). RED before the fix:
/// under Tiling the left edge is identical at smudge 0 vs 0.9 (the smear never touched the far edge).
#[test]
fn watercolor_smudge_wraps_across_the_tiling_seam() {
    let size = 64u32;
    fn run(smudge: f32, tiling: bool) -> PainterTool {
        let size = 64u32;
        // White canvas with a RED right THIRD (x∈[42,63]) — plenty of paint for the wrapped smear to drag.
        let mut src = vec![255u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 42..size {
                let i = ((y * size + x) * 4) as usize;
                src[i..i + 4].copy_from_slice(&[230u8, 15, 15, 255]);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 6.0,
            color: [0.1, 0.2, 0.85], // blue wash, so any RED on the far edge can only be dragged base
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.12, // very light wash so the (smeared) base reads through clearly
            depth: 1.0,
            wet_smudge: smudge,
            wet_rewet: 0.0, // isolate the physical smear from the wet-on-wet rewet
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        t.paint.tiling = [tiling, false];
        // Rightward stroke near the right edge, crossing the seam (dabs at x≈52..72, radius 6 ⇒ the copies
        // wrap onto the left edge); a dense step (2 px) so the drag accumulates, y=32.
        assert!(t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Down)));
        let mut x = 50.0f32;
        while x < 74.0 {
            x += 2.0;
            t.on_canvas_pointer(cp([x, 32.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 32.0], PointerPhase::Up)));
        t
    }
    let redness = |c: [u8; 4]| i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2;
    let (lx, ly) = (2u32, 32u32); // a wrapped left-edge pixel
    let base = run(0.0, true); // Tiling on, no smudge: wash-only far edge (wash wraps in both runs)
    let smeared = run(0.9, true); // Tiling on + smudge: the wrapped smear drags red onto the far edge
    let off = run(0.9, false); // Tiling OFF: the smear can't reach the far edge (proves it's the wrap)
    let b = px(&base, size, lx, ly);
    let s = px(&smeared, size, lx, ly);
    let o = px(&off, size, lx, ly);
    assert!(
        redness(s) > redness(b) + 20,
        "the wrapped smear dragged red onto the far edge: smeared {s:?} vs wash-only {b:?}"
    );
    assert!(
        redness(s) > redness(o) + 20,
        "the far-edge red is the Tiling WRAP, not a non-tiled path: tiled {s:?} vs off {o:?}"
    );
}

/// Watercolor **dirty-rect** — the live recomposite is LOCAL to the frame's new dabs (wet_edges
/// `renderFrame`), so the per-frame cost tracks the brush, not the grown stroke (the old cumulative-bbox
/// recomposite was ~quadratic along a stroke — the "Performance muito aquém do MVP" symptom). Proof by
/// sentinel: a pixel poked into the ALREADY-painted area, far behind the stroke frontier, must survive
/// the live passes untouched (a full-bbox recomposite would overwrite it) — and then be recomposited by
/// the pen-up bake (wet_edges `endStroke`), which makes ONE cumulative pass from the incremental bbox.
#[test]
fn watercolor_live_recomposite_is_local_to_the_frame() {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 6.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.70],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 2.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.5,
        depth: 2.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Paint the left third of a horizontal band, live.
    assert!(t.on_canvas_pointer(cp([30.0, 128.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([50.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([70.0, 128.0], PointerPhase::Move));
    let washed = px(&t, size, 30, 128);
    assert_ne!(
        washed,
        [255, 255, 255, 255],
        "the stroke start is washed live"
    );
    // Poke a sentinel into the already-painted area. Every later dab lands ≥ 40 px away — far beyond
    // the influence radius (radius 6 + spread 4 + pads) — so a frame-local recomposite must not touch it.
    const SENTINEL: [u8; 4] = [7, 250, 11, 255];
    {
        let buf = Arc::make_mut(&mut t.canvas_rgba);
        let i = ((128 * size + 30) * 4) as usize;
        buf[i..i + 4].copy_from_slice(&SENTINEL);
    }
    // Extend the stroke far to the right: the live passes recomposite only around the new dabs.
    t.on_canvas_pointer(cp([120.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([170.0, 128.0], PointerPhase::Move));
    assert_eq!(
        px(&t, size, 30, 128),
        SENTINEL,
        "a live pass recomposited far behind the frontier — the dirty rect is not frame-local"
    );
    // The frame dirty rect is consumed by each composite; the cumulative one spans the whole band.
    assert!(t.paint.wet_frame_dirty.is_none(), "frame rect consumed");
    let cum = t.paint.wet_cum_dirty.expect("cumulative rect tracked");
    // (The stroke smoother lags the dabs behind the pointer, so the right edge trails the cursor.)
    assert!(
        cum.x <= 25 && cum.x + cum.w >= 120,
        "cumulative rect spans the stroke: {cum:?}"
    );
    // Pen-up: the bake recomposites the WHOLE stroke from the tracked bbox — the sentinel is repainted.
    assert!(t.on_canvas_pointer(cp([220.0, 128.0], PointerPhase::Up)));
    let baked = px(&t, size, 30, 128);
    assert_ne!(
        baked, SENTINEL,
        "the pen-up bake recomposites the full stroke"
    );
    assert_ne!(
        baked,
        [255, 255, 255, 255],
        "…back to the wash, not the base"
    );
}

/// Watercolor dirty-rect × moving preview (Drag Dot/Anchored/Line): those methods CLEAR the coverage and
/// re-stamp the whole shape each frame, so the frame dirty rect must be the UNION of the old + new shape
/// (`clear_wet_coverage` folds the cumulative rect in) — a rect of only the new dabs would leave the old
/// position composited as a stale trail. A Drag Dot moved across the canvas must restore the base at its
/// old position, live.
#[test]
fn watercolor_moving_preview_restores_the_old_position() {
    let size = 96u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 5.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.70],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 3.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 2.0,
        stroke_method: StrokeMethod::DragDot,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([70.0, 32.0], PointerPhase::Down)));
    assert_ne!(
        px(&t, size, 70, 32),
        [255, 255, 255, 255],
        "the dot preview is washed at the press point"
    );
    // Drag the dot far away: the old position is no longer covered → the live pass restores the base.
    t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Move));
    assert_eq!(
        px(&t, size, 70, 32),
        [255, 255, 255, 255],
        "the moved preview left a stale trail at the old position"
    );
    assert_ne!(
        px(&t, size, 24, 32),
        [255, 255, 255, 255],
        "the dot is washed at the new position"
    );
    assert!(t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Up)));
    assert_ne!(
        px(&t, size, 24, 32),
        [255, 255, 255, 255],
        "the release point keeps the committed dot"
    );
}

/// Watercolor render-path is gated on an OPEN stroke (the frozen base exists): the shape editors
/// (Line/Arc/Ellipse/Polygon/Free Hand, `stroke_multi`) stamp via the drag-preview WITHOUT the stroke
/// lifecycle — routed into the watercolor accumulation they painted NOTHING (no composite ever ran)
/// and leaked never-cleared coverage. Outside a stroke a dab must fall through to the plain deposit.
#[test]
fn watercolor_editor_stamp_deposits_without_an_open_stroke() {
    let size = 48u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.85],
        watercolor: true,
        ..Default::default()
    };
    // No pointer Down: this is how the shape editors stamp (stamp_drag_preview → stamp_dabs).
    let dab = Dab {
        center: [24.0, 24.0],
        radius_px: 8.0,
        coverage: 1.0,
        color: t.paint.brush.color,
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    };
    t.stamp_dabs(&[dab]);
    assert_ne!(
        px(&t, size, 24, 24),
        [255, 255, 255, 255],
        "an editor dab with Watercolor on must paint (plain deposit), not a dead brush"
    );
    assert!(
        t.paint.stroke_coverage.iter().all(|&c| c == 0),
        "no watercolor coverage may leak outside an open stroke"
    );
}

/// Manual perf probe (not a gate): per-frame watercolor cost along a LONG stroke on a big canvas —
/// the dirty-rect must keep it ~constant (the old cumulative recomposite grew it ~quadratically).
/// Run: `cargo test -p ph2d-tool-painter --release -- --ignored watercolor_perf`
#[test]
#[ignore = "manual perf probe — run in --release and read the printed ms"]
fn watercolor_perf_frame_cost_probe() {
    probe(0.0);
    probe(1.0);
}

fn probe(wet: f32) {
    let size = 2048u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 16.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.70],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 2.0,
        edge_spread: 8.0,
        granulation: 0.4,
        warp: 3.0,
        fill: 0.5,
        depth: 2.0,
        wet_rewet: wet,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    let y = 1024.0f32;
    assert!(t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down)));
    let n = 440usize; // ~1760 px of stroke, 4 px per Move
    let mut ms = Vec::with_capacity(n);
    for i in 0..n {
        let x = 100.0 + (i as f32 + 1.0) * 4.0;
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let t0 = std::time::Instant::now();
    assert!(t.on_canvas_pointer(cp([100.0 + n as f32 * 4.0, y], PointerPhase::Up)));
    let commit_ms = t0.elapsed().as_secs_f64() * 1e3;
    let avg = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    let max = ms.iter().cloned().fold(0.0f64, f64::max);
    eprintln!(
        "watercolor per-frame (wet {wet}): first-40 {:.3} ms · last-40 {:.3} ms · max {max:.3} ms · commit {commit_ms:.3} ms ({n} moves, {size}² canvas)",
        avg(&ms[..40]),
        avg(&ms[n - 40..]),
    );
    eprintln!("total live {:.1} ms", ms.iter().sum::<f64>());
}

/// DIFERENCIAL (diagnóstico da regressão do Spread, Enio 2026-07-06): o composite incremental
/// (dirty-rect por frame) deve ser equivalente a recompor o bbox cumulativo inteiro a cada frame
/// (o comportamento antigo). Pinta um traço vivo com Edge/Spread/Warp/Granulation realistas, então
/// força UMA recomposição full do cumulativo (sem commit) e compara byte a byte (tolerância ±1 por
/// arredondamento de prefix-sum do blur). Divergência ⇒ o dirty-rect deixa pixels stale.
#[test]
fn watercolor_incremental_composite_matches_full_recompose() {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.24, 0.39, 0.63],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 3.0,
        edge_spread: 12.0,
        granulation: 0.4,
        warp: 2.5,
        fill: 0.35,
        depth: 2.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Long diagonal live stroke, pen still down.
    assert!(t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down)));
    for i in 1..=40 {
        let p = 30.0 + i as f32 * 4.5;
        t.on_canvas_pointer(cp([p, 30.0 + i as f32 * 3.5], PointerPhase::Move));
    }
    let incremental: Vec<u8> = t.canvas_rgba.to_vec();
    // Force ONE full recompose of the whole cumulative bbox (what the old code did every frame).
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    let mut worst = 0i32;
    let mut worst_i = 0usize;
    for (i, (a, b)) in incremental.iter().zip(full.iter()).enumerate() {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        if d > worst {
            worst = d;
            worst_i = i;
        }
    }
    assert!(
        worst <= 1,
        "incremental deixou pixel stale: Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// **Paridade incremental com ÁGUA (Enio 2026-07-09, "retângulo no preview com Charge 1 +
/// Dilution > 0"):** o composite vivo por dirty-rect tem que bater com a recomposição full
/// TAMBÉM com o canal d'água ativo — o anel lê o halo numa coordenada SERRILHADA (±JAG_PX), e a
/// janela viva não padava esse deslocamento: perto da borda da janela o blur do halo perdia
/// suporte e os valores mudavam a cada frame (retângulos que somem no pen-up).
#[test]
fn watercolor_incremental_composite_matches_full_with_water() {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.24, 0.39, 0.63],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 3.0,
        edge_spread: 12.0,
        granulation: 0.4,
        warp: 2.5,
        fill: 0.35,
        depth: 2.0,
        wet_rewet: 0.3,
        wet_dilution: 0.6, // água carregada — o caso do smoke (Charge 1 default)
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A assado (SEM água) — o anel/lift d'água só liga sobre PIGMENTO (bp_ring > 0); numa
    // tela virgem o halo nem é lido e a paridade passa vazia.
    t.paint.brush.wet_dilution = 0.0;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp([30.0 + i as f32 * 4.5, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([210.0, 60.0], PointerPhase::Up));
    // Traço B com ÁGUA, VIVO, cruzando o wash em diagonal — paridade no estado ao vivo.
    t.paint.brush.wet_dilution = 0.6;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down)));
    for i in 1..=40 {
        let p = 30.0 + i as f32 * 4.5;
        t.on_canvas_pointer(cp([p, 30.0 + i as f32 * 3.5], PointerPhase::Move));
    }
    let incremental: Vec<u8> = t.canvas_rgba.to_vec();
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    let mut worst = 0i32;
    let mut worst_i = 0usize;
    for (i, (a, b)) in incremental.iter().zip(full.iter()).enumerate() {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        if d > worst {
            worst = d;
            worst_i = i;
        }
    }
    assert!(
        worst <= 1,
        "incremental com água deixou pixel stale: Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// Paridade incremental×full nos **params do APP** (investigação 2026-07-09, doc 12 take 7): todo
/// repro anterior do harness rodou `Falloff::Constant`/hardness 1/warp 0/gran 0/sem papel/radius
/// 12-14/**sem `on_tick`** — o app real roda o preset Watercolor (feather auto-shape, warp 6,
/// gran 0.3, PaperCold, spacing 0.05), radius 60-100 e o heartbeat por frame (soak/secagem ativos).
/// Este cenário replica o gesto do smoke do Enio (wash assado + traço diagonal VIVO cruzando) na
/// escala do app, com `on_tick(16)` intercalado por Move. O retângulo do preview (sintoma B) é a
/// classe "janela incremental ≠ full": se este teste FALHAR, a reprodução do gap harness×app está
/// fechada na árvore.
fn watercolor_app_params_incremental_vs_full(
    wet_charge: f32,
    edge_spread: f32,
    probe_at: Option<u32>,
) -> (Vec<u8>, Vec<u8>, u32) {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let size = 512u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    // Preset "Watercolor Basic" (watercolor_settings::apply_brush_preset idx 1) + água do smoke.
    // Falloff/hardness ficam no default do app (Smooth/0 + watercolor_shape_auto = feather).
    // Gap restante conhecido: pressão fixa 1.0 (o desktop manda pressão real; dynamics.size_pressure
    // encolhe o primeiro dab) — o dump [wet-diag] do app fecha esse resíduo.
    t.paint.brush = BrushSpec {
        radius_px: 80.0,
        color: [0.24, 0.39, 0.63],
        spacing: 0.05,
        watercolor: true,
        fill: 0.12,
        depth: 1.2,
        edge_gain: 3.0,
        edge_spread,
        warp: 6.0,
        granulation: 0.30,
        pigment: false,
        paper: TextureSettings {
            kind: TextureKind::PaperCold,
            mapping: TextureMapping::Tiled,
            ..TextureSettings::default()
        },
        wet_rewet: 0.3,
        wet_dilution: 0.0, // wash A sem água (liga no traço B)
        wet_charge,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A assado (SEM água), horizontal — com o heartbeat do app entre os Moves.
    assert!(t.on_canvas_pointer(cp([60.0, 250.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp([60.0 + i as f32 * 10.0, 250.0], PointerPhase::Move));
        t.on_tick(16.0);
    }
    t.on_canvas_pointer(cp([460.0, 250.0], PointerPhase::Up));
    t.on_tick(16.0);
    // Traço B com ÁGUA, VIVO, cruzando o wash em diagonal (cruza em ~(250,250)).
    t.paint.brush.wet_dilution = 0.6;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([100.0, 60.0], PointerPhase::Down)));
    for i in 1..=40 {
        t.on_canvas_pointer(cp(
            [100.0 + i as f32 * 7.5, 60.0 + i as f32 * 9.5],
            PointerPhase::Move,
        ));
        // O app entrega ~2 eventos de ponteiro por frame (120 Hz pointer / 60 Hz frame): um segundo
        // Move no MESMO batch antes do tick, como o shell faz.
        t.on_canvas_pointer(cp(
            [
                100.0 + (i as f32 + 0.5) * 7.5,
                60.0 + (i as f32 + 0.5) * 9.5,
            ],
            PointerPhase::Move,
        ));
        t.on_tick(16.0);
        // Dwell do gesto real: logo após cruzar o wash o artista PARA a caneta (~2 s) — o soak
        // jorra sob a nib parada e liga o branch global `soaked` dos fields; só a janela do frame
        // é recomposta, o resto da união fica com o field pré-soak (o retângulo).
        if i == 22 {
            for _ in 0..10 {
                t.on_tick(200.0);
            }
        }
        // Probe MID-STROKE: o retângulo do smoke é transiente (frames posteriores repintam por
        // cima); a comparação de estado-final não o captura — esta sim.
        if probe_at == Some(i) {
            let incremental: Vec<u8> = t.canvas_rgba.to_vec();
            t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
            t.apply_watercolor(false);
            let full: Vec<u8> = t.canvas_rgba.to_vec();
            return (incremental, full, size);
        }
    }
    let incremental: Vec<u8> = t.canvas_rgba.to_vec();
    t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
    t.apply_watercolor(false);
    let full: Vec<u8> = t.canvas_rgba.to_vec();
    (incremental, full, size)
}

fn worst_byte_delta(incremental: &[u8], full: &[u8]) -> (i32, usize) {
    let mut worst = 0i32;
    let mut worst_i = 0usize;
    for (i, (a, b)) in incremental.iter().zip(full.iter()).enumerate() {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        if d > worst {
            worst = d;
            worst_i = i;
        }
    }
    (worst, worst_i)
}

/// Diag espacial do gap (rode com `--ignored --nocapture`): o diff incremental×full forma um
/// RETÂNGULO coerente (o artefato do smoke) ou speckle disperso (ruído de arredondamento)?
#[test]
#[ignore = "diag exploratório — imprime o mapa espacial do diff incremental×full nos params do app"]
fn watercolor_app_params_diff_spatial_map() {
    for (label, charge, spread, probe) in [
        ("diluted(chg=1,spr=7)", 1.0f32, 7.0f32, None),
        ("diluted(chg=1,spr=30)", 1.0, 30.0, None),
        ("mixer(chg=0.7,spr=30)", 0.7, 30.0, None),
        ("MID diluted(chg=1,spr=7)@23", 1.0, 7.0, Some(23)),
        ("MID diluted(chg=1,spr=30)@23", 1.0, 30.0, Some(23)),
        ("MID mixer(chg=0.7,spr=30)@23", 0.7, 30.0, Some(23)),
    ] {
        let (inc, full, size) = watercolor_app_params_incremental_vs_full(charge, spread, probe);
        let s = size as usize;
        let (worst, _) = worst_byte_delta(&inc, &full);
        let mut count = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        // Mapa 32×32 (célula = 16px): nº de pixels com Δ≥1 por célula.
        let mut grid = vec![0u32; 32 * 32];
        for i in 0..s * s {
            let d = (0..4)
                .map(|c| (i32::from(inc[i * 4 + c]) - i32::from(full[i * 4 + c])).abs())
                .max()
                .unwrap_or(0);
            if d >= 1 {
                count += 1;
                let (x, y) = (i % s, i / s);
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
                grid[(y / 16).min(31) * 32 + (x / 16).min(31)] += 1;
            }
        }
        eprintln!(
            "[spatial-diag {label}] worst=Δ{worst} pixels_diff={count} bbox=({x0},{y0})..({x1},{y1})"
        );
        for gy in 0..32 {
            let row: String = (0..32)
                .map(|gx| match grid[gy * 32 + gx] {
                    0 => '.',
                    1..=9 => 'o',
                    10..=99 => 'O',
                    _ => '#',
                })
                .collect();
            eprintln!("[spatial-diag {label}] {row}");
        }
    }
}

#[test]
#[ignore = "RED conhecido (doc 12 take 7): Δ2 de staleness incremental nos params do app (sub-visível; \
tolerância do gate é ≤1). Vira gate regular quando o residual for corrigido."]
fn watercolor_app_params_incremental_matches_full_diluted() {
    // Sintoma B do smoke (Charge 1 + Dilution > 0): retângulo no preview que some no mouse-up.
    let (inc, full, size) = watercolor_app_params_incremental_vs_full(1.0, 7.0, None);
    let (worst, worst_i) = worst_byte_delta(&inc, &full);
    assert!(
        worst <= 1,
        "params do APP: incremental deixou pixel stale (retângulo do preview): Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

#[test]
#[ignore = "RED conhecido (doc 12 take 7): Δ2 de staleness incremental nos params do app com mixer \
ligado (sub-visível). Vira gate regular quando o residual for corrigido."]
fn watercolor_app_params_incremental_matches_full_mixer_on() {
    // Sintoma A do smoke (Charge < 1, mixer ligado): borda dura na junção entre traços.
    let (inc, full, size) = watercolor_app_params_incremental_vs_full(0.7, 7.0, None);
    let (worst, worst_i) = worst_byte_delta(&inc, &full);
    assert!(
        worst <= 1,
        "params do APP c/ mixer: incremental deixou pixel stale na travessia: Δ{} no byte {} (px {},{})",
        worst,
        worst_i,
        (worst_i / 4) % size as usize,
        (worst_i / 4) / size as usize
    );
}

/// Granulation **Amount is inert without a settling substrate** (Enio 2026-07-06): with NO Grain image
/// and "Same as Paper" OFF there is nothing to settle into, so cranking Amount must not texture the
/// wash (it granulated out of thin air via the built-in-noise fallback). With Same-as-Paper ON (the
/// default) the paper tooth — built-in noise before a Paper is wired — granulates as before
/// (`watercolor_granulation_textures_the_wash` pins that side).
#[test]
fn watercolor_granulation_amount_is_inert_without_a_source() {
    let size = 64u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 26.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        warp: 0.0,
        fill: 0.6,
        granulation: 1.0,             // full Amount…
        granulation_use_paper: false, // …but no source: Same-as-Paper off…
        ..Default::default()          // …and no Grain image set
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up)));
    // Deep-interior window (cover ≈ 1): with no substrate the wash must be FLAT (zero variance).
    let mut vals = Vec::new();
    for y in 24..40 {
        for x in 24..40 {
            vals.push(f64::from(px(&t, size, x, y)[0]));
        }
    }
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let var = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64;
    assert!(
        var < 0.5,
        "Amount with no Grain image and Same-as-Paper off must not granulate (variance {var:.2})"
    );
}

/// Watercolor **Wet** (wet-on-wet rewetting, per-pixel, no physics — Enio 2026-07-06): a wash crossing
/// a dry red band must (a) LIFT the paint under it (the band under the wash reads lighter — pigment
/// pulled off the paper), and (b) DISSOLVE its colour into the wet region (the wash a few px OUTSIDE
/// the band reads redder — the one-shot diffusion bleed). Smudge 0 isolates the rewet; Wet 0 is the
/// control (and stays byte-identical to the plain wash, which the 13 base watercolor tests pin).
#[test]
fn watercolor_wet_lifts_and_bleeds_the_painted_paint() {
    fn run(wet: f32) -> PainterTool {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (40..70).contains(&x) {
                    [217u8, 13, 13, 255] // dry red band mid-canvas
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62], // a light blue wash — lift/bleed must read through it
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the rewet from the edge pooling
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            wet_smudge: 0.0,
            wet_rewet: wet,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 100.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up)));
        t
    }
    let size = 128u32;
    let dry = run(0.0);
    let wet = run(1.0);
    // (a) LIFT: deep inside the band, under the wash, the band's pigment is pulled off the paper — the
    // channel it absorbed least (R, its own reflectance) brightens strongly toward the paper. (Overall
    // luminance is NOT the right meter: the dissolved red simultaneously tints the wash, darkening G/B —
    // the pigment redistributes rather than vanishing.)
    let (ix, iy) = (55u32, 64u32);
    let d = px(&dry, size, ix, iy);
    let w = px(&wet, size, ix, iy);
    assert!(
        w[0] > d[0] + 40,
        "Wet must lift the paint under the wash: wet {w:?} vs dry {d:?}"
    );
    // (b) BLEED: in the wash a few px OUTSIDE the band, the dissolved red tints the wet region.
    let (ox, oy) = (73u32, 64u32);
    let d = px(&dry, size, ox, oy);
    let w = px(&wet, size, ox, oy);
    let redness = |c: [u8; 4]| i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2;
    assert!(
        redness(w) > redness(d) + 15,
        "Wet must bleed the dissolved colour beyond the band: wet {w:?} vs dry {d:?}"
    );
}

/// Wet **redistributes the wash's own pigment** on ANY canvas (blank included): more water = the
/// interior thins (pigment migrates out) while the receding front pools harder — so the Spread ring
/// reads MORE intense under Wet, never drowned (the old uniform pool + the white-canvas presence bug
/// flattened it — Enio 2026-07-06). On blank canvas there is still no lift/dissolve (nothing darkens
/// the paper), only the redistribution.
#[test]
fn watercolor_wet_redistributes_the_wash_on_blank_canvas() {
    fn run(wet: f32) -> PainterTool {
        let size = 96u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 2.0,
            edge_spread: 5.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.4,
            depth: 2.0,
            wet_rewet: wet,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([24.0, 48.0], PointerPhase::Down)));
        for x in [36.0, 48.0, 60.0, 72.0] {
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([72.0, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let dry = run(0.0);
    let wet = run(1.0);
    let lum = |c: [u8; 4]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
    // Deep interior of the band: the wet wash is LIGHTER (its pigment migrated to the front).
    let di = px(&dry, size, 48, 48);
    let wi = px(&wet, size, 48, 48);
    assert!(
        lum(wi) > lum(di),
        "Wet must thin the wash interior: wet {wi:?} vs dry {di:?}"
    );
    // The rim band (just inside the boundary): the wet wash pools HARDER (darker ring).
    let dr = px(&dry, size, 48, 41);
    let wr = px(&wet, size, 48, 41);
    assert!(
        lum(wr) < lum(dr),
        "Wet must intensify the receding-front pool: wet {wr:?} vs dry {dr:?}"
    );
}

/// Wet lift **stays in the paint's hue without Pigment** (Enio 2026-07-06, screenshot: sem Pigment a
/// tinta rewetted ficava "pálida e amarelada"): rewetting red paint with a red wash must read light
/// RED (pink) — the density-proportional log-space lift walks the colour down its own Beer–Lambert
/// curve — never the cream of the virtual paper (the old linear lerp desaturated straight to cream,
/// R−G collapsing). Pigment OFF is the whole point here.
#[test]
fn watercolor_wet_lift_stays_in_hue_without_pigment() {
    let size = 96u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for px4 in src.chunks_exact_mut(4) {
        px4.copy_from_slice(&[217, 13, 13, 255]); // a dry red wash everywhere
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.05, 0.05], // red over red — hue must survive the lift
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 7.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.12,
        depth: 1.2,
        pigment: false, // the un-checked path under test
        wet_smudge: 0.0,
        wet_rewet: 1.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([30.0, 48.0], PointerPhase::Down)));
    for x in [42.0, 54.0, 66.0] {
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([66.0, 48.0], PointerPhase::Up)));
    // Deep interior of the wash: lifted (lighter than the dry red) but still RED-dominant, not cream.
    let c = px(&t, size, 48, 48);
    assert!(c[0] > 220, "the lift lightened the red: {c:?}");
    assert!(
        i32::from(c[0]) - i32::from(c[1]) > 60,
        "the lifted paint must stay in hue (pink, not cream): {c:?}"
    );
    assert!(
        (i32::from(c[1]) - i32::from(c[2])).abs() < 20,
        "no yellow cast (G≈B for a lifted red): {c:?}"
    );
}

/// At full Wet the subtractive paint-mix runs at full strength with or WITHOUT the Pigment checkbox —
/// `mix = max(Pigment's Mix, wet)` — so the two paths converge byte-identical (the RYB blend is "o
/// segredo" of the good wet-on-wet, Enio 2026-07-06; it must not be locked behind the checkbox). At
/// `wet = 0` only the checkbox drives it (the byte-identical default, pinned by the base suite).
#[test]
fn watercolor_wet_drives_the_paint_mix_without_pigment() {
    fn run(pigment: bool) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px4 in src.chunks_exact_mut(4) {
            px4.copy_from_slice(&[217, 13, 13, 255]); // dry red everywhere
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 12.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.85], // blue wash over red — the mix is unmistakable
            space_attenuation: false,
            watercolor: true,
            edge_gain: 2.0,
            edge_spread: 7.0,
            granulation: 0.3,
            warp: 3.0,
            fill: 0.12,
            depth: 1.2,
            pigment,
            wet_smudge: 0.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([30.0, 48.0], PointerPhase::Down)));
        for x in [42.0, 54.0, 66.0] {
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([66.0, 48.0], PointerPhase::Up)));
        t
    }
    let off = run(false);
    let on = run(true);
    assert_eq!(
        off.canvas_rgba.as_slice(),
        on.canvas_rgba.as_slice(),
        "at wet = 1 the paint-mix must run fully with Pigment unchecked (max(Mix, wet) = 1 both ways)"
    );
}

/// **T1 (doc 11 §5 F1) — the beige is dead:** a watercolor stroke on a TRANSPARENT layer over a
/// white layer below must flatten to the SAME appearance as the identical stroke painted directly
/// on an opaque white base. The old virtual-cream ground baked `T·PAPER·film_a` of beige into the
/// pixels — over a white backdrop the wash carried a permanent warm cast ("puxa para o bege").
#[test]
fn watercolor_ground_is_the_real_backdrop_not_a_virtual_cream() {
    let size = 96u32;
    fn wet_brush() -> BrushSpec {
        BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.15, 0.15],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 1.5,
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.2,
            depth: 1.2,
            wet_smudge: 0.0,
            wet_rewet: 0.0,
            ..Default::default()
        }
    }
    fn stroke(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([20.0, 48.0], PointerPhase::Down)));
        let mut x = 20.0f32;
        while x < 76.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
    }
    // (a) Reference: the stroke painted directly on an opaque white base.
    let mut direct = PainterTool::default();
    direct.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    direct.paint.brush = wet_brush();
    for slot in &mut direct.paint.brush_by_mode {
        *slot = direct.paint.brush;
    }
    stroke(&mut direct);
    // (b) The stroke on a TRANSPARENT layer added above the same white base.
    let mut layered = PainterTool::default();
    layered.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    layered.add_raster_layer("wash").expect("add layer");
    layered.paint.brush = wet_brush();
    for slot in &mut layered.paint.brush_by_mode {
        *slot = layered.paint.brush;
    }
    stroke(&mut layered);
    // Flatten (b) through the real compositor and compare inside the wash.
    let active = layered.layers.active().expect("active");
    let src = crate::tool::ToolPixelSource {
        active_id: active,
        active_rgba: &layered.canvas_rgba,
        images: &layered.images,
    };
    let flat = crate::compositor::composite(&layered.layers, &src, size, size);
    let mut worst = 0i32;
    for y in 40..57u32 {
        for x in 24..72u32 {
            let i = ((y * size + x) * 4) as usize;
            let d = px(&direct, size, x, y);
            for c in 0..3 {
                worst = worst.max((i32::from(flat[i + c]) - i32::from(d[c])).abs());
            }
        }
    }
    assert!(
        worst <= 2,
        "flatten(transparent layer over white) must equal painting on white directly \
         (un-premultiply bake, no ground baked in); worst channel delta {worst}"
    );
}

/// **T3-cinza (doc 11 §5 F1) — the rewet presence is ground-relative:** with the document PAPER
/// COLOUR set to the same mid-gray as the canvas, a plain gray canvas IS the paper — nothing to
/// lift, so Wet must not brighten the wash's interior. (A gray canvas under the default WHITE
/// paper is legitimately liftable paint — Rebelle rewets a gray fill the same way; the paper
/// colour field is exactly how the artist declares "this gray is my paper".) The old global-cream
/// reference had no such control and read ANY non-cream ground as paint.
#[test]
fn watercolor_wet_reads_no_paint_on_a_paper_colored_ground() {
    fn run(wet: f32) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px4 in src.chunks_exact_mut(4) {
            px4.copy_from_slice(&[100, 100, 100, 255]); // uniform mid-gray, no paint anywhere
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.set_paper_color_rgb8(100, 100, 100); // declare the gray as the document paper
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the rewet from the edge pooling
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            wet_smudge: 0.0,
            wet_rewet: wet,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 80.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let dry = run(0.0);
    let wet = run(1.0);
    // Interior of the wash: with no paint below, wet vs dry may differ only by the wash's own
    // redistribution (interior thinning) — never by a lift toward a foreign paper colour.
    for x in [30u32, 48, 66] {
        let d = px(&dry, size, x, 48);
        let w = px(&wet, size, x, 48);
        for c in 0..3 {
            let delta = (i32::from(w[c]) - i32::from(d[c])).abs();
            assert!(
                delta <= 12,
                "plain gray must not be lifted (presence 0): x={x} c={c} wet {w:?} vs dry {d:?}"
            );
        }
    }
}

/// **T2 (doc 11 §5 F1) — paint LIGHTER than the old cream is liftable now:** a pale near-white
/// pink band (250,225,225) on white reads presence 0 against the old cream reference (no channel
/// darker than the paper ⇒ invisible to the rewet); against the real white ground its |Δ| = 30 on
/// G/B registers, so Wet lifts it toward white.
#[test]
fn watercolor_wet_lifts_paint_lighter_than_the_old_cream() {
    fn run(wet: f32) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (36..60).contains(&x) {
                    [250u8, 225, 225, 255] // pale pink band — LIGHTER than the old cream paper
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.98, 0.98, 0.98], // near-clear water: the lift dominates, not the wash's film
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            opacity: 0.0, // near-clear water = no body film; isolate the wet LIFT (its own test)
            wet_smudge: 0.0,
            wet_rewet: wet,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 80.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let dry = run(0.0);
    let wet = run(1.0);
    let d = px(&dry, size, 48, 48);
    let w = px(&wet, size, 48, 48);
    assert!(
        i32::from(w[1]) >= i32::from(d[1]) + 4 && i32::from(w[2]) >= i32::from(d[2]) + 4,
        "Wet must lift a pale band toward the white ground (old cream reference read it as \
         presence 0): wet {w:?} vs dry {d:?}"
    );
}

/// **F3 (doc 11 §5) — soak: the longer the water sits, the farther/deeper it dissolves.** Holding
/// the wet brush parked over a dry band (the tick heartbeat pours dwell) must (a) deepen the lift
/// under the nib and (b) push the dissolved tint FARTHER outside the band than a pass-through
/// stroke — the dissolve's blur lerps toward a 2× radius where the soak accumulated.
#[test]
fn watercolor_soak_deepens_and_widens_the_dissolve_while_parked() {
    fn run(hold_s: f32) -> PainterTool {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (52..76).contains(&x) {
                    [217u8, 13, 13, 255] // dry red band mid-canvas
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            wet_smudge: 0.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down)));
        let mut x = 40.0f32;
        while x < 64.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        // Park over the band: each tick pours soak under the nib (0 ticks = pass-through control).
        let mut held = 0.0f32;
        while held < hold_s {
            t.paint_tick(0.1);
            held += 0.1;
        }
        assert!(t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up)));
        t
    }
    let size = 128u32;
    let quick = run(0.0);
    let held = run(2.0);
    // (a) Deeper lift under the parked nib (soak boosts the lift fraction).
    let q = px(&quick, size, 62, 64);
    let h = px(&held, size, 62, 64);
    // R saturates near the ground already (the band's own reflectance) — the deepened lift +
    // boosted dissolve read on the absorbed channels (G/B rise as more red mass is pulled out).
    assert!(
        i32::from(h[1]) > i32::from(q[1]) + 6,
        "2 s of dwell must deepen the lift under the nib: held {h:?} vs quick {q:?}"
    );
    // (b) Wider bleed: the dissolved red reaches farther LEFT of the band (into the wash) after
    // the hold — measure the farthest x (walking away from the band edge at 52) still tinted.
    let redness = |t: &PainterTool, x: u32| {
        let c = px(t, size, x, 64);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    let extent = |t: &PainterTool| {
        // Baseline: wash 18 px from the band — beyond even the 2× (soaked) blur radius.
        let base = redness(t, 34);
        // Walk AWAY-from-band → band edge; the FIRST tinted x is the farthest reach of the bleed.
        let mut e = 0u32;
        for x in 35..52u32 {
            if redness(t, x) > base + 10 {
                e = 52 - x;
                break;
            }
        }
        e
    };
    let eq = extent(&quick);
    let eh = extent(&held);
    // Also compare the total tint MASS beyond the band edge — the first-crossing extent is
    // threshold-granular, the mass meter sees the whole widened profile.
    let mass = |t: &PainterTool| {
        let base = redness(t, 34);
        (40..52u32)
            .map(|x| (redness(t, x) - base).max(0))
            .sum::<i32>()
    };
    let (mq, mh) = (mass(&quick), mass(&held));
    // Margin 1.15: measured +21% at the default knobs (SOAK_DISSOLVE doubles the tint under full
    // soak; the deepened lift is asserted above) — deterministic engine, so no flake headroom
    // needed. The perceptual tuning surface is the named SOAK_* consts (doc 11 §5 F3).
    assert!(
        eh >= eq && mh as f32 >= mq as f32 * 1.15,
        "2 s of dwell must push the dissolved tint farther/heavier into the wash:          held {eh}px/mass {mh} vs quick {eq}px/mass {mq}"
    );
}

/// **Spread clears the centre of the pool** (Enio 2026-07-07): a wet pool's interior LIGHTENS as the
/// pigment migrates to the receding front, and — the recovered dynamic — the clearing gets STRONGER
/// with Spread (a wider wet front empties the centre more). Before the fix, raising the cap to 48 let
/// Spread exceed the pool radius, `inner = blur(cov)` never saturated, and the edge term FLOODED the
/// centre (flat dark blob). The `core_r` cap + Spread-scaled thinning restore + strengthen it.
#[test]
fn watercolor_spread_clears_the_pool_centre() {
    fn centre_vs_rim(spread: f32) -> (i32, i32) {
        let size = 200u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 34.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.20, 0.35, 0.75],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 2.0,
            edge_spread: spread,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.5,
            depth: 2.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Up)));
        let lum = |c: [u8; 4]| {
            (0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32) as i32
        };
        // Centre (x=100) vs a rim sample well inside the pool's dark ring (x=118, ~18px out).
        (lum(px(&t, size, 100, 100)), lum(px(&t, size, 118, 100)))
    }
    let (c48, r48) = centre_vs_rim(48.0);
    let (c16, _) = centre_vs_rim(16.0);
    // (a) At high Spread the centre is LIGHTER than the rim (the pool clears, not floods).
    assert!(
        c48 > r48 + 40,
        "high Spread must clear the pool centre: centre {c48} vs rim {r48}"
    );
    // (b) The clearing SCALES with Spread — Spread 48 clears the centre more than Spread 16.
    assert!(
        c48 > c16 + 20,
        "the clearing must strengthen with Spread: centre@48 {c48} vs centre@16 {c16}"
    );
}

/// **High-Spread live cost stays bounded** (Enio 2026-07-07 FPS fix): the rewet blur fields
/// downsample at wide Spread (`RewetFields`, `ds > 1`) + the no-Wet window uses the capped feather
/// reach, so a Spread-48 stroke's per-frame recomposite is a small multiple of the Spread-8 cost,
/// NOT the ~9× the full-res spread²-window path cost (measured 10.3 → 3.0 ms @2048²). Asserts the
/// SHAPE of the scaling (ratio), not an absolute ms — deterministic, machine-independent.
#[test]
#[ignore] // release-only timing; run with `--release -- --ignored`
fn watercolor_high_spread_frame_cost_bounded() {
    fn live_ms(spread: f32, dwell: bool) -> f64 {
        let size = 2048u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 16.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.70],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 2.0,
            edge_spread: spread,
            granulation: 0.4,
            warp: 3.0,
            fill: 0.5,
            depth: 2.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        let y = 1024.0f32;
        assert!(t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down)));
        let n = 80usize;
        let mut ms = Vec::with_capacity(n);
        for i in 0..n {
            let x = 100.0 + (i as f32 + 1.0) * 4.0;
            if dwell {
                for _ in 0..3 {
                    t.paint_tick(0.033);
                }
            }
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        ms.iter().sum::<f64>() / ms.len() as f64
    }
    let lo = live_ms(8.0, false);
    let hi = live_ms(48.0, true);
    eprintln!(
        "live spread=8 {lo:.3} ms · spread=48+dwell {hi:.3} ms · ratio {:.1}",
        hi / lo
    );
    // The old spread²-window path made this ratio ~12×; the downsample + capped reach keep it low.
    assert!(
        hi < lo * 8.0,
        "high-Spread dwell frame cost must stay a small multiple of the baseline: \
         {hi:.3} ms vs {lo:.3} ms (ratio {:.1})",
        hi / lo
    );
}

/// **T5 (doc 11 §5 F2) — the Wet Mix carries picked-up colour downstream.** A wet mixer brush
/// (Charge < 1, some Pull) crossing a dry RED band on white picks the red up and drags it along the
/// gesture: downstream of the band the deposited stroke is redder than the same stroke with the mixer
/// OFF (Charge 1), and the carried red DECAYS with distance as the brush resamples the white beyond.
#[test]
fn watercolor_wet_mix_carries_colour_downstream() {
    fn run(charge: f32) -> PainterTool {
        let size = 160u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (44..56).contains(&x) {
                    [210u8, 30, 30, 255] // dry red band
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 7.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.20, 0.35, 0.75], // blue brush — carried red reads as a purple shift
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the mixer from edge pooling
            edge_spread: 4.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.6,
            depth: 1.5,
            wet_rewet: 0.0, // isolate the mixer from the per-pixel rewet
            wet_charge: charge,
            wet_pull: 0.6,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([16.0, 80.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 130.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Up)));
        t
    }
    let size = 160u32;
    let mixed = run(0.2); // pickup 0.8
    let plain = run(1.0); // mixer off
    let redness = |t: &PainterTool, x: u32| {
        let c = px(t, size, x, 80);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    // (a) Downstream (x=70, just past the band) the mixer stroke carries red the plain one lacks.
    assert!(
        redness(&mixed, 70) > redness(&plain, 70) + 15,
        "the mixer must carry the band's red downstream: mixed {} vs plain {}",
        redness(&mixed, 70),
        redness(&plain, 70)
    );
    // (b) The carried red DECAYS with distance (near the band > far from it).
    assert!(
        redness(&mixed, 70) > redness(&mixed, 110) + 8,
        "the carried colour must decay downstream: near {} vs far {}",
        redness(&mixed, 70),
        redness(&mixed, 110)
    );
}

/// **T6 (doc 11 §5 F2) — Charge controls the pickup amount.** A lower Charge (more depleted reserve)
/// blends MORE of the picked-up surface into the deposit: painting a blue mixer stroke straight over
/// a red field, a low Charge deposits a redder (more mixed) result than a high Charge.
#[test]
fn watercolor_wet_mix_charge_controls_pickup() {
    fn run(charge: f32) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px4 in src.chunks_exact_mut(4) {
            px4.copy_from_slice(&[210, 30, 30, 255]); // dry red everywhere
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.30, 0.80],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            edge_spread: 4.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.6,
            depth: 1.5,
            wet_rewet: 0.0,
            wet_charge: charge,
            wet_pull: 0.3,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 80.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let redness = |t: &PainterTool| {
        let c = px(t, size, 60, 48);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    let low = run(0.1); // heavy pickup
    let high = run(0.9); // light pickup
    assert!(
        redness(&low) > redness(&high) + 15,
        "lower Charge must pick up more of the red surface: low {} vs high {}",
        redness(&low),
        redness(&high)
    );
}

/// **T7 (doc 11 §5 F2) — the mixer is inert at the default Charge = 1.** With a full fresh reserve
/// the brush deposits pure fresh colour: a blue stroke straight over a red field stays BLUE (no red
/// picked up), regardless of Pull — the byte-identical-default guarantee (the mixer path is skipped).
#[test]
fn watercolor_wet_mix_default_charge_deposits_pure_colour() {
    let size = 96u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for px4 in src.chunks_exact_mut(4) {
        px4.copy_from_slice(&[210, 30, 30, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.10, 0.25, 0.85],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.7,
        depth: 2.0,
        wet_rewet: 0.0,
        wet_charge: 1.0, // default → mixer OFF
        wet_pull: 0.8,   // even with Pull set, no pickup at full charge
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
    let mut x = 16.0f32;
    while x < 80.0 {
        x += 3.0;
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
    let c = px(&t, size, 60, 48);
    // Deposit is blue: B channel dominates, no red carried up from the field.
    assert!(
        c[2] > c[0] + 40,
        "Charge 1 must deposit pure fresh blue (mixer off), got {c:?}"
    );
}

/// **Wet Mix exit bleed mirrors the entry** (Enio 2026-07-07, photo). A wet mixer brush (Charge < 1,
/// Pull 0) drawn ACROSS a painted pool picks its colour up; the ENTRY into the pool always bled the
/// picked-up colour into the incoming stroke, but the EXIT was a HARD CUT — the reservoir reset to the
/// bare surface the instant the centre left, and the following dabs overwrote the carried colour
/// (source-over recency). The asymmetric load/unload reservoir (fast load, slow unload) makes the
/// picked-up colour LINGER past the pool, so the exit bleeds too. Asserts the exit is no longer a hard
/// cut (bleeds red near the pool, fading with distance) and that its red EXTENT is comparable to the
/// entry's — not a perfect mirror (the entry deposits at full pickup, the exit is a fading carry), but
/// a real symmetric-looking bleed on both sides.
#[test]
fn watercolor_wet_mix_exit_bleed_mirrors_entry() {
    let size = 160u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    let (band0, band1) = (55u32, 105u32); // wide red pool
    for y in band0..band1 {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[210, 30, 30, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 11.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.30, 0.80],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.15,
        wet_pull: 0.0, // the reported Charge-only case
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 140.0 {
        y += 3.0;
        t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([80.0, y], PointerPhase::Up)));
    let redness = |yy: u32| {
        let c = px(&t, size, 80, yy);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    // (a) The EXIT (below the pool) bleeds red near the edge — NOT the old flat blue (~ −113).
    let exit_near = redness(band1 + 1);
    assert!(
        exit_near > 15,
        "the exit must bleed the picked-up red past the pool (was a hard cut): {exit_near}"
    );
    // (b) The exit bleed FADES with distance (a gradient, not a slab).
    assert!(
        redness(band1 + 1) > redness(band1 + 7) + 15,
        "the exit bleed must fade with distance: near {} vs far {}",
        redness(band1 + 1),
        redness(band1 + 7)
    );
    // (c) Both sides bleed red over a comparable EXTENT (the reach mirrors, even if the entry — at
    //     full pickup — peaks higher than the fading exit carry). Count rows still red (> 8) each way.
    let entry_reach = (1..15).filter(|&d| redness(band0 - d) > 8).count();
    let exit_reach = (1..15).filter(|&d| redness(band1 + d) > 8).count();
    assert!(
        exit_reach >= 3 && exit_reach + 3 >= entry_reach,
        "the exit red reach must be comparable to the entry (mirror), entry {entry_reach} exit {exit_reach}"
    );
}

/// **Wet Mix carried colour is saturated, not watery** (Enio 2026-07-07). The mixer's disc pickup
/// averaged the RAW surface colour, so a brush half over a red pool picked up a pink AVERAGE of red +
/// white — the carried mix read bleached toward white instead of a rich blue+red purple. Presence-
/// weighting the sample (bare ground contributes to the weight, not the hue) picks up SATURATED red,
/// so the carried mix is a real purple. Asserts the carried region downstream of a red pool is
/// purple (R and B both well above G), not a pale near-grey.
#[test]
fn watercolor_wet_mix_carried_colour_is_saturated_not_watery() {
    let size = 160u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    let (band0, band1) = (55u32, 95u32);
    for y in band0..band1 {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[210, 30, 30, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.30, 0.80],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.2,
        wet_pull: 0.6,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 145.0 {
        y += 3.0;
        t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([80.0, y], PointerPhase::Up)));
    // Just past the pool: a rich purple (R and B each clearly above G), not a pale near-grey wash.
    // Margins re-pinned WITH the W-A decision (doc 12 OPT-1, 2026-07-07): the subtractive
    // (absorbance-space) mix legitimately weights the heavily-carried RED pigment more than the old
    // sRGB lerp did — deep maroon-purple (measured (166,67,92): R−G=99, B−G=25), exactly how real
    // red+blue pigment mixes. The test's INTENT is unchanged: G suppressed on both sides + strongly
    // chromatic (the original bug was a pale wash bleached toward white).
    let c = px(&t, size, 80, band1 + 3);
    let (r, g, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
    assert!(
        r > g + 60 && b > g + 15,
        "the carried mix must be a saturated purple (R,B > G), not watery: {c:?}"
    );
}

/// **Wet Mix deposit priority: a low-pickup dab can't wash out a high-pickup one** (Enio 2026-07-07).
/// The mixer scales each dab's colour-deposit alpha by its pickup strength, so a bare-ground dab
/// (leaving a pool) barely writes and cannot overwrite the picked-up colour laid by the in-pool dabs.
/// Reproduces the reported crossing: a blue mixer stroke drawn through a red pool — the pool's EXIT
/// edge must stay coloured (the picked-up red survives the exiting dabs), not wash back to plain blue.
/// (Some entry>exit difference is inherent to a DIRECTIONAL smudge — entering a pool ≠ leaving it —
/// but the exit must retain a clear share of the pickup, not be a hard cut.)
#[test]
fn watercolor_wet_mix_exit_edge_keeps_pickup() {
    let size = 200u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    let (b0, b1) = (78u32, 122u32); // red band rows
    for y in b0..b1 {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[210, 30, 30, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 20.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.30, 0.80],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.2,
        wet_pull: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([100.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 175.0 {
        y += 3.0;
        t.on_canvas_pointer(cp([100.0, y], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([100.0, y], PointerPhase::Up)));
    let purple = |yy: u32| {
        let c = px(&t, size, 100, yy);
        i32::from(c[0]) + i32::from(c[2]) - 2 * i32::from(c[1])
    };
    let entry = purple(b0 + 4); // entry edge (into the pool from the top)
    let exit = purple(b1 - 4); // exit edge (leaving the pool at the bottom)
    // The exit edge keeps a clear majority of the entry's pickup (was a near-hard-cut before the
    // priority deposit + asymmetric reservoir).
    assert!(
        exit > 60 && exit * 3 >= entry * 2,
        "the exit edge must retain the picked-up colour (not wash to blue): entry {entry} exit {exit}"
    );
}

/// **Shape-editor bake runs the watercolor wash (doc 13 #3).** A shape editor (here a Line) committed
/// with a Watercolor brush must bake the OPTICAL wash (frozen base + rim / Ragged-Edge warp) — not the
/// plain source-over deposit. RED before #3: the shape editors stamp WITHOUT the stroke lifecycle, so no
/// base is frozen (`watercolor_base` stays `None`), the `stamp_dabs` watercolor gate is false, and the
/// bake is BYTE-IDENTICAL to the same shape drawn with Watercolor OFF. GREEN: the optics diverge it.
#[test]
fn watercolor_shape_editor_bake_runs_the_wash() {
    fn line_brush(t: &mut PainterTool, watercolor: bool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            warp: 3.0, // Ragged Edge on — a signature only the wash produces
            stroke_method: StrokeMethod::Line,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    fn draw_and_commit(t: &mut PainterTool) {
        t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down)); // corner 1
        t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
        t.on_canvas_pointer(cp([8.0, 44.0], PointerPhase::Down)); // corner 2 (press)
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move)); // drag to the final spot
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
        assert!(t.commit_open_shape(), "Apply baked the open line");
    }
    let size = 64u32;
    let differs = |a: &PainterTool, b: &PainterTool| {
        (0..size * size).any(|i| px(a, size, i % size, i / size) != px(b, size, i % size, i / size))
    };
    let blank = white_canvas(size, 8.0);

    let mut plain = white_canvas(size, 8.0);
    line_brush(&mut plain, false);
    draw_and_commit(&mut plain);

    let mut wet = white_canvas(size, 8.0);
    line_brush(&mut wet, true);
    draw_and_commit(&mut wet);

    assert!(
        differs(&plain, &blank),
        "the line committed paint (control)"
    );
    assert!(
        differs(&wet, &plain),
        "watercolor optics ran on the shape bake (was byte-identical to plain before #3)"
    );
}

/// **A watercolor shape preview leaves no trail (doc 13 #3).** The moving/resizing shape preview
/// re-composites the wash each frame over the restored-pristine canvas — including the rim/warp that
/// reach BEYOND the dab bbox. If the save/restore footprint didn't cover that reach, a wrong drag would
/// leave a rim trail. Proof: a line dragged straight to its final corner and the SAME line dragged
/// through two wrong spots first must bake BYTE-IDENTICALLY (the wobble peels clean).
#[test]
fn watercolor_shape_preview_leaves_no_trail() {
    fn line_brush(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            warp: 3.0,
            stroke_method: StrokeMethod::Line,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    let size = 64u32;

    // Straight to the final corner.
    let mut direct = white_canvas(size, 8.0);
    line_brush(&mut direct);
    direct.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    direct.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    direct.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Down));
    direct.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    assert!(direct.commit_open_shape(), "direct line baked");

    // Same final corner, but dragged through two WRONG spots first.
    let mut wobbled = white_canvas(size, 8.0);
    line_brush(&mut wobbled);
    wobbled.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    wobbled.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    wobbled.on_canvas_pointer(cp([30.0, 8.0], PointerPhase::Down)); // wrong 1
    wobbled.on_canvas_pointer(cp([44.0, 56.0], PointerPhase::Move)); // wrong 2
    wobbled.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move)); // final
    wobbled.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    assert!(wobbled.commit_open_shape(), "wobbled line baked");

    for i in 0..size * size {
        let (x, y) = (i % size, i / size);
        assert_eq!(
            px(&wobbled, size, x, y),
            px(&direct, size, x, y),
            "wobble left a wash trail at ({x},{y}) — the preview footprint missed the rim reach"
        );
    }
}

/// **The watercolor wash ignores the Brush Blend mode (doc 13 #4).** The optical deposit is source-over +
/// Beer–Lambert optics — `BrushBlend` is never read on the wash path, so the Brush Blend dropdown is
/// INERT in watercolor mode (why the panel hides it there). Two washes identical but for `brush.blend`
/// bake byte-for-byte. Refutable: wiring blend into the wash turns this RED (and would un-justify the hide).
#[test]
fn watercolor_wash_ignores_the_brush_blend_mode() {
    fn wash(t: &mut PainterTool, blend: ph2d_painter_brush::BrushBlend) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            warp: 3.0,
            blend, // the wash must ignore this entirely
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Up));
    }
    let size = 64u32;
    let mut mix = white_canvas(size, 8.0);
    wash(&mut mix, ph2d_painter_brush::BrushBlend::Mix);
    let mut mult = white_canvas(size, 8.0);
    wash(&mut mult, ph2d_painter_brush::BrushBlend::Multiply);
    for i in 0..size * size {
        let (x, y) = (i % size, i / size);
        assert_eq!(
            px(&mix, size, x, y),
            px(&mult, size, x, y),
            "brush Blend changed the wash at ({x},{y}) — it must be inert in watercolor"
        );
    }
}

/// **Watercolor respects the Selection + protection-mask gates** (the audit hole, Enio 2026-07-07):
/// the optical path used to short-circuit BEFORE the canvas gates in `stamp_dabs`, so a watercolor
/// stroke painted straight through an active selection and the Sculpt-style protection scratch.
/// Now the wash never FORMS on gated-out texels (splat gates) AND the composite keep-lerps the final
/// bytes toward the frozen base (restore semantics — warp-proof: this stroke runs Ragged Edge > 0,
/// whose displaced sampling used to be the leak vector).
#[test]
fn watercolor_respects_selection_and_protection_masks() {
    fn wet_brush(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            warp: 4.0, // Ragged Edge ON: proves the composite gate stops the warped-sampling leak
            wet_rewet: 1.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    let size = 64u32;

    // ── Selection: left half selected; a stroke straddling x=32 must clip at the border. ──
    let mut t = white_canvas(size, 8.0);
    wet_brush(&mut t);
    t.set_rect_selection(0, 0, 32, 64);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    assert_ne!(
        px(&t, size, 28, 32),
        [255, 255, 255, 255],
        "inside the selection the wash painted"
    );
    for x in [38u32, 44, 50] {
        assert_eq!(
            px(&t, size, x, 32),
            [255, 255, 255, 255],
            "outside the selection stays pristine (x={x}) — the watercolor gate hole"
        );
    }

    // ── Protection scratch: right half painted black (= frozen); the wash must not land there. ──
    let mut t = white_canvas(size, 8.0);
    wet_brush(&mut t);
    t.ensure_mask_scratch();
    assert!(t.mask_scratch_active(), "scratch installed on the layer");
    {
        let scratch = Arc::make_mut(&mut t.paint.mask_scratch_rgba);
        for y in 0..size {
            for x in 32..size {
                let i = ((y * size + x) * 4) as usize;
                scratch[i] = 0; // black = protect (mask_value 0 = frozen)
                scratch[i + 1] = 0;
                scratch[i + 2] = 0;
                scratch[i + 3] = 255;
            }
        }
    }
    assert!(t.mask_protection_active(), "protection gate armed");
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    assert_ne!(
        px(&t, size, 28, 32),
        [255, 255, 255, 255],
        "unprotected side painted"
    );
    for x in [38u32, 44, 50] {
        assert_eq!(
            px(&t, size, x, 32),
            [255, 255, 255, 255],
            "protected texels stay frozen (x={x})"
        );
    }
}

/// **Alpha-lock (doc 13 #8) — the watercolor wash paints only into EXISTING alpha.**
/// Canvas = left half opaque white (α=255), right half transparent (α=0); alpha-lock ON; a wet stroke
/// (Ragged Edge on, so the warped sampling can REACH the transparent side) straddles the α boundary.
/// The opaque side takes the wash with its alpha preserved; the transparent side stays fully
/// transparent — the layer's silhouette is frozen, exactly like the non-wc dab (`acc[3] = pre_alpha`).
/// RED before the fix: the composite deposits `cov_a` alpha wherever coverage reaches, transparent or
/// not (`out_a = ab + (1−ab)·cov_a` with `ab = 0` ⇒ `out_a = cov_a > 0`).
#[test]
fn watercolor_alpha_lock_paints_only_into_existing_alpha() {
    let size = 64u32;
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
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        warp: 4.0, // Ragged Edge ON: the warped sampling reaches the transparent side (composite gate)
        wet_rewet: 1.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    let active = t.layers.active().expect("active layer");
    t.layers.get_mut(active).expect("layer").alpha_locked = true;

    // Stroke centred on the α boundary (x=32), radius 8 ⇒ the disc covers x∈[24,40].
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));

    // Opaque side: the wash landed (colour moved) and the alpha is still fully opaque.
    let inside = px(&t, size, 26, 32);
    assert_ne!(
        [inside[0], inside[1], inside[2]],
        [255, 255, 255],
        "opaque side took the watercolor wash"
    );
    assert_eq!(inside[3], 255, "opaque side alpha preserved");

    // Transparent side (inside the disc at x=36/38, and warp-reach at x=44): alpha-lock froze it.
    for x in [36u32, 38, 44] {
        assert_eq!(
            px(&t, size, x, 32)[3],
            0,
            "alpha-lock kept the transparent side transparent (x={x})"
        );
    }
}

/// **Tiling (doc 13 #2) — the watercolor wash wraps seamlessly across the sprite seam.** A wet dab
/// hard against the right edge (radius crosses x=64) with X-tiling on must ALSO deposit the wrapped
/// part on the left edge, so the painted texture tiles. RED before the fix: the watercolor route
/// short-circuits `stamp_dabs` BEFORE `tiled_dabs`, so only the original (un-wrapped) dab forms.
#[test]
fn watercolor_tiling_wraps_the_wash_across_the_seam() {
    let size = 64u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    t.paint.tiling = [true, false]; // seamless wrap on X

    // Dab at x=62 (r=8 ⇒ footprint [54,70] crosses the far edge at x=64).
    assert!(t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Up));

    assert_ne!(
        px(&t, size, 61, 32),
        [255, 255, 255, 255],
        "the wash landed on the right edge"
    );
    // The wrapped copy (shifted −64 ⇒ centre −2, footprint [−10,6]) paints x∈[0,6] on the left edge —
    // unreachable from x=62 without the wrap (distance 60 ≫ radius 8), so any paint here IS the tile.
    assert_ne!(
        px(&t, size, 2, 32),
        [255, 255, 255, 255],
        "tiling wrapped the wash onto the left edge (seamless seam)"
    );
}

/// **A dynamic SHAPE's wash crosses the Tiling seam (Enio 2026-07-11).** The shape editors re-stamp
/// through `stamp_drag_preview_watercolor` (a re-stamp preview, NOT the stroke lifecycle), which took the
/// RAW dabs — so with seamless Tiling a shape crossing the border was cut there instead of wrapping. RED
/// before the fix: the left (wrapped) edge stays pristine. GREEN: the tiled dabs form the wash on the
/// opposite edge too, matching the plain stroke (`stamp_dabs`). Control: tiling OFF leaves it pristine.
#[test]
fn watercolor_shape_wash_crosses_the_tiling_seam() {
    fn line_brush(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            stroke_method: StrokeMethod::Line,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    // A vertical Line at x=62 (r=8 ⇒ footprint [54,70] crosses the far edge at x=64): two clicks place the
    // start + end anchors, then Apply bakes.
    fn draw_and_commit(t: &mut PainterTool) {
        t.on_canvas_pointer(cp([62.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([62.0, 20.0], PointerPhase::Up));
        t.on_canvas_pointer(cp([62.0, 44.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([62.0, 44.0], PointerPhase::Up));
        assert!(t.commit_open_shape(), "Apply baked the open line");
    }
    let size = 64u32;

    // Tiling ON (X): the wrapped copy (centre −2, footprint [−10,6]) paints x∈[0,6] on the LEFT edge —
    // unreachable from x=62 without the wrap (distance 60 ≫ radius 8), so any paint here IS the tile.
    let mut tiled = white_canvas(size, 8.0);
    line_brush(&mut tiled);
    tiled.paint.tiling = [true, false];
    draw_and_commit(&mut tiled);
    assert_ne!(
        px(&tiled, size, 61, 32),
        [255, 255, 255, 255],
        "the shape wash landed on the right edge"
    );
    assert_ne!(
        px(&tiled, size, 2, 32),
        [255, 255, 255, 255],
        "the shape wash wrapped across the seam onto the left edge (was cut before the fix)"
    );

    // Control: tiling OFF ⇒ the left edge stays pristine (proves the wrap is what paints it).
    let mut plain = white_canvas(size, 8.0);
    line_brush(&mut plain);
    draw_and_commit(&mut plain);
    assert_eq!(
        px(&plain, size, 2, 32),
        [255, 255, 255, 255],
        "without tiling the shape wash never reaches the far edge (control)"
    );
}

/// **A texture-param change re-renders the still-wet wash — central AND every Tiling copy (Enio
/// 2026-07-11).** After pen-up the wash bakes, but the last wash stays re-renderable while the paper is
/// wet: changing the Grain Size re-renders the whole committed wash (not just the next stroke). The setter
/// alone is inert (only stores the value); the paint tick applies it. With Tiling on, the WRAPPED copy
/// re-renders too. RED before the feature: the baked wash never reacts to a Size change.
#[test]
fn watercolor_texture_size_rerenders_the_wet_wash_and_all_tiles() {
    let size = 64u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        granulation: 1.0, // granulation ON so the Grain texture modulates the wash visibly
        texture: ph2d_painter_brush::TextureSettings {
            kind: ph2d_painter_brush::TextureKind::Noise,
            size: [1.0, 1.0],
            ..Default::default()
        },
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    t.paint.tiling = [true, true];

    // Paint a wash at x=62 (footprint [54,70] crosses the far edge ⇒ a wrapped copy paints x∈[0,6]); lift.
    assert!(t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Up));

    let central_before = px(&t, size, 60, 32);
    let wrapped_before = px(&t, size, 2, 32);
    assert_ne!(
        central_before,
        [255, 255, 255, 255],
        "the wash baked at the right edge"
    );
    assert_ne!(
        wrapped_before,
        [255, 255, 255, 255],
        "tiling wrapped the wash to the left edge"
    );

    // The setter alone only STORES the value — the baked canvas is untouched until the tick applies it.
    t.set_brush_texture_size(0, 6.0);
    assert_eq!(
        px(&t, size, 60, 32),
        central_before,
        "the Size setter must not touch the canvas by itself"
    );

    // The paint tick re-renders the still-wet wash with the new Grain Size.
    t.paint_tick(0.016);
    let central_after = px(&t, size, 60, 32);
    let wrapped_after = px(&t, size, 2, 32);
    assert_ne!(
        central_before, central_after,
        "the wet wash re-rendered centrally with the new Grain Size"
    );
    assert_ne!(
        wrapped_before, wrapped_after,
        "the WRAPPED Tiling copy re-rendered too (all tiles update together)"
    );

    // Once the session dries the wash is permanent — a further Size change no longer re-renders it.
    t.dry_session_now();
    let dry_before = px(&t, size, 60, 32);
    t.set_brush_texture_size(0, 12.0);
    t.paint_tick(0.016);
    assert_eq!(
        px(&t, size, 60, 32),
        dry_before,
        "a dried wash is permanent — texture edits no longer re-render it"
    );
}

/// **Alpha-lock is a no-op where the layer is fully opaque (byte-identical, §0.6).** On an opaque
/// canvas every texel has `ka = 1` ⇒ the splat gate is `1.0` and the composite's α-pin re-writes the
/// already-opaque α — so a locked stroke must be byte-for-byte the same as the unlocked one.
#[test]
fn watercolor_alpha_lock_is_a_noop_on_fully_opaque() {
    fn wet(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            warp: 4.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    fn stroke(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    }
    let size = 64u32;

    let mut unlocked = white_canvas(size, 8.0);
    wet(&mut unlocked);
    stroke(&mut unlocked);

    let mut locked = white_canvas(size, 8.0);
    wet(&mut locked);
    let active = locked.layers.active().expect("active layer");
    locked.layers.get_mut(active).expect("layer").alpha_locked = true;
    stroke(&mut locked);

    for y in 0..size {
        for x in 0..size {
            assert_eq!(
                px(&locked, size, x, y),
                px(&unlocked, size, x, y),
                "alpha-lock changed a fully-opaque pixel at ({x},{y})"
            );
        }
    }
}

/// **Shape "Automatic" (doc 13 #1) — the continuity + capability contract.**
/// (a) CONTINUITY: unchecking Automatic (which auto-selects the `Falloff::Watercolor` preset — the
/// built-in feather as a curve) paints a stroke BYTE-IDENTICAL to Automatic: the manual path with the
/// default knobs is the same stamp, so the checkbox transition never pops. (b) CAPABILITY: with a
/// half-blank Shape image the manual stamp is ASYMMETRIC (the image drives the watercolor silhouette),
/// which Automatic's round feather can never produce.
#[test]
fn watercolor_shape_automatic_continuity_and_image_silhouette() {
    fn stroke(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    }
    fn wet(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 0.0,
            falloff: Falloff::Smooth,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    let size = 64u32;

    // (a) Automatic ON (default) …
    let mut auto_t = white_canvas(size, 8.0);
    wet(&mut auto_t);
    stroke(&mut auto_t);
    // … vs the panel toggle OFF (routes through the real seam: also auto-selects Falloff::Watercolor).
    let mut manual_t = white_canvas(size, 8.0);
    wet(&mut manual_t);
    manual_t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_SHAPE_WATERCOLOR_AUTO,
    ));
    let b = manual_t.brush_settings();
    assert!(!b.watercolor_shape_auto, "toggle turned Automatic off");
    assert_eq!(
        manual_t.paint.brush.falloff,
        Falloff::Watercolor,
        "unchecking auto-selects the Watercolor falloff preset (continuity)"
    );
    for slot in &mut manual_t.paint.brush_by_mode {
        *slot = manual_t.paint.brush;
    }
    stroke(&mut manual_t);
    assert_eq!(
        auto_t.canvas_rgba.as_slice(),
        manual_t.canvas_rgba.as_slice(),
        "Automatic OFF + Watercolor falloff must paint BYTE-IDENTICAL to Automatic ON"
    );

    // (b) Manual + a Shape image whose RIGHT half is blank → the wash silhouette goes asymmetric.
    let mut img_t = white_canvas(size, 8.0);
    wet(&mut img_t);
    img_t.paint.brush.watercolor_shape_auto = false;
    img_t.paint.brush.falloff = Falloff::Watercolor;
    let mut lum = vec![255u8; 16 * 16];
    for y in 0..16 {
        for x in 8..16 {
            lum[y * 16 + x] = 0; // right half of the tip: no coverage
        }
    }
    img_t.set_brush_shape_image(lum, 16, 16);
    for slot in &mut img_t.paint.brush_by_mode {
        *slot = img_t.paint.brush;
    }
    assert!(t_dab_paints_asymmetric(&mut img_t, size));
}

/// Helper for the image-silhouette assertion: one dab at the canvas centre; returns whether the
/// painted result differs left-vs-right of the centre column (the half-blank tip must show).
fn t_dab_paints_asymmetric(t: &mut PainterTool, size: u32) -> bool {
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let mut left_painted = 0u32;
    let mut right_painted = 0u32;
    for y in 24..40u32 {
        for dx in 1..8u32 {
            if px(t, size, 32 - dx, y) != [255, 255, 255, 255] {
                left_painted += 1;
            }
            if px(t, size, 32 + dx, y) != [255, 255, 255, 255] {
                right_painted += 1;
            }
        }
    }
    assert!(
        left_painted > 0,
        "the covered half of the tip painted (left {left_painted})"
    );
    assert!(
        left_painted > right_painted * 2,
        "the blank half must paint far less (left {left_painted} vs right {right_painted})"
    );
    true
}

/// **Manual Shape stamp honours Flatten + Rotate + grey-tip normalisation** (Enio 2026-07-07,
/// smoke round 2): (a) `dab_flatten` squeezes the watercolor footprint into an ellipse and
/// `dab_angle_deg` orients it — they flowed through `footprint_deform` on the plain dab but the
/// watercolor envelope used the raw round distance; (b) a GREY tip image must paint the same wash
/// as a WHITE one (the per-stroke max-luminance normaliser: coverage is wetness geometry that must
/// saturate — a raw grey tip starved the optics: pale centre, dead rim).
#[test]
fn watercolor_manual_shape_flatten_rotate_and_grey_tip_normalise() {
    fn wet_manual(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 12.0,
            hardness: 0.0,
            falloff: Falloff::Watercolor,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            watercolor_shape_auto: false,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            warp: 0.0, // no organic boundary noise — the extents measure the FOOTPRINT
            granulation: 0.0, // no mottle either
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
    }
    fn dab(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    }
    /// Painted extent along x and y through the centre row/column.
    fn extent(t: &PainterTool, size: u32) -> (u32, u32) {
        let (mut ex, mut ey) = (0u32, 0u32);
        for d in 0..16u32 {
            if px(t, size, 32 + d, 32) != [255, 255, 255, 255] {
                ex = d;
            }
            if px(t, size, 32, 32 + d) != [255, 255, 255, 255] {
                ey = d;
            }
        }
        (ex, ey)
    }
    let size = 64u32;

    // (a) Flatten 0.8: the footprint squeezes the minor (y) axis at angle 0…
    let mut t = white_canvas(size, 12.0);
    wet_manual(&mut t);
    t.paint.brush.dab_flatten = 0.8;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    dab(&mut t);
    let (ex, ey) = extent(&t, size);
    assert!(
        ex >= ey + 3,
        "Flatten must squeeze the watercolor footprint (x extent {ex} vs y {ey})"
    );
    // … and Rotate 90° swaps the axes.
    let mut t = white_canvas(size, 12.0);
    wet_manual(&mut t);
    t.paint.brush.dab_flatten = 0.8;
    t.paint.brush.dab_angle_deg = 90;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    dab(&mut t);
    let (ex, ey) = extent(&t, size);
    assert!(
        ey >= ex + 3,
        "Rotate 90° must re-orient the flattened footprint (x {ex} vs y {ey})"
    );

    // (b) Grey tip == white tip byte-for-byte (the normaliser rescales 128/255 → 1.0).
    let mut white_tip = white_canvas(size, 12.0);
    wet_manual(&mut white_tip);
    white_tip.set_brush_shape_image(vec![255u8; 16 * 16], 16, 16);
    for slot in &mut white_tip.paint.brush_by_mode {
        *slot = white_tip.paint.brush;
    }
    dab(&mut white_tip);
    let mut grey_tip = white_canvas(size, 12.0);
    wet_manual(&mut grey_tip);
    grey_tip.set_brush_shape_image(vec![128u8; 16 * 16], 16, 16);
    for slot in &mut grey_tip.paint.brush_by_mode {
        *slot = grey_tip.paint.brush;
    }
    dab(&mut grey_tip);
    assert_eq!(
        white_tip.canvas_rgba.as_slice(),
        grey_tip.canvas_rgba.as_slice(),
        "a uniformly grey tip must paint the SAME wash as a white one (normalised wetness)"
    );
}

/// **A TEXTURED tip keeps the typical watercolor** (Enio 2026-07-07: "não tem como o algoritmo que
/// faz a aquarela típica funcionar com textura no slot shape?"): the tip's texture must NOT hole the
/// wash — water fills the tip's outer silhouette (saturated coverage → body + rim at the OUTER
/// boundary) while the texture becomes pigment DENSITY within (`stroke_density` × the fill term).
/// A streaky tip therefore paints a fully-wet wash whose interior VARIES with the streaks instead of
/// showing white gaps.
#[test]
fn watercolor_textured_tip_keeps_typical_wash_with_density_variation() {
    let size = 64u32;
    let mut t = white_canvas(size, 12.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 0.0,
        falloff: Falloff::Watercolor,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        watercolor_shape_auto: false,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        warp: 0.0,        // measure the footprint, not the organic noise
        granulation: 0.0, // no mottle — the only interior variation is the tip density
        ..Default::default()
    };
    // Streaky tip: 4-px columns alternating white (255) / mid (100) — bristle-like texture.
    let mut lum = vec![255u8; 32 * 32];
    for y in 0..32 {
        for x in 0..32 {
            if (x / 4) % 2 == 1 {
                lum[y * 32 + x] = 100;
            }
        }
    }
    t.set_brush_shape_image(lum, 32, 32);
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));

    // (a) TYPICAL WASH: every pixel in the core (radius/2) is painted — no white holes from the
    //     mid-tone streaks (they are fully WET; only their pigment density differs).
    let mut holes = 0u32;
    for y in 27..38u32 {
        for x in 27..38u32 {
            if px(&t, size, x, y) == [255, 255, 255, 255] {
                holes += 1;
            }
        }
    }
    assert_eq!(
        holes, 0,
        "mid-tone streaks must stay WET (no white holes in the core)"
    );

    // (b) TEXTURE AS DENSITY: the streak pattern shows as intensity variation — the painted core's
    //     green channel is not uniform (min/max spread beyond rounding noise).
    let (mut lo, mut hi) = (255u8, 0u8);
    for y in 30..35u32 {
        for x in 27..38u32 {
            let g = px(&t, size, x, y)[1];
            lo = lo.min(g);
            hi = hi.max(g);
        }
    }
    assert!(
        hi - lo >= 8,
        "the tip texture must read as pigment-density variation (green spread {lo}..{hi})"
    );
}

/// **W-A (doc 12 OPT-1) — the subtractive-mixing discriminant: blue over yellow makes GREEN, not
/// grey.** A blue mixer brush (Charge 0.3) crossing a dry YELLOW pool must deposit a GREEN-dominant
/// mix at the pool's exit (absorbance-space lerp = pigment mixing). The sRGB lerp this replaced
/// deposited khaki/grey — R≈G, measured (128,128,115) pre-fix — the exact "blue and yellow make
/// gray" defect the Mixbox paper names as the flagship failure of RGB-mixing paint software.
#[test]
fn watercolor_wet_mix_blue_over_yellow_deposits_green() {
    let size = 160u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            let p = if (44..70).contains(&x) {
                [250u8, 220, 40, 255] // dry yellow pool
            } else {
                [255u8, 255, 255, 255]
            };
            src[i..i + 4].copy_from_slice(&p);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 7.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.10, 0.20, 0.70], // blue
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.3,
        wet_pull: 0.6,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([16.0, 80.0], PointerPhase::Down)));
    let mut x = 16.0f32;
    while x < 130.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([130.0, 80.0], PointerPhase::Up));
    // At the pool's exit edge the deposited colour must be GREEN-dominant (pigment mix of blue +
    // yellow), with a real margin — the grey failure mode had R ≈ G.
    // (x = 66/74 sit at the pool's trailing edge, where the carried mix peaks; farther out the
    // carry is already decaying toward the brush's blue and the G margin thins by design.)
    for probe_x in [66u32, 74] {
        let i = ((80 * size + probe_x) * 4) as usize;
        let c = &t.paint.stroke_color[i..i + 4];
        let (r, g, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
        assert!(
            g > r + 10 && g > b + 10,
            "blue over yellow must deposit GREEN at the pool exit (x={probe_x}): rgba=({r},{g},{b})"
        );
    }
    // Far downstream the carry decays and the deposit returns toward the BRUSH's blue.
    let i = ((80 * size + 100) * 4) as usize;
    let c = &t.paint.stroke_color[i..i + 4];
    assert!(
        c[2] > c[0],
        "downstream the carry decays back toward the blue brush: rgba={c:?}"
    );
}

/// **GRAN-1 (doc 12, Curtis §4.5): granulação é deposição nos VALES — vales escuros, picos claros.**
/// O sinal antigo era INVERTIDO (picos h altos escureciam) e o clamp furava o wash com speckle
/// branco em amount alto. Grain map metade preta (h=0, vales) / metade branca (h=1, picos),
/// granulation 1.0: o wash sobre os VALES deposita mais (mais escuro) que sobre os picos — e
/// nenhum texel do wash fica branco puro (sem speckle: o gate é limitado por γ < 1).
#[test]
fn watercolor_granulation_deposits_into_valleys_not_peaks() {
    let size = 64u32;
    let mut t = white_canvas(size, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0, // isolate the fill term (no rim)
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 1.0,
        granulation_use_paper: false, // the Grain slot IS the granulation map
        ..Default::default()
    };
    // Canvas-anchored granulation map: left half BLACK (valleys), right half WHITE (peaks).
    let mut lum = vec![255u8; 16 * 16];
    for y in 0..16 {
        for x in 0..8 {
            lum[y * 16 + x] = 0;
        }
    }
    t.set_brush_texture_image(lum, 16, 16);
    t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
    // Image sampling uses the dab-space convention (`u·0.5 + 0.5`): one tile unit = HALF the
    // image, so Size 8 makes the full 16-px image span the 64-px canvas — WHITE (peaks) lands on
    // the left half, BLACK (valleys) on the right (the u-wrap crosses the halves at x = 32).
    t.paint.brush.texture.size = [8.0, 8.0];
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down)));
    for i in 1..=20 {
        t.on_canvas_pointer(cp([12.0 + i as f32 * 2.0, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    // Mean green channel of the wash core on each half (row 32, away from the rim).
    let mean_g = |x0: u32, x1: u32| -> f32 {
        let mut acc = 0.0f32;
        for x in x0..x1 {
            acc += f32::from(px(&t, size, x, 32)[1]);
        }
        acc / (x1 - x0) as f32
    };
    let peaks = mean_g(16, 28); // white-map half (h = 1) — left under the dab-space wrap
    let valleys = mean_g(36, 48); // black-map half (h = 0) — right
    assert!(
        valleys + 8.0 < peaks,
        "valleys must deposit MORE pigment (darker) than peaks: valleys {valleys:.1} vs peaks {peaks:.1}"
    );
    // No white speckle: every core texel is painted (the old symmetric clamp punched h-low texels to 0
    // density... and at the new sign, γ < 1 bounds the peak side — nothing in the core stays white).
    for x in 16..48u32 {
        assert_ne!(
            px(&t, size, x, 32),
            [255, 255, 255, 255],
            "no white speckle inside the wash (x={x})"
        );
    }
}

/// **Settle take 3 está LIGADO (Enio 2026-07-08: "nem sei se está funcionando")**: o preview vivo
/// roda a ~80% do settle e o bake aplica 100% — então soltar a caneta CLAREIA os PICOS do tooth
/// (o pigmento termina de ceder pros vales) enquanto os VALES ficam praticamente iguais. Se live
/// e bake fossem idênticos (WYSIWYG) o delta seria 0; se o preview estivesse longe (take 1) o
/// delta seria um pop. Este teste pina o meio-termo: delta presente, pequeno e direcional.
#[test]
fn watercolor_granulation_bake_settles_beyond_the_live_preview() {
    let size = 64u32;
    let mut t = white_canvas(size, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 1.0,
        granulation_use_paper: false,
        wet_rewet: 0.0, // no water: live settle = GRAN_SETTLE_BASE exactly
        ..Default::default()
    };
    let mut lum = vec![255u8; 16 * 16];
    for y in 0..16 {
        for x in 0..8 {
            lum[y * 16 + x] = 0;
        }
    }
    t.set_brush_texture_image(lum, 16, 16);
    t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
    t.paint.brush.texture.size = [8.0, 8.0];
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down)));
    for i in 1..=20 {
        t.on_canvas_pointer(cp([12.0 + i as f32 * 2.0, 32.0], PointerPhase::Move));
    }
    // LIVE snapshot (last composite before release; the Up lands at the same position, so the
    // coverage is already saturated — the only delta left is the settle).
    let live: Vec<u8> = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    let mean_g = |buf: &[u8], x0: u32, x1: u32| -> f32 {
        let mut acc = 0.0f32;
        for x in x0..x1 {
            acc += f32::from(buf[((32 * size + x) * 4 + 1) as usize]);
        }
        acc / (x1 - x0) as f32
    };
    // PEAKS (white-map half, left under the dab-space wrap): the bake sheds MORE pigment → lighter.
    let (peaks_live, peaks_baked) = (mean_g(&live, 16, 28), mean_g(&t.canvas_rgba, 16, 28));
    assert!(
        peaks_baked > peaks_live + 2.0,
        "the bake must settle beyond the live preview on the PEAKS (live {peaks_live:.1} → baked {peaks_baked:.1})"
    );
    // Upper bound = the physics ceiling at FULL amount (gate 0.28 → 0.10 ⇒ ~50 bytes here);
    // at the default Granulation 0.3 the felt delta is ~⅓ of this (Enio's smoke: "preview
    // próximo do bake"). The bound guards against a runaway (e.g. live base accidentally 0).
    assert!(
        peaks_baked - peaks_live < 80.0,
        "…bounded set, not a runaway pop (live {peaks_live:.1} → baked {peaks_baked:.1})"
    );
    // VALLEYS (black-map half): full deposit in both → essentially unchanged by the release.
    let (val_live, val_baked) = (mean_g(&live, 36, 48), mean_g(&t.canvas_rgba, 36, 48));
    assert!(
        (val_baked - val_live).abs() < 3.0,
        "valleys keep their deposit across the release (live {val_live:.1} → baked {val_baked:.1})"
    );
}

/// **MIX-1 (doc 12, W-C): Charge DEPLETA com a distância do traço** — a assinatura nº 1 do
/// Procreate (Handbook: "the longer you drag your stroke out... the trail of color it leaves will
/// become fainter"). Um traço LONGO em canvas branco com Charge baixo (reserva curta, nada a
/// captar no branco) precisa desbotar: a cauda deposita menos que a cabeça. Charge = 1 (default)
/// pula o mixer inteiro — byte-idêntico, coberto pela suíte.
#[test]
fn watercolor_wet_mix_charge_depletes_along_the_stroke() {
    let size = 256u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        wet_charge: 0.25, // short reserve; white canvas ⇒ nothing to pick up ⇒ pure depletion
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([12.0, 128.0], PointerPhase::Down)));
    let mut x = 12.0f32;
    while x < 240.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 128.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([240.0, 128.0], PointerPhase::Up));
    let mean_g = |x0: u32, x1: u32| -> f32 {
        let mut acc = 0.0f32;
        for x in x0..x1 {
            acc += f32::from(px(&t, size, x, 128)[1]);
        }
        acc / (x1 - x0) as f32
    };
    let head = mean_g(20, 60); // fresh reserve
    let tail = mean_g(190, 230); // depleted
    assert!(
        tail > head + 15.0,
        "the trail must fade as the Charge depletes (head G {head:.1} vs tail G {tail:.1} — lighter = fainter)"
    );
}

/// **MIX-1 regressão (Enio smoke 2026-07-08):** um pincel ESGOTADO cruzando uma poça deposita
/// proporcional às DUAS intensidades — (a) poça PÁLIDA ⇒ quase nada ("explode em muito pigmento"
/// era o `depl = max(fresh, t)` com `t` = peso de mistura, que salta pra ~1 em qualquer poça);
/// (b) poça RICA ⇒ o smudge continua vivo (o fix não pode matar o carry). E (c) a CABEÇA de um
/// traço com Charge baixo mantém a anatomia completa (reserva começa em 1.0 — Charge controla a
/// duração, nunca a intensidade inicial: escalar a cobertura inundava o interior com edge residual).
#[test]
fn watercolor_wet_mix_depleted_brush_respects_pool_intensity() {
    let size = 256u32;
    let spec = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    let stroke_v = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 12.0], PointerPhase::Down)));
        let mut y = 12.0f32;
        while y < 240.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 240.0], PointerPhase::Up));
    };
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = spec;
    // Pale pool at x∈[40..110], rich pool at x∈[150..220], both horizontal at y = 200 — far enough
    // down that a vertical Charge-0.1 stroke (span ≈ 107 px) arrives fully depleted (travel ≈ 188).
    t.paint.brush.color = [1.0, 0.78, 0.78]; // pale pink: little pigment to pick up
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([40.0, 200.0], PointerPhase::Down)));
    let mut x = 40.0f32;
    while x < 110.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 200.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([110.0, 200.0], PointerPhase::Up));
    t.paint.brush.color = [0.75, 0.05, 0.05]; // rich red: a real reservoir to smudge from
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([150.0, 200.0], PointerPhase::Down)));
    let mut x = 150.0f32;
    while x < 220.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 200.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([220.0, 200.0], PointerPhase::Up));
    // Two depleted blue crossings (Charge 0.1), one through each pool.
    t.paint.brush.color = [0.15, 0.25, 0.7];
    t.paint.brush.wet_charge = 0.1;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    stroke_v(&mut t, 75.0); // through the PALE pool
    stroke_v(&mut t, 185.0); // through the RICH pool
    let g = |x: u32, y: u32| f32::from(px(&t, size, x, y)[1]);
    let head = g(75, 16); // (c) stroke head: full fresh reserve — a strong mark
    assert!(
        head < 140.0,
        "the head of a low-Charge stroke must open at FULL reserve (G {head:.1} — dark = strong)"
    );
    let bare_trail = g(75, 160); // depleted, outside any pool: ~plain water
    let pale_cross = g(75, 200); // (a) depleted × pale pool
    let pale_pool = g(55, 200); // the pale pool away from the crossing
    assert!(
        pale_cross > pale_pool - 45.0,
        "a depleted brush over a PALE pool must not explode with pigment (pool G {pale_pool:.1} → crossing G {pale_cross:.1})"
    );
    assert!(
        pale_cross > head + 20.0,
        "…and deposits far less than the brush's own fresh head (head G {head:.1}, crossing G {pale_cross:.1})"
    );
    let rich_cross = g(185, 200); // (b) depleted × rich pool
    let below_rich = g(185, 226); // just past the rich pool: the carried smudge trails out
    assert!(
        below_rich < bare_trail - 8.0,
        "crossing a RICH pool must re-ink the depleted brush (trail after pool G {below_rich:.1} vs bare trail G {bare_trail:.1})"
    );
    assert!(
        rich_cross < pale_cross,
        "the smudge tracks the pool's intensity (rich crossing G {rich_cross:.1} < pale crossing G {pale_cross:.1})"
    );
}

/// **EDGE-1 (doc 12, W-C): washes que se encostam MOLHADOS fundem — sem contorno duplo.** Dentro
/// da janela de secagem (~8,5 s, DiVerdi) o segundo traço CONTINUA a sessão molhada: os buffers
/// acumulam a UNIÃO e o bake re-renderiza tudo sobre a base da sessão — um wash só, um rim só
/// (o rim interno do primeiro traço DERRETE no re-bake, e o traço novo não desenha rim sobre o
/// vizinho — Curtis §3-4). Depois de seco, o mesmo gesto volta a empilhar rim por cima (glazing)
/// e o mapa seco é DROPADO junto com a sessão (fast path de volta, sem custo ocioso).
#[test]
fn watercolor_touching_wet_washes_merge_without_double_rim() {
    let run = |dry_first: bool| -> f32 {
        let size = 192u32;
        let mut t = white_canvas(size, 8.0);
        t.paint.brush = BrushSpec {
            radius_px: 12.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.12,
            depth: 1.0,
            edge_gain: 2.5,
            edge_spread: 6.0,
            warp: 0.0,
            granulation: 0.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        let stroke_v = |t: &mut PainterTool, x: f32| {
            assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
            let mut y = 30.0f32;
            while y < 160.0 {
                y += 2.0;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x, 160.0], PointerPhase::Up));
        };
        stroke_v(&mut t, 80.0); // wash A (band x ≈ 68..92) — pours moisture at its bake
        assert!(
            !t.paint.canvas_wet.is_empty(),
            "the bake must pour the wash into the persistent wet map"
        );
        if dry_first {
            for _ in 0..140 {
                t.paint_tick(0.5); // 70 s of heartbeat — way past the ~60 s drying window
            }
            assert!(
                t.paint.canvas_wet.is_empty(),
                "a fully-dried wet map must be dropped (composite fast path back)"
            );
        }
        stroke_v(&mut t, 88.0); // wash B overlaps A deeply (union band ≈ 68..100)
        // Junction probes: B's would-be LEFT rim (x 78-81, deep in A's interior) + A's OWN right
        // rim position (x 88-91). Merged (wet): both re-render as union INTERIOR — light. Dried
        // first: B lays a rim over A AND A's baked rim persists under B — both bands dark.
        let mut acc = 0.0f32;
        for x in [78u32, 79, 80, 81, 88, 89, 90, 91] {
            for y in 80..110u32 {
                acc += f32::from(px(&t, size, x, y)[1]);
            }
        }
        acc / (8.0 * 30.0)
    };
    let wet_junction = run(false);
    let dry_junction = run(true);
    assert!(
        wet_junction > dry_junction + 40.0,
        "wet washes must MERGE into one rim-less junction — both the new stroke's rim over the \
         neighbour and the neighbour's old inner rim must be gone (wet G {wet_junction:.1} vs \
         dried-first double-contour G {dry_junction:.1})"
    );
}

/// **EDGE-1 #3 (Enio smoke 2026-07-11) — "Wet the layer" (Rebelle):** o botão **Wet** re-molha o canvas e
/// reabre uma sessão molhada sobre a tinta EXISTENTE, com um rewet FORÇADO — então um traço de água clara
/// (Rewet do pincel = 0) feito depois LEVANTA a tinta seca (clareia rumo ao papel), coisa que NÃO acontece
/// sem apertar Wet (água clara sobre papel seco não reativa nada). Propriedade: mesmo A seco + mesmo B de
/// água, o núcleo de A fica mais claro COM Wet que SEM. DIRETIVA §4 (o forced-rewet é o discriminador).
#[test]
fn watercolor_wet_button_reactivates_dry_paint() {
    let size = 96u32;
    fn dry_wash_a(size: u32) -> PainterTool {
        let mut t = white_canvas(size, 8.0);
        t.paint.brush = BrushSpec {
            radius_px: 16.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.8, 0.1, 0.1], // a solid dark-red wash to reactivate
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 1.5,
            edge_gain: 0.0,
            warp: 0.0,
            granulation: 0.0,
            wet_rewet: 0.0, // the brush itself does NOT rewet — only the Wet button will
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        // Wash A — a vertical band at x = 48.
        assert!(t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Down)));
        let mut y = 20.0f32;
        while y < 76.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([48.0, 76.0], PointerPhase::Up));
        // Dry it FULLY (past the ~10 s window) so the session tears down — A is dry paint now.
        for _ in 0..60 {
            t.paint_tick(0.5);
        }
        assert!(
            t.paint.canvas_wet.is_empty(),
            "A must be fully dry before the test"
        );
        t
    }
    // A clear-water stroke over A (same brush, near-white colour, low fill → the deposit is negligible,
    // any change is the LIFT). `press_wet` toggles the Wet button before painting.
    let clear_water_over_a = |press_wet: bool| -> [u8; 4] {
        let mut t = dry_wash_a(size);
        t.paint.brush.color = [0.98, 0.98, 0.98];
        t.paint.brush.fill = 0.1;
        t.paint.brush.opacity = 0.0; // no body: pure transparent water, so any change is the LIFT only
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        if press_wet {
            t.wet_canvas_now();
        }
        assert!(t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Down)));
        let mut y = 20.0f32;
        while y < 76.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([48.0, 76.0], PointerPhase::Up));
        px(&t, size, 48, 48)
    };
    let lum = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    let with_wet = clear_water_over_a(true);
    let without_wet = clear_water_over_a(false);
    assert!(
        lum(with_wet) > lum(without_wet) + 30,
        "the Wet button must reactivate the dry wash (a clear-water stroke lifts it lighter): \
         with Wet {with_wet:?} (lum {}) vs without {without_wet:?} (lum {})",
        lum(with_wet),
        lum(without_wet),
    );
}

/// **EDGE-1 #4 (Enio smoke 2026-07-11):** cada traço seca no SEU próprio relógio — um segundo traço
/// (longe do primeiro) NÃO pode re-molhar o wash anterior. `stroke_coverage` é a UNIÃO da sessão; despejar
/// isso na moisture pela rect cumulativa re-molhava TUDO a 255 no bake de cada traço (resetando a secagem
/// dos anteriores). O pour agora usa só a footprint do traço atual. Propriedade: seco parcialmente o A,
/// pinto B longe, e a umidade de A no seu núcleo NÃO sobe (sem o fix ela voltaria a 255). DIRETIVA §4.
#[test]
fn watercolor_second_stroke_does_not_reset_the_first_strokes_drying() {
    let size = 160u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.2,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    let stroke_v = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
        let mut y = 30.0f32;
        while y < 130.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 130.0], PointerPhase::Up));
    };
    let wet_at = |t: &PainterTool, x: usize, y: usize| t.paint.canvas_wet[y * size as usize + x];

    stroke_v(&mut t, 40.0); // wash A (band x ≈ 28..52) — pours moisture at its bake
    assert!(
        !t.paint.canvas_wet.is_empty(),
        "A's bake must pour moisture"
    );
    // Dry A partially: ~4 s of heartbeat (rate 25.5 ⇒ ~102 bytes off 255, still comfortably wet).
    for _ in 0..8 {
        t.paint_tick(0.5);
    }
    let a_before = wet_at(&t, 40, 80);
    assert!(
        a_before > 0 && a_before < 220,
        "A must have partially dried before B (got {a_before})"
    );

    stroke_v(&mut t, 120.0); // wash B (band x ≈ 108..132) — FAR from A, no overlap
    let a_after = wet_at(&t, 40, 80);
    let b_fresh = wet_at(&t, 120, 80);
    assert!(
        u32::from(b_fresh) > u32::from(a_after) + 40,
        "B is freshly wet, A stayed drier: B {b_fresh} vs A {a_after}"
    );
    assert!(
        a_after <= a_before,
        "painting B must NOT re-wet A's core (its own drying clock): A was {a_before}, became {a_after}"
    );
}

/// **#4b (Enio smoke 2026-07-11, "retângulo gigante na união"):** um traço cujo BBOX apenas CONTÉM um wash
/// anterior (sem a footprint cobri-lo) NÃO pode re-molhar esse wash. `stroke_coverage` é a UNIÃO; o pour
/// despejava-a sobre o RECT do traço → re-molhava os pixels do vizinho DENTRO do rect a 255 (um retângulo
/// de umidade que o overlay pintava). Fix: o pour restringe à footprint DONA (`owner == cur_o`). Sonda: um
/// pixel de A coberto por A, DENTRO do bbox do diagonal B, mas FORA da footprint de B — não re-molha.
/// **Undo apaga a umidade (Enio smoke 2026-07-11):** desfazer um traço de aquarela tem que limpar o mapa
/// de umidade — o canvas voltou, mas o overlay continuava mostrando o damp do traço desfeito.
#[test]
fn watercolor_undo_clears_the_moisture() {
    let size = 64u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.3, 0.9],
        space_attenuation: false,
        watercolor: true,
        fill: 0.4,
        depth: 1.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([32.0, 20.0], PointerPhase::Down)));
    let mut y = 20.0f32;
    while y < 44.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([32.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([32.0, 44.0], PointerPhase::Up));
    assert!(
        t.paint.canvas_wet.iter().any(|&w| w > 0),
        "the stroke must have poured moisture"
    );
    assert!(t.undo_last(), "the stroke is undoable");
    assert!(
        t.paint.canvas_wet.is_empty(),
        "undo must clear the wet session's moisture map (no stale damp overlay)"
    );
}

#[test]
fn watercolor_overlapping_bbox_does_not_rewet_the_neighbour_wash() {
    let size = 80u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.3, 0.9],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    let wet_at = |t: &PainterTool, x: usize, y: usize| t.paint.canvas_wet[y * size as usize + x];
    // A: horizontal band at y = 30 (x ≈ 12..68). Bake, then dry a little.
    assert!(t.on_canvas_pointer(cp([20.0, 30.0], PointerPhase::Down)));
    let mut x = 20.0f32;
    while x < 60.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Up));
    for _ in 0..6 {
        t.paint_tick(0.5);
    }
    let a_probe_before = wet_at(&t, 25, 30); // on A, will fall inside B's bbox but NOT B's footprint
    assert!(
        a_probe_before > 0 && a_probe_before < 230,
        "A's probe must be wet-but-decayed before B ({a_probe_before})"
    );
    // B: DIAGONAL from (20,60) to (60,20) — its bbox (20..60, 20..60) CONTAINS A, but at x=25 B sits at
    // y≈55, so (25,30) is inside B's bbox yet OUTSIDE B's footprint. Same wet session (A still wet).
    assert!(t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Down)));
    let mut s = 0.0f32;
    while s < 40.0 {
        s += 2.0;
        t.on_canvas_pointer(cp([20.0 + s, 60.0 - s], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 20.0], PointerPhase::Up));
    let a_probe_after = wet_at(&t, 25, 30);
    assert!(
        a_probe_after <= a_probe_before,
        "B's bbox merely CONTAINS A here — its footprint doesn't reach (25,30), so it must NOT re-wet it: \
         was {a_probe_before}, became {a_probe_after}"
    );
}

/// **#18 (Enio smoke 2026-07-11):** mudar params de Wash (Body/Concentration/Edge/Opacity) entre traços e
/// cruzar um traço úmido não pode imprimir borda dura na junção — os params por-dono degrauavam na fronteira
/// de posse (Bug #8 lição #4). O campo suavizado (`build_style_field`) espalha os params na fronteira.
#[test]
fn watercolor_param_change_junction_is_soft() {
    let size = 96u32;
    let mut t = white_canvas(size, 8.0);
    // A: vertical band x=48, params X (Body alto, Concentration alta) → renderiza ESCURO.
    t.paint.brush = BrushSpec {
        radius_px: 9.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.3, 0.9],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([48.0, 15.0], PointerPhase::Down)));
    let mut y = 15.0;
    while y < 80.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([48.0, 80.0], PointerPhase::Up));
    // B: horizontal band y=48 crossing A, params Y (Body baixo) → CLARO. Mesma sessão úmida.
    t.paint.brush.fill = 0.12;
    t.paint.brush.depth = 1.0;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([15.0, 48.0], PointerPhase::Down)));
    let mut x = 15.0;
    while x < 80.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 48.0], PointerPhase::Up));
    // Max |grad| do verde ao longo de x=48, cruzando a fronteira A(escuro)/B(claro) em y~40. Sem o campo
    // suavizado os params degrauam ali (medido 118 bytes/px); com o campo, ~13.
    let g = |yy: u32| f32::from(px(&t, size, 48, yy)[1]);
    let mut maxg = 0.0f32;
    for yy in 33..47u32 {
        maxg = maxg.max((g(yy + 1) - g(yy)).abs());
    }
    assert!(
        maxg < 22.0,
        "a junção com params trocados deve ser SUAVE, não degrau (grad {maxg} bytes/px)"
    );
}

/// **#2 (Enio smoke 2026-07-11):** a umidade é lançada AO VIVO durante o traço — o damp aparece enquanto
/// pinta, não só no mouse-up ("a umidade só aparece no mouse up. isso é muito feio"). Sem SOLTAR, o mapa de
/// umidade já tem que estar populado sobre a região pintada (o pour antes rodava só no bake).
#[test]
fn watercolor_wetness_is_laid_live_during_the_stroke() {
    let size = 96u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([48.0, 30.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([48.0, 50.0], PointerPhase::Move));
    // NO pen-up yet — the moisture must already be present (live pour), not empty until the bake.
    assert!(
        t.paint.canvas_wet.iter().any(|&w| w > 0),
        "the wetness must be laid LIVE during the stroke (not only at pen-up)"
    );
}

/// **#3a/#12b (Enio smoke 2026-07-11):** o papel seca das BORDAS para o CENTRO — a poça molhada recede pelo
/// perímetro, não uniformemente. Pinto uma banda sólida (interior despeja umidade plana), seco PARCIALMENTE,
/// e o núcleo profundo continua mais úmido que a borda (a secagem uniforme esvaziaria os dois juntos).
#[test]
fn watercolor_drying_recedes_from_the_edges() {
    let size = 96u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 18.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 1.0,
        edge_gain: 0.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // A solid vertical band centred on x = 48 (radius 18 ⇒ x ≈ 30..66).
    assert!(t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Down)));
    let mut y = 20.0f32;
    while y < 76.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([48.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([48.0, 76.0], PointerPhase::Up));
    let wet_at = |t: &PainterTool, x: usize| t.paint.canvas_wet[48 * size as usize + x];
    let center0 = wet_at(&t, 48);
    assert!(
        center0 > 200,
        "the band interior pours a high flat moisture ({center0})"
    );
    // Dry partially (the erosion recedes the perimeter faster than the flat interior decay).
    for _ in 0..12 {
        t.paint_tick(0.5);
    }
    let center = wet_at(&t, 48);
    // x = 34 is INTERIOR (coverage ~1 ⇒ it poured the same flat moisture as the centre): under a UNIFORM
    // decay it would equal the centre, but the edges-to-centre recession has eaten into it (front near x=34).
    let edge = wet_at(&t, 34);
    assert!(center > 0, "the deep centre must still be wet ({center})");
    assert!(
        u32::from(edge) + 40 < u32::from(center),
        "the recession must eat an INTERIOR pixel ahead of the deep centre (edge {edge} vs centre {center})"
    );
}

/// **EDGE-1 regressão (Enio smoke 2026-07-09, "traços duplicados"):** a janela de secagem vencendo
/// NO MEIO de um traço aberto não pode duplicar o wash. O teardown antigo derrubava a base da
/// sessão no tick (mapa zerado) mas deixava os buffers da união vivos (o traço aberto é dono
/// deles) — o bake do pen-up caía no fallback per-stroke (que JÁ contém a união assada) e
/// re-renderizava tudo por cima: dupla contagem, o conjunto escurecia de vez. O teardown agora é
/// ATÔMICO e adiado até não haver traço aberto.
#[test]
fn watercolor_session_drying_mid_stroke_does_not_double_bake() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.12,
        depth: 1.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A (vertical band at x = 60), baked + poured into the session.
    assert!(t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
    let a_interior_before = f32::from(px(&t, size, 60, 95)[1]);
    // Stroke C opens while the paper is still wet (session continues), then the drying deadline
    // fires MID-STROKE (one big heartbeat zeroes the map), then C finishes far from A.
    assert!(t.on_canvas_pointer(cp([140.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 90.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([140.0, y], PointerPhase::Move));
    }
    t.paint_tick(70.0); // way past the ~60 s window — the map zeroes with the stroke OPEN
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([140.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([140.0, 160.0], PointerPhase::Up));
    // A's interior (far from C) must be untouched by C's bake — the double-count re-rendered the
    // whole union over its own baked pixels and darkened it hard.
    let a_interior_after = f32::from(px(&t, size, 60, 95)[1]);
    assert!(
        (a_interior_after - a_interior_before).abs() <= 2.0,
        "the drying deadline mid-stroke must not re-bake (duplicate) the neighbour wash \
         (A interior G {a_interior_before:.1} -> {a_interior_after:.1})"
    );
}

/// **Sessão molhada — params POR TRAÇO (doc 13 topo, Enio 2026-07-09):** traço 1 com Concentration
/// (depth) alta + traço 2 com baixa na MESMA sessão — o re-bake da união resolvia os params
/// CORRENTES do brush pro conjunto ("no mouse up o primeiro traço é convertido para 0.3"), e
/// qualquer mudança propagava pelas poças na janela retangular do composite. Com a tabela de
/// estilos + mapa de dono, cada wash mantém o SEU caráter byte-exato; o traço novo usa o dele.
#[test]
fn watercolor_session_keeps_each_strokes_style() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 2.0, // Concentration ALTA no traço 1
        edge_gain: 1.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    let stroke_v = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
        let mut y = 30.0f32;
        while y < 160.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 160.0], PointerPhase::Up));
    };
    stroke_v(&mut t, 60.0); // wash A (banda x ≈ 48..72), baked
    let a_probe: Vec<[u8; 4]> = (80..110u32).map(|y| px(&t, size, 60, y)).collect();
    // Concentration BAIXA no traço 2 — mesma sessão molhada (imediato).
    t.paint.brush.depth = 0.6;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    stroke_v(&mut t, 140.0); // wash B, longe do probe de A mas na mesma união/sessão
    let a_after: Vec<[u8; 4]> = (80..110u32).map(|y| px(&t, size, 60, y)).collect();
    assert_eq!(
        a_probe, a_after,
        "o wash 1 deve manter SUA Concentration byte-exata após o re-bake da união"
    );
    // E o wash 2 usa a Concentration DELE (bem mais claro que o 1).
    let g = |x: u32| f32::from(px(&t, size, x, 95)[1]);
    assert!(
        g(140) > g(60) + 30.0,
        "o wash 2 rende com a própria Concentration baixa (B G {:.0} vs A G {:.0})",
        g(140),
        g(60)
    );
}

/// **EDGE-2 (doc 12, W-C): backrun/bloom de ÁGUA LIMPA** — o gesto canônico era inalcançável por
/// construção (Dilution 1 → flow 0 → cobertura 0 → `cw ≤ 0` pulava TODO o caminho). Agora a água
/// carregada poura o canal próprio (`stroke_water`) e o composite a trata como superfície viva:
/// sobre um wash assentado, o interior do pool CLAREIA (lift — "whitened wake") e o pigmento
/// empurrado escurece o CONTORNO serrilhado (anel `água − halo`, Curtis §2.2 "severely darkened
/// edges"). Água em papel branco = nada (lift/dissolve/anel ∝ presença de tinta).
#[test]
fn watercolor_clean_water_backrun_blooms_on_wet_wash() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.35,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 6.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.0, // Rewet OFF — a ÁGUA sozinha tem que produzir o bloom
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash vertical (banda x ≈ 46..74), assentado (seca a sessão).
    assert!(t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
    for _ in 0..140 {
        t.paint_tick(0.5);
    }
    assert!(t.paint.canvas_wet.is_empty(), "sessão do wash deve secar");
    let wash_before = f32::from(px(&t, size, 60, 95)[1]);
    // Gota de ÁGUA PURA (Dilution 1) parada dentro do wash + um rabisco em papel branco.
    // Raio 30 ≫ o blur do halo (12 px): o interior fica livre do casco (raw ≈ halo) e o anel
    // forma no contorno — numa gota pequena o casco cobre o centro (bloom todo-anel, físico).
    t.paint.brush.radius_px = 30.0;
    t.paint.brush.wet_dilution = 1.0;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([60.0, 95.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([61.0, 95.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 95.0], PointerPhase::Up));
    assert!(t.on_canvas_pointer(cp([150.0, 40.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([151.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([150.0, 40.0], PointerPhase::Up));
    // (a) interior do pool clareou (pigmento empurrado — whitened wake).
    let pool_after = f32::from(px(&t, size, 60, 95)[1]);
    assert!(
        pool_after > wash_before + 10.0,
        "água limpa deve CLAREAR o interior do pool (antes G {wash_before:.0} → depois G {pool_after:.0})"
    );
    // (b) o contorno do pool escureceu em algum ponto do anel (raio ~10-20 px, serrilhado).
    let mut ring_min = 255.0f32;
    for r in 22..=40u32 {
        for &(dx, dy) in &[
            (r as i32, 0),
            (-(r as i32), 0),
            (0, r as i32),
            (0, -(r as i32)),
        ] {
            let (x, y) = ((60 + dx) as u32, (95 + dy) as u32);
            if x < size && y < size {
                ring_min = ring_min.min(f32::from(px(&t, size, x, y)[1]));
            }
        }
    }
    assert!(
        ring_min < wash_before - 8.0,
        "o pigmento empurrado deve ESCURECER o contorno do pool (wash G {wash_before:.0}, anel mín G {ring_min:.0})"
    );
    // (c) água em papel branco = invisível.
    let blank = px(&t, size, 150, 40);
    assert_eq!(
        &blank[..3],
        &[255, 255, 255],
        "água pura sobre papel em branco não deposita nada"
    );
}

/// **EDGE-3 (doc 12, W-C): rim ASSINADO com conservação (Curtis §4.3.3)** — o pigmento que
/// escurece a borda MIGROU do interior/franja; o lobo negativo do unsharp (antes clampado fora)
/// EMPALIDECE a franja. Propriedade refutável: com Edge > 0, o rim escurece E a franja fica MAIS
/// CLARA que a mesma franja com Edge = 0 (na fórmula aditiva antiga, Edge > 0 só podia escurecer
/// ou manter QUALQUER pixel — nunca clarear).
#[test]
fn watercolor_signed_rim_pales_the_fringe() {
    let run = |gain: f32| -> PainterTool {
        let size = 160u32;
        let mut t = white_canvas(size, 8.0);
        t.paint.brush = BrushSpec {
            radius_px: 16.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            depth: 2.0,
            edge_gain: gain,
            edge_spread: 6.0,
            warp: 0.0,
            granulation: 0.0,
            opacity: 0.0, // isolate the signed rim from the body/opacity film (its own test)
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
        let mut y = 30.0f32;
        while y < 130.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([80.0, 130.0], PointerPhase::Up));
        t
    };
    let plain = run(0.0);
    let rimmed = run(3.0);
    let size = 160u32;
    let g = |t: &PainterTool, x: u32| -> f32 {
        let mut a = 0.0f32;
        for y in 60..100u32 {
            a += f32::from(px(t, size, x, y)[1]);
        }
        a / 40.0
    };
    // Interior profundo (x = 84): o tom NÃO desloca com Edge (o resíduo cru-vs-endurecido do
    // inner deslocava o wash inteiro — a reclamação literal da auditoria).
    assert!(
        (g(&rimmed, 84) - g(&plain, 84)).abs() <= 2.0,
        "Edge não pode deslocar o tom do interior (plain G {:.0} vs rimmed G {:.0})",
        g(&plain, 84),
        g(&rimmed, 84)
    );
    // Rim (justo dentro da silhueta, x ≈ 90): mais escuro com Edge.
    assert!(
        g(&rimmed, 90) < g(&plain, 90) - 8.0,
        "o rim deve escurecer com Edge (plain G {:.0} vs rimmed G {:.0})",
        g(&plain, 90),
        g(&rimmed, 90)
    );
    // Franja (onde inner > cw, x ≈ 94): mais CLARA com Edge — o lobo negativo (conservação).
    let (fp, fr) = (g(&plain, 94), g(&rimmed, 94));
    assert!(
        fr > fp + 4.0,
        "a franja deve EMPALIDECER com Edge — pigmento migrou pro rim (plain G {fp:.0} vs rimmed G {fr:.0})"
    );
}

/// **EDGE-4 (doc 12, W-C): o rim conta a história do gesto** — a amplitude deixa de ser uniforme:
/// onde o pincel DEMOROU (soak/dwell) o rim fortalece (`gain·(1 + k·soak)`), onde o depósito foi
/// tênue ele esmaece (`×(0.5 + 0.5·alpha)`). Propriedade: segurando o pincel parado num ponto do
/// traço (com Bleed > 0, o que poura dwell), o rim ADJACENTE ao dwell sai mais escuro que o rim
/// do resto do traço — mesma geometria, história diferente.
#[test]
fn watercolor_rim_strengthens_where_the_brush_dwelled() {
    let size = 160u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.15,
        depth: 1.0, // rim em MEIO-TOM: no escuro o Beer–Lambert comprime e o boost some
        edge_gain: 0.8, // abaixo do clamp do edge (≤1) — o boost do dwell precisa de headroom
        edge_spread: 6.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.5, // Bleed on — the dwell pours soak
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
    }
    // PARK at (80, 120) — the heartbeat pours dwell under the nib.
    for _ in 0..30 {
        t.paint_tick(0.1); // 3 s parked
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));
    // Rim column (x ≈ 88, just inside the silhouette): dwell zone (y ≈ 118-124) vs plain zone
    // (y ≈ 55-75, far from both the head and the dwell).
    let rim = |y0: u32, y1: u32| -> f32 {
        let mut a = 0.0f32;
        for y in y0..y1 {
            a += f32::from(px(&t, size, 88, y)[1]);
        }
        a / (y1 - y0) as f32
    };
    let (plain, dwelled) = (rim(55, 75), rim(116, 126));
    assert!(
        dwelled < plain - 6.0,
        "o rim deve FORTALECER onde o pincel demorou (rim plain G {plain:.0} vs dwell G {dwelled:.0})"
    );
}

/// **W-C reprodutibilidade (Enio smoke 2026-07-09, "área retangular clareia a poça vizinha"):**
/// o composite é função PURA do estado da sessão — re-renderizar a janela viva do traço 2 sobre
/// pixels JÁ ASSADOS do traço 1 reproduz o bake byte-exato. Antes: os campos de rewet liam o
/// base per-stroke (que contém a poça 1 assada) → dissolve/pool/mix "re-molhavam" a vizinha só
/// por ela cair na janela; o settle da granulação seguia a flag `commit` do frame; o soak (dwell)
/// zerava a cada pen-down. O probe fica FORA da zona de fusão legítima (EDGE-1 derrete rims que
/// se aproximam dentro do raio do blur).
#[test]
fn watercolor_session_rerender_reproduces_the_bake_byte_exact() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.5, // cobre o vazamento do settle (fonte = Same as Paper, default)
        wet_rewet: 0.5,   // Bleed on — arma o caminho de rewet + o dwell
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A (banda x ≈ 48..72) com DWELL no fim — o soak boosta o rim do bake (EDGE-4).
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    for _ in 0..20 {
        t.paint_tick(0.15); // 3 s parado — poura dwell em (60, 120)
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..140u32 {
            for x in 40..=66u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Traço B na MESMA sessão, perto mas sem encostar (gap ≥ 4 px de cobertura): a janela viva
    // dele cobre a borda direita de A, mas a cobertura não — A não pode mudar UM byte.
    assert!(t.on_canvas_pointer(cp([88.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([88.0, y], PointerPhase::Move));
    }
    assert_eq!(
        probe(&t),
        baked,
        "o re-render VIVO da janela de B não pode alterar a poça assada de A"
    );
    t.on_canvas_pointer(cp([88.0, 120.0], PointerPhase::Up));
    assert_eq!(
        probe(&t),
        baked,
        "o re-bake da união no pen-up de B deve reproduzir A byte-exato"
    );
    // Sanidade: B realmente pintou (o teste não passa por janela vazia).
    assert!(px(&t, size, 88, 80)[1] < 250, "B pintou de verdade");
}

/// **W-C deriva de params (doc 13, "qualquer mudança no brush propaga pelas poças"):** trocar o
/// brush entre traços da sessão (Size/Spread/cor/…) não pode re-renderizar a poça assada com a
/// GEOMETRIA do brush novo — `core_r` (raio do blur do rim) e `spread_thin` agora são por-DONO
/// na tabela de estilos, como os params de aparência (#1); o raio dos campos usa o máximo da
/// sessão (inerte em canvas virgem, onde os campos são zero).
#[test]
fn watercolor_session_brush_changes_do_not_touch_baked_washes() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.5,
        wet_rewet: 0.5,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A (banda x ≈ 48..72), assado.
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..140u32 {
            for x in 40..=72u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Brush RADICALMENTE diferente pro traço B (mesma sessão): size 24, Spread 24, corpo raso,
    // Concentration alta, azul. Nada disso pode tocar os bytes assados de A.
    t.paint.brush.radius_px = 24.0;
    t.paint.brush.edge_spread = 24.0;
    t.paint.brush.fill = 0.1;
    t.paint.brush.depth = 3.0;
    t.paint.brush.edge_gain = 0.3;
    t.paint.brush.wet_rewet = 0.2;
    t.paint.brush.granulation = 0.0;
    t.paint.brush.color = [0.1, 0.2, 0.9];
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([106.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([106.0, y], PointerPhase::Move));
    }
    assert_eq!(
        probe(&t),
        baked,
        "a janela viva do brush trocado não pode re-estilizar/re-blurar a poça assada de A"
    );
    t.on_canvas_pointer(cp([106.0, 120.0], PointerPhase::Up));
    assert_eq!(
        probe(&t),
        baked,
        "o re-bake da união com o brush trocado deve reproduzir A byte-exato"
    );
    assert!(px(&t, size, 106, 80)[0] < 250, "B (azul) pintou de verdade");
}

/// #13 (doc 14, smoke Enio 2026-07-10): mudar o SUBSTRATO (paper kind / Same-as-Paper / grain)
/// entre traços da MESMA sessão não pode re-renderizar a poça assada de A com o substrato NOVO —
/// o sintoma do "aplica a tudo" + retângulos. O substrato precisa ser POR-DONO como a geometria e
/// a aparência (#1). RED até o fix por-dono do #13 (o composite lê paper/gran GLOBAIS do brush vivo).
#[test]
fn watercolor_session_substrate_change_does_not_touch_baked_washes() {
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 24.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        edge_gain: 1.0,
        edge_spread: 24.0,
        warp: 0.0,
        granulation: 0.9, // forte, pra o substrato pesar no bake
        wet_rewet: 0.3,
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::PaperCold;
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A (banda x ≈ 36..84), assado com PaperCold.
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    // Sonda o NÚCLEO de A (x 48..64): dono de A e DENTRO da janela de re-bake de B (dab 72..120
    // padded 26 = 46..146), mas FORA da cobertura de B (disco em x=96, borda esquerda ~72) — então
    // B nunca vira dono ali. Só o substrato global (bug) mudaria esses bytes no commit de B.
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..140u32 {
            for x in 48..=64u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Traço B (mesma sessão, mesma geometria) TROCA o papel para None — a janela larga de B cobre A,
    // e o composite re-texturiza a poça assada de A com o substrato novo (o bug).
    t.paint.brush.paper.kind = TextureKind::None;
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([96.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([96.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([96.0, 120.0], PointerPhase::Up));
    assert_eq!(
        probe(&t),
        baked,
        "trocar o papel re-texturizou a poça assada de A (substrato global — bug #13)"
    );
}

/// **Costura na junção (Enio smoke 2026-07-09, cruz rápida com Dilution — take knobs):** a linha
/// dura seguia a fronteira de ROUBO DE DONO do traço novo dentro da união (a presença union crua
/// do `lift_wash` e o deepen full-strength no gate flipavam em 1 px ali — 29 bytes medidos).
/// Fix zero-custo: `lift_wash` lê a presença BORRADA (`bp_u`, já amostrada pro anel) e o
/// `backrun` escala pela presença da fonte. Mesmos params nos dois traços — a junção tem que
/// derreter (o degrau residual é o taper suave da água).
#[test]
fn watercolor_water_junction_owner_line_is_smooth() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.35,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.3,
        wet_dilution: 0.5,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // VERTICAL primeiro (x = 60), HORIZONTAL por último (y = 95) — a ordem do smoke; a fronteira
    // de dono do horizontal corta o vertical em y ≈ 81.
    assert!(t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Up));
    assert!(t.on_canvas_pointer(cp([20.0, 95.0], PointerPhase::Down)));
    let mut x = 20.0f32;
    while x < 170.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 95.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([170.0, 95.0], PointerPhase::Up));
    let mut max_step = 0.0f32;
    let mut at_y = 0u32;
    for y in 60..95u32 {
        let a = f32::from(px(&t, size, 60, y)[1]);
        let b = f32::from(px(&t, size, 60, y + 1)[1]);
        if (a - b).abs() > max_step {
            max_step = (a - b).abs();
            at_y = y;
        }
    }
    assert!(
        max_step <= 15.0,
        "a junção da cruz deve derreter — degrau máx G {max_step:.0} em y={at_y} \
         (sem o fix a linha de dono degrauzava 29)"
    );
}

/// **O retângulo do preview com Dilution (Enio 2026-07-09, "só com charge 1 e dilution > 0"):**
/// um traço com Dilution rega o PRÓPRIO corpo; a máscara dos campos union excluía o traço VIVO
/// (`owner != cur`) — instável: no bake do traço A o próprio pigmento fica de fora (sem
/// auto-anel), mas no pen-down de B o A vira "estrangeiro" e lift/anel RETROAGEM sobre o wash
/// inteiro dentro da janela viva de B (o retângulo; some no pen-up porque o commit re-assa tudo
/// uniforme). Água agora interage só com tinta SECA (base da sessão); molhado funde pela união.
/// Propriedade: os bytes assados de A não mudam UM byte durante o traço B.
#[test]
fn watercolor_diluted_wash_is_not_retroactively_rewetted_by_next_stroke() {
    let size = 192u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.3,
        depth: 1.5,
        edge_gain: 1.5,
        edge_spread: 8.0,
        warp: 0.0,
        granulation: 0.5,
        wet_rewet: 0.3,
        wet_dilution: 0.6, // o gatilho do smoke (Charge 1 default)
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    // Wash A assado (banda x ≈ 48..72), com a própria água (Dilution).
    assert!(t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Up));
    let probe = |t: &PainterTool| -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for y in 60..110u32 {
            for x in 44..=66u32 {
                v.push(px(t, size, x, y));
            }
        }
        v
    };
    let baked = probe(&t);
    // Traço B perto (sem encostar): a janela viva cobre A — A não pode mudar um byte.
    assert!(t.on_canvas_pointer(cp([92.0, 40.0], PointerPhase::Down)));
    let mut y = 40.0f32;
    while y < 120.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([92.0, y], PointerPhase::Move));
    }
    assert_eq!(
        probe(&t),
        baked,
        "o wash diluído assado não pode ser re-molhado retroativamente na janela viva de B"
    );
    t.on_canvas_pointer(cp([92.0, 120.0], PointerPhase::Up));
    assert_eq!(probe(&t), baked, "nem no re-bake da união do pen-up de B");
}

/// Regressão do smoke 2026-07-09 (gesto A, doc 12 take 8): o diag `[wet-diag]` mostrou
/// `sess=false` no pen-down IMEDIATAMENTE após o Enio reduzir o slider de Charge — mexer num
/// slider watercolor NÃO pode quebrar a sessão molhada (o traço seguinte vira glaze sobre
/// "seco" e ganha o rim duro by-design na junção: a "borda dura ao reduzir Charge"). O gesto
/// com Charge intacto (2→3) manteve `sess=true`, isolando o slider como o gatilho.
#[test]
fn watercolor_wet_session_survives_charge_slider_change() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 20.0,
        watercolor: true,
        wet_charge: 1.0,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([60.0, 128.0], PointerPhase::Down)));
    for i in 1..=20 {
        t.on_canvas_pointer(cp([60.0 + i as f32 * 7.0, 128.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([200.0, 128.0], PointerPhase::Up));
    assert!(
        t.wet_session_continues(),
        "controle: a sessão deve estar viva logo após o bake"
    );
    // O caminho REAL do painel (handle_panel_event → route_brush_watercolor_event →
    // set_brush_wet_charge), não o setter puro.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_CHARGE,
        0.4608,
    ));
    let n = (size as usize) * (size as usize);
    assert!(
        t.wet_session_continues(),
        "mexer no slider de Charge quebrou a sessão molhada — guards: wet_rect={} cov={} col={} \
         base={} arc={}",
        t.paint.canvas_wet_rect.is_some(),
        t.paint.stroke_coverage.len() == n,
        t.paint.stroke_color.len() == n * 4,
        t.paint
            .wet_session_base
            .as_ref()
            .is_some_and(|b| b.len() == n * 4),
        t.paint
            .wet_session_canvas
            .as_ref()
            .is_some_and(|c| Arc::ptr_eq(c, &t.canvas_rgba)),
    );
}

/// Diag do take 10 (rode com `--ignored --nocapture`): perfil 1D de luminância pela junção
/// (eixo do traço 2) vs a borda orgânica do próprio wash — quantifica a DUREZA da fronteira
/// do clareamento (bytes/px) pra calibrar a spec de suavidade.
#[test]
#[ignore = "diag exploratório — imprime perfis de transição da junção (take 10)"]
fn watercolor_junction_transition_profile() {
    for (label, wet) in [("chg<1 wet=0", 0.0f32), ("chg<1 wet=1", 1.0f32)] {
        let size = 600u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 49.8,
            color: [1.0, 0.27, 0.27],
            spacing: 0.05,
            watercolor: true,
            fill: 0.120,
            depth: 1.20,
            edge_gain: 0.70,
            edge_spread: 22.8,
            warp: 11.1,
            granulation: 0.30,
            wet_charge: 0.4841,
            wet_dilution: 0.2918,
            wet_pull: 0.22,
            wet_rewet: 0.0, // traço 1 sem rewet (como no smoke)
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        // Traço 1: vertical por x=300.
        assert!(t.on_canvas_pointer(cp([300.0, 80.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([300.0, 80.0 + i as f32 * 11.0], PointerPhase::Move));
            t.on_tick(16.0);
        }
        t.on_canvas_pointer(cp([300.0, 520.0], PointerPhase::Up));
        t.on_tick(16.0);
        // Traço 2: horizontal por y=300, com o Rewet do caso.
        t.paint.brush.wet_rewet = wet;
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([80.0, 300.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([80.0 + i as f32 * 11.0, 300.0], PointerPhase::Move));
            // Traço LENTO real (o soak do smoke: 30k px) — ~8 frames por move.
            for _ in 0..8 {
                t.on_tick(16.0);
            }
        }
        t.on_canvas_pointer(cp([520.0, 300.0], PointerPhase::Up));
        eprintln!(
            "[profile] soak_px={} water={}",
            t.paint.wet_soak.iter().filter(|&&v| v > 0).count(),
            !t.paint.stroke_water.is_empty(),
        );
        let px = &t.canvas_rgba;
        // Luminância média numa faixa 11px de altura centrada em y=300, x de 180 a 420.
        let lum = |x: usize| -> f32 {
            let mut s = 0.0f32;
            for y in 295..306usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        let prof: Vec<f32> = (180..420).map(lum).collect();
        let mut grad_max = 0.0f32;
        let mut grad_at = 0usize;
        for i in 1..prof.len() {
            let g = (prof[i] - prof[i - 1]).abs();
            if g > grad_max {
                grad_max = g;
                grad_at = 180 + i;
            }
        }
        // Baseline orgânico: a borda superior do traço 2 longe da junção (scan vertical em x=150).
        let lumv = |y: usize| -> f32 {
            let mut s = 0.0f32;
            for x in 145..156usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        let vprof: Vec<f32> = (220..300).map(lumv).collect();
        let mut vgrad_max = 0.0f32;
        for i in 1..vprof.len() {
            vgrad_max = vgrad_max.max((vprof[i] - vprof[i - 1]).abs());
        }
        eprintln!(
            "[profile {label}] junção: grad_max={grad_max:.1} bytes/px em x={grad_at} | \
             borda própria: grad_max={vgrad_max:.1} bytes/px | razão={:.2}",
            grad_max / vgrad_max.max(0.01)
        );
        let cells: Vec<String> = prof.iter().step_by(8).map(|v| format!("{v:.0}")).collect();
        eprintln!("[profile {label}] perfil x180..420/8: {}", cells.join(" "));
        // Scan VERTICAL em x=300 (dentro do traço 1): cruza a borda do footprint do traço 2 —
        // a fronteira do lift/rewet DENTRO da tinta velha (os arcos duros da foto do take 10).
        let lumj = |y: usize| -> f32 {
            let mut s = 0.0f32;
            for x in 295..306usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        let jprof: Vec<f32> = (180..420).map(lumj).collect();
        let mut jgrad_max = 0.0f32;
        let mut jgrad_at = 0usize;
        for i in 1..jprof.len() {
            let g = (jprof[i] - jprof[i - 1]).abs();
            if g > jgrad_max {
                jgrad_max = g;
                jgrad_at = 180 + i;
            }
        }
        eprintln!(
            "[profile {label}] lift-boundary (scan vertical x=300): grad_max={jgrad_max:.1}              bytes/px em y={jgrad_at} | razão vs borda própria={:.2}",
            jgrad_max / vgrad_max.max(0.01)
        );
        let jcells: Vec<String> = jprof.iter().step_by(8).map(|v| format!("{v:.0}")).collect();
        eprintln!("[profile {label}] perfil y180..420/8: {}", jcells.join(" "));
        // Sonda dos MAPAS no penhasco (x=300, y=jgrad_at±8): quem degraua ali?
        for y in (jgrad_at.saturating_sub(8))..(jgrad_at + 9) {
            let idx = y * size as usize + 300;
            eprintln!(
                "[maps {label}] y={y} lum={:.0} col_a={} depl={} cov={} own={}",
                lumj(y),
                t.paint.stroke_color.get(idx * 4 + 3).copied().unwrap_or(0),
                t.paint.stroke_deplete.get(idx).copied().unwrap_or(0),
                t.paint.stroke_coverage.get(idx).copied().unwrap_or(0),
                t.paint.wet_styles.owner.get(idx).copied().unwrap_or(0),
            );
        }
        // Buffers do caminho wet no mesmo corte: água do traço 2 + soak (cru) — qual degraua?
        for y in (jgrad_at.saturating_sub(8))..(jgrad_at + 9) {
            let idx = y * size as usize + 300;
            eprintln!(
                "[wetmaps {label}] y={y} water={} soak={}",
                t.paint.stroke_water.get(idx).copied().unwrap_or(0),
                t.paint.wet_soak.get(idx).copied().unwrap_or(0),
            );
        }
    }
}

/// Spec do take 10 (smoke 2026-07-09, rodada 3 + foto): o CLAREAMENTO da junção é o look
/// desejado ("perdeu o efeito de clareamento" — veto ao clamp do take 9), o defeito é só a
/// FRONTEIRA dura dele. Dois portadores corrigidos: (a) o flip raw↔depositado na janela fixa
/// `COL_LO..COL_HI` atravessada em ~1px espacial (→ lerp proporcional `ca8/255`); (b) o `st.wet`
/// BINÁRIO do mapa de dono nos termos wet-driven quando o Rewet difere entre traços da sessão
/// (→ campo borrado `wet_field`). Este gate exige AMBOS: clareia E suave.
#[test]
fn watercolor_junction_lightening_is_soft_and_preserved() {
    for (label, wet2) in [("wet=0", 0.0f32), ("wet=1", 1.0f32)] {
        let size = 600u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 49.8,
            color: [1.0, 0.27, 0.27],
            spacing: 0.05,
            watercolor: true,
            fill: 0.120,
            depth: 1.20,
            edge_gain: 0.70,
            edge_spread: 22.8,
            warp: 11.1,
            granulation: 0.30,
            wet_charge: 0.4841,
            wet_dilution: 0.2918,
            wet_pull: 0.22,
            wet_rewet: 0.0,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([300.0, 80.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([300.0, 80.0 + i as f32 * 11.0], PointerPhase::Move));
            t.on_tick(16.0);
        }
        t.on_canvas_pointer(cp([300.0, 520.0], PointerPhase::Up));
        t.on_tick(16.0);
        t.paint.brush.wet_rewet = wet2;
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        assert!(t.on_canvas_pointer(cp([80.0, 300.0], PointerPhase::Down)));
        for i in 1..=40 {
            t.on_canvas_pointer(cp([80.0 + i as f32 * 11.0, 300.0], PointerPhase::Move));
            for _ in 0..8 {
                t.on_tick(16.0);
            }
        }
        t.on_canvas_pointer(cp([520.0, 300.0], PointerPhase::Up));
        let px = &t.canvas_rgba;
        // Scan vertical em x=300 (dentro do traço 1, cruzando a fronteira do traço 2).
        let lum = |y: usize| -> f32 {
            let mut s = 0.0f32;
            for x in 295..306usize {
                let i = (y * size as usize + x) * 4;
                s += (f32::from(px[i]) + f32::from(px[i + 1]) + f32::from(px[i + 2])) / 3.0;
            }
            s / 11.0
        };
        // (a) O clareamento EXISTE: platô da junção > corpo do traço 1.
        let body: f32 = (200..230).map(lum).sum::<f32>() / 30.0;
        let plateau: f32 = (290..311).map(lum).sum::<f32>() / 21.0;
        assert!(
            plateau > body + 2.0,
            "[{label}] o clareamento da junção sumiu (veto do take 9): corpo={body:.1} \
             platô={plateau:.1}"
        );
        // (b) A fronteira é SUAVE: nenhum degrau de 1px acima de 4 bytes no scan interno
        // (pré-fix: 7.5 com wet=0, 11.5 com wet=1).
        let prof: Vec<f32> = (200..400).map(lum).collect();
        let mut grad_max = 0.0f32;
        let mut grad_at = 0usize;
        for i in 1..prof.len() {
            let g = (prof[i] - prof[i - 1]).abs();
            if g > grad_max {
                grad_max = g;
                grad_at = 200 + i;
            }
        }
        assert!(
            grad_max <= 4.0,
            "[{label}] fronteira dura na junção: grad {grad_max:.1} bytes/px em y={grad_at}"
        );
    }
}

/// #11 (doc 13): o slider de Drying Time mapeia SEGUNDOS → taxa de secagem (`255/seg`) e volta,
/// com clamp em `2..60 s`. Canvas-level (não muda por modo de pincel).
#[test]
fn watercolor_dry_time_slider_maps_seconds_to_rate() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 16 * 16 * 4], 16, 16);
    t.set_dry_time_s(10.0);
    assert!((t.dry_time_s() - 10.0).abs() < 0.05, "10 s round-trips");
    assert!(
        (t.paint.dry_rate_per_s - 25.5).abs() < 0.1,
        "10 s ⇒ ~25.5 bytes/s"
    );
    t.set_dry_time_s(60.0);
    assert!(
        (t.paint.dry_rate_per_s - 4.25).abs() < 0.05,
        "60 s ⇒ ~4.25 bytes/s"
    );
    t.set_dry_time_s(0.5); // abaixo do mínimo → clamp em 2 s
    assert!((t.dry_time_s() - 2.0).abs() < 0.05, "clamp inferior 2 s");
    t.set_dry_time_s(999.0); // acima do máximo → clamp em 60 s
    assert!((t.dry_time_s() - 60.0).abs() < 0.05, "clamp superior 60 s");
    // Default = ~10 s (o knob CANVAS_WET_DRY_DEFAULT).
    let fresh = PainterTool::default();
    assert!((fresh.dry_time_s() - 10.0).abs() < 0.05, "default ~10 s");
}

/// #9 (doc 13): o botão Dry encerra a sessão molhada NA HORA — os pixels assados ficam, mas a
/// fusão com traços futuros acaba (canvas_wet zerado, sessão morta).
#[test]
fn watercolor_dry_button_ends_the_wet_session() {
    let size = 64u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 16.0,
        watercolor: true,
        ..Default::default()
    };
    for slot in &mut t.paint.brush_by_mode {
        *slot = t.paint.brush;
    }
    assert!(t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down)));
    for i in 1..=12 {
        t.on_canvas_pointer(cp([16.0 + i as f32 * 3.0, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    assert!(
        t.wet_session_continues(),
        "precondição: sessão viva após o bake"
    );
    let baked = t.canvas_rgba.clone();
    t.dry_session_now();
    assert!(!t.wet_session_continues(), "Dry encerrou a sessão");
    assert!(t.paint.canvas_wet.is_empty(), "canvas_wet zerado");
    assert!(t.paint.canvas_wet_rect.is_none(), "rect de umidade zerado");
    assert_eq!(t.canvas_rgba, baked, "Dry NÃO toca os pixels assados");
}

/// #10 (doc 13): o botão Wet re-molha o canvas inteiro SEM depositar pigmento (canvas_wet = 255
/// full + rect = canvas todo), sem tocar os pixels.
#[test]
fn watercolor_wet_button_moistens_the_canvas() {
    let size = 32u32;
    let mut t = PainterTool::default();
    let src = vec![200u8; (size * size * 4) as usize];
    t.set_source(src.clone(), size, size);
    t.wet_canvas_now();
    let n = (size * size) as usize;
    assert_eq!(t.paint.canvas_wet.len(), n, "canvas_wet dimensionado");
    assert!(
        t.paint.canvas_wet.iter().all(|&w| w == 255),
        "umidade cheia"
    );
    assert_eq!(
        t.paint.canvas_wet_rect,
        Some((0, 0, size as usize, size as usize)),
        "rect = canvas inteiro"
    );
    assert_eq!(
        t.canvas_rgba.as_slice(),
        src.as_slice(),
        "Wet NÃO deposita pigmento"
    );
}

/// Costura do ROUTE (doc 13 #9-#11): os ids novos da Wetness card despacham pelos setters certos
/// via `route_brush_watercolor_event` — o par do seam.rs do painel (que cobre o forward).
#[test]
fn watercolor_route_dispatches_wetness_controls() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    assert!(t.route_brush_watercolor_event(&PanelEvent::SetValue(
        core_ids::PAINTER_WATERCOLOR_DRY_TIME,
        30.0
    )));
    assert!(
        (t.dry_time_s() - 30.0).abs() < 0.05,
        "DRY_TIME → set_dry_time_s"
    );
    assert!(
        t.route_brush_watercolor_event(&PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_WET_NOW))
    );
    assert!(!t.paint.canvas_wet.is_empty(), "WET_NOW → wet_canvas_now");
    assert!(
        t.route_brush_watercolor_event(&PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_DRY_NOW))
    );
    assert!(t.paint.canvas_wet.is_empty(), "DRY_NOW → dry_session_now");
}

/// #12a (doc 14): o accessor `canvas_wet_view` expõe o mapa de umidade + rect para o overlay
/// on-canvas — Some quando molhado, None quando seco. (O véu em si é smoke-only.)
#[test]
fn watercolor_canvas_wet_view_exposes_moisture() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    assert!(t.canvas_wet_view().is_none(), "seco → None");
    t.wet_canvas_now();
    let (bytes, w, h, rect) = t.canvas_wet_view().expect("molhado → Some");
    assert_eq!((w, h), (32, 32));
    assert_eq!(rect, [0, 0, 32, 32], "rect = canvas inteiro");
    assert!(bytes.iter().all(|&b| b == 255), "umidade cheia");
    t.dry_session_now();
    assert!(t.canvas_wet_view().is_none(), "secou → None");
}

// ── Impasto (#16) — the foundation gate: the master switch is the ONLY gate ───────────────────────

/// Paint the SAME rich stroke — Shape + Grain + Randomize Color + Jitter Scale/Rotate + Symmetry +
/// Tiling, i.e. every feature Enio asked to be integrated — through a brush the caller may tweak.
/// Returns the canvas bytes. One tool per call, so the two runs share nothing but the code.
fn impasto_rich_stroke(tweak: impl FnOnce(&mut BrushSpec)) -> Vec<u8> {
    use ph2d_painter_brush::{MirrorAxis, TextureKind, TextureMapping};
    let size = 48u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let mut b = BrushSpec {
        radius_px: 9.0,
        hardness: 0.3,
        color: [0.2, 0.5, 0.9],
        space_attenuation: false,
        // Everything that must ride the SAME dab list / SAME stamp mask as the height will.
        dab_flatten: 0.4,
        dab_angle_deg: 20,
        jitter_scale: 0.3,
        jitter_rotate: 0.4,
        jitter_spacing: 0.2,
        color_jitter_enabled: true,
        color_jitter_hue: 0.3,
        color_jitter_sat: 0.2,
        color_jitter_val: 0.2,
        grain_depth: 0.8,
        ..Default::default()
    };
    b.shape.kind = TextureKind::Checker; // procedural silhouette (no image pixels needed)
    b.shape.mapping = TextureMapping::ViewPlane;
    b.texture.kind = TextureKind::Noise; // Grain
    b.texture.mapping = TextureMapping::ViewPlane;
    b.symmetry.enabled = true;
    b.symmetry.axis = MirrorAxis::X;
    b.symmetry.center = [24.0, 24.0];
    tweak(&mut b);
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.paint.tiling = [true, true]; // wrap on both axes
    t.on_canvas_pointer(cp([6.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 22.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 41.0], PointerPhase::Up));
    (*t.canvas_rgba).clone()
}

#[test]
fn impasto_off_is_byte_identical() {
    // T1.3, the foundation of #16: while the master switch is off, NONE of the impasto knobs may be
    // read. If any of them leaks into the colour path — even as a rounding difference — this fails.
    //
    // The knobs deliberately carry live *when-enabled* defaults (depth 0.5, smoothing 0.2), so the
    // default is inert because of the SWITCH, not because the values happen to be neutral. This gate
    // is what says so: it drives every knob to a wild value and demands the same bytes.
    //
    // The stroke is the rich one on purpose — Shape, Grain, Randomize Color, Jitter Scale/Rotate,
    // Symmetry and Tiling all active. Those are exactly the features the height channel is going to
    // share the dab list and the stamp mask with, so if wiring the height ever perturbs the dab
    // stream (a re-ordered RNG draw, an extra dab, a differently-shaped mask), the COLOUR moves too
    // and this gate catches it — in the one configuration where it is hardest to notice by eye.
    let baseline = impasto_rich_stroke(|_| {});
    let wild = impasto_rich_stroke(|b| {
        b.impasto_depth = 1.0;
        b.impasto_smoothing = 1.0;
        b.impasto_source = DepthSource::Grain;
        b.impasto_draw_to = DrawTo::Depth; // would suppress ALL pigment if it were read
        // ...but the master switch stays OFF.
        b.impasto = false;
    });
    assert_eq!(
        baseline, wild,
        "with Impasto off, the impasto settings must not reach a single pixel"
    );
    assert!(
        baseline.iter().any(|&b| b != 255),
        "sanity: the fixture actually painted (an all-white canvas would make this gate vacuous)"
    );
}

// ── Impasto (#16) — the height channel rides the SHARED dab list ──────────────────────────────────

/// The relief the artist would see on the active layer (committed + the open stroke's envelope).
fn relief(t: &PainterTool) -> Vec<f32> {
    let id = t.layers.active().expect("a layer is active");
    t.layer_height_view(id).unwrap_or_default()
}

/// A canvas with an impasto brush ready to sculpt. Hard disk ⇒ a deterministic, level plateau.
fn impasto_canvas(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 6.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.5,
        // The artist's defaults (Depth 1 / Body 0 / Smoothing 1, Enio 2026-07-12) are for PAINTING; a
        // fixture that inherited them would be asserting about the settle blur and the round profile
        // in gates that are about neither. Pin the two that would blur the claim, per-gate.
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t
}

#[test]
fn impasto_tiling_sculpts_the_opposite_edge() {
    // THE structural gate (plan §5, T3.4.4). The height must consume the dab list the COLOUR consumes —
    // already wrapped by Tiling. A height pass that walked its own geometry would paint relief only
    // where the brush physically is, and the wrapped edge would come out flat: paint on one side of the
    // seam, no thickness on the other. That is how "Tiling doesn't work in Impasto" gets born, and this
    // is the test that refuses to let it.
    let size = 40u32;
    let mut t = impasto_canvas(size);
    t.paint.tiling = [true, false]; // wrap on X
    t.on_canvas_pointer(cp([1.0, 20.0], PointerPhase::Down)); // hard against the LEFT edge
    t.on_canvas_pointer(cp([1.0, 20.0], PointerPhase::Up));
    let h = relief(&t);
    assert!(!h.is_empty(), "the stroke laid down relief");
    let at = |x: u32, y: u32| h[(y * size + x) as usize];
    assert!(at(0, 20) > 0.0, "relief where the brush is");
    assert!(
        at(size - 1, 20) > 0.0,
        "and relief on the WRAPPED edge — the height rides the same tiled dab list as the colour"
    );
    assert_eq!(at(20, 20), 0.0, "and nowhere the brush never went");
}

#[test]
fn impasto_symmetry_mirrors_the_relief() {
    // The Symmetry twin of the Tiling gate: `push_symmetric` mirrors dabs INTO the list, so the mirrored
    // dab carries its height for free — if, and only if, the height reads the list.
    let size = 40u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.symmetry.enabled = true;
    b.symmetry.axis = ph2d_painter_brush::MirrorAxis::X; // mirror across the vertical centre line
    b.symmetry.center = [20.0, 20.0];
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([8.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([8.0, 12.0], PointerPhase::Up));
    let h = relief(&t);
    let at = |x: u32, y: u32| h[(y * size + x) as usize];
    assert!(at(8, 20) > 0.0, "relief under the brush");
    assert!(
        at(32, 20) > 0.0,
        "and its mirror image — 8 and 32 straddle the axis at x=20"
    );
    // And NOTHING in between. This half of the gate exists because a mutation found the hole: the body
    // is swept back to the PREVIOUS dab along the path, and `push_symmetric` INTERLEAVES its copies
    // (`[base, mirror, base, mirror, …]`). Link the immediate neighbour in the list and you sweep a
    // capsule from every dab to its own MIRROR — a bar of paint straight across the canvas. The
    // assertions above pass happily with that bar present, which is exactly the kind of gate that lets a
    // bug ship. The path predecessor is `copies` entries back, and this is what says so.
    for y in 18..23 {
        assert_eq!(
            at(20, y),
            0.0,
            "no relief on the mirror axis — the stroke and its reflection must not be joined by a bar \
             (x=20, y={y})"
        );
    }
}

#[test]
fn impasto_one_stroke_is_one_thickness_but_two_strokes_add() {
    // The envelope (T1.2). Scrubbing back and forth WITHIN a stroke must not build a staircase of
    // paint — a loaded brush passing over its own line leaves one thickness. But a SECOND stroke
    // genuinely piles more on. Both halves matter: envelope-everything would make impasto unbuildable;
    // add-everything would make a single slow stroke pile up under the cursor.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Move)); // pass 2 over the same point
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Move)); // pass 3
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let one = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        (one - 0.5).abs() < 1e-5,
        "one stroke = one depth, got {one}"
    );

    // A second, separate stroke over the same paint.
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let two = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        (two - 1.0).abs() < 1e-5,
        "a second stroke lays MORE paint on top, got {two}"
    );
}

#[test]
fn impasto_eraser_takes_the_relief_with_the_paint() {
    // T1.6 — not optional, a correction: without it the eraser removes the pigment and the light pass
    // keeps reporting a ridge. Ghost relief.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert!(relief(&t)[(20 * 40 + 20) as usize] > 0.0, "relief is there");

    t.paint.eraser = true;
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let left = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        left.abs() < 1e-5,
        "the eraser scrubbed the relief away with the paint, got {left}"
    );
}

#[test]
fn impasto_draw_to_depth_sculpts_without_painting() {
    // T1.8 — the palette knife: thickness, no pigment. The canvas must come out BYTE-identical while
    // the relief changes. (A "sculpt" that quietly tinted the canvas would be a lie.)
    let mut t = impasto_canvas(40);
    let mut b = t.paint.brush;
    b.impasto_draw_to = DrawTo::Depth;
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    let before = (*t.canvas_rgba).clone();
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert_eq!(
        *t.canvas_rgba, before,
        "Draw To = Depth deposits no pigment — the canvas is untouched"
    );
    assert!(
        relief(&t)[(20 * 40 + 20) as usize] > 0.0,
        "...but the relief is there"
    );
}

#[test]
fn impasto_undo_takes_back_the_relief_with_the_pixels() {
    // The relief lives in the undo snapshot. If it didn't, Ctrl+Z would restore the colour and leave
    // the thickness — paint that is gone but still catches the light.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert!(relief(&t)[(20 * 40 + 20) as usize] > 0.0);
    assert!(t.undo_last(), "the stroke is one undo step");
    let h = relief(&t);
    assert!(
        h.is_empty() || h[(20 * 40 + 20) as usize].abs() < 1e-6,
        "undo took the relief back with the pixels"
    );
    assert!(t.redo_last(), "and it redoes");
    assert!(
        relief(&t)[(20 * 40 + 20) as usize] > 0.0,
        "and redo brings it back"
    );
}

#[test]
fn watercolor_is_untouched_by_impasto() {
    // ★ THE BARRIER (plan §2, Enio's explicit order: "Watercolor é uma implementação à parte e não deve
    // ser tocada ou ferida"). With the wash on, an impasto brush must be INERT: the canvas byte-identical
    // to the same wash with Impasto off, and not one texel of relief deposited. The architecture already
    // guarantees it (`stamp_dabs` short-circuits into the optical path before the router, where the
    // height choke point lives) — this test is what will notice the day someone changes that.
    let wash = |impasto: bool| -> (Vec<u8>, Vec<f32>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
        let b = BrushSpec {
            radius_px: 8.0,
            color: [0.8, 0.2, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            impasto,
            impasto_depth: 1.0,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([16.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([32.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([32.0, 24.0], PointerPhase::Up));
        let h = relief(&t);
        ((*t.canvas_rgba).clone(), h)
    };
    let (plain, h_off) = wash(false);
    let (with_impasto, h_on) = wash(true);
    assert!(
        plain.iter().any(|&b| b != 255),
        "sanity: the wash actually painted"
    );
    assert_eq!(
        plain, with_impasto,
        "Impasto must not move a single pixel of the watercolor path"
    );
    assert!(h_off.is_empty(), "and the wash deposits no relief...");
    assert!(
        h_on.iter().all(|&v| v == 0.0),
        "...with Impasto ticked either — the card is hidden there, and the code path never runs"
    );
}

#[test]
fn impasto_on_does_not_disturb_the_pigment() {
    // RULE 2, and the gate that guards it. The Grain's per-dab random frame is drawn from a PERSISTENT
    // rng stream (`tex_rng`). The height pass has to resolve the same frames the colour pass will — so
    // it reads a COPY of that stream and throws it away. If it ever wrote the stream back (the obvious,
    // wrong thing), every colour dab would draw the NEXT random frame instead of its own: the relief and
    // the pigment would carry different grain, and the artist would see the texture change the moment
    // they ticked Impasto — a checkbox for RELIEF silently repainting the COLOUR.
    //
    // So: turning Impasto ON must add thickness and change not one pixel of pigment.
    let stroke = |impasto: bool| -> Vec<u8> {
        use ph2d_painter_brush::{TextureKind, TextureMapping};
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
        let mut b = BrushSpec {
            radius_px: 7.0,
            color: [0.2, 0.6, 0.3],
            space_attenuation: false,
            impasto,
            impasto_depth: 1.0,
            impasto_source: DepthSource::Grain,
            ..Default::default()
        };
        // A Grain that DRAWS from the rng every dab — the whole point. A static grain would make this
        // gate vacuous: with nothing consuming the stream, a stream bug is invisible.
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::ViewPlane;
        b.texture.random_angle = true;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let without = stroke(false);
    let with = stroke(true);
    assert!(
        without.iter().any(|&b| b != 255),
        "sanity: the fixture painted"
    );
    assert_eq!(
        without, with,
        "Impasto adds THICKNESS, not a different painting — the height pass must not consume the \
         colour's random stream"
    );
}

#[test]
fn impasto_per_layer_color_leaves_one_coherent_relief() {
    // T1.7 — the plan called this "the one place `for free` does not hold", because the Per-Layer Color
    // route BYPASSES the ordinary cached routes: it composites N tinted shape layers onto the canvas
    // itself. It turned out to be free after all, and for a reason worth writing down: the height is
    // taken at the ONE choke point ABOVE the whole route dispatch, from the union silhouette that all N
    // layers already flatten into. So the relief is ONE coherent body — the thickness of the paint the
    // brush laid — not N stacked steps, one per shape layer, which is exactly the artefact the plan
    // feared. Nothing about this is guaranteed by the code reading well; it is guaranteed by this test.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    // Two shape layers, each a solid 8×8 tile → the union silhouette is the whole tip.
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    assert!(
        t.brush_settings().shape_per_layer_color,
        "the fixture really is in Per-Layer Color mode"
    );
    let mut b = t.paint.brush;
    b.radius_px = 8.0;
    b.impasto = true;
    b.impasto_depth = 0.6;
    b.space_attenuation = false;
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));

    let h = relief(&t);
    assert!(!h.is_empty(), "the Per-Layer Color route laid down relief");
    let centre = h[(32 * 64 + 32) as usize];
    assert!(centre > 0.0, "there IS a body under the dab ({centre})");
    // ONE dab of depth 0.6 ⇒ at most 0.6 anywhere. Two layers each contributing their own step would
    // land at ~1.2 — the "N stacked steps" artefact, and the whole reason this task existed.
    let peak = h.iter().fold(0.0f32, |m, &v| m.max(v));
    assert!(
        peak > 0.3,
        "sanity: a real body, not a sliver — else the bound below would pass vacuously (peak {peak})"
    );
    assert!(
        peak <= 0.6 + 1e-5,
        "the relief is ONE body of the brush's depth, not one step per shape layer (peak {peak})"
    );
}

// ── Impasto (#16) — the light pass ────────────────────────────────────────────────────────────────

/// The composited, LIT preview.
fn lit(t: &mut PainterTool) -> Vec<u8> {
    let (rgba, _, _) = t.take_preview_arc().expect("a preview");
    (*rgba).clone()
}

#[test]
fn impasto_light_leaves_flat_paint_byte_identical() {
    // THE contract of the whole pass (T2.3, and stronger than the plan asked). The shading is RELATIVE:
    // a pixel's response is divided by a flat surface's response. So where there is no relief the pass
    // multiplies by exactly 1 and adds exactly 0.
    //
    // The naive `rgb × (N·L)` would fail this: a flat surface lit from 45° returns 0.707, so switching
    // the light on would darken the ENTIRE painting by 30%. That bug is in half the emboss filters ever
    // shipped, and this is the assertion that refuses it.
    let mut t = impasto_canvas(40);
    let mut b = t.paint.brush;
    b.impasto = false; // paint normally: pigment, no body
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let unlit = lit(&mut t);

    // Now switch the light on, with the relief still empty. Not one byte may move.
    t.paint.impasto_show = true;
    t.invalidate_composite();
    let with_light = lit(&mut t);
    assert_eq!(
        unlit, with_light,
        "no relief ⇒ the light pass is a no-op, to the byte"
    );
}

#[test]
fn impasto_light_reads_as_raised_not_engraved() {
    // The APPEARANCE oracle ([[feedback_oracle_must_model_appearance_not_implementation]]): an oracle
    // derived from the shader would go green with the relief inverted on screen. So assert the thing a
    // human sees instead — a RIDGE lit from the left is BRIGHT on its left flank and DARK on its right.
    // Get the sign wrong (the classic emboss bug) and the paint reads as a groove carved INTO the
    // canvas; the arithmetic is just as self-consistent, and every shader-shaped oracle passes.
    let size = 60u32;
    let mut t = impasto_canvas(size);
    // A SOFT brush, deliberately: `impasto_canvas` paints with a hard disk, whose relief is a plateau
    // with vertical walls — h is identical at the centre and at both "flanks", so there is no gradient
    // to light and the test would have been asserting about nothing. (The sanity check below caught
    // exactly that on the first run.) A smooth falloff gives a real ridge with real flanks.
    let mut soft = t.paint.brush;
    soft.hardness = 0.0;
    soft.falloff = Falloff::Smooth;
    soft.radius_px = 10.0;
    t.paint.brush = soft;
    for slot in &mut t.paint.brush_by_mode {
        *slot = soft;
    }
    t.paint.impasto_light_angle_deg = 180; // from the LEFT (-x)
    t.paint.impasto_light_elev_deg = 30;
    t.paint.impasto_shine = 0.0; // isolate the diffuse term — the highlight is a separate question
    // A vertical ridge of paint down the middle.
    t.on_canvas_pointer(cp([30.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 50.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 50.0], PointerPhase::Up));

    let h = relief(&t);
    let img = lit(&mut t);
    let lum = |x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    // The flanks are FOUND from the relief itself — its steepest wall on each side of the crest. With
    // the body curve the interior is a PLATEAU, so a fixed offset (the old `25`/`35`) lands on flat
    // paint and asserts about nothing.
    let hx = |x: u32| h[(30 * size + x) as usize];
    assert!(hx(30) > 0.0, "the ridge is there");
    let steepest = |xs: std::ops::Range<u32>| {
        xs.max_by(|&a, &b| {
            let ga = (hx(a + 1) - hx(a - 1)).abs();
            let gb = (hx(b + 1) - hx(b - 1)).abs();
            ga.partial_cmp(&gb).unwrap()
        })
        .expect("a non-empty search band")
    };
    let (left_flank, right_flank) = (steepest(2..30), steepest(31..size - 2));
    assert!(
        hx(left_flank) < hx(30) && hx(right_flank) < hx(30),
        "the fixture really is a ridge (it falls away on both sides)"
    );
    let l = lum(left_flank, 30);
    let r = lum(right_flank, 30);

    // The reference is THE SAME PAINT WITH THE LIGHT OFF — not some other pixel. (My first attempt
    // used the canvas at x=2 as "flat": that is bare white paper, not flat paint, so the comparison was
    // meaningless and the assert failed for a reason that had nothing to do with the shading.)
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let base_img = lit(&mut t);
    let base = |x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(base_img[i]) + u32::from(base_img[i + 1]) + u32::from(base_img[i + 2])) as f32
            / 3.0
    };
    let (bl, br) = (base(left_flank, 30), base(right_flank, 30));
    t.paint.impasto_show = true;
    t.invalidate_composite();

    // THE appearance claim, stated the way an artist would: the flank turned TOWARD the light gets
    // brighter than the paint really is, and the flank turned AWAY gets darker. That is what "raised"
    // looks like. An implementation that merely darkened every edge would fail the first half; one with
    // the normal's sign flipped would fail both, and would look like a groove carved into the canvas.
    assert!(
        l > bl,
        "the flank facing the light is BRIGHTER than the paint under it ({l} vs {bl})"
    );
    assert!(
        r < br,
        "the flank turned away is DARKER than the paint under it ({r} vs {br})"
    );
    assert!(
        l > r,
        "so, lit from the left, the left flank beats the right ({l} vs {r})"
    );

    // Rotate the light 180° and the bright flank must SWAP. (A pass that merely darkened edges — any
    // edge, regardless of the light — would sail through the assertion above and die here.)
    t.paint.impasto_light_angle_deg = 0; // from the RIGHT (+x)
    t.invalidate_composite();
    let img = lit(&mut t);
    let lum2 = |x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    let (l2, r2) = (lum2(left_flank, 30), lum2(right_flank, 30));
    assert!(
        r2 > l2,
        "move the light to the RIGHT and the bright flank follows it ({l2} vs {r2})"
    );
}

#[test]
fn impasto_light_off_is_byte_identical_and_a_hidden_layer_casts_none() {
    // T2.3: `Show Impasto` off ⇒ the pass does not run ⇒ the composite is what it always was.
    // And the relief of a HIDDEN layer must go dark with it — otherwise the light keeps reporting a
    // ridge over paint that is no longer on screen.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));

    t.paint.impasto_show = true;
    t.invalidate_composite();
    let shaded = lit(&mut t);

    t.paint.impasto_show = false;
    t.invalidate_composite();
    let plain = lit(&mut t);
    assert_ne!(shaded, plain, "the light pass is actually doing something");

    // Hide the layer that carries the relief: with the light back ON, the composite must match the
    // unlit one (no relief is visible, so none is lit).
    t.paint.impasto_show = true;
    let id = t.layers.active().expect("active layer");
    t.set_layer_visible(id, false);
    t.invalidate_composite();
    let hidden_lit = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let hidden_plain = lit(&mut t);
    assert_eq!(
        hidden_lit, hidden_plain,
        "a hidden layer's relief catches no light"
    );
}

#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn impasto_perf_kill_criterion() {
    // The kill-criterion frozen BEFORE the build (plan §7, DIRETIVA §5): canvas 2048², r=100, a dragged
    // stroke, Show Impasto on. Target ≤ 4 ms/move for the whole impasto cost (deposit + light over the
    // dirty rect); KILL at 8 ms after two CPU optimisation attempts — at which point the feature is
    // GPU-only or it does not exist in this form. Numbers, in ms, in --release. No verdict by vibes.
    use std::time::Instant;
    const MOVES: u32 = 20;
    // The same stroke with the feature OFF and ON. The number that matters is the DELTA — the frame
    // already costs something without Impasto, and charging that to Impasto would flatter it.
    let run = |impasto: bool| -> (f64, f64) {
        let size = 2048u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 100.0,
            color: [0.2, 0.4, 0.8],
            space_attenuation: false,
            impasto,
            impasto_depth: 0.7,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([200.0, 1024.0], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let (mut worst, mut total) = (0.0f64, 0.0f64);
        for i in 0..MOVES {
            let x = 220.0 + f64::from(i) * 40.0;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x as f32, 1024.0], PointerPhase::Move));
            let _ = t.take_preview_arc(); // deposit + composite + light: what a frame really costs
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            worst = worst.max(ms);
            total += ms;
        }
        (total / f64::from(MOVES), worst)
    };
    let (off_mean, off_worst) = run(false);
    let (on_mean, on_worst) = run(true);
    println!(
        "@2048² r100 — impasto OFF: mean {off_mean:.2} ms, worst {off_worst:.2} ms\n\
         @2048² r100 — impasto  ON: mean {on_mean:.2} ms, worst {on_worst:.2} ms\n\
         >>> IMPASTO COST: mean {:.2} ms/move, worst {:.2} ms/move (target ≤4, kill at 8)",
        on_mean - off_mean,
        on_worst - off_worst
    );
}

#[test]
fn impasto_smoothing_settles_the_paint_it_just_laid() {
    // Smoothing is a knob I very nearly shipped DEAD — declared in the spec, threaded to the panel, and
    // read by nothing. (That is the exact species the 2026-07-12 sweep spent itself exterminating, so
    // shipping a fresh one would have been quite the joke.) It settles the deposit like a heavy medium
    // relaxing: the ridges soften. Measured as what it IS — the peak gradient of the relief falls.
    let ridge = |smoothing: f32| -> Vec<f32> {
        let size = 60u32;
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.impasto_smoothing = smoothing;
        b.radius_px = 8.0; // a hard disk ⇒ a sharp-walled slab: maximum gradient to soften
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([30.0, 15.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([30.0, 45.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([30.0, 45.0], PointerPhase::Up));
        relief(&t)
    };
    let steepest = |h: &[f32]| {
        let size = 60usize;
        let mut m = 0.0f32;
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                let g = (h[y * size + x + 1] - h[y * size + x - 1]).abs();
                m = m.max(g);
            }
        }
        m
    };
    let sharp = ridge(0.0);
    let settled = ridge(1.0);
    let (gs, gt) = (steepest(&sharp), steepest(&settled));
    assert!(gs > 0.0, "the sharp ridge has walls to soften");
    assert!(
        gt < gs * 0.7,
        "Smoothing settles the paint: the steepest wall falls from {gs} to {gt}"
    );
    // Volume is conserved — settling SPREADS the paint, it does not evaporate it. (A blur that leaked
    // volume would quietly flatten every stroke the artist smoothed.)
    let vol = |h: &[f32]| h.iter().sum::<f32>();
    let (vs, vt) = (vol(&sharp), vol(&settled));
    assert!(
        (vt - vs).abs() < vs * 0.05,
        "the paint spreads, it does not vanish ({vs} → {vt})"
    );
}

#[test]
fn impasto_hides_itself_in_every_mode_it_does_not_apply_to() {
    // §1.2 of the plan, as an EXECUTABLE gate. The card is painted only when `impasto_applies`, and a
    // card that is not painted registers no hit — so this one predicate is what makes the whole matrix
    // real. A prose checklist in a doc does not bite; this does.
    //
    // Watercolor is the one that matters most: Enio's order was that it "é uma implementação à parte e
    // não deve ser tocada ou ferida". Impasto must not so much as APPEAR there.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    assert!(
        t.impasto_applies(),
        "a plain Paint brush is where Impasto lives"
    );
    assert!(
        t.brush_settings().impasto_applies,
        "and the panel is told so"
    );

    // Watercolor: the wash is a separate implementation and thin paint besides.
    t.paint.brush.watercolor = true;
    assert!(!t.impasto_applies(), "hidden under the Watercolor wash");
    assert!(!t.brush_settings().impasto_applies);
    t.paint.brush.watercolor = false;

    // Eraser: it TAKES relief away (that is wired), but it has no body of its own to configure.
    t.paint.eraser = true;
    assert!(!t.impasto_applies(), "hidden for the Eraser");
    t.paint.eraser = false;

    // The pixel-processing modes: Smear / Blur / Clone move paint that is already down (dragging the
    // relief with it is `Plow` — named, deferred); Mask is a grayscale channel with no body; Inpaint is
    // a heal disc that ignores the brush entirely.
    for mode in [
        PaintMode::Smear,
        PaintMode::Blur,
        PaintMode::Clone,
        PaintMode::Mask,
        PaintMode::Inpaint,
        PaintMode::Selection,
    ] {
        t.paint.paint_mode = mode;
        assert!(
            !t.impasto_applies(),
            "Impasto must not show up in {mode:?} — it deposits no fresh paint"
        );
        assert!(!t.brush_settings().impasto_applies);
    }
    t.paint.paint_mode = PaintMode::Paint;
    assert!(t.impasto_applies(), "and it comes back in Paint");
}

#[test]
fn impasto_panel_events_reach_the_brush() {
    // The seam test in the panel proves the widget forwards the event; this proves the TOOL consumes it
    // and the value lands in the spec. Both halves are needed: either one alone leaves a knob that looks
    // wired and is not (`feedback_tool_unit_green_integration_dead`).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);

    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_ENABLE));
    assert!(t.paint.brush.impasto, "Enable reached the brush");

    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_IMPASTO_DEPTH, -0.4));
    assert!(
        (t.paint.brush.impasto_depth + 0.4).abs() < 1e-6,
        "Depth, negative (carving) and all"
    );

    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_SOURCE_GRAIN));
    assert_eq!(t.paint.brush.impasto_source, DepthSource::Grain);

    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_DRAW_DEPTH));
    assert_eq!(t.paint.brush.impasto_draw_to, DrawTo::Depth);

    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_IMPASTO_SMOOTHING,
        0.9,
    ));
    assert!((t.paint.brush.impasto_smoothing - 0.9).abs() < 1e-6);

    // The canvas half.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_SHOW));
    assert!(!t.paint.impasto_show, "Show Impasto toggled off");
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_IMPASTO_LIGHT_ANGLE,
        200.0,
    ));
    assert_eq!(t.paint.impasto_light_angle_deg, 200);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_IMPASTO_LIGHT_ELEV,
        1.0,
    ));
    assert_eq!(
        t.paint.impasto_light_elev_deg, 5,
        "elevation floors at 5° — a grazing light divides by ~0"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_IMPASTO_SHINE, 0.7));
    assert!((t.paint.impasto_shine - 0.7).abs() < 1e-6);

    // Reset restores the settings — and must NOT delete relief the artist already sculpted.
    t.paint.brush.impasto = true;
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    let sculpted = relief(&t);
    assert!(
        sculpted.iter().any(|&v| v != 0.0),
        "there is relief on the canvas"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_RESET));
    assert!(!t.paint.brush.impasto, "Reset restored the defaults");
    assert_eq!(
        relief(&t),
        sculpted,
        "...and did NOT delete the artist's sculpting — Reset is for the SETTINGS"
    );
}

#[test]
fn impasto_light_does_not_shade_paint_that_is_not_there() {
    // Enio's smoke (2026-07-12) showed a pale echo hugging each stroke where the eye saw no paint.
    // The light pass is a MULTIPLY, so it cannot tint bare white — but it CAN darken the brush's
    // near-invisible falloff tail: (255,248,248) × 0.75 = (191,186,186), a pink-grey halo over paint
    // nobody could see before. And the normal comes from the SLOPE, not the height: a film of paint one
    // thousandth deep, carrying a grain that swings per texel, has micro-slopes as steep as a real
    // ridge's — so it was shaded just as hard. Relief where there is no paint.
    //
    // The fix is the physical one (`BODY_MIN`): below a real body of paint, the pass fades to a no-op.
    // Measured: 53 offending pixels before, 0 after.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 200u32;
    let paint = |impasto: bool| -> (Vec<u8>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 20.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain, // the per-texel grain is what makes the slopes steep
            impasto_smoothing: 0.15,
            jitter_spacing: 0.6, // the sweep grows with jitter — it must not overshoot onto bare canvas
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::ViewPlane;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        let path = [[100.0, 40.0], [110.0, 90.0], [100.0, 140.0], [110.0, 170.0]];
        t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
        for p in &path[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(path[3], PointerPhase::Up));
        let canvas = (*t.canvas_rgba).clone();
        let (comp, _, _) = t.take_preview_arc().expect("preview");
        (canvas, (*comp).clone())
    };
    let (canvas, unlit) = paint(false);
    let (_, litc) = paint(true);

    // Where the canvas is still ≥ 96% white, there is nothing the eye reads as painted.
    let (mut faint, mut drifted, mut worst) = (0u32, 0u32, 0i32);
    for i in (0..canvas.len()).step_by(4) {
        if 255 - i32::from(canvas[i + 1]) > 10 {
            continue; // real paint — the light SHOULD shade this
        }
        faint += 1;
        let d = (i32::from(litc[i + 1]) - i32::from(unlit[i + 1])).abs();
        if d > 8 {
            drifted += 1;
        }
        worst = worst.max(d);
    }
    assert!(
        faint > 10_000,
        "sanity: the fixture has a large unpainted field"
    );
    assert_eq!(
        drifted, 0,
        "the light pass shaded {drifted} pixels that carry no paint (worst drift {worst} levels) — \
         relief where there is no paint"
    );
}

#[test]
fn impasto_shadowed_paint_is_dark_but_never_black() {
    // The black smudges on the stroke ENDS of Enio's smoke: a cap is where the height drops from full
    // to nothing across a pixel, so it is the steepest slope on the canvas — the first place a diffuse
    // term with a floor of ZERO bites. It multiplied the pixel straight to black. Paint in shadow is
    // dark; it is not a hole. `AMBIENT` is the floor, folded so a FLAT surface still returns exactly 1
    // (the byte-identity contract is untouched — `impasto_light_leaves_flat_paint_byte_identical`).
    let size = 60u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.impasto_depth = 1.0; // maximum relief ⇒ the steepest walls this brush can make
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Up));

    let h = relief(&t);
    let lum = |img: &[u8], x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    t.paint.impasto_show = true;
    t.invalidate_composite();
    let shaded = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let plain = lit(&mut t);

    // The darkest LIT pixel that actually carries paint — measured against that same pixel unlit.
    let mut worst_ratio = f32::MAX;
    for y in 0..size {
        for x in 0..size {
            if h[(y * size + x) as usize].abs() < 0.05 {
                continue; // no body here — not what this gate is about
            }
            let base = lum(&plain, x, y).max(1.0);
            worst_ratio = worst_ratio.min(lum(&shaded, x, y) / base);
        }
    }
    assert!(
        worst_ratio < 1.0,
        "sanity: something on this stroke IS in shadow (else the gate proves nothing)"
    );
    assert!(
        worst_ratio > 0.25,
        "the deepest shadow on the paint crushed it to {:.0}% of its colour — paint in shadow is \
         dark, not a black hole",
        worst_ratio * 100.0
    );
}

/// The relief a straight Grain-sourced stroke lays down under `mapping` — the shared fixture for the
/// two questions below (is there relief at all, and does it corrugate).
fn grain_relief_stroke(mapping: ph2d_painter_brush::TextureMapping) -> (Vec<f32>, usize) {
    use ph2d_painter_brush::TextureKind;
    let size = 320u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let mut b = BrushSpec {
        radius_px: 40.0, // spacing = 0.10 × 2 × 40 = 8 px exactly
        color: [0.9, 0.1, 0.1],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.7,
        impasto_source: DepthSource::Grain,
        impasto_smoothing: 0.0,
        ..Default::default()
    };
    b.texture.kind = TextureKind::Noise;
    b.texture.mapping = mapping;
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    let step = b.dab_spacing_px().round().max(2.0) as usize;
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([280.0, 160.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([280.0, 160.0], PointerPhase::Up));
    let h = relief(&t);
    // The height straight down the centreline of the stroke.
    (
        (70..270).map(|x| h[(160 * size + x) as usize]).collect(),
        step,
    )
}

/// Peak relief of that stroke — used to keep the ratio below from being computed over an empty field.
fn relief_peak(mapping: ph2d_painter_brush::TextureMapping) -> f32 {
    grain_relief_stroke(mapping)
        .0
        .iter()
        .fold(0.0f32, |m, &v| m.max(v.abs()))
}

/// How much of the height variance along a straight stroke is a pure function of the DAB PHASE
/// (`x mod spacing`). 1.0 = the relief is corrugated at exactly the dab pitch; ~0 = it is not.
fn dab_phase_variance(mapping: ph2d_painter_brush::TextureMapping) -> f32 {
    let (line, step) = grain_relief_stroke(mapping);
    let mean = line.iter().sum::<f32>() / line.len() as f32;
    let total: f32 = line.iter().map(|x| (x - mean) * (x - mean)).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut explained = 0.0f32;
    for ph in 0..step {
        let bin: Vec<f32> = line.iter().skip(ph).step_by(step).copied().collect();
        if bin.is_empty() {
            continue;
        }
        let bm = bin.iter().sum::<f32>() / bin.len() as f32;
        explained += bin.len() as f32 * (bm - mean) * (bm - mean);
    }
    explained / total
}

#[test]
fn impasto_grain_relief_corrugates_unless_the_grain_is_anchored_to_the_canvas() {
    // The ribs Enio saw across every stroke (2026-07-12), quantified — and NOT an engine bug, which
    // is exactly why it needs a gate rather than a fix.
    //
    // A **ViewPlane** grain is DAB-relative: each dab stamps the identical noise in its own frame. At
    // 10% spacing the dabs overlap tenfold, so the height envelope repeats at the dab pitch and the
    // relief corrugates across the travel. Under the dome kernel that was ~100% of the along-stroke
    // variance; the body curve attenuates it (every dab whose SOLID band covers the pixel bids full
    // body, so the envelope keeps more of the grain and less of the silhouette's phase) — measured
    // **0.70** now. Still corduroy, still the wrong arming for `DepthSource::Grain`.
    //
    // Anchor the grain to the CANVAS (Tiled) and consecutive dabs bite different noise: **~0.02** —
    // the marks read as bristle streaks ALONG the path, which is what a loaded brush leaves. The
    // smoke arms it that way; this gate is here so the day someone "simplifies" the mapping, the
    // corduroy does not come back silently.
    use ph2d_painter_brush::TextureMapping;
    // ANTI-VACUITY, twice. (1) `dab_phase_variance` divides by the total variance — a zero relief
    // returns 0 and the "must not corrugate" assertions pass while proving nothing (this gate shipped
    // green in exactly that state for one commit, pre-`GRAIN_GROOVE`). (2) A relief SATURATED flat by
    // the envelope-of-many-bids would also pass them — so the centreline must still carry real groove
    // texture, not a ceiling.
    assert!(
        relief_peak(TextureMapping::ViewPlane) > 0.15 && relief_peak(TextureMapping::Tiled) > 0.15,
        "sanity: both configurations actually lay down relief — else the ratios below are vacuous"
    );
    let (line, _) = grain_relief_stroke(TextureMapping::Tiled);
    let (lo, hi) = line
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    assert!(
        hi - lo > 0.02,
        "sanity: the canvas-anchored grooves are alive on the centreline (spread {:.3}) — a saturated \
         ceiling would sit at ~0.000 and make the phase ratios below vacuous. (Measured 0.043 — the \
         same under the dome kernel, since on the spine w = 1 and body(1) = 1: the grain-coverage fold \
         compresses Noise more than a first guess says.)",
        hi - lo
    );
    let dab_relative = dab_phase_variance(TextureMapping::ViewPlane);
    let canvas_anchored = dab_phase_variance(TextureMapping::Tiled);
    assert!(
        dab_relative > 0.5,
        "a dab-relative grain DOES still corrugate at the dab pitch — that is the physics this gate \
         records (got {dab_relative:.2}; ~1.0 under the dome kernel, 0.70 under the body curve)"
    );
    assert!(
        canvas_anchored < 0.2,
        "a canvas-anchored grain must NOT corrugate: consecutive dabs bite different noise \
         (got {canvas_anchored:.2}, expected ~0.02)"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn flat_probe_exact_smoke_arming() {
    // Enio's second smoke came out completely FLAT. Reproduce the smoke's arming EXACTLY — through the
    // same public setters, in the same order, not by hand-building a BrushSpec — and report what the
    // relief and the shading actually are. (Hand-building the spec is how a probe agrees with itself and
    // misses the product.)
    let size = 240u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(40.0);
    t.set_brush_texture_kind(ph2d_painter_brush::TextureKind::Noise.to_u8());
    t.set_brush_texture_mapping(ph2d_painter_brush::TextureMapping::Tiled.to_u8());
    t.toggle_brush_impasto();
    t.set_brush_impasto_depth(0.7);
    t.set_brush_impasto_source(DepthSource::Grain.to_u8());
    t.set_brush_impasto_smoothing(0.15);

    let b = t.paint.brush;
    println!(
        "spec: impasto={} depth={} source={:?} grain_kind={:?} grain_mapping={:?} grain_active={} \
         radius={}",
        b.impasto,
        b.impasto_depth,
        b.impasto_source,
        b.texture.kind,
        b.texture.mapping,
        b.texture.is_active(),
        b.radius_px
    );
    println!(
        "deposits_height={} deposits_color={}",
        b.deposits_height(),
        b.deposits_color()
    );

    t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([140.0, 120.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 180.0], PointerPhase::Up));

    let h = relief(&t);
    let hmax = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    let nonzero = h.iter().filter(|v| v.abs() > 1e-6).count();
    println!("relief: {} pixels, max |h| = {hmax:.4}", nonzero);

    println!("impasto_visible = {}", t.impasto_visible());
    t.invalidate_composite();
    let shaded = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let plain = lit(&mut t);
    let mut worst = 0i32;
    let mut moved = 0u32;
    for i in (0..plain.len()).step_by(4) {
        let d = (i32::from(shaded[i + 1]) - i32::from(plain[i + 1])).abs();
        if d > 2 {
            moved += 1;
        }
        worst = worst.max(d);
    }
    println!("light: {moved} pixels moved >2 levels, worst {worst} levels");

    // Compare against the mapping that DID show relief, and against Grain off entirely — the height is
    // `depth × coverage × w × g`, so a weak `g` alone can gut it.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    for (name, kind, mapping, tex_size) in [
        (
            "Grain OFF (Uniform src)",
            TextureKind::None,
            TextureMapping::Tiled,
            1.0f32,
        ),
        (
            "Noise ViewPlane",
            TextureKind::Noise,
            TextureMapping::ViewPlane,
            1.0,
        ),
        (
            "Noise Tiled size 1.0",
            TextureKind::Noise,
            TextureMapping::Tiled,
            1.0,
        ),
        (
            "Noise Tiled size 0.2",
            TextureKind::Noise,
            TextureMapping::Tiled,
            0.2,
        ),
        (
            "Noise Tiled size 0.1",
            TextureKind::Noise,
            TextureMapping::Tiled,
            0.1,
        ),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: if matches!(kind, TextureKind::None) {
                DepthSource::Uniform
            } else {
                DepthSource::Grain
            },
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = kind;
        b.texture.mapping = mapping;
        b.texture.size = [tex_size, tex_size];
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([140.0, 120.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([90.0, 180.0], PointerPhase::Up));
        let h = relief(&t);
        let hmax = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        // Steepest local slope — what the light actually reads.
        let mut slope = 0.0f32;
        for y in 1..size as usize - 1 {
            for x in 1..size as usize - 1 {
                let g = (h[y * size as usize + x + 1] - h[y * size as usize + x - 1]).abs() * 0.5;
                slope = slope.max(g);
            }
        }
        t.invalidate_composite();
        let sh = lit(&mut t);
        t.paint.impasto_show = false;
        t.invalidate_composite();
        let pl = lit(&mut t);
        let worst = (0..pl.len())
            .step_by(4)
            .map(|i| (i32::from(sh[i + 1]) - i32::from(pl[i + 1])).abs())
            .max()
            .unwrap_or(0);
        println!(
            "  {name:24} max|h|={hmax:.3}  steepest slope={slope:.4}/px  light moves up to {worst} levels"
        );
    }
}

#[test]
fn impasto_grain_textures_the_body_instead_of_removing_it() {
    // Enio's second smoke came out FLAT (2026-07-12), and this was half the reason. The funnel is
    // `h = depth · coverage · w · g`, so `DepthSource::Grain` was MULTIPLYING the body by the grain —
    // and a Noise grain's samples average well under half. The artist asked for Depth 0.7 and got 0.21:
    // a bristle brush laying a third of the paint it should. A tuft does not deposit a third of the
    // paint; it deposits the paint, with GROOVES in it.
    //
    // (The other half was `SLOPE_GAIN`, which I had picked by taste at 8 — a real stroke's steepest
    // slope is 0.026/px, so it tilted the normal 6° and lit nothing. Both were mine. `SLOPE_GAIN` has
    // since been retired for the physical `DEPTH_UNIT_PX` — impasto_light.rs tells that story.)
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let body = |grain: bool| -> (f32, f32) {
        let size = 240u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: if grain {
                DepthSource::Grain
            } else {
                DepthSource::Uniform
            },
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        if grain {
            b.texture.kind = TextureKind::Noise;
            b.texture.mapping = TextureMapping::Tiled;
        }
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([140.0, 120.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([90.0, 180.0], PointerPhase::Up));
        let h = relief(&t);
        let peak = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        // Spread of the relief INSIDE the stroke — the striations. A body with no variation is not a
        // bristle mark; a gate that only checked the peak would happily accept flat paint.
        let inside: Vec<f32> = h.iter().copied().filter(|v| v.abs() > 0.05).collect();
        let (lo, hi) = inside
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        (peak, hi - lo)
    };
    let (uniform_peak, _) = body(false);
    let (grain_peak, grain_spread) = body(true);
    assert!(
        (uniform_peak - 0.7).abs() < 0.02,
        "sanity: Uniform lays the Depth the artist asked for ({uniform_peak:.2})"
    );
    assert!(
        grain_peak > uniform_peak * 0.5,
        "the Grain must TEXTURE the body, not remove it: peak {grain_peak:.2} vs Uniform's \
         {uniform_peak:.2} — the artist asked for thick paint and got a film"
    );
    assert!(
        grain_spread > 0.1,
        "...and it must still carry striations ({grain_spread:.2}) — a smooth body is not a bristle mark"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn spacing_probe_relief_must_not_depend_on_dab_pitch() {
    // Enio's experiment (2026-07-12): the SAME brush at spacing 0.1 / 0.05 / 0.01 produces three
    // visibly different reliefs — heavy corduroy, mild, then a smooth tube. That cannot be right: the
    // thickness of paint is a property of the brush and the path, not of how finely the engine chose to
    // sample it.
    //
    // Thesis: the envelope is a `max` of DISCRETE domes. Between two dab centres the distance to either
    // grows, so the max DIPS — and the dip is deepest where the falloff is steep, i.e. on the FLANKS.
    // (My earlier probe sampled the CENTRELINE, where the falloff sits on its plateau and barely dips —
    // which is exactly why it reported "no corrugation" while the screen was corrugated.)
    let size = 300u32;
    let scan = |spacing: f32, off_axis: i32| -> (f32, f32) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Uniform, // NO grain — isolate the geometry
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        let h = relief(&t);
        let y = (150 + off_axis) as usize;
        let line: Vec<f32> = (60..240).map(|x| h[y * size as usize + x]).collect();
        // Ripple: peak-to-peak of the height along a line that SHOULD be perfectly flat (a straight
        // stroke of constant width), and the steepest along-path slope (what the light reads).
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        let ripple = hi - lo;
        let slope = line
            .windows(3)
            .map(|w| ((w[2] - w[0]) * 0.5).abs())
            .fold(0.0f32, f32::max);
        (ripple, slope)
    };
    for spacing in [0.10f32, 0.05, 0.01] {
        let (r_axis, s_axis) = scan(spacing, 0);
        let (r_flank, s_flank) = scan(spacing, 30);
        println!(
            "UNIFORM  spacing {spacing:.2} ({:>4.1} px)  centre: ripple {r_axis:.4} slope {s_axis:.4}  \
             flank: ripple {r_flank:.4} slope {s_flank:.4}",
            spacing * 2.0 * 40.0
        );
    }
    // Now the SMOKE's actual configuration — Grain source over a canvas-anchored noise. If the ribs are
    // here and not in Uniform, the geometry is not the culprit.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let scan_grain = |spacing: f32, mapping: TextureMapping, off_axis: i32| -> (f32, f32) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = mapping;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        let h = relief(&t);
        let y = (150 + off_axis) as usize;
        let line: Vec<f32> = (60..240).map(|x| h[y * size as usize + x]).collect();
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        let slope = line
            .windows(3)
            .map(|w| ((w[2] - w[0]) * 0.5).abs())
            .fold(0.0f32, f32::max);
        (hi - lo, slope)
    };
    for spacing in [0.10f32, 0.05, 0.01] {
        for (name, m) in [
            ("Grain/Tiled    ", TextureMapping::Tiled),
            ("Grain/ViewPlane", TextureMapping::ViewPlane),
        ] {
            let (r, sl) = scan_grain(spacing, m, 0);
            let (rf, slf) = scan_grain(spacing, m, 30);
            println!(
                "{name} spacing {spacing:.2}  centre: ripple {r:.4} slope {sl:.4}  flank: ripple {rf:.4} slope {slf:.4}"
            );
        }
    }
}

#[test]
fn impasto_relief_is_the_same_at_any_dab_spacing() {
    // Enio's experiment, 2026-07-12, and one of the best bug reports this line got: the same brush at
    // spacing 0.1 / 0.05 / 0.01 produced heavy corduroy, mild corduroy, and a smooth tube. Three
    // different paintings from one brush.
    //
    // The thickness of paint is a property of the brush and the PATH. It cannot depend on how finely the
    // engine chose to sample that path — that is an implementation detail leaking onto the canvas.
    //
    // The cause was geometric, not the grain: the envelope was a `max` of discrete DISCS, and between
    // two centres the distance to either grows, so the maximum DIPS. Wider spacing, deeper scallops.
    // (My first probe measured the centreline of a Grain stroke and reported "no corrugation" — it was
    // looking at the wrong thing. The geometry shows up with the grain OFF.)
    //
    // Now each dab sweeps its body BACK along its own heading, so the union is the stroke's true
    // distance field. Measured before: ripple 0.0148 / 0.0025 / 0.0000. After: 0.0000 at every spacing.
    let size = 300u32;
    let stroke = |spacing: f32| -> Vec<f32> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Uniform, // grain OFF — this is about the GEOMETRY
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        relief(&t)
    };
    // A straight stroke of constant width must leave a relief that is FLAT along its length. Any ripple
    // here is the dab pitch printing itself onto the paint.
    let ripple = |h: &[f32], row: usize| {
        let line: Vec<f32> = (60..240).map(|x| h[row * size as usize + x]).collect();
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        hi - lo
    };
    let coarse = stroke(0.10);
    let fine = stroke(0.01);
    assert!(
        coarse.iter().fold(0.0f32, |m, &v| m.max(v)) > 0.6,
        "sanity: the coarse stroke really did lay down a body"
    );
    for row in [150usize, 180] {
        // 180 = 30 px off the axis: the flank, where the falloff is steep.
        assert!(
            ripple(&coarse, row) < 0.002,
            "at spacing 0.10 the relief ripples {:.4} along a stroke that should be flat — the dab \
             pitch is printing itself onto the paint (row {row})",
            ripple(&coarse, row)
        );
    }
    // And the two spacings must agree: same brush, same path, same paint.
    let peak = |h: &[f32]| h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    let (pc, pf) = (peak(&coarse), peak(&fine));
    assert!(
        (pc - pf).abs() < 0.02,
        "coarse and fine sampling of the SAME stroke must deposit the same thickness ({pc:.3} vs {pf:.3})"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn sweep_probe_jitter_spacing() {
    // The sweep reaches back exactly one nominal pitch. Jitter Spacing scatters the dabs, so a gap can
    // open wider than that — does the corrugation come back?
    let size = 300u32;
    for js in [0.0f32, 0.5, 1.0] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 40.0,
            spacing: 0.10,
            jitter_spacing: js,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Uniform,
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        let h = relief(&t);
        let line: Vec<f32> = (60..240).map(|x| h[150 * size as usize + x]).collect();
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        println!("jitter_spacing {js:.1} → ripple {:.4}", hi - lo);
    }
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn spacing_probe_curved_full_field() {
    // The strokes STILL differ with spacing on screen. My gate measured a straight stroke's peak and
    // ripple — which is not the same as "the two paintings are the same painting". Compare the whole
    // field, on a CURVE, and split it: is the difference in the COLOUR, in the RELIEF, or in the LIGHT?
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 260u32;
    let run = |spacing: f32| -> (Vec<u8>, Vec<f32>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        // A curve, like the smoke's S.
        t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
        for p in [[120.0, 90.0], [90.0, 140.0], [130.0, 190.0], [110.0, 225.0]] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([110.0, 225.0], PointerPhase::Up));
        let canvas = (*t.canvas_rgba).clone();
        let h = relief(&t);
        let (comp, _, _) = t.take_preview_arc().expect("preview");
        (canvas, h, (*comp).clone())
    };
    let (c_coarse, h_coarse, l_coarse) = run(0.10);
    let (c_fine, h_fine, l_fine) = run(0.01);
    let _ = (&h_coarse, &h_fine);

    let du8 = |a: &[u8], b: &[u8]| {
        let mut worst = 0i32;
        let mut n = 0u32;
        for i in (0..a.len()).step_by(4) {
            let d = (0..3)
                .map(|c| (i32::from(a[i + c]) - i32::from(b[i + c])).abs())
                .max()
                .unwrap_or(0);
            if d > 8 {
                n += 1;
            }
            worst = worst.max(d);
        }
        (n, worst)
    };
    let (cn, cw) = du8(&c_coarse, &c_fine);
    let (ln, lw) = du8(&l_coarse, &l_fine);
    let hw = h_coarse
        .iter()
        .zip(h_fine.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    let hn = h_coarse
        .iter()
        .zip(h_fine.iter())
        .filter(|(a, b)| (*a - *b).abs() > 0.05)
        .count();
    println!("COLOUR  (canvas_rgba): {cn} px differ >8 levels, worst {cw}");
    println!("RELIEF  (height):      {hn} px differ >0.05,     worst {hw:.3}");
    println!("LIGHT   (composite):   {ln} px differ >8 levels, worst {lw}");

    // Is the colour difference the engine's ratified "spacing changes deposit density" — the thing
    // "Adjust Strength for Spacing" exists to normalise, and which Enio turned OFF by default in
    // 2026-06-24? Turn it back on and see whether the three strokes converge.
    let run_att = |spacing: f32| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: true, // <- the only change
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
        for p in [[120.0, 90.0], [90.0, 140.0], [130.0, 190.0], [110.0, 225.0]] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([110.0, 225.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let (an, aw) = du8(&run_att(0.10), &run_att(0.01));
    println!("COLOUR with Adjust Strength ON: {an} px differ >8 levels, worst {aw}");
}

#[test]
fn impasto_depth_and_smoothing_are_live_on_the_stroke_already_painted() {
    // Enio 2026-07-12: "Depth e Smooth devem atualizar em tempo real após o traço ser feito como as
    // outras propriedades fazem." An artist lays a stroke and then dials the thickness in while LOOKING
    // at it — a knob that only affects the next stroke is a knob you have to guess with.
    //
    // The bar is not "something changes". It is: dragging Depth after the fact must land on EXACTLY the
    // relief you would have got by painting with that Depth from the start. A live edit that merely
    // approximates the real thing is a second, silently-divergent code path — and this line has already
    // paid for one of those.
    let paint = |depth: f32, smoothing: f32, retune: Option<(f32, f32)>| -> Vec<f32> {
        let mut t = impasto_canvas(60);
        let mut b = t.paint.brush;
        b.radius_px = 8.0;
        b.impasto_depth = depth;
        b.impasto_smoothing = smoothing;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
        if let Some((d, s)) = retune {
            // The stroke is DONE. Now move the sliders, exactly as the panel would.
            t.set_brush_impasto_depth(d);
            t.set_brush_impasto_smoothing(s);
        }
        relief(&t)
    };
    // Painted at 0.3/0.0, then re-tuned to 0.8/0.6 — versus painted at 0.8/0.6 in the first place.
    let retuned = paint(0.3, 0.0, Some((0.8, 0.6)));
    let native = paint(0.8, 0.6, None);
    assert!(
        native.iter().fold(0.0f32, |m, &v| m.max(v)) > 0.3,
        "sanity: the reference stroke has a real body"
    );
    let worst = retuned
        .iter()
        .zip(native.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-5,
        "re-tuning Depth/Smoothing after the stroke must land on the same relief as painting with them \
         from the start (worst divergence {worst})"
    );

    // Carving live, too — Depth is signed, and flipping it must flip the paint already down.
    let carved = paint(0.5, 0.0, Some((-0.5, 0.0)));
    assert!(
        carved.iter().any(|&v| v < -0.1),
        "dragging Depth negative carves the stroke that is already on the canvas"
    );

    // ...but a SECOND stroke ends the live edit: only the last one is re-derivable, and re-tuning must
    // never resurrect or rescale the ones before it.
    let mut t = impasto_canvas(60);
    let mut b = t.paint.brush;
    b.radius_px = 8.0;
    b.impasto_depth = 0.5;
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([15.0, 15.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([15.0, 15.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([45.0, 45.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([45.0, 45.0], PointerPhase::Up));
    let first_before = relief(&t)[(15 * 60 + 15) as usize];
    // SANITY, and it is load-bearing: committing the second stroke must not have erased the first. I
    // wrote this gate without it, and a mutation that made the live buffer FORGET its ground sailed
    // straight through — because it destroyed the first stroke's relief BEFORE the reference was read,
    // so the test compared zero against zero and approved. Read the reference, then check it is real.
    assert!(
        first_before > 0.2,
        "the first stroke is still on the layer after the second one commits ({first_before})"
    );
    t.set_brush_impasto_depth(1.0);
    let h = relief(&t);
    assert!(
        (h[(15 * 60 + 15) as usize] - first_before).abs() < 1e-5,
        "the FIRST stroke is committed paint — a later Depth drag must not reach back and re-sculpt it"
    );
    assert!(
        h[(45 * 60 + 45) as usize] > first_before * 1.5,
        "...while the last stroke, the live one, does follow the slider"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn halo_probe_translucent_edge() {
    // Enio 2026-07-12, white canvas vs black: a whitish halo rims the LIT flank on white and vanishes on
    // black. The light is a MULTIPLY on the composite — and at the stroke's translucent edge the
    // composite is mostly PAPER. So the pass is brightening the paper showing THROUGH the paint, and on
    // white paper `×1.65` bleaches a pale pink straight to white. Bucket the pixels by how much paint
    // they carry and see where the shift lands.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 200u32;
    let run = |paper: u8| -> (Vec<u8>, Vec<u8>, Vec<f32>) {
        let mut t = PainterTool::default();
        t.set_source(vec![paper; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.on_canvas_pointer(cp([70.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([110.0, 100.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([80.0, 160.0], PointerPhase::Up));
        let canvas = (*t.canvas_rgba).clone();
        let h = relief(&t);
        t.paint.impasto_show = true;
        t.invalidate_composite();
        let lit_px = (*t.take_preview_arc().unwrap().0).clone();
        t.paint.impasto_show = false;
        t.invalidate_composite();
        let plain = (*t.take_preview_arc().unwrap().0).clone();
        let _ = canvas;
        (lit_px, plain, h)
    };
    for (name, paper) in [("WHITE paper", 255u8), ("BLACK paper", 0u8)] {
        let (litp, plain, h) = run(paper);
        // "Ink" = how far the pixel is from bare paper: 0 = untouched, 1 = fully covered.
        let core = plain
            .chunks_exact(4)
            .map(|p| (i32::from(p[0]) - i32::from(p[1])).abs())
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let mut buckets = [(0u32, 0i32); 5]; // 0-20 / 20-40 / 40-60 / 60-80 / 80-100 % ink
        for i in (0..plain.len()).step_by(4) {
            let ink = (i32::from(plain[i]) - i32::from(plain[i + 1])).abs() as f32 / core;
            if ink <= 0.02 {
                continue;
            }
            let b = ((ink * 5.0) as usize).min(4);
            let shift = (i32::from(litp[i + 1]) - i32::from(plain[i + 1])).abs();
            buckets[b].0 += 1;
            buckets[b].1 = buckets[b].1.max(shift);
        }
        let hmax = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        println!("{name} (relief peak {hmax:.2}):");
        for (i, (n, worst)) in buckets.iter().enumerate() {
            println!(
                "   ink {:>3}-{:>3}%: {n:>6} px, light shifts up to {worst:>3} levels",
                i * 20,
                (i + 1) * 20
            );
        }
    }
}

#[test]
fn impasto_light_shades_the_paint_not_the_paper_showing_through_it() {
    // Enio, 2026-07-12, two photographs: the same strokes on a WHITE canvas and on a BLACK one. On
    // white, a bleached halo rimmed every stroke. On black it simply was not there — the tell. The pass
    // MULTIPLIES the composited pixel, and at a stroke's translucent edge that pixel is mostly PAPER
    // seen through the paint; shading it in full shades the paper, and on white that bleaches.
    //
    // The gate is stated as the property that is INDEFENSIBLE, and no more: **paint with no body gets
    // no light — not one byte — however the light is dialled.** Everything else the artist can judge
    // with their eyes; this they cannot, because a halo hides exactly where the paint is faintest.
    //
    // What this deliberately does NOT assert any more (it did, and it was wrong): that a lit edge keeps
    // its saturation. Under the artist's defaults (Depth 1, Body 0 — the relief follows the falloff all
    // the way out) a translucent edge DOES have relief, so the light legitimately brightens it; and
    // brightening a pixel whose pigment channel is already at the ceiling costs saturation, in paint as
    // in physics. Measured, it lands at 21% of the ink — and it is not a defect, it is the light. Pin
    // the paper instead: that line is absolute.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 200u32;
    let render = |shine: f32, show: bool| -> (Vec<u8>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size); // WHITE paper: the hard case
        let mut b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_source: DepthSource::Grain, // per-texel slopes: the harshest case for the weight
            ..Default::default()                // …and otherwise the ARTIST's defaults, on purpose
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.paint.impasto_shine = shine;
        t.on_canvas_pointer(cp([70.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([110.0, 100.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([80.0, 160.0], PointerPhase::Up));
        t.paint.impasto_show = show;
        t.invalidate_composite();
        let img = lit(&mut t);
        let active = t.layers.active().expect("a layer");
        let cov = t.covers.get(&active).cloned().unwrap_or_default();
        (img, cov)
    };
    // The light at its LOUDEST — full Shine, the artist's Depth/Body/Angle/Elevation.
    let (loud, cov) = render(1.0, true);
    let (unlit, _) = render(1.0, false);

    let w_tail = ph2d_painter_brush::height::W_TAIL;
    let (mut bodyless, mut drifted, mut worst) = (0u32, 0u32, 0i32);
    for p in 0..(size * size) as usize {
        if f32::from(cov[p]) / 255.0 >= w_tail {
            continue; // paint with a body — the light SHOULD model it
        }
        bodyless += 1;
        for c in 0..3 {
            let d = i32::from(loud[p * 4 + c]) - i32::from(unlit[p * 4 + c]);
            if d != 0 {
                drifted += 1;
            }
            worst = worst.max(d.abs());
        }
    }
    assert!(
        bodyless > 30_000,
        "sanity: most of this canvas is bare paper or a faint stain ({bodyless} px)"
    );
    assert_eq!(
        drifted, 0,
        "the light moved {drifted} channels of paint that has NO body (worst {worst} levels) — that is \
         the white halo: the pass shading the paper seen through the paint. It vanished on Enio's black \
         canvas because there was nothing white to bleach, which is how we know it is the paper and not \
         the pigment."
    );
}

#[test]
fn impasto_soft_stroke_reads_as_a_body_with_an_edge() {
    // THE appearance gate of the Fase 4 redesign (plan §10, T4.5) — derived from the DEFINITION of
    // thick paint, not from the shader ([[feedback_oracle_must_model_appearance_not_implementation]]):
    // a body of paint has a level top, a wall at its edge, and a stain that carries no relief. The
    // DEFAULT brush (hardness 0, Smooth — the one Enio actually smokes with) must read that way.
    //
    // Every threshold here was RED under the dome kernel, by the opening measurements (plan §10):
    // the dome curved everywhere (no plateau, tail relief 0.07, shading smeared over 62% of the
    // stroke's width with its weak peak — 7.3 levels — at 31%, nothing at the edge).
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.radius_px = 40.0;
    b.impasto_depth = 0.7;
    b.impasto_body = 1.0; // this gate IS the body curve (the artist's default is the round profile)
    b.impasto_source = DepthSource::Uniform; // isolate the body curve — grain is another gate
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.paint.impasto_light_angle_deg = 90; // straight across a horizontal stroke
    t.paint.impasto_light_elev_deg = 45;
    t.paint.impasto_shine = 0.0; // the diffuse modelling is the claim; the glint has its own gates
    t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
    for i in 1..=8 {
        t.on_canvas_pointer(cp(
            [40.0 + 10.0 * f32::from(i as u8), 80.0],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));

    let h = relief(&t);
    let img = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let base = lit(&mut t);
    let active = t.layers.active().unwrap();
    let cov = t.covers.get(&active).cloned().unwrap_or_default();

    // Cross-section at mid-stroke, from the spine outward.
    let x = 80u32;
    let lum = |img: &[u8], y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    let rows: Vec<(u32, u8, f32, f32)> = (80..160u32)
        .map(|y| {
            let i = (y * size + x) as usize;
            (y - 80, cov[i], h[i], lum(&img, y) - lum(&base, y))
        })
        .collect();
    let painted: Vec<&(u32, u8, f32, f32)> = rows.iter().filter(|r| r.1 > 4).collect();
    let half_width = painted.last().expect("a painted cross-section").0;
    let spine_h = rows[0].2;
    assert!(spine_h > 0.6, "sanity: the stroke laid its full depth");

    // 1. The top is a PLATEAU: at 25% of the half-width the paint is as thick as at the spine.
    //    (Dome: ~0.86 of the spine there — curved from the very centre.)
    let at = |frac: f32| {
        let d = (frac * half_width as f32) as u32;
        rows.iter().find(|r| r.0 == d).expect("inside the canvas")
    };
    assert!(
        at(0.25).2 >= 0.98 * spine_h,
        "the interior is a level film, not a dome (h {} at 25% vs spine {spine_h})",
        at(0.25).2
    );

    // 2. The stain carries NO body: past 85% of the painted half-width the relief is zero.
    //    (Dome: 0.065 there — relief over near-invisible paint, the halo's raw material.)
    assert!(
        at(0.85).2 == 0.0 && at(0.95).2 == 0.0,
        "the translucent rim is FLAT ({} / {})",
        at(0.85).2,
        at(0.95).2
    );

    // 3. The light lives on the WALL: the response is concentrated (≤ 40% of the painted width moves
    //    ≥ 3 levels; the dome smeared 62%), and its peak is a real edge, not a haze (≥ 8 levels;
    //    the dome managed 7.3 with everything on).
    let visible = painted.iter().filter(|r| r.3.abs() >= 3.0).count();
    let concentration = visible as f32 / painted.len().max(1) as f32;
    assert!(
        concentration <= 0.40,
        "the shading is concentrated at the edge, not smeared over the stroke ({:.0}% of the width)",
        concentration * 100.0
    );
    let peak = rows
        .iter()
        .max_by(|a, b| a.3.abs().partial_cmp(&b.3.abs()).unwrap())
        .unwrap();
    assert!(
        peak.3.abs() >= 8.0,
        "the wall actually catches the light ({:.1} levels)",
        peak.3.abs()
    );
    // …and the peak sits ON the wall (where the height is falling), not on the plateau.
    let peak_h = rows.iter().find(|r| r.0 == peak.0).unwrap().2;
    assert!(
        peak_h < 0.95 * spine_h && peak_h > 0.0,
        "the brightest response is on the wall itself (h {peak_h} at the peak, spine {spine_h})"
    );
}

#[test]
fn impasto_strokes_pile_up_only_to_the_glass() {
    // T4.2 — Corel Painter documents the same limit: accumulated impasto "top[s] out and appear[s]
    // as if the strokes are pressed against glass". Strokes ADD (a second stroke genuinely piles
    // more on — `impasto_one_stroke_is_one_thickness_but_two_strokes_add` pins that), but not
    // forever: without the ceiling, five loads make a mesa whose walls dwarf every brush-mark on
    // top of it, which is the other road back to unreadable relief. RED without the clamp: h = 3.
    let mut t = impasto_canvas(40);
    let mut b = t.paint.brush;
    b.impasto_depth = 1.0;
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    for _ in 0..3 {
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    }
    let h = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        (h - 2.0).abs() < 1e-5,
        "three full loads stop at the glass (two): got {h}"
    );
}

#[test]
fn impasto_body_zero_obeys_the_falloff() {
    // Enio's smoke on the Fase 4 body (2026-07-12): "parece ter perdido a capacidade de obedecer
    // toda a suavidade do falloff — não consigo relevos perfeitamente arredondados como antes."
    // He is right: the body curve crushed EVERY profile to plateau + wall, and the plan §0 promise
    // ("a Shape-Tone ramp vira escultura") died with it. The state of the art ships both schools
    // behind a control (PS Technique Smooth↔Chisel; Blender Draw vs Layer brushes) — so does the
    // brush now: **Body = 0** must hand the cross-section back to the silhouette, exactly.
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.radius_px = 40.0;
    b.impasto_depth = 0.7;
    b.impasto_source = DepthSource::Uniform;
    b.impasto_smoothing = 0.0; // the raw deposit IS the claim — no settling on top
    b.impasto_body = 0.0; // the round school
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
    for i in 1..=8 {
        t.on_canvas_pointer(cp(
            [40.0 + 10.0 * f32::from(i as u8), 80.0],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));

    let h = relief(&t);
    let at = |d: u32| h[((80 + d) * size + 80) as usize];
    let spine = at(0);
    assert!(spine > 0.6, "sanity: full depth on the spine");
    // A dome, not a mesa: the height falls from the very centre (no plateau)...
    assert!(
        at(10) < 0.97 * spine,
        "no plateau — the falloff's curve starts at the spine ({} vs {spine})",
        at(10)
    );
    // ...keeps falling monotonically...
    assert!(
        at(10) > at(20) && at(20) > at(30),
        "monotone rounded flank ({} > {} > {})",
        at(10),
        at(20),
        at(30)
    );
    // ...and the soft tail CARRIES relief again (the wall is gone; body 1 zeroes this pixel).
    assert!(
        at(30) > 0.01,
        "the falloff's soft tail sculpts the relief at Body 0 ({})",
        at(30)
    );
    // And the round school must not resurrect the halo: the tail has height now, but the light
    // still ignores paint that is not there — bare-white pixels do not move.
    t.paint.impasto_light_angle_deg = 90;
    t.paint.impasto_light_elev_deg = 45;
    t.paint.impasto_shine = 0.0;
    t.invalidate_composite();
    let img = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let base = lit(&mut t);
    let mut worst = 0i32;
    for i in (0..base.len()).step_by(4) {
        if 255 - i32::from(base[i + 1]) > 10 {
            continue; // real paint — allowed to shade
        }
        worst = worst.max((i32::from(img[i + 1]) - i32::from(base[i + 1])).abs());
    }
    assert!(
        worst <= 8,
        "rounded relief must not shade the near-invisible tail (worst drift {worst} levels)"
    );
}

/// Paint one stroke on a fresh canvas with `arm` applied to the brush, then apply `edit` through the
/// PUBLIC setters (the panel's own route) and return the relief. With `edit` a no-op this is simply
/// "what the brush painted".
fn impasto_stroke_then_edit(
    arm: impl FnOnce(&mut BrushSpec),
    edit: impl FnOnce(&mut PainterTool),
) -> Vec<f32> {
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 120u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 24.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.impasto_depth = 0.6;
    b.impasto_body = 1.0;
    b.impasto_smoothing = 0.0;
    b.impasto_source = DepthSource::Uniform;
    b.texture.kind = TextureKind::Noise; // a grain to carve, so Depth Source has something to say
    b.texture.mapping = TextureMapping::Tiled;
    arm(&mut b);
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 70.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));
    edit(&mut t);
    relief(&t)
}

#[test]
fn impasto_every_body_knob_edits_the_last_stroke_live() {
    // Enio, 2026-07-12: "coloque todos os parâmetros vivos em tempo real para ajustes depois do traço."
    //
    // THE claim, stated so it can only be true one way: for every knob in the Body card, dialling it
    // AFTER the stroke gives the same relief as having painted the stroke with it from the start. That
    // is only possible because the stroke stores its INGREDIENTS (the paint it laid + the grain it
    // sampled) and the relief is a pure function of them — bake anything into the height that
    // `derive_height` cannot see and this goes red, which is exactly what Body and Depth Source did
    // before this gate existed (they were dead after pen-up, and the panel said nothing).
    type Arm = fn(&mut BrushSpec);
    type Edit = fn(&mut PainterTool);
    let cases: [(&str, Arm, Edit); 4] = [
        (
            "Depth",
            |b| b.impasto_depth = -0.9,
            |t| t.set_brush_impasto_depth(-0.9),
        ),
        (
            "Body",
            |b| b.impasto_body = 0.0,
            |t| t.set_brush_impasto_body(0.0),
        ),
        (
            "Depth Source",
            |b| b.impasto_source = DepthSource::Grain,
            |t| t.set_brush_impasto_source(DepthSource::Grain.to_u8()),
        ),
        (
            "Smoothing",
            |b| b.impasto_smoothing = 0.8,
            |t| t.set_brush_impasto_smoothing(0.8),
        ),
    ];
    let baseline = impasto_stroke_then_edit(|_| {}, |_| {});
    for (name, arm, edit) in cases {
        let painted_with_it = impasto_stroke_then_edit(arm, |_| {});
        let edited_after = impasto_stroke_then_edit(|_| {}, edit);
        // The knob must actually DO something (else the equality below is vacuous — the trap that let
        // a dead knob ship green once already).
        let moved = painted_with_it
            .iter()
            .zip(baseline.iter())
            .filter(|(a, b)| (*a - *b).abs() > 1e-4)
            .count();
        assert!(
            moved > 200,
            "{name}: the knob changes the deposit at all ({moved} px moved) — else this gate is vacuous"
        );
        assert_eq!(
            painted_with_it.len(),
            edited_after.len(),
            "{name}: same canvas"
        );
        let worst = painted_with_it
            .iter()
            .zip(edited_after.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst < 1e-5,
            "{name}: dialling it AFTER the stroke must give the relief of having painted with it \
             (worst pixel differs by {worst})"
        );
    }
}

#[test]
fn impasto_shine_glints_on_the_wall_without_bleaching_the_rim() {
    // Enio, 2026-07-12: "shine não funciona." He was right, and the cause was geometric: the relief's
    // slope exists ONLY over the coverage band `W_TAIL..W_SOLID` (that IS the wall), while the glint
    // had been gated ABOVE `W_SOLID` — i.e. allowed only on the plateau, which is flat, where the pass
    // early-outs. Measured: 94% of the sloped pixels sat below the gate, and Shine 0 → 1 moved the
    // brightest pixel by ONE level. A knob that does nothing.
    //
    // This gate pins BOTH halves, because fixing either one alone is how it broke: the glint must be
    // VISIBLE (it was not) and it must not BLEACH the translucent rim (the white halo of the first
    // photograph — which came back the moment the glint was let onto the wall as a flat `+ add`, since
    // on a rim pixel the red channel is already at the ceiling and only the other channels move).
    let size = 160u32;
    let paint_with = |shine: f32| -> (Vec<u8>, Vec<f32>, Vec<u8>) {
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.7;
        // RED paint on white paper: the rim's "ink" is measured as `R − G`, so the canvas fixture's own
        // dark blue would read as zero ink and the bleach half of this gate would be vacuous. (It said
        // so out loud on the first run — which is the anti-vacuity clause earning its keep.)
        b.color = [0.9, 0.1, 0.1];
        // And the SMOKE's own arming — a grain-sourced brush over noise. Not decoration: with a plain
        // Uniform brush the highlight never reaches the translucent rim at all, so the bleach half of
        // this gate passed even with a flat additive highlight (proved by mutation). The grain carves
        // crests everywhere, including out on the thin paint, which is precisely the condition that
        // photographed as a halo.
        b.impasto_source = DepthSource::Grain;
        b.impasto_smoothing = 0.15;
        b.texture.kind = ph2d_painter_brush::TextureKind::Noise;
        b.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.paint.impasto_light_angle_deg = 90;
        t.paint.impasto_light_elev_deg = 45;
        t.paint.impasto_shine = shine;
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
        let img = lit(&mut t);
        let h = relief(&t);
        let active = t.layers.active().expect("a layer");
        let cov = t.covers.get(&active).cloned().unwrap_or_default();
        (img, h, cov)
    };
    let (matte, h, cov) = paint_with(0.0);
    let (glossy, _, _) = paint_with(1.0);

    // 1. The glint is VISIBLE. (RED with the glint gated to the plateau: the brightest gain was 1.)
    let (mut best, mut best_i) = (0i32, 0usize);
    for i in (0..matte.len()).step_by(4) {
        let gain = i32::from(glossy[i + 1]) - i32::from(matte[i + 1]); // green: the pigment is red
        if gain > best {
            best = gain;
            best_i = i / 4;
        }
    }
    assert!(
        best >= 40,
        "Shine must actually light the paint (brightest gain {best} levels)"
    );

    // 2. It lands on the WALL — sloped paint with a real body — not on the flat plateau or the stain.
    let px = |i: usize| (i % size as usize, i / size as usize);
    let (bx, by) = px(best_i);
    let gx = (h[best_i + 1] - h[best_i - 1]).abs();
    let gy = (h[best_i + size as usize] - h[best_i - size as usize]).abs();
    assert!(
        gx.max(gy) > 0.005,
        "the brightest glint sits on SLOPED paint (grad {gx:.4}/{gy:.4} at {bx},{by})"
    );
    assert!(
        f32::from(cov[best_i]) / 255.0 > 0.4,
        "…and on paint with a body, not on the translucent stain (coverage {})",
        f32::from(cov[best_i]) / 255.0
    );

    // 3. And at FULL Shine the pass is still a STRICT NO-OP on the translucent stain — the paint too
    //    thin to have a body (`cover < W_TAIL`). That is the halo's actual door, and it is now nailed
    //    shut by construction: no body ⇒ no relief AND no lighting weight, so those pixels come out
    //    byte-identical no matter how the light is dialled.
    //
    //    What this deliberately does NOT claim: that a highlight never washes a *lit wall* toward
    //    white. It does — that is what a highlight is (the worst "bleached" pixel under an earlier,
    //    stricter version of this assertion turned out to be paint at 70% coverage whose red channel
    //    the DIFFUSE had already driven to 255; the chroma metric could not tell an honest glint from
    //    the halo). The default look is guarded instead by
    //    `impasto_light_shades_the_paint_not_the_paper_showing_through_it`, which is the gate that
    //    catches a flat additive highlight (proved: it goes red at 19% survival).
    let w_tail = ph2d_painter_brush::height::W_TAIL;
    let unlit = {
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.7;
        b.color = [0.9, 0.1, 0.1];
        b.impasto_source = DepthSource::Grain;
        b.impasto_smoothing = 0.15;
        b.texture.kind = ph2d_painter_brush::TextureKind::Noise;
        b.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.paint.impasto_show = false; // the light pass does not run at all
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
        lit(&mut t)
    };
    let _ = &h;
    let (mut stain_px, mut drifted, mut worst_drift) = (0u32, 0u32, 0i32);
    for p in 0..(size * size) as usize {
        let c = f32::from(cov[p]) / 255.0;
        if c == 0.0 || c >= w_tail {
            continue; // bare paper, or paint with a body — not the stain
        }
        stain_px += 1;
        // (The stain CAN carry a little height — `Smoothing` settles the paint and the blur spreads it
        // past the body's edge, which is what settling paint does. What must not happen is the LIGHT
        // reading it: the weight is the body curve, which is zero here, so the pixels stay untouched.)
        for ch in 0..3 {
            let d = i32::from(glossy[p * 4 + ch]) - i32::from(unlit[p * 4 + ch]);
            if d != 0 {
                drifted += 1;
            }
            worst_drift = worst_drift.max(d.abs());
        }
    }
    assert!(
        stain_px > 300,
        "sanity: the fixture HAS a translucent stain ({stain_px} px) — else this claim is vacuous"
    );
    assert_eq!(
        drifted, 0,
        "at full Shine the pass moved {drifted} channels of the translucent stain (worst {worst_drift} \
         levels) — the light must not touch paint too thin to have a body"
    );
}
