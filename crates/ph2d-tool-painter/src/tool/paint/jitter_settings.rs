//! Per-dab **randomize** setters (Jitter Scale / Jitter Rotate / Randomize Color), the single clamp
//! source for those UI edits. Split from `brush_settings.rs` for the workspace LOC cap; a submodule
//! of `paint` so it shares `PainterTool`'s private `paint.brush` access. The values feed
//! `ph2d_painter_brush::jitter::per_dab` at stamp time.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::{BrushSpec, MirrorAxis, TextureSettings};

impl PainterTool {
    /// Route the per-dab randomize controls (Randomize Color enable + Hue/Sat/Value, Jitter Scale,
    /// Jitter Rotate) AND the seamless Tiling toggles from the layers panel's generic channel to the
    /// setters above. Returns `true` when it consumed the event. Mirrors `route_texture_layer_event`;
    /// called from `handle_panel_event` before the main match. These ids are `PAINTER_BRUSH_COLOR_*` /
    /// `PAINTER_BRUSH_JITTER_*` / `PAINTER_BRUSH_TILING_*`, which the texture-layer router does not claim.
    pub(crate) fn route_brush_jitter_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE => {
                self.toggle_brush_color_jitter_enabled();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TILING_X => {
                self.toggle_brush_tiling(0);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TILING_Y => {
                self.toggle_brush_tiling(1);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_REPEAT_IMAGE => {
                self.toggle_repeat_image();
                true
            }
            // Per-section reset icon buttons (Inspector-Transform pattern) → restore that section's
            // brush fields to defaults.
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_RANDOMIZE_RESET => {
                self.reset_brush_randomize();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TEXTURE_RESET => {
                self.reset_brush_texture();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RESET => {
                self.reset_brush_shape();
                true
            }
            // ── Shape section toggle (silhouette rotation follows the stroke). A random per-dab spin is
            //    the Stroke Jitter Rotate (whole stamp), not a per-slot toggle (Enio 2026-07-19). ─
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAKE => {
                self.toggle_brush_shape_rake();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_COLOR_RAMP_RESET => {
                self.reset_brush_color_ramp();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_STROKE_RESET => {
                self.reset_brush_stroke();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TILING_RESET => {
                self.reset_brush_tiling();
                true
            }
            // ── Symmetry section: master / circular toggles, the X/Y/Custom axis segments, the two
            //    canvas pick-mode buttons, and the section reset. Plain paint state (no undo). ─
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_USE => {
                self.toggle_symmetry_enabled();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_CIRCULAR => {
                self.toggle_symmetry_circular();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_X => {
                self.set_symmetry_axis(MirrorAxis::X);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_Y => {
                self.set_symmetry_axis(MirrorAxis::Y);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM => {
                self.set_symmetry_axis(MirrorAxis::Custom);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_DRAW_LINE => {
                self.begin_symmetry_pick_line();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_PICK_CENTER => {
                self.begin_symmetry_pick_center();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_SYMMETRY_RESET => {
                self.reset_symmetry();
                true
            }
            PanelEvent::SetValue(id, v) => {
                let v = *v as f32;
                match *id {
                    x if x == core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE => {
                        self.set_brush_color_jitter(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_COLOR_JITTER_SAT => {
                        self.set_brush_color_jitter(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL => {
                        self.set_brush_color_jitter(2, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_JITTER_SCALE => {
                        self.set_brush_jitter_scale(v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_JITTER_ROTATE => {
                        self.set_brush_jitter_rotate(v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_JITTER_SPACING => {
                        self.set_brush_jitter_spacing(v);
                        true
                    }
                    // Grain Depth + Shape geometry sliders (the Shape/Grain sections).
                    x if x == core_ids::PAINTER_BRUSH_GRAIN_DEPTH => {
                        self.set_brush_grain_depth(v);
                        true
                    }
                    x if x == core_ids::PAINTER_SHAPE_ANGLE => {
                        self.set_brush_shape_angle(v);
                        true
                    }
                    x if x == core_ids::PAINTER_SHAPE_OFFSET_X => {
                        self.set_brush_shape_offset(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_SHAPE_OFFSET_Y => {
                        self.set_brush_shape_offset(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_SHAPE_SIZE_X => {
                        self.set_brush_shape_size(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_SHAPE_SIZE_Y => {
                        self.set_brush_shape_size(1, v);
                        true
                    }
                    x if core_ids::PAINTER_SHAPE_PARAMS.contains(&x) => {
                        if let Some(slot) =
                            core_ids::PAINTER_SHAPE_PARAMS.iter().position(|&p| p == x)
                        {
                            self.set_brush_shape_param_norm(slot, v);
                        }
                        true
                    }
                    // Symmetry segment count: the `0..1` track maps onto the integer span 3..12.
                    x if x == core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS => {
                        self.set_symmetry_segments((3.0 + v * 9.0).round() as u32);
                        true
                    }
                    // Per-layer-colour opacity box (`0..100`) → the source document layer's opacity.
                    x => {
                        let mut hit = false;
                        for i in 0..super::shape_layers::MAX_SHAPE_LAYERS as u8 {
                            if x == core_ids::painter_shape_layer_opacity_id(i) {
                                self.set_brush_shape_layer_opacity(i as usize, v / 100.0);
                                hit = true;
                                break;
                            }
                        }
                        hit
                    }
                }
            }
            // ── Shape **source** picker (None / Image) — sibling of the Shape Click/SetValue routes
            //    above; lives here (not the main `handle_panel_event` match) to keep that file under the
            //    workspace LOC cap. `Image` requests a file pick; anything else clears the image. ─
            PanelEvent::SelectOption(id, value) if *id == core_ids::PAINTER_SHAPE_KIND => {
                if let Ok(k) = value.parse::<u8>() {
                    self.set_brush_shape_kind(k);
                }
                true
            }
            // ── Grain-ramp invert/B&W + the Shape Colour Ramp events — split into a sibling helper to
            //    keep both this function and `trait_impls.rs` under the workspace / per-fn LOC cap. ─
            _ => self.route_brush_ramp_event(event),
        }
    }

    /// The Grain-ramp invert / B&W toggle + the Shape **Colour Ramp** events (Click + SelectOption) —
    /// split from [`Self::route_brush_jitter_event`] for the per-fn LOC cap. Returns `true` when
    /// consumed. The Shape ramp is the colour twin of the Grain ramp (mode/interp/alpha/swatch/move).
    fn route_brush_ramp_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INVERT => {
                self.ramp_invert();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_BW => {
                self.toggle_texture_ramp_bw();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAMP_ENABLE => {
                self.toggle_shape_ramp_enabled();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAMP_ADD => {
                self.shape_ramp_add_stop();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAMP_REMOVE => {
                self.shape_ramp_remove_last_stop();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAMP_INVERT => {
                self.shape_ramp_invert();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAMP_BW => {
                self.toggle_shape_ramp_bw();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_RAMP_RESET => {
                self.reset_shape_ramp();
                true
            }
            PanelEvent::SelectOption(id, value) if *id == core_ids::PAINTER_SHAPE_RAMP_MODE => {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_shape_ramp_mode(m);
                }
                true
            }
            PanelEvent::SelectOption(id, value) if *id == core_ids::PAINTER_SHAPE_RAMP_INTERP => {
                if let Ok(i) = value.parse::<u8>() {
                    self.set_shape_ramp_interp(i);
                }
                true
            }
            PanelEvent::SelectOption(id, value)
                if *id == core_ids::PAINTER_SHAPE_RAMP_ALPHA_MODE =>
            {
                if let Ok(m) = value.parse::<u8>() {
                    self.set_shape_ramp_alpha_mode(m);
                }
                true
            }
            PanelEvent::SelectOption(id, value) if *id == core_ids::PAINTER_SHAPE_RAMP_SWATCH => {
                let mut it = value.split(',').filter_map(|p| p.parse::<i32>().ok());
                if let (Some(sid), Some(r), Some(g), Some(b), Some(a)) =
                    (it.next(), it.next(), it.next(), it.next(), it.next())
                {
                    self.shape_ramp_set_stop_color(sid as u8, [r as u8, g as u8, b as u8, a as u8]);
                }
                true
            }
            PanelEvent::SelectOption(id, value) if *id == core_ids::PAINTER_SHAPE_RAMP_EDIT => {
                if let Some((sid, x)) = parse_id_f32(value) {
                    self.shape_ramp_move_stop(sid, x);
                }
                true
            }
            _ => self.route_shape_layer_event(event),
        }
    }

    /// Per-Layer Color (multi-layer Shape): the mode toggle, a layer's colour checkbox, or a layer's
    /// colour-swatch pick (`"i,r,g,b"`, u8). Factory ids are matched by a bounded scan over the cap.
    fn route_shape_layer_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_USE_LAYERS => {
                self.capture_layers_as_brush_shape();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_PER_LAYER_COLOR => {
                self.toggle_brush_shape_per_layer_color();
                true
            }
            PanelEvent::Click(id) => {
                for i in 0..super::shape_layers::MAX_SHAPE_LAYERS as u8 {
                    if *id == core_ids::painter_shape_layer_color_check_id(i) {
                        self.toggle_brush_shape_layer_color(i as usize);
                        return true;
                    }
                }
                false
            }
            PanelEvent::SelectOption(id, value) => {
                for i in 0..super::shape_layers::MAX_SHAPE_LAYERS as u8 {
                    if *id == core_ids::painter_shape_layer_color_swatch_id(i) {
                        let mut it = value.split(',').filter_map(|p| p.parse::<i32>().ok());
                        // `"i,r,g,b"` (the leading `i` is redundant with the matched id).
                        if let (Some(_li), Some(r), Some(g), Some(b)) =
                            (it.next(), it.next(), it.next(), it.next())
                        {
                            let n = |c: i32| (c as f32 / 255.0).clamp(0.0, 1.0);
                            self.set_brush_shape_layer_color(i as usize, [n(r), n(g), n(b)]);
                        }
                        return true;
                    }
                    if *id == core_ids::painter_shape_layer_blend_id(i) {
                        if let Ok(mode) = value.parse::<u8>() {
                            self.set_brush_shape_layer_blend(i as usize, mode);
                        }
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Toggle the **Randomize Color** master enable (per-dab HSV scatter of the brush colour).
    pub fn toggle_brush_color_jitter_enabled(&mut self) {
        self.paint.brush.color_jitter_enabled = !self.paint.brush.color_jitter_enabled;
    }

    /// Set the Randomize-Color amount for `slot` (`0` = Hue, `1` = Saturation, `2` = Value) from the
    /// slider's `0..1` track; out-of-range slots are ignored.
    pub fn set_brush_color_jitter(&mut self, slot: usize, t: f32) {
        let t = t.clamp(0.0, 1.0);
        match slot {
            0 => self.paint.brush.color_jitter_hue = t,
            1 => self.paint.brush.color_jitter_sat = t,
            2 => self.paint.brush.color_jitter_val = t,
            _ => {}
        }
    }

    /// Set **Jitter Scale** (per-dab radius scatter) from the slider's `0..1` track.
    pub fn set_brush_jitter_scale(&mut self, t: f32) {
        self.paint.brush.jitter_scale = t.clamp(0.0, 1.0);
    }

    /// Set **Jitter Rotate** (per-dab texture-rotation scatter) from the slider's `0..1` track.
    pub fn set_brush_jitter_rotate(&mut self, t: f32) {
        self.paint.brush.jitter_rotate = t.clamp(0.0, 1.0);
    }

    /// Set **Jitter Spacing** (per-gap dab-spacing scatter) from the slider's `0..1` track.
    pub fn set_brush_jitter_spacing(&mut self, t: f32) {
        self.paint.brush.jitter_spacing = t.clamp(0.0, 1.0);
    }

    /// Reset the **Randomize Color** section to defaults (no per-dab colour scatter). Plain paint
    /// state — no undo / pixel touch. The Color-Ramp reset lives in [`super::ramp`], Tiling in
    /// [`super::tiling`].
    pub fn reset_brush_randomize(&mut self) {
        let d = BrushSpec::default();
        let b = &mut self.paint.brush;
        b.color_jitter_enabled = d.color_jitter_enabled;
        b.color_jitter_hue = d.color_jitter_hue;
        b.color_jitter_sat = d.color_jitter_sat;
        b.color_jitter_val = d.color_jitter_val;
    }

    /// Reset the **Texture** section to defaults — clears the assigned texture and its modulation
    /// (mapping / angle / Rake / Random / offset / size / params). The Color Ramp has its own reset.
    pub fn reset_brush_texture(&mut self) {
        self.paint.brush.texture = TextureSettings::default();
    }

    /// Reset the **Shape** section: clear the Shape image (the silhouette reverts to the falloff) and
    /// reset the Shape slot's rotation / offset / size + the Falloff preset & curve + the dab
    /// flatten/rotate gizmo to defaults.
    pub fn reset_brush_shape(&mut self) {
        self.clear_brush_shape_image();
        self.paint.brush.shape = TextureSettings::default();
        let d = BrushSpec::default();
        self.paint.brush.falloff = d.falloff;
        self.paint.brush.custom_falloff = d.custom_falloff;
        self.paint.brush.hardness = d.hardness;
        self.paint.brush.dab_flatten = d.dab_flatten;
        self.paint.brush.dab_angle_deg = d.dab_angle_deg;
    }

    /// Reset the **Stroke** section to defaults (method / spacing / Adjust-Strength / jitter group /
    /// dash / samples / stabilizer / rate / edge-to-edge). Accumulate (a top basic) and Tiling (its
    /// own section) are left untouched.
    pub fn reset_brush_stroke(&mut self) {
        let d = BrushSpec::default();
        let b = &mut self.paint.brush;
        b.stroke_method = d.stroke_method;
        b.spacing = d.spacing;
        b.space_attenuation = d.space_attenuation;
        b.jitter = d.jitter;
        b.jitter_absolute_px = d.jitter_absolute_px;
        b.jitter_unit = d.jitter_unit;
        b.jitter_scale = d.jitter_scale;
        b.jitter_rotate = d.jitter_rotate;
        b.jitter_spacing = d.jitter_spacing;
        b.dash_ratio = d.dash_ratio;
        b.dash_samples = d.dash_samples;
        b.input_samples = d.input_samples;
        b.stabilizer = d.stabilizer;
        b.airbrush_rate_s = d.airbrush_rate_s;
        b.edge_to_edge = d.edge_to_edge;
    }
}

/// Parse a `"id:value"` ramp-edit payload (stable stop id + an `f32`).
fn parse_id_f32(s: &str) -> Option<(u8, f32)> {
    let mut it = s.split(':');
    match (it.next()?.parse::<u8>(), it.next()?.parse::<f32>()) {
        (Ok(id), Ok(v)) => Some((id, v)),
        _ => None,
    }
}
