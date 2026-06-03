//! Stage 7 — Posterize + Quantize.
//!
//! Both are sequential CPU stages by construction:
//! - Posterize w/ Floyd-Steinberg propagates per-pixel error to four
//!   forward neighbours — strict raster-scan order, no SIMD/GPU port.
//! - K-Means++ Quantize iterates a population-wide cluster fit then
//!   re-maps every pixel — sample-bounded, deterministic seed for stable
//!   palettes on identical inputs.
//!
//! Pipeline runs them AFTER all GPU-amenable stages (auto-WB) so the
//! chained shader path can read back once before this section.

use super::util::clamp8;
use crate::color_utils::{linear_rgb_to_oklab, linear_to_srgb_u8, oklab_to_linear_rgb};

/// Smallest value of `levels` that activates posterize. `0`/`1` are
/// reserved "off" sentinels (a 1-level posterize is meaningless: every
/// pixel would map to the same value).
pub const POSTERIZE_LEVELS_MIN: u32 = 2;
/// Cap matching the legacy panel's discrete option list (`2, 3, 4, 6, 8,
/// 16`). Higher values would round-trip nearly unchanged.
pub const POSTERIZE_LEVELS_MAX: u32 = 16;

/// Smallest k for K-Means++ quantization (mirror of the legacy panel's
/// `4, 8, 16, 32, 64, 128, 256` list). Below 2 the algorithm collapses
/// to a single colour, which is the "off" sentinel.
pub const QUANTIZE_COLORS_MIN: u32 = 2;
/// Hard cap — 256 colours is the indexed-image standard and matches the
/// legacy panel's top option. Above it the sample budget (30k pixels)
/// no longer covers the centroid space cleanly.
pub const QUANTIZE_COLORS_MAX: u32 = 256;

/// K-Means++ sample cap (legacy parity). Quantize on very large images
/// would otherwise pay O(N · k) per iteration; the sampled subset
/// already covers cluster space well at this size.
const QUANTIZE_SAMPLE_CAP: usize = 30_000;

/// Max K-Means iterations (legacy parity). Convergence usually trips the
/// `QUANTIZE_CONVERGE_EPS` early-exit by iter 4-6 on natural images.
const QUANTIZE_MAX_ITER: usize = 10;

/// Centroid Δ threshold (OKLab units). When every centroid moves less
/// than this between iterations we stop early — palette already stable.
const QUANTIZE_CONVERGE_EPS: f32 = 0.001;

/// Deterministic xorshift seed for the K-Means++ initial-centroid draw.
/// Hard-coded so the same input + `num_colors` always produces the same
/// palette across runs (important for snapshot tests + a user re-running
/// Quantize getting the same result, not a new palette every time).
const QUANTIZE_SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Reduce each RGB channel to `levels` discrete steps. When `dithering`
/// is `true`, Floyd-Steinberg error diffusion (7/16 right, 3/16 bottom-
/// left, 5/16 bottom, 1/16 bottom-right) carries the per-channel
/// quantization residue forward through the raster — the legacy
/// pattern, smoother on gradients than the plain map.
///
/// `levels < 2` is the off-sentinel (no-op). Alpha is preserved.
/// In-place on straight-alpha RGBA8.
///
/// **Color space: sRGB gamma (intentional).** The Tier 3 audit considered
/// migrating to linear sRGB for theoretical consistency with the other
/// Phase 2 stages, but FS dithering in linear preserves *physical* light
/// average rather than *perceptual* brightness — a uniform mid-grey 128
/// would dither to ~21% white pixels (linear mean 0.214 = sRGB 128) and
/// the perceived brightness would shift drastically. Pixel-art workflows
/// expect the dithered mosaic to read as the same grey, so the
/// quantization step + error diffusion stay in sRGB. This is also what
/// every reference implementation (legacy engine, Aseprite, GIMP)
/// expects, so palette outputs stay byte-compatible.
pub fn posterize(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    levels: u32,
    dithering: bool,
    dither_strength: f32,
    dither_grain: u32,
) {
    if levels < POSTERIZE_LEVELS_MIN || w == 0 || h == 0 {
        return;
    }
    let levels = levels.min(POSTERIZE_LEVELS_MAX);
    let step = 255.0 / ((levels - 1) as f32);
    let total = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), total * 4);

    let strength = if !dithering {
        0.0
    } else {
        dither_strength.clamp(0.0, 1.0)
    };
    let grain = dither_grain.clamp(1, 8);

    if strength <= f32::EPSILON {
        for px in rgba.chunks_exact_mut(4) {
            for c in &mut px[..3] {
                *c = posterize_value(*c as f32, step);
            }
        }
        return;
    }

    // Floyd-Steinberg path. Grain>1 downsamples to a (w/grain × h/grain)
    // working buffer (block average), runs FS on that grid, then re-
    // upsamples (nearest) into the output. Grain=1 is per-pixel FS.
    let gw = w.div_ceil(grain);
    let gh = h.div_ceil(grain);
    let gtotal = (gw as usize) * (gh as usize);
    let mut buf = vec![0.0_f32; gtotal * 3];

    if grain == 1 {
        for (i, px) in rgba.chunks_exact(4).enumerate() {
            buf[i * 3] = px[0] as f32;
            buf[i * 3 + 1] = px[1] as f32;
            buf[i * 3 + 2] = px[2] as f32;
        }
    } else {
        for by in 0..gh {
            for bx in 0..gw {
                let x0 = (bx * grain) as usize;
                let y0 = (by * grain) as usize;
                let x1 = (x0 + grain as usize).min(w as usize);
                let y1 = (y0 + grain as usize).min(h as usize);
                let mut acc = [0.0_f32; 3];
                let mut count = 0u32;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let pi = (y * w as usize + x) * 4;
                        acc[0] += rgba[pi] as f32;
                        acc[1] += rgba[pi + 1] as f32;
                        acc[2] += rgba[pi + 2] as f32;
                        count += 1;
                    }
                }
                let bi = ((by * gw + bx) as usize) * 3;
                let inv = if count == 0 { 0.0 } else { 1.0 / count as f32 };
                buf[bi] = acc[0] * inv;
                buf[bi + 1] = acc[1] * inv;
                buf[bi + 2] = acc[2] * inv;
            }
        }
    }

    let w_i = gw as isize;
    let h_i = gh as isize;
    for y in 0..h_i {
        for x in 0..w_i {
            let bi = ((y * w_i + x) * 3) as usize;
            for ch in 0..3 {
                let old = buf[bi + ch];
                let new_v = posterize_value(old, step);
                buf[bi + ch] = new_v as f32;
                let err = old - new_v as f32;
                if x + 1 < w_i {
                    buf[bi + 3 + ch] += err * (7.0 / 16.0);
                }
                if y + 1 < h_i {
                    let below = (((y + 1) * w_i + x) * 3) as usize;
                    if x > 0 {
                        buf[below - 3 + ch] += err * (3.0 / 16.0);
                    }
                    buf[below + ch] += err * (5.0 / 16.0);
                    if x + 1 < w_i {
                        buf[below + 3 + ch] += err * (1.0 / 16.0);
                    }
                }
            }
        }
    }

    // Sample (downsampled) buffer back to full res; lerp with the
    // per-pixel plain posterize result by `strength`.
    for y in 0..h as usize {
        for x in 0..w as usize {
            let bx = (x / grain as usize).min(gw as usize - 1);
            let by = (y / grain as usize).min(gh as usize - 1);
            let bi = (by * gw as usize + bx) * 3;
            let pi = (y * w as usize + x) * 4;
            for ch in 0..3 {
                let dith = buf[bi + ch];
                let plain = posterize_value(rgba[pi + ch] as f32, step) as f32;
                let out = plain + (dith - plain) * strength;
                rgba[pi + ch] = clamp8(out);
            }
        }
    }
}

fn posterize_value(v: f32, step: f32) -> u8 {
    let clamped = v.clamp(0.0, 255.0);
    let quantized = (clamped / step).round() * step;
    clamp8(quantized)
}

/// Reduce an image to `num_colors` perceptually balanced colours via
/// K-Means++ clustering in OKLab. Sampling caps the per-iteration cost
/// at [`QUANTIZE_SAMPLE_CAP`] pixels; the resulting palette is mapped
/// back across every opaque pixel (alpha = 0 pixels are skipped — a
/// transparent pixel has no colour to assign).
///
/// `num_colors < 2` is the off-sentinel (no-op). Reproducibility: the
/// K-Means++ seeding RNG is fixed ([`QUANTIZE_SEED`]) so re-quantizing
/// the same image with the same `num_colors` yields the same palette,
/// not a new one each invocation.
pub fn quantize(rgba: &mut [u8], w: u32, h: u32, num_colors: u32) {
    if num_colors < QUANTIZE_COLORS_MIN || w == 0 || h == 0 {
        return;
    }
    let k = num_colors.min(QUANTIZE_COLORS_MAX) as usize;
    let total = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), total * 4);

    // ── Sample opaque pixels into OKLab ─────────────────────────────
    let sample_stride = (total / QUANTIZE_SAMPLE_CAP).max(1);
    let mut samples: Vec<[f32; 3]> = Vec::with_capacity(total / sample_stride + 1);
    for i in (0..total).step_by(sample_stride) {
        let px = &rgba[i * 4..i * 4 + 4];
        if px[3] == 0 {
            continue;
        }
        let lr = crate::color_utils::srgb_to_linear_u8(px[0]);
        let lg = crate::color_utils::srgb_to_linear_u8(px[1]);
        let lb = crate::color_utils::srgb_to_linear_u8(px[2]);
        samples.push(linear_rgb_to_oklab(lr, lg, lb));
    }
    if samples.is_empty() {
        return;
    }

    // ── K-Means++ palette in OKLab ──────────────────────────────────
    let centroids = kmeans_pp_oklab(&samples, k);

    // ── Materialise palette in sRGB (one round-trip per centroid) ───
    let palette_srgb: Vec<[u8; 3]> = centroids
        .iter()
        .map(|c| {
            let lin = oklab_to_linear_rgb(c[0], c[1], c[2]);
            [
                linear_to_srgb_u8(lin[0].max(0.0)),
                linear_to_srgb_u8(lin[1].max(0.0)),
                linear_to_srgb_u8(lin[2].max(0.0)),
            ]
        })
        .collect();

    // Re-encode the palette to OKLab too — we round-tripped through
    // sRGB8 quantization, so OKLab distance against the SHIPPED palette
    // colours (not the raw centroids) is what matches the visual swap.
    let palette_lab: Vec<[f32; 3]> = palette_srgb
        .iter()
        .map(|p| {
            let lr = crate::color_utils::srgb_to_linear_u8(p[0]);
            let lg = crate::color_utils::srgb_to_linear_u8(p[1]);
            let lb = crate::color_utils::srgb_to_linear_u8(p[2]);
            linear_rgb_to_oklab(lr, lg, lb)
        })
        .collect();

    // ── Map every opaque pixel to its nearest palette colour ─────────
    for i in 0..total {
        let px_off = i * 4;
        if rgba[px_off + 3] == 0 {
            continue;
        }
        let lr = crate::color_utils::srgb_to_linear_u8(rgba[px_off]);
        let lg = crate::color_utils::srgb_to_linear_u8(rgba[px_off + 1]);
        let lb = crate::color_utils::srgb_to_linear_u8(rgba[px_off + 2]);
        let lab = linear_rgb_to_oklab(lr, lg, lb);
        let mut best = 0usize;
        let mut best_d = f32::INFINITY;
        for (j, c) in palette_lab.iter().enumerate() {
            let dl = lab[0] - c[0];
            let da = lab[1] - c[1];
            let db = lab[2] - c[2];
            let d = dl * dl + da * da + db * db;
            if d < best_d {
                best_d = d;
                best = j;
            }
        }
        let pal = palette_srgb[best];
        rgba[px_off] = pal[0];
        rgba[px_off + 1] = pal[1];
        rgba[px_off + 2] = pal[2];
    }
}

/// xorshift64 — minimal deterministic RNG seeded from [`QUANTIZE_SEED`].
/// Used only by the K-Means++ initialisation; quality requirements are
/// modest (uniform draws over a small sample set) and we want zero
/// external deps.
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// K-Means++ in OKLab. Returns `k` centroids (or fewer when the sample
/// set already has ≤ `k` points). The implementation is a 1:1 port of
/// the legacy [`quantize.ts`] — D²-weighted seeding then up to
/// [`QUANTIZE_MAX_ITER`] Lloyd iterations with the
/// [`QUANTIZE_CONVERGE_EPS`] early-exit.
fn kmeans_pp_oklab(samples: &[[f32; 3]], k: usize) -> Vec<[f32; 3]> {
    if samples.is_empty() {
        return vec![[0.5, 0.0, 0.0]];
    }
    if samples.len() <= k {
        return samples.to_vec();
    }
    let mut rng = QUANTIZE_SEED;
    let mut centroids: Vec<[f32; 3]> = Vec::with_capacity(k);
    centroids.push(samples[(xorshift64(&mut rng) as usize) % samples.len()]);

    // D²-weighted random selection for the remaining (k-1) centroids.
    while centroids.len() < k {
        let mut distances = Vec::with_capacity(samples.len());
        let mut total_d = 0.0_f32;
        for s in samples {
            let mut min_d = f32::INFINITY;
            for c in &centroids {
                let dl = s[0] - c[0];
                let da = s[1] - c[1];
                let db = s[2] - c[2];
                let d = dl * dl + da * da + db * db;
                if d < min_d {
                    min_d = d;
                }
            }
            distances.push(min_d);
            total_d += min_d;
        }
        if total_d <= 0.0 {
            centroids.push(samples[(xorshift64(&mut rng) as usize) % samples.len()]);
            continue;
        }
        let threshold = (xorshift64(&mut rng) as f32 / u64::MAX as f32) * total_d;
        let mut cumulative = 0.0_f32;
        let mut picked = false;
        for (i, d) in distances.iter().enumerate() {
            cumulative += d;
            if cumulative >= threshold {
                centroids.push(samples[i]);
                picked = true;
                break;
            }
        }
        if !picked {
            centroids.push(samples[(xorshift64(&mut rng) as usize) % samples.len()]);
        }
    }

    // Lloyd iterations: assign → average → repeat until stable.
    let mut assignments = vec![0_usize; samples.len()];
    for _ in 0..QUANTIZE_MAX_ITER {
        for (i, s) in samples.iter().enumerate() {
            let mut best = 0usize;
            let mut best_d = f32::INFINITY;
            for (j, c) in centroids.iter().enumerate() {
                let dl = s[0] - c[0];
                let da = s[1] - c[1];
                let db = s[2] - c[2];
                let d = dl * dl + da * da + db * db;
                if d < best_d {
                    best_d = d;
                    best = j;
                }
            }
            assignments[i] = best;
        }
        let mut sums = vec![[0.0_f32; 3]; k];
        let mut counts = vec![0_u32; k];
        for (i, s) in samples.iter().enumerate() {
            let j = assignments[i];
            sums[j][0] += s[0];
            sums[j][1] += s[1];
            sums[j][2] += s[2];
            counts[j] += 1;
        }
        let mut moved = false;
        for j in 0..k {
            if counts[j] == 0 {
                continue;
            }
            let inv = 1.0 / counts[j] as f32;
            let new_c = [sums[j][0] * inv, sums[j][1] * inv, sums[j][2] * inv];
            if (centroids[j][0] - new_c[0]).abs() > QUANTIZE_CONVERGE_EPS
                || (centroids[j][1] - new_c[1]).abs() > QUANTIZE_CONVERGE_EPS
                || (centroids[j][2] - new_c[2]).abs() > QUANTIZE_CONVERGE_EPS
            {
                moved = true;
            }
            centroids[j] = new_c;
        }
        if !moved {
            break;
        }
    }
    centroids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posterize_off_is_noop() {
        let mut rgba = vec![17, 41, 89, 255, 33, 200, 7, 128];
        let original = rgba.clone();
        posterize(&mut rgba, 2, 1, 0, false, 1.0, 1);
        assert_eq!(rgba, original, "level 0 must be a no-op");
        posterize(&mut rgba, 2, 1, 1, false, 1.0, 1);
        assert_eq!(rgba, original, "level 1 is below MIN, must be a no-op");
    }

    #[test]
    fn posterize_plain_produces_levels_minus_one_steps() {
        // 2 levels → snap to {0, 255}. A mid-range pixel rounds toward
        // the nearest endpoint; alpha untouched.
        let mut rgba = vec![100, 200, 50, 200];
        posterize(&mut rgba, 1, 1, 2, false, 1.0, 1);
        assert!(rgba[0] == 0 || rgba[0] == 255);
        assert!(rgba[1] == 0 || rgba[1] == 255);
        assert!(rgba[2] == 0 || rgba[2] == 255);
        assert_eq!(rgba[3], 200, "alpha must pass through");
    }

    #[test]
    fn posterize_dithered_preserves_average_brightness() {
        // A uniform mid-grey field with FS dithering should land on a
        // mix of the two surrounding palette entries (0 and 255 for
        // levels=2), with the average within a few LSB of the input.
        let w = 32_u32;
        let h = 32_u32;
        let total = (w * h) as usize;
        let mut rgba = vec![128_u8; total * 4];
        for i in 0..total {
            rgba[i * 4 + 3] = 255;
        }
        let avg_before =
            rgba.chunks_exact(4).map(|p| p[0] as u32).sum::<u32>() as f32 / total as f32;
        posterize(&mut rgba, w, h, 2, true, 1.0, 1);
        let avg_after =
            rgba.chunks_exact(4).map(|p| p[0] as u32).sum::<u32>() as f32 / total as f32;
        assert!(
            (avg_before - avg_after).abs() < 6.0,
            "FS dithering must preserve global mean (before={avg_before}, after={avg_after})"
        );
    }

    #[test]
    fn quantize_off_is_noop() {
        let mut rgba = vec![17, 41, 89, 255, 33, 200, 7, 128];
        let original = rgba.clone();
        quantize(&mut rgba, 2, 1, 0);
        assert_eq!(rgba, original, "color count 0 must be a no-op");
        quantize(&mut rgba, 2, 1, 1);
        assert_eq!(
            rgba, original,
            "color count 1 is below MIN, must be a no-op"
        );
    }

    #[test]
    fn quantize_reduces_distinct_colours() {
        // 4×4 image with 16 distinct gradient colours, quantize to 4.
        // After mapping, distinct (R, G, B) triples must be ≤ 4.
        let w = 4_u32;
        let h = 4_u32;
        let mut rgba = Vec::with_capacity(64);
        for y in 0..h {
            for x in 0..w {
                rgba.extend_from_slice(&[
                    (x.saturating_mul(80) as u8),
                    (y.saturating_mul(80) as u8),
                    ((x + y).saturating_mul(40) as u8),
                    255,
                ]);
            }
        }
        quantize(&mut rgba, w, h, 4);
        let mut palette: std::collections::BTreeSet<(u8, u8, u8)> = Default::default();
        for px in rgba.chunks_exact(4) {
            palette.insert((px[0], px[1], px[2]));
        }
        assert!(
            palette.len() <= 4,
            "quantize(k=4) produced {} colours",
            palette.len()
        );
    }

    #[test]
    fn quantize_skips_fully_transparent_pixels() {
        // A transparent pixel's RGB must NOT be replaced by a palette
        // entry — the palette is derived from opaque pixels only and
        // remapping a transparent pixel would silently shift alpha-
        // composited content.
        let mut rgba = vec![10, 20, 30, 0, 200, 200, 200, 255];
        quantize(&mut rgba, 2, 1, 2);
        assert_eq!(
            &rgba[0..3],
            &[10, 20, 30],
            "transparent RGB must pass through"
        );
    }

    #[test]
    fn quantize_is_deterministic_for_same_input() {
        // Same input + k must produce IDENTICAL output across calls —
        // the K-Means++ RNG is fixed-seeded ([`QUANTIZE_SEED`]) so
        // re-quantizing the same image gives the same palette, not a
        // new one each time.
        let w = 8_u32;
        let h = 8_u32;
        let mut rgba_a = Vec::with_capacity(256);
        for i in 0..64 {
            rgba_a.extend_from_slice(&[
                (i * 4) as u8,
                (255 - i * 4) as u8,
                ((i * 7) % 255) as u8,
                255,
            ]);
        }
        let mut rgba_b = rgba_a.clone();
        quantize(&mut rgba_a, w, h, 4);
        quantize(&mut rgba_b, w, h, 4);
        assert_eq!(
            rgba_a, rgba_b,
            "K-Means++ must be deterministic per fixed seed"
        );
    }
}
