//! The cached **multi-layer coloured stamp** on the tool side (per-layer-colour mode): bake the
//! z-ordered layer composite — each layer tinted by its resolved colour — once into a
//! [`ph2d_painter_brush::ColorStampMask`], then scale-blit it per dab. Same performance profile as the
//! grayscale cached stamp ([`super::stamp_cache`]): the N-layer compositing runs only on an appearance
//! change, never per pixel per dab. Split from `stamp_cache` for the workspace LOC cap.
//!
//! The baked stamp folds in the Shape **Angle** + dab flatten (its View basis), like the grayscale
//! stamp; per-dab Shape Rake/Random rotation of the coloured stamp is a follow-up (it would re-bake or
//! rotate-at-blit). The Shape colour ramp is intentionally NOT consulted here — per-layer-colour mode
//! supersedes it (the panel hides the ramp section while it is on).

use super::stamp_cache::mask_size_for;
use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, TextureSettings, blit_color_stamp, render_color_stamp_mask,
};
use std::sync::Arc;

/// Identifies the appearance the cached [`ph2d_painter_brush::ColorStampMask`] was baked for. The bake
/// depends on the Shape frame (Angle / Size / Offset), the captured layers + their colours
/// (`layers_version` bumps on any of those), the brush base colour (an un-coloured layer paints it), the
/// Grain (image + depth), the dab flatten/rotate, and the mask resolution — but NOT the radius (scaled).
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ColorStampKey {
    shape: TextureSettings,
    layers_version: u64,
    brush_color: [f32; 3],
    texture: TextureSettings,
    grain_image_version: u64,
    grain_depth: f32,
    dab_flatten: f32,
    dab_angle_deg: u16,
    size: u32,
}

impl PainterTool {
    /// Scale-blit the cached multi-layer coloured stamp for each dab — the per-layer-colour path of
    /// [`Self::stamp_dabs`]. The per-texel colour comes from the baked stamp; `spec.color` is unused.
    pub(super) fn stamp_dabs_cached_color(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_color_stamp_cache(brush, mask_size_for(max_r));
        let Some((stamp, _)) = self.paint.color_stamp_cache.as_ref() else {
            return;
        };
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            if let Some(r) = blit_color_stamp(
                buf,
                w,
                h,
                d.center,
                d.radius_px,
                stamp,
                &spec,
                d.coverage,
                alpha_locked,
            ) {
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

    /// Re-bake the coloured stamp when the appearance / mask size changed; a no-op on a hit.
    fn ensure_color_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
        let key = ColorStampKey {
            shape: brush.shape,
            layers_version: self.paint.shape_layers.version(),
            brush_color: brush.color,
            texture: brush.texture,
            grain_image_version: self.paint.texture_image_version,
            grain_depth: brush.grain_depth,
            dab_flatten: brush.dab_flatten,
            dab_angle_deg: brush.dab_angle_deg,
            size,
        };
        if self.paint.color_stamp_cache.as_ref().map(|(_, k)| *k) == Some(key) {
            return;
        }
        let stamp = {
            let masks = self.paint.shape_layers.masks();
            let colors = self.paint.shape_layers.resolved_colors(brush.color);
            let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            render_color_stamp_mask(brush, &masks, &colors, grain_image.as_ref(), size)
        };
        self.paint.color_stamp_cache = Some((stamp, key));
    }
}
