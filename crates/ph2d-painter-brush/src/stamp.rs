//! Cached brush **stamp** — render the falloff × texture mask once, scale-blit it per dab.
//!
//! Clean-room model of Blender's brush-image cache (`brush_painter_2d` + `BrushPainterCache`): the
//! per-pixel falloff + texture sample is expensive, so doing it for every dab (and, for the Anchored
//! method, every footprint pixel on every pointer move) is what tanks the FPS. Instead the dab is
//! **scale-invariant** for the View mapping — falloff(`d/R`) and the View texture(`rel/R`) depend on
//! the radius `R` only through scaling — so we render the unit mask once into a [`StampMask`] and
//! [`blit_stamp`] (bilinear scale-blit) it for any size. The texture is sampled once per stroke, not
//! per pixel per dab. Tiled / Stencil / Random / Rake are NOT scale-invariant (canvas-relative or
//! per-dab) and keep the per-pixel [`crate::stamp_dab_textured`] path — see
//! [`crate::TextureSettings::is_cacheable`].

use crate::blend::BrushBlend;
use crate::dab::{DirtyRect, encode, parallel_band_stamp};
use crate::spec::BrushSpec;
use crate::texture::ImageMask;

/// A rendered brush stamp: per-texel coverage (falloff × baked View texture) over the dab's bounding
/// square `[-1, 1]²`, as `u8` alpha (`0` = no paint, `255` = full). Scale-invariant, so one mask
/// serves every dab size via [`blit_stamp`].
pub struct StampMask {
    data: Vec<u8>,
    size: u32,
}

impl StampMask {
    /// The mask's resolution (texels per side).
    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }
}

/// Render the unit brush stamp (**silhouette × Grain**) at `size`×`size`. The silhouette is the Shape
/// image (`shape_image`) when the Shape slot is active, else the procedural falloff (byte-identical to
/// the pre-Shape engine). `grain_image` supplies the pixels for a [`crate::TextureKind::Image`] Grain.
/// The caller renders this only for cacheable settings ([`BrushSpec::dab_mask_cacheable`]); the result
/// is deterministic (HR-5).
#[must_use]
pub fn render_stamp_mask(
    spec: &BrushSpec,
    grain_image: Option<&ImageMask>,
    shape_image: Option<&ImageMask>,
    shape_ramp_lut: Option<&[f32]>,
    size: u32,
) -> StampMask {
    let size = size.max(1);
    let mut data = vec![0u8; (size as usize) * (size as usize)];
    let grain_active = spec.texture.is_active();
    let shape_active = spec.shape_silhouette_active(shape_image.is_some());
    let depth = spec.grain_depth();
    // The dab flatten/rotate — folded into the bases (Shape + Grain) and the falloff `t` so the cached
    // stamp is an elliptical, rotated tip exactly like the per-pixel path.
    let footprint = spec.footprint_deform();
    // The View static bases (no Rake/Random/Jitter ⇒ the rng / canvas / extra-rot args are unused).
    let grain_basis = crate::texture::dab_basis(
        &spec.texture,
        [0.0, 0.0],
        &mut 0u64,
        [1.0, 1.0],
        [1.0, 0.0],
        footprint,
    );
    let shape_basis = shape_active.then(|| {
        crate::texture::dab_basis(
            &spec.shape,
            [0.0, 0.0],
            &mut 0u64,
            [1.0, 1.0],
            [1.0, 0.0],
            footprint,
        )
    });
    let inv = 2.0 / size as f32;
    for j in 0..size {
        let v = (j as f32 + 0.5) * inv - 1.0;
        let row = (j as usize) * (size as usize);
        for i in 0..size {
            let u = (i as f32 + 0.5) * inv - 1.0;
            let mut w = match &shape_basis {
                Some(sb) => {
                    let raw = crate::texture::sample_shape_silhouette_unit(
                        &spec.shape,
                        sb,
                        u,
                        v,
                        shape_image,
                    );
                    let sv = crate::texture::remap_shape_value(raw, shape_ramp_lut);
                    let t = footprint.falloff_t(u, v);
                    spec.compose_shape_silhouette(sv, spec.falloff_weight(t))
                }
                None => {
                    let t = footprint.falloff_t(u, v);
                    spec.falloff_weight(t)
                }
            };
            if w > 0.0 && grain_active {
                let g = crate::texture::sample_unit(&spec.texture, &grain_basis, u, v, grain_image);
                w *= crate::texture::grain_coverage(g, depth, spec.effective_granulation());
            }
            data[row + i as usize] = encode(w);
        }
    }
    StampMask { data, size }
}

/// Read-only per-dab blit context shared by the row bands.
struct BlitCtx<'a> {
    mask: &'a StampMask,
    color: [f32; 3],
    blend: BrushBlend,
    /// Watercolor Pigment mix (`0` = plain blend, byte-identical) — the subtractive RYB amount.
    pigment_mix: f32,
    cx: f32,
    cy: f32,
    inv_radius: f32,
    coverage: f32,
    preserve_alpha: bool,
    x0: i64,
    x1: i64,
    stride: usize,
}

/// Composite a cached [`StampMask`] onto `buf` at `center` with `radius` (bilinear scale-blit), using
/// the brush's blend mode + colour. `coverage` folds in Flow × Strength exactly as
/// [`crate::stamp_dab`] does. Returns the touched [`DirtyRect`], or `None` if fully off-canvas /
/// zero-coverage. Parallelised for large footprints (shared with the per-pixel path).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blit_stamp(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    mask: &StampMask,
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
) -> Option<DirtyRect> {
    debug_assert!(
        buf.len() >= (width as usize) * (height as usize) * 4,
        "buffer too small"
    );
    let coverage =
        coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let stride = (width as usize) * 4;
    let ctx = BlitCtx {
        mask,
        color: spec.color,
        blend: spec.blend,
        pigment_mix: spec.effective_pigment_mix(),
        cx,
        cy,
        inv_radius: 1.0 / radius,
        coverage,
        preserve_alpha,
        x0,
        x1,
        stride,
    };
    let touched = parallel_band_stamp(buf, y0, y1, x0, x1, stride, |dst, band_y0| {
        blit_band(&ctx, dst, band_y0)
    });
    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// Blit the cached mask into the full-width row band `dst` (first row `band_y0`).
fn blit_band(ctx: &BlitCtx, dst: &mut [u8], band_y0: i64) -> bool {
    let mut touched = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let v = ((py as f32 + 0.5) - ctx.cy) * ctx.inv_radius;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            let u = ((px as f32 + 0.5) - ctx.cx) * ctx.inv_radius;
            let cov = sample_mask(ctx.mask, u, v);
            if cov <= 0.0 {
                continue;
            }
            let i = row + (px as usize) * 4;
            let prev = [
                f32::from(dst[i]) / 255.0,
                f32::from(dst[i + 1]) / 255.0,
                f32::from(dst[i + 2]) / 255.0,
                f32::from(dst[i + 3]) / 255.0,
            ];
            let a = if ctx.preserve_alpha {
                cov * ctx.coverage * prev[3]
            } else {
                cov * ctx.coverage
            };
            if a <= 0.0 {
                continue;
            }
            let out = crate::blend::blend_over_pigment(ctx.blend, prev, ctx.color, a, ctx.pigment_mix);
            dst[i] = encode(out[0]);
            dst[i + 1] = encode(out[1]);
            dst[i + 2] = encode(out[2]);
            dst[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    touched
}

/// Bilinear coverage from `mask` at the dab-relative unit coord `(u, v) ∈ [-1, 1]` (centre `(0,0)`),
/// clamp-to-edge (the rim texels are `0`, so clamping reads no paint). Centre-coord convention.
/// `pub(crate)` so the sibling [`crate::stamp_ramped`] colour-ramp blit reuses the same sampler.
pub(crate) fn sample_mask(mask: &StampMask, u: f32, v: f32) -> f32 {
    let s = mask.size as f32;
    let mx = (u + 1.0) * 0.5 * s - 0.5;
    let my = (v + 1.0) * 0.5 * s - 0.5;
    let (fx, fy) = (mx.floor(), my.floor());
    let (tx, ty) = (mx - fx, my - fy);
    let si = mask.size as i64;
    let sz = mask.size as usize;
    let clamp = |c: i64| c.clamp(0, si - 1) as usize;
    let (x0, x1) = (clamp(fx as i64), clamp(fx as i64 + 1));
    let (y0, y1) = (clamp(fy as i64), clamp(fy as i64 + 1));
    let at = |xi: usize, yi: usize| f32::from(mask.data[yi * sz + xi]) / 255.0;
    let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * tx;
    let bot = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * tx;
    top + (bot - top) * ty
}

/// Read-only per-dab context for the canvas-cache blit, shared across the row bands.
struct CanvasBlitCtx<'a> {
    spec: &'a BrushSpec,
    basis: crate::texture::TexDabBasis,
    image: Option<ImageMask<'a>>,
    /// The Shape silhouette frame + image (dab-relative), when the Shape slot is active; else the
    /// falloff is the silhouette. The Grain stays canvas-fixed (cached); the silhouette is per-pixel.
    shape_basis: Option<crate::texture::TexDabBasis>,
    shape_image: Option<ImageMask<'a>>,
    /// The Shape value-ramp LUT (B&W tonal remap of the silhouette); `None` ⇒ no remap.
    shape_ramp_lut: Option<&'a [f32]>,
    /// The dab flatten/rotate — applied to the falloff `t` (the Shape carries it in `shape_basis`).
    footprint: crate::footprint::FootprintDeform,
    cx: f32,
    cy: f32,
    inv_radius: f32,
    radius: f32,
    coverage: f32,
    preserve_alpha: bool,
    x0: i64,
    x1: i64,
    width: usize,
}

/// Composite a **Tiled / Stencil** dab using a lazily-filled CANVAS-SPACE texture cache: the texture
/// value at each canvas pixel is computed once (the first dab to touch it) into `tex` + `ready`, then
/// reused by every later dab — so a large Anchored re-stamp pays the procedural cost **once per pixel
/// per stroke**, not per move. The falloff (dab-relative) is still evaluated per pixel (cheap), so the
/// result is bit-identical to the per-pixel path (no approximation). `tex` / `ready` are canvas-sized
/// (`width*height`); the caller persists them and invalidates on a texture-settings / canvas-size
/// change ([`crate::TextureSettings::is_canvas_cacheable`] gates eligibility). Returns the touched rect.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blit_canvas_cached(
    canvas: &mut [u8],
    tex: &mut [u8],
    ready: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    spec: &BrushSpec,
    image: Option<&ImageMask>,
    shape_image: Option<&ImageMask>,
    shape_ramp_lut: Option<&[f32]>,
    coverage: f32,
    preserve_alpha: bool,
) -> Option<DirtyRect> {
    let coverage =
        coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    // The dab flatten/rotate (Shape + falloff are dab-relative ⇒ deform; the canvas-fixed Grain does not).
    let footprint = spec.footprint_deform();
    // The canvas-fixed texture frame (Tiled rotation / Stencil rect); dab-independent (no Jitter, and
    // canvas-fixed ⇒ identity footprint — the dab flatten never deforms a canvas-locked texture).
    let basis = crate::texture::dab_basis(
        &spec.texture,
        [0.0, 0.0],
        &mut 0u64,
        [width as f32, height as f32],
        [1.0, 0.0],
        crate::footprint::FootprintDeform::identity(),
    );
    // The Shape silhouette frame (dab-relative; static View in this cached path — Rake/Random route to
    // the per-pixel path). `None` ⇒ the falloff is the silhouette.
    let shape_basis = spec
        .shape_silhouette_active(shape_image.is_some())
        .then(|| {
            crate::texture::dab_basis(
                &spec.shape,
                [0.0, 0.0],
                &mut 0u64,
                [1.0, 1.0],
                [1.0, 0.0],
                footprint,
            )
        });
    let ctx = CanvasBlitCtx {
        spec,
        basis,
        image: image.copied(),
        shape_basis,
        shape_image: shape_image.copied(),
        shape_ramp_lut,
        footprint,
        cx,
        cy,
        inv_radius: 1.0 / radius,
        radius,
        coverage,
        preserve_alpha,
        x0,
        x1,
        width: width as usize,
    };
    let touched = crate::dab::parallel_band_cached(
        canvas,
        tex,
        ready,
        width,
        y0,
        y1,
        x0,
        x1,
        |cb, tb, rb, band_y0| canvas_blit_band(&ctx, cb, tb, rb, band_y0),
    );
    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// Blit one row band: lazily fill the canvas texture cache for un-`ready` pixels, then
/// `falloff × cached_texture × coverage` into the canvas band.
fn canvas_blit_band(
    ctx: &CanvasBlitCtx,
    canvas: &mut [u8],
    tex: &mut [u8],
    ready: &mut [u8],
    band_y0: i64,
) -> bool {
    let (cstride, mstride) = (ctx.width * 4, ctx.width);
    let mut touched = false;
    let depth = ctx.spec.grain_depth();
    for r in 0..canvas.len() / cstride {
        let py = band_y0 + r as i64;
        let dy = (py as f32 + 0.5) - ctx.cy;
        for px in ctx.x0..ctx.x1 {
            // SILHOUETTE: the Shape image (dab-relative) or, by default, the falloff — same rule as the
            // per-pixel path, so the cached Tiled/Stencil Grain stays bit-identical with a Shape on top.
            let mut w = match &ctx.shape_basis {
                Some(sb) => {
                    let raw = crate::texture::sample_shape_silhouette(
                        &ctx.spec.shape,
                        sb,
                        px,
                        py,
                        [ctx.cx, ctx.cy],
                        ctx.radius,
                        ctx.shape_image.as_ref(),
                    );
                    let sv = crate::texture::remap_shape_value(raw, ctx.shape_ramp_lut);
                    let dx = (px as f32 + 0.5) - ctx.cx;
                    let t = ctx
                        .footprint
                        .falloff_t(dx * ctx.inv_radius, dy * ctx.inv_radius);
                    ctx.spec
                        .compose_shape_silhouette(sv, ctx.spec.falloff_weight(t))
                }
                None => {
                    let dx = (px as f32 + 0.5) - ctx.cx;
                    let t = ctx
                        .footprint
                        .falloff_t(dx * ctx.inv_radius, dy * ctx.inv_radius);
                    ctx.spec.falloff_weight(t)
                }
            };
            if w <= 0.0 {
                continue; // outside the silhouette → no cache fill
            }
            let mi = r * mstride + px as usize;
            if ready[mi] == 0 {
                // First dab to touch this canvas pixel: compute its (canvas-fixed) texture once.
                let tv = crate::texture::sample(
                    &ctx.spec.texture,
                    &ctx.basis,
                    px,
                    py,
                    [ctx.cx, ctx.cy],
                    ctx.radius,
                    ctx.image.as_ref(),
                );
                tex[mi] = encode(tv);
                ready[mi] = 1;
            }
            // GRAIN Depth + watercolor Granulation (the shared combine); byte-identical at Depth 1 / Gran 0.
            let g = f32::from(tex[mi]) / 255.0;
            w *= crate::texture::grain_coverage(g, depth, ctx.spec.effective_granulation());
            if w <= 0.0 {
                continue;
            }
            let i = r * cstride + (px as usize) * 4;
            let prev = [
                f32::from(canvas[i]) / 255.0,
                f32::from(canvas[i + 1]) / 255.0,
                f32::from(canvas[i + 2]) / 255.0,
                f32::from(canvas[i + 3]) / 255.0,
            ];
            let a = if ctx.preserve_alpha {
                w * ctx.coverage * prev[3]
            } else {
                w * ctx.coverage
            };
            if a <= 0.0 {
                continue;
            }
            let out = crate::blend::blend_over_pigment(
                ctx.spec.blend,
                prev,
                ctx.spec.color,
                a,
                ctx.spec.effective_pigment_mix(),
            );
            canvas[i] = encode(out[0]);
            canvas[i + 1] = encode(out[1]);
            canvas[i + 2] = encode(out[2]);
            canvas[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    touched
}

#[cfg(test)]
mod tests;
