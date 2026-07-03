//! Brush **Grain-texture / Stencil / Dab** parameter setters — the Texture-section UI-edit clamp source.
//! Split from `brush_settings` for the workspace LOC cap; a child module of `paint`, so it keeps access to
//! `PaintState`'s module-private fields.

use super::PainterTool;
use ph2d_painter_brush::{
    TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
    TextureMapping,
};

impl PainterTool {
    /// Set the brush texture (Grain) kind from a wire discriminant (out-of-range → None). Picking
    /// [`TextureKind::Image`] requests a file pick from the shell (the engine has no I/O).
    pub fn set_brush_texture_kind(&mut self, k: u8) {
        let was_none = self.paint.brush.texture.kind == TextureKind::None;
        let kind = TextureKind::from_u8(k);
        self.paint.brush.texture.kind = kind;
        self.reset_texture_params();
        if kind == TextureKind::Image {
            self.paint.texture_image_pending = true;
        }
        if was_none && kind != TextureKind::None {
            self.on_grain_assigned(); // None→Grain: flip Shape colour→tone + reset the Grain ramp
        }
        self.arm_stencil_preview();
    }

    /// Assign the default procedural texture (Noise) — the Texture section's "New" button.
    pub fn new_brush_texture(&mut self) {
        let was_none = self.paint.brush.texture.kind == TextureKind::None;
        self.paint.brush.texture.kind = TextureKind::Noise;
        self.reset_texture_params();
        if was_none {
            self.on_grain_assigned(); // None→Grain: flip Shape colour→tone + reset the Grain ramp
        }
    }

    /// Set the texture mapping from a wire discriminant (out-of-range → View Plane). Re-fits a loaded
    /// Grain Image's aspect for the new mapping (Stencil → the rect; the rest → the Size), so the image
    /// is never squashed in any mode (Enio 2026-06-28).
    pub fn set_brush_texture_mapping(&mut self, m: u8) {
        let m = TextureMapping::from_u8(m);
        self.paint.brush.texture.mapping = m;
        self.fit_grain_image_aspect(m);
        self.arm_stencil_preview();
    }

    /// Set the texture rotation from the slider's `0..1` track → `0..=TEX_ANGLE_MAX_DEG` degrees.
    pub fn set_brush_texture_angle_norm(&mut self, t: f32) {
        self.paint.brush.texture.angle_deg =
            (t.clamp(0.0, 1.0) * f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
    }

    /// Toggle "Rake" (the texture rotation follows the stroke direction).
    pub fn toggle_brush_texture_rake(&mut self) {
        let tex = &mut self.paint.brush.texture;
        tex.rake = !tex.rake;
    }

    /// Toggle "Random" (the texture rotation is randomised per dab).
    pub fn toggle_brush_texture_random(&mut self) {
        let tex = &mut self.paint.brush.texture;
        tex.random_angle = !tex.random_angle;
    }

    /// Set the texture offset for `axis` (`0`=X / `1`=Y) from the `0..1` track → `[TEX_OFFSET_MIN, MAX]` (tiles).
    pub fn set_brush_texture_offset_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_OFFSET_MAX - TEX_OFFSET_MIN;
            self.paint.brush.texture.offset[axis] = TEX_OFFSET_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set the texture scale for `axis` (`0`=X / `1`=Y) from the `0..1` track → `[TEX_SIZE_MIN, TEX_SIZE_MAX]`.
    pub fn set_brush_texture_size_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_SIZE_MAX - TEX_SIZE_MIN;
            self.paint.brush.texture.size[axis] = TEX_SIZE_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set per-pattern parameter `slot` from the `0..1` track (normalized; each pattern maps its own range).
    pub fn set_brush_texture_param_norm(&mut self, slot: usize, t: f32) {
        if slot < ph2d_painter_brush::MAX_TEX_PARAMS {
            self.paint.brush.texture.params[slot] = t.clamp(0.0, 1.0);
        }
        self.arm_stencil_preview();
    }

    /// Enable / disable the texture **Color Ramp** (on → the texture's scalar drives the per-texel colour).
    pub fn set_texture_ramp_enabled(&mut self, on: bool) {
        self.paint.texture_ramp_enabled = on;
    }

    /// Replace the texture Color Ramp (`ph2d_color::ColorRamp`); re-bakes the LUT before the next stamp.
    pub fn set_texture_ramp(&mut self, ramp: ph2d_color::ColorRamp) {
        self.paint.texture_ramp = ramp;
        self.paint.texture_ramp_dirty = true;
    }

    /// The current texture Color Ramp (for the panel widget + tests).
    #[must_use]
    pub fn texture_ramp(&self) -> &ph2d_color::ColorRamp {
        &self.paint.texture_ramp
    }

    /// Whether the texture Color Ramp is enabled.
    #[must_use]
    pub fn texture_ramp_enabled(&self) -> bool {
        self.paint.texture_ramp_enabled
    }

    /// Reset the texture params to the active kind's `param_specs` defaults (unused slots stay at the
    /// neutral `0.5`). Called on a kind change so each pattern starts from its own sensible values.
    fn reset_texture_params(&mut self) {
        let mut params = [0.5; ph2d_painter_brush::MAX_TEX_PARAMS];
        for (i, s) in ph2d_painter_brush::param_specs(self.paint.brush.texture.kind)
            .iter()
            .enumerate()
        {
            params[i] = s.default;
        }
        self.paint.brush.texture.params = params;
    }

    /// Set the absolute texture offset for `axis` (tile fractions, clamped) — used by the Stencil
    /// drag gesture, which computes a target value directly rather than a slider track.
    pub fn set_brush_texture_offset(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.offset[axis] = v.clamp(TEX_OFFSET_MIN, TEX_OFFSET_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the absolute texture scale for `axis` (clamped) — used by the Stencil drag gesture.
    pub fn set_brush_texture_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the texture rotation directly in whole **degrees** (the number-field path, Enio 2026-06-25).
    pub fn set_brush_texture_angle(&mut self, deg: f32) {
        self.paint.brush.texture.angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
        self.arm_stencil_preview();
    }

    /// Set the **Stencil** rect centre for `axis` (`−1..1`, clamped) — the gizmo's own offset, separate
    /// from the texture tiling. Driven by both the Stencil card's number box and the on-canvas drag.
    pub fn set_brush_stencil_offset(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.stencil_offset[axis] = v.clamp(TEX_OFFSET_MIN, TEX_OFFSET_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the **Stencil** rect half-extent fraction for `axis` (`0.1..10`, clamped; `0.5` = 50 % of
    /// the sprite) — the gizmo's own size, separate from the texture tiling.
    pub fn set_brush_stencil_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.stencil_size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the **Stencil** rect rotation directly in whole **degrees** — the gizmo's own angle.
    pub fn set_brush_stencil_angle(&mut self, deg: f32) {
        self.paint.brush.texture.stencil_angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
        self.arm_stencil_preview();
    }

    /// Set the **Dab Flatten** (`0..1`, clamped) — the Shape-panel gizmo squishes the dab footprint
    /// (falloff + Shape + View-Grain) into an ellipse. The engine clamps the effective minor axis.
    pub fn set_brush_dab_flatten(&mut self, v: f32) {
        self.paint.brush.dab_flatten = v.clamp(0.0, 1.0);
    }

    /// Set the **Dab rotation** of the flatten/rotate gizmo in whole **degrees**.
    pub fn set_brush_dab_angle(&mut self, deg: f32) {
        self.paint.brush.dab_angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
    }
}
