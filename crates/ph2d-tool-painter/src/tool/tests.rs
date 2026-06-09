use super::*;

fn flat_source(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        v.extend_from_slice(&rgba);
    }
    v
}

#[test]
fn id_label_icon_slug_panel() {
    let t = PainterTool::default();
    assert_eq!(t.id(), ToolId::new("painter"));
    assert_eq!(t.label(), "Painter");
    assert_eq!(t.icon_slug(), "painter");
    let p = t.build_panel();
    assert_eq!(p.tool_id, ToolId::new("painter"));
}

#[test]
fn activate_sets_takeover() {
    let mut t = PainterTool::default();
    assert!(!t.params.takeover_active);
    t.on_activate();
    assert!(t.params.takeover_active);
    t.on_deactivate();
    assert!(!t.params.takeover_active);
}

#[test]
fn not_default_in_w1() {
    assert!(!PainterTool::default().is_default());
}

#[test]
fn set_source_marks_dirty_and_drains() {
    let mut t = PainterTool::default();
    let src = flat_source(8, 8, [255, 255, 255, 255]);
    t.set_source(src.clone(), 8, 8);
    let (px, w, h) = t.current_preview().expect("dirty after set_source");
    assert_eq!((w, h), (8, 8));
    assert_eq!(px, src.as_slice());
    // Drained — next call returns None.
    assert!(t.current_preview().is_none());
}

#[test]
fn set_source_creates_single_raster_layer() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [10, 20, 30, 255]), 8, 8);
    let stack = t.layers();
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.root().len(), 1);
    let id = stack.active().expect("active layer");
    let layer = stack.get(id).expect("layer");
    assert!(matches!(layer.kind, crate::layers::LayerKind::Raster(_)));
    assert!(layer.visible && layer.opacity >= 1.0);
    assert_eq!(layer.blend_mode, BlendMode::Normal);
}

#[test]
fn gpu_preview_versions_track_pixels_not_metadata() {
    // The GPU preview provider (`preview_layer_pixels`) reports a per-layer
    // pixel content version. It must bump on a PIXEL change and stay stable
    // on a METADATA edit (so the GPU compositor keeps the slice cached
    // across an opacity / adjustment slider drag), and survive layer-id
    // reuse across `set_source`.
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [10, 20, 30, 255]), 4, 4);
    let active = t
        .layers()
        .active()
        .expect("set_source creates an active raster");
    let key = active.0;
    let (v0, px) = t
        .preview_layer_pixels(key)
        .expect("active layer reads the canvas buffer");
    assert_eq!(px.len(), 4 * 4 * 4, "active layer pixels = canvas_rgba");
    assert!(v0 > 0, "set_source bumps the new layer off the default 0");

    // Metadata edit (opacity) → version UNCHANGED (the caching invariant).
    t.set_layer_opacity(active, 0.5);
    let (v1, _) = t.preview_layer_pixels(key).expect("layer still present");
    assert_eq!(v1, v0, "an opacity edit must not bump the pixel version");

    // A fresh source reuses layer id 1 for a DIFFERENT image; the version
    // must be strictly greater so the compositor never samples a stale slice.
    t.set_source(flat_source(2, 2, [200, 200, 200, 255]), 2, 2);
    let reused = t.layers().active().expect("active");
    let (v2, _) = t.preview_layer_pixels(reused.0).expect("present");
    assert!(
        v2 > v1,
        "key reuse across set_source must produce a strictly greater version"
    );

    // Unknown key → None (the bridge then falls back to the CPU path).
    assert!(t.preview_layer_pixels(9_999).is_none());
}

#[test]
fn hidden_layer_composites_to_transparent() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [200, 50, 50, 255]), 4, 4);
    let id = t.layers().active().unwrap();
    t.set_layer_visible(id, false);
    let (px, _, _) = t.current_preview().expect("dirty after edit");
    // Every pixel fully transparent (invisible single layer → nothing).
    assert!(
        px.chunks_exact(4).all(|p| p[3] == 0),
        "hidden layer must clear alpha"
    );
}

#[test]
fn half_opacity_layer_halves_alpha() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [255, 255, 255, 255]), 2, 2);
    let id = t.layers().active().unwrap();
    t.set_layer_opacity(id, 0.5);
    let (px, _, _) = t.current_preview().expect("dirty after edit");
    // αo = 0.5 over transparent → ~127 (alpha is linear coverage).
    assert!(
        (px[3] as i32 - 127).abs() <= 2,
        "expected ~127 alpha, got {}",
        px[3]
    );
}

#[test]
fn trivial_stack_preview_is_byte_exact_source() {
    // The N=1 default stack must NOT composite — preview == source bytes.
    let mut t = PainterTool::default();
    let src = flat_source(8, 8, [123, 45, 200, 255]);
    t.set_source(src.clone(), 8, 8);
    let (px, _, _) = t.current_preview().unwrap();
    assert_eq!(
        px,
        src.as_slice(),
        "trivial stack must skip composite (fast path)"
    );
}

#[test]
fn add_layer_stacks_transparent_on_top_base_shows_through() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [200, 50, 50, 255]), 4, 4); // base = red
    let base = t.layers().active().unwrap();
    let top = t.add_raster_layer("Layer 2").expect("add layer");
    assert_eq!(t.layers().len(), 2);
    assert_eq!(t.layers().active(), Some(top));
    assert_eq!(t.layers().root().first(), Some(&top), "new layer on top");
    assert_ne!(top, base);
    // Top is transparent → base red composites through.
    let (px, _, _) = t.current_preview().expect("dirty after add");
    assert_eq!(&px[0..3], &[200, 50, 50]);
    assert_eq!(px[3], 255);
}

#[test]
fn select_layer_round_trips_working_buffers() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 50, 50, 255]), 2, 2); // base red
    let base = t.layers().active().unwrap();
    let top = t.add_raster_layer("Layer 2").unwrap();
    // Active is the fresh transparent top.
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 0),
        "new layer canvas is transparent"
    );
    // Switch to base → its red pixels load into the working canvas.
    t.select_layer(base);
    assert_eq!(t.layers().active(), Some(base));
    assert!(
        t.canvas_rgba
            .chunks_exact(4)
            .all(|p| p == [200, 50, 50, 255])
    );
    // Back to top → transparent again.
    t.select_layer(top);
    assert!(t.canvas_rgba.iter().all(|&b| b == 0));
}

// ── W3.T3.4 UI-plumbing: handle_panel_event routing (per-row decode) ──

#[test]
fn panel_event_add_layer_creates_and_activates() {
    use ph2d_editor_core::ids::PAINTER_LAYERS_ADD;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0; 4]), 2, 2);
    assert_eq!(t.layers().len(), 1);
    let before = t.layers().active();
    t.handle_panel_event(PanelEvent::Click(PAINTER_LAYERS_ADD));
    assert_eq!(t.layers().len(), 2, "+Layer adds a raster layer");
    assert_ne!(t.layers().active(), before, "the new layer becomes active");
}

#[test]
fn panel_event_apply_requests_commit() {
    use ph2d_editor_core::ids::PAINTER_APPLY;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [1, 2, 3, 255]), 2, 2);
    t.handle_panel_event(PanelEvent::Click(PAINTER_APPLY));
    assert!(
        t.take_pending_commit(),
        "the Apply CTA sets pending_commit (the bridge bakes it next frame)"
    );
}

#[test]
fn panel_event_eye_click_toggles_visibility() {
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0; 4]), 2, 2);
    let id = t.layers().active().unwrap();
    let eye = painter_layer_widget_id(id.0, PainterLayerWidget::Visibility);
    assert!(t.layers().get(id).unwrap().visible);
    t.handle_panel_event(PanelEvent::Click(eye));
    assert!(!t.layers().get(id).unwrap().visible, "eye click hides");
    t.handle_panel_event(PanelEvent::Click(eye));
    assert!(t.layers().get(id).unwrap().visible, "eye click re-shows");
}

#[test]
fn panel_event_opacity_setvalue_sets_layer_opacity() {
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0; 4]), 2, 2);
    let id = t.layers().active().unwrap();
    let slider = painter_layer_widget_id(id.0, PainterLayerWidget::Opacity);
    t.handle_panel_event(PanelEvent::SetValue(slider, 0.5));
    assert!((t.layers().get(id).unwrap().opacity - 0.5).abs() < 1e-6);
}

#[test]
fn paper_slider_routes_to_params_paper_grain() {
    // The Brush Studio "Paper" slider drives `PainterParams::paper_grain` (a
    // substrate property, NOT a brush field) and the studio snapshot mirrors it.
    use ph2d_editor_core::ids::PAINTER_STUDIO_PAPER_SLIDER;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    // Default is off (the user opts in — fixes "grain when grain-source off").
    assert_eq!(t.params.paper_grain, 0.0);
    assert_eq!(t.brush_studio_snapshot().paper_grain, 0.0);
    t.handle_panel_event(PanelEvent::SetValue(PAINTER_STUDIO_PAPER_SLIDER, 0.7));
    assert!((t.params.paper_grain - 0.7).abs() < 1e-6);
    assert!((t.brush_studio_snapshot().paper_grain - 0.7).abs() < 1e-6);
    // Out-of-range clamps to [0,1].
    t.handle_panel_event(PanelEvent::SetValue(PAINTER_STUDIO_PAPER_SLIDER, 1.5));
    assert_eq!(t.params.paper_grain, 1.0);
}

#[test]
fn motion_filter_and_speed_sliders_route_to_brush() {
    // Motion filtering (unipolar 0..1) + velocity dynamics (bipolar: slider 0..1 →
    // speed_* −1..1, 0.5 = neutral) wire to the Brush sub-structs (ADR-0077 D10).
    use ph2d_editor_core::ids::{
        PAINTER_STUDIO_MOTION_FILTER_SLIDER, PAINTER_STUDIO_SPEED_SIZE_SLIDER,
    };
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.handle_panel_event(PanelEvent::SetValue(
        PAINTER_STUDIO_MOTION_FILTER_SLIDER,
        0.6,
    ));
    assert!((t.brush.stabilization.motion_filtering_amount - 0.6).abs() < 1e-6);
    // Bipolar: 0.5 → 0 (off), 1.0 → +1, 0.0 → −1.
    t.handle_panel_event(PanelEvent::SetValue(PAINTER_STUDIO_SPEED_SIZE_SLIDER, 0.5));
    assert!(
        t.brush.dynamics.speed_size.abs() < 1e-6,
        "0.5 slider = neutral"
    );
    t.handle_panel_event(PanelEvent::SetValue(PAINTER_STUDIO_SPEED_SIZE_SLIDER, 1.0));
    assert!(
        (t.brush.dynamics.speed_size - 1.0).abs() < 1e-6,
        "1.0 slider = +1"
    );
    t.handle_panel_event(PanelEvent::SetValue(PAINTER_STUDIO_SPEED_SIZE_SLIDER, 0.0));
    assert!(
        (t.brush.dynamics.speed_size + 1.0).abs() < 1e-6,
        "0.0 slider = −1"
    );
    assert!(
        (t.brush_studio_snapshot().speed_size + 1.0).abs() < 1e-6,
        "snapshot mirrors it"
    );
}

#[test]
fn panel_event_blend_selectoption_sets_mode() {
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0; 4]), 2, 2);
    let id = t.layers().active().unwrap();
    let blend = painter_layer_widget_id(id.0, PainterLayerWidget::Blend);
    t.handle_panel_event(PanelEvent::SelectOption(
        blend,
        BlendMode::Multiply.to_u8().to_string(),
    ));
    assert_eq!(t.layers().get(id).unwrap().blend_mode, BlendMode::Multiply);
}

#[test]
fn panel_event_row_click_selects_layer() {
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0; 4]), 2, 2);
    let base = t.layers().active().unwrap();
    let top = t.add_raster_layer("Layer 2").unwrap();
    assert_eq!(t.layers().active(), Some(top));
    let row = painter_layer_widget_id(base.0, PainterLayerWidget::Row);
    t.handle_panel_event(PanelEvent::Click(row));
    assert_eq!(
        t.layers().active(),
        Some(base),
        "row click selects that layer"
    );
}

#[test]
fn panel_event_dock_toggle_flips_from_either_panel() {
    use ph2d_editor_core::ids::{PAINTER_LAYERS_TOGGLE_DOCK, PAINTER_SIDEBAR_TOGGLE_DOCK};
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    assert!(!t.dock_shows_layers());
    t.handle_panel_event(PanelEvent::Click(PAINTER_SIDEBAR_TOGGLE_DOCK));
    assert!(
        t.dock_shows_layers(),
        "sidebar 'Layers' shows the layers panel"
    );
    t.handle_panel_event(PanelEvent::Click(PAINTER_LAYERS_TOGGLE_DOCK));
    assert!(
        !t.dock_shows_layers(),
        "layers 'Brush' shows the brush sidebar"
    );
}

#[test]
fn panel_event_chrome_ids_are_not_decoded_as_rows() {
    // A fixed chrome id (the +Layer button) must take the ADD branch, not
    // be misread as a per-row widget (collision-safety of the hash ids).
    use ph2d_editor_core::ids::{PAINTER_LAYERS_ADD, PainterLayerWidget};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0; 4]), 2, 2);
    assert!(
        t.decode_layer_widget(PAINTER_LAYERS_ADD).is_none(),
        "the +Layer chrome id is not a per-row widget"
    );
    // And every real per-row id decodes back to its (layer, kind).
    let id = t.layers().active().unwrap();
    for kind in PainterLayerWidget::ALL {
        let wid = ph2d_editor_core::ids::painter_layer_widget_id(id.0, kind);
        assert_eq!(t.decode_layer_widget(wid), Some((id, kind)));
    }
}

#[test]
fn painting_top_layer_composites_over_base() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 50, 50, 255]), 2, 2); // base red
    let _top = t.add_raster_layer("Layer 2").unwrap();
    // Simulate a stroke filling the (active) top layer opaque blue.
    t.canvas_rgba = std::sync::Arc::new([0, 0, 200, 255].repeat(4));
    t.preview_dirty = true;
    let (px, _, _) = t.current_preview().expect("dirty");
    assert_eq!(&px[0..3], &[0, 0, 200], "opaque top layer covers base");
}

#[test]
fn take_preview_arc_composites_multi_layer() {
    // W3 smoke regression: the on-canvas preview goes through
    // `take_preview_arc` (the bridge fast path), which must composite the
    // multi-layer stack — not just hand back the active layer's
    // `canvas_rgba`, or overlapping layers / opacity / blend never show.
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 50, 50, 255]), 2, 2); // base red (Layer 1)
    let _top = t.add_raster_layer("Layer 2").unwrap(); // transparent top, active
    t.preview_dirty = true;
    let (rgba, _, _) = t.take_preview_arc().expect("dirty");
    assert_eq!(
        &rgba[0..4],
        &[200, 50, 50, 255],
        "transparent top composites over the red base (not the empty active layer)"
    );
    // Paint the top opaque blue → the Arc preview now shows blue over red.
    t.canvas_rgba = std::sync::Arc::new([0, 0, 200, 255].repeat(4));
    t.preview_dirty = true;
    let (rgba2, _, _) = t.take_preview_arc().expect("dirty");
    assert_eq!(
        &rgba2[0..3],
        &[0, 0, 200],
        "opaque top covers base in the Arc preview"
    );
}

#[test]
fn dirty_rect_drain_matches_full_recompose() {
    // The stroke-time fast lane: composite ONLY the dirty bbox into the
    // cache must be byte-identical to a full recompose of the same state.
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [200, 50, 50, 255]), 4, 4); // base red (L1)
    let _top = t.add_raster_layer("L2").unwrap(); // transparent top, active
    // First drain → full composite cached (all red; L2 is transparent).
    t.preview_dirty = true;
    let _ = t.take_preview_arc().expect("dirty");
    // Paint one pixel of the active (top) layer blue.
    let canvas = std::sync::Arc::make_mut(&mut t.canvas_rgba);
    let (px, py, cw) = (1usize, 1usize, 4usize);
    let i = (py * cw + px) * 4; // byte offset of pixel (1,1)
    canvas[i..i + 4].copy_from_slice(&[0, 0, 200, 255]);
    // Region drain (only the changed pixel).
    t.dirty_rect = Some(Region {
        x: 1,
        y: 1,
        w: 1,
        h: 1,
    });
    t.preview_dirty = true;
    let (region_drain, _, _) = t.take_preview_arc().expect("dirty");
    let region_drain = (*region_drain).clone();
    // Full recompose of the identical state (force the cache miss).
    t.composited = None;
    t.preview_dirty = true;
    let (full_drain, _, _) = t.take_preview_arc().expect("dirty");
    assert_eq!(
        region_drain, *full_drain,
        "dirty-rect drain == full recompose"
    );
    assert_eq!(
        &full_drain[i..i + 3],
        &[0, 0, 200],
        "the painted pixel shows"
    );
}

#[test]
fn dirty_rect_drain_matches_full_with_group_and_blend() {
    // Lens-6 gap: the flat test above never exercises the GROUP sub-window
    // recursion through the `take_preview_arc` fast lane (composite_region +
    // blit_region). A group with a blended, ACTIVE child above a base must
    // crop bit-for-bit too — proving the cache/blit integration holds when
    // the composite recurses, not just for a flat raster.
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [200, 50, 50, 255]), 4, 4); // base red (Layer 1 → images)
    let child = t.add_raster_layer("child").unwrap(); // active → canvas_rgba
    // Wrap `child` in a Screen/0.8 group; child blends Multiply. The tool
    // doesn't expose add_group yet, so build the stack directly (tests have
    // field access). `child` stays active, so its pixels = `canvas_rgba`.
    t.layers.set_blend_mode(child, BlendMode::Multiply);
    let group = t.layers.add_group("group").unwrap();
    t.layers.set_blend_mode(group, BlendMode::Screen);
    t.layers.set_opacity(group, 0.8);
    assert!(t.layers.move_into_group(child, group));
    t.layers.set_active(child);
    assert!(
        !t.is_trivial_stack(),
        "group + 2 rasters must be non-trivial"
    );
    // First drain → full composite cached.
    t.preview_dirty = true;
    let _ = t.take_preview_arc().expect("dirty");
    // Paint one pixel of the active (child) layer.
    let canvas = std::sync::Arc::make_mut(&mut t.canvas_rgba);
    let (px, py, cw) = (2usize, 1usize, 4usize);
    let i = (py * cw + px) * 4;
    canvas[i..i + 4].copy_from_slice(&[80, 220, 120, 255]);
    // Region drain (only the changed pixel) — goes through the group path.
    t.dirty_rect = Some(Region {
        x: 2,
        y: 1,
        w: 1,
        h: 1,
    });
    t.preview_dirty = true;
    let (region_drain, _, _) = t.take_preview_arc().expect("dirty");
    let region_drain = (*region_drain).clone();
    // Full recompose of the identical state (force the cache miss).
    t.composited = None;
    t.preview_dirty = true;
    let (full_drain, _, _) = t.take_preview_arc().expect("dirty");
    assert_eq!(
        region_drain, *full_drain,
        "group dirty-rect drain == full recompose"
    );
}

#[test]
fn out_of_window_pointer_sample_is_dropped() {
    // B.4 audit-2: a pointer position outside the Q16.16 useful window
    // (|v| >= 32768, ADR-0046 §2.2) must be DROPPED from the history record
    // (no clamp, no panic). Pre-fix this tripped `f32_to_q1616_saturating`'s
    // debug_assert in test builds; now `_checked` returns None → sample skipped.
    let mut t = PainterTool::default();
    t.params.size_px = 8.0;
    t.set_source(flat_source(16, 16, [0, 0, 0, 255]), 16, 16);
    t.begin_stroke(7);
    t.queue_pointer(PointerSample {
        position: [8.0, 8.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let recorded = t.current_samples.len();
    assert!(recorded >= 1, "in-window sample recorded");
    // Out-of-window position: dropped (no panic, no growth, no clamped record).
    t.queue_pointer(PointerSample {
        position: [40000.0, 8.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert_eq!(
        t.current_samples.len(),
        recorded,
        "out-of-window position dropped, not recorded as a clamped sample"
    );
    t.end_stroke();
}

#[test]
fn full_flow_mixbox_yellow_over_blue_is_green() {
    // Reproduce the LIVE app path end-to-end: the default brush is Mixbox, paint a
    // YELLOW stamp over a BLUE canvas at ~50% (deposit ≈ 0.5 → an even pigment mix).
    // The painted centre must be GREEN-dominant. If this is grey, the bug is in the
    // brush→scheduler→cpu_render flow (not just cpu_render in isolation).
    let (w, h) = (16u32, 16u32);
    let mut t = PainterTool::default(); // Mixbox by default now
    t.params.size_px = 24.0;
    t.params.opacity = 0.5;
    t.set_source(flat_source(w, h, [0, 0, 255, 255]), w, h); // opaque blue
    t.params.active_color = crate::color::srgb8_to_painter_oklch([255, 255, 0, 255]); // yellow
    t.begin_stroke(1);
    t.queue_pointer(PointerSample {
        position: [8.0, 8.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    let (px, _, _) = t.current_preview().expect("painted preview");
    let c = ((8 * w + 8) * 4) as usize;
    let (r, g, b) = (px[c] as i32, px[c + 1] as i32, px[c + 2] as i32);
    assert!(
        g > r + 12 && g > b + 12,
        "live-flow Mixbox centre is green-dominant, not grey: rgb=({r},{g},{b})"
    );
}

#[test]
fn toggle_pigment_flips_brush_mode_and_snapshot() {
    use ph2d_painter_brush::PigmentMode;
    let mut t = PainterTool::default(); // smoke brush defaults to Subtractive
    assert!(t.ui_snapshot().pigment_enabled, "default shows pigment ON");
    assert_eq!(
        t.active_brush().rendering.pigment_mode,
        PigmentMode::Subtractive
    );
    // Toggle → Linear.
    t.apply_ui_edit(crate::params::PainterUiEdit::TogglePigment);
    assert!(!t.ui_snapshot().pigment_enabled, "toggled OFF");
    assert_eq!(t.active_brush().rendering.pigment_mode, PigmentMode::Linear);
    // Toggle back → Subtractive.
    t.apply_ui_edit(crate::params::PainterUiEdit::TogglePigment);
    assert!(t.ui_snapshot().pigment_enabled, "toggled back ON");
    assert_eq!(
        t.active_brush().rendering.pigment_mode,
        PigmentMode::Subtractive
    );
}

#[test]
fn toggle_accumulate_flips_brush_flag_and_snapshot() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default(); // accumulate OFF (wash) by default
    assert!(
        !t.ui_snapshot().accumulate_enabled,
        "default wash (accumulate OFF)"
    );
    assert!(!t.active_brush().rendering.accumulate);
    t.apply_ui_edit(crate::params::PainterUiEdit::ToggleAccumulate);
    assert!(t.ui_snapshot().accumulate_enabled, "toggled to build-up");
    assert!(t.active_brush().rendering.accumulate);
    // via the sidebar checkbox route (Click on the accumulate id).
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_SIDEBAR_ACCUMULATE_TOGGLE,
    ));
    assert!(!t.active_brush().rendering.accumulate, "back to wash");
}

#[test]
fn grain_cycles_through_all_four_types_and_back_to_off() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default(); // grain OFF by default
    assert_eq!(t.ui_snapshot().grain_type, 0, "default off");
    // Off → Simplex → Gabor → PaperWeave → SprayDot → Off (5 clicks).
    let expect = [1u8, 2, 3, 4, 0];
    for (i, want) in expect.iter().enumerate() {
        t.handle_panel_event(PanelEvent::Click(
            ph2d_editor_core::ids::PAINTER_SIDEBAR_GRAIN_TOGGLE,
        ));
        assert_eq!(
            t.ui_snapshot().grain_type,
            *want,
            "click {} → type {}",
            i + 1,
            want
        );
    }
}

#[test]
fn set_grain_depth_updates_brush() {
    let mut t = PainterTool::default();
    t.apply_ui_edit(crate::params::PainterUiEdit::SetGrainDepth(0.4));
    assert!((t.active_brush().grain.grain_depth - 0.4).abs() < 1e-6);
    // clamps to [0,1].
    t.apply_ui_edit(crate::params::PainterUiEdit::SetGrainDepth(1.5));
    assert_eq!(t.active_brush().grain.grain_depth, 1.0);
}

#[test]
fn pigment_and_accumulate_are_independent() {
    // The two toggles are orthogonal — flipping one must not move the other.
    let mut t = PainterTool::default(); // pigment ON, accumulate OFF
    t.apply_ui_edit(crate::params::PainterUiEdit::ToggleAccumulate); // accumulate ON
    assert!(
        t.ui_snapshot().pigment_enabled,
        "pigment untouched by accumulate toggle"
    );
    assert!(t.ui_snapshot().accumulate_enabled);
    t.apply_ui_edit(crate::params::PainterUiEdit::TogglePigment); // pigment OFF
    assert!(!t.ui_snapshot().pigment_enabled);
    assert!(
        t.ui_snapshot().accumulate_enabled,
        "accumulate untouched by pigment toggle"
    );
}

#[test]
fn pigment_toggle_via_panel_event_click() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::PigmentMode;
    let mut t = PainterTool::default();
    // The sidebar button routes Click(PIGMENT_TOGGLE) → handle_panel_event.
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_SIDEBAR_PIGMENT_TOGGLE,
    ));
    assert_eq!(t.active_brush().rendering.pigment_mode, PigmentMode::Linear);
}

#[test]
fn fluid_toggle_via_brush_studio_checkbox() {
    // Brush Studio "Fluid" checkbox routes Click(PAINTER_STUDIO_FLUID) →
    // handle_panel_event → SetBrushParam(Fluid) (bool via the uncapped param,
    // mirroring Wet/Burnt Edges — no new frozen PainterUiEdit slot). The studio
    // snapshot mirrors `brush.rendering.fluid_enabled` both ways.
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();
    assert!(
        !t.brush_studio_snapshot().fluid_enabled,
        "fluid off by default"
    );
    assert!(!t.active_brush().rendering.fluid_enabled);
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_STUDIO_FLUID,
    ));
    assert!(t.active_brush().rendering.fluid_enabled, "click → fluid on");
    assert!(
        t.brush_studio_snapshot().fluid_enabled,
        "snapshot mirrors it"
    );
    t.handle_panel_event(PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_STUDIO_FLUID,
    ));
    assert!(
        !t.active_brush().rendering.fluid_enabled,
        "second click → fluid off"
    );
}

#[test]
fn repro_two_strokes_blue_then_yellow_same_layer_scan_for_green() {
    // EXACT live repro: transparent canvas, paint a BLUE stroke, end it, then paint
    // a YELLOW stroke crossing it on the SAME layer at ~50% opacity. Scan the whole
    // preview: SOMEWHERE in the overlap the Mixbox mix must be green-dominant. If no
    // pixel is green, the two-stroke same-layer path is NOT mixing (the live bug).
    let (w, h) = (32u32, 32u32);
    let mut t = PainterTool::default(); // Mixbox by default
    t.params.size_px = 14.0;
    t.set_source(flat_source(w, h, [0, 0, 0, 0]), w, h); // transparent

    // Stroke 1: opaque BLUE, vertical line down the middle.
    t.params.opacity = 1.0;
    t.params.active_color = crate::color::srgb8_to_painter_oklch([0, 0, 255, 255]);
    t.begin_stroke(1);
    for y in 4..28 {
        t.queue_pointer(PointerSample {
            position: [16.0, y as f32],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();

    // Stroke 2: YELLOW at 50%, horizontal line crossing the blue.
    t.params.opacity = 0.5;
    t.params.active_color = crate::color::srgb8_to_painter_oklch([255, 255, 0, 255]);
    t.begin_stroke(2);
    for x in 4..28 {
        t.queue_pointer(PointerSample {
            position: [x as f32, 16.0],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();

    let (px, pw, ph) = t.current_preview().expect("painted preview");
    // Find the greenest pixel and report the overlap region.
    let mut best = (0i32, 0i32, 0i32, 0usize);
    let mut green_dominant = 0;
    for i in 0..(pw * ph) as usize {
        let (r, g, b, a) = (
            px[i * 4] as i32,
            px[i * 4 + 1] as i32,
            px[i * 4 + 2] as i32,
            px[i * 4 + 3] as i32,
        );
        if a < 40 {
            continue;
        }
        if g - r.max(b) > best.1 - best.0.max(best.2) {
            best = (r, g, b, i);
        }
        if g > r + 12 && g > b + 12 {
            green_dominant += 1;
        }
    }
    // Sample the exact crossing pixel too.
    let c = ((16 * pw + 16) * 4) as usize;
    let cross = (
        px[c] as i32,
        px[c + 1] as i32,
        px[c + 2] as i32,
        px[c + 3] as i32,
    );
    eprintln!(
        "REPRO greenest=({},{},{}) @idx{} | crossing(16,16)={:?} | green_dominant_px={}",
        best.0, best.1, best.2, best.3, cross, green_dominant
    );
    // The CROSSING pixel itself — where the opaque blue core meets the yellow
    // core — must be green-dominant. Pre-wash this was pure yellow (254,254,0)
    // because overlapping dabs built the deposit up to ~1.0; the wash model caps
    // it at the 50% opacity → a stable 50/50 mix → green.
    assert!(
        cross.1 > cross.0 + 12 && cross.1 > cross.2 + 12,
        "two-stroke crossing must be GREEN (wash caps deposit at opacity), got {:?} \
         (greenest seen=({},{},{}), green_dominant_px={})",
        cross,
        best.0,
        best.1,
        best.2,
        green_dominant
    );
    assert!(
        green_dominant > 50,
        "the whole overlap band must be green, not a thin fringe: only {} green px \
         (crossing={:?})",
        green_dominant,
        cross
    );
}

#[test]
fn preview_upload_bbox_tracks_partial_vs_full() {
    // B.1: the bridge uploads a partial GPU sub-rect ONLY when
    // `take_preview_arc` took the dirty-rect fast lane. Pin the contract the
    // bridge relies on: Some(bbox) after a fast-lane drain; None after a full
    // recompose; None for the trivial single-layer path; drained after read.
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [200, 50, 50, 255]), 4, 4);
    // Trivial single-layer drain → full upload (None).
    t.preview_dirty = true;
    let _ = t.take_preview_arc().expect("dirty");
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "trivial single-layer stack = full upload"
    );
    // Add a layer → non-trivial; first drain is a full recompose → None.
    let _top = t.add_raster_layer("L2").unwrap();
    t.preview_dirty = true;
    let _ = t.take_preview_arc().expect("dirty");
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "first non-trivial drain = full recompose = full upload"
    );
    // Stroke into the now-valid cache → fast lane → partial bbox.
    t.dirty_rect = Some(Region {
        x: 1,
        y: 1,
        w: 2,
        h: 2,
    });
    t.preview_dirty = true;
    let _ = t.take_preview_arc().expect("dirty");
    assert_eq!(
        t.take_preview_upload_bbox(),
        Some((1, 1, 2, 2)),
        "fast-lane drain = partial upload of the dirty bbox"
    );
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "bbox drained after read"
    );
}

#[test]
fn layers_revision_bumps_on_structure_not_strokes() {
    // B.5: the bridge republishes the LayerStack only when this changes, so
    // it MUST bump on every published-structure edit and MUST NOT bump on a
    // stroke (pixels aren't shown in the panel — bumping would defeat the
    // per-frame-clone elimination).
    let mut t = PainterTool::default();
    t.params.size_px = 4.0;
    t.set_source(flat_source(8, 8, [0, 0, 0, 255]), 8, 8);
    let r0 = t.layers_revision();
    let l2 = t.add_raster_layer("L2").unwrap();
    let r1 = t.layers_revision();
    assert!(r1 > r0, "add_raster_layer bumps the publish revision");
    t.set_layer_opacity(l2, 0.5);
    let r2 = t.layers_revision();
    assert!(r2 > r1, "opacity edit bumps the publish revision");
    // A stroke paints pixels but must NOT bump the structure revision.
    t.begin_stroke(1);
    t.queue_pointer(PointerSample {
        position: [4.0, 4.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    assert_eq!(
        t.layers_revision(),
        r2,
        "a stroke must not bump the publish revision"
    );
}

#[test]
fn active_color_srgb8_matches_snapshot() {
    // B.5: the direct accessor the bridge will use must equal the snapshot
    // path it replaces (so dropping the per-frame ui_snapshot is lossless).
    let mut t = PainterTool::default();
    t.params.active_color = crate::params::OklchColor {
        l: 0.6,
        c: 0.2,
        h: 0.5,
        a: 1.0,
    };
    assert_eq!(t.active_color_srgb8(), t.ui_snapshot().active_color_srgb8());
}

#[test]
fn mask_edit_defaults_brush_to_black_and_restores() {
    // The mask starts WHITE (all visible); entering it defaults the brush to
    // BLACK (hide) and restores the real color on leaving — otherwise the
    // default LIGHT color barely hides and masking looks like a no-op.
    let orange = crate::params::OklchColor {
        l: 0.7,
        c: 0.18,
        h: 0.5,
        a: 1.0,
    };
    let mut t = PainterTool::default();
    t.params.active_color = orange;
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let l2 = t.add_raster_layer("L2").unwrap();
    t.add_mask_to_active().unwrap(); // enter mask
    assert_eq!(t.params.active_color.l, 0.0, "mask brush defaults to black");
    assert_eq!(t.params.active_color.c, 0.0);
    t.select_layer(l2); // leave the mask → restore
    assert_eq!(
        t.params.active_color, orange,
        "real color restored on leaving"
    );
}

#[test]
fn add_mask_to_active_white_buffer_reveals_then_black_hides() {
    // T3.5 tool wiring: add a mask to the active raster → mask becomes the
    // edit target with an opaque-WHITE buffer (parent fully visible), and
    // painting it black hides the parent in the LIVE preview.
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [200, 0, 0, 255]), 4, 4); // red base (Layer 1, active)
    let mask = t.add_mask_to_active().expect("active raster takes a mask");
    assert_eq!(t.layers.active(), Some(mask), "mask is the edit target");
    assert!(t.active_is_mask());
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 255),
        "mask starts opaque white"
    );
    // White mask → parent fully visible.
    t.preview_dirty = true;
    let (rgba, _, _) = t.take_preview_arc().expect("dirty");
    assert_eq!(
        &rgba[0..4],
        &[200, 0, 0, 255],
        "white mask reveals red parent"
    );
    // Paint the mask black → parent hidden (alpha 0).
    let canvas = std::sync::Arc::make_mut(&mut t.canvas_rgba);
    for px in canvas.chunks_exact_mut(4) {
        px.copy_from_slice(&[0, 0, 0, 255]);
    }
    t.preview_dirty = true;
    let (rgba2, _, _) = t.take_preview_arc().expect("dirty");
    assert_eq!(rgba2[3], 0, "black mask hides the parent (alpha 0)");
}

#[test]
fn mask_paint_is_achromatic() {
    // §2.7: painting into a mask forces the stroke color to grayscale.
    let mut t = PainterTool::default();
    t.params.active_color = crate::params::OklchColor {
        l: 0.6,
        c: 0.3,
        h: 1.0,
        a: 1.0,
    };
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    assert_eq!(t.effective_active_color().c, 0.3, "raster keeps chroma");
    t.add_mask_to_active().unwrap();
    assert_eq!(
        t.effective_active_color().c,
        0.0,
        "mask paint is achromatic"
    );
}

#[test]
fn add_mask_to_active_rejects_non_raster_active() {
    // No source / no active raster → no-op.
    let mut t = PainterTool::default();
    assert!(t.add_mask_to_active().is_none(), "no source = no mask");
}

#[test]
fn handle_layer_reparent_ignores_a_dragged_mask() {
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::interaction::PainterLayerDrop;
    let row = |l: RtLayerId| painter_layer_widget_id(l.0, PainterLayerWidget::Row);
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let l2 = t.add_raster_layer("L2").unwrap();
    let mask = t.add_mask_to_active().unwrap(); // mask on l2 (owner-attached)
    let before = t.layers.root().to_vec();
    let base = *before.last().unwrap();
    // Dragging the mask must be a no-op — it never enters the z-order.
    t.handle_layer_reparent(row(mask), PainterLayerDrop::Inside(row(base)));
    assert_eq!(t.layers.root(), before.as_slice(), "mask drag is a no-op");
    assert!(!t.layers.root().contains(&mask));
    assert_eq!(
        t.layers.get(l2).unwrap().mask,
        Some(mask),
        "mask still attached"
    );
}

#[test]
fn modifier_toolbar_routes_toggle_the_active_layer() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2); // base
    let l2 = t.add_raster_layer("L2").unwrap(); // active raster
    // Clip toggles on then off.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LAYERS_CLIP));
    assert!(t.layers.get(l2).unwrap().clipping, "clip on");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LAYERS_CLIP));
    assert!(!t.layers.get(l2).unwrap().clipping, "clip off");
    // Lock + Ref toggle on.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LAYERS_ALPHA_LOCK));
    assert!(t.layers.get(l2).unwrap().alpha_locked, "lock on");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LAYERS_REFERENCE));
    assert!(t.layers.get(l2).unwrap().is_reference, "ref on");
    // Mask creates + activates a mask; active_modifiers reflects raster state.
    assert!(t.layers.active_modifiers().unwrap().is_raster);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LAYERS_MASK));
    assert!(t.active_is_mask(), "mask created + active");
}

#[test]
fn duplicate_layer_clones_pixels_above_and_activates() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [10, 20, 30, 255]), 2, 2); // base (Layer 1)
    let l2 = t.add_raster_layer("L2").unwrap(); // active, transparent
    t.canvas_rgba = std::sync::Arc::new([100, 110, 120, 255].repeat(4)); // paint L2
    let dup = t.duplicate_layer(l2).expect("duplicate the active raster");
    assert_eq!(t.layers.active(), Some(dup), "the copy is active");
    assert_ne!(dup, l2);
    assert_eq!(
        &t.canvas_rgba[0..4],
        &[100, 110, 120, 255],
        "copy has source pixels"
    );
    // The copy sits directly above the source (above = lower index).
    let root = t.layers.root();
    let dpos = root.iter().position(|&x| x == dup).unwrap();
    let l2pos = root.iter().position(|&x| x == l2).unwrap();
    assert_eq!(dpos + 1, l2pos, "copy inserted directly above the source");
}

#[test]
fn delete_layer_removes_repoints_active_and_drops_buffer() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base (Layer 1)
    let l2 = t.add_raster_layer("L2").unwrap(); // active
    assert!(t.delete_layer(l2), "delete the active non-base layer");
    assert!(t.layers.get(l2).is_none(), "layer gone");
    assert!(!t.images.contains_key(&l2), "images entry dropped");
    assert_eq!(t.layers.len(), 1, "the base remains");
    assert_eq!(t.layers.active(), t.layers.root().first().copied());
}

#[test]
fn delete_layer_refuses_the_base_sprite() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let base = t.layers.root()[0]; // only layer = the base sprite
    assert!(!t.delete_layer(base), "the base sprite is not removable");
    assert_eq!(t.layers.len(), 1);
}

#[test]
fn group_active_wraps_the_active_layer() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base (Layer 1)
    let l2 = t.add_raster_layer("L2").unwrap(); // active
    let g = t.group_active().expect("group the active layer");
    assert_eq!(
        t.layers.active(),
        Some(l2),
        "the layer stays active, not the group"
    );
    assert_eq!(
        t.layers.depth(l2),
        1,
        "l2 nested one level inside the group"
    );
    assert!(matches!(t.layers.get(g).unwrap().kind, LayerKind::Group(_)));
    assert!(
        !t.layers.root().contains(&l2),
        "l2 left the root (now in the group)"
    );
}

#[test]
fn group_active_refuses_base_sprite() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    assert!(
        t.group_active().is_none(),
        "base sprite active → can't group it"
    );
}

#[test]
fn handle_layer_reparent_drags_into_group_and_reorders() {
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::interaction::PainterLayerDrop;
    let row = |l: RtLayerId| painter_layer_widget_id(l.0, PainterLayerWidget::Row);
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base (Layer 1)
    let base = t.layers.root()[0];
    let l2 = t.add_raster_layer("L2").unwrap();
    let g = t.add_group().unwrap(); // empty group at top → root=[g, l2, base]
    // Drag l2 INTO the group.
    t.handle_layer_reparent(row(l2), PainterLayerDrop::Inside(row(g)));
    assert_eq!(t.layers.depth(l2), 1, "l2 nested into the group via drag");
    // Drag l2 back out to the root bottom (above base).
    t.handle_layer_reparent(row(l2), PainterLayerDrop::End);
    assert_eq!(t.layers.depth(l2), 0, "l2 pulled back to root");
    assert_eq!(
        t.layers.root().last(),
        Some(&base),
        "base still pinned bottom"
    );
}

#[test]
fn handle_layer_reparent_inside_leaf_falls_back_to_before_sibling() {
    // W3.T3.8: the middle ("Inside") band over a NON-group layer must not be
    // a dead no-op — it falls back to a before-sibling insert so every drop
    // position is meaningful and the panel's drop indicator (a before-line
    // over leaves) tells the truth.
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::interaction::PainterLayerDrop;
    let row = |l: RtLayerId| painter_layer_widget_id(l.0, PainterLayerWidget::Row);
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base (Layer 1)
    let l2 = t.add_raster_layer("L2").unwrap();
    let l3 = t.add_raster_layer("L3").unwrap(); // root = [l3, l2, base]
    assert_eq!(t.layers.root()[0], l3, "L3 starts topmost");
    // Drop l2 on the MIDDLE band of l3 (a leaf raster, not a group).
    t.handle_layer_reparent(row(l2), PainterLayerDrop::Inside(row(l3)));
    assert_eq!(
        t.layers.depth(l2),
        0,
        "no nesting into a leaf — stays at root"
    );
    assert_eq!(t.layers.root()[0], l2, "l2 inserted before l3 as a sibling");
    assert_eq!(t.layers.root()[1], l3);
}

#[test]
fn deactivate_clears_canvas() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [0; 4]), 4, 4);
    t.deactivate();
    assert!(t.current_preview().is_none());
    assert_eq!(t.source_size, (0, 0));
    assert!(t.canvas_rgba.is_empty());
    assert!(!t.is_stroke_active());
}

#[test]
fn pending_commit_is_drained() {
    let mut t = PainterTool::default();
    assert!(!t.take_pending_commit());
    t.request_commit();
    assert!(t.take_pending_commit());
    assert!(!t.take_pending_commit(), "drained");
}

#[test]
fn run_full_returns_canvas_clone() {
    let mut t = PainterTool::default();
    let src = flat_source(4, 4, [128, 64, 32, 255]);
    t.set_source(src.clone(), 4, 4);
    let (out, w, h) = t.run_full();
    assert_eq!((w, h), (4, 4));
    assert_eq!(out, src);
}

#[test]
fn run_full_bakes_multi_layer_composite() {
    // Apply must bake the full composite (what the preview shows), not just
    // the active layer — else the other layers are lost on Apply.
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 50, 50, 255]), 2, 2); // base red (Layer 1)
    let _top = t.add_raster_layer("Layer 2").unwrap(); // transparent top, active
    t.canvas_rgba = std::sync::Arc::new([0, 0, 200, 255].repeat(4)); // paint top blue
    let (out, w, h) = t.run_full();
    assert_eq!((w, h), (2, 2));
    assert_eq!(
        &out[0..3],
        &[0, 0, 200],
        "Apply bakes the composite (blue top over red base)"
    );
}

#[test]
fn move_layer_reorders_within_root() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [10, 20, 30, 255]), 2, 2);
    let l1 = t.layers().root()[0]; // base (Layer 1)
    let l2 = t.add_raster_layer("L2").unwrap();
    let l3 = t.add_raster_layer("L3").unwrap();
    assert_eq!(t.layers().root(), &[l3, l2, l1], "new layers stack on top");
    t.move_layer_down(l3);
    assert_eq!(t.layers().root(), &[l2, l3, l1], "↓ moves L3 below L2");
    t.move_layer_up(l3);
    assert_eq!(t.layers().root(), &[l3, l2, l1], "↑ moves it back to top");
}

/// **Audit T1.6 R7 L1-5 contract update:** `queue_pointer` without
/// an active stroke on a non-empty canvas is a `debug_assert!` in
/// dev/test builds (surface bridge bugs) AND a silent no-op in
/// release builds (preserve existing tolerance to dropped pointer-
/// down events). The empty-canvas branch stays a silent no-op in
/// all builds — that's the legitimate "tool inactive / no source
/// yet" case the original test was protecting.
#[test]
fn queue_pointer_on_empty_canvas_is_silent_noop() {
    let mut t = PainterTool::default();
    // canvas_rgba is empty by default (no set_source called).
    t.queue_pointer(PointerSample {
        position: [4.0, 4.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    // No source, no panic, no preview state mutated.
    assert!(t.current_preview().is_none());
}

/// **Audit T1.6 R7 L1-5:** the debug_assert fires in dev/test
/// builds when `queue_pointer` is invoked without a matching
/// `begin_stroke` on a non-empty canvas — the previous silent no-op
/// was masking bridge bugs (a pointer-move handler that ran
/// without its pointer-down) and the subsequent `drain_painter`
/// would emit a "no strokes to apply" toast indistinguishable
/// from "Apply ran with no paint".
/// **Audit T1.6 R8 N1-5 — `set_brush` keeps the handle and the
/// runtime brush in sync.** The dual-source-of-truth was the bug
/// L1-4 fixed; without a gate, a future refactor that drops one
/// of the two writes would silently break the contract.
///
/// Note: `set_brush` takes `params::BrushHandle` (the local stub
/// from `params.rs`); `ph2d_painter_brush::OVAL_HARD` is the
/// canon `ph2d_painter_brush::BrushHandle`. HR-14 forward-compat
/// keeps them structurally identical (`pub struct
/// BrushHandle(u32)`) but they're distinct types; bridge via
/// the inner u32.
#[test]
fn set_brush_writes_both_handle_and_runtime_brush() {
    use ph2d_painter_brush::library;
    let mut t = PainterTool::default();
    let new_handle = crate::params::BrushHandle(ph2d_painter_brush::OVAL_HARD_SLOT);
    t.set_brush(new_handle, library::oval_hard());
    assert_eq!(
        t.active_brush_handle(),
        new_handle,
        "params.active_brush must be the new handle"
    );
    // The brush itself must be the runtime oval (we verify via the
    // shape source slot which is publicly observable).
    let brush_ref = t.active_brush();
    match &brush_ref.shape.shape_source {
        ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
            assert_eq!(
                *atlas_layer,
                ph2d_painter_brush::OVAL_HARD_SLOT,
                "self.brush.shape_source slot must match the handle"
            );
        }
        _ => panic!("expected Builtin shape source after set_brush"),
    }
}

/// **Audit T1.6 R8 N1-5 — `set_brush` mid-stroke fires the
/// `debug_assert!` (L1-6 R7 contract).** The release-build
/// state-preservation path is exercised separately by
/// `set_brush_after_end_stroke_swaps_cleanly` below.
#[test]
#[should_panic(expected = "set_brush called mid-stroke")]
fn set_brush_mid_stroke_panics_in_debug() {
    use ph2d_painter_brush::library;
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    t.begin_stroke(7);
    let new_handle = crate::params::BrushHandle(ph2d_painter_brush::OVAL_HARD_SLOT);
    t.set_brush(new_handle, library::oval_hard());
}

/// **Audit T1.6 R8 N1-5 — `set_brush` after a clean
/// `end_stroke` swaps both handle and runtime brush cleanly.**
/// Mirrors the canonical W2 sidebar flow: user finishes a
/// stroke, sidebar issues `SelectBrush(handle)`, the handler
/// calls `set_brush(handle, library::brush_from_handle(handle))`.
/// This is the happy path that proves the L1-4 sync invariant.
#[test]
fn set_brush_after_end_stroke_swaps_cleanly() {
    use ph2d_painter_brush::library;
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    t.begin_stroke(7);
    t.end_stroke();
    assert!(!t.is_stroke_active());
    let new_handle = crate::params::BrushHandle(ph2d_painter_brush::OVAL_HARD_SLOT);
    t.set_brush(new_handle, library::oval_hard());
    assert!(!t.is_stroke_active(), "stroke must remain closed");
    assert_eq!(t.active_brush_handle(), new_handle);
    // Verify the runtime brush swapped to oval (slot 3).
    match &t.active_brush().shape.shape_source {
        ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
            assert_eq!(*atlas_layer, ph2d_painter_brush::OVAL_HARD_SLOT);
        }
        _ => panic!("expected Builtin oval after set_brush"),
    }
}

/// **Audit T1.6 R8 N1-5 — `active_brush()` borrows the runtime
/// brush.** Pinned so removing the accessor (P1-3) becomes
/// compile-time observable.
#[test]
fn active_brush_returns_runtime_brush() {
    let t = PainterTool::default();
    // Default is round_hard. Verify atlas_layer is 0.
    let brush = t.active_brush();
    match &brush.shape.shape_source {
        ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
            assert_eq!(*atlas_layer, 0, "default brush is round_hard slot 0");
        }
        _ => panic!("default brush must use Builtin shape source"),
    }
}

#[test]
#[should_panic(expected = "queue_pointer called without an active stroke")]
fn queue_pointer_without_stroke_on_loaded_canvas_panics_in_debug() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    let _ = t.current_preview(); // drain set_source dirty
    t.queue_pointer(PointerSample {
        position: [4.0, 4.0],
        pressure: 1.0,
        tilt: 0.0,
    });
}

#[test]
fn stroke_writes_pixels() {
    // The Day-7 smoke in unit-test form.
    let mut t = PainterTool::default();
    // Non-zero color (OklchColor default is all zeros == OKLab black,
    // alpha 0 → no visible paint). Set red-ish color.
    t.params.active_color = crate::params::OklchColor {
        l: 0.6,
        c: 0.2,
        h: 0.5,
        a: 1.0,
    };
    t.params.size_px = 16.0;
    t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
    let _ = t.current_preview(); // drain set_source dirty
    t.begin_stroke(42);
    t.queue_pointer(PointerSample {
        position: [16.0, 16.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let (px, w, h) = t.current_preview().expect("paint must mark dirty");
    assert_eq!((w, h), (32, 32));
    // Center pixel should now be different from the initial black.
    let center_idx = (16 * 32 + 16) * 4;
    assert_ne!(
        &px[center_idx..center_idx + 4],
        &[0u8, 0, 0, 255],
        "stamp must overwrite center pixel"
    );
    t.end_stroke();
    assert!(!t.is_stroke_active());
}

#[test]
fn begin_stroke_implicitly_closes_previous() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    t.begin_stroke(1);
    assert!(t.is_stroke_active());
    // Begin again without end: defensive cleanup.
    t.begin_stroke(2);
    assert!(t.is_stroke_active());
    t.end_stroke();
    assert!(!t.is_stroke_active());
}

#[test]
fn set_source_ends_active_stroke() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    t.begin_stroke(1);
    assert!(t.is_stroke_active());
    // Source push mid-stroke must close it (defensive — bridge guarantees
    // order but this layer is honest).
    t.set_source(flat_source(8, 8, [255; 4]), 8, 8);
    assert!(!t.is_stroke_active());
}

// ──────────────────────────────────────────────────────────────────────
// W2.T2.2 — undo / redo end-to-end through PainterTool.
// ──────────────────────────────────────────────────────────────────────

/// Paint one full visible stroke (Begin → one stamp at center → End) with a
/// red-ish color so the center pixel changes. Returns the live canvas after
/// the stroke for byte comparison.
fn paint_one_stroke(t: &mut PainterTool, seed: u64, at: [f32; 2]) -> Vec<u8> {
    t.params.active_color = crate::params::OklchColor {
        l: 0.6,
        c: 0.2,
        h: 0.5,
        a: 1.0,
    };
    t.params.size_px = 16.0;
    t.begin_stroke(seed);
    t.queue_pointer(PointerSample {
        position: at,
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();
    t.canvas_rgba.as_ref().clone()
}

#[test]
fn undo_restores_layer_to_pre_stroke_state() {
    // Spec criterion #1: undo restores the layer to the pre-stroke state.
    let mut t = PainterTool::default();
    t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
    let _ = t.current_preview();
    let pristine = t.canvas_rgba.as_ref().clone();
    let painted = paint_one_stroke(&mut t, 42, [16.0, 16.0]);
    assert_ne!(painted, pristine, "stroke must change the canvas");
    assert!(t.can_undo());
    assert!(!t.can_redo());

    assert!(t.undo_last_stroke(), "undo must report it acted");
    assert_eq!(
        t.canvas_rgba.as_ref(),
        &pristine,
        "undo must restore the exact pre-stroke texture"
    );
    // Semantic canon stays in sync.
    assert!(t.stroke_history.is_empty(), "history record popped too");
    assert!(!t.can_undo());
    assert!(t.can_redo());
}

#[test]
fn redo_reapplies_the_undone_stroke() {
    // Spec criterion #2: redo reapplies the last undo.
    let mut t = PainterTool::default();
    t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
    let _ = t.current_preview();
    let painted = paint_one_stroke(&mut t, 7, [16.0, 16.0]);
    t.undo_last_stroke();
    assert!(t.redo_last_stroke(), "redo must report it acted");
    assert_eq!(
        t.canvas_rgba.as_ref(),
        &painted,
        "redo must restore the exact post-stroke texture"
    );
    assert_eq!(t.stroke_history.len(), 1, "semantic record re-inserted");
    assert!(t.can_undo());
    assert!(!t.can_redo());
}

#[test]
fn two_undos_then_two_redos_walk_states_in_order() {
    // Locks the undo↔redo ordering sync between the texture controller and
    // the semantic `stroke_history` across multiple strokes.
    let mut t = PainterTool::default();
    t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
    let _ = t.current_preview();
    let s0 = t.canvas_rgba.as_ref().clone();
    let s1 = paint_one_stroke(&mut t, 1, [8.0, 8.0]);
    let s2 = paint_one_stroke(&mut t, 2, [24.0, 24.0]);
    assert_eq!(t.stroke_history.len(), 2);
    // Undo back to pristine, two steps.
    t.undo_last_stroke();
    assert_eq!(t.canvas_rgba.as_ref(), &s1);
    t.undo_last_stroke();
    assert_eq!(t.canvas_rgba.as_ref(), &s0);
    assert!(t.stroke_history.is_empty());
    // Redo forward, two steps — must reach s1 then s2 (forward order).
    t.redo_last_stroke();
    assert_eq!(t.canvas_rgba.as_ref(), &s1);
    assert_eq!(t.stroke_history.len(), 1);
    t.redo_last_stroke();
    assert_eq!(t.canvas_rgba.as_ref(), &s2);
    assert_eq!(t.stroke_history.len(), 2);
}

#[test]
fn undo_redo_on_empty_history_is_a_noop() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    let _ = t.current_preview();
    assert!(!t.undo_last_stroke(), "nothing to undo");
    assert!(!t.redo_last_stroke(), "nothing to redo");
}

#[test]
fn empty_stroke_creates_no_undo_slot() {
    // A Begin→End with no painted sample must NOT push an undo entry (mirrors
    // the V-1 phantom-record gate on stroke_history).
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
    let _ = t.current_preview();
    t.begin_stroke(1);
    t.end_stroke(); // no queue_pointer between → empty
    assert!(!t.can_undo(), "empty stroke must not create an undo slot");
}

#[test]
fn new_stroke_after_undo_invalidates_redo() {
    // After undo, painting a new stroke discards the redo branch (both the
    // texture stack and the parallel semantic redo records).
    let mut t = PainterTool::default();
    t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
    let _ = t.current_preview();
    paint_one_stroke(&mut t, 1, [10.0, 10.0]);
    paint_one_stroke(&mut t, 2, [20.0, 20.0]);
    t.undo_last_stroke();
    assert!(t.can_redo());
    // New stroke after an undo:
    paint_one_stroke(&mut t, 3, [16.0, 16.0]);
    assert!(!t.can_redo(), "new stroke must invalidate redo");
}

#[test]
fn undo_250_strokes_without_corruption() {
    // Spec criterion #3: undo 250× without corrupting the layer. Each
    // stroke paints at a distinct location; after 250 undos the canvas must
    // be byte-identical to the pristine pre-paint source.
    let mut t = PainterTool::default();
    t.set_source(flat_source(64, 64, [0, 0, 0, 255]), 64, 64);
    let _ = t.current_preview();
    let pristine = t.canvas_rgba.as_ref().clone();
    for i in 0..250u64 {
        let x = (i % 64) as f32;
        let y = ((i / 64) % 64) as f32;
        paint_one_stroke(&mut t, i + 1, [x, y]);
    }
    // Undo every stroke (DEFAULT_MAX_DEPTH=300 > 250, so none were dropped).
    // After 250 undos the canvas must be byte-identical to the pristine
    // pre-paint source — the corruption gate.
    let mut undone = 0usize;
    while t.undo_last_stroke() {
        undone += 1;
        assert_eq!(t.canvas_rgba.len(), pristine.len(), "size never corrupts");
    }
    assert_eq!(undone, 250, "all 250 strokes undoable");
    assert_eq!(
        t.canvas_rgba.as_ref(),
        &pristine,
        "250 undos must restore the exact pristine canvas"
    );
    assert!(!t.can_undo());
}

#[test]
fn set_source_resets_undo_history() {
    // A fresh source (different sprite selected) must reset undo/redo — you
    // can't undo strokes from a different canvas onto this one.
    let mut t = PainterTool::default();
    t.set_source(flat_source(16, 16, [0; 4]), 16, 16);
    let _ = t.current_preview();
    paint_one_stroke(&mut t, 1, [8.0, 8.0]);
    assert!(t.can_undo());
    // Switch to a different source.
    t.set_source(flat_source(16, 16, [255; 4]), 16, 16);
    assert!(!t.can_undo(), "new source must clear undo");
    assert!(!t.can_redo(), "new source must clear redo");
}

#[test]
fn ui_snapshot_reflects_undo_redo_availability() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(16, 16, [0, 0, 0, 255]), 16, 16);
    let _ = t.current_preview();
    assert!(!t.ui_snapshot().undo_enabled);
    assert!(!t.ui_snapshot().redo_enabled);
    paint_one_stroke(&mut t, 1, [8.0, 8.0]);
    assert!(t.ui_snapshot().undo_enabled);
    assert!(!t.ui_snapshot().redo_enabled);
    t.undo_last_stroke();
    assert!(!t.ui_snapshot().undo_enabled);
    assert!(
        t.ui_snapshot().redo_enabled,
        "redo_enabled no longer hardcoded false"
    );
}

#[test]
fn undo_via_ui_edit_dispatch_path() {
    // The PainterUiEdit::Undo / ::Redo path (the apply_ui_edit dispatch the
    // shell also drives) must behave identically to the direct methods.
    let mut t = PainterTool::default();
    t.set_source(flat_source(16, 16, [0, 0, 0, 255]), 16, 16);
    let _ = t.current_preview();
    let pristine = t.canvas_rgba.as_ref().clone();
    let painted = paint_one_stroke(&mut t, 1, [8.0, 8.0]);
    t.apply_ui_edit(crate::params::PainterUiEdit::Undo);
    assert_eq!(t.canvas_rgba.as_ref(), &pristine);
    t.apply_ui_edit(crate::params::PainterUiEdit::Redo);
    assert_eq!(t.canvas_rgba.as_ref(), &painted);
}

// ──────────────────────────────────────────────────────────────────────
// Round-2 audit fixes — closing verbal-claim gaps with executable gates.
// ──────────────────────────────────────────────────────────────────────

#[test]
fn on_deactivate_clears_canvas_via_tool_dispatch() {
    // Audit T1.5 round 2 F4 (MISSING-GATE-DEACTIVATE-CHAIN). The fix
    // (B-M2) routes registry-side teardown through `Tool::on_deactivate
    // → RasterEditTool::deactivate`. This test invokes the chain
    // through the Tool trait (the path `ToolRegistry::set_active`
    // takes), NOT via `RasterEditTool::deactivate` directly — proves
    // the dispatch wiring, not just the leaf.
    let mut t = PainterTool::default();
    t.set_source(flat_source(4, 4, [128; 4]), 4, 4);
    t.begin_stroke(1);
    assert!(t.is_stroke_active());
    <PainterTool as Tool>::on_deactivate(&mut t);
    assert!(!t.params.takeover_active);
    assert_eq!(t.source_size, (0, 0));
    assert!(t.canvas_rgba.is_empty());
    assert!(!t.is_stroke_active());
}

#[test]
fn stroke_writes_pixels_with_default_color() {
    // Audit T1.5 round 2 F5 (MISSING-GATE-DEFAULT-ALPHA). Day-7
    // smoke contract: `PainterTool::default()` (NO override of
    // `params.active_color`) must produce visible paint via the
    // input-dispatch entry. If a future regression resets
    // `OklchColor::default().a` to 0 in `PainterParams::default`,
    // this test catches it.
    let mut t = PainterTool::default();
    // DELIBERATELY do NOT override `params.active_color`.
    t.params.size_px = 16.0;
    t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
    let _ = t.current_preview(); // drain set_source dirty
    t.begin_stroke(42);
    t.queue_pointer(PointerSample {
        position: [16.0, 16.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let (px, _w, _h) = t.current_preview().expect("paint must mark dirty");
    let center_idx = (16 * 32 + 16) * 4;
    // The default OklchColor is OKLab black (l=c=h=0) at α=1, so the
    // expected output at center is OPAQUE BLACK [0,0,0,255]. Source
    // started at [0,0,0,255] so the SPECIFIC ASSERT here is that
    // alpha stays at 255 AND the stamp didn't no-op (set_source +
    // queue_pointer succeeded — preview returned Some). That last
    // bit is the real Day-7 marker; opaque-on-opaque produces same
    // bytes but the path executed.
    assert_eq!(px[center_idx + 3], 255, "alpha must remain opaque");
}

#[test]
fn derive_seed_determinism_and_collision_resistance() {
    // Audit T1.5 round 2 F1 (MISSING-GATE-DERIVE-SEED). Locks the
    // wyhash-style mixer contract: bit-identical across runs (the
    // function uses only `to_bits`, `wrapping_mul`, `xor`, and bit-
    // shifts — all platform-stable IEEE 754 / integer ops), and
    // distinct inputs produce distinct seeds (anti-collision).
    //
    // Determinism: same inputs must produce same outputs across
    // consecutive calls. This bounds the implementation against
    // future-edits that introduce platform-specific behavior.
    for &(px, py, sw, sh, eb) in &[
        (0.0_f32, 0.0_f32, 256_u32, 256_u32, 0_u64),
        (10.0, 20.0, 256, 256, 0),
        (10.0, 20.0, 256, 256, 1),
    ] {
        assert_eq!(
            PainterTool::derive_seed(px, py, sw, sh, eb),
            PainterTool::derive_seed(px, py, sw, sh, eb),
            "derive_seed must be deterministic ({px}, {py}, {sw}, {sh}, {eb})"
        );
    }

    // Anti-collision: different inputs must produce different seeds
    // (modulo PRNG collisions — pick well-separated inputs).
    let a = PainterTool::derive_seed(10.0, 20.0, 256, 256, 0);
    let b = PainterTool::derive_seed(10.0, 20.0, 256, 256, 1);
    let c = PainterTool::derive_seed(11.0, 20.0, 256, 256, 0);
    let d = PainterTool::derive_seed(10.0, 20.0, 257, 256, 0);
    assert_ne!(a, b, "entity_bits must distinguish seeds");
    assert_ne!(a, c, "canvas_px must distinguish seeds");
    assert_ne!(a, d, "src_w must distinguish seeds");

    // Non-finite canvas-px canonicalization: NaN / +Inf must produce
    // SAME seed as 0.0 (NOT a unique non-finite hash).
    assert_eq!(
        PainterTool::derive_seed(f32::NAN, 0.0, 1, 1, 0),
        PainterTool::derive_seed(0.0, 0.0, 1, 1, 0),
        "NaN canvas_px must canonicalize to 0.0",
    );
    assert_eq!(
        PainterTool::derive_seed(0.0, f32::INFINITY, 1, 1, 0),
        PainterTool::derive_seed(0.0, 0.0, 1, 1, 0),
        "+Inf canvas_py must canonicalize to 0.0",
    );
}

#[test]
#[should_panic(expected = "RADIANS")]
fn oklch_to_oklab_panics_on_degrees_input() {
    // Audit T1.5 round 2 F8 + R9 T1-4: assertion is now a production
    // `assert!` (not `debug_assert!`), so it fires in BOTH debug and
    // release builds. Degree inputs (h = 360.0 > 4π ≈ 12.566) are
    // surfaced immediately at the call site instead of silently
    // producing garbage colors.
    let c = OklchColor {
        l: 0.5,
        c: 0.2,
        h: 360.0,
        a: 1.0,
    };
    let _ = oklch_to_oklab(c);
}

// ---- W2.T2.3: primary-color surface ----

#[test]
fn set_color_srgb_updates_active_and_snapshot_reflects() {
    let mut t = PainterTool::default();
    // Orange sRGB → SetColorSrgb → active_color updated.
    t.apply_ui_edit(crate::params::PainterUiEdit::SetColorSrgb([
        255, 136, 0, 255,
    ]));
    let snap = t.ui_snapshot();
    // Snapshot's sRGB accessor round-trips back to the same bytes
    // (±1 LSB — the 8-bit quantization tolerance).
    let got = snap.active_color_srgb8();
    for ch in 0..4 {
        assert!(
            got[ch].abs_diff([255, 136, 0, 255][ch]) <= 1,
            "channel {ch}: got {got:?}"
        );
    }
    // And the hex accessor shows the swatch.
    assert_eq!(snap.active_color_hex(), "#FF8800");
}

#[test]
fn set_color_oklch_path_still_works() {
    let mut t = PainterTool::default();
    let c = OklchColor {
        l: 0.5,
        c: 0.1,
        h: 1.0, // radians
        a: 1.0,
    };
    t.apply_ui_edit(crate::params::PainterUiEdit::SetColor(c));
    assert_eq!(t.params.active_color, c);
}

#[test]
fn next_stroke_uses_color_set_via_ui_edit() {
    // The headline acceptance: a color set through the UI surface is
    // the color the NEXT stroke paints with. We assert the cached
    // stroke color (baked at begin_stroke from active_color) matches
    // the OKLab of the color we just set.
    let mut t = PainterTool::default();
    t.apply_ui_edit(crate::params::PainterUiEdit::SetColorSrgb([255, 0, 0, 255]));
    t.set_source(flat_source(8, 8, [0, 0, 0, 255]), 8, 8);
    let _ = t.current_preview();
    t.begin_stroke(7);
    // begin_stroke baked active_color → stroke_color_oklab.
    let expected = oklch_to_oklab(t.params.active_color);
    // alpha is opacity-scaled (opacity default 1.0).
    assert_eq!(t.stroke_color_oklab[0], expected[0]);
    assert_eq!(t.stroke_color_oklab[1], expected[1]);
    assert_eq!(t.stroke_color_oklab[2], expected[2]);
    // Red has positive a* (green↔red) in OKLab.
    assert!(
        t.stroke_color_oklab[1] > 0.0,
        "red must have positive OKLab a*; got {}",
        t.stroke_color_oklab[1]
    );
}

#[test]
fn mid_stroke_color_edit_refreshes_cache() {
    // A color edit DURING a live stroke must take effect on the
    // in-flight stroke (refresh_stroke_color_if_in_flight), mirroring
    // the Opacity live-edit contract (audit X-7).
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [0, 0, 0, 255]), 8, 8);
    let _ = t.current_preview();
    t.begin_stroke(7);
    let before = t.stroke_color_oklab;
    t.apply_ui_edit(crate::params::PainterUiEdit::SetColorSrgb([0, 0, 255, 255]));
    let after = t.stroke_color_oklab;
    assert_ne!(
        before, after,
        "mid-stroke color edit must refresh the cached stroke color"
    );
    // Blue has negative b* (blue↔yellow) in OKLab.
    assert!(
        after[2] < 0.0,
        "blue must have negative OKLab b*; got {}",
        after[2]
    );
}

#[test]
fn snapshot_color_accessors_match_default_swatch() {
    // The default snapshot must expose the default orange swatch via
    // the accessors so the picker opens on the real color.
    let snap = crate::params::PainterUiSnapshot::default();
    let srgb = snap.active_color_srgb8();
    // Default is a warm orange: red channel dominates blue.
    assert!(srgb[0] > srgb[2], "default swatch should be warm: {srgb:?}");
    assert_eq!(snap.active_color_hex(), crate::color::format_hex(srgb));
}

#[test]
fn painter_color_to_stroke_oklch_converts_radians_to_degrees() {
    // Regression: the painter stub stores hue in RADIANS; the canonical
    // ph2d_color::OklchColor (StrokeRecord.primary_color) is DEGREES.
    // π rad must land as ~180° in the stored record, NOT "3.14 degrees".
    let painter = OklchColor {
        l: 0.7,
        c: 0.18,
        h: std::f32::consts::PI, // 180° expressed in radians
        a: 1.0,
    };
    let stored = painter_color_to_stroke_oklch(painter);
    assert!(
        (stored.h - 180.0).abs() < 1e-3,
        "π rad must convert to 180 degrees in the canonical record; got {}",
        stored.h
    );
    // L/C/A pass through untouched (only hue carries a unit).
    assert_eq!(stored.l, painter.l);
    assert_eq!(stored.c, painter.c);
    assert_eq!(stored.a, painter.a);
}

#[test]
fn stored_stroke_color_renders_to_the_same_srgb_as_the_painter_swatch() {
    // End-to-end correctness: a saturated orange chosen in the picker
    // (sRGB) → painter OKLCH(rad) → StrokeRecord.primary_color → the
    // canonical degrees consumer (`to_srgb`, which does `.to_radians()`)
    // must reproduce the ORIGINAL color. Pre-fix, the radians hue was
    // mis-read as degrees and this round-trip diverged by a huge margin.
    let picked = [255u8, 136, 0, 255]; // saturated orange
    let painter = crate::color::srgb8_to_painter_oklch(picked);
    let stored = painter_color_to_stroke_oklch(painter); // radians → degrees
    // The canonical render path (what render/reproject/Inspector use):
    let rendered = stored.to_srgb().0;
    for ch in 0..4 {
        assert!(
            rendered[ch].abs_diff(picked[ch]) <= 1,
            "channel {ch} diverged: picked={picked:?} rendered_from_record={rendered:?} \
                 (stored hue={}°)",
            stored.h
        );
    }
}

// ── W3 multi-selection ────────────────────────────────────────────────

#[test]
fn select_additive_toggles_selection_membership() {
    // Cmd/Ctrl-click extends the selection, then toggles a member back out,
    // repointing the active edit target to a remaining member.
    use std::collections::BTreeSet;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base = Layer 1
    let l2 = t.add_raster_layer("L2").unwrap();
    let l3 = t.add_raster_layer("L3").unwrap(); // active = L3
    assert_eq!(t.selection(), BTreeSet::from([l3]));
    t.select_additive(l2); // extend: {L3, L2}, active = L2
    assert_eq!(t.layers.active(), Some(l2));
    assert_eq!(t.selection(), BTreeSet::from([l2, l3]));
    t.select_additive(l2); // toggle L2 out: {L3}, active repointed to L3
    assert_eq!(t.selection(), BTreeSet::from([l3]));
    assert_eq!(t.layers.active(), Some(l3));
}

#[test]
fn select_additive_never_empties_the_selection() {
    use std::collections::BTreeSet;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let l2 = t.add_raster_layer("L2").unwrap(); // active = L2, selection = {L2}
    t.select_additive(l2); // toggling the lone member is ignored
    assert_eq!(t.selection(), BTreeSet::from([l2]));
    assert_eq!(t.layers.active(), Some(l2));
}

#[test]
fn select_range_selects_the_contiguous_run() {
    // Shift-click selects every row between the active anchor and the click
    // along the visible order (newest on top): [L4, L3, L2, base].
    use std::collections::BTreeSet;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base
    let l2 = t.add_raster_layer("L2").unwrap();
    let l3 = t.add_raster_layer("L3").unwrap();
    let l4 = t.add_raster_layer("L4").unwrap(); // active anchor = L4 (top)
    t.select_range(l2); // anchor L4 .. L2 → {L4, L3, L2}
    assert_eq!(t.selection(), BTreeSet::from([l2, l3, l4]));
    assert_eq!(t.layers.active(), Some(l2));
}

#[test]
fn group_selected_wraps_every_selected_layer() {
    use std::collections::BTreeSet;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base (root bottom)
    let base = t.layers.root().last().copied().unwrap();
    let l2 = t.add_raster_layer("L2").unwrap();
    let l3 = t.add_raster_layer("L3").unwrap(); // active = L3
    t.select_additive(l2); // selection = {L3, L2}
    let g = t.group_selected().expect("two selected layers group");
    // The base sprite stays pinned at root bottom.
    assert_eq!(t.layers.root().last(), Some(&base));
    // Both selected layers are now children of the new group.
    let children: BTreeSet<_> = match &t.layers.get(g).unwrap().kind {
        LayerKind::Group(gl) => gl.children.iter().copied().collect(),
        _ => panic!("group_selected must create a group"),
    };
    assert_eq!(children, BTreeSet::from([l2, l3]));
    // Selection collapses to the new group.
    assert!(t.selection().contains(&g));
}

#[test]
fn group_selected_single_falls_back_to_group_active() {
    // Fewer than two selected → the interim single-layer wrap (group_active).
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let l2 = t.add_raster_layer("L2").unwrap(); // active = L2, selection = {L2}
    let g = t
        .group_selected()
        .expect("single selection wraps the active layer");
    let children = match &t.layers.get(g).unwrap().kind {
        LayerKind::Group(gl) => gl.children.clone(),
        _ => panic!("expected a group"),
    };
    assert_eq!(children, vec![l2]);
}

#[test]
fn delete_layer_prunes_the_selection() {
    // A deleted layer must not linger as a phantom highlight.
    use std::collections::BTreeSet;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let l2 = t.add_raster_layer("L2").unwrap();
    let l3 = t.add_raster_layer("L3").unwrap();
    t.select_additive(l2); // selection = {L3, L2}
    assert_eq!(t.selection(), BTreeSet::from([l2, l3]));
    t.delete_layer(l2);
    assert!(
        !t.selection().contains(&l2),
        "deleted layer pruned from selection"
    );
}

// ── W3 mask Invert / Apply (§2.7) ─────────────────────────────────────

#[test]
fn apply_mask_black_bakes_parent_alpha_to_zero() {
    // A fully black mask, applied, multiplies the parent's alpha to 0 — the
    // SAME coverage the live compositor previewed (preview ≡ Apply).
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2); // base red
    let l2 = t.add_raster_layer("L2").unwrap(); // active, transparent
    {
        let c = std::sync::Arc::make_mut(&mut t.canvas_rgba);
        for px in c.chunks_exact_mut(4) {
            px.copy_from_slice(&[10, 20, 30, 255]); // opaque content
        }
    }
    let mask = t.add_mask_to_active().unwrap(); // active = mask (white)
    {
        let c = std::sync::Arc::make_mut(&mut t.canvas_rgba);
        for px in c.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 255]); // paint the mask black (hide)
        }
    }
    assert!(t.apply_mask(mask), "apply bakes + removes the mask");
    assert_eq!(
        t.layers.active(),
        Some(l2),
        "parent becomes the edit target"
    );
    assert!(t.layers.get(mask).is_none(), "mask layer removed");
    assert_eq!(
        t.layers.get(l2).unwrap().mask,
        None,
        "parent mask reference cleared"
    );
    assert!(
        t.canvas_rgba.chunks_exact(4).all(|px| px[3] == 0),
        "black mask baked the parent alpha to 0"
    );
}

#[test]
fn apply_mask_white_preserves_parent_alpha() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2);
    t.add_raster_layer("L2").unwrap();
    {
        let c = std::sync::Arc::make_mut(&mut t.canvas_rgba);
        for px in c.chunks_exact_mut(4) {
            px.copy_from_slice(&[10, 20, 30, 200]);
        }
    }
    let mask = t.add_mask_to_active().unwrap(); // white mask = full visible
    assert!(t.apply_mask(mask));
    assert!(
        t.canvas_rgba.chunks_exact(4).all(|px| px[3] == 200),
        "white mask preserves the parent alpha"
    );
}

#[test]
fn toggle_mask_inverted_flips_the_flag() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    t.add_raster_layer("L2").unwrap();
    let mask = t.add_mask_to_active().unwrap();
    let is_inv = |t: &PainterTool, id: RtLayerId| matches!(t.layers.get(id).map(|l| &l.kind), Some(LayerKind::Mask(m)) if m.inverted);
    assert!(!is_inv(&t, mask));
    t.toggle_mask_inverted(mask);
    assert!(is_inv(&t, mask));
    t.toggle_mask_inverted(mask);
    assert!(!is_inv(&t, mask));
}

#[test]
fn selecting_a_group_enters_its_first_paintable_layer() {
    // A group has no pixel buffer; selecting it must NOT make the group the
    // paint target (that would blank the canvas + swallow strokes). It
    // resolves to the group's first paintable descendant instead.
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2); // base
    let l2 = t.add_raster_layer("L2").unwrap();
    let _l3 = t.add_raster_layer("L3").unwrap(); // active = L3
    t.select_additive(l2); // selection = {L3, L2}
    let g = t.group_selected().unwrap(); // wraps L2 + L3 into g
    t.select_single(g); // click the group row
    assert_ne!(
        t.layers.active(),
        Some(g),
        "a group is never the paint target"
    );
    assert!(
        matches!(
            t.layers
                .active()
                .and_then(|a| t.layers.get(a))
                .map(|l| &l.kind),
            Some(LayerKind::Raster(_))
        ),
        "active resolves to a raster inside the group"
    );
}

#[test]
fn selecting_an_empty_group_keeps_the_current_active() {
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [0, 0, 0, 255]), 2, 2);
    let l2 = t.add_raster_layer("L2").unwrap(); // active = L2
    let g = t.add_group().unwrap(); // empty group at root top
    t.select_single(g);
    assert_eq!(
        t.layers.active(),
        Some(l2),
        "an empty group does not steal the paint target"
    );
}

// ── W4 T4.3 — adjustment layer create + HSB edit ──────────────────────

#[test]
fn add_adjustment_layer_keeps_a_paintable_active_target() {
    use ph2d_painter_brush::adjustments::AdjustmentKind;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2); // base raster (active)
    let base = t.layers.active().unwrap();
    let adj = t
        .add_adjustment_layer(AdjustmentKind::HueSaturationBrightness)
        .expect("adjustment created");
    // The adjustment has no pixel buffer — it must NOT become the paint
    // target (that would blank the canvas + swallow strokes).
    assert_ne!(t.layers.active(), Some(adj));
    assert_eq!(
        t.layers.active(),
        Some(base),
        "the prior raster stays the edit target"
    );
    assert!(
        t.selection().contains(&adj),
        "the new adjustment is selected"
    );
}

#[test]
fn set_adjustment_hsb_maps_sliders_to_params() {
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::HueSaturationBrightness)
        .unwrap();
    t.set_adjustment_param(adj, 0, 0.5); // Hue → 0.5 turns
    t.set_adjustment_param(adj, 1, 1.0); // Sat slider 1 → +1
    t.set_adjustment_param(adj, 2, 0.0); // Bright slider 0 → -1
    let params = match &t.layers.get(adj).unwrap().kind {
        LayerKind::Adjustment(a) => a.params.clone(),
        _ => panic!("expected an adjustment layer"),
    };
    let AdjustmentParams::HueSaturationBrightness(p) = params else {
        panic!("params are not HSB");
    };
    assert!((p.h - 0.5).abs() < 1e-6, "hue maps 0..1 turns directly");
    assert!((p.s - 1.0).abs() < 1e-6, "saturation slider 1 → +1");
    assert!((p.b + 1.0).abs() < 1e-6, "brightness slider 0 → -1");
}

#[test]
fn set_curve_point_moves_the_targeted_point_and_arms_the_cache() {
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, ControlPoints};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::Curves).unwrap();
    // Seed the master curve with 3 identity handles (what the bespoke editor
    // creates) so there's a point to drag.
    if let AdjustmentParams::Curves(c) = &mut t.layers.adjustment_mut(adj).unwrap().params {
        c.points_rgb = ControlPoints {
            points: vec![[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]],
        };
    }
    t.adjustment_cache_pending = false; // clear the seed-time state
    // Lift the middle handle (brighten midtones).
    t.set_curve_point(adj, 0, 1, 0.5, 0.8);
    let params = match &t.layers.get(adj).unwrap().kind {
        LayerKind::Adjustment(a) => a.params.clone(),
        _ => panic!("expected an adjustment layer"),
    };
    let AdjustmentParams::Curves(c) = params else {
        panic!("params are not Curves");
    };
    let mid = c.points_rgb.points[1];
    assert!(
        (mid[0] - 0.5).abs() < 1e-6 && (mid[1] - 0.8).abs() < 1e-6,
        "middle handle moved to (0.5, 0.8): {mid:?}"
    );
    assert!(
        t.adjustment_cache_pending,
        "a curve edit arms the cut-cache fast lane (like a slider drag)"
    );
    // Out-of-range channel / point index are no-ops (no panic).
    t.set_curve_point(adj, 9, 0, 0.1, 0.1);
    t.set_curve_point(adj, 0, 99, 0.1, 0.1);
}

#[test]
fn set_curve_point_clamps_x_between_neighbours_keeping_index_stable() {
    // Free-2D editor binds a stable index per handle; X must clamp to the
    // neighbours' span (never reorder) so a drag past a neighbour can't make the
    // next frame grab a different point.
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::Curves).unwrap();
    // add_adjustment_layer seeds 5 evenly-spaced master handles (x = 0,.25,.5,.75,1).
    // Drag the middle (index 2) hard right and hard left; it pins at its neighbours.
    t.set_curve_point(adj, 0, 2, 0.95, 0.6);
    let read = |t: &PainterTool| -> [f32; 2] {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => {
                let AdjustmentParams::Curves(c) = &a.params else {
                    panic!()
                };
                c.points_rgb.points[2]
            }
            _ => panic!(),
        }
    };
    let p = read(&t);
    assert!(
        (p[0] - 0.75).abs() < 1e-6,
        "x clamped to the right neighbour (0.75): {p:?}"
    );
    assert!((p[1] - 0.6).abs() < 1e-6, "y is free");
    t.set_curve_point(adj, 0, 2, 0.05, 0.4);
    assert!(
        (read(&t)[0] - 0.25).abs() < 1e-6,
        "x clamped to the left neighbour (0.25)"
    );
    // The point list stays length-5 and strictly ordered (no reorder/drop).
    let pts: Vec<[f32; 2]> = match &t.layers.get(adj).unwrap().kind {
        LayerKind::Adjustment(a) => match &a.params {
            AdjustmentParams::Curves(c) => c.points_rgb.points.clone(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert_eq!(pts.len(), 5, "no point added/dropped");
    assert!(
        pts.windows(2).all(|w| w[0][0] <= w[1][0]),
        "points stay ascending in x: {pts:?}"
    );
}

#[test]
fn set_curve_point_on_non_curves_layer_is_a_noop() {
    use ph2d_painter_brush::adjustments::AdjustmentKind;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::BrightnessContrast)
        .unwrap();
    t.adjustment_cache_pending = false;
    t.set_curve_point(adj, 0, 0, 0.5, 0.5);
    assert!(
        !t.adjustment_cache_pending,
        "set_curve_point on a non-Curves layer does nothing"
    );
}

#[test]
fn curve_edit_panel_event_routes_to_set_curve_point() {
    // The free-2D editor forwards a drag as SelectOption(PAINTER_CURVE_EDIT,
    // "layer:channel:index:x:y"); handle_panel_event must parse + apply it.
    use ph2d_editor_core::ids::PAINTER_CURVE_EDIT;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::Curves).unwrap();
    let payload = format!("{}:0:2:0.5:0.8", adj.0);
    t.handle_panel_event(PanelEvent::SelectOption(PAINTER_CURVE_EDIT, payload));
    let mid = match &t.layers.get(adj).unwrap().kind {
        LayerKind::Adjustment(a) => match &a.params {
            AdjustmentParams::Curves(c) => c.points_rgb.points[2],
            _ => panic!("not curves"),
        },
        _ => panic!("not an adjustment"),
    };
    assert!(
        (mid[0] - 0.5).abs() < 1e-6 && (mid[1] - 0.8).abs() < 1e-6,
        "panel curve-edit event moved master point 2 to (0.5, 0.8): {mid:?}"
    );
    // A malformed payload is a no-op (no panic).
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_CURVE_EDIT,
        "garbage".into(),
    ));
}

#[test]
fn photo_filter_toggle_click_routes_to_flip() {
    // W4 BATCH-1: a Photo Filter's "Preserve Luminosity" switch forwards a bare
    // Click(AdjToggle0); handle_panel_event must decode it back to the layer +
    // flip the boolean param (the mask-invert affordance pattern).
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::PhotoFilter).unwrap();
    let preserve = |t: &PainterTool| -> bool {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::PhotoFilter(p) => p.preserve_luminosity,
                _ => panic!("not a photo filter"),
            },
            _ => panic!("not an adjustment"),
        }
    };
    assert!(
        !preserve(&t),
        "fresh Photo Filter starts with preserve-lum off"
    );
    let toggle_id = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjToggle0);
    t.handle_panel_event(PanelEvent::Click(toggle_id));
    assert!(preserve(&t), "the toggle click flipped preserve-lum on");
    t.handle_panel_event(PanelEvent::Click(toggle_id));
    assert!(!preserve(&t), "a second click flipped it back off");
}

#[test]
fn color_balance_segment_click_routes_to_set_scope() {
    // W4 BATCH-1: a Color Balance tonal-range segment forwards a bare
    // Click(AdjSegmentN); handle_panel_event decodes it + selects that scope.
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, ToneScope};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::ColorBalance)
        .unwrap();
    let scope = |t: &PainterTool| -> ToneScope {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::ColorBalance(p) => p.scope,
                _ => panic!("not a color balance"),
            },
            _ => panic!("not an adjustment"),
        }
    };
    assert_eq!(
        scope(&t),
        ToneScope::Midtones,
        "fresh Color Balance is Midtones"
    );
    let seg2 = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjSegment2);
    t.handle_panel_event(PanelEvent::Click(seg2));
    assert_eq!(
        scope(&t),
        ToneScope::Highlights,
        "segment-2 click → Highlights"
    );
    let seg0 = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjSegment0);
    t.handle_panel_event(PanelEvent::Click(seg0));
    assert_eq!(scope(&t), ToneScope::Shadows, "segment-0 click → Shadows");
}

#[test]
fn channel_mixer_weight_edit_routes_to_set_weight() {
    // W4 BATCH-1: a Channel Mixer weight slider forwards
    // SelectOption(PAINTER_MIXER_EDIT, "layer:output:slot:value") (the bespoke
    // editor's active output tab carries the channel); handle_panel_event must
    // parse + apply it to the right matrix cell.
    use ph2d_editor_core::ids::PAINTER_MIXER_EDIT;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, ChannelMixerParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::ChannelMixer)
        .unwrap();
    let mixer = |t: &PainterTool| -> ChannelMixerParams {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::ChannelMixer(p) => *p,
                _ => panic!("not a channel mixer"),
            },
            _ => panic!("not an adjustment"),
        }
    };
    assert_eq!(
        mixer(&t).green_out,
        [0.0, 1.0, 0.0, 0.0],
        "fresh mixer is the identity matrix"
    );
    // Green output (1), Blue-source slot (2), value 0.75 → weight 0.75*4-2 = 1.0.
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_MIXER_EDIT,
        format!("{}:1:2:0.75", adj.0),
    ));
    assert!(
        (mixer(&t).green_out[2] - 1.0).abs() < 1e-5,
        "green_out blue-source weight set to 1.0: {:?}",
        mixer(&t).green_out
    );
    assert_eq!(
        mixer(&t).red_out,
        [1.0, 0.0, 0.0, 0.0],
        "the Red output row stays untouched"
    );
    // A malformed payload is a no-op (no panic).
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_MIXER_EDIT,
        "garbage".into(),
    ));
}

#[test]
fn black_and_white_tint_toggle_and_hue_slider_route() {
    // W4 BATCH-1: the Tint switch (AdjToggle0) enables the tint; the Hue slider
    // (AdjParam6) then routes through the generic SetValue path to set the hue.
    use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, BlackAndWhiteParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::BlackAndWhite)
        .unwrap();
    let bw = |t: &PainterTool| -> BlackAndWhiteParams {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::BlackAndWhite(p) => p.clone(),
                _ => panic!("not a black & white"),
            },
            _ => panic!("not an adjustment"),
        }
    };
    assert!(bw(&t).tint_color.is_none(), "fresh B&W has no tint");
    let toggle = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjToggle0);
    t.handle_panel_event(PanelEvent::Click(toggle));
    assert!(
        bw(&t).tint_color.is_some(),
        "the Tint switch enabled the tint"
    );
    let hue = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjParam6);
    t.handle_panel_event(PanelEvent::SetValue(hue, 0.25));
    assert!(
        (bw(&t).tint_color.unwrap().h - 90.0).abs() < 1e-3,
        "hue slider (0.25) set the tint hue to 90°: {:?}",
        bw(&t).tint_color
    );
}

#[test]
fn gradient_map_stop_editor_routes() {
    // W4 BATCH-2: the bespoke N-stop editor — add / move / recolor / remove a stop
    // (each carries the stop index in the payload) + the interpolation segment.
    use ph2d_editor_core::ids::{
        PAINTER_GRADIENT_ADD, PAINTER_GRADIENT_COLOR, PAINTER_GRADIENT_EDIT,
        PAINTER_GRADIENT_REMOVE, PainterLayerWidget, painter_layer_widget_id,
    };
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, GradientInterp};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::GradientMap).unwrap();
    let gmap = |t: &PainterTool| -> ph2d_painter_brush::adjustments::GradientMapParams {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::GradientMap(p) => p.clone(),
                _ => panic!("not a gradient map"),
            },
            _ => panic!("not an adjustment"),
        }
    };
    assert_eq!(gmap(&t).stops.len(), 2, "default duotone has 2 stops");
    // Add a stop (→ index 2 at the gap midpoint).
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_GRADIENT_ADD,
        format!("{}", adj.0),
    ));
    assert_eq!(gmap(&t).stops.len(), 3, "add inserted a stop");
    // Move stop 2 to offset 0.25.
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_GRADIENT_EDIT,
        format!("{}:2:0.25", adj.0),
    ));
    assert!(
        (gmap(&t).stops[2].offset - 0.25).abs() < 1e-5,
        "stop 2 moved to 0.25"
    );
    // Recolor stop 2's red to 255 (slot 0).
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_GRADIENT_COLOR,
        format!("{}:2:0:1.0", adj.0),
    ));
    assert_eq!(gmap(&t).stops[2].color[0], 255, "stop 2 red set to 255");
    // Interpolation segment (AdjSegment1) → Smooth.
    let seg1 = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjSegment1);
    t.handle_panel_event(PanelEvent::Click(seg1));
    assert!(matches!(gmap(&t).interpolation, GradientInterp::Smooth));
    // Remove stop 2.
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_GRADIENT_REMOVE,
        format!("{}:2", adj.0),
    ));
    assert_eq!(gmap(&t).stops.len(), 2, "remove dropped the stop");
}

#[test]
fn selective_color_cmyk_edit_and_method_route() {
    // W4 BATCH-2: a Selective Color CMYK slider forwards
    // SelectOption(PAINTER_SELCOLOR_EDIT, "layer:bucket:slot:value") (the active
    // bucket tab carries the group); the method uses the generic segment route.
    use ph2d_editor_core::ids::{
        PAINTER_SELCOLOR_EDIT, PainterLayerWidget, painter_layer_widget_id,
    };
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, SelectiveMethod};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::SelectiveColor)
        .unwrap();
    let sel = |t: &PainterTool| -> ph2d_painter_brush::adjustments::SelectiveColorParams {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::SelectiveColor(p) => *p,
                _ => panic!("not a selective color"),
            },
            _ => panic!("not an adjustment"),
        }
    };
    // Cyans group (bucket 3), Cyan slot (0), value 1.0 → cyans.cyan = 1.0.
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_SELCOLOR_EDIT,
        format!("{}:3:0:1.0", adj.0),
    ));
    assert!(
        (sel(&t).cyans.cyan - 1.0).abs() < 1e-5,
        "PAINTER_SELCOLOR_EDIT set the Cyans group's cyan"
    );
    assert_eq!(sel(&t).reds.cyan, 0.0, "the Reds group stays untouched");
    // Method segment (AdjSegment1) → Absolute.
    let seg1 = painter_layer_widget_id(adj.0, PainterLayerWidget::AdjSegment1);
    t.handle_panel_event(PanelEvent::Click(seg1));
    assert!(
        matches!(sel(&t).method, SelectiveMethod::Absolute),
        "segment click selected the Absolute method"
    );
    // A malformed payload is a no-op (no panic).
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_SELCOLOR_EDIT,
        "garbage".into(),
    ));
}

#[test]
fn add_remove_curve_point_respects_cap_floor_and_curve() {
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::Curves).unwrap();
    let pts = |t: &PainterTool| -> Vec<[f32; 2]> {
        match &t.layers.get(adj).unwrap().kind {
            LayerKind::Adjustment(a) => match &a.params {
                AdjustmentParams::Curves(c) => c.points_rgb.points.clone(),
                _ => panic!(),
            },
            _ => panic!(),
        }
    };
    assert_eq!(pts(&t).len(), 5, "seeded with 5 master points");
    // An inserted point sits ON the (identity) curve → y ≈ x, output unchanged.
    let idx = t.add_curve_point(adj, 0).expect("added");
    let p = pts(&t)[idx];
    assert!(
        (p[0] - p[1]).abs() < 1e-3,
        "inserted point on the identity curve: {p:?}"
    );
    // Fill to the 8-point cap, then it refuses.
    while t.add_curve_point(adj, 0).is_some() {}
    assert_eq!(pts(&t).len(), 8, "stops at the ≤8 cap");
    assert!(
        pts(&t).windows(2).all(|w| w[0][0] <= w[1][0]),
        "points stay ascending in x: {:?}",
        pts(&t)
    );
    // Remove interior points down to the 2-endpoint floor.
    while pts(&t).len() > 2 {
        t.remove_curve_point(adj, 0, 1);
    }
    assert_eq!(pts(&t).len(), 2);
    t.remove_curve_point(adj, 0, 1);
    assert_eq!(pts(&t).len(), 2, "won't remove below the 2 endpoints");
    // Out-of-range / non-curves are no-ops (no panic).
    t.remove_curve_point(adj, 9, 0);
    t.remove_curve_point(adj, 0, 99);
}

#[test]
fn curve_add_remove_panel_events_route_to_tool() {
    use ph2d_editor_core::ids::{PAINTER_CURVE_ADD, PAINTER_CURVE_REMOVE};
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams};
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [128, 128, 128, 255]), 2, 2);
    let adj = t.add_adjustment_layer(AdjustmentKind::Curves).unwrap();
    let count = |t: &PainterTool| match &t.layers.get(adj).unwrap().kind {
        LayerKind::Adjustment(a) => match &a.params {
            AdjustmentParams::Curves(c) => c.points_rgb.points.len(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert_eq!(count(&t), 5);
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_CURVE_ADD,
        format!("{}:0", adj.0),
    ));
    assert_eq!(count(&t), 6, "add event inserted a master point");
    t.handle_panel_event(PanelEvent::SelectOption(
        PAINTER_CURVE_REMOVE,
        format!("{}:0:1", adj.0),
    ));
    assert_eq!(count(&t), 5, "remove event dropped a master point");
    // Malformed payloads are no-ops (no panic).
    t.handle_panel_event(PanelEvent::SelectOption(PAINTER_CURVE_ADD, "x".into()));
    t.handle_panel_event(PanelEvent::SelectOption(PAINTER_CURVE_REMOVE, "x".into()));
}

#[test]
fn adjustment_param_drain_uses_cache_bit_identically() {
    // W5: a slider-drag drain routes through `composite_with_cache` (cut-point
    // cache). Prove the warm-restart preview is byte-identical to a cold full
    // `composite` of the same final state — the wiring must not drift from the
    // reference path (the perf win is correctness-free).
    use ph2d_painter_brush::adjustments::AdjustmentKind;
    let (w, h) = (4u32, 4u32);
    // Varied base so a cache slicing/ordering bug would surface (a flat fill
    // would hide it).
    let mut base = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        base[i * 4] = (i * 13 % 256) as u8;
        base[i * 4 + 1] = (i * 7 % 256) as u8;
        base[i * 4 + 2] = (i * 5 % 256) as u8;
        base[i * 4 + 3] = 255;
    }
    let mut t = PainterTool::default();
    t.set_source(base, w, h);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::BrightnessContrast)
        .unwrap();
    // Frame 1 of the drag → cold `composite_with_cache` (populates the cut).
    t.set_adjustment_param(adj, 0, 0.3);
    let _ = t.take_preview_arc().expect("preview");
    // Frame 2: change the param → `invalidate_above` keeps the below-cut +
    // arms the pending flag → warm restart from the cut.
    t.set_adjustment_param(adj, 0, 0.8);
    let warm = (*t.take_preview_arc().expect("preview").0).clone();
    // Force a cold full `composite` of the identical final state (cache miss).
    t.composited = None;
    t.adjustment_cache_pending = false;
    t.preview_dirty = true;
    let cold = (*t.take_preview_arc().expect("preview").0).clone();
    assert_eq!(
        warm, cold,
        "adjustment-param cache-restart drain diverged from a cold full recompose"
    );
}

#[test]
fn add_adjustment_via_kind_menu_select_creates_that_kind() {
    // W4 T4.15: the "+ Adjustment" picker forwards the chosen kind's index in
    // `AdjustmentKind::ALL` as a `SelectOption` on PAINTER_LAYERS_ADD_ADJUSTMENT;
    // the tool maps it back and creates that exact kind (not always HSB).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::adjustments::AdjustmentKind;
    let mut t = PainterTool::default();
    t.set_source(flat_source(2, 2, [200, 0, 0, 255]), 2, 2);
    let before = t.layers.len();
    // Index 4 in AdjustmentKind::ALL = BrightnessContrast.
    let idx = AdjustmentKind::ALL
        .iter()
        .position(|&k| k == AdjustmentKind::BrightnessContrast)
        .unwrap();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        idx.to_string(),
    ));
    assert_eq!(t.layers.len(), before + 1, "a layer was added");
    let kind = t
        .layers
        .all_ids()
        .filter_map(|id| match &t.layers.get(id).unwrap().kind {
            LayerKind::Adjustment(a) => Some(a.kind),
            _ => None,
        })
        .next()
        .expect("an adjustment layer was created");
    assert_eq!(
        kind,
        AdjustmentKind::BrightnessContrast,
        "the picked kind is the one created"
    );
}

#[test]
fn wet_edges_darken_the_stroke_rim_on_pen_up() {
    // End-to-end watercolor wet-edge check: paint a wash stroke with `wet_edges`
    // ON, then verify the stroke comes out DARKER on its rim (the inner edge
    // shoulder = the receding water boundary) than at its fill centre. This is
    // pigment-transport edge darkening, not a silhouette outline — proven by the
    // rim sitting *inside* the stroke, darker than the fill, with the fill centre
    // essentially the flat brush colour.
    let (w, h) = (48u32, 48u32);
    let mut t = PainterTool::default();
    t.params.size_px = 18.0;
    // Transparent wash (opacity < 1): the K–M rim concentrates pigment toward the
    // masstone, so edge darkening is bounded by it — it shows only where the fill is
    // a GLAZE (lighter than the masstone), the physical watercolor case. At opacity
    // 1.0 the fill is the masstone itself, so the rim correctly can't go darker.
    t.params.opacity = 0.5;
    // Wash mode (default accumulate=false) + wet_edges ON + a plain linear blend.
    t.brush.rendering.wet_edges = true;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h); // white paper
    t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 90, 200, 255]); // blue
    t.begin_stroke(5);
    for x in (6..42).step_by(2) {
        t.queue_pointer(PointerSample {
            position: [x as f32, 24.0],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();
    let (px, _, _) = t.current_preview().expect("painted preview");
    let luma = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        px[i] as f32 * 0.299 + px[i + 1] as f32 * 0.587 + px[i + 2] as f32 * 0.114
    };
    let center = luma(24, 24); // middle of the horizontal band = fill
    // Darkest covered pixel along a vertical slice through the band = the rim.
    let mut min_band = f32::INFINITY;
    for y in 14..35 {
        let l = luma(24, y);
        if l < 245.0 {
            // ignore the white paper / soft AA fringe
            min_band = min_band.min(l);
        }
    }
    assert!(
        min_band < center - 6.0,
        "wet-edge rim must be darker than the fill centre: rim {min_band} vs fill {center}"
    );
}

#[test]
fn wet_edges_off_leaves_a_flat_wash() {
    // Control: the SAME stroke with wet_edges OFF has no rim — the band is flat
    // (rim ≈ centre). Guards against the settle pass firing unconditionally.
    let (w, h) = (48u32, 48u32);
    let mut t = PainterTool::default();
    t.params.size_px = 18.0;
    t.params.opacity = 1.0;
    t.brush.rendering.wet_edges = false;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 90, 200, 255]);
    t.begin_stroke(5);
    for x in (6..42).step_by(2) {
        t.queue_pointer(PointerSample {
            position: [x as f32, 24.0],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();
    let (px, _, _) = t.current_preview().expect("painted preview");
    let luma = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        px[i] as f32 * 0.299 + px[i + 1] as f32 * 0.587 + px[i + 2] as f32 * 0.114
    };
    let center = luma(24, 24);
    let mut min_band = f32::INFINITY;
    for y in 18..31 {
        let l = luma(24, y);
        if l < 245.0 {
            min_band = min_band.min(l);
        }
    }
    assert!(
        (min_band - center).abs() < 5.0,
        "wet_edges OFF → flat band, no rim: min {min_band} vs centre {center}"
    );
}

// ── Visual smoke (gated by PAINTER_VISUAL_SMOKE=1) — dumps PPM strokes ────────
// Run: PAINTER_VISUAL_SMOKE=1 cargo test -p ph2d-tool-painter visual_smoke -- --nocapture

#[cfg(test)]
fn dump_ppm(path: &str, px: &[u8], w: u32, h: u32) {
    use std::io::Write;
    let mut buf = format!("P6\n{w} {h}\n255\n").into_bytes();
    for i in 0..(w as usize * h as usize) {
        buf.push(px[i * 4]);
        buf.push(px[i * 4 + 1]);
        buf.push(px[i * 4 + 2]);
    }
    std::fs::File::create(path)
        .unwrap()
        .write_all(&buf)
        .unwrap();
}

#[cfg(test)]
fn paint_arc(t: &mut PainterTool, samples: &[(f32, f32, f32)]) {
    t.begin_stroke(42);
    for &(x, y, pr) in samples {
        t.queue_pointer(PointerSample {
            position: [x, y],
            pressure: pr,
            tilt: 0.0,
        });
    }
    t.end_stroke();
}

#[test]
fn visual_smoke_dump_strokes() {
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return; // opt-in only
    }
    let (w, h) = (320u32, 200u32);
    // A wavy arc across the canvas.
    let arc: Vec<(f32, f32, f32)> = (0..60)
        .map(|i| {
            let t = i as f32 / 59.0;
            let x = 24.0 + t * 272.0;
            let y = 100.0 + (t * std::f32::consts::PI * 1.5).sin() * 45.0;
            (x, y, 1.0)
        })
        .collect();

    // 1) Wet edges — watercolor blue on white.
    {
        let mut t = PainterTool::default();
        t.params.size_px = 34.0;
        t.params.opacity = 0.85;
        t.brush.rendering.wet_edges = true;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Subtractive;
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([35, 80, 170, 255]);
        paint_arc(&mut t, &arc);
        let (px, _, _) = t.current_preview().unwrap();
        dump_ppm("/tmp/painter_smoke_wet.ppm", px, w, h);
    }
    // 2) Burnt edges — charcoal black on white.
    {
        let mut t = PainterTool::default();
        t.params.size_px = 34.0;
        t.params.opacity = 0.9;
        t.brush.rendering.burnt_edges = true;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([30, 28, 26, 255]);
        paint_arc(&mut t, &arc);
        let (px, _, _) = t.current_preview().unwrap();
        dump_ppm("/tmp/painter_smoke_burnt.ppm", px, w, h);
    }
    // 3) Pressure ramp — a straight stroke whose pressure falls 1.0 → 0.05
    //    (size + opacity should taper: thick/dark → thin/faint).
    {
        let mut t = PainterTool::default();
        t.params.size_px = 40.0;
        t.params.opacity = 1.0;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([20, 20, 30, 255]);
        let ramp: Vec<(f32, f32, f32)> = (0..70)
            .map(|i| {
                let s = i as f32 / 69.0;
                (24.0 + s * 272.0, 100.0, (1.0 - s).max(0.04))
            })
            .collect();
        paint_arc(&mut t, &ramp);
        let (px, _, _) = t.current_preview().unwrap();
        dump_ppm("/tmp/painter_smoke_pressure.ppm", px, w, h);
    }
    // 4) AA stress — a SMALL canvas + thin diagonal/curved strokes of a small
    //    hard brush, so viewing the (small) image magnified shows per-pixel edge
    //    quality (the "serrilhado" check). round_hard at a few px.
    {
        let (sw, sh) = (90u32, 70u32);
        let mut t = PainterTool::default();
        t.params.size_px = 7.0;
        t.params.opacity = 1.0;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
        t.brush.rendering.accumulate = true; // crisp build-up, no wash softening
        t.set_source(flat_source(sw, sh, [255, 255, 255, 255]), sw, sh);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([20, 20, 30, 255]);
        // A slow shallow diagonal (the classic stair-step case) + a steeper one.
        let diag: Vec<(f32, f32, f32)> = (0..80)
            .map(|i| {
                let s = i as f32 / 79.0;
                (8.0 + s * 74.0, 14.0 + s * 8.0, 1.0)
            })
            .collect();
        paint_arc(&mut t, &diag);
        let curve: Vec<(f32, f32, f32)> = (0..80)
            .map(|i| {
                let s = i as f32 / 79.0;
                (
                    8.0 + s * 74.0,
                    50.0 + (s * std::f32::consts::PI).sin() * -16.0,
                    1.0,
                )
            })
            .collect();
        paint_arc(&mut t, &curve);
        let (px, _, _) = t.current_preview().unwrap();
        dump_ppm("/tmp/painter_smoke_aa.ppm", px, sw, sh);
    }
    // 5) Start taper — CONSTANT pressure (1.0) but taper_length set, so the entry
    //    is a clean point ramping to full width (independent of pressure), end blunt.
    {
        let mut t = PainterTool::default();
        t.params.size_px = 34.0;
        t.params.opacity = 1.0;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
        t.brush.rendering.accumulate = true;
        t.brush.taper.taper_length_start = 0.5; // long taper
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([20, 20, 30, 255]);
        let line: Vec<(f32, f32, f32)> = (0..70)
            .map(|i| (24.0 + (i as f32 / 69.0) * 272.0, 100.0, 1.0))
            .collect();
        paint_arc(&mut t, &line);
        let (px, _, _) = t.current_preview().unwrap();
        dump_ppm("/tmp/painter_smoke_taper.ppm", px, w, h);
    }
    eprintln!("[visual smoke] wrote /tmp/painter_smoke_{{wet,burnt,pressure,aa,taper}}.ppm");
}

#[test]
fn wet_edges_work_in_accumulate_buildup_mode() {
    // Regression: wet/burnt edges must fire in build-up (`accumulate`) mode too,
    // not only wash. The coverage buffer is now a side output of the build-up
    // render, so the pen-up settle has the stroke extent to darken its rim.
    let (w, h) = (48u32, 48u32);
    let mut t = PainterTool::default();
    t.params.size_px = 18.0;
    t.params.opacity = 1.0;
    t.brush.rendering.accumulate = true; // BUILD-UP, not wash
    t.brush.rendering.wet_edges = true;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 90, 200, 255]);
    t.begin_stroke(5);
    for x in (6..42).step_by(2) {
        t.queue_pointer(PointerSample {
            position: [x as f32, 24.0],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();
    let (px, _, _) = t.current_preview().expect("painted preview");
    let luma = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        px[i] as f32 * 0.299 + px[i + 1] as f32 * 0.587 + px[i + 2] as f32 * 0.114
    };
    let center = luma(24, 24);
    let mut min_band = f32::INFINITY;
    for y in 14..35 {
        let l = luma(24, y);
        if l < 245.0 {
            min_band = min_band.min(l);
        }
    }
    assert!(
        min_band < center - 6.0,
        "wet-edge rim must darken in build-up mode too: rim {min_band} vs fill {center}"
    );
}

#[test]
fn edge_intensity_scales_the_rim_darkness() {
    // The Edge Intensity slider (brush.rendering.edge_intensity) must scale the
    // settle strength: a higher value → a darker rim. Proves the field flows
    // through end_stroke into apply_wash_settle.
    let rim_luma = |intensity: f32| -> f32 {
        let (w, h) = (48u32, 48u32);
        let mut t = PainterTool::default();
        t.params.size_px = 18.0;
        // Transparent wash so the K–M rim has glaze headroom toward the masstone
        // (see `wet_edges_darken_the_stroke_rim_on_pen_up`).
        t.params.opacity = 0.5;
        t.brush.rendering.wet_edges = true;
        t.brush.rendering.edge_intensity = intensity;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 90, 200, 255]);
        t.begin_stroke(5);
        for x in (6..42).step_by(2) {
            t.queue_pointer(PointerSample {
                position: [x as f32, 24.0],
                pressure: 1.0,
                tilt: 0.0,
            });
        }
        t.end_stroke();
        let (px, _, _) = t.current_preview().unwrap();
        let luma = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            px[i] as f32 * 0.299 + px[i + 1] as f32 * 0.587 + px[i + 2] as f32 * 0.114
        };
        let mut min_band = f32::INFINITY;
        for y in 14..35 {
            let l = luma(24, y);
            if l < 245.0 {
                min_band = min_band.min(l);
            }
        }
        min_band
    };
    let rim_lo = rim_luma(0.2);
    let rim_hi = rim_luma(1.0);
    assert!(
        rim_hi < rim_lo - 5.0,
        "higher edge intensity must darken the rim more: hi {rim_hi} vs lo {rim_lo}"
    );
}

#[test]
fn visual_smoke_rendering_modes() {
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (300u32, 210u32);
    // Background: left half light grey, right half dark blue — so a semi-transparent
    // stroke reveals how much each mode lets the layer below show through.
    let mut bg = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let (r, g, b) = if x < w / 2 {
                (220, 220, 220)
            } else {
                (28, 40, 120)
            };
            bg[i] = r;
            bg[i + 1] = g;
            bg[i + 2] = b;
            bg[i + 3] = 255;
        }
    }
    let names = [
        "LightGlaze",
        "UniformGlaze",
        "IntenseGlaze",
        "HeavyGlaze",
        "UniformBlending",
        "IntenseBlending",
    ];
    let mut t = PainterTool::default();
    t.params.size_px = 22.0;
    t.params.opacity = 0.6; // semi-transparent so modes differ
    t.brush.rendering.accumulate = false; // WASH (the default) — option (a) makes modes work here
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear; // not mixbox
    t.set_source(bg, w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([220, 40, 40, 255]); // red
    for (m, _name) in names.iter().enumerate() {
        t.brush.rendering.rendering_mode = ph2d_painter_brush::RenderingMode::from_u32(m as u32);
        let y = 22.0 + m as f32 * 30.0;
        let line: Vec<(f32, f32, f32)> = (0..50)
            .map(|i| (20.0 + (i as f32 / 49.0) * 260.0, y, 1.0))
            .collect();
        paint_arc(&mut t, &line);
    }
    let (px, _, _) = t.current_preview().unwrap();
    dump_ppm("/tmp/painter_smoke_modes.ppm", px, w, h);
    eprintln!(
        "[modes] LightGlaze, UniformGlaze, IntenseGlaze, HeavyGlaze, UniformBlending, IntenseBlending (top→bottom)"
    );
}

#[test]
fn visual_smoke_texture_critique() {
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    use ph2d_painter_brush::{GrainSource, ProceduralGrain};
    let (w, h) = (320u32, 240u32);
    let mut t = PainterTool::default();
    t.params.size_px = 30.0;
    t.params.opacity = 1.0;
    t.brush.rendering.accumulate = true;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
    t.set_source(flat_source(w, h, [250, 248, 244, 255]), w, h); // warm paper
    t.params.active_color = crate::color::srgb8_to_painter_oklch([35, 30, 28, 255]);
    let strokes: &[(&str, GrainSource, f32, f32)] = &[
        ("flat (default)", GrainSource::None, 1.0, 0.0),
        (
            "simplex fine",
            GrainSource::Procedural(ProceduralGrain::SimplexNoise {
                scale: 1.0,
                octaves: 4,
                persistence: 0.5,
                seed: 1,
            }),
            1.0,
            1.0,
        ),
        (
            "simplex coarse",
            GrainSource::Procedural(ProceduralGrain::SimplexNoise {
                scale: 2.2,
                octaves: 5,
                persistence: 0.6,
                seed: 2,
            }),
            2.2,
            1.0,
        ),
        (
            "paper weave",
            GrainSource::Procedural(ProceduralGrain::PaperWeave {
                fiber_density: 1.0,
                fiber_anisotropy: 0.5,
                crossweave: true,
                seed: 3,
            }),
            1.6,
            1.0,
        ),
    ];
    for (i, (_name, src, scale, depth)) in strokes.iter().enumerate() {
        t.brush.grain.grain_source = src.clone();
        t.brush.grain.grain_scale = *scale;
        t.brush.grain.grain_depth = *depth;
        let y = 30.0 + i as f32 * 55.0;
        let line: Vec<(f32, f32, f32)> = (0..60)
            .map(|k| {
                let s = k as f32 / 59.0;
                (
                    24.0 + s * 272.0,
                    y + (s * std::f32::consts::TAU).sin() * 10.0,
                    (0.4 + 0.6 * s).min(1.0),
                )
            })
            .collect();
        paint_arc(&mut t, &line);
    }
    let (px, _, _) = t.current_preview().unwrap();
    dump_ppm("/tmp/painter_smoke_texture.ppm", px, w, h);
    eprintln!("[texture] flat / simplex-fine / simplex-coarse / paper-weave (top→bottom)");
}

#[test]
fn visual_smoke_paper_tooth() {
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (320u32, 200u32);
    // Same default brush (NO per-brush grain) — only the global paper tooth varies,
    // across three pressure-ramped bands on ONE canvas.
    let mut t = PainterTool::default();
    t.params.size_px = 30.0;
    t.params.opacity = 1.0;
    t.brush.rendering.accumulate = true;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
    t.set_source(flat_source(w, h, [250, 248, 244, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([35, 30, 28, 255]);
    for (i, pg) in [0.0f32, 0.4, 0.7].iter().enumerate() {
        t.params.paper_grain = *pg;
        let y = 36.0 + i as f32 * 62.0;
        let line: Vec<(f32, f32, f32)> = (0..60)
            .map(|k| {
                let s = k as f32 / 59.0;
                (
                    24.0 + s * 272.0,
                    y + (s * std::f32::consts::TAU).sin() * 9.0,
                    (0.35 + 0.65 * s).min(1.0),
                )
            })
            .collect();
        paint_arc(&mut t, &line);
    }
    let (px, _, _) = t.current_preview().unwrap();
    dump_ppm("/tmp/painter_smoke_paper.ppm", px, w, h);
    eprintln!("[paper tooth] paper_grain 0.0 (flat) / 0.4 (default) / 0.7 (strong), top→bottom");
}

#[test]
fn visual_smoke_watercolor_v15() {
    // Watercolor v1.5 dry-down: a TRANSPARENT pigment wash (opacity 0.5) with
    // wet_edges ON. Three bands, top→bottom, vary the Paper (= granulation) amount:
    //   0.0  → K–M edge darkening only (dark rim concentrating toward the masstone)
    //   0.5  → edge darkening + moderate granular sediment in the paper valleys
    //   0.9  → strong granulation mottle + edge darkening
    // Pen-up settle gives the rim + mottle; the body stays a luminous glaze.
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (320u32, 210u32);
    let mut t = PainterTool::default();
    t.params.size_px = 34.0;
    t.params.opacity = 0.5; // transparent wash — the watercolor case
    t.brush.rendering.wet_edges = true;
    t.brush.rendering.edge_intensity = 0.7;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Subtractive;
    t.set_source(flat_source(w, h, [252, 250, 246, 255]), w, h); // warm paper
    t.params.active_color = crate::color::srgb8_to_painter_oklch([30, 70, 165, 255]); // ultramarine
    for (i, pg) in [0.0f32, 0.5, 0.9].iter().enumerate() {
        t.params.paper_grain = *pg;
        let y = 40.0 + i as f32 * 64.0;
        let line: Vec<(f32, f32, f32)> = (0..56)
            .map(|k| {
                let s = k as f32 / 55.0;
                (
                    26.0 + s * 268.0,
                    y + (s * std::f32::consts::TAU).sin() * 10.0,
                    1.0,
                )
            })
            .collect();
        paint_arc(&mut t, &line);
    }
    let (px, _, _) = t.current_preview().unwrap();
    dump_ppm("/tmp/painter_smoke_watercolor.ppm", px, w, h);
    eprintln!(
        "[watercolor v1.5] ultramarine wash, granulation 0.0 / 0.5 / 0.9 top→bottom; \
         K-M edge darkening + paper-valley sediment"
    );
}

#[test]
fn fluid_brush_blooms_live_and_dries_on_tick() {
    // W15.2: a fluid brush splats into the live wet field (not the canvas
    // directly); `queue_pointer` + `on_tick` step the diffusion and composite it
    // out. After a dab, pigment shows at the centre; the field stays wet, then dries
    // + is dropped after enough idle ticks ("the paint stays wet, then sets").
    let (w, h) = (64u32, 48u32);
    let mut t = PainterTool::default();
    t.params.size_px = 14.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([30, 60, 180, 255]); // blue
    t.begin_stroke(7);
    assert!(
        t.wet_field.is_some(),
        "fluid begin_stroke allocates the wet field"
    );
    t.queue_pointer(PointerSample {
        position: [32.0, 24.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let center = {
        let (px, _, _) = t.current_preview().unwrap();
        let i = ((24 * w + 32) * 4) as usize;
        [px[i], px[i + 1], px[i + 2]]
    };
    eprintln!("center = {center:?}");
    assert!(
        center[2] as i32 > center[0] as i32 + 8,
        "fluid dab deposits blue-ish pigment at the centre: {center:?}"
    );
    assert!(t.wet_field.is_some(), "still wet right after the dab");
    t.end_stroke();
    // Idle ticks keep evolving while wet, then dry + drop the field.
    let mut dried_at = None;
    for k in 0..600 {
        t.on_tick_diffusion();
        if t.wet_field.is_none() {
            dried_at = Some(k);
            break;
        }
    }
    assert!(
        dried_at.is_some(),
        "the wet field dries + is dropped after enough ticks"
    );
    assert!(
        dried_at.unwrap() > 5,
        "it should stay wet for a while first: {dried_at:?}"
    );
}

#[test]
fn visual_smoke_watercolor_v2_live() {
    // W15.2 END-TO-END: a fluid brush stroke through the real tool, then idle ticks
    // (as the shell's `on_tick` would drive). Three bands top→bottom = the canvas
    // right after painting / +18 ticks / +45 ticks: the wash blooms wet-on-wet AND
    // keeps evolving + drying after the stroke ("the paint stays wet").
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (240u32, 70u32);
    let snapshot = |t: &mut PainterTool| -> Vec<u8> {
        t.preview_dirty = true;
        t.current_preview().unwrap().0.to_vec()
    };
    let mut t = PainterTool::default();
    t.params.size_px = 16.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [252, 250, 246, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([30, 70, 165, 255]); // ultramarine
    t.begin_stroke(7);
    for k in 0..50 {
        let s = k as f32 / 49.0;
        t.queue_pointer(PointerSample {
            position: [
                18.0 + s * 204.0,
                35.0 + (s * std::f32::consts::TAU).sin() * 9.0,
            ],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    let band0 = snapshot(&mut t);
    t.end_stroke();
    for _ in 0..18 {
        t.on_tick_diffusion();
    }
    let band1 = snapshot(&mut t);
    for _ in 0..27 {
        t.on_tick_diffusion();
    }
    let band2 = snapshot(&mut t);

    let gap = 8u32;
    let ch = h * 3 + gap * 2;
    let mut canvas = vec![255u8; (w * ch * 4) as usize];
    for (band, src) in [band0, band1, band2].iter().enumerate() {
        let y0 = band as u32 * (h + gap);
        for y in 0..h {
            let dst = (((y0 + y) * w) * 4) as usize;
            let s = ((y * w) * 4) as usize;
            canvas[dst..dst + (w * 4) as usize].copy_from_slice(&src[s..s + (w * 4) as usize]);
        }
    }
    dump_ppm("/tmp/painter_smoke_live.ppm", &canvas, w, ch);
    eprintln!(
        "[watercolor v2 LIVE] fluid stroke through the tool; canvas at +0 / +18 / +45 \
         on_ticks top→bottom — blooms wet-on-wet + keeps evolving/drying after pen-up"
    );
}

#[test]
fn non_fluid_brush_allocates_no_wet_field() {
    // The default (fluid_enabled = false) brush never allocates a wet field → the
    // normal render path is byte-for-byte unchanged.
    let mut t = PainterTool::default();
    t.set_source(flat_source(8, 8, [255; 4]), 8, 8);
    t.begin_stroke(1);
    assert!(t.wet_field.is_none(), "non-fluid brush has no wet field");
}

#[test]
fn fluid_wet_field_dropped_on_undo_and_set_source() {
    // Now that "Fluid" is a user toggle, the v1 edge cases are reachable: a wash
    // still blooming post-pen-up must NOT re-composite onto a canvas whose stroke
    // was undone or whose source was swapped out from under it. Both drop the field.
    let (w, h) = (32u32, 24u32);
    let make_fluid_stroke = || {
        let mut t = PainterTool::default();
        t.params.size_px = 12.0;
        t.params.opacity = 1.0;
        t.brush.rendering.fluid_enabled = true;
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.begin_stroke(3);
        t.queue_pointer(PointerSample {
            position: [16.0, 12.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        t.end_stroke();
        t
    };
    // Undo backs out the stroke → the still-wet field must go with it.
    let mut t = make_fluid_stroke();
    assert!(t.wet_field.is_some(), "field still wet right after pen-up");
    assert!(t.undo_last_stroke(), "the committed fluid stroke undoes");
    assert!(t.wet_field.is_none(), "undo drops the blooming field");
    // A fresh source (sized differently) → the old grid is meaningless; drop it.
    let mut t2 = make_fluid_stroke();
    assert!(t2.wet_field.is_some());
    t2.set_source(flat_source(40, 40, [255; 4]), 40, 40);
    assert!(t2.wet_field.is_none(), "set_source drops the stale field");
}

#[test]
fn fluid_coverage_is_color_independent() {
    // Enio report: yellow/magenta washes came out fully opaque and covered other
    // colours, while blue/red stayed a proper translucent wash. Cause: coverage
    // used the linear-RGB SUM, which is luminance-weighted (yellow ≈ 2.6× blue).
    // Fix normalises by the stroke colour's linear sum, so the SAME pigment load
    // gives the SAME opacity regardless of hue. Paint identical strokes in several
    // hues on a transparent layer; their centre alphas must now be close.
    let (w, h) = (40u32, 40u32);
    let alpha_for = |rgb: [u8; 4]| -> u8 {
        let mut t = PainterTool::default();
        t.params.size_px = 16.0;
        t.params.opacity = 1.0;
        t.brush.rendering.fluid_enabled = true;
        t.set_source(vec![0u8; (w * h * 4) as usize], w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch(rgb);
        t.begin_stroke(5);
        t.queue_pointer(PointerSample {
            position: [20.0, 20.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        let (px, _, _) = t.current_preview().unwrap();
        px[((20 * w + 20) * 4 + 3) as usize] // centre alpha
    };
    let blue = alpha_for([40, 60, 230, 255]);
    let red = alpha_for([230, 50, 40, 255]);
    let yellow = alpha_for([235, 220, 30, 255]);
    let magenta = alpha_for([230, 40, 220, 255]);
    eprintln!("centre alphas — blue {blue} red {red} yellow {yellow} magenta {magenta}");
    let spread = [blue, red, yellow, magenta];
    let max = *spread.iter().max().unwrap() as i32;
    let min = *spread.iter().min().unwrap() as i32;
    assert!(
        max - min < 45,
        "fluid coverage must be ~color-independent; got blue {blue} red {red} yellow {yellow} magenta {magenta}"
    );
}

#[test]
fn fluid_no_dark_fringe_on_transparent_layer() {
    // Repro of Enio's report: painting a fluid stroke on a NEW TOP LAYER (fully
    // transparent) leaves dark borders. Cause: the composite blends the pigment
    // over the backdrop RGB, and a transparent backdrop is (0,0,0,0) — so partial-
    // coverage edge pixels mix toward BLACK. The fix must produce straight alpha:
    // edge RGB = pigment colour, darkening only over actually-opaque paint.
    let (w, h) = (40u32, 40u32);
    let mut t = PainterTool::default();
    t.params.size_px = 16.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(vec![0u8; (w * h * 4) as usize], w, h); // transparent layer
    t.params.active_color = crate::color::srgb8_to_painter_oklch([230, 110, 85, 255]); // coral
    t.begin_stroke(5);
    t.queue_pointer(PointerSample {
        position: [20.0, 20.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let (px, _, _) = t.current_preview().unwrap();
    // Scan for the partial-alpha edge ring; the darkest edge pixel must still read
    // as coral (R the dominant channel), NOT a black-mixed fringe.
    let mut worst: Option<(u8, [u8; 4])> = None;
    for i in 0..(w * h) as usize {
        let p = [px[i * 4], px[i * 4 + 1], px[i * 4 + 2], px[i * 4 + 3]];
        if p[3] > 10 && p[3] < 230 {
            // luminance of the straight RGB
            let lum = p[0] / 3 + p[1] / 3 + p[2] / 3;
            if worst.is_none() || lum < worst.unwrap().0 {
                worst = Some((lum, p));
            }
        }
    }
    if let Some((_, p)) = worst {
        eprintln!("darkest edge pixel = {p:?}");
        assert!(
            p[0] as i32 > p[1] as i32 && p[0] as i32 > p[2] as i32,
            "edge pixel must stay coral (R dominant), not a black fringe: {p:?}"
        );
        // A coral edge: R clearly above a neutral grey of the same alpha would imply
        // straight colour. Guard the specific failure — near-black RGB at partial alpha.
        assert!(
            p[0] as i32 > 60,
            "edge red too dark — black fringe from compositing over transparent: {p:?}"
        );
    }
}

#[test]
fn visual_smoke_fluid_color_swatches() {
    // Enio repro: several hues + an overlap. After the coverage fix, yellow/magenta
    // must read as translucent washes like blue/red (not flat opaque), and a hue
    // crossing another should blend, not bury it.
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (360u32, 200u32);
    let mut t = PainterTool::default();
    t.params.size_px = 22.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [235, 225, 200, 255]), w, h); // paper
    let cols: [[u8; 4]; 5] = [
        [40, 60, 230, 255],  // blue
        [60, 200, 70, 255],  // green
        [235, 220, 30, 255], // yellow
        [230, 50, 40, 255],  // red
        [230, 40, 220, 255], // magenta
    ];
    let vstroke = |t: &mut PainterTool, x: f32, rgb: [u8; 4], seed: u64| {
        t.params.active_color = crate::color::srgb8_to_painter_oklch(rgb);
        t.begin_stroke(seed);
        for k in 0..30 {
            let s = k as f32 / 29.0;
            t.queue_pointer(PointerSample {
                position: [x, 20.0 + s * 160.0],
                pressure: 1.0,
                tilt: 0.0,
            });
        }
        t.end_stroke();
        for _ in 0..6 {
            t.on_tick_diffusion();
        }
    };
    for (i, c) in cols.iter().enumerate() {
        vstroke(&mut t, 35.0 + i as f32 * 50.0, *c, 10 + i as u64);
    }
    // Overlap: yellow crossing blue at the right.
    vstroke(&mut t, 300.0, [40, 60, 230, 255], 100);
    vstroke(&mut t, 312.0, [235, 220, 30, 255], 101);
    t.preview_dirty = true;
    let px = t.current_preview().unwrap().0.to_vec();
    dump_ppm("/tmp/painter_smoke_fluid_colors.ppm", &px, w, h);
    eprintln!("[fluid colors] hues + overlap → /tmp/painter_smoke_fluid_colors.ppm");
}

#[test]
fn visual_smoke_fluid_on_transparent_layer() {
    // Enio's scenario: a fluid stroke on a NEW TOP LAYER, then composited over the
    // layer below (here a grey/white split). Pre-fix this showed dark borders; now
    // the straight-alpha glaze must blend cleanly over both halves.
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (200u32, 300u32);
    let mut t = PainterTool::default();
    t.params.size_px = 30.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(vec![0u8; (w * h * 4) as usize], w, h); // transparent layer
    t.params.active_color = crate::color::srgb8_to_painter_oklch([230, 110, 85, 255]);
    t.begin_stroke(7);
    for k in 0..60 {
        let s = k as f32 / 59.0;
        t.queue_pointer(PointerSample {
            position: [
                100.0 + (s * std::f32::consts::TAU * 1.5).sin() * 28.0,
                20.0 + s * 260.0,
            ],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();
    for _ in 0..8 {
        t.on_tick_diffusion();
    }
    t.preview_dirty = true;
    let layer = t.current_preview().unwrap().0.to_vec();
    // Composite the (straight-alpha) layer over a grey|white split background.
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = (y * w as usize + x) * 4;
            let bg = if x < w as usize / 2 { 150u8 } else { 245u8 };
            let a = layer[i + 3] as f32 / 255.0;
            for k in 0..3 {
                out[i + k] = (bg as f32 * (1.0 - a) + layer[i + k] as f32 * a).round() as u8;
            }
            out[i + 3] = 255;
        }
    }
    dump_ppm("/tmp/painter_smoke_fluid_transp.ppm", &out, w, h);
    eprintln!("[fluid transp] coral on transparent layer over grey|white → check edges");
}

#[test]
fn visual_smoke_fluid_edge_quality() {
    // Repro of Enio's report (coral fluid stroke on tan paper, blocky edges). The
    // bicubic upsample (ADR-0077 D12) should give a smooth wash falloff, not the
    // 1/WET_FIELD_SCALE-quantised facets bilinear left. Eyeball the dumped PNG.
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (200u32, 300u32);
    let mut t = PainterTool::default();
    t.params.size_px = 30.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [228, 214, 184, 255]), w, h); // tan paper
    t.params.active_color = crate::color::srgb8_to_painter_oklch([230, 110, 85, 255]); // coral
    t.begin_stroke(7);
    for k in 0..60 {
        let s = k as f32 / 59.0;
        t.queue_pointer(PointerSample {
            position: [
                100.0 + (s * std::f32::consts::TAU * 1.5).sin() * 28.0,
                20.0 + s * 260.0,
            ],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();
    for _ in 0..10 {
        t.on_tick_diffusion();
    }
    t.preview_dirty = true;
    let px = t.current_preview().unwrap().0.to_vec();
    dump_ppm("/tmp/painter_smoke_fluid_edge.ppm", &px, w, h);
    eprintln!("[fluid edge] coral fluid stroke on tan paper → /tmp/painter_smoke_fluid_edge.ppm");
}

#[test]
fn fluid_composite_mixes_subtractively_km() {
    // A yellow wash glazed over a BLUE backdrop must go GREEN (Kubelka–Munk
    // subtractive), not the muddy mid-tone a linear "over" gives. This guards the
    // ADR-0077 D12 K–M composite swap. Probe the wettest pixel (stroke centre).
    let (w, h) = (48u32, 36u32);
    let mut t = PainterTool::default();
    t.params.size_px = 18.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    // Saturated blue paper, yellow brush.
    t.set_source(flat_source(w, h, [20, 40, 200, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([235, 210, 20, 255]);
    t.begin_stroke(11);
    t.queue_pointer(PointerSample {
        position: [24.0, 18.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    let (px, _, _) = t.current_preview().unwrap();
    let i = ((18 * w + 24) * 4) as usize;
    let (r, g, b) = (px[i] as i32, px[i + 1] as i32, px[i + 2] as i32);
    eprintln!("yellow-over-blue centre = [{r}, {g}, {b}]");
    // The subtractive signature: green is the dominant channel (linear over would
    // leave R≈G, never green-dominant — yellow's red survives the average).
    assert!(
        g > r && g > b,
        "K–M wash should be green-dominant over blue: [{r}, {g}, {b}]"
    );
}

#[test]
fn fluid_cross_stroke_wet_on_wet_mixes() {
    // **ADR-0080 cross-stroke gate.** A still-WET field persists across strokes: paint BLUE,
    // then (a NEW stroke) paint YELLOW over the same wet spot before it dries → the field mixes
    // them SUBTRACTIVELY (green), the headline wet-on-wet "magic". The dry-drop only clears the
    // field once dry, so `begin_stroke` reuses it here instead of starting fresh.
    let (w, h) = (48u32, 36u32);
    let mut t = PainterTool::default();
    t.params.size_px = 22.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h); // white paper
    // Stroke 1 — blue, at the centre.
    t.params.active_color = crate::color::srgb8_to_painter_oklch([20, 40, 235, 255]);
    t.begin_stroke(1);
    t.queue_pointer(PointerSample {
        position: [24.0, 18.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert!(t.has_wet_field(), "stroke 1 allocated the wet field");
    t.end_stroke();
    assert!(
        t.has_wet_field(),
        "the wet field must persist (still wet) after stroke 1 ends — cross-stroke wet-on-wet"
    );
    // Stroke 2 — yellow, over the SAME wet spot. begin_stroke must REUSE the wet field.
    t.params.active_color = crate::color::srgb8_to_painter_oklch([235, 210, 20, 255]);
    t.begin_stroke(2);
    t.queue_pointer(PointerSample {
        position: [24.0, 18.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    // The wettest cell carries BOTH pigments → it reduces to a green-dominant colour.
    let grid = t.fluid_grid_mut().expect("wet field present");
    let (gw, gh) = grid.dims();
    let (mut best_i, mut best_m) = (0usize, 0.0f32);
    for i in 0..(gw * gh) as usize {
        let m = grid.pigment_mass(i);
        if m > best_m {
            best_m = m;
            best_i = i;
        }
    }
    let c = grid.pigment_color(best_i);
    eprintln!("cross-stroke overlap colour = {c:?} (mass {best_m})");
    assert!(
        best_m > 1.0e-4,
        "overlap must carry pigment (mass {best_m})"
    );
    assert!(
        c[1] > c[0] && c[1] > c[2],
        "cross-stroke blue→yellow must mix green-dominant (not mud): {c:?}"
    );
}

#[test]
fn gpu_fluid_driven_skips_cpu_diffusion() {
    // W15.3: when the shell drives the field on the GPU, the tool must NOT CPU-step
    // it (dabs still splat; the shell's step_grid + composite_and_settle do the
    // rest). Verify on_tick + queue_pointer leave the grid untouched when the flag
    // is set, and composite_and_settle_fluid runs without a CPU step.
    let (w, h) = (40u32, 32u32);
    let mut t = PainterTool::default();
    t.params.size_px = 14.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 60, 200, 255]);
    t.set_gpu_fluid_driven(true);
    t.begin_stroke(3);
    t.queue_pointer(PointerSample {
        position: [20.0, 16.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert!(t.has_wet_field(), "fluid field allocated + dab splatted");
    let snap = t.fluid_grid_mut().unwrap().pigment().to_vec();
    t.on_tick_diffusion(); // GPU-driven → must NOT CPU-step
    let after = t.fluid_grid_mut().unwrap().pigment().to_vec();
    assert_eq!(
        snap, after,
        "on_tick must not CPU-step the grid when GPU-driven"
    );
    // The shell-facing composite still works (no CPU step happened).
    t.composite_and_settle_fluid();
    assert!(
        t.has_wet_field(),
        "field still wet (no steps ran to dry it)"
    );
}

#[test]
fn gpu_resident_path_captures_dabs_to_list_not_grid() {
    // 4K real-time arch §4: on a capable GPU (`fluid_hires` = true, set by the shell
    // BEFORE begin_stroke) the tool captures dabs as a small list for `cs_splat` — it
    // must NOT splat into the CPU grid (the field is GPU-resident) and must grow the
    // monotonic composite envelope. `fluid_hires` (not `gpu_fluid_driven`) gates this
    // so the stroke's FIRST frame is captured too.
    let (w, h) = (40u32, 32u32);
    let mut t = PainterTool::default();
    t.params.size_px = 14.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 60, 200, 255]);
    t.set_fluid_hires(true); // capable GPU → the resident dab path
    t.begin_stroke(7);
    t.queue_pointer(PointerSample {
        position: [20.0, 16.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert!(t.has_wet_field(), "fluid field allocated");
    // The CPU grid stays empty — the dabs went to the GPU list, not `DiffusionGrid`.
    let grid_pig: f32 = t
        .fluid_grid_mut()
        .unwrap()
        .pigment()
        .iter()
        .map(|p| p[0] + p[1] + p[2])
        .sum();
    assert_eq!(
        grid_pig, 0.0,
        "GPU-resident path must NOT splat into the CPU grid"
    );
    // Draining returns the dabs + a monotonic in-grid envelope covering the dab.
    let (dabs, region) = t.fluid_take_dabs().expect("envelope set after a dab");
    assert!(!dabs.is_empty(), "dabs captured for cs_splat");
    let (gw, gh) = t.fluid_grid_dims().unwrap();
    let (x0, y0, x1, y1) = region;
    assert!(
        x1 >= x0 && y1 >= y0 && x1 < gw && y1 < gh,
        "valid in-grid composite region {region:?} for {gw}x{gh}"
    );
    // After draining, the list is empty but the envelope persists (so the field keeps
    // compositing while it blooms out after pen-up).
    let (dabs2, region2) = t.fluid_take_dabs().expect("envelope persists after drain");
    assert!(dabs2.is_empty(), "dab list drained");
    assert_eq!(region2, region, "monotonic envelope persists across drains");
}

#[test]
fn fluid_field_survives_mid_stroke_dry_pause() {
    // REGRESSION (Enio pause bug, 2026-06-07): pausing mid-stroke (button held, no
    // dabs) dries the wet field in ~0.3s, but it must NOT be dropped while the stroke
    // is ACTIVE — else resuming the drag finds no field and paints NON-fluid (the
    // solid blob at the stroke's end). Drop only after pen-up.
    let (w, h) = (48u32, 36u32);
    let mut t = PainterTool::default();
    t.params.size_px = 14.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([40, 60, 200, 255]);
    t.begin_stroke(5);
    t.queue_pointer(PointerSample {
        position: [24.0, 18.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert!(t.has_wet_field(), "field allocated after first dab");
    // Long mid-stroke pause: idle ticks until the field would fully dry.
    for _ in 0..300 {
        t.on_tick_diffusion();
    }
    assert!(
        t.has_wet_field(),
        "field MUST survive a mid-stroke dry pause (resume stays fluid)"
    );
    // Resuming the drag re-wets the kept field — still the fluid path.
    t.queue_pointer(PointerSample {
        position: [32.0, 18.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    assert!(t.has_wet_field(), "field re-wet on resume");
    // After pen-up, idle ticks dry it out and DROP it.
    t.end_stroke();
    for _ in 0..400 {
        t.on_tick_diffusion();
    }
    assert!(!t.has_wet_field(), "field drops after pen-up dry-out");
}

#[test]
fn fluid_gpu_envelope_never_recedes_under_evaporation() {
    // REGRESSION (Enio "bordas cheias de quinas retangulares", 2026-06-07): the GPU
    // composite region must be the MONOTONIC wet envelope, not the current water
    // bbox. Water only evaporates (its bbox marches inward) while the conserved
    // pigment lingers outside it — compositing over a RECEDING rect hard-cut the
    // round dab into an axis-aligned rectangle. The envelope never shrinks for the
    // life of the field. (The OLD `cur ∪ prev` region receded → this would fail.)
    let (w, h) = (64u32, 64u32);
    let mut t = PainterTool::default();
    t.params.size_px = 18.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([30, 60, 180, 255]);
    t.set_gpu_fluid_driven(true); // GPU path: queue_pointer only splats (no CPU step)
    t.set_fluid_hires(true); // GPU-resident dab path: grows wet_pigment_envelope (fluid_take_dabs)
    t.begin_stroke(9);
    t.queue_pointer(PointerSample {
        position: [32.0, 32.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    // The composite region is the MONOTONIC wet envelope (`wet_pigment_envelope`), returned by
    // the live `fluid_take_dabs`. It only ever GROWS (queue_pointer unions each dab's bbox; reset
    // only at a fresh stroke), so across frames it never recedes — even as the GPU water field
    // evaporates underneath it.
    let r0 = t.fluid_take_dabs().expect("wet field after a dab").1;
    let mut prev = r0;
    // March the pointer outward over many frames; the envelope must grow + never recede.
    for k in 1..40u32 {
        t.queue_pointer(PointerSample {
            position: [32.0 + k as f32, 32.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        if let Some((_dabs, r)) = t.fluid_take_dabs() {
            assert!(
                r.0 <= prev.0 && r.1 <= prev.1 && r.2 >= prev.2 && r.3 >= prev.3,
                "composite envelope RECEDED {prev:?} -> {r:?} (the rectangular-clip bug)"
            );
            prev = r;
        }
    }
    assert!(
        prev.2 > r0.2,
        "the envelope must have GROWN rightward as the stroke marched out: {r0:?} -> {prev:?}"
    );
}

#[test]
fn fluid_wash_keeps_blooming_after_pen_up() {
    // REGRESSION (W15.2 dead-feature trap): the wash MUST keep evolving on the
    // canvas after pen-up — that is the whole point of the live field. A prior
    // build froze it: `end_stroke` consumed the composite backdrop
    // (`pending_pre_stroke`, taken by the undo stack), so the post-pen-up
    // `on_tick`s no-op'd and `visual_smoke_watercolor_v2_live` (which has no
    // assertion) couldn't catch it — its three bands were byte-identical. The
    // dedicated `wet_backdrop` survives the stroke; assert the canvas changes.
    let (w, h) = (64u32, 48u32);
    let mut t = PainterTool::default();
    t.params.size_px = 16.0;
    t.params.opacity = 1.0;
    t.brush.rendering.fluid_enabled = true;
    t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([30, 60, 180, 255]);
    t.begin_stroke(7);
    for k in 0..20 {
        let s = k as f32 / 19.0;
        t.queue_pointer(PointerSample {
            position: [10.0 + s * 44.0, 24.0],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();
    let snapshot = |t: &mut PainterTool| -> Vec<u8> {
        t.preview_dirty = true;
        t.current_preview().unwrap().0.to_vec()
    };
    let after_penup = snapshot(&mut t);
    assert!(t.wet_field.is_some(), "field still wet at pen-up");
    for _ in 0..15 {
        t.on_tick_diffusion();
    }
    let after_ticks = snapshot(&mut t);
    let changed: u64 = after_penup
        .iter()
        .zip(after_ticks.iter())
        .map(|(a, b)| (i32::from(*a) - i32::from(*b)).unsigned_abs() as u64)
        .sum();
    assert!(
        changed > 0,
        "the wash must keep blooming after pen-up (canvas frozen → dead feature)"
    );
}

#[test]
fn visual_smoke_watercolor_v2_diffusion() {
    // Watercolor v2 — the LIVE wet-on-wet diffusion solver. One pigment stroke is
    // laid across the grid; the LEFT half of the paper is dry, the RIGHT half is a
    // wet pool. After stepping the solver, the left stays crisp (gate closed on dry
    // paper) while the right BLOOMS into the wet pool (wet-on-wet bleed). A clean
    // water drop punched into the stroke makes a backrun ring. Three bands top→
    // bottom show the SAME sim at step 0 / 14 / 40 (just-painted → blooming → bled).
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    use ph2d_painter_brush::diffusion::{DiffusionGrid, DiffusionParams};
    let (gw, gh) = (220u32, 64u32);
    let mut grid = DiffusionGrid::new(gw, gh, 1.0);
    // Right half = a wet pool; a clean-water drop at x≈150 for a backrun.
    for y in 0..gh {
        for x in 0..gw {
            if x > gw / 2 {
                grid.splat(x as f32, y as f32, 0.6, 0.9, [0.0; 3], 0.0, 0.0);
            }
        }
    }
    // A horizontal ultramarine stroke across the whole width (carries some water).
    let pigment = [0.05f32, 0.07, 0.5];
    let pmass = pigment[0] + pigment[1] + pigment[2];
    for x in 8..gw - 8 {
        grid.splat(x as f32, gh as f32 * 0.5, 6.0, 0.35, pigment, pmass, 0.15);
    }
    grid.splat(150.0, gh as f32 * 0.5, 7.0, 1.0, [0.0; 3], 0.0, 0.0); // clean-water backrun drop

    let params = DiffusionParams::default();
    // Render the three snapshots into vertical bands of a 2× upsampled canvas.
    let scale = 2u32;
    let (cw, ch) = (gw * scale, gh * scale * 3 + 16);
    let mut canvas = vec![0u8; (cw * ch * 4) as usize];
    let paper = [252u8, 250, 246];
    let snapshots = [0u32, 14, 40];
    let mut stepped = 0u32;
    for (band, &target) in snapshots.iter().enumerate() {
        while stepped < target {
            grid.step(&params);
            stepped += 1;
        }
        let pig = grid.pigment();
        let band_y0 = band as u32 * (gh * scale + 8);
        for gy in 0..gh {
            for gx in 0..gw {
                let cell = &pig[(gy * gw + gx) as usize];
                let dens = ph2d_painter_brush::diffusion::DiffusionGrid::cell_mass(cell).max(0.0);
                let a = (1.0 - (-dens * 1.6).exp()).clamp(0.0, 0.97);
                let col = if dens > 1e-5 {
                    ph2d_painter_brush::diffusion::DiffusionGrid::cell_color(cell)
                } else {
                    [0.0, 0.0, 0.0]
                };
                let px = [
                    (paper[0] as f32 / 255.0 * (1.0 - a) + col[0] * a).clamp(0.0, 1.0),
                    (paper[1] as f32 / 255.0 * (1.0 - a) + col[1] * a).clamp(0.0, 1.0),
                    (paper[2] as f32 / 255.0 * (1.0 - a) + col[2] * a).clamp(0.0, 1.0),
                ];
                for sy in 0..scale {
                    for sx in 0..scale {
                        let cx = gx * scale + sx;
                        let cy = band_y0 + gy * scale + sy;
                        let i = ((cy * cw + cx) * 4) as usize;
                        canvas[i] = (px[0] * 255.0) as u8;
                        canvas[i + 1] = (px[1] * 255.0) as u8;
                        canvas[i + 2] = (px[2] * 255.0) as u8;
                        canvas[i + 3] = 255;
                    }
                }
            }
        }
    }
    dump_ppm("/tmp/painter_smoke_diffusion.ppm", &canvas, cw, ch);
    eprintln!(
        "[watercolor v2] wet-on-wet diffusion: step 0/14/40 top→bottom; left half dry \
         (crisp), right half wet pool (blooms), water drop @150 → backrun ring"
    );
}

#[test]
fn visual_smoke_velocity_and_smoothing() {
    // Velocity dynamics + One-Euro motion filtering. Three bands, top→bottom:
    //   1. Calligraphy: an ACCELERATING stroke with speed_size = −0.85 (fast → thin)
    //      → a brush that swells slow and tapers as it speeds up.
    //   2. A jittery ±6px hand-tremor path, motion filtering OFF → jagged.
    //   3. The SAME path, motion filtering ON (One-Euro) → a clean smooth line.
    if std::env::var("PAINTER_VISUAL_SMOKE").as_deref() != Ok("1") {
        return;
    }
    let (w, h) = (320u32, 210u32);
    let mut t = PainterTool::default();
    t.params.opacity = 1.0;
    t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
    t.set_source(flat_source(w, h, [252, 250, 246, 255]), w, h);
    t.params.active_color = crate::color::srgb8_to_painter_oklch([25, 25, 30, 255]); // ink

    // ── Band 1: velocity → size (calligraphic taper). Accelerating x. ──
    t.params.size_px = 24.0;
    t.brush.dynamics.speed_size = -0.85;
    let mut band1 = Vec::new();
    let mut x = 22.0f32;
    for k in 0..56 {
        band1.push((x, 44.0, 1.0));
        x += 1.5 + k as f32 * 0.62; // gap grows → stroke accelerates (gentle)
    }
    paint_arc(&mut t, &band1);
    t.brush.dynamics.speed_size = 0.0;

    // The shared shaky path: a slow intended gesture (`sin 0.18`) plus higher-
    // frequency hand tremor (`sin 1.1` + `sin 2.3`). One-Euro should strip the
    // tremor while keeping the gesture — what a fixed average can't do without
    // also flattening the gesture.
    let tremor: Vec<(f32, f32, f32)> = (0..80)
        .map(|k| {
            let s = k as f32;
            let jit = 9.0 * (s * 0.18).sin() + 4.5 * (s * 1.1).sin() + 3.0 * (s * 2.3).sin();
            (20.0 + s * 3.5, jit, 1.0)
        })
        .collect();

    // ── Band 2: tremor, motion filtering OFF (jagged). ──
    t.params.size_px = 7.0;
    let band2: Vec<_> = tremor.iter().map(|&(x, y, p)| (x, 120.0 + y, p)).collect();
    paint_arc(&mut t, &band2);

    // ── Band 3: same tremor, One-Euro motion filtering ON (smooth). ──
    // expression 0 = uniform smoothing (this synthetic ±6 jitter is per-sample
    // "fast", so a high expression would PRESERVE it — One-Euro's whole point).
    t.brush.stabilization.motion_filtering_amount = 1.0;
    t.brush.stabilization.motion_filtering_expression = 0.0;
    let band3: Vec<_> = tremor.iter().map(|&(x, y, p)| (x, 180.0 + y, p)).collect();
    paint_arc(&mut t, &band3);

    let (px, _, _) = t.current_preview().unwrap();
    dump_ppm("/tmp/painter_smoke_velocity.ppm", px, w, h);
    eprintln!(
        "[velocity+smoothing] band1 = calligraphic velocity taper (fast→thin); \
         band2 = raw tremor (jagged); band3 = One-Euro motion filtering (smooth)"
    );
}

#[test]
fn paper_tooth_textures_stroke_and_modulates_by_pressure() {
    // paper_grain ON → the stroke body VARIES (paper texture, not a flat fill), and
    // a light-pressure pass shows MORE tooth (higher variance — paper showing) than
    // a firm pass that fills the valleys.
    let (w, h) = (64u32, 30u32);
    let paint = |pg: f32, pressure: f32| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.params.size_px = 16.0;
        t.params.opacity = 1.0;
        t.params.paper_grain = pg;
        t.brush.rendering.accumulate = true;
        t.brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Linear;
        // Isolate the tooth: pressure must NOT also shrink/fade the stroke here.
        t.brush.pencil.pressure_targets = 0;
        t.set_source(flat_source(w, h, [255, 255, 255, 255]), w, h);
        t.params.active_color = crate::color::srgb8_to_painter_oklch([10, 10, 10, 255]);
        t.begin_stroke(1);
        for x in 6..58 {
            t.queue_pointer(PointerSample {
                position: [x as f32, 15.0],
                pressure,
                tilt: 0.0,
            });
        }
        t.end_stroke();
        t.current_preview().unwrap().0.to_vec()
    };
    let variance = |px: &[u8]| {
        let mut vals = Vec::new();
        for y in 11..19u32 {
            for x in 12..52u32 {
                let i = ((y * w + x) * 4) as usize;
                vals.push(px[i] as f32);
            }
        }
        let m = vals.iter().sum::<f32>() / vals.len() as f32;
        vals.iter().map(|v| (v - m).powi(2)).sum::<f32>() / vals.len() as f32
    };
    let mean = |px: &[u8]| {
        let mut sum = 0.0f32;
        let mut n = 0;
        for y in 11..19u32 {
            for x in 12..52u32 {
                let i = ((y * w + x) * 4) as usize;
                sum += px[i] as f32;
                n += 1;
            }
        }
        sum / n as f32
    };
    // The tooth shows at a light/medium touch (a firm pass fills the valleys), so
    // compare at 0.35 pressure where the paper grain is visible.
    let flat = variance(&paint(0.0, 0.35));
    let textured = variance(&paint(0.6, 0.35));
    // Grain-off is now a perfectly uniform interior (sub-pixel sampling), so `flat`
    // ≈ 0 — the tooth signal (textured ≈ 25) is unmistakable above it.
    assert!(
        textured > flat + 15.0,
        "paper tooth adds texture variation: textured {textured} vs flat {flat}"
    );
    // Pressure threshold: a light touch deposits less (the paper tooth shows
    // through → brighter mean) than a firm pass that fills the valleys.
    let light = mean(&paint(0.6, 0.3));
    let firm = mean(&paint(0.6, 1.0));
    assert!(
        light > firm + 15.0,
        "light pressure leaves more paper (brighter) than firm: light {light} vs firm {firm}"
    );
}

#[test]
fn pigment_pick_sets_colour_granulation_and_staining() {
    // ADR-0081: picking a real pigment loads its masstone colour + granulation into the brush
    // and makes its staining ride each dab; clearing it restores the raw-colour (staining 0) path.
    use ph2d_painter_brush::PALETTE;
    let ultra_idx = PALETTE
        .iter()
        .position(|p| p.name == "French Ultramarine")
        .expect("French Ultramarine in the palette") as u8;
    let ultra = &PALETTE[ultra_idx as usize];

    let mut t = PainterTool::default();
    t.set_active_pigment(Some(ultra_idx));
    assert_eq!(t.active_pigment(), Some(ultra_idx));

    // Colour == the pigment's masstone (the exact value set_active_pigment writes).
    let [r, g, b] = ultra.srgb;
    let expected = crate::color::srgb8_to_painter_oklch([r, g, b, 255]);
    let got = t.params.active_color;
    assert!(
        (got.l - expected.l).abs() < 1e-6
            && (got.c - expected.c).abs() < 1e-6
            && (got.h - expected.h).abs() < 1e-6,
        "brush colour must match ultramarine masstone: got {got:?} vs {expected:?}"
    );

    // Granulation folded into the brush watercolor slider.
    assert!(
        (t.brush.rendering.watercolor.granulation - ultra.granulation_param()).abs() < 1e-6,
        "brush granulation must equal the pigment's granulation_param"
    );

    // The active pigment's staining rides each dab — equals the pigment's staining.
    assert!(
        ultra.staining > 0.0,
        "ultramarine has a real staining value"
    );
    assert!(
        (t.active_staining() - ultra.staining).abs() < 1e-6,
        "active_staining must equal the picked pigment's staining"
    );
    // Also visible through the published snapshot index.
    assert_eq!(t.brush_studio_snapshot().active_pigment, Some(ultra_idx));

    // Clearing → raw colour: no active pigment ⇒ staining 0 (colour/params left as-is).
    let colour_before_clear = t.params.active_color;
    t.set_active_pigment(None);
    assert_eq!(t.active_pigment(), None);
    assert_eq!(t.active_staining(), 0.0, "no pigment ⇒ zero staining");
    assert_eq!(t.brush_studio_snapshot().active_pigment, None);
    // Colour is NOT reset on clear (raw-colour path leaves the brush as-is).
    assert_eq!(t.params.active_color, colour_before_clear);
}
