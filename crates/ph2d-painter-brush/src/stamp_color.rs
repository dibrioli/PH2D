//! The **multi-layer coloured stamp** — a Shape made of several z-ordered luminance layers, each
//! painted in its OWN colour, composited top-over-bottom into one cached RGBA stamp.
//!
//! This is the per-layer-colour mode of the Shape slot ([`docs/Painter/`]): when the Shape image has
//! several layers and the artist turns on "Per-Layer Color", every layer gets a colour (a custom pick,
//! or the brush's base colour) and the higher layer paints ABOVE the lower one — a multi-colour, 3-D-
//! looking tip. Like the grayscale [`crate::stamp`] it is **scale-invariant**: the N-layer composite is
//! baked ONCE into a unit-square [`ColorStampMask`] (premultiplied RGBA) and bilinearly scale-blitted
//! per dab, so the per-dab cost is one blit — the layer compositing never runs per pixel per dab.
//!
//! Premultiplied storage is deliberate: bilinear sampling of straight colour across an alpha edge bleeds
//! the rim colour inward (dark fringing); premultiplied sampling then un-premultiplying at blit time is
//! the correct, fringe-free path. Deterministic (HR-5): only `+ − × ÷` + the shared samplers.

use crate::blend::BrushBlend;
use crate::dab::{DirtyRect, encode, parallel_band_stamp};
use crate::spec::BrushSpec;
use crate::texture::ImageMask;

/// A baked multi-layer coloured stamp: per-texel **premultiplied** RGBA over the dab's `[-1, 1]²` square,
/// as `u8`. Scale-invariant — one stamp serves every dab size via [`blit_color_stamp`].
pub struct ColorStampMask {
    data: Vec<u8>,
    size: u32,
}

impl ColorStampMask {
    /// The stamp's resolution (texels per side).
    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The premultiplied-RGBA texels (`size·size·4`), for the panel's coloured Shape preview.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// The Grain's per-channel multiplier for a sampled Grain value `g`. WITHOUT a ramp: the scalar value
/// (Grain-Depth-blended toward `1`) on all four channels. WITH the Grain Colour Ramp: `ramp[g]` (each
/// channel Depth-blended toward neutral `1`) so the Grain colour tints the premultiplied composite; the
/// alpha channel scales coverage either way. Deterministic (HR-5: float-only + the shared LUT sampler).
fn grain_modulation(g: f32, depth: f32, ramp: Option<&[[f32; 4]]>) -> [f32; 4] {
    match ramp {
        Some(lut) => {
            let c = crate::dab::ramp_sample(lut, g);
            [
                1.0 + (c[0] - 1.0) * depth,
                1.0 + (c[1] - 1.0) * depth,
                1.0 + (c[2] - 1.0) * depth,
                1.0 + (c[3] - 1.0) * depth,
            ]
        }
        None => {
            let m = if depth >= 1.0 {
                g
            } else {
                1.0 + (g - 1.0) * depth
            };
            [m, m, m, m]
        }
    }
}

/// Render the unit multi-layer coloured stamp at `size`×`size`. `layers` are the Shape silhouette layers
/// in **bottom-to-top** z-order; `layer_colors` is each layer's already-resolved straight RGB (a custom
/// pick, or the brush base colour for an un-coloured layer) — same length as `layers`. Each layer's image
/// REPLACES the falloff (a crisp finite tip, like a single-image Shape), and the layers composite with
/// "over" so a higher layer paints above a lower one. The **Grain** (`spec.texture` + `grain_image`) bites
/// the whole composite's coverage, exactly like the grayscale stamp. With `grain_ramp` (the Grain Colour
/// Ramp LUT, 256 straight-sRGBA entries) the Grain value indexes a COLOUR that multiplies the composite
/// (Grain-Depth-blended toward neutral white) — so the Grain ramp tints the multi-colour tip instead of
/// only darkening it. Deterministic (HR-5).
#[must_use]
pub fn render_color_stamp_mask(
    spec: &BrushSpec,
    layers: &[ImageMask],
    layer_colors: &[[f32; 3]],
    grain_image: Option<&ImageMask>,
    grain_ramp: Option<&[[f32; 4]]>,
    size: u32,
) -> ColorStampMask {
    let size = size.max(1);
    let mut data = vec![0u8; (size as usize) * (size as usize) * 4];
    let grain_active = spec.texture.is_active();
    let depth = spec.grain_depth();
    let footprint = spec.footprint_deform();
    // The Grain + Shape View bases (no Rake/Random/Jitter ⇒ the rng / canvas / extra-rot args are unused).
    let grain_basis = crate::texture::dab_basis(
        &spec.texture,
        [0.0, 0.0],
        &mut 0u64,
        [1.0, 1.0],
        [1.0, 0.0],
        footprint,
    );
    let shape_basis = crate::texture::dab_basis(
        &spec.shape,
        [0.0, 0.0],
        &mut 0u64,
        [1.0, 1.0],
        [1.0, 0.0],
        footprint,
    );
    let inv = 2.0 / size as f32;
    for j in 0..size {
        let v = (j as f32 + 0.5) * inv - 1.0;
        let row = (j as usize) * (size as usize) * 4;
        for i in 0..size {
            let u = (i as f32 + 0.5) * inv - 1.0;
            // Composite the layers bottom→top — each higher layer "over" the accumulated lower ones.
            let mut acc = [0.0f32; 4]; // premultiplied rgba
            for (li, layer) in layers.iter().enumerate() {
                let a = crate::texture::sample_shape_silhouette_unit(
                    &spec.shape,
                    &shape_basis,
                    u,
                    v,
                    Some(layer),
                );
                if a <= 0.0 {
                    continue;
                }
                let c = layer_colors.get(li).copied().unwrap_or([0.0, 0.0, 0.0]);
                let inv_a = 1.0 - a;
                acc[0] = c[0] * a + acc[0] * inv_a;
                acc[1] = c[1] * a + acc[1] * inv_a;
                acc[2] = c[2] * a + acc[2] * inv_a;
                acc[3] = a + acc[3] * inv_a;
            }
            // Grain bites the whole stamp's coverage (premultiplied → scale all four channels). With a
            // Grain Colour Ramp, the Grain value indexes a COLOUR (depth-blended toward neutral white)
            // that multiplies the premultiplied composite — tinting the tip by the Grain pattern.
            if acc[3] > 0.0 && grain_active {
                let g = crate::texture::sample_unit(&spec.texture, &grain_basis, u, v, grain_image);
                let m = grain_modulation(g, depth, grain_ramp);
                acc[0] *= m[0];
                acc[1] *= m[1];
                acc[2] *= m[2];
                acc[3] *= m[3];
            }
            let o = row + (i as usize) * 4;
            data[o] = encode(acc[0]);
            data[o + 1] = encode(acc[1]);
            data[o + 2] = encode(acc[2]);
            data[o + 3] = encode(acc[3]);
        }
    }
    ColorStampMask { data, size }
}

/// Render the unit **ramp-coloured** stamp at `size`×`size` — the cached fast path for a **Grain +
/// Colour Ramp** brush (e.g. a coloured Shape ramp over a coloured Grain). The Grain value indexes the
/// colour `ramp` (exactly like the per-pixel path's grain-ramp colour), the silhouette (the Shape image,
/// else the falloff) is the coverage, and — when `strength_alpha` ([`crate::RampAlphaMode::Strength`]) —
/// the ramp's alpha scales coverage. Scale-invariant (View Grain + a static silhouette), so the whole
/// grain×ramp colour bakes ONCE and scale-blits, instead of resampling the Grain + ramp per pixel per dab
/// (the slow path for the Line/Curve/Circle/Polygon fills). `shape_tone_lut` remaps the silhouette tone
/// (Shape ramp B&W on). Premultiplied RGBA. Deterministic (HR-5).
#[must_use]
pub fn render_ramp_color_stamp(
    spec: &BrushSpec,
    grain_image: Option<&ImageMask>,
    shape_image: Option<&ImageMask>,
    shape_tone_lut: Option<&[f32]>,
    ramp: &[[f32; 4]],
    strength_alpha: bool,
    size: u32,
) -> ColorStampMask {
    let size = size.max(1);
    let mut data = vec![0u8; (size as usize) * (size as usize) * 4];
    let footprint = spec.footprint_deform();
    let grain_basis = crate::texture::dab_basis(
        &spec.texture,
        [0.0, 0.0],
        &mut 0u64,
        [1.0, 1.0],
        [1.0, 0.0],
        footprint,
    );
    let shape_active = spec.shape_silhouette_active(shape_image.is_some());
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
        let row = (j as usize) * (size as usize) * 4;
        for i in 0..size {
            let u = (i as f32 + 0.5) * inv - 1.0;
            // Silhouette coverage — the Shape image REPLACES the falloff (else the bare falloff), exactly
            // like `render_stamp_mask`, so the cached colour matches the per-pixel path's coverage.
            let t = footprint.falloff_t(u, v);
            let w = match &shape_basis {
                Some(sb) => {
                    let raw = crate::texture::sample_shape_silhouette_unit(
                        &spec.shape,
                        sb,
                        u,
                        v,
                        shape_image,
                    );
                    let sv = crate::texture::remap_shape_value(raw, shape_tone_lut);
                    spec.compose_shape_silhouette(sv, spec.falloff_weight(t))
                }
                None => spec.falloff_weight(t),
            };
            if w <= 0.0 {
                continue; // leaves the texel transparent (premul [0,0,0,0])
            }
            // The Grain value indexes the colour ramp (the per-pixel `if grain: ramp[grain]` path).
            let s = crate::texture::sample_unit(&spec.texture, &grain_basis, u, v, grain_image);
            let c = crate::dab::ramp_sample(ramp, s);
            let a = if strength_alpha { w * c[3] } else { w };
            if a <= 0.0 {
                continue;
            }
            let o = row + (i as usize) * 4;
            data[o] = encode(c[0] * a);
            data[o + 1] = encode(c[1] * a);
            data[o + 2] = encode(c[2] * a);
            data[o + 3] = encode(a);
        }
    }
    ColorStampMask { data, size }
}

/// Read-only per-dab context for the coloured-stamp blit, shared across the row bands.
struct ColorBlitCtx<'a> {
    stamp: &'a ColorStampMask,
    blend: BrushBlend,
    cx: f32,
    cy: f32,
    inv_radius: f32,
    coverage: f32,
    preserve_alpha: bool,
    x0: i64,
    x1: i64,
    stride: usize,
}

/// Composite a cached [`ColorStampMask`] onto `buf` at `center` with `radius` (bilinear scale-blit). The
/// per-texel colour comes from the stamp itself; `spec` supplies the blend + Flow × Strength (its
/// `color` is unused — the layers already carry colour). Mirrors [`crate::blit_stamp`]. Returns the
/// touched [`DirtyRect`], or `None` if fully off-canvas / zero-coverage.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blit_color_stamp(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    stamp: &ColorStampMask,
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
    let ctx = ColorBlitCtx {
        stamp,
        blend: spec.blend,
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
        blit_color_band(&ctx, dst, band_y0)
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

/// Blit the cached coloured stamp into the full-width row band `dst` (first row `band_y0`).
fn blit_color_band(ctx: &ColorBlitCtx, dst: &mut [u8], band_y0: i64) -> bool {
    let mut touched = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let v = ((py as f32 + 0.5) - ctx.cy) * ctx.inv_radius;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            let u = ((px as f32 + 0.5) - ctx.cx) * ctx.inv_radius;
            let (rgb, cov) = sample_color_mask(ctx.stamp, u, v);
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
            let out = crate::blend::blend_over(ctx.blend, prev, rgb, a);
            dst[i] = encode(out[0]);
            dst[i + 1] = encode(out[1]);
            dst[i + 2] = encode(out[2]);
            dst[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    touched
}

/// Bilinear (straight RGB, coverage) from the premultiplied `stamp` at the dab-relative unit coord
/// `(u, v) ∈ [-1, 1]`, clamp-to-edge. Samples premultiplied (fringe-free across the alpha edge) then
/// un-premultiplies the colour. Returns `([0; 3], 0.0)` where the composite is transparent.
fn sample_color_mask(stamp: &ColorStampMask, u: f32, v: f32) -> ([f32; 3], f32) {
    let s = stamp.size as f32;
    let mx = (u + 1.0) * 0.5 * s - 0.5;
    let my = (v + 1.0) * 0.5 * s - 0.5;
    let (fx, fy) = (mx.floor(), my.floor());
    let (tx, ty) = (mx - fx, my - fy);
    let si = stamp.size as i64;
    let sz = stamp.size as usize;
    let clamp = |c: i64| c.clamp(0, si - 1) as usize;
    let (x0, x1) = (clamp(fx as i64), clamp(fx as i64 + 1));
    let (y0, y1) = (clamp(fy as i64), clamp(fy as i64 + 1));
    let ch = |xi: usize, yi: usize, k: usize| f32::from(stamp.data[(yi * sz + xi) * 4 + k]) / 255.0;
    let lerp2 = |k: usize| {
        let top = ch(x0, y0, k) + (ch(x1, y0, k) - ch(x0, y0, k)) * tx;
        let bot = ch(x0, y1, k) + (ch(x1, y1, k) - ch(x0, y1, k)) * tx;
        top + (bot - top) * ty
    };
    let pa = lerp2(3);
    if pa <= 0.0 {
        return ([0.0; 3], 0.0);
    }
    let inv = 1.0 / pa;
    ([lerp2(0) * inv, lerp2(1) * inv, lerp2(2) * inv], pa)
}

/// Over-accumulate a layer's per-dab **coverage** (the `stamp`'s alpha × `coverage`) into the per-stroke
/// `cov` map (`width×height`, `u8` `[0,255]`), at `center`/`radius`. The COLOUR is ignored — only the
/// silhouette × Grain coverage matters here; the colour is re-applied at recomposite time. Used by the
/// per-layer-colour stroke path: each layer accumulates its own coverage map across the whole stroke, so
/// the layers can be re-composited bottom→top in z-order (the top layer above ALL of the lower layer's
/// painting), instead of a per-dab composite where a later dab's lower layer buries the earlier highlight.
/// `over` accumulation (`a + prev·(1−a)`) so overlapping dabs build up like normal paint. Returns the
/// touched [`DirtyRect`]; `coverage` is the dab's already-folded Flow × Strength × pressure coverage.
#[must_use]
pub fn accumulate_color_stamp_coverage(
    cov: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    stamp: &ColorStampMask,
    coverage: f32,
) -> Option<DirtyRect> {
    let coverage = coverage.clamp(0.0, 1.0);
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
    let inv_radius = 1.0 / radius;
    let mut touched = false;
    for py in y0..y1 {
        let v = ((py as f32 + 0.5) - cy) * inv_radius;
        let row = (py as usize) * (width as usize);
        for px in x0..x1 {
            let u = ((px as f32 + 0.5) - cx) * inv_radius;
            let (_, a_tex) = sample_color_mask(stamp, u, v);
            let a = a_tex * coverage;
            if a <= 0.0 {
                continue;
            }
            let idx = row + px as usize;
            let prev = f32::from(cov[idx]) / 255.0;
            cov[idx] = encode(a + prev * (1.0 - a)); // over-accumulate
            touched = true;
        }
    }
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

/// Over-accumulate ONE dab of ONE Shape layer — tinted by the dab's `color` — into that layer's per-stroke
/// **premultiplied-RGBA** map (`acc`, `width·height·4`), with the dab's resolved per-dab `shape_basis`
/// (Rake / Random rotation) and the Grain. This is the DYNAMIC per-layer-colour path: unlike the cached
/// [`accumulate_color_stamp_coverage`] (constant orientation, one colour per layer, baked once), it
/// resamples the silhouette per pixel so each dab can rotate (Random / Rake) and carry its own colour
/// (Randomize Color rides on `color`). The layers stay separate maps so they still recomposite in z-order
/// (the higher above the lower across the whole stroke). `coverage` = the dab's Flow × Strength × pressure
/// coverage. `over` accumulation (`s + prev·(1−s)`). Returns the touched rect. Deterministic (HR-5).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn accumulate_shape_layer_rgba(
    acc: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    spec: &BrushSpec,
    shape_basis: &crate::texture::TexDabBasis,
    layer: &ImageMask,
    grain: Option<(&crate::texture::TexDabBasis, Option<&ImageMask>)>,
    grain_ramp: Option<&[[f32; 4]]>,
    color: [f32; 3],
    coverage: f32,
) -> Option<DirtyRect> {
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    let depth = spec.grain_depth();
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let mut touched = false;
    for py in y0..y1 {
        let v = ((py as f32 + 0.5) - cy) * inv_radius;
        let row = (py as usize) * (width as usize);
        for px in x0..x1 {
            let u = ((px as f32 + 0.5) - cx) * inv_radius;
            let mut a = crate::texture::sample_shape_silhouette_unit(
                &spec.shape,
                shape_basis,
                u,
                v,
                Some(layer),
            );
            if a <= 0.0 {
                continue;
            }
            // The Grain modulates per channel: a scalar (value) without a ramp, or `ramp[g]` (a colour
            // that tints this layer's colour) with one — the alpha channel scales coverage either way.
            let mut tint = color;
            if let Some((gb, gi)) = grain {
                // Canvas-fixed Grain (Tiled / Stencil) MUST sample at the absolute canvas pixel so the
                // Stencil rect mask applies; the dab-local `sample_unit` has no rect → colour leaked
                // outside the Stencil (Enio 2026-06-27). View grain keeps the scale-invariant unit form.
                let g = if spec.texture.mapping.is_canvas_fixed() {
                    crate::texture::sample(&spec.texture, gb, px, py, center, radius, gi)
                } else {
                    crate::texture::sample_unit(&spec.texture, gb, u, v, gi)
                };
                let m = grain_modulation(g, depth, grain_ramp);
                tint = [color[0] * m[0], color[1] * m[1], color[2] * m[2]];
                a *= m[3];
                // Stencil: no paint outside the rect even when a ramp recolours `g = 0` (Enio 2026-06-28).
                a *= crate::texture::stencil_gate(&spec.texture, gb, px, py);
            }
            a *= coverage;
            if a <= 0.0 {
                continue;
            }
            let o = (row + px as usize) * 4;
            let inv = 1.0 - a;
            // `acc` holds premultiplied RGBA; over-accumulate this dab's premultiplied colour.
            acc[o] = encode(tint[0] * a + f32::from(acc[o]) / 255.0 * inv);
            acc[o + 1] = encode(tint[1] * a + f32::from(acc[o + 1]) / 255.0 * inv);
            acc[o + 2] = encode(tint[2] * a + f32::from(acc[o + 2]) / 255.0 * inv);
            acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
            touched = true;
        }
    }
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

#[cfg(test)]
mod tests;
