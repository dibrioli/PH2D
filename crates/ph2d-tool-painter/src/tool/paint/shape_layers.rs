//! The brush **Shape** as a stack of z-ordered luminance layers, for the per-layer-colour mode (a
//! multi-colour, 3-D-looking tip). Captured from the Painter document's own raster layers (the active
//! sprite's layers, or an imported multi-layer doc). Held here because the heavy pixels can't live in
//! the `Copy` `BrushSpec`. Split from `paint.rs` for the workspace LOC cap.
//!
//! Two modes share the same captured stack:
//! - **Per-Layer Color OFF**: the layers are flattened (alpha-over, bottom→top, scaled by each layer's
//!   `opacity`) into the single [`super::PaintState::shape_image`], so the Shape behaves like a one-layer
//!   image and the Shape colour ramp works as before — the flatten is computed once at capture.
//! - **Per-Layer Color ON**: each layer paints with its OWN captured colours by default (or a custom tint
//!   when the artist checks its box), under its picked blend mode, scaled by its per-layer opacity; the
//!   higher layer paints above the lower one. The ramp section is hidden + nullified.

use super::brush_settings::BrushTextureImage;
use ph2d_painter_brush::texture::{ImageMask, ImageRgb};

/// Max Shape layers the per-layer-colour UI exposes (one row per layer) + the snapshot carries. A deep
/// stack past this is flattened-only (no per-layer colour) — well beyond any hand-authored brush tip.
pub const MAX_SHAPE_LAYERS: usize = 16;

/// The captured Shape layer stack + the per-layer-colour mode state. `layers` are **bottom-to-top**
/// (last = topmost), so the engine composites them in order. `color_on[i]` / `color[i]` align with
/// `layers[i]`. Empty `layers` ⇒ no multi-layer Shape (the single `shape_image` path is used).
///
/// Per-layer colour source: **Texture Color is the DEFAULT** — `color_on[i]` off ⇒ the layer paints its
/// OWN captured per-pixel RGB `rgb[i]` (or, with no captured RGB, the brush base colour). The "Layer N
/// Color" checkbox sets `color_on[i]` ⇒ a **custom tint** `color[i]` instead. The per-layer **blend** is
/// the panel-picked `manual_blend[i]` (the "B" dropdown). **Opacity** `opacity[i]` scales that layer's tip
/// contribution at recomposite AND mirrors the SOURCE document layer's opacity (`doc_ids[i]`): the box is
/// two-way with that layer's Layers-panel slider (guarded to the captured source, never the wrong layer).
#[derive(Default)]
pub(super) struct ShapeLayers {
    layers: Vec<BrushTextureImage>,
    /// The mode toggle (the "Per-Layer Color" checkbox). Only meaningful with ≥ 1 captured layer.
    per_layer_color: bool,
    /// Per-layer "use a custom colour" toggle (the "Layer N Color" checkbox). `false` (default) ⇒ the
    /// layer paints its own captured texture colour (`rgb[i]`), or the brush base colour if none captured.
    color_on: Vec<bool>,
    /// Per-layer custom colour (straight RGB), used when `color_on[i]`.
    color: Vec<[f32; 3]>,
    /// Per-layer captured per-pixel **straight RGB** (`w·h·3`) — the default paint colour when `color_on[i]`
    /// is off. Empty ⇒ no real colour captured for that layer (falls back to the brush base colour).
    rgb: Vec<Vec<u8>>,
    /// Per-layer blend mode ([`ph2d_painter_effects::BlendMode`] discriminant; the "Blend" dropdown).
    /// Mirrors the SOURCE document layer's blend mode (`doc_ids[i]`) — a remote control, two-way with its
    /// Layers-panel blend chip. The bottom layer (index 0, the base) has none (Photoshop Background). `0` = Normal.
    manual_blend: Vec<u8>,
    /// Per-layer **opacity** `0..1` (the numeric box), applied as a scale on that layer's tip contribution
    /// at recomposite. Mirrors the SOURCE document layer's opacity (`doc_ids[i]`): the box edits that
    /// layer's slider (and vice versa, via the auto-recapture which re-reads it here).
    opacity: Vec<f32>,
    /// Per-layer source **document layer id** (`LayerId.0`) — so the opacity box edits that layer's opacity
    /// (two-way with its Layers-panel slider). `0` ⇒ no back-reference (a non-document capture).
    doc_ids: Vec<u64>,
    /// Bumped on any change that re-bakes the coloured stamp (layers / mode / a colour / a toggle).
    version: u64,
}

impl ShapeLayers {
    /// Replace the captured stack with `layers` (`(luminance, w, h)`, bottom-to-top), capped at
    /// [`MAX_SHAPE_LAYERS`]. Resets the per-layer colours to "off" (each layer paints its texture colour
    /// until the artist assigns a custom one). Does NOT change the mode toggle. Bumps the version.
    pub(super) fn set_layers(&mut self, layers: Vec<(Vec<u8>, u32, u32)>) {
        let n = layers.len().min(MAX_SHAPE_LAYERS);
        self.layers = layers
            .into_iter()
            .take(n)
            .map(|(lum, w, h)| BrushTextureImage::new(lum, w, h))
            .collect();
        self.color_on = vec![false; n];
        self.color = vec![[0.0, 0.0, 0.0]; n];
        // The captured `rgb` / `opacity` / `doc_ids` are re-filled by the capture path via
        // `set_layers_meta`; `restore_assignments` re-applies the artist's `color_on` / `color` /
        // `manual_blend` across a same-source re-capture. Opacity is NOT preserved — it mirrors the live
        // source layer, re-read each capture. Reset to neutral defaults here.
        self.rgb = vec![Vec::new(); n];
        self.manual_blend = vec![0; n];
        self.opacity = vec![1.0; n];
        self.doc_ids = vec![0; n];
        self.version = self.version.wrapping_add(1);
    }

    /// Fill the per-layer captured metadata in lock-step with [`Self::set_layers`] (same order / count):
    /// each layer's per-pixel straight `rgb` (`w·h·3`, empty if unavailable), its current document `opacity`
    /// and `blend` (mirrored into the box + dropdown), and its source document `doc_ids` (`LayerId.0`).
    /// Called by the capture path after `set_layers`. Bumps the version (the real-colour bake reads `rgb`).
    pub(super) fn set_layers_meta(
        &mut self,
        rgb: Vec<Vec<u8>>,
        opacity: Vec<f32>,
        blend: Vec<u8>,
        doc_ids: Vec<u64>,
    ) {
        for i in 0..self.layers.len() {
            if let Some(r) = rgb.get(i) {
                self.rgb[i] = r.clone();
            }
            if let Some(&o) = opacity.get(i) {
                self.opacity[i] = o.clamp(0.0, 1.0);
            }
            if let Some(&b) = blend.get(i) {
                self.manual_blend[i] = b;
            }
            if let Some(&d) = doc_ids.get(i) {
                self.doc_ids[i] = d;
            }
        }
        self.version = self.version.wrapping_add(1);
    }

    /// Drop the stack (revert to the single-image / falloff Shape).
    pub(super) fn clear(&mut self) {
        self.layers.clear();
        self.color_on.clear();
        self.color.clear();
        self.rgb.clear();
        self.manual_blend.clear();
        self.opacity.clear();
        self.doc_ids.clear();
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

    /// Snapshot the per-layer **artist assignments** `(color_on, color)` — the only state the artist owns —
    /// for re-applying across an automatic re-capture of the same Shape source (so editing the source
    /// sprite keeps the custom colours). The captured `rgb` / `opacity` / `blend` / `doc_ids` are NOT
    /// snapshotted: opacity + blend MIRROR the live source layer, so they are re-read fresh each capture.
    pub(super) fn assignments_snapshot(&self) -> (Vec<bool>, Vec<[f32; 3]>) {
        (self.color_on.clone(), self.color.clone())
    }

    /// Re-apply an [`Self::assignments_snapshot`] by index (up to the current layer count) after a
    /// re-capture, bumping the version so the cached stamps + preview re-bake with the restored state.
    pub(super) fn restore_assignments(&mut self, on: &[bool], color: &[[f32; 3]]) {
        for i in 0..self.layers.len() {
            if let (Some(&o), Some(c)) = (on.get(i), color.get(i)) {
                self.color_on[i] = o;
                self.color[i] = *c;
            }
        }
        self.version = self.version.wrapping_add(1);
    }

    /// Flatten the stack into one luminance silhouette (alpha-over, bottom→top) for the OFF-mode / ramp /
    /// preview single-image path — each layer's coverage scaled by its `opacity[i]` (so a half-opacity
    /// layer shows the ones below through it). `None` if there are no layers (or they disagree on size).
    pub(super) fn flatten(&self) -> Option<(Vec<u8>, u32, u32)> {
        let first = self.layers.first()?;
        let (w, h) = {
            let (_, w, h) = first.parts();
            (w, h)
        };
        let n = (w as usize) * (h as usize);
        let mut acc = vec![0.0f32; n];
        for (li, layer) in self.layers.iter().enumerate() {
            let (lum, lw, lh) = layer.parts();
            if lw != w || lh != h || lum.len() < n {
                return None; // require uniform size (document layers share the canvas size)
            }
            let op = self.opacity.get(li).copied().unwrap_or(1.0);
            for (a_dst, &px) in acc.iter_mut().zip(lum.iter()) {
                let a = f32::from(px) / 255.0 * op;
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

    /// Set layer `i`'s blend mode ([`ph2d_painter_effects::BlendMode`] discriminant). Bumps version. The
    /// "B" dropdown.
    pub(super) fn set_manual_blend(&mut self, i: usize, mode: u8) {
        if let Some(b) = self.manual_blend.get_mut(i) {
            *b = mode;
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Set layer `i`'s **opacity** (`0..1`, the numeric box) — the brush-side scale on its tip
    /// contribution. The caller (`set_brush_shape_layer_opacity`) also writes the source document layer's
    /// opacity (so the box is two-way with its Layers-panel slider). Bumps version (re-bakes the preview).
    pub(super) fn set_opacity(&mut self, i: usize, v: f32) {
        if let Some(o) = self.opacity.get_mut(i) {
            *o = v.clamp(0.0, 1.0);
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Layer `i`'s opacity (`0..1`), `1.0` by default.
    pub(super) fn opacity(&self, i: usize) -> f32 {
        self.opacity.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0)
    }

    /// Layer `i`'s source document layer id (`LayerId.0`), or `None` for a non-document capture.
    pub(super) fn doc_id(&self, i: usize) -> Option<u64> {
        self.doc_ids.get(i).copied().filter(|&d| d != 0)
    }

    /// Per-layer opacities (bottom-to-top), for the recomposite's per-layer scale.
    pub(super) fn opacities(&self) -> Vec<f32> {
        (0..self.layers.len()).map(|i| self.opacity(i)).collect()
    }

    /// Any layer paints its captured **texture colour** (`color_on[i]` off AND it has captured RGB) — this
    /// is the default mode and drives the route to the per-pixel dynamic path (which samples that RGB).
    pub(super) fn any_texture_color(&self) -> bool {
        (0..self.layers.len()).any(|i| self.rgb_image(i).is_some())
    }

    /// Layer `i`'s blend mode ([`ph2d_painter_effects::BlendMode`] discriminant; always the manual pick).
    pub(super) fn effective_blend(&self, i: usize) -> u8 {
        self.manual_blend.get(i).copied().unwrap_or(0)
    }

    /// Borrow layer `i`'s captured RGB as an engine [`ImageRgb`] for the default **texture-colour** path —
    /// `Some` only when the layer is NOT custom-coloured (`color_on` off) AND has captured RGB; `None`
    /// otherwise (custom tint, or no RGB captured ⇒ the flat brush-base fallback).
    pub(super) fn rgb_image(&self, i: usize) -> Option<ImageRgb<'_>> {
        if self.color_on.get(i).copied().unwrap_or(false) {
            return None;
        }
        let rgb = self.rgb.get(i)?;
        if rgb.is_empty() {
            return None;
        }
        let (_, w, h) = self.layers.get(i)?.parts();
        (rgb.len() >= (w as usize) * (h as usize) * 3).then_some(ImageRgb {
            rgb,
            width: w,
            height: h,
        })
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

    /// `(manual_blend, opacity)` fixed-size arrays for the panel snapshot (the "B" blend dropdown state +
    /// the opacity box per row). Entries `0..count` are valid.
    pub(super) fn panel_blend_arrays(&self) -> ([u8; MAX_SHAPE_LAYERS], [f32; MAX_SHAPE_LAYERS]) {
        let mut blend = [0u8; MAX_SHAPE_LAYERS];
        let mut opacity = [1.0f32; MAX_SHAPE_LAYERS];
        for i in 0..self.layers.len().min(MAX_SHAPE_LAYERS) {
            blend[i] = self.manual_blend.get(i).copied().unwrap_or(0);
            opacity[i] = self.opacity(i);
        }
        (blend, opacity)
    }
}
