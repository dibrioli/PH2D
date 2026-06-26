//! The stamp **route dispatcher**: given a batch of dabs, pick which of the four stamp paths
//! ([`super::stamp_cache`]) to use based on the Shape + Grain slots (cacheable / canvas-cached /
//! per-pixel / ramped). Split from `paint.rs` for the workspace LOC cap; the routes themselves live in
//! `stamp_cache`.

use super::ramp_lut::RampLutOwner;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

impl PainterTool {
    /// Stamp a batch of dabs into `canvas_rgba` (with the brush Shape + Grain, if any) + accumulate the
    /// dirty rect. With **Tiling** on, each dab is first replicated across the wrapped sprite edges
    /// (`tiling::tiled_dabs`) so a stroke near a border is seamless when the sprite repeats as a tile.
    pub(super) fn stamp_dabs(&mut self, dabs: &[Dab]) {
        if self.paint.tiling[0] || self.paint.tiling[1] {
            let wrapped = super::tiling::tiled_dabs(dabs, self.source_size, self.paint.tiling);
            self.stamp_dabs_inner(&wrapped);
        } else {
            self.stamp_dabs_inner(dabs);
        }
    }

    /// The actual stamp dispatch (already tiled if needed); [`Self::stamp_dabs`] wraps first.
    pub(super) fn stamp_dabs_inner(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let (w, h) = self.source_size;
        let mut brush = self.paint.brush;
        // Eraser overrides the blend with Erase Alpha (the drawing blend in `brush.blend` is kept).
        if self.paint.eraser {
            brush.blend = ph2d_painter_brush::BrushBlend::EraseAlpha;
        }
        // Alpha lock: the dab paints only into the active layer's existing alpha (clip/mask composite-time).
        let alpha_locked = self
            .layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked);
        let has_shape_image = self.paint.shape_image.is_some();
        // Per-layer-colour Shape (multi-layer, mode on): bake the z-ordered tinted layers + recomposite.
        // Guarded by an ACTIVE Image silhouette so a stale `per_layer_color` flag (e.g. after the Shape
        // was reset to None) can never route a non-Image Shape into the coloured path (Enio 2026-06-26).
        if self.paint.shape_layers.is_color_mode() && brush.shape_silhouette_active(has_shape_image)
        {
            self.stamp_dabs_cached_color(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Grain Jitter-Rotate OR a per-dab Shape rotation (Rake / Random) → each dab needs its own
        // basis, so the constant-orientation caches are skipped (the per-pixel path resolves per dab).
        let per_dab_rotation =
            brush.has_per_dab_rotation() || brush.shape_has_per_dab_rotation(has_shape_image);
        // Accumulate OFF caps the stroke at Strength (per-pixel mask); skip the caches when Strength < 1.
        let accumulate_cap = !brush.accumulate && brush.strength < 1.0;
        // A Colour Ramp owns the painted COLOUR (baked LUT): the **Shape** ramp (its B&W filter off →
        // colourise the silhouette) or the **Grain** ramp (indexed by the Grain pattern). With NO Grain
        // the COVERAGE (the Shape silhouette, OR the bare falloff when there's no Shape image) indexes the
        // ramp, and the StampMask already caches that coverage → blit the cached mask applying
        // `ramp[coverage]` (as cheap as a plain cached stamp, NOT a per-pixel coverage recompute per dab —
        // critical for the Line/Curve/Circle/Polygon fills, which re-stamp the WHOLE shape every move).
        // Per-pixel otherwise: a Grain to index, per-dab rotation, or the Accumulate cap (Enio 2026-06-26).
        let grain_active = brush.texture.is_active();
        let owner = self.color_ramp_owner(grain_active);
        if owner != RampLutOwner::None {
            // `dab_mask_cacheable` already covers the no-Shape falloff case (a static View silhouette);
            // requiring `shape_silhouette_active` here needlessly forced a colour ramp on a plain brush
            // onto the per-pixel path — the cause of the slow ramped fills (Enio 2026-06-26).
            let static_ok =
                brush.dab_mask_cacheable(has_shape_image) && !per_dab_rotation && !accumulate_cap;
            // TextureAlpha mode punches the sprite alpha (a different blend) → keep it per-pixel; the
            // coloured stamp bakes coverage straight, so it serves None / Strength.
            let bakeable_alpha = !matches!(
                self.active_ramp_alpha_mode(owner),
                ph2d_painter_brush::RampAlphaMode::TextureAlpha
            );
            if static_ok && !grain_active {
                // No Grain: the silhouette/falloff coverage indexes the ramp → the cached grayscale
                // StampMask + `ramp[coverage]` at blit (cheap as a plain cached stamp).
                self.stamp_dabs_cached_ramped(dabs, &brush, alpha_locked, w, h);
            } else if static_ok && grain_active && bakeable_alpha {
                // Grain + ramp: the Grain VALUE indexes the ramp → bake the grain×ramp colour ONCE into
                // a coloured stamp + scale-blit (vs the per-pixel Grain+ramp recompute — the slow fills).
                self.stamp_dabs_cached_ramp_color(dabs, &brush, alpha_locked, w, h);
            } else {
                self.stamp_dabs_ramped(dabs, &brush, alpha_locked, w, h);
            }
            return;
        }
        // Whether either slot genuinely shapes the dab beyond the falloff: a Shape image (silhouette) or
        // an active Grain. A bare falloff brush (neither) stays on the per-pixel path, as before.
        let want_cache =
            brush.shape_silhouette_active(has_shape_image) || brush.texture.is_active();
        // Both slots static & View ⇒ bake silhouette × Grain once into the scale-invariant stamp (Blender).
        if want_cache
            && brush.dab_mask_cacheable(has_shape_image)
            && !per_dab_rotation
            && !accumulate_cap
        {
            self.stamp_dabs_cached(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Grain Tiled / Stencil is canvas-fixed but dab-independent — cache each canvas pixel's Grain
        // once per stroke + look it up per dab; the silhouette (Shape or falloff) stays per-pixel.
        if brush.texture.is_canvas_cacheable() && !per_dab_rotation && !accumulate_cap {
            self.stamp_dabs_canvas_cached(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Per-pixel path (no cache applies, or the Accumulate cap): resolves the texture frame +
        // Randomize-Color colour per dab and threads the per-stroke coverage mask.
        self.stamp_dabs_per_pixel(dabs, &brush, alpha_locked, w, h);
    }
}
