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

/// The bounding union of two touched rects.
fn union_rect(a: DirtyRect, b: DirtyRect) -> DirtyRect {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// One dab of a fused **batch** accumulate: the dab's centre/radius plus its already-folded
/// Flow × Strength × pressure coverage.
#[derive(Copy, Clone, Debug)]
pub struct FusedDab {
    pub center: [f32; 2],
    pub radius: f32,
    pub coverage: f32,
}

/// **Batched + row-banded** fused accumulate — one call per Move instead of one per dab, so the dab
/// loop can be split into disjoint row bands across the cores. Byte-for-byte identical to calling
/// [`accumulate_color_stamps_fused`] once per dab in order: each canvas pixel belongs to exactly ONE
/// band, and every band walks the dabs in the same order, so the per-pixel `over`-accumulate sequence
/// is exactly the serial one (HR-5; gate `batched_fused_accumulate_is_bit_identical_to_sequential`).
/// Returns the union of the touched per-dab rects (same union the per-dab loop produced). Small
/// batches stay serial via the per-dab kernel, so the thread spawn never costs more than it saves.
#[must_use]
pub fn accumulate_color_stamps_fused_batch(
    covs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[FusedDab],
    stamps: &[ColorStampMask],
) -> Option<DirtyRect> {
    let n = stamps.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    // Per-dab clipped boxes (identical arithmetic to the per-dab kernel); drop the no-op dabs.
    struct Box_ {
        di: usize,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        cx: f32,
        cy: f32,
        inv_radius: f32,
        coverage: f32,
    }
    let mut boxes: Vec<Box_> = Vec::with_capacity(dabs.len());
    for (di, d) in dabs.iter().enumerate() {
        let coverage = d.coverage.clamp(0.0, 1.0);
        if coverage <= 0.0 || d.radius <= 0.0 {
            continue;
        }
        let (cx, cy) = (d.center[0], d.center[1]);
        let x0 = (cx - d.radius).floor().max(0.0) as i64;
        let y0 = (cy - d.radius).floor().max(0.0) as i64;
        let x1 = ((cx + d.radius).ceil() as i64 + 1).min(width as i64);
        let y1 = ((cy + d.radius).ceil() as i64 + 1).min(height as i64);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        boxes.push(Box_ {
            di,
            x0,
            y0,
            x1,
            y1,
            cx,
            cy,
            inv_radius: 1.0 / d.radius,
            coverage,
        });
    }
    if boxes.is_empty() {
        return None;
    }
    let gy0 = boxes.iter().map(|b| b.y0).min().expect("non-empty") as usize;
    let gy1 = boxes.iter().map(|b| b.y1).max().expect("non-empty") as usize;
    let area: usize = boxes
        .iter()
        .map(|b| ((b.x1 - b.x0) * (b.y1 - b.y0)) as usize)
        .sum();
    let rows = gy1 - gy0;
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .clamp(1, rows.max(1));
    // Small batches: the per-dab serial kernel (identical bytes by construction).
    if area < crate::dab::PARALLEL_MIN_AREA || threads == 1 || rows <= 1 {
        let mut union: Option<DirtyRect> = None;
        for b in &boxes {
            let d = &dabs[b.di];
            if let Some(r) = accumulate_color_stamps_fused(
                covs, width, height, d.center, d.radius, stamps, d.coverage,
            ) {
                union = Some(union.map_or(r, |u| union_rect(u, r)));
            }
        }
        return union;
    }
    let wus = width as usize;
    let rpb = rows.div_ceil(threads);
    let nbands = rows.div_ceil(rpb);
    // Split every layer map into the SAME disjoint row bands, then transpose to per-band slice sets.
    let mut bands: Vec<Vec<&mut [u8]>> = (0..nbands).map(|_| Vec::with_capacity(n)).collect();
    for map in covs.iter_mut() {
        let mut rest = &mut map[gy0 * wus..gy1 * wus];
        for band in bands.iter_mut() {
            let take = (rpb * wus).min(rest.len());
            let (head, tail) = rest.split_at_mut(take);
            band.push(head);
            rest = tail;
        }
    }
    let size = stamps[0].size;
    debug_assert!(
        stamps.iter().all(|s| s.size == size),
        "fused accumulate requires all stamps baked at the same size"
    );
    let s = size as f32;
    let si = size as i64;
    let sz = size as usize;
    let boxes = &boxes;
    // Per-band touched flags (one per dab), OR-merged after the join so the returned union rect is
    // exactly the per-dab loop's (a dab whose stamp contributed nothing returns no rect).
    let touched_by_band: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let handles: Vec<_> = bands
            .into_iter()
            .enumerate()
            .map(|(bi, mut maps)| {
                scope.spawn(move || {
                    let band_y0 = (gy0 + bi * rpb) as i64;
                    let band_y1 = band_y0 + (maps[0].len() / wus) as i64;
                    let clamp = |c: i64| c.clamp(0, si - 1) as usize;
                    let mut touched = vec![false; dabs.len()];
                    for b in boxes {
                        let y0 = b.y0.max(band_y0);
                        let y1 = b.y1.min(band_y1);
                        for py in y0..y1 {
                            let v = ((py as f32 + 0.5) - b.cy) * b.inv_radius;
                            let my = (v + 1.0) * 0.5 * s - 0.5;
                            let fy = my.floor();
                            let ty = my - fy;
                            let yr0 = clamp(fy as i64) * sz;
                            let yr1 = clamp(fy as i64 + 1) * sz;
                            let row = ((py - band_y0) as usize) * wus;
                            for px in b.x0..b.x1 {
                                let u = ((px as f32 + 0.5) - b.cx) * b.inv_radius;
                                let mx = (u + 1.0) * 0.5 * s - 0.5;
                                let fx = mx.floor();
                                let tx = mx - fx;
                                let xc0 = clamp(fx as i64);
                                let xc1 = clamp(fx as i64 + 1);
                                let o00 = (yr0 + xc0) * 4 + 3;
                                let o10 = (yr0 + xc1) * 4 + 3;
                                let o01 = (yr1 + xc0) * 4 + 3;
                                let o11 = (yr1 + xc1) * 4 + 3;
                                let idx = row + px as usize;
                                for (i, stamp) in stamps.iter().enumerate() {
                                    let d = &stamp.data;
                                    let c = |o: usize| f32::from(d[o]) / 255.0;
                                    let top = c(o00) + (c(o10) - c(o00)) * tx;
                                    let bot = c(o01) + (c(o11) - c(o01)) * tx;
                                    let a_tex = top + (bot - top) * ty;
                                    let a = a_tex * b.coverage;
                                    if a <= 0.0 {
                                        continue;
                                    }
                                    let cov = &mut maps[i];
                                    let prev = f32::from(cov[idx]) / 255.0;
                                    cov[idx] = encode(a + prev * (1.0 - a)); // over-accumulate
                                    touched[b.di] = true;
                                }
                            }
                        }
                    }
                    touched
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("fused accumulate band panicked"))
            .collect()
    });
    let mut union: Option<DirtyRect> = None;
    for b in boxes {
        if touched_by_band.iter().any(|t| t[b.di]) {
            let r = DirtyRect {
                x: b.x0 as u32,
                y: b.y0 as u32,
                w: (b.x1 - b.x0) as u32,
                h: (b.y1 - b.y0) as u32,
            };
            union = Some(union.map_or(r, |u| union_rect(u, r)));
        }
    }
    union
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
                let m = grain_modulation(g, depth, grain_ramp);
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

/// One dab of the DYNAMIC per-layer batch ([`accumulate_shape_layers_rgba_batch`]) — everything the
/// per-dab serial loop resolved before its pixel work: the dab-local Shape / Grain bases (drawn from the
/// tex RNG **in dab order** by the caller, so the stream is byte-identical to the serial loop), the
/// per-layer colours with the dab's Randomize-Color shift folded in, and the folded coverage.
pub struct DynDab {
    pub center: [f32; 2],
    pub radius: f32,
    pub spec: BrushSpec,
    pub shape_basis: crate::texture::TexDabBasis,
    pub grain_basis: Option<crate::texture::TexDabBasis>,
    pub colors: Vec<[f32; 3]>,
    pub coverage: f32,
}

/// **Batched + row-banded + layer-fused** dynamic accumulate — the per-layer-colour TEXTURE-COLOUR path
/// (the capture default), which was the live "FPS 60→10" cliff: `accumulate_shape_layer_rgba` ran
/// serially per dab × per layer (measured 354 ms/move N3 · 1.87 s/move N16 @2048² r100). One call per
/// Move: the dab loop splits into disjoint row bands across the cores, and within a pixel the layers
/// share the dab's (u,v) + the Grain sample + the Stencil gate (identical for every layer — computed
/// once, lazily, instead of ×N). Byte-identical to the serial per-dab loop: each pixel belongs to ONE
/// band, bands walk dabs (then layers) in the serial order, and every per-(layer,pixel) operation is the
/// same arithmetic (gate `batched_dynamic_accumulate_is_bit_identical_to_sequential`). Returns the union
/// of the touched per-dab rects.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn accumulate_shape_layers_rgba_batch(
    accs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[DynDab],
    layers: &[ImageMask<'_>],
    layer_rgb: &[Option<ImageRgb<'_>>],
    grain_image: Option<&ImageMask<'_>>,
    grain_ramp: Option<&[[f32; 4]]>,
) -> Option<DirtyRect> {
    let n = layers.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    debug_assert_eq!(accs.len(), n, "one premul-RGBA map per layer");
    struct Box_ {
        di: usize,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        cx: f32,
        cy: f32,
        inv_radius: f32,
        coverage: f32,
    }
    let mut boxes: Vec<Box_> = Vec::with_capacity(dabs.len());
    for (di, d) in dabs.iter().enumerate() {
        let coverage = d.coverage.clamp(0.0, 1.0);
        if coverage <= 0.0 || d.radius <= 0.0 {
            continue;
        }
        let (cx, cy) = (d.center[0], d.center[1]);
        let x0 = (cx - d.radius).floor().max(0.0) as i64;
        let y0 = (cy - d.radius).floor().max(0.0) as i64;
        let x1 = ((cx + d.radius).ceil() as i64 + 1).min(width as i64);
        let y1 = ((cy + d.radius).ceil() as i64 + 1).min(height as i64);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        boxes.push(Box_ {
            di,
            x0,
            y0,
            x1,
            y1,
            cx,
            cy,
            inv_radius: 1.0 / d.radius,
            coverage,
        });
    }
    if boxes.is_empty() {
        return None;
    }
    // Small batches: the per-dab serial kernel — the byte-identity baseline the parity gate compares
    // the banded path against; spawning threads there would cost more than it saves.
    let area: usize = boxes
        .iter()
        .map(|b| ((b.x1 - b.x0) * (b.y1 - b.y0)) as usize)
        .sum();
    if area < crate::dab::PARALLEL_MIN_AREA {
        let mut union: Option<DirtyRect> = None;
        for b in &boxes {
            let d = &dabs[b.di];
            for (i, layer) in layers.iter().enumerate() {
                let grain = d.grain_basis.as_ref().map(|gb| (gb, grain_image));
                if let Some(r) = accumulate_shape_layer_rgba(
                    &mut accs[i],
                    width,
                    height,
                    d.center,
                    d.radius,
                    &d.spec,
                    &d.shape_basis,
                    layer,
                    grain,
                    grain_ramp,
                    d.colors[i],
                    layer_rgb.get(i).and_then(|o| o.as_ref()),
                    d.coverage,
                ) {
                    union = Some(union.map_or(r, |u| union_rect(u, r)));
                }
            }
        }
        return union;
    }
    let gy0 = boxes.iter().map(|b| b.y0).min().expect("non-empty") as usize;
    let gy1 = boxes.iter().map(|b| b.y1).max().expect("non-empty") as usize;
    let rows = gy1 - gy0;
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .clamp(1, rows.max(1));
    let wus = width as usize;
    let rpb = rows.div_ceil(threads);
    let nbands = rows.div_ceil(rpb);
    // Split every layer map into the SAME disjoint row bands, transposed to per-band slice sets.
    let mut bands: Vec<Vec<&mut [u8]>> = (0..nbands).map(|_| Vec::with_capacity(n)).collect();
    for map in accs.iter_mut() {
        let mut rest = &mut map[gy0 * wus * 4..gy1 * wus * 4];
        for band in bands.iter_mut() {
            let take = (rpb * wus * 4).min(rest.len());
            let (head, tail) = rest.split_at_mut(take);
            band.push(head);
            rest = tail;
        }
    }
    let boxes = &boxes;
    let touched_by_band: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let handles: Vec<_> = bands
            .into_iter()
            .enumerate()
            .map(|(bi, mut maps)| {
                scope.spawn(move || {
                    let band_y0 = (gy0 + bi * rpb) as i64;
                    let band_y1 = band_y0 + (maps[0].len() / (wus * 4)) as i64;
                    let mut touched = vec![false; dabs.len()];
                    for b in boxes {
                        let d = &dabs[b.di];
                        let depth = d.spec.grain_depth();
                        let y0 = b.y0.max(band_y0);
                        let y1 = b.y1.min(band_y1);
                        for py in y0..y1 {
                            let v = ((py as f32 + 0.5) - b.cy) * b.inv_radius;
                            let row = ((py - band_y0) as usize) * wus;
                            for px in b.x0..b.x1 {
                                let u = ((px as f32 + 0.5) - b.cx) * b.inv_radius;
                                // The Grain modulation + Stencil gate are IDENTICAL for every layer at
                                // this pixel (basis + coord only) — computed once, lazily (the serial
                                // loop skipped them whenever the layer silhouette was 0).
                                let mut grain_mg: Option<([f32; 4], f32)> = None;
                                for (i, layer) in layers.iter().enumerate() {
                                    let mut a = crate::texture::sample_shape_silhouette_unit(
                                        &d.spec.shape,
                                        &d.shape_basis,
                                        u,
                                        v,
                                        Some(layer),
                                    );
                                    if a <= 0.0 {
                                        continue;
                                    }
                                    let base = match layer_rgb.get(i).and_then(|o| o.as_ref()) {
                                        Some(img) => sample_shape_rgb_unit(
                                            &d.spec.shape,
                                            &d.shape_basis,
                                            u,
                                            v,
                                            img,
                                        ),
                                        None => d.colors[i],
                                    };
                                    let mut tint = base;
                                    if let Some(gb) = d.grain_basis.as_ref() {
                                        let (m, gate) = *grain_mg.get_or_insert_with(|| {
                                            let g = if d.spec.texture.mapping.is_canvas_fixed() {
                                                crate::texture::sample(
                                                    &d.spec.texture,
                                                    gb,
                                                    px,
                                                    py,
                                                    d.center,
                                                    d.radius,
                                                    grain_image,
                                                )
                                            } else {
                                                crate::texture::sample_unit(
                                                    &d.spec.texture,
                                                    gb,
                                                    u,
                                                    v,
                                                    grain_image,
                                                )
                                            };
                                            (
                                                grain_modulation(g, depth, grain_ramp),
                                                crate::texture::stencil_gate(
                                                    &d.spec.texture,
                                                    gb,
                                                    px,
                                                    py,
                                                ),
                                            )
                                        });
                                        tint = [base[0] * m[0], base[1] * m[1], base[2] * m[2]];
                                        a *= m[3];
                                        a *= gate;
                                    }
                                    a *= b.coverage;
                                    if a <= 0.0 {
                                        continue;
                                    }
                                    let acc = &mut maps[i];
                                    let o = (row + px as usize) * 4;
                                    let inv = 1.0 - a;
                                    acc[o] = encode(tint[0] * a + f32::from(acc[o]) / 255.0 * inv);
                                    acc[o + 1] =
                                        encode(tint[1] * a + f32::from(acc[o + 1]) / 255.0 * inv);
                                    acc[o + 2] =
                                        encode(tint[2] * a + f32::from(acc[o + 2]) / 255.0 * inv);
                                    acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
                                    touched[b.di] = true;
                                }
                            }
                        }
                    }
                    touched
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("dynamic accumulate band panicked"))
            .collect()
    });
    let mut union: Option<DirtyRect> = None;
    for b in boxes {
        if touched_by_band.iter().any(|t| t[b.di]) {
            let r = DirtyRect {
                x: b.x0 as u32,
                y: b.y0 as u32,
                w: (b.x1 - b.x0) as u32,
                h: (b.y1 - b.y0) as u32,
            };
            union = Some(union.map_or(r, |u| union_rect(u, r)));
        }
    }
    union
}

/// **Batched + row-banded fused RGBA** accumulate — the constant-orientation **texture-colour** cached
/// path: each layer's stamp is pre-BAKED with its per-pixel colour (premultiplied RGBA,
/// `render_color_stamp_mask` with the layer's captured RGB), so the per-canvas-pixel work collapses to
/// one shared bilinear coordinate + N four-channel premul samples — instead of the dynamic kernel's
/// per-layer silhouette + RGB resampling (~7× per px). Same structure as
/// [`accumulate_color_stamps_fused_batch`]; the per-(layer,pixel) accumulate matches
/// [`accumulate_shape_layer_rgba`]'s `over` (`c·a_cov + prev·(1−a·cov)` in premul form). NOT
/// byte-identical to the dynamic path (the stamp is quantised at the mask resolution — the same
/// trade the flat cached path already makes); it IS bit-identical across band counts (each pixel
/// belongs to one band; dab order preserved).
#[must_use]
pub fn accumulate_color_stamps_rgba_batch(
    accs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[FusedDab],
    stamps: &[ColorStampMask],
) -> Option<DirtyRect> {
    let n = stamps.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    debug_assert_eq!(accs.len(), n, "one premul-RGBA map per layer");
    let size = stamps[0].size;
    debug_assert!(
        stamps.iter().all(|s| s.size == size),
        "fused accumulate requires all stamps baked at the same size"
    );
    struct Box_ {
        di: usize,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        cx: f32,
        cy: f32,
        inv_radius: f32,
        coverage: f32,
    }
    let mut boxes: Vec<Box_> = Vec::with_capacity(dabs.len());
    for (di, d) in dabs.iter().enumerate() {
        let coverage = d.coverage.clamp(0.0, 1.0);
        if coverage <= 0.0 || d.radius <= 0.0 {
            continue;
        }
        let (cx, cy) = (d.center[0], d.center[1]);
        let x0 = (cx - d.radius).floor().max(0.0) as i64;
        let y0 = (cy - d.radius).floor().max(0.0) as i64;
        let x1 = ((cx + d.radius).ceil() as i64 + 1).min(width as i64);
        let y1 = ((cy + d.radius).ceil() as i64 + 1).min(height as i64);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        boxes.push(Box_ {
            di,
            x0,
            y0,
            x1,
            y1,
            cx,
            cy,
            inv_radius: 1.0 / d.radius,
            coverage,
        });
    }
    if boxes.is_empty() {
        return None;
    }
    let gy0 = boxes.iter().map(|b| b.y0).min().expect("non-empty") as usize;
    let gy1 = boxes.iter().map(|b| b.y1).max().expect("non-empty") as usize;
    let rows = gy1 - gy0;
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .clamp(1, rows.max(1));
    let wus = width as usize;
    let rpb = rows.div_ceil(threads);
    let nbands = rows.div_ceil(rpb);
    let mut bands: Vec<Vec<&mut [u8]>> = (0..nbands).map(|_| Vec::with_capacity(n)).collect();
    for map in accs.iter_mut() {
        let mut rest = &mut map[gy0 * wus * 4..gy1 * wus * 4];
        for band in bands.iter_mut() {
            let take = (rpb * wus * 4).min(rest.len());
            let (head, tail) = rest.split_at_mut(take);
            band.push(head);
            rest = tail;
        }
    }
    let s = size as f32;
    let si = size as i64;
    let sz = size as usize;
    let boxes = &boxes;
    let touched_by_band: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let handles: Vec<_> = bands
            .into_iter()
            .enumerate()
            .map(|(bi, mut maps)| {
                scope.spawn(move || {
                    let band_y0 = (gy0 + bi * rpb) as i64;
                    let band_y1 = band_y0 + (maps[0].len() / (wus * 4)) as i64;
                    let clamp = |c: i64| c.clamp(0, si - 1) as usize;
                    let mut touched = vec![false; dabs.len()];
                    for b in boxes {
                        let y0 = b.y0.max(band_y0);
                        let y1 = b.y1.min(band_y1);
                        for py in y0..y1 {
                            let v = ((py as f32 + 0.5) - b.cy) * b.inv_radius;
                            let my = (v + 1.0) * 0.5 * s - 0.5;
                            let fy = my.floor();
                            let ty = my - fy;
                            let yr0 = clamp(fy as i64) * sz;
                            let yr1 = clamp(fy as i64 + 1) * sz;
                            let row = ((py - band_y0) as usize) * wus;
                            for px in b.x0..b.x1 {
                                let u = ((px as f32 + 0.5) - b.cx) * b.inv_radius;
                                let mx = (u + 1.0) * 0.5 * s - 0.5;
                                let fx = mx.floor();
                                let tx = mx - fx;
                                let xc0 = clamp(fx as i64);
                                let xc1 = clamp(fx as i64 + 1);
                                // The four texel byte offsets, shared by every layer.
                                let o00 = (yr0 + xc0) * 4;
                                let o10 = (yr0 + xc1) * 4;
                                let o01 = (yr1 + xc0) * 4;
                                let o11 = (yr1 + xc1) * 4;
                                let o = (row + px as usize) * 4;
                                for (i, stamp) in stamps.iter().enumerate() {
                                    let d = &stamp.data;
                                    // Premultiplied 4-channel bilinear (shared coords).
                                    let ch = |c: usize| {
                                        let f = |off: usize| f32::from(d[off + c]) / 255.0;
                                        let top = f(o00) + (f(o10) - f(o00)) * tx;
                                        let bot = f(o01) + (f(o11) - f(o01)) * tx;
                                        top + (bot - top) * ty
                                    };
                                    let sa = ch(3);
                                    let a = sa * b.coverage;
                                    if a <= 0.0 {
                                        continue;
                                    }
                                    let acc = &mut maps[i];
                                    let inv = 1.0 - a;
                                    // Premul over-accumulate: `tint·a + prev·(1−a)` with the stamp's
                                    // premul colour already = tint·sa, scaled by the dab coverage.
                                    acc[o] = encode(
                                        ch(0) * b.coverage + f32::from(acc[o]) / 255.0 * inv,
                                    );
                                    acc[o + 1] = encode(
                                        ch(1) * b.coverage + f32::from(acc[o + 1]) / 255.0 * inv,
                                    );
                                    acc[o + 2] = encode(
                                        ch(2) * b.coverage + f32::from(acc[o + 2]) / 255.0 * inv,
                                    );
                                    acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
                                    touched[b.di] = true;
                                }
                            }
                        }
                    }
                    touched
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("rgba accumulate band panicked"))
            .collect()
    });
    let mut union: Option<DirtyRect> = None;
    for b in boxes {
        if touched_by_band.iter().any(|t| t[b.di]) {
            let r = DirtyRect {
                x: b.x0 as u32,
                y: b.y0 as u32,
                w: (b.x1 - b.x0) as u32,
                h: (b.y1 - b.y0) as u32,
            };
            union = Some(union.map_or(r, |u| union_rect(u, r)));
        }
    }
    union
}
