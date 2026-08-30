//! **Os meios que REESCREVEM pixels que já existem, em vez de depositar tinta nova.** Smear, Blur,
//! Inpaint, Clone, o conta-gotas, o balde (Fill / ColorDrop, com o slider modal do limiar) e a camada
//! isolada do composite. O que os une é a fonte: todos leem a tela antes de escrever nela.

use super::*;

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
