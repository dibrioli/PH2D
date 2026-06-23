//! The Blender-style cached brush stamp on the tool side: render the falloff × View-texture mask
//! once (per appearance / size), then scale-blit it per dab. This turns a large textured re-stamp
//! (Anchored re-drawn every pointer move) from a per-pixel falloff+texture recompute into a cheap
//! bilinear blit — the texture is sampled once per stroke, not per pixel per dab. Eligibility is
//! [`ph2d_painter_brush::TextureSettings::is_cacheable`] (checked by the caller in `paint.rs`).

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, Falloff, FalloffCurve, TextureSettings, blit_canvas_cached, blit_stamp,
    render_stamp_mask,
};
use std::sync::Arc;

/// Lazily-filled CANVAS-SPACE texture cache for the **Tiled / Stencil** mappings (canvas-fixed, so a
/// pixel's texture value is dab-independent): each canvas pixel's texture is computed once per stroke
/// and reused by every later dab — see [`ph2d_painter_brush::blit_canvas_cached`]. `tex` + `ready`
/// are `width*height`; persisted across the stroke (and strokes) until the key / size changes.
pub(super) struct CanvasTexCache {
    tex: Vec<u8>,
    ready: Vec<u8>,
    width: u32,
    height: u32,
    key: CanvasKey,
}

/// The cache holds the **texture** only (falloff is applied at blit), so it invalidates on the
/// texture settings or the imported image — NOT on falloff / hardness / colour / blend / radius.
#[derive(Clone, Copy, PartialEq)]
struct CanvasKey {
    texture: TextureSettings,
    image_version: u64,
}

/// Identifies the appearance the cached [`ph2d_painter_brush::StampMask`] was rendered for. The mask depends on the
/// falloff shape, the (View) texture, the imported image, and the mask resolution — but NOT on the
/// radius (we scale), colour or blend (applied at blit time), so an Anchored size-drag mostly reuses
/// one mask.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct StampKey {
    falloff: Falloff,
    hardness: f32,
    custom: FalloffCurve,
    texture: TextureSettings,
    image_version: u64,
    size: u32,
}

/// Mask resolution for a dab of `radius` px: enough texels to be 1:1 (`2·radius`), rounded up to a
/// power of two so a growing Anchored dab only re-renders a few times, clamped. Beyond the max the
/// texture is bilinearly upscaled (softer) — Blender caps its brush image likewise.
fn mask_size_for(radius: f32) -> u32 {
    const MIN: u32 = 32;
    const MAX: u32 = 1024;
    ((2.0 * radius.ceil()).max(1.0) as u32)
        .next_power_of_two()
        .clamp(MIN, MAX)
}

impl PainterTool {
    /// Scale-blit the cached stamp for each dab (the cacheable path of [`Self::stamp_dabs`]).
    pub(super) fn stamp_dabs_cached(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_stamp_cache(brush, mask_size_for(max_r));
        let Some((mask, _)) = self.paint.stamp_cache.as_ref() else {
            return;
        };
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            if let Some(r) = blit_stamp(
                buf,
                w,
                h,
                d.center,
                d.radius_px,
                mask,
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

    /// Scale-blit each dab through the lazily-filled canvas texture cache — the Tiled / Stencil path
    /// of [`Self::stamp_dabs`]. The texture is computed once per canvas pixel per stroke; the falloff
    /// is still per-pixel (so it's appearance-identical to the per-pixel path).
    pub(super) fn stamp_dabs_canvas_cached(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        self.ensure_canvas_cache(brush, w, h);
        // Disjoint borrows: the canvas (Arc), the cache (tex+ready), and the imported image are all
        // separate fields, so they can be held at once.
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let Some(cache) = self.paint.canvas_tex_cache.as_mut() else {
            return;
        };
        let (tex, ready) = (&mut cache.tex, &mut cache.ready);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            if let Some(r) = blit_canvas_cached(
                buf,
                tex,
                ready,
                w,
                h,
                d.center,
                d.radius_px,
                &spec,
                image.as_ref(),
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

    /// (Re)allocate the canvas texture cache when the texture / image / canvas size changed; the
    /// fresh `ready` (all zero) means every pixel recomputes once on next touch. No-op on a hit.
    fn ensure_canvas_cache(&mut self, brush: &BrushSpec, w: u32, h: u32) {
        let key = CanvasKey {
            texture: brush.texture,
            image_version: self.paint.texture_image_version,
        };
        if matches!(&self.paint.canvas_tex_cache, Some(c) if c.width == w && c.height == h && c.key == key)
        {
            return;
        }
        let n = (w as usize) * (h as usize);
        self.paint.canvas_tex_cache = Some(CanvasTexCache {
            tex: vec![0u8; n],
            ready: vec![0u8; n],
            width: w,
            height: h,
            key,
        });
    }

    /// Re-render the cached stamp mask when the appearance / mask size changed; a no-op on a hit.
    fn ensure_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
        let key = StampKey {
            falloff: brush.falloff,
            hardness: brush.hardness,
            custom: brush.custom_falloff,
            texture: brush.texture,
            image_version: self.paint.texture_image_version,
            size,
        };
        if self.paint.stamp_cache.as_ref().map(|(_, k)| *k) == Some(key) {
            return;
        }
        let mask = {
            let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            render_stamp_mask(brush, image.as_ref(), size)
        };
        self.paint.stamp_cache = Some((mask, key));
    }
}
