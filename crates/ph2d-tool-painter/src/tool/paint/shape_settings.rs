//! Imported-image slots (brush **Grain** image + **Shape** image) + the **Shape** geometry and
//! **Grain Depth** setters — the single UI-edit clamp source for those, split from `brush_settings`
//! for the workspace LOC cap. The Shape is the dab silhouette (its image *replaces* the falloff); the
//! Grain image is the texture inside it.

use super::brush_settings::BrushTextureImage;
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
    TextureMapping,
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
        // Fit the image's aspect ratio so it's never squashed — the Stencil rect takes the aspect; the
        // other mappings put it in the Grain Size (Enio 2026-06-28). Reset on each new image.
        self.fit_grain_image_aspect(self.paint.brush.texture.mapping);
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
        // Per-layer captured metadata, aligned 1:1 with `layers`: the straight RGB (the default texture
        // colour), the document layer's current opacity (mirrored into the box), and its document layer id
        // (so the opacity box edits that layer's slider — two-way).
        let mut rgb_layers: Vec<Vec<u8>> = Vec::new();
        let mut opacity: Vec<f32> = Vec::new();
        let mut blend: Vec<u8> = Vec::new();
        let mut doc_ids: Vec<u64> = Vec::new();
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
            // The silhouette = the layer's RAW alpha channel (no opacity folded in — the per-layer opacity
            // is applied at recomposite from `opacity[i]`, which mirrors the source layer's opacity).
            let lum: Vec<u8> = (0..n).map(|i| rgba[i * 4 + 3]).collect();
            // The straight per-pixel RGB — the default paint colour for the layer (Texture Color).
            let mut rgb = Vec::with_capacity(n * 3);
            for i in 0..n {
                rgb.push(rgba[i * 4]);
                rgb.push(rgba[i * 4 + 1]);
                rgb.push(rgba[i * 4 + 2]);
            }
            layers.push((lum, w, h));
            rgb_layers.push(rgb);
            opacity.push(layer.opacity.clamp(0.0, 1.0));
            blend.push(layer.blend_mode.to_u8());
            doc_ids.push(id.0);
        }
        if !layers.is_empty() {
            // Re-capturing the SAME source sprite keeps the per-layer artist assignments (colours + the
            // manual blend picks — editing the sprite shouldn't wipe them); a fresh capture from a different
            // sprite starts neutral. `set_brush_shape_layers` resets them, so snapshot/restore around it.
            // `rgb` / `opacity` / `doc_ids` are always re-read fresh (opacity mirrors the live source layer).
            let same_source = self.bound_doc.is_some() && self.bound_doc == self.shape_source_doc;
            let assignments = same_source.then(|| self.paint.shape_layers.assignments_snapshot());
            self.set_brush_shape_layers(layers);
            self.paint
                .shape_layers
                .set_layers_meta(rgb_layers, opacity, blend, doc_ids);
            // The real opacities only land HERE — `set_layers` above reset them to 1.0 and the flatten
            // inside `set_brush_shape_layers` baked that. Re-bake now that the metadata is in.
            self.reflatten_shape_image();
            if let Some((on, color)) = assignments {
                self.paint.shape_layers.restore_assignments(&on, &color);
            }
            self.shape_source_doc = self.bound_doc;
            self.shape_source_revision = self.shape_source_content_revision();
        }
    }

    /// The Shape source document's content revision — `layers_revision` (opacity / visibility / blend /
    /// structure / undo) folded with `pixel_clock` (paint / undo). Any edit to the active source bumps one
    /// of them, so [`Self::refresh_shape_source_if_changed`] re-captures when it differs from capture time.
    fn shape_source_content_revision(&self) -> u64 {
        self.layers_revision
            .wrapping_mul(0x100_0000_01b3)
            .wrapping_add(self.pixel_clock)
    }

    /// Auto-refresh the multi-layer Shape when its source sprite changed (the bridge calls this each
    /// frame). Re-captures — preserving the per-layer colours — when the active doc IS the Shape source
    /// and its content revision moved since the last capture, so editing the reference sprite (paint,
    /// **opacity**, visibility, **undo/redo**, …) updates the brush Shape without a manual re-capture.
    /// Skipped mid-stroke (waits for pointer-up) so a drag re-captures once, not per dab.
    pub fn refresh_shape_source_if_changed(&mut self) {
        // Run while the captured Shape source IS the current document (equality holds even when both are
        // the unbound `None` doc — so the box ⇄ Layers-panel sync still works without a document binding).
        if self.shape_source_doc != self.bound_doc
            || self.paint.shape_layers.is_empty()
            || self.paint.stroke.is_some()
        {
            return;
        }
        if self.shape_source_content_revision() != self.shape_source_revision {
            self.capture_layers_as_brush_shape(); // also refreshes `shape_source_revision`
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
        self.reflatten_shape_image();
    }

    /// Re-bake the flattened Shape silhouette from the current layers **and their metadata**, bumping the
    /// image version so every cache keyed on it re-bakes.
    ///
    /// **Why it is its own function (sweep 2026-07-12):** `flatten()` scales each layer by
    /// `opacity[i]` — but `set_layers()` RESETS the opacities to 1.0, and the capture only installs the
    /// real ones afterwards (`set_layers_meta`). So the flatten always ran against all-1.0, and
    /// `set_opacity` never re-flattened at all: the per-layer **Opacity** box was dead in every mode
    /// except Per-Layer Color (which applies opacity at recomposite time, bypassing the flatten). One
    /// choke point, called by everything that changes what the silhouette should look like.
    pub(super) fn reflatten_shape_image(&mut self) {
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
        // Flipping the mode MID-STROKE (the panel and the canvas are both live — that is how Enio's Rake
        // panic happened) invalidates the accumulation, and silently: `pre` is the PRE-stroke canvas, and
        // any dab painted while the mode was OFF went straight to `canvas_rgba` — it is in neither `pre`
        // nor the coverage maps. The next batch's recomposite rebuilds the region from `pre` and those dabs
        // EVAPORATE. Dropping the accumulation makes the next batch re-snapshot the CURRENT canvas (which
        // does contain them) as its base, so the stroke continues from what is actually on screen.
        // Sweep 2026-07-12; the `fits()` guard validates the maps' SHAPE, not the stroke's CONTINUITY.
        self.paint.per_layer_stroke.reset();
    }

    /// Toggle Shape layer `i`'s custom-colour checkbox (off ⇒ it paints the brush base colour).
    pub fn toggle_brush_shape_layer_color(&mut self, i: usize) {
        self.paint.shape_layers.toggle_layer_color(i);
    }

    /// Set Shape layer `i`'s custom colour (straight RGB); turns its checkbox on.
    pub fn set_brush_shape_layer_color(&mut self, i: usize, rgb: [f32; 3]) {
        self.paint.shape_layers.set_layer_color(i, rgb);
    }

    /// Set Shape layer `i`'s blend mode (the "Blend" dropdown). Updates the brush side immediately AND —
    /// when the captured Shape source IS the document being painted — edits that SOURCE layer's blend mode
    /// (so the dropdown is two-way with its Layers-panel blend chip, like the opacity box). Guarded to the
    /// captured source so it never edits a coincidentally-id-matching layer in a different document.
    pub fn set_brush_shape_layer_blend(&mut self, i: usize, mode: u8) {
        self.paint.shape_layers.set_manual_blend(i, mode);
        // Remote-control the SHAPE SOURCE layer's blend mode wherever it lives (see the opacity setter).
        if let Some(doc) = self.paint.shape_layers.doc_id(i) {
            let src = self.shape_source_doc;
            self.set_shape_source_layer_blend(
                src,
                crate::layers::LayerId(doc),
                ph2d_painter_effects::BlendMode::from_u8(mode),
            );
        }
    }

    /// Set Shape layer `i`'s **opacity** (`0..1`). Updates the brush side immediately (the tip scale) AND
    /// — when the captured Shape source IS the document being painted — edits that SOURCE layer's opacity,
    /// so the box is two-way with its Layers-panel slider. The guard `shape_source_doc == bound_doc` is the
    /// bug fix: it never edits a coincidentally-id-matching layer in a DIFFERENT document being painted.
    pub fn set_brush_shape_layer_opacity(&mut self, i: usize, op01: f32) {
        self.paint.shape_layers.set_opacity(i, op01);
        self.reflatten_shape_image(); // the silhouette is scaled by opacity — re-bake it
        // Remote-control the SHAPE SOURCE layer's opacity wherever it lives (the live bound doc, or — when
        // the artist switched away to paint a different sprite with this shape — its stashed stack). Never
        // the painted document's layer.
        if let Some(doc) = self.paint.shape_layers.doc_id(i) {
            let src = self.shape_source_doc;
            self.set_shape_source_layer_opacity(
                src,
                crate::layers::LayerId(doc),
                op01.clamp(0.0, 1.0),
            );
        }
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

    /// Set the Shape **Follow** mode (how the silhouette relates to the stroke): `0` = Off (fixed Angle),
    /// `1` = Rake (rotate each stamp to the tangent), `2` = Flow (lay the pattern in the stroke's
    /// arc-length frame — parallel lines through curves). The two are mutually exclusive and back the same
    /// dropdown, so setting one clears the other from a SINGLE door (no divergent `rake`/`flow` state).
    pub fn set_brush_shape_follow(&mut self, mode: u8) {
        let shape = &mut self.paint.brush.shape;
        shape.rake = mode == 1;
        shape.flow = mode == 2;
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

    /// Re-fit a loaded Grain **Image** so its aspect ratio is respected for `mapping` (never squashed):
    /// **Stencil** shapes the rect to the image (Size stays `1:1`, the image fills it once); the other
    /// mappings put the aspect in the Grain **Size** (`sx : sy = h : w`, so a unit dab step maps to a
    /// square image step — the tile reads at the image's proportions). No-op without an image. The shell
    /// re-calls this on a new image and on a mapping change (Enio 2026-06-28).
    pub(super) fn fit_grain_image_aspect(&mut self, mapping: TextureMapping) {
        let Some((w, h)) = self.paint.texture_image.as_ref().map(|i| {
            let (_, w, h) = i.parts();
            (w, h)
        }) else {
            return;
        };
        if mapping.is_stencil() {
            let (cw, ch) = self.source_size;
            self.paint.brush.texture.size = [1.0, 1.0];
            self.paint.brush.texture.stencil_size =
                stencil_size_for_image(w, h, [cw as f32, ch as f32]);
        } else {
            self.paint.brush.texture.size = grain_size_for_image(w, h);
        }
    }
}

/// The Grain **Size** (`sx : sy = h : w`, larger axis at `1.0`) that keeps an Image's pixels square in
/// View / Tiled — a unit dab step maps to `[sx·w, sy·h]` in image px, so `sx·w = sy·h` ⇒ no squash.
/// A 2:1 image → `[0.5, 1.0]`. Falls back to `[1, 1]` for a degenerate image.
fn grain_size_for_image(img_w: u32, img_h: u32) -> [f32; 2] {
    if img_w == 0 || img_h == 0 {
        return [1.0, 1.0];
    }
    let a = img_w as f32 / img_h as f32; // image aspect
    if a >= 1.0 { [1.0 / a, 1.0] } else { [1.0, a] }
}

/// The Stencil rect Size (canvas fractions per axis) whose on-canvas rect carries the IMAGE's aspect
/// ratio, fit into a half-canvas box (the larger axis at `0.5`). Because the rect's pixel size is
/// `size · canvas` per axis, `rect_w / rect_h = img_w / img_h` ⇒ `sx / sy = (img_w/img_h)·(ch/cw)`.
/// Falls back to a centred square for a degenerate image / canvas.
fn stencil_size_for_image(img_w: u32, img_h: u32, canvas: [f32; 2]) -> [f32; 2] {
    const BASE: f32 = 0.5;
    if img_w == 0 || img_h == 0 || canvas[0] <= 0.0 || canvas[1] <= 0.0 {
        return [BASE, BASE];
    }
    let r = (img_w as f32 / img_h as f32) * (canvas[1] / canvas[0]);
    if r >= 1.0 {
        [BASE, BASE / r]
    } else {
        [BASE * r, BASE]
    }
}
