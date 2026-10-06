//! `impl Tool` + `impl RasterEditTool` for `PainterTool` (layers + effects host): `handle_panel_event`
//! routes the panel events; `RasterEditTool` is the shell push / composite-preview / Apply-bake interface.

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
        // BAKE any live Deform Transform float (commit it as one undo entry) — switching tools must persist
        // the transform AND keep it undoable, like the open-shape bake below. No-op outside Transform.
        self.end_transform(true);
        // BAKE any open shape editor first (Apply) — switching to another tool must never ERASE a drawn
        // shape; it's applied into the canvas so the deferred-bake below persists it (Enio 2026-07-03).
        self.commit_open_shape();
        // Abandon every REMAINING in-progress edit (pending Fill, armed Eyedropper, Mask scratch, …) before
        // the canvas is torn down, so nothing rides into the next activation. See `paint::lifecycle`.
        self.reset_transient_edit_state();
        // Persistence (Enio 2026-06-24): with unbaked edits, KEEP the canvas + flag a deferred bake so
        // the shell persists it into the sprite before teardown; otherwise tear down now.
        if self.has_unbaked_edits() {
            self.deferred_bake = true;
        } else {
            <Self as RasterEditTool>::deactivate(self);
        }
    }

    fn handle_panel_event(&mut self, event: ph2d_editor_core::tool::PanelEvent) {
        // The frozen generic channel (ADR-0040 TG-B): the layers panel emits PanelEvent::{Click,
        // SetValue, SelectOption}, each routed to the matching layer / adjustment edit.
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        let appearance_before = self.appearance_sig(); // re-fill an open shape live on any appearance change
        if self.route_texture_layer_event(&event)
            || self.route_brush_jitter_event(&event)
            || self.route_brush_watercolor_event(&event)
            || self.route_brush_wetpaint_event(&event)
            || self.route_brush_impasto_event(&event)
            || self.route_substrate_event(&event)
            || self.route_shape_deposit_event(&event)
            || self.route_brush_stencil_event(&event)
            || self.route_composite_event(&event)
            || self.route_brush_dab_event(&event)
            || self.route_inpaint_event(&event)
            || self.route_fill_event(&event)
            || self.route_selection_event(&event)
            || self.route_deform_event(&event)
            || self.route_sculpt_event(&event)
        {
            self.refill_if_appearance_changed(appearance_before);
            return;
        }
        if let Some(e) = super::layer_edit::decode(&event, |id| self.decode_layer_widget(id)) {
            self.apply_layer_edit(e);
            self.refill_if_appearance_changed(appearance_before);
            return;
        }
        match event {
            // ⭐ Os DOIS segmentos do grupo *Brush | Layers* — cada um ESCOLHE o seu lado, e
            //   tocar no que já está escolhido não faz nada (ver `set_dock_shows_layers`).
            PanelEvent::Click(id) if id == crate::ids::PAINTER_LAYERS_TOGGLE_DOCK => {
                self.set_dock_shows_layers(true);
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_SIDEBAR_TOGGLE_DOCK => {
                self.set_dock_shows_layers(false);
            }
            // Apply CTA — commit the composite to the sprite next frame.
            PanelEvent::Click(id) if id == crate::ids::PAINTER_APPLY => {
                self.request_commit();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_ERASER => {
                self.toggle_brush_eraser();
            }
            // Stroke-section toggles.
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_SPACE_ATTEN => {
                self.toggle_brush_space_attenuation();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_ACCUMULATE => {
                self.toggle_brush_accumulate();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_LINE_SOLID => {
                self.toggle_style_solid();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_LINE_WIRE_CONNECTION => {
                self.toggle_wire_connection_line();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_LINE_SKETCHY_MAGNETIFY => {
                self.toggle_sketchy_magnetify();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_GRID_SHOW => {
                self.toggle_grid_show();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_EDGE_TO_EDGE => {
                self.toggle_brush_edge_to_edge();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAKE => {
                self.toggle_brush_texture_rake();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_TEXTURE_NEW => {
                self.new_brush_texture();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE => {
                self.toggle_texture_ramp_enabled();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD => {
                self.ramp_add_stop();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE => {
                self.ramp_remove_last_stop();
            }
            PanelEvent::Click(id) if id == crate::ids::PAINTER_BRUSH_FALLOFF_ADD => {
                self.add_brush_falloff_point(); // Brush Custom-falloff "+" point button
            }
            // Os cliques por camada são do `layer_edit` (lido antes deste `match`).
            PanelEvent::Click(_) => {}
            // ── Layers per-row sliders (opacity + adjustment params), stored 0..1 → mapped per id. ─
            PanelEvent::SetValue(id, v) => {
                if id == crate::ids::PAINTER_BRUSH_SIZE_SLIDER {
                    self.set_brush_size_norm(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_STRENGTH_SLIDER {
                    self.set_brush_strength(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_SPACING {
                    self.set_brush_spacing(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_OFFSET {
                    self.set_brush_offset(v as f32);
                } else if id == crate::ids::PAINTER_LINE_SKETCHY_REACH {
                    self.set_sketchy_reach_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_SKETCHY_DENSITY {
                    self.set_sketchy_density_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_SKETCHY_WIDTH {
                    self.set_thread_width_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_SKETCHY_OPACITY {
                    self.set_thread_opacity(v as f32);
                } else if id == crate::ids::PAINTER_LINE_WIRE_HISTORY {
                    self.set_wire_history_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_RIBBON_WEIGHT {
                    self.set_ribbon_weight_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_RIBBON_FRICTION {
                    self.set_ribbon_friction_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_RIBBON_GRAVITY {
                    self.set_ribbon_gravity_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_RIBBON_RUNGS {
                    self.set_ribbon_rungs_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_ROUGH_AMOUNT {
                    self.set_rough_amount_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_ROUGH_BOWING {
                    self.set_rough_bowing_norm(v as f32);
                } else if id == crate::ids::PAINTER_LINE_ROUGH_PASSES {
                    self.set_rough_passes_norm(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_JITTER {
                    self.set_brush_jitter_norm(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_DASH_RATIO {
                    self.set_brush_dash_ratio(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_DASH_LENGTH {
                    self.set_brush_dash_length_norm(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_INPUT_SAMPLES {
                    self.set_brush_input_samples_norm(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_STABILIZE {
                    self.set_brush_stabilizer(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_RATE {
                    self.set_brush_airbrush_rate_norm(v as f32);
                } else if let Some(axis) = crate::ids::PAINTER_BRUSH_GRID_CELL
                    .iter()
                    .position(|&p| p == id)
                    .and_then(crate::GridAxis::from_slot)
                {
                    self.set_grid_cell_norm(axis, v as f32);
                } else if let Some(axis) = crate::ids::PAINTER_BRUSH_GRID_OFFSET
                    .iter()
                    .position(|&p| p == id)
                    .and_then(crate::GridAxis::from_slot)
                {
                    self.set_grid_offset_norm(axis, v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_GRID_FIT {
                    self.set_grid_fit_norm(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_TEXTURE_ANGLE {
                    self.set_brush_texture_angle(v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_TEXTURE_OFFSET_X {
                    self.set_brush_texture_offset(0, v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y {
                    self.set_brush_texture_offset(1, v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_TEXTURE_SIZE_X {
                    self.set_brush_texture_size(0, v as f32);
                } else if id == crate::ids::PAINTER_BRUSH_TEXTURE_SIZE_Y {
                    self.set_brush_texture_size(1, v as f32);
                } else if let Some(slot) = crate::ids::PAINTER_BRUSH_TEXTURE_PARAMS
                    .iter()
                    .position(|&p| p == id)
                {
                    self.set_brush_texture_param_norm(slot, v as f32);
                }
            }
            // ── Preset pick (top of panel): value = preset idx (0 = Digital, 1 = Watercolor). ──
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_PRESET => {
                if let Ok(idx) = value.parse::<u8>() {
                    self.apply_brush_preset(idx);
                }
            }
            // ── Paint Mode pick: value = the `PaintMedia` wire u8. The four media are exclusive, and
            //    `set_paint_media` is the only thing that knows it (2026-07-22). ────────────────────
            // ── Card Line: o TIPO de linha procedural (plano 38 W2). ──────────────────────────
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_LINE_TYPE => {
                if let Ok(v) = value.parse::<u8>() {
                    self.set_line_kind(v);
                }
            }
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_MEDIA => {
                if let Ok(v) = value.parse::<u8>() {
                    self.set_paint_media(crate::PaintMedia::from_u8(v));
                }
            }
            // ── Brush section blend pick: value = `BrushBlend` wire u8. ──────
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_BLEND => {
                if let Ok(mode) = value.parse::<u8>() {
                    self.set_brush_blend(mode);
                }
            }
            // ── Falloff section preset pick: value = `Falloff` wire u8. ──────
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_FALLOFF => {
                if let Ok(preset) = value.parse::<u8>() {
                    self.set_brush_falloff(preset);
                }
            }
            // Stroke Method: the wire u8 (dropdown / rail shape pick) or the "brush" sentinel (rail Brush
            // button → restore the last non-shape method). See `paint::stroke_ctl`.
            PanelEvent::SelectOption(id, value) if id == core_ids::PAINTER_BRUSH_STROKE_METHOD => {
                self.apply_stroke_method_command(&value);
            }
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_JITTER_UNIT => {
                if let Ok(u) = value.parse::<u8>() {
                    self.set_brush_jitter_unit(u);
                }
            }
            // ── Texture section dropdowns: kind picker + mapping (value = wire u8). ─
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_TEXTURE_KIND => {
                if let Ok(k) = value.parse::<u8>() {
                    self.set_brush_texture_kind(k);
                }
            }
            // ── Watercolor Paper slot kind + mapping pickers (value = wire u8). ─
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_WATERCOLOR_PAPER_KIND =>
            {
                if let Ok(k) = value.parse::<u8>() {
                    self.set_brush_paper_kind(k);
                }
            }
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_WATERCOLOR_FLOW_KIND =>
            {
                if let Ok(k) = value.parse::<u8>() {
                    self.set_brush_edge_flow_kind(k);
                }
            }
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_TEXTURE_MAPPING =>
            {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_brush_texture_mapping(m);
                }
            }
            // ── Color Ramp dropdowns: Mode + Interpolation (value = wire u8). ─
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE =>
            {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_texture_ramp_mode(m);
                }
            }
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP =>
            {
                if let Ok(i) = value.parse::<u8>() {
                    self.set_texture_ramp_interp(i);
                }
            }
            // Ramp alpha action: Off / → Strength / → Sprite (value = `RampAlphaMode` wire u8).
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE =>
            {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_texture_ramp_alpha_mode(m);
                }
            }
            // Ramp stop colour from the picker: value = "stop,r,g,b,a" (sRGB bytes, straight alpha).
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH =>
            {
                let mut it = value.split(',').filter_map(|p| p.parse::<i32>().ok());
                if let (Some(id), Some(r), Some(g), Some(b), Some(a)) =
                    (it.next(), it.next(), it.next(), it.next(), it.next())
                {
                    self.ramp_set_stop_color(id as u8, [r as u8, g as u8, b as u8, a as u8]);
                }
            }
            // Ramp stop drag on the bar: value = "id:x" (stable id, x = normalized position `0..1`).
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT =>
            {
                let mut it = value.split(':');
                if let (Some(Ok(sid)), Some(Ok(x))) = (
                    it.next().map(str::parse::<u8>),
                    it.next().map(str::parse::<f32>),
                ) {
                    self.ramp_move_stop(sid, x);
                }
            }
            // Custom-falloff curve point 2-D drag: value = "id:x:y" (stable id keeps the grab across re-sort).
            PanelEvent::SelectOption(id, value) if id == crate::ids::PAINTER_BRUSH_FALLOFF_EDIT => {
                let mut it = value.split(':');
                if let (Some(i), Some(xs), Some(ys)) = (it.next(), it.next(), it.next())
                    && let (Ok(pid), Ok(x), Ok(y)) =
                        (i.parse::<u8>(), xs.parse::<f32>(), ys.parse::<f32>())
                {
                    self.set_brush_falloff_point(pid, x, y);
                }
            }
            // ── Custom-falloff "−" point button: value = stable point id. ───
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_BRUSH_FALLOFF_REMOVE =>
            {
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
                    // Uma chamada, não três: a porta única re-carimba a forma aberta ao mudar a cor,
                    // e três chamadas de canal seriam três re-stamps para uma escolha só.
                    self.set_brush_color_srgb8([r, g, b]);
                }
            }
            // ── Watercolor PAPER colour from the shared picker: value = "r,g,b" (8-bit native),
            // forwarded by the panel's per-frame read-back — the document ground the optics see. ──
            PanelEvent::SelectOption(id, value)
                if id == crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB =>
            {
                let mut it = value.split(',');
                if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next())
                    && let (Ok(r), Ok(g), Ok(b)) =
                        (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
                {
                    self.set_paper_color_rgb8(r, g, b);
                    self.papel_segue_a_cor(); // com o papel aplicado ele muda AO VIVO
                }
            }
            // Os pedidos por camada (ajustes, mistura) são do `layer_edit`.
            PanelEvent::SelectOption(..) => {}
            PanelEvent::Toggle(_, _) => {}
        }
        self.refill_if_appearance_changed(appearance_before);
    }

    /// Per-frame heartbeat (ADR-0040-amendment-2): airbrush timer + stabilizer catch-up; `dt_ms` = real wall ms since last frame.
    fn on_tick(&mut self, dt_ms: f32) {
        self.paint_tick(dt_ms * 1e-3);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        Some(self)
    }

    /// The painter consumes canvas pointer samples to paint dabs (ADR-0040 Amendment 3; [`crate::tool::paint`]).
    fn as_canvas_paint_mut(&mut self) -> Option<&mut dyn CanvasPaintTool> {
        Some(self)
    }

    fn is_default(&self) -> bool {
        false
    }
}
