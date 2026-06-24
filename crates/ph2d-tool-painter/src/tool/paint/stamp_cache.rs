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
                color: d.color,
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
                color: d.color,
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

    /// Stamp each dab through the **Color Ramp** colour path — the texture's scalar indexes the baked
    /// ramp LUT for the per-texel colour (the scalar-only caches can't carry colour, so this is
    /// per-pixel; the 256-entry LUT keeps the lookup cheap). Mirrors the per-pixel branch of
    /// [`Self::stamp_dabs`], calling [`ph2d_painter_brush::stamp_dab_ramped`].
    pub(super) fn stamp_dabs_ramped(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        self.ensure_ramp_lut();
        let textured = brush.texture.is_active();
        // Disjoint borrows: the imported image + the ramp LUT (both `self.paint` sub-fields) and the
        // canvas (`self.canvas_rgba`) are held at once; the texture RNG is copied out + written back.
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let lut = &self.paint.texture_ramp_lut;
        let alpha_mode = self.paint.texture_ramp_alpha_mode;
        let mut tex_rng = self.paint.tex_rng;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for (i, d) in dabs.iter().enumerate() {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    super::brush_settings::dab_tangent(dabs, i),
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation,
                )
            });
            if let Some(r) = ph2d_painter_brush::stamp_dab_ramped(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
                lut,
                alpha_mode,
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
        self.paint.tex_rng = tex_rng;
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Per-pixel stamp path — used when no cache applies (no texture, a canvas-relative / per-dab
    /// mapping, or per-dab Jitter Rotate). Resolves the per-dab texture frame (with `d.rotation`) and
    /// the per-dab Randomize-Color `d.color`, then stamps. The texture RNG is copied out (canvas
    /// borrow) + restored, exactly like [`Self::stamp_dabs_ramped`].
    pub(super) fn stamp_dabs_per_pixel(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let textured = brush.texture.is_active();
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let mut tex_rng = self.paint.tex_rng;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for (i, d) in dabs.iter().enumerate() {
            // Per-dab Randomize Color rides on `d.color`; the radius is already jittered in `d`.
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    super::brush_settings::dab_tangent(dabs, i),
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation,
                )
            });
            if let Some(r) = ph2d_painter_brush::stamp_dab_textured(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
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
        self.paint.tex_rng = tex_rng;
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// (Re)bake the ramp LUT when the ramp changed: `eval` is linear RGBA, but the dab blends in the
    /// layer's straight-sRGB space (matching `brush.color` = the picked colour / 255), so the RGB is
    /// converted linear → sRGB; alpha stays straight (no gamma).
    fn ensure_ramp_lut(&mut self) {
        if !self.paint.texture_ramp_dirty && self.paint.texture_ramp_lut.len() == 256 {
            return;
        }
        let mut lut = vec![[0.0f32; 4]; 256];
        self.paint.texture_ramp.bake_into(&mut lut);
        for c in &mut lut {
            c[0] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[0])) / 255.0;
            c[1] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[1])) / 255.0;
            c[2] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[2])) / 255.0;
        }
        self.paint.texture_ramp_lut = lut;
        self.paint.texture_ramp_dirty = false;
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
