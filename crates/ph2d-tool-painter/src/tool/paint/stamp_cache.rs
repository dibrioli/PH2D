//! The Blender-style cached brush stamp on the tool side: render the falloff × View-texture mask
//! once (per appearance / size), then scale-blit it per dab. This turns a large textured re-stamp
//! (Anchored re-drawn every pointer move) from a per-pixel falloff+texture recompute into a cheap
//! bilinear blit — the texture is sampled once per stroke, not per pixel per dab. Eligibility is
//! [`ph2d_painter_brush::TextureSettings::is_cacheable`] (checked by the caller in `paint.rs`).

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, Falloff, FalloffCurve, StrokeMethod, TextureSettings, blit_canvas_cached,
    blit_stamp, blit_stamp_ramped, render_stamp_mask,
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
    /// The Shape slot + its image version + Grain Depth all change the baked silhouette × grain mask.
    shape: TextureSettings,
    shape_image_version: u64,
    /// The Shape value ramp (B&W tonal remap) also changes the baked silhouette.
    shape_ramp_version: u64,
    grain_depth: f32,
    /// The dab flatten + rotate reshapes the cached mask into a rotated ellipse, so they key it.
    dab_flatten: f32,
    dab_angle_deg: u16,
    size: u32,
}

/// Mask resolution for a dab of `radius` px: enough texels to be 1:1 (`2·radius`), rounded up to a
/// power of two so a growing Anchored dab only re-renders a few times, clamped. Beyond the max the
/// texture is bilinearly upscaled (softer) — Blender caps its brush image likewise.
pub(super) fn mask_size_for(radius: f32) -> u32 {
    const MIN: u32 = 32;
    const MAX: u32 = 1024;
    ((2.0 * radius.ceil()).max(1.0) as u32)
        .next_power_of_two()
        .clamp(MIN, MAX)
}

/// Size the per-stroke coverage mask for the Accumulate-OFF (Strength < 1) cap. The FILL methods
/// (Line/Curve/Circle/Polygon/Free Hand) re-stamp the WHOLE stroke each call (drag-preview restore), so
/// the cap must start FRESH each time — else the mask is already AT the cap from the prior re-stamp and
/// the new one adds nothing, so a Strength change (which re-fills) ERASES the stroke. The incremental
/// methods (Space/Dots/Airbrush) accumulate across batches (resize only zero-fills the new tail). A free
/// fn taking just the mask so it doesn't conflict with the per-pixel paths' other field borrows. Enio.
fn prepare_stroke_mask(mask: &mut Vec<u8>, len: usize, method: StrokeMethod) {
    if !matches!(
        method,
        StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
    ) {
        mask.clear(); // fill re-stamp → fresh cap
    }
    mask.resize(len, 0);
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

    /// Like [`Self::stamp_dabs_cached`] but the per-texel colour comes from the **Shape Colour Ramp**
    /// indexed by the cached coverage — the cacheable fast path for a no-Grain colour-ramp stroke. The
    /// silhouette is rendered once into the [`ph2d_painter_brush::StampMask`] (the no-Grain gate already
    /// suppresses the B&W tone ramp, so the mask is the raw silhouette × falloff coverage) and the ramp
    /// is a 256-entry LUT lookup at blit time — vs the per-pixel [`Self::stamp_dabs_ramped`] recomputing
    /// the silhouette every pixel every dab (Enio 2026-06-26).
    pub(super) fn stamp_dabs_cached_ramped(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let owner = self.color_ramp_owner(brush.texture.is_active());
        self.ensure_ramp_lut(owner);
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_stamp_cache(brush, mask_size_for(max_r));
        let Some((mask, _)) = self.paint.stamp_cache.as_ref() else {
            return;
        };
        let ramp = self.paint.texture_ramp_lut.as_slice();
        let alpha_mode = self.active_ramp_alpha_mode(owner);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            if let Some(r) = blit_stamp_ramped(
                buf,
                w,
                h,
                d.center,
                d.radius_px,
                mask,
                &spec,
                d.coverage,
                alpha_locked,
                ramp,
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
        self.ensure_shape_ramp_lut();
        // Disjoint borrows: the canvas (Arc), the cache (tex+ready), the imported images + the Shape
        // ramp LUT are all separate fields, so they can be held at once.
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        // Shape **tone** ramp applies when its B&W filter is on (the Grain owns colour); see
        // `stamp_dabs_ramped` (Enio 2026-06-26).
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
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
                shape_image.as_ref(),
                shape_ramp_lut,
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
        let owner = self.color_ramp_owner(brush.texture.is_active());
        self.ensure_ramp_lut(owner);
        self.ensure_shape_ramp_lut();
        let textured = brush.texture.is_active();
        // Disjoint borrows: the imported image + the ramp LUT (both `self.paint` sub-fields) and the
        // canvas (`self.canvas_rgba`) are held at once; the texture RNG is copied out + written back.
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        // Shape **tone** ramp applies when the Shape ramp's B&W filter is on (then the Grain / brush
        // owns colour and the Shape remaps the silhouette tone); off ⇒ the Shape ramp is the colour
        // owner, so no tone remap (Enio 2026-06-26).
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let lut = &self.paint.texture_ramp_lut;
        let alpha_mode = self.active_ramp_alpha_mode(owner);
        let mut tex_rng = self.paint.tex_rng;
        // Accumulate OFF (Strength < 1) caps each pixel's stroke coverage at Strength — thread the
        // per-stroke mask so a Color-Ramp stroke honours Accumulate too (Enio 2026-06-25).
        let accumulate_cap = !brush.accumulate && brush.strength < 1.0;
        if accumulate_cap {
            prepare_stroke_mask(
                &mut self.paint.stroke_mask,
                (w as usize) * (h as usize),
                brush.stroke_method,
            );
        }
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut mask: Option<&mut [u8]> =
            accumulate_cap.then_some(self.paint.stroke_mask.as_mut_slice());
        let mut touched: Option<Region> = None;
        for d in dabs.iter() {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            // The Rake heading is the dab's own smoothed path direction `d.dir` (computed once in the
            // engine, where the path geometry is known). Both slots read the same heading; `dab_basis`
            // ignores it unless that slot's Rake is on, and falls back to the base Angle for `[0, 0]`.
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.shape,
                    d.dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation, // Jitter Rotate spins the Shape with the Grain (the whole stamp)
                    spec.footprint_deform(),
                )
            });
            let shape_in = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    d.dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation,
                    spec.footprint_deform(),
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
                shape_in,
                lut,
                alpha_mode,
                mask.as_deref_mut(),
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
        self.ensure_shape_ramp_lut();
        let textured = brush.texture.is_active();
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        // Shape **tone** ramp applies when its B&W filter is on (the Grain owns colour); see
        // `stamp_dabs_ramped` (Enio 2026-06-26).
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        let mut tex_rng = self.paint.tex_rng;
        // Accumulate OFF (Strength < 1): hand the per-pixel blit the per-stroke coverage mask so it
        // caps each pixel at Strength. `paint_begin` cleared it on pointer-down; grow it to canvas size
        // (only the first dab of a stroke actually zero-fills — later dabs/frames keep the accumulation).
        let accumulate_cap = !brush.accumulate && brush.strength < 1.0;
        if accumulate_cap {
            prepare_stroke_mask(
                &mut self.paint.stroke_mask,
                (w as usize) * (h as usize),
                brush.stroke_method,
            );
        }
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut mask: Option<&mut [u8]> =
            accumulate_cap.then_some(self.paint.stroke_mask.as_mut_slice());
        let mut touched: Option<Region> = None;
        for d in dabs.iter() {
            // Per-dab Randomize Color rides on `d.color`; the radius is already jittered in `d`.
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            // The Rake heading is the dab's own smoothed path direction `d.dir` (computed once in the
            // engine). Shape draws its Random from `tex_rng` here, *before* the Grain, so a brush with
            // no Shape Random leaves the Grain stream byte-identical; `dab_basis` ignores `d.dir` unless
            // that slot's Rake is on, falling back to the base Angle for `[0, 0]`.
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.shape,
                    d.dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation, // Jitter Rotate spins the Shape with the Grain (the whole stamp)
                    spec.footprint_deform(),
                )
            });
            let shape_in = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    d.dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation,
                    spec.footprint_deform(),
                )
            });
            if let Some(r) = ph2d_painter_brush::stamp_dab_textured_masked(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
                shape_in,
                mask.as_deref_mut(),
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

    /// Re-render the cached stamp mask when the appearance / mask size changed; a no-op on a hit.
    fn ensure_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
        self.ensure_shape_ramp_lut();
        let key = StampKey {
            falloff: brush.falloff,
            hardness: brush.hardness,
            custom: brush.custom_falloff,
            texture: brush.texture,
            image_version: self.paint.texture_image_version,
            shape: brush.shape,
            shape_image_version: self.paint.shape_image_version,
            shape_ramp_version: self.paint.shape_ramp_version,
            grain_depth: brush.grain_depth,
            dab_flatten: brush.dab_flatten,
            dab_angle_deg: brush.dab_angle_deg,
            size,
        };
        if self.paint.stamp_cache.as_ref().map(|(_, k)| *k) == Some(key) {
            return;
        }
        let mask = {
            let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
            let shape_ramp_lut = self.shape_tone_lut_slice();
            render_stamp_mask(
                brush,
                image.as_ref(),
                shape_image.as_ref(),
                shape_ramp_lut,
                size,
            )
        };
        self.paint.stamp_cache = Some((mask, key));
    }
}
