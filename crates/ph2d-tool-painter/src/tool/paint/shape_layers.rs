//! The brush **Shape** as a stack of z-ordered luminance layers, for the per-layer-colour mode (a
//! multi-colour, 3-D-looking tip). Captured from the Painter document's own raster layers (the active
//! sprite's layers, or an imported multi-layer doc). Held here because the heavy pixels can't live in
//! the `Copy` `BrushSpec`. Split from `paint.rs` for the workspace LOC cap.
//!
//! Two modes share the same captured stack:
//! - **Per-Layer Color OFF** (default): the layers are flattened (alpha-over, bottom→top) into the
//!   single [`super::PaintState::shape_image`], so the Shape behaves exactly like a one-layer image and
//!   the Shape colour ramp works as before — the flatten is computed once at capture.
//! - **Per-Layer Color ON**: each layer is painted in its own colour (a custom pick, or the brush base
//!   colour) and the higher layer paints above the lower one — baked into the cached coloured stamp
//!   ([`ph2d_painter_brush::render_color_stamp_mask`]); the ramp section is hidden + nullified.

use super::brush_settings::BrushTextureImage;
use ph2d_painter_brush::texture::ImageMask;

/// Max Shape layers the per-layer-colour UI exposes (one row per layer) + the snapshot carries. A deep
/// stack past this is flattened-only (no per-layer colour) — well beyond any hand-authored brush tip.
pub const MAX_SHAPE_LAYERS: usize = 16;

/// The captured Shape layer stack + the per-layer-colour mode state. `layers` are **bottom-to-top**
/// (last = topmost), so the engine composites them in order. `color_on[i]` / `color[i]` align with
/// `layers[i]`. Empty `layers` ⇒ no multi-layer Shape (the single `shape_image` path is used).
#[derive(Default)]
pub(super) struct ShapeLayers {
    layers: Vec<BrushTextureImage>,
    /// The mode toggle (the "Per-Layer Color" checkbox). Only meaningful with ≥ 1 captured layer.
    per_layer_color: bool,
    /// Per-layer "use a custom colour" toggle (`false` ⇒ the layer uses the brush base colour).
    color_on: Vec<bool>,
    /// Per-layer custom colour (straight RGB), used when `color_on[i]`.
    color: Vec<[f32; 3]>,
    /// Bumped on any change that re-bakes the coloured stamp (layers / mode / a colour / a toggle).
    version: u64,
}

impl ShapeLayers {
    /// Replace the captured stack with `layers` (`(luminance, w, h)`, bottom-to-top), capped at
    /// [`MAX_SHAPE_LAYERS`]. Resets the per-layer colours to "off" (each layer paints the brush colour
    /// until the artist assigns one). Does NOT change the mode toggle. Bumps the version.
    pub(super) fn set_layers(&mut self, layers: Vec<(Vec<u8>, u32, u32)>) {
        let n = layers.len().min(MAX_SHAPE_LAYERS);
        self.layers = layers
            .into_iter()
            .take(n)
            .map(|(lum, w, h)| BrushTextureImage::new(lum, w, h))
            .collect();
        self.color_on = vec![false; n];
        self.color = vec![[0.0, 0.0, 0.0]; n];
        self.version = self.version.wrapping_add(1);
    }

    /// Drop the stack (revert to the single-image / falloff Shape).
    pub(super) fn clear(&mut self) {
        self.layers.clear();
        self.color_on.clear();
        self.color.clear();
        self.per_layer_color = false;
        self.version = self.version.wrapping_add(1);
    }

    /// Number of captured layers.
    pub(super) fn len(&self) -> usize {
        self.layers.len()
    }

    /// No layers captured.
    pub(super) fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// The per-layer-colour mode is engaged (toggle on + at least one captured layer) — the route
    /// dispatcher uses this to pick the coloured-stamp path, and the panel to hide the ramp section.
    pub(super) fn is_color_mode(&self) -> bool {
        self.per_layer_color && !self.layers.is_empty()
    }

    /// The mode toggle (raw, regardless of whether layers are present).
    pub(super) fn per_layer_color(&self) -> bool {
        self.per_layer_color
    }

    /// Monotonic version (cache invalidation key for the coloured stamp).
    pub(super) fn version(&self) -> u64 {
        self.version
    }

    /// Borrow the layers as engine [`ImageMask`]s (bottom-to-top), for the coloured-stamp bake.
    pub(super) fn masks(&self) -> Vec<ImageMask<'_>> {
        self.layers.iter().map(BrushTextureImage::as_mask).collect()
    }

    /// Each layer's resolved colour: its custom pick when `color_on[i]`, else the brush base `brush_color`
    /// (the "Color" at the top of the brush panel) — exactly the substitution the spec calls for.
    pub(super) fn resolved_colors(&self, brush_color: [f32; 3]) -> Vec<[f32; 3]> {
        (0..self.layers.len())
            .map(|i| {
                if self.color_on.get(i).copied().unwrap_or(false) {
                    self.color.get(i).copied().unwrap_or(brush_color)
                } else {
                    brush_color
                }
            })
            .collect()
    }

    /// Snapshot the per-layer colour assignments `(color_on, color)`, for re-applying across an automatic
    /// re-capture of the same Shape source (so editing the source sprite keeps the colours).
    pub(super) fn colors_snapshot(&self) -> (Vec<bool>, Vec<[f32; 3]>) {
        (self.color_on.clone(), self.color.clone())
    }

    /// Re-apply a [`Self::colors_snapshot`] by index (up to the current layer count) after a re-capture,
    /// bumping the version so the cached stamps + preview re-bake with the restored colours.
    pub(super) fn restore_colors(&mut self, on: &[bool], color: &[[f32; 3]]) {
        for i in 0..self.layers.len() {
            if let (Some(&o), Some(c)) = (on.get(i), color.get(i)) {
                self.color_on[i] = o;
                self.color[i] = *c;
            }
        }
        self.version = self.version.wrapping_add(1);
    }

    /// Flatten the stack into one luminance silhouette (alpha-over, bottom→top) for the OFF-mode /
    /// ramp / preview single-image path. `None` if there are no layers (or they disagree on size).
    pub(super) fn flatten(&self) -> Option<(Vec<u8>, u32, u32)> {
        let first = self.layers.first()?;
        let (w, h) = {
            let (_, w, h) = first.parts();
            (w, h)
        };
        let n = (w as usize) * (h as usize);
        let mut acc = vec![0.0f32; n];
        for layer in &self.layers {
            let (lum, lw, lh) = layer.parts();
            if lw != w || lh != h || lum.len() < n {
                return None; // require uniform size (document layers share the canvas size)
            }
            for (a_dst, &px) in acc.iter_mut().zip(lum.iter()) {
                let a = f32::from(px) / 255.0;
                *a_dst = a + *a_dst * (1.0 - a); // over-composite the coverage
            }
        }
        let lum = acc
            .iter()
            .map(|&a| (a * 255.0 + 0.5).clamp(0.0, 255.0) as u8)
            .collect();
        Some((lum, w, h))
    }

    /// Toggle the per-layer-colour mode (the "Per-Layer Color" checkbox). Bumps the version.
    pub(super) fn toggle_per_layer_color(&mut self) {
        self.per_layer_color = !self.per_layer_color;
        self.version = self.version.wrapping_add(1);
    }

    /// Toggle layer `i`'s custom-colour checkbox (off ⇒ it paints the brush base colour). Bumps version.
    pub(super) fn toggle_layer_color(&mut self, i: usize) {
        if let Some(on) = self.color_on.get_mut(i) {
            *on = !*on;
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Set layer `i`'s custom colour (straight RGB, clamped); turns its checkbox on. Bumps version.
    pub(super) fn set_layer_color(&mut self, i: usize, rgb: [f32; 3]) {
        if let (Some(c), Some(on)) = (self.color.get_mut(i), self.color_on.get_mut(i)) {
            *c = [
                rgb[0].clamp(0.0, 1.0),
                rgb[1].clamp(0.0, 1.0),
                rgb[2].clamp(0.0, 1.0),
            ];
            *on = true;
            self.version = self.version.wrapping_add(1);
        }
    }

    /// `(count, color_on, color)` for the panel snapshot — fixed-size arrays so the snapshot stays `Copy`.
    pub(super) fn snapshot(&self) -> (u8, [bool; MAX_SHAPE_LAYERS], [[f32; 3]; MAX_SHAPE_LAYERS]) {
        let mut on = [false; MAX_SHAPE_LAYERS];
        let mut col = [[0.0; 3]; MAX_SHAPE_LAYERS];
        for i in 0..self.layers.len().min(MAX_SHAPE_LAYERS) {
            on[i] = self.color_on.get(i).copied().unwrap_or(false);
            col[i] = self.color.get(i).copied().unwrap_or([0.0; 3]);
        }
        (self.layers.len().min(MAX_SHAPE_LAYERS) as u8, on, col)
    }
}
