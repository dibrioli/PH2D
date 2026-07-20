//! Per-dab **accumulate kernels** for the per-layer-colour stroke path — split out of `stamp_color`
//! for the workspace LOC cap. Each layer keeps its own per-stroke map so the layers recomposite in
//! z-order (the top layer above ALL of the lower's painting across the WHOLE stroke). Three kernels:
//! the cached coverage accumulate (one layer), the **fused** multi-layer coverage accumulate (the hot
//! path), and the dynamic premultiplied-RGBA accumulate (per-dab rotation / colour). Child module of
//! `stamp_color`, so it reads the private `ColorStampMask` / `sample_color_mask` / `grain_modulation`.

use super::{ColorStampMask, grain_modulation, sample_color_mask};
use crate::dab::{DirtyRect, encode};
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, ImageRgb, sample_shape_rgb_unit};

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

/// **Fused** per-layer coverage accumulate — the hot path of the cached per-layer-colour stroke. Byte-for-byte
/// equivalent to calling [`accumulate_color_stamp_coverage`] once per layer (`covs[i]` ⊕ `stamps[i]`), but it
/// is the bottleneck (96.5% of a per-layer-colour Move — `HANDOFF_per_layer_color_perf_artifacts` §1.R), so it
/// removes the per-layer redundant work: all `stamps` share `size`, so the bilinear sample coordinates +
/// the four texel offsets are computed ONCE per canvas pixel and reused across the N layers, and only the
/// ALPHA channel is sampled (the per-layer-colour path discards the stamp RGB — colour is re-applied at
/// recomposite). The per-texel arithmetic per layer is identical to [`accumulate_color_stamp_coverage`]'s
/// alpha lerp + `over` accumulate, so the result is bit-identical (HR-5 + the correctness tests hold — gate:
/// `fused_per_layer_accumulate_is_bit_identical_to_sequential`). `covs.len()` must equal `stamps.len()`.
/// Returns the union touched rect (all layers share the dab box, so it is the dab box iff anything touched).
#[must_use]
pub fn accumulate_color_stamps_fused(
    covs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    stamps: &[ColorStampMask],
    coverage: f32,
) -> Option<DirtyRect> {
    let coverage = coverage.clamp(0.0, 1.0);
    let n = stamps.len();
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 || n == 0 {
        return None;
    }
    debug_assert_eq!(covs.len(), n, "one coverage map per layer");
    let size = stamps[0].size;
    debug_assert!(
        stamps.iter().all(|s| s.size == size),
        "fused accumulate requires all stamps baked at the same size"
    );
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let s = size as f32;
    let si = size as i64;
    let sz = size as usize;
    let clamp = |c: i64| c.clamp(0, si - 1) as usize;
    let mut touched = false;
    for py in y0..y1 {
        let v = ((py as f32 + 0.5) - cy) * inv_radius;
        let my = (v + 1.0) * 0.5 * s - 0.5;
        let fy = my.floor();
        let ty = my - fy;
        let yr0 = clamp(fy as i64) * sz;
        let yr1 = clamp(fy as i64 + 1) * sz;
        let row = (py as usize) * (width as usize);
        for px in x0..x1 {
            let u = ((px as f32 + 0.5) - cx) * inv_radius;
            let mx = (u + 1.0) * 0.5 * s - 0.5;
            let fx = mx.floor();
            let tx = mx - fx;
            let xc0 = clamp(fx as i64);
            let xc1 = clamp(fx as i64 + 1);
            // The four alpha-texel byte offsets, shared by every layer (all stamps share `size`).
            let o00 = (yr0 + xc0) * 4 + 3;
            let o10 = (yr0 + xc1) * 4 + 3;
            let o01 = (yr1 + xc0) * 4 + 3;
            let o11 = (yr1 + xc1) * 4 + 3;
            let idx = row + px as usize;
            for (i, stamp) in stamps.iter().enumerate() {
                let d = &stamp.data;
                // Alpha-only bilinear — identical arithmetic to `sample_color_mask`'s `lerp2(3)`.
                let c = |o: usize| f32::from(d[o]) / 255.0;
                let top = c(o00) + (c(o10) - c(o00)) * tx;
                let bot = c(o01) + (c(o11) - c(o01)) * tx;
                let a_tex = top + (bot - top) * ty;
                let a = a_tex * coverage;
                if a <= 0.0 {
                    continue;
                }
                let cov = &mut covs[i];
                let prev = f32::from(cov[idx]) / 255.0;
                cov[idx] = encode(a + prev * (1.0 - a)); // over-accumulate
                touched = true;
            }
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
/// (Randomize Color rides on `color`). When `rgb` is `Some` (the layer's "Texture Color" mode), the per-
/// pixel colour is sampled from the layer's OWN captured RGB at the same coord as the silhouette instead
/// of the flat `color` — so the tip paints with the texture's real colours. The layers stay separate maps
/// so they still recomposite in z-order (the higher above the lower across the whole stroke). `coverage` =
/// the dab's Flow × Strength × pressure coverage. `over` accumulation (`s + prev·(1−s)`). Returns the
/// touched rect. Deterministic (HR-5).
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
    rgb: Option<&ImageRgb>,
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
            // The per-pixel colour: the layer's OWN captured RGB (Texture Color mode) sampled at the same
            // coord as the silhouette, else the flat resolved `color`.
            let base = match rgb {
                Some(img) => sample_shape_rgb_unit(&spec.shape, shape_basis, u, v, img),
                None => color,
            };
            // The Grain modulates per channel: a scalar (value) without a ramp, or `ramp[g]` (a colour
            // that tints this layer's colour) with one — the alpha channel scales coverage either way.
            let mut tint = base;
            if let Some((gb, gi)) = grain {
                // Canvas-fixed Grain (Tiled / Stencil) MUST sample at the absolute canvas pixel so the
                // Stencil rect mask applies; the dab-local `sample_unit` has no rect → colour leaked
                // outside the Stencil (Enio 2026-06-27). View grain keeps the scale-invariant unit form.
                let g = if spec.texture.mapping.is_canvas_fixed() {
                    crate::texture::sample(&spec.texture, gb, px, py, center, radius, gi)
                } else {
                    crate::texture::sample_unit(&spec.texture, gb, u, v, gi)
                };
                let m = grain_modulation(g, depth, spec.effective_granulation(), grain_ramp);
                tint = [base[0] * m[0], base[1] * m[1], base[2] * m[2]];
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
