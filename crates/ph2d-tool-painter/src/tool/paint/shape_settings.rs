//! Imported-image slots (brush **Grain** image + **Shape** image) + the **Shape** geometry and
//! **Grain Depth** setters — the single UI-edit clamp source for those, split from `brush_settings`
//! for the workspace LOC cap. The Shape is the dab silhouette (its image *replaces* the falloff); the
//! Grain image is the texture inside it.

use super::brush_settings::BrushTextureImage;
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
};

impl PainterTool {
    // ── Brush Grain (texture) image ──────────────────────────────────────────────────────────

    /// Store an imported grayscale `lum` image (`width × height`, row-major) as the brush **Grain** and
    /// switch the kind to Image. Called by the shell after a file pick + decode.
    pub fn set_brush_texture_image(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        let was_none = self.paint.brush.texture.kind == TextureKind::None;
        self.paint.texture_image = Some(BrushTextureImage::new(lum, width, height));
        self.paint.brush.texture.kind = TextureKind::Image;
        self.paint.texture_image_pending = false;
        // Invalidate the cached stamp's baked Image mask.
        self.paint.texture_image_version = self.paint.texture_image_version.wrapping_add(1);
        // None→Grain (a direct image assign that didn't go through `set_brush_texture_kind`): flip the
        // Shape ramp colour→tone + reset the now-Grain colour ramp.
        if was_none {
            self.on_grain_assigned();
        }
    }

    /// Take (and clear) the "the user picked Image — open a file picker" request. The shell polls
    /// this each frame; on a successful pick it calls [`Self::set_brush_texture_image`].
    pub fn take_brush_texture_image_request(&mut self) -> bool {
        std::mem::take(&mut self.paint.texture_image_pending)
    }

    /// The brush's imported Image texture as `(luminance, w, h)` for the panel's Grain preview (the
    /// pixels can't live in the `Copy` snapshot). `None` if unassigned; gate publishes on the version.
    #[must_use]
    pub fn brush_texture_image(&self) -> Option<(&[u8], u32, u32)> {
        self.paint
            .texture_image
            .as_ref()
            .map(BrushTextureImage::parts)
    }

    /// Monotonic version of the brush texture image (bumped by [`Self::set_brush_texture_image`]).
    #[must_use]
    pub fn brush_texture_image_version(&self) -> u64 {
        self.paint.texture_image_version
    }

    // ── Grain Depth + Shape section setters ──────────────────────────────────────────────────

    /// Set the **Grain Depth** from the slider's `0..1` track (`1` = full bite, the default).
    pub fn set_brush_grain_depth(&mut self, t: f32) {
        self.paint.brush.grain_depth = t.clamp(0.0, 1.0);
    }

    /// Store an imported grayscale `lum` image (`width × height`, row-major) as the brush **Shape** (the
    /// silhouette tip) and switch the Shape kind to Image. Called by the shell (Hierarchy "Use as Brush
    /// Shape") after reading the sprite's pixels.
    pub fn set_brush_shape_image(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.paint.shape_image = Some(BrushTextureImage::new(lum, width, height));
        self.paint.brush.shape.kind = TextureKind::Image;
        self.paint.shape_image_pending = false;
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);
    }

    /// Set the Shape **source** from the dropdown's wire discriminant (the "Texture" picker; mirror of
    /// `set_brush_texture_kind` for the Grain slot). [`TextureKind::Image`] requests a file pick from the
    /// shell (the engine has no I/O); [`TextureKind::None`] reverts to the bare falloff; any procedural
    /// kind installs that pattern (masked by the falloff) and resets its params to the kind's defaults.
    /// All three drop any previously-imported image (a procedural Shape never reads pixels).
    pub fn set_brush_shape_kind(&mut self, k: u8) {
        let kind = TextureKind::from_u8(k);
        self.paint.brush.shape.kind = kind;
        // Changing the Shape source (None / procedural / a fresh single Image pick) invalidates any
        // captured multi-layer stack → drop it (and the Per-Layer Color mode + its UI). Without this, a
        // dropdown → None left the per-layer state on, keeping the panel rows AND routing a now-`None`
        // Shape into the coloured-stamp path (garbage paint). The multi-layer CAPTURE
        // (`set_brush_shape_layers`) re-populates it afterwards (Enio 2026-06-26).
        self.paint.shape_layers.clear();
        match kind {
            TextureKind::Image => self.paint.shape_image_pending = true, // shell opens a file picker
            TextureKind::None => self.paint.shape_image = None,
            _ => {
                self.paint.shape_image = None; // procedural Shape: no pixels
                self.reset_shape_params();
            }
        }
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);
    }

    /// Reset the Shape pattern params to the current kind's neutral defaults (mirror of the Grain's
    /// `reset_texture_params`). Called when a procedural Shape kind is picked.
    fn reset_shape_params(&mut self) {
        let mut params = [0.5; ph2d_painter_brush::MAX_TEX_PARAMS];
        for (i, s) in ph2d_painter_brush::param_specs(self.paint.brush.shape.kind)
            .iter()
            .enumerate()
        {
            params[i] = s.default;
        }
        self.paint.brush.shape.params = params;
    }

    /// Take (and clear) the "the user picked Image in the Shape dropdown — open a file picker" request.
    /// The shell polls this each frame; on a successful pick it calls [`Self::set_brush_shape_image`].
    pub fn take_brush_shape_image_request(&mut self) -> bool {
        std::mem::take(&mut self.paint.shape_image_pending)
    }

    /// Clear the Shape image (and reset the slot to `None`) — the silhouette reverts to the falloff.
    pub fn clear_brush_shape_image(&mut self) {
        self.paint.shape_image = None;
        self.paint.shape_layers.clear();
        self.paint.brush.shape.kind = TextureKind::None;
        self.paint.shape_image_pending = false;
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);
    }

    // ── Multi-layer Shape (per-layer colour) ──────────────────────────────────────────────────────

    /// Capture the **active Painter document's** visible raster layers (z-ordered, bottom-to-top) as a
    /// multi-layer Shape — each layer's ALPHA becomes its silhouette, recolourable per layer. The source
    /// the spec calls for (the sprite's own layers / an imported multi-layer doc); fully in-tool, reading
    /// the live buffers (`canvas_rgba` for the active layer, `images` for the rest). A no-op with no
    /// visible raster layers. Groups / non-raster layers are skipped (top-level rasters only, v1).
    pub fn capture_layers_as_brush_shape(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let n = (w as usize) * (h as usize);
        let active = self.layers.active();
        // Top-level ids are top-to-bottom; the engine composites bottom-to-top, so reverse.
        let ids: Vec<crate::layers::LayerId> = self.layers.root().iter().copied().rev().collect();
        let mut layers: Vec<(Vec<u8>, u32, u32)> = Vec::new();
        for id in ids {
            let Some(layer) = self.layers.get(id) else {
                continue;
            };
            if !layer.visible || !matches!(layer.kind, crate::layers::LayerKind::Raster(_)) {
                continue;
            }
            let rgba: &[u8] = if Some(id) == active {
                self.canvas_rgba.as_slice()
            } else if let Some(img) = self.images.get(&id) {
                img.rgba8.as_slice()
            } else {
                continue;
            };
            if rgba.len() < n * 4 {
                continue;
            }
            // The layer's silhouette = its alpha channel (where the layer has content).
            let lum: Vec<u8> = (0..n).map(|i| rgba[i * 4 + 3]).collect();
            layers.push((lum, w, h));
        }
        if !layers.is_empty() {
            self.set_brush_shape_layers(layers);
        }
    }

    /// The number of capturable Shape layers in the active document (visible top-level raster layers).
    /// Drives whether the panel's "Use Document Layers" button shows (only with `> 1`).
    #[must_use]
    pub fn capturable_layer_count(&self) -> usize {
        self.layers
            .root()
            .iter()
            .filter(|&&id| {
                self.layers.get(id).is_some_and(|l| {
                    l.visible && matches!(l.kind, crate::layers::LayerKind::Raster(_))
                })
            })
            .count()
    }

    /// Store a **multi-layer** Shape: `layers` are `(luminance, w, h)` in bottom-to-top z-order (the
    /// Painter document's own visible raster layers, or an imported multi-layer doc), capped at
    /// [`super::shape_layers::MAX_SHAPE_LAYERS`]. Also flattens them (alpha-over) into the single
    /// [`super::PaintState::shape_image`] so the OFF-mode / colour-ramp / preview single-image path is
    /// unchanged. The "Per-Layer Color" checkbox appears (panel) once `> 1` layer is captured.
    pub fn set_brush_shape_layers(&mut self, layers: Vec<(Vec<u8>, u32, u32)>) {
        self.paint.shape_layers.set_layers(layers);
        match self.paint.shape_layers.flatten() {
            Some((lum, w, h)) => {
                self.paint.shape_image = Some(BrushTextureImage::new(lum, w, h));
                self.paint.brush.shape.kind = TextureKind::Image;
            }
            None => self.paint.shape_image = None,
        }
        self.paint.shape_image_pending = false;
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);
    }

    /// Toggle the **Per-Layer Color** mode (each layer paints its own colour; the higher above the
    /// lower). While on, the Shape colour ramp is hidden + nullified (the route uses the coloured stamp).
    pub fn toggle_brush_shape_per_layer_color(&mut self) {
        self.paint.shape_layers.toggle_per_layer_color();
    }

    /// Toggle Shape layer `i`'s custom-colour checkbox (off ⇒ it paints the brush base colour).
    pub fn toggle_brush_shape_layer_color(&mut self, i: usize) {
        self.paint.shape_layers.toggle_layer_color(i);
    }

    /// Set Shape layer `i`'s custom colour (straight RGB); turns its checkbox on.
    pub fn set_brush_shape_layer_color(&mut self, i: usize, rgb: [f32; 3]) {
        self.paint.shape_layers.set_layer_color(i, rgb);
    }

    /// Render the multi-layer **coloured Shape preview** (premultiplied RGBA, `size×size`) for the panel:
    /// the per-layer composite (each layer tinted, higher above lower) at the current Shape frame. `None`
    /// unless Per-Layer Color is on with captured layers. A single small composite — cheap; the panel
    /// composites it over its checker. Tracks the live Shape transform / colours (re-rendered per frame).
    #[must_use]
    pub fn brush_shape_color_preview(&self) -> Option<(Vec<u8>, u32)> {
        if !self.paint.shape_layers.is_color_mode() {
            return None;
        }
        const SIZE: u32 = 128;
        let masks = self.paint.shape_layers.masks();
        let colors = self
            .paint
            .shape_layers
            .resolved_colors(self.paint.brush.color);
        let grain = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let stamp = ph2d_painter_brush::render_color_stamp_mask(
            &self.paint.brush,
            &masks,
            &colors,
            grain.as_ref(),
            SIZE,
        );
        Some((stamp.data().to_vec(), SIZE))
    }

    /// `(layer count, per-layer-colour mode on)` for the panel — the checkbox only shows for `> 1` layer.
    #[must_use]
    pub fn brush_shape_layers(&self) -> (u8, bool) {
        (
            self.paint.shape_layers.len().min(u8::MAX as usize) as u8,
            self.paint.shape_layers.per_layer_color(),
        )
    }

    /// `(color_on, color)` arrays for the panel's per-layer rows (entries `0..count` are valid).
    #[must_use]
    pub fn brush_shape_layer_colors(
        &self,
    ) -> (
        [bool; super::shape_layers::MAX_SHAPE_LAYERS],
        [[f32; 3]; super::shape_layers::MAX_SHAPE_LAYERS],
    ) {
        let (_, on, col) = self.paint.shape_layers.snapshot();
        (on, col)
    }

    /// The brush's imported Shape image as `(luminance, w, h)` for the panel's Shape preview. `None` if
    /// unassigned; the bridge publishes on the version (the pixels can't live in the `Copy` snapshot).
    #[must_use]
    pub fn brush_shape_image(&self) -> Option<(&[u8], u32, u32)> {
        self.paint
            .shape_image
            .as_ref()
            .map(BrushTextureImage::parts)
    }

    /// Monotonic version of the brush Shape image (bumped by [`Self::set_brush_shape_image`]).
    #[must_use]
    pub fn brush_shape_image_version(&self) -> u64 {
        self.paint.shape_image_version
    }

    /// Set a procedural Shape per-pattern param `slot` (Contrast / Brightness / the kind's knob) from the
    /// slider's `0..1` track; out-of-range slots are ignored. Mirror of `set_brush_texture_param_norm`.
    pub fn set_brush_shape_param_norm(&mut self, slot: usize, t: f32) {
        if slot < self.paint.brush.shape.params.len() {
            self.paint.brush.shape.params[slot] = t.clamp(0.0, 1.0);
        }
    }

    /// Set the Shape rotation from the slider's `0..1` track → `0..=TEX_ANGLE_MAX_DEG` degrees.
    pub fn set_brush_shape_angle_norm(&mut self, t: f32) {
        self.paint.brush.shape.angle_deg =
            (t.clamp(0.0, 1.0) * f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
    }

    /// Toggle Shape "Rake" (the silhouette rotation follows the stroke direction).
    pub fn toggle_brush_shape_rake(&mut self) {
        let shape = &mut self.paint.brush.shape;
        shape.rake = !shape.rake;
    }

    /// Toggle Shape "Random" (the silhouette rotation is randomised per dab).
    pub fn toggle_brush_shape_random(&mut self) {
        let shape = &mut self.paint.brush.shape;
        shape.random_angle = !shape.random_angle;
    }

    /// Set the Shape offset for `axis` (`0` = X, `1` = Y) from the slider's `0..1` track.
    pub fn set_brush_shape_offset_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_OFFSET_MAX - TEX_OFFSET_MIN;
            self.paint.brush.shape.offset[axis] = TEX_OFFSET_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set the Shape scale for `axis` (`0` = X, `1` = Y) from the slider's `0..1` track.
    pub fn set_brush_shape_size_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_SIZE_MAX - TEX_SIZE_MIN;
            self.paint.brush.shape.size[axis] = TEX_SIZE_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    // ── Real-value setters (the number-field path, Enio 2026-06-25) — the chip shows + scrubs the real
    //    value (degrees / tile-fractions / scale), so these clamp the raw value (mirror the Grain's). ──

    /// Set the Shape rotation directly in whole **degrees**.
    pub fn set_brush_shape_angle(&mut self, deg: f32) {
        self.paint.brush.shape.angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
    }

    /// Set the Shape **offset** for `axis` directly (real tile fractions, clamped).
    pub fn set_brush_shape_offset(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.shape.offset[axis] = v.clamp(TEX_OFFSET_MIN, TEX_OFFSET_MAX);
        }
    }

    /// Set the Shape **scale** for `axis` directly (real, clamped).
    pub fn set_brush_shape_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.shape.size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
    }
}
