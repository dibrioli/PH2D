//! `impl Tool` + `impl RasterEditTool` for `PainterTool` (the layers + effects
//! host). `handle_panel_event` routes the layers-panel events; `RasterEditTool`
//! is the shell's source-push / composite-preview / Apply-bake interface.

use super::*;

impl Tool for PainterTool {
    fn id(&self) -> ToolId {
        ToolId::new("painter")
    }

    fn label(&self) -> &str {
        "Painter"
    }

    fn icon_slug(&self) -> &str {
        "painter"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The layers panel (`ph2d-panel-painter-layers`) is a docked panel, not a
        // floating tool panel — this stays an empty titled stub.
        FloatingPanel::new(self.id(), "Painter")
    }

    fn on_activate(&mut self) {
        // Install the takeover UI (suppresses the normal PH2D chrome; ADR-0043 §1.1).
        self.params.takeover_active = true;
    }

    fn on_deactivate(&mut self) {
        self.params.takeover_active = false;
        // Drop any in-progress shape session (Curve/Circle) before the canvas is torn down (no
        // restore — the working buffer is cleared next).
        self.discard_open_shape();
        // Full teardown: clears canvas_rgba + source_size + commit flag + dock mode.
        <Self as RasterEditTool>::deactivate(self);
    }

    fn handle_panel_event(&mut self, event: ph2d_editor_core::tool::PanelEvent) {
        // The frozen generic channel (ADR-0040 TG-B): the layers panel emits
        // PanelEvent::{Click, SetValue, SelectOption}. Each is routed to the
        // matching layer / adjustment edit (the single source of truth in
        // `tool/layers.rs`).
        use ph2d_editor_core::ids::{self as core_ids, PainterLayerWidget};
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            // ── Dock toggle — layers panel header button ───────────────────
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_TOGGLE_DOCK => {
                self.toggle_dock();
            }
            // ── Layers panel: "+ Layer" (create + activate a raster on top) ─
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_ADD => {
                let name = format!("Layer {}", self.layers.len() + 1);
                self.add_raster_layer(name);
            }
            // ── Header actions: duplicate / delete / group the active layer ─
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_DUPLICATE => {
                if let Some(active) = self.layers.active() {
                    self.duplicate_layer(active);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_DELETE => {
                if let Some(active) = self.layers.active() {
                    self.delete_layer(active);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_GROUP => {
                self.group_selected();
            }
            // ── Modifier toolbar (acts on the ACTIVE layer) ────────────────
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_MASK => {
                self.add_mask_to_active();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_CLIP => {
                if let Some(a) = self.layers.active() {
                    let now = self.layers.get(a).is_some_and(|l| l.clipping);
                    self.set_layer_clipping(a, !now);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_ALPHA_LOCK => {
                if let Some(a) = self.layers.active() {
                    let now = self.layers.get(a).is_some_and(|l| l.alpha_locked);
                    self.set_layer_alpha_locked(a, !now);
                }
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_LAYERS_REFERENCE => {
                if let Some(a) = self.layers.active() {
                    let now = self.layers.get(a).is_some_and(|l| l.is_reference);
                    self.set_layer_reference(a, !now);
                }
            }
            // ── Apply CTA — commit the composite to the sprite (next frame). ─
            PanelEvent::Click(id) if id == core_ids::PAINTER_APPLY => {
                self.request_commit();
            }
            // ── Brush Eraser toggle (Brush-properties view). ───────────────
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_ERASER => {
                self.toggle_brush_eraser();
            }
            // ── Stroke section toggles. ────────────────────────────────────
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_SPACE_ATTEN => {
                self.toggle_brush_space_attenuation();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_EDGE_TO_EDGE => {
                self.toggle_brush_edge_to_edge();
            }
            // ── Texture section toggles + "New" (assign the default procedural). ─
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_RAKE => {
                self.toggle_brush_texture_rake();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_RANDOM => {
                self.toggle_brush_texture_random();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_NEW => {
                self.new_brush_texture();
            }
            // ── Texture Color Ramp: enable toggle + add / remove stop. ─────
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE => {
                self.toggle_texture_ramp_enabled();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD => {
                self.ramp_add_stop();
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE => {
                self.ramp_remove_last_stop();
            }
            // ── Brush Custom-falloff "+" point button. ─────────────────────
            PanelEvent::Click(id) if id == core_ids::PAINTER_BRUSH_FALLOFF_ADD => {
                self.add_brush_falloff_point();
            }
            // ── Layers panel: per-row click (row select / visibility eye) ──
            PanelEvent::Click(id) => {
                if let Some((layer, kind)) = self.decode_layer_widget(id) {
                    match kind {
                        // Multi-select: the panel stashed the Cmd/Shift state for
                        // this row click. Shift = range, Cmd/Ctrl = toggle additive,
                        // plain = single.
                        PainterLayerWidget::Row => {
                            let (cmd, shift) = take_pending_select_mods(id);
                            if shift {
                                self.select_range(layer);
                            } else if cmd {
                                self.select_additive(layer);
                            } else {
                                self.select_single(layer);
                            }
                        }
                        PainterLayerWidget::Visibility => {
                            let now = self.layers.get(layer).map(|l| l.visible).unwrap_or(true);
                            self.set_layer_visible(layer, !now);
                        }
                        PainterLayerWidget::MoveUp => self.move_layer_up(layer),
                        PainterLayerWidget::MoveDown => self.move_layer_down(layer),
                        PainterLayerWidget::MaskInvert => self.toggle_mask_inverted(layer),
                        PainterLayerWidget::MaskApply => {
                            self.apply_mask(layer);
                        }
                        PainterLayerWidget::AdjToggle0 => self.flip_adjustment_toggle(layer, 0),
                        PainterLayerWidget::AdjToggle1 => self.flip_adjustment_toggle(layer, 1),
                        PainterLayerWidget::AdjSegment0 => self.set_adjustment_segment(layer, 0),
                        PainterLayerWidget::AdjSegment1 => self.set_adjustment_segment(layer, 1),
                        PainterLayerWidget::AdjSegment2 => self.set_adjustment_segment(layer, 2),
                        _ => {}
                    }
                }
            }
            // ── Layers panel: per-row sliders (opacity + adjustment params),
            // all stored 0..1; the tool maps each to its target. ───────────
            PanelEvent::SetValue(id, v) => {
                // Brush-properties sliders (fixed ids, tool-global): Size + Strength.
                if id == core_ids::PAINTER_BRUSH_SIZE_SLIDER {
                    self.set_brush_size_norm(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_STRENGTH_SLIDER {
                    self.set_brush_strength(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_SPACING {
                    self.set_brush_spacing(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_JITTER {
                    self.set_brush_jitter_norm(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_DASH_RATIO {
                    self.set_brush_dash_ratio(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_DASH_LENGTH {
                    self.set_brush_dash_length_norm(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_INPUT_SAMPLES {
                    self.set_brush_input_samples_norm(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_STABILIZE {
                    self.set_brush_stabilizer(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_RATE {
                    self.set_brush_airbrush_rate_norm(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_TEXTURE_ANGLE {
                    self.set_brush_texture_angle_norm(v as f32);
                } else if id == core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X {
                    self.set_brush_texture_offset_norm(0, v as f32);
                } else if id == core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y {
                    self.set_brush_texture_offset_norm(1, v as f32);
                } else if id == core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X {
                    self.set_brush_texture_size_norm(0, v as f32);
                } else if id == core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y {
                    self.set_brush_texture_size_norm(1, v as f32);
                } else if let Some(slot) = core_ids::PAINTER_BRUSH_TEXTURE_PARAMS
                    .iter()
                    .position(|&p| p == id)
                {
                    self.set_brush_texture_param_norm(slot, v as f32);
                } else if let Some((layer, kind)) = self.decode_layer_widget(id) {
                    match kind {
                        PainterLayerWidget::Opacity => self.set_layer_opacity(layer, v as f32),
                        PainterLayerWidget::AdjParam0 => {
                            self.set_adjustment_param(layer, 0, v as f32)
                        }
                        PainterLayerWidget::AdjParam1 => {
                            self.set_adjustment_param(layer, 1, v as f32)
                        }
                        PainterLayerWidget::AdjParam2 => {
                            self.set_adjustment_param(layer, 2, v as f32)
                        }
                        PainterLayerWidget::AdjParam3 => {
                            self.set_adjustment_param(layer, 3, v as f32)
                        }
                        PainterLayerWidget::AdjParam4 => {
                            self.set_adjustment_param(layer, 4, v as f32)
                        }
                        PainterLayerWidget::AdjParam5 => {
                            self.set_adjustment_param(layer, 5, v as f32)
                        }
                        PainterLayerWidget::AdjParam6 => {
                            self.set_adjustment_param(layer, 6, v as f32)
                        }
                        PainterLayerWidget::AdjParam7 => {
                            self.set_adjustment_param(layer, 7, v as f32)
                        }
                        _ => {}
                    }
                }
            }
            // ── "+ Adjustment" kind pick: value = index into `AdjustmentKind::ALL`. ─
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_LAYERS_ADD_ADJUSTMENT =>
            {
                if let Ok(idx) = value.parse::<usize>()
                    && let Some(&kind) =
                        ph2d_painter_effects::adjustments::AdjustmentKind::ALL.get(idx)
                {
                    self.add_adjustment_layer(kind);
                }
            }
            // ── Brush section blend pick: value = `BrushBlend` wire u8. ──────
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_BLEND => {
                if let Ok(mode) = value.parse::<u8>() {
                    self.set_brush_blend(mode);
                }
            }
            // ── Falloff section preset pick: value = `Falloff` wire u8. ──────
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_FALLOFF => {
                if let Ok(preset) = value.parse::<u8>() {
                    self.set_brush_falloff(preset);
                }
            }
            // ── Stroke section dropdowns: method + jitter unit (value = wire u8). ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_STROKE_METHOD => {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_brush_stroke_method(m);
                }
            }
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_JITTER_UNIT => {
                if let Ok(u) = value.parse::<u8>() {
                    self.set_brush_jitter_unit(u);
                }
            }
            // ── Texture section dropdowns: kind picker + mapping (value = wire u8). ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_TEXTURE_KIND => {
                if let Ok(k) = value.parse::<u8>() {
                    self.set_brush_texture_kind(k);
                }
            }
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_BRUSH_TEXTURE_MAPPING =>
            {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_brush_texture_mapping(m);
                }
            }
            // ── Color Ramp dropdowns: Mode + Interpolation (value = wire u8). ─
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE =>
            {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_texture_ramp_mode(m);
                }
            }
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP =>
            {
                if let Ok(i) = value.parse::<u8>() {
                    self.set_texture_ramp_interp(i);
                }
            }
            // Ramp stop colour from the picker: value = "stop,r,g,b" (sRGB bytes).
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH =>
            {
                let mut it = value.split(',').filter_map(|p| p.parse::<i32>().ok());
                if let (Some(id), Some(r), Some(g), Some(b)) =
                    (it.next(), it.next(), it.next(), it.next())
                {
                    self.ramp_set_stop_color(id as u8, [r as u8, g as u8, b as u8]);
                }
            }
            // Ramp stop drag on the bar: value = "id:x" (stable id, x = normalized position `0..1`).
            PanelEvent::SelectOption(id, value)
                if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT =>
            {
                let mut it = value.split(':');
                if let (Some(Ok(sid)), Some(Ok(x))) = (
                    it.next().map(str::parse::<u8>),
                    it.next().map(str::parse::<f32>),
                ) {
                    self.ramp_move_stop(sid, x);
                }
            }
            // ── Custom-falloff curve point 2-D drag: value = "id:x:y". The
            // stable id (not a sorted index) keeps the handle grabbed across the
            // re-sort when a point is dragged past another. ─────────────────
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_FALLOFF_EDIT => {
                let mut it = value.split(':');
                if let (Some(i), Some(xs), Some(ys)) = (it.next(), it.next(), it.next())
                    && let (Ok(pid), Ok(x), Ok(y)) =
                        (i.parse::<u8>(), xs.parse::<f32>(), ys.parse::<f32>())
                {
                    self.set_brush_falloff_point(pid, x, y);
                }
            }
            // ── Custom-falloff "−" point button: value = stable point id. ───
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_FALLOFF_REMOVE => {
                if let Ok(pid) = value.parse::<u8>() {
                    self.remove_brush_falloff_point(pid);
                }
            }
            // ── Brush colour from the shared Blender picker: value = "r,g,b"
            // (8-bit native), forwarded by the panel's per-frame read-back. ──
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_COLOR_THUMB => {
                let mut it = value.split(',');
                if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next())
                    && let (Ok(r), Ok(g), Ok(b)) =
                        (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
                {
                    self.set_brush_color_channel(0, f32::from(r) / 255.0);
                    self.set_brush_color_channel(1, f32::from(g) / 255.0);
                    self.set_brush_color_channel(2, f32::from(b) / 255.0);
                }
            }
            // ── Curves editor 2-D point drag: value = "layer:channel:index:x:y". ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_CURVE_EDIT => {
                let mut it = value.split(':');
                if let (Some(l), Some(c), Some(i), Some(xs), Some(ys)) =
                    (it.next(), it.next(), it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(ch), Ok(idx), Ok(x), Ok(y)) = (
                        l.parse::<u64>(),
                        c.parse::<u8>(),
                        i.parse::<usize>(),
                        xs.parse::<f32>(),
                        ys.parse::<f32>(),
                    )
                {
                    self.set_curve_point(RtLayerId(layer), ch, idx, x, y);
                }
            }
            // ── Channel Mixer weight edit: value = "layer:output:slot:value". ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_MIXER_EDIT => {
                let mut it = value.split(':');
                if let (Some(l), Some(o), Some(s), Some(v)) =
                    (it.next(), it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(output), Ok(slot), Ok(val)) = (
                        l.parse::<u64>(),
                        o.parse::<usize>(),
                        s.parse::<usize>(),
                        v.parse::<f32>(),
                    )
                {
                    self.set_channel_mixer_weight(RtLayerId(layer), output, slot, val);
                }
            }
            // ── Gradient Map editor: stop drag / add / remove / selected color. ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_GRADIENT_EDIT => {
                let mut it = value.split(':');
                if let (Some(l), Some(i), Some(o)) = (it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(idx), Ok(off)) =
                        (l.parse::<u64>(), i.parse::<usize>(), o.parse::<f32>())
                {
                    self.set_gradient_stop_offset(RtLayerId(layer), idx, off);
                }
            }
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_GRADIENT_ADD => {
                if let Ok(layer) = value.parse::<u64>() {
                    self.add_gradient_stop(RtLayerId(layer));
                }
            }
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_GRADIENT_REMOVE => {
                let mut it = value.split(':');
                if let (Some(l), Some(i)) = (it.next(), it.next())
                    && let (Ok(layer), Ok(idx)) = (l.parse::<u64>(), i.parse::<usize>())
                {
                    self.remove_gradient_stop(RtLayerId(layer), idx);
                }
            }
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_GRADIENT_COLOR => {
                let mut it = value.split(':');
                if let (Some(l), Some(st), Some(s), Some(v)) =
                    (it.next(), it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(stop), Ok(slot), Ok(val)) = (
                        l.parse::<u64>(),
                        st.parse::<usize>(),
                        s.parse::<usize>(),
                        v.parse::<f32>(),
                    )
                {
                    self.set_gradient_stop_color(RtLayerId(layer), stop, slot, val);
                }
            }
            // ── Selective Color CMYK edit: value = "layer:bucket:slot:value". ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_SELCOLOR_EDIT => {
                let mut it = value.split(':');
                if let (Some(l), Some(bk), Some(s), Some(v)) =
                    (it.next(), it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(bucket), Ok(slot), Ok(val)) = (
                        l.parse::<u64>(),
                        bk.parse::<usize>(),
                        s.parse::<usize>(),
                        v.parse::<f32>(),
                    )
                {
                    self.set_selective_color_value(RtLayerId(layer), bucket, slot, val);
                }
            }
            // ── Curves editor add a point: value = "layer:channel". ──────────
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_CURVE_ADD => {
                let mut it = value.split(':');
                if let (Some(l), Some(c)) = (it.next(), it.next())
                    && let (Ok(layer), Ok(ch)) = (l.parse::<u64>(), c.parse::<u8>())
                {
                    self.add_curve_point(RtLayerId(layer), ch);
                }
            }
            // ── Curves editor remove a point: value = "layer:channel:index". ─
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_CURVE_REMOVE => {
                let mut it = value.split(':');
                if let (Some(l), Some(c), Some(i)) = (it.next(), it.next(), it.next())
                    && let (Ok(layer), Ok(ch), Ok(idx)) =
                        (l.parse::<u64>(), c.parse::<u8>(), i.parse::<usize>())
                {
                    self.remove_curve_point(RtLayerId(layer), ch, idx);
                }
            }
            // ── Layers panel: per-row blend-mode pick (value = wire u8) ────
            PanelEvent::SelectOption(id, value) => {
                if let Some((layer, PainterLayerWidget::Blend)) = self.decode_layer_widget(id)
                    && let Ok(mode) = value.parse::<u8>()
                {
                    self.set_layer_blend_mode(layer, BlendMode::from_u8(mode));
                }
            }
            PanelEvent::Toggle(_, _) => {}
        }
    }

    /// Per-frame heartbeat (ADR-0040-amendment-2): drives the airbrush timer (deposit dabs at the
    /// brush Rate while held) and the stabilizer catch-up (converge the lagged path to a parked
    /// cursor). `dt_ms` is the real wall time since the last frame (shell `frame_ms_now`).
    fn on_tick(&mut self, dt_ms: f32) {
        self.paint_tick(dt_ms * 1e-3);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        Some(self)
    }

    /// The painter consumes canvas pointer samples to paint dabs into the active
    /// raster layer (ADR-0040 Amendment 3). See [`crate::tool::paint`].
    fn as_canvas_paint_mut(&mut self) -> Option<&mut dyn CanvasPaintTool> {
        Some(self)
    }

    fn is_default(&self) -> bool {
        false
    }
}

impl RasterEditTool for PainterTool {
    /// Initialize the working canvas from the sprite source. RGBA8 straight; it
    /// is the base of the single-raster layer stack the layers panel grows from.
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        assert_eq!(
            rgba.len(),
            (width as usize) * (height as usize) * 4,
            "set_source rgba length must equal width*height*4"
        );
        // A new working canvas invalidates any open shape session (its restore record points at the
        // OLD buffer). Drop it without restoring.
        self.discard_open_shape();
        self.canvas_rgba = Arc::new(rgba);
        self.source_size = (width, height);
        self.preview_dirty = true;
        // A fresh source = a fresh single-raster stack. The lone layer's pixels
        // ARE `canvas_rgba` (the active working buffer), so `images` stays empty
        // and the stack is trivial → the fast preview path. Multi-layer state
        // only appears once the layers panel adds layers.
        self.layers = LayerStack::new();
        self.layers.add_raster("Layer 1", width, height);
        self.images.clear();
        self.layer_pixel_versions.clear();
        let active = self.layers.active();
        self.bump_layer_pixels(active);
        self.composited = None;
        self.layers_revision = self.layers_revision.wrapping_add(1);
        // A fresh source is a different working canvas — undo/redo over the OLD
        // model is meaningless on the NEW one.
        self.undo.clear();
    }

    /// Borrow the current composite iff it changed since the last call (drains
    /// `preview_dirty`). Trivial stack → `canvas_rgba` byte-for-byte (fast path);
    /// non-trivial → the composited layer stack (CPU reference).
    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        let (w, h) = self.source_size;
        if self.is_trivial_stack() {
            return Some((&self.canvas_rgba, w, h));
        }
        let composited = {
            let active = self.layers.active().unwrap_or(RtLayerId(0));
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            composite(&self.layers, &src, w, h)
        };
        self.composited = Some(Arc::new(composited));
        Some((
            self.composited
                .as_ref()
                .map(|a| a.as_slice())
                .unwrap_or(&[]),
            w,
            h,
        ))
    }

    fn take_pending_commit(&mut self) -> bool {
        std::mem::take(&mut self.pending_commit)
    }

    /// Bake the final canvas for commit (Apply): the full layer COMPOSITE (base
    /// layer + every layer above with their opacity / blend / visibility /
    /// adjustments) — exactly what the live preview shows.
    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        let (w, h) = self.source_size;
        if !self.is_trivial_stack() {
            let active = self.layers.active().unwrap_or(RtLayerId(0));
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            return (composite(&self.layers, &src, w, h), w, h);
        }
        // Trivial single-layer fast path: take the Arc out so refcount==1 →
        // `unwrap_or_clone` returns the inner Vec without allocation.
        let canvas = std::mem::replace(&mut self.canvas_rgba, Arc::new(Vec::new()));
        (Arc::unwrap_or_clone(canvas), w, h)
    }

    fn deactivate(&mut self) {
        self.canvas_rgba = Arc::new(Vec::new());
        self.source_size = (0, 0);
        self.preview_dirty = false;
        self.pending_commit = false;
        // Reset the dock-mode flag so re-activating starts on the Layers view.
        self.dock_shows_layers = true;
    }
}
