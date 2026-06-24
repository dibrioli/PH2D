//! Texture-layer host — create a [`LayerKind::Texture`] layer and edit its spec (kind / params /
//! size / offset / Color Ramp) in real time. Every edit re-renders the layer's canvas-sized RGBA8
//! buffer from the spec so the compositor (which blends a Texture layer exactly like a raster) shows
//! the change live. `impl PainterTool` (one of several blocks in this crate).
//!
//! The active Texture layer keeps its pixels in `canvas_rgba` (the "active is never in images"
//! invariant); a non-active one keeps them in `images[id]`. The re-render targets whichever holds the
//! live pixels, so a slider drag on the selected texture layer updates the preview immediately.

use super::super::*;
use crate::layers::{HARD_CAP_LAYERS, TextureLayer};
use ph2d_color::{RampColorMode, RampInterp, RampStop};
use ph2d_painter_brush::{
    MAX_TEX_PARAMS, RampAlphaMode, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN,
    TextureKind, param_specs, render_texture_layer,
};

impl PainterTool {
    /// Add a Texture layer (procedural fill recoloured by a Color Ramp) at the top of the stack and
    /// make it the active/selected layer, with its pixels rendered into `canvas_rgba`. No-op (`None`)
    /// before a source is set or at the hard cap.
    pub fn add_texture_layer(&mut self) -> Option<RtLayerId> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.layers.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let undo_before = self.snapshot_model();
        self.flush_active_to_images();
        let spec = TextureLayer::default();
        let buf = render_texture_buffer(&spec, w, h);
        let id = self.layers.add_texture(spec)?; // sets active = id (top)
        self.images.remove(&id); // active lives in canvas_rgba, not images
        self.canvas_rgba = Arc::new(buf);
        self.bump_layer_pixels(Some(id));
        self.commit_structural_edit(undo_before);
        self.reset_selection_to(id);
        self.invalidate_composite();
        Some(id)
    }

    /// The active layer iff it is a Texture layer (the panel only shows the texture editor for the
    /// active texture layer, so its edits target this).
    fn active_texture_id(&self) -> Option<RtLayerId> {
        let a = self.layers.active()?;
        self.layers.get(a).filter(|l| l.is_texture()).map(|_| a)
    }

    /// Re-render `id`'s texture into the buffer that currently holds its pixels (`canvas_rgba` when it
    /// is active, else `images[id]`), bump its pixel version, and invalidate the composite so the live
    /// preview re-renders. No-op before a source / for a non-texture id.
    ///
    /// PERF (known): this is **O(canvas)** — the whole sprite is re-sampled (procedural noise can be
    /// multi-octave) + ramp-LUT-indexed + sRGB-encoded on the CPU, with no dirty-rect, on EVERY edit.
    /// On the typical sprite this is fine; on a 2K/4K canvas a continuous slider drag re-renders
    /// millions of pixels per `SetValue` and will stutter. Follow-up (mirror of the texture-brush
    /// stamp-cache + the adjustment dirty-rect lever): render a downscaled proxy during a live drag and
    /// the full-res buffer once on release, or coalesce the per-frame `SetValue` burst.
    fn rerender_texture_layer(&mut self, id: RtLayerId) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let Some(spec) = self.layers.get(id).and_then(|l| match &l.kind {
            LayerKind::Texture(t) => Some(t.clone()),
            _ => None,
        }) else {
            return;
        };
        let buf = render_texture_buffer(&spec, w, h);
        if self.layers.active() == Some(id) {
            self.canvas_rgba = Arc::new(buf);
        } else {
            self.images.insert(
                id,
                LayerImage {
                    width: w,
                    height: h,
                    rgba8: buf,
                },
            );
        }
        self.bump_layer_pixels(Some(id));
        self.invalidate_composite();
    }

    // ── Per-property setters (act on the active texture layer; each re-renders live) ──

    /// Set the texture kind from a wire discriminant and reset the per-pattern params to that kind's
    /// `param_specs` defaults (so each pattern starts from sensible knobs).
    pub fn set_texture_layer_kind(&mut self, k: u8) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            t.kind = k;
            let mut params = [0.5f32; MAX_TEX_PARAMS];
            for (i, s) in param_specs(TextureKind::from_u8(k)).iter().enumerate() {
                params[i] = s.default;
            }
            t.params = params;
        }
        self.rerender_texture_layer(id);
    }

    /// Set per-pattern parameter `slot` from the slider's `0..1` track (stored normalized).
    pub fn set_texture_layer_param_norm(&mut self, slot: usize, t01: f32) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if slot < MAX_TEX_PARAMS
            && let Some(t) = self.layers.texture_mut(id)
        {
            t.params[slot] = t01.clamp(0.0, 1.0);
        }
        self.rerender_texture_layer(id);
    }

    /// Set the texture scale for `axis` (`0` = X, `1` = Y) from the slider's `0..1` track.
    pub fn set_texture_layer_size_norm(&mut self, axis: usize, t01: f32) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if axis < 2
            && let Some(t) = self.layers.texture_mut(id)
        {
            t.size[axis] = TEX_SIZE_MIN + t01.clamp(0.0, 1.0) * (TEX_SIZE_MAX - TEX_SIZE_MIN);
        }
        self.rerender_texture_layer(id);
    }

    /// Set the texture offset for `axis` (`0` = X, `1` = Y) from the slider's `0..1` track.
    pub fn set_texture_layer_offset_norm(&mut self, axis: usize, t01: f32) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if axis < 2
            && let Some(t) = self.layers.texture_mut(id)
        {
            t.offset[axis] =
                TEX_OFFSET_MIN + t01.clamp(0.0, 1.0) * (TEX_OFFSET_MAX - TEX_OFFSET_MIN);
        }
        self.rerender_texture_layer(id);
    }

    // ── Color Ramp setters (mirror the brush ramp ops, but on the layer's own ramp) ──

    /// Toggle whether the Color Ramp recolours the texture (off → grayscale fill).
    pub fn toggle_texture_layer_ramp_enabled(&mut self) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            t.ramp_enabled = !t.ramp_enabled;
        }
        self.rerender_texture_layer(id);
    }

    /// Reset the active texture layer's **Color Ramp** to defaults (ramp off, default gradient, alpha
    /// off) — the layer-editor counterpart of [`super::super::PainterTool::reset_brush_color_ramp`].
    pub fn reset_texture_layer_ramp(&mut self) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            t.ramp = ph2d_color::ColorRamp::default();
            t.ramp_enabled = false;
            t.ramp_alpha_mode = RampAlphaMode::None.to_u8();
        }
        self.rerender_texture_layer(id);
    }

    /// Set the ramp colour-interpolation space (`RampColorMode` wire).
    pub fn set_texture_layer_ramp_mode(&mut self, m: u8) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            t.ramp.color_mode = RampColorMode::from_u8(m);
        }
        self.rerender_texture_layer(id);
    }

    /// Set the ramp interpolation mode (`RampInterp` wire).
    pub fn set_texture_layer_ramp_interp(&mut self, i: u8) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            t.ramp.interp = RampInterp::from_u8(i);
        }
        self.rerender_texture_layer(id);
    }

    /// Set what the ramp colour's alpha does (`RampAlphaMode` wire): opaque recolour / translucent.
    pub fn set_texture_layer_ramp_alpha_mode(&mut self, m: u8) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            t.ramp_alpha_mode = RampAlphaMode::from_u8(m).to_u8();
        }
        self.rerender_texture_layer(id);
    }

    /// Move the ramp stop with stable `sid` to normalized position `pos` (tracked by id, so it may be
    /// dragged past its neighbours).
    pub fn texture_layer_ramp_move_stop(&mut self, sid: u8, pos: f32) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id)
            && let Some(idx) = t.ramp.index_of_id(sid)
        {
            t.ramp.set_position(idx, pos.clamp(0.0, 1.0));
        }
        self.rerender_texture_layer(id);
    }

    /// Set the colour of the ramp stop with stable `sid` from straight-sRGB **RGBA** bytes (picker
    /// space): RGB → the ramp's linear space, alpha straight.
    pub fn texture_layer_ramp_set_stop_color(&mut self, sid: u8, rgba: [u8; 4]) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        let lin = ph2d_color::srgb::srgb_to_linear_byte;
        if let Some(t) = self.layers.texture_mut(id)
            && let Some(idx) = t.ramp.index_of_id(sid)
        {
            t.ramp.set_color(
                idx,
                [
                    lin(rgba[0]),
                    lin(rgba[1]),
                    lin(rgba[2]),
                    f32::from(rgba[3]) / 255.0,
                ],
            );
        }
        self.rerender_texture_layer(id);
    }

    /// Add a ramp stop at the midpoint of the largest gap (coloured on the current gradient there).
    pub fn texture_layer_ramp_add_stop(&mut self) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            let pos = largest_gap_midpoint(&t.ramp);
            let color = t.ramp.eval(pos);
            t.ramp.add_stop(RampStop::new(pos, color));
        }
        self.rerender_texture_layer(id);
    }

    /// Remove the last (highest-position) ramp stop (the ramp always keeps at least one).
    pub fn texture_layer_ramp_remove_last_stop(&mut self) {
        let Some(id) = self.active_texture_id() else {
            return;
        };
        if let Some(t) = self.layers.texture_mut(id) {
            let last = t.ramp.len().saturating_sub(1);
            t.ramp.remove_stop(last);
        }
        self.rerender_texture_layer(id);
    }

    /// Route a Texture-layer panel event: the "+ Texture" create action (always), plus — when a
    /// Texture layer is active — its Texture-section edits (the shared fixed-id widgets), applied to
    /// the layer instead of the brush. Returns `true` when consumed; `false` falls through to the
    /// brush/layer handlers (so the brush's own texture section keeps working).
    pub(crate) fn route_texture_layer_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        // "+ Texture" creates a layer regardless of the current selection / dock view.
        if let PanelEvent::Click(id) = event
            && *id == core_ids::PAINTER_LAYERS_ADD_TEXTURE
        {
            self.add_texture_layer();
            return true;
        }
        // The shared Texture-section widgets (PAINTER_BRUSH_TEXTURE_*) are painted by EITHER the
        // brush's Texture section (Brush dock view) OR the layer's inline editor (Layers dock view).
        // Only intercept them for the layer when the Layers view is showing AND a texture layer is
        // active — otherwise the brush's own texture section must keep reaching the brush handlers.
        if !self.dock_shows_layers || self.active_texture_id().is_none() {
            return false;
        }
        match event {
            PanelEvent::Click(id) => match *id {
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE => {
                    self.toggle_texture_layer_ramp_enabled();
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD => {
                    self.texture_layer_ramp_add_stop();
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE => {
                    self.texture_layer_ramp_remove_last_stop();
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_COLOR_RAMP_RESET => {
                    self.reset_texture_layer_ramp();
                    true
                }
                _ => false,
            },
            PanelEvent::SetValue(id, v) => {
                let v = *v as f32;
                match *id {
                    x if x == core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X => {
                        self.set_texture_layer_size_norm(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y => {
                        self.set_texture_layer_size_norm(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X => {
                        self.set_texture_layer_offset_norm(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y => {
                        self.set_texture_layer_offset_norm(1, v);
                        true
                    }
                    x => {
                        if let Some(slot) = core_ids::PAINTER_BRUSH_TEXTURE_PARAMS
                            .iter()
                            .position(|&p| p == x)
                        {
                            self.set_texture_layer_param_norm(slot, v);
                            true
                        } else {
                            false
                        }
                    }
                }
            }
            PanelEvent::SelectOption(id, value) => match *id {
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_KIND => {
                    if let Ok(k) = value.parse::<u8>() {
                        self.set_texture_layer_kind(k);
                    }
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE => {
                    if let Ok(m) = value.parse::<u8>() {
                        self.set_texture_layer_ramp_mode(m);
                    }
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP => {
                    if let Ok(i) = value.parse::<u8>() {
                        self.set_texture_layer_ramp_interp(i);
                    }
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE => {
                    if let Ok(m) = value.parse::<u8>() {
                        self.set_texture_layer_ramp_alpha_mode(m);
                    }
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH => {
                    let mut it = value.split(',').filter_map(|p| p.parse::<i32>().ok());
                    if let (Some(sid), Some(r), Some(g), Some(b), Some(a)) =
                        (it.next(), it.next(), it.next(), it.next(), it.next())
                    {
                        self.texture_layer_ramp_set_stop_color(
                            sid as u8,
                            [r as u8, g as u8, b as u8, a as u8],
                        );
                    }
                    true
                }
                x if x == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT => {
                    let mut it = value.split(':');
                    if let (Some(Ok(sid)), Some(Ok(pos))) = (
                        it.next().map(str::parse::<u8>),
                        it.next().map(str::parse::<f32>),
                    ) {
                        self.texture_layer_ramp_move_stop(sid, pos);
                    }
                    true
                }
                _ => false,
            },
            PanelEvent::Toggle(_, _) => false,
        }
    }
}

/// The position to insert a new ramp stop: the midpoint of the largest gap between existing stops (and
/// the edges to `0` / `1`). Mirrors the brush's `ramp_add_stop`.
fn largest_gap_midpoint(ramp: &ph2d_color::ColorRamp) -> f32 {
    let stops = ramp.stops();
    match stops.len() {
        0 => 0.5,
        1 => {
            if stops[0].pos < 0.5 {
                1.0
            } else {
                0.0
            }
        }
        _ => {
            let mut best = (0.0f32, -1.0f32); // (midpoint, gap)
            let mut prev = 0.0;
            for s in stops {
                if s.pos - prev > best.1 {
                    best = ((prev + s.pos) * 0.5, s.pos - prev);
                }
                prev = s.pos;
            }
            if 1.0 - prev > best.1 {
                best.0 = (prev + 1.0) * 0.5;
            }
            best.0
        }
    }
}

/// Render a Texture layer's spec into a fresh canvas-sized straight-sRGB8 RGBA buffer.
fn render_texture_buffer(spec: &TextureLayer, w: u32, h: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
    let lut = texture_layer_ramp_lut(spec);
    let ramp = lut
        .as_ref()
        .map(|l| (&l[..], RampAlphaMode::from_u8(spec.ramp_alpha_mode)));
    render_texture_layer(
        TextureKind::from_u8(spec.kind),
        spec.params,
        spec.size,
        spec.offset,
        None, // imported Image pixels aren't held by a texture layer → Image kind fills flat
        ramp,
        &mut buf,
        w,
        h,
    );
    buf
}

/// Bake the layer's `ColorRamp` into a 256-entry **sRGB-straight RGBA** LUT (RGB linear→sRGB, alpha
/// straight) — the exact form `render_texture_layer` + the panel preview consume. `None` (→ grayscale)
/// when the ramp is disabled.
fn texture_layer_ramp_lut(spec: &TextureLayer) -> Option<[[f32; 4]; 256]> {
    if !spec.ramp_enabled {
        return None;
    }
    let mut lut = [[0.0f32; 4]; 256];
    spec.ramp.bake_into(&mut lut); // linear RGBA in the chosen interp/colour space
    for c in lut.iter_mut() {
        c[0] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[0])) / 255.0;
        c[1] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[1])) / 255.0;
        c[2] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[2])) / 255.0;
        // alpha stays straight
    }
    Some(lut)
}
