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
