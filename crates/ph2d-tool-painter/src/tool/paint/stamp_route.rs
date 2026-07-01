//! The stamp **route dispatcher**: given a batch of dabs, pick which of the four stamp paths
//! ([`super::stamp_cache`]) to use based on the Shape + Grain slots (cacheable / canvas-cached /
//! per-pixel / ramped). Split from `paint.rs` for the workspace LOC cap; the routes themselves live in
//! `stamp_cache`.

use super::ramp_lut::RampLutOwner;
use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushSpec, Dab};
use std::sync::Arc;

impl PainterTool {
    /// Whether the active stroke method lets the shell coalesce a burst of raw pointer Moves into ONE
    /// delivery per frame (the restore + whole-shape re-stamp fill methods only show the latest position,
    /// so it is byte-identical). Delegates to [`ph2d_painter_brush::StrokeMethod::coalesces_canvas_motion`].
    /// The FPS-drop / "Raw rises" fix (`HANDOFF_per_layer_color_perf_artifacts` §1.R).
    #[must_use]
    pub fn coalesces_canvas_motion(&self) -> bool {
        self.paint.brush.stroke_method.coalesces_canvas_motion()
    }

    /// Stamp a batch of dabs into `canvas_rgba` (with the brush Shape + Grain, if any) + accumulate the
    /// dirty rect. With **Tiling** on, each dab is first replicated across the wrapped sprite edges
    /// (`tiling::tiled_dabs`) so a stroke near a border is seamless when the sprite repeats as a tile.
    pub(super) fn stamp_dabs(&mut self, dabs: &[Dab]) {
        // Smear drags canvas content between consecutive dab centres — it ignores the brush colour /
        // blend / ramp routing, so short-circuit before all of it. It needs the UNtiled dab chain (a
        // single `last_smear_pos` source) + applies Shape/Grain/flatten via the mask and Tiling via
        // per-endpoint offsets itself, so it runs here, not in `stamp_dabs_inner`.
        if matches!(self.paint.paint_mode, PaintMode::Smear) {
            let (w, h) = self.source_size;
            self.stamp_dabs_smear(dabs, w, h);
            return;
        }
        if self.paint.tiling[0] || self.paint.tiling[1] {
            let wrapped = super::tiling::tiled_dabs(dabs, self.source_size, self.paint.tiling);
            self.stamp_dabs_inner(&wrapped);
        } else {
            self.stamp_dabs_inner(dabs);
        }
    }

    /// **Smear** route: drag the canvas content from each dab centre to the next (Blender 2D
    /// `paint_2d_lift_smear` + INTERPOLATE ≡ Krita Color-Smudge "Smearing"). Consecutive dab centres
    /// give the lift→stamp displacement; [`PaintState::last_smear_pos`](super::PaintState) chains the
    /// source across pointer batches within one stroke. Smear amount = brush Strength × per-dab
    /// coverage (pressure).
    ///
    /// The per-pixel weight is the brush's full dab mask — **Shape** silhouette × **Grain** ×
    /// **flatten/rotate** — via the shared cached [`ph2d_painter_brush::StampMask`] when any of those
    /// shape the dab (and it is cacheable); else the plain round falloff. **Tiling** wraps each smear
    /// across the enabled sprite edges (the same offset applied to lift + stamp, so a wrapped copy
    /// keeps its drag). The engine snapshots each source region, so overlapping read/write never feeds
    /// back.
    pub(super) fn stamp_dabs_smear(&mut self, dabs: &[Dab], w: u32, h: u32) {
        if dabs.is_empty() {
            return;
        }
        let base = self.paint.brush;
        let strength = base.strength.clamp(0.0, 1.0);
        let has_shape_image = self.paint.shape_image.is_some();
        // Use the full dab mask when a Shape / Grain / flatten shapes the dab AND it's cacheable
        // (static View); else the plain round falloff. flatten/rotate is always baked into the mask,
        // so a flattened plain brush routes through the mask too.
        let want_mask = (base.shape_silhouette_active(has_shape_image)
            || base.texture.is_active()
            || base.dab_flatten > 0.0)
            && base.dab_mask_cacheable(has_shape_image);
        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let source_size = self.source_size;

        // Phase 1 (reads self): build the smear ops (from, to, radius, coverage), applying Tiling wrap
        // offsets to BOTH endpoints, and advance the source chain.
        let mut from = self.paint.last_smear_pos;
        let mut ops: Vec<([f32; 2], [f32; 2], f32, f32)> = Vec::with_capacity(dabs.len());
        for d in dabs {
            if let Some(prev) = from {
                if tiled {
                    for off in
                        super::tiling::tiled_offsets(d.center, d.radius_px, source_size, tiling)
                    {
                        ops.push((
                            [prev[0] + off[0], prev[1] + off[1]],
                            [d.center[0] + off[0], d.center[1] + off[1]],
                            d.radius_px,
                            d.coverage,
                        ));
                    }
                } else {
                    ops.push((prev, d.center, d.radius_px, d.coverage));
                }
            }
            from = Some(d.center);
        }
        self.paint.last_smear_pos = from;
        if ops.is_empty() {
            return;
        }

        // Phase 2: build the mask (if wanted), then apply each op to the canvas.
        if want_mask {
            let max_r = ops.iter().map(|o| o.2).fold(0.0_f32, f32::max);
            self.ensure_stamp_cache(&base, super::stamp_cache::mask_size_for(max_r));
        }
        let mask = if want_mask {
            self.paint.stamp_cache.as_ref().map(|(m, _)| m)
        } else {
            None
        };
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for (f, t, radius, coverage) in ops {
            let dirty = match mask {
                Some(m) => ph2d_painter_brush::smear_blit_stamp(
                    buf,
                    w,
                    h,
                    f,
                    t,
                    radius,
                    m,
                    strength * coverage,
                ),
                None => ph2d_painter_brush::smear_dab(
                    buf,
                    w,
                    h,
                    f,
                    t,
                    &BrushSpec {
                        radius_px: radius,
                        ..base
                    },
                    strength * coverage,
                ),
            };
            if let Some(r) = dirty {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        if let Some(rect) = touched {
            self.mark_dirty(rect);
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
        // Per-layer-colour Shape (multi-layer, mode on): the z-ordered tinted layers recomposite onto the
        // canvas. Guarded by an ACTIVE Image silhouette so a stale `per_layer_color` flag (e.g. after the
        // Shape was reset to None) can never route a non-Image Shape into the coloured path (Enio).
        if self.paint.shape_layers.is_color_mode() && brush.shape_silhouette_active(has_shape_image)
        {
            // Per-dab dynamics the constant-orientation cached path can't express → the per-pixel dynamic
            // path: Shape Rake / Random rotation, Randomize Color, or Grain Jitter Rotate. Randomize Color
            // is `has_colour_jitter_amount()` (Hue/Sat/Val > 0) — the engine's actual gate; the legacy
            // `color_jitter_enabled` flag is dead (the panel drives it by amount), so checking it here
            // meant Randomize Color alone never reached the dynamic path (it only worked WITH Rake/Random).
            // A canvas-fixed Grain (Tiled / Stencil) cannot be baked into the dab-LOCAL coloured stamp
            // (`render_color_stamp_mask` samples in stamp-local coords) — doing so ignored the Stencil
            // rect entirely, so the colour leaked OUTSIDE it (worst with a big Anchored dab). The per-pixel
            // dynamic path samples the Grain at each canvas pixel through its canvas-fixed basis, so it
            // masks the rect correctly (Enio 2026-06-27).
            if brush.shape_has_per_dab_rotation(has_shape_image)
                || brush.grain_has_per_dab_rotation()
                || brush.has_colour_jitter_amount()
                || brush.has_per_dab_rotation()
                || (brush.texture.is_active() && brush.texture.mapping.is_canvas_fixed())
                // Texture colour (the default) samples each layer's per-pixel RGB — only the per-pixel
                // dynamic path can (the cached stamp carries one flat colour per layer).
                || self.paint.shape_layers.any_texture_color()
            {
                self.stamp_dabs_per_layer_dynamic(dabs, &brush, alpha_locked, w, h);
            } else {
                self.stamp_dabs_cached_color(dabs, &brush, alpha_locked, w, h);
            }
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
