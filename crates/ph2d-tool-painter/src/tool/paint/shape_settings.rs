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
        self.paint.texture_image = Some(BrushTextureImage::new(lum, width, height));
        self.paint.brush.texture.kind = TextureKind::Image;
        self.paint.texture_image_pending = false;
        // Invalidate the cached stamp's baked Image mask.
        self.paint.texture_image_version = self.paint.texture_image_version.wrapping_add(1);
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
        self.paint.brush.shape.kind = TextureKind::None;
        self.paint.shape_image_pending = false;
        self.paint.shape_image_version = self.paint.shape_image_version.wrapping_add(1);
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
}
