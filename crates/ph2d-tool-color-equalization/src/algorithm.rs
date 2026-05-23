//! Color Equalization — pure CPU pipeline.
//!
//! `std`-only, no editor / ECS / external image deps. Operates on
//! straight-alpha RGBA8 (`w*h*4` bytes, row-major) and produces a fresh
//! RGBA8 buffer of the same dimensions. Three stages:
//!
//! 1. [`clahe`] — Contrast-Limited Adaptive Histogram Equalization on the
//!    BT.709 luminance channel, with bilinear interpolation of per-tile
//!    cumulative distribution functions at the pixel level
//!    (Zuiderveld 1994, *Graphics Gems IV* pp. 474-485).
//! 2. [`adjust_bcs`] — brightness / contrast / saturation in linear-light
//!    sRGB (sRGB → linearize → adjust → delinearize). Brightness is an
//!    additive offset, contrast a multiplicative scale around `0.5`, and
//!    saturation a mix between linear-luma grayscale and the original.
//! 3. [`auto_white_balance`] — Gray-World channel gains in sRGB (mean per
//!    channel → `gain = mean_gray / mean_channel`), applied only over
//!    opaque pixels.
//!
//! [`run_pipeline`] threads them together; each stage is also usable
//! standalone for tests / future GPU parity work.

use crate::params::ColorEqualizationParams;

/// Number of bins in the per-tile luminance histogram. 8-bit luminance
/// means a one-to-one mapping; the LUT step turns this into a per-tile
/// 256-byte table.
const HISTOGRAM_BINS: usize = 256;

/// Run the full Color Equalization pipeline against `rgba` (straight-alpha
/// RGBA8, `w * h * 4` bytes) with `params`, writing into `out` (resized
/// to match). Stages run in order: CLAHE → brightness/contrast/saturation
/// → optional auto-WB.
///
/// The output buffer is reused across runs (HR-3) — the caller owns it.
/// Caller may pass an empty `Vec<u8>` on first call; subsequent calls
/// reuse the allocation.
pub fn run_pipeline(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &ColorEqualizationParams,
    out: &mut Vec<u8>,
) {
    let expected = (w as usize) * (h as usize) * 4;
    assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
    out.clear();
    out.resize(expected, 0);
    if w == 0 || h == 0 {
        return;
    }

    // Stage 1 — CLAHE (writes through into `out`).
    clahe(rgba, w, h, params.clip_limit, params.tile_grid_size, out);

    // Stage 2 — brightness / contrast / saturation in linear sRGB. Runs in
    // place over `out`.
    let identity_bcs =
        params.brightness == 0.0 && params.contrast == 1.0 && params.saturation == 0.0;
    if !identity_bcs {
        adjust_bcs(out, params.brightness, params.contrast, params.saturation);
    }

    // Stage 3 — Gray-World auto white balance (also in place over `out`).
    if params.auto_wb {
        auto_white_balance(out);
    }
}

// ── Stage 1 — CLAHE ───────────────────────────────────────────────────────

/// Apply Contrast-Limited Adaptive Histogram Equalization to the luminance
/// of a straight-alpha RGBA8 image.
///
/// `clip_limit` is the Zuiderveld redistribution ceiling in units of "mean
/// bin count" (`1.0` = uniform histogram → no contrast boost; `2.0` = the
/// canonical default; values above `4` tend to amplify noise harshly).
///
/// `tile_grid_size` is the number of tiles per side (`N×N` partition). The
/// per-tile histograms are 256-bin; per-pixel the LUTs of the four
/// surrounding tile centres are bilinearly interpolated. Boundary pixels
/// fall back to fewer tiles (corner = 1, edge = 2, interior = 4) by
/// clamping the tile-centre indices.
///
/// RGB is reconstructed from the new luminance via a hue-preserving
/// multiplicative scale (`scale = new_L / max(old_L, 1)`), then clamped
/// per-channel — this keeps the original chroma while moving the
/// brightness onto the equalized curve. Alpha passes through unchanged.
pub fn clahe(src: &[u8], w: u32, h: u32, clip_limit: f32, tile_grid_size: u32, dst: &mut [u8]) {
    let n_px = (w as usize) * (h as usize);
    assert_eq!(src.len(), n_px * 4);
    assert_eq!(dst.len(), n_px * 4);
    if w == 0 || h == 0 {
        return;
    }

    // Degenerate tile grid → fall through (preserves source). The panel
    // clamps to ≥4 already, but a tool consumer might bypass that.
    let n_tiles = tile_grid_size.max(1);

    // 1. Luminance buffer (BT.709, straight-alpha sRGB → 8-bit luma).
    let mut luma = vec![0u8; n_px];
    for (i, px) in src.chunks_exact(4).enumerate() {
        luma[i] = luminance_bt709(px[0], px[1], px[2]);
    }

    // 2. Per-tile LUT table: `n_tiles * n_tiles` LUTs of 256 bytes each.
    //    Index `(ty, tx)` → `lut[ty * n_tiles + tx]`.
    let mut luts: Vec<[u8; HISTOGRAM_BINS]> = Vec::with_capacity((n_tiles * n_tiles) as usize);
    let mut histogram = [0u32; HISTOGRAM_BINS];
    for ty in 0..n_tiles {
        for tx in 0..n_tiles {
            let (x0, x1) = tile_span(tx, n_tiles, w);
            let (y0, y1) = tile_span(ty, n_tiles, h);
            histogram.fill(0);
            for y in y0..y1 {
                let row = (y as usize) * (w as usize);
                for x in x0..x1 {
                    histogram[luma[row + x as usize] as usize] += 1;
                }
            }
            let tile_area = (x1 - x0) * (y1 - y0);
            luts.push(build_clahe_lut(&histogram, tile_area, clip_limit));
        }
    }

    // 3. Per-pixel bilinear interp of the 4 nearest tile centres.
    //    Tile centre `(tx, ty)` sits at `((tx+0.5)*tile_w, (ty+0.5)*tile_h)`.
    let tile_w = (w as f32) / (n_tiles as f32);
    let tile_h = (h as f32) / (n_tiles as f32);
    for y in 0..h {
        let fy = (y as f32 + 0.5) / tile_h - 0.5;
        let ty_lo = fy.floor() as i32;
        let ty_hi = ty_lo + 1;
        let wy = fy - ty_lo as f32; // 0..1 weight toward `ty_hi`.
        let ty_lo_c = ty_lo.clamp(0, n_tiles as i32 - 1) as u32;
        let ty_hi_c = ty_hi.clamp(0, n_tiles as i32 - 1) as u32;
        for x in 0..w {
            let fx = (x as f32 + 0.5) / tile_w - 0.5;
            let tx_lo = fx.floor() as i32;
            let tx_hi = tx_lo + 1;
            let wx = fx - tx_lo as f32;
            let tx_lo_c = tx_lo.clamp(0, n_tiles as i32 - 1) as u32;
            let tx_hi_c = tx_hi.clamp(0, n_tiles as i32 - 1) as u32;

            let idx = (y as usize) * (w as usize) + x as usize;
            let l_in = luma[idx] as usize;
            let l00 = luts[(ty_lo_c * n_tiles + tx_lo_c) as usize][l_in] as f32;
            let l10 = luts[(ty_lo_c * n_tiles + tx_hi_c) as usize][l_in] as f32;
            let l01 = luts[(ty_hi_c * n_tiles + tx_lo_c) as usize][l_in] as f32;
            let l11 = luts[(ty_hi_c * n_tiles + tx_hi_c) as usize][l_in] as f32;
            let top = l00 + wx * (l10 - l00);
            let bot = l01 + wx * (l11 - l01);
            let new_l = top + wy * (bot - top);

            let src_off = idx * 4;
            let r = src[src_off];
            let g = src[src_off + 1];
            let b = src[src_off + 2];
            let a = src[src_off + 3];
            let scale = if l_in == 0 {
                // Fully black pixel — no hue to preserve. Spread the new
                // luma evenly across RGB (achromatic grey).
                let v = clamp8(new_l);
                dst[src_off] = v;
                dst[src_off + 1] = v;
                dst[src_off + 2] = v;
                dst[src_off + 3] = a;
                continue;
            } else {
                new_l / l_in as f32
            };
            dst[src_off] = clamp8(r as f32 * scale);
            dst[src_off + 1] = clamp8(g as f32 * scale);
            dst[src_off + 2] = clamp8(b as f32 * scale);
            dst[src_off + 3] = a;
        }
    }
}

/// BT.709 luminance of straight-alpha sRGB (`Y = 0.2126·R + 0.7152·G +
/// 0.0722·B`, rounded to 8 bits).
fn luminance_bt709(r: u8, g: u8, b: u8) -> u8 {
    let y = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    clamp8(y)
}

/// `[start, end)` tile span across `total` pixels for `idx`-th tile of
/// `n_tiles` (integer split with the trailing tile absorbing the remainder
/// so total coverage stays exact).
fn tile_span(idx: u32, n_tiles: u32, total: u32) -> (u32, u32) {
    let start = idx * total / n_tiles;
    let end = if idx + 1 == n_tiles {
        total
    } else {
        (idx + 1) * total / n_tiles
    };
    (start, end)
}

/// Per-tile CLAHE LUT (Zuiderveld redistribution + CDF normalization).
///
/// 1. `clip_count = round(clip_limit * tile_area / 256)`.
/// 2. Truncate each bin at `clip_count`; sum the total excess.
/// 3. Redistribute `excess / 256` uniformly to every bin; any residual
///    excess is doled out one-per-bin cyclically until exhausted (the
///    Zuiderveld "even-out the leftovers" step, mandatory when
///    `excess % 256 != 0`).
/// 4. CDF + normalize so `lut[255] = 255`.
fn build_clahe_lut(
    histogram: &[u32; HISTOGRAM_BINS],
    tile_area: u32,
    clip_limit: f32,
) -> [u8; HISTOGRAM_BINS] {
    let mean = (tile_area as f32 / HISTOGRAM_BINS as f32).max(1.0);
    let clip = (clip_limit.max(1.0) * mean).round() as u32;
    let mut clipped = *histogram;
    let mut excess: u32 = 0;
    for bin in clipped.iter_mut() {
        if *bin > clip {
            excess += *bin - clip;
            *bin = clip;
        }
    }
    // Redistribute. Split excess into the uniform per-bin share + the
    // leftover (Zuiderveld dual loop). The leftover is sprayed in order so
    // the LUT stays deterministic (HR-5).
    let per_bin = excess / HISTOGRAM_BINS as u32;
    let mut residual = excess - per_bin * HISTOGRAM_BINS as u32;
    for bin in clipped.iter_mut() {
        *bin += per_bin;
    }
    if residual > 0 {
        // Spray the leftover one count at a time, walking the bins.
        let mut i = 0;
        while residual > 0 {
            clipped[i] += 1;
            i = (i + 1) % HISTOGRAM_BINS;
            residual -= 1;
        }
    }

    // 4. Build the CDF and normalize to `[0, 255]`. `tile_area` is the
    //    total post-redistribution count (the bumps net to zero since we
    //    only moved counts within the histogram).
    let mut lut = [0u8; HISTOGRAM_BINS];
    if tile_area == 0 {
        // Identity LUT for an empty tile (shouldn't happen with the
        // `max(1)` guard on `n_tiles`, but kept for robustness).
        for (i, v) in lut.iter_mut().enumerate() {
            *v = i as u8;
        }
        return lut;
    }
    let scale = 255.0 / tile_area as f32;
    let mut acc: u32 = 0;
    for (i, count) in clipped.iter().enumerate() {
        acc += count;
        lut[i] = clamp8(acc as f32 * scale);
    }
    lut
}

// ── Stage 2 — brightness / contrast / saturation ──────────────────────────

/// Apply additive brightness + multiplicative contrast (around `0.5`) +
/// luma-mix saturation in linear-light sRGB, in place over straight-alpha
/// RGBA8.
///
/// - `brightness` in `[-1, 1]` — added to each linear channel before clamp.
/// - `contrast` in `[0.5, 2.0]` — `c → (c - 0.5) · k + 0.5`.
/// - `saturation` in `[-1, 1]` — mix factor `1 + saturation` between linear
///   luma (`Y = 0.2126·R + 0.7152·G + 0.0722·B`) and the original.
///
/// Caller should clamp inputs to those ranges; mid-pipeline only clamps
/// the linear output to `[0, 1]` before delinearizing.
pub fn adjust_bcs(rgba: &mut [u8], brightness: f32, contrast: f32, saturation: f32) {
    let sat_mix = 1.0 + saturation;
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3];
        if a == 0 {
            // Transparent pixels: RGB is undefined per straight-alpha
            // convention. Skip to avoid lifting bg colour out of zero
            // alpha (HR-5 — adjust must not change visible output).
            continue;
        }
        let mut r = srgb_to_linear(px[0]);
        let mut g = srgb_to_linear(px[1]);
        let mut b = srgb_to_linear(px[2]);

        // Brightness (additive).
        r += brightness;
        g += brightness;
        b += brightness;

        // Contrast (around 0.5).
        r = (r - 0.5) * contrast + 0.5;
        g = (g - 0.5) * contrast + 0.5;
        b = (b - 0.5) * contrast + 0.5;

        // Saturation — mix with linear luma.
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        r = y + sat_mix * (r - y);
        g = y + sat_mix * (g - y);
        b = y + sat_mix * (b - y);

        px[0] = linear_to_srgb(r.clamp(0.0, 1.0));
        px[1] = linear_to_srgb(g.clamp(0.0, 1.0));
        px[2] = linear_to_srgb(b.clamp(0.0, 1.0));
    }
}

/// sRGB 8-bit → linear `[0, 1]` (IEC 61966-2-1 transfer).
fn srgb_to_linear(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.040_45 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// linear `[0, 1]` → sRGB 8-bit (IEC 61966-2-1 transfer, rounded).
fn linear_to_srgb(c: f32) -> u8 {
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    clamp8(s * 255.0)
}

// ── Stage 3 — auto-WB (Gray-World) ────────────────────────────────────────

/// Gray-World auto white balance applied in place over straight-alpha
/// RGBA8 in sRGB space.
///
/// Averages R / G / B independently over opaque pixels (`alpha > 0`), then
/// computes `gain = mean_gray / mean_channel` per channel and rescales
/// every pixel. Transparent pixels are skipped (no contribution to the
/// mean, no rescale) so a sprite with a transparent border is balanced
/// against the visible subject only.
///
/// Falls back to a no-op when there are no opaque pixels or any channel
/// mean is zero (a fully black or single-channel image — no information to
/// balance against).
pub fn auto_white_balance(rgba: &mut [u8]) {
    let mut sum_r: u64 = 0;
    let mut sum_g: u64 = 0;
    let mut sum_b: u64 = 0;
    let mut count: u64 = 0;
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        sum_r += px[0] as u64;
        sum_g += px[1] as u64;
        sum_b += px[2] as u64;
        count += 1;
    }
    if count == 0 {
        return;
    }
    let mean_r = sum_r as f32 / count as f32;
    let mean_g = sum_g as f32 / count as f32;
    let mean_b = sum_b as f32 / count as f32;
    if mean_r == 0.0 || mean_g == 0.0 || mean_b == 0.0 {
        return;
    }
    let mean_gray = (mean_r + mean_g + mean_b) / 3.0;
    let gain_r = mean_gray / mean_r;
    let gain_g = mean_gray / mean_g;
    let gain_b = mean_gray / mean_b;
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        px[0] = clamp8(px[0] as f32 * gain_r);
        px[1] = clamp8(px[1] as f32 * gain_g);
        px[2] = clamp8(px[2] as f32 * gain_b);
    }
}

// ── Utility ──────────────────────────────────────────────────────────────

fn clamp8(v: f32) -> u8 {
    if v < 0.0 {
        0
    } else if v >= 255.0 {
        255
    } else {
        (v + 0.5) as u8
    }
}

/// Aspect-fit `(sw, sh)` inside a `max_dim × max_dim` box without
/// upscaling. The preview cap uses this to bound CLAHE work per slider
/// drag (briefing PREVIEW cap 512²).
pub fn aspect_fit_within(sw: u32, sh: u32, max_dim: u32) -> (u32, u32) {
    if sw == 0 || sh == 0 || max_dim == 0 {
        return (sw.max(1), sh.max(1));
    }
    if sw <= max_dim && sh <= max_dim {
        return (sw, sh);
    }
    if sw >= sh {
        let dh = ((sh as u64 * max_dim as u64) / sw as u64).max(1) as u32;
        (max_dim, dh)
    } else {
        let dw = ((sw as u64 * max_dim as u64) / sh as u64).max(1) as u32;
        (dw, max_dim)
    }
}

/// Bilinear-interpolating RGBA8 resize, own implementation (no `image`
/// dep). Maps each destination pixel back to a fractional source position
/// and bilinearly samples the four neighbours per channel (alpha
/// included).
pub fn resize_bilinear_rgba(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut dst = vec![0u8; (dw as usize) * (dh as usize) * 4];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return dst;
    }
    let sx_scale = sw as f32 / dw as f32;
    let sy_scale = sh as f32 / dh as f32;
    for y in 0..dh {
        let sy = (y as f32 + 0.5) * sy_scale - 0.5;
        let sy0 = sy.floor().max(0.0) as i32;
        let sy1 = (sy0 + 1).min(sh as i32 - 1);
        let sy0_c = sy0.clamp(0, sh as i32 - 1);
        let wy = (sy - sy0 as f32).clamp(0.0, 1.0);
        for x in 0..dw {
            let sx = (x as f32 + 0.5) * sx_scale - 0.5;
            let sx0 = sx.floor().max(0.0) as i32;
            let sx1 = (sx0 + 1).min(sw as i32 - 1);
            let sx0_c = sx0.clamp(0, sw as i32 - 1);
            let wx = (sx - sx0 as f32).clamp(0.0, 1.0);
            let dst_off = ((y as usize) * (dw as usize) + x as usize) * 4;
            for c in 0..4 {
                let p00 = src[((sy0_c as usize) * (sw as usize) + sx0_c as usize) * 4 + c] as f32;
                let p10 = src[((sy0_c as usize) * (sw as usize) + sx1 as usize) * 4 + c] as f32;
                let p01 = src[((sy1 as usize) * (sw as usize) + sx0_c as usize) * 4 + c] as f32;
                let p11 = src[((sy1 as usize) * (sw as usize) + sx1 as usize) * 4 + c] as f32;
                let top = p00 + wx * (p10 - p00);
                let bot = p01 + wx * (p11 - p01);
                dst[dst_off + c] = clamp8(top + wy * (bot - top));
            }
        }
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4×4 RGBA8 with a single solid colour + opaque alpha.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    /// Read the RGBA pixel at `(x, y)` from a `w`-wide RGBA8 buffer.
    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn clamp8_handles_overflow_and_underflow() {
        assert_eq!(clamp8(-10.0), 0);
        assert_eq!(clamp8(0.0), 0);
        assert_eq!(clamp8(127.4), 127);
        assert_eq!(clamp8(127.6), 128);
        assert_eq!(clamp8(255.0), 255);
        assert_eq!(clamp8(999.0), 255);
    }

    #[test]
    fn srgb_round_trip_8bit_holds() {
        // Every 8-bit value should round-trip with at most ±1 LSB error.
        for v in [0u8, 1, 64, 127, 128, 200, 254, 255] {
            let lin = srgb_to_linear(v);
            let back = linear_to_srgb(lin);
            assert!(
                back.abs_diff(v) <= 1,
                "round trip {v} → {lin} → {back} drifted >1 LSB"
            );
        }
    }

    #[test]
    fn luminance_bt709_canonical_constants() {
        assert_eq!(luminance_bt709(0, 0, 0), 0);
        assert_eq!(luminance_bt709(255, 255, 255), 255);
        // Pure green is the heaviest BT.709 channel: 0.7152·255 ≈ 182.
        assert!(luminance_bt709(0, 255, 0).abs_diff(182) <= 1);
    }

    #[test]
    fn clahe_is_noop_on_uniform_image() {
        // Single-colour input has a Dirac-delta histogram per tile; the
        // CDF maps that bin to 255 and every other bin to 0. So the
        // output is still uniform — but maximally bright. To get a true
        // pass-through, the image must already have luminance 255.
        // Verify the always-255-luma case (solid white).
        let src = solid(16, 16, [255, 255, 255]);
        let mut dst = vec![0u8; src.len()];
        clahe(&src, 16, 16, 2.0, 4, &mut dst);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(px(&dst, 16, x, y), [255, 255, 255, 255]);
            }
        }
    }

    #[test]
    fn clahe_preserves_alpha() {
        let mut src = solid(8, 8, [128, 64, 200]);
        // Make a few pixels semi-transparent.
        src[3] = 50;
        src[(7 + 7 * 8) * 4 + 3] = 0;
        let mut dst = vec![0u8; src.len()];
        clahe(&src, 8, 8, 2.0, 4, &mut dst);
        assert_eq!(dst[3], 50);
        assert_eq!(dst[(7 + 7 * 8) * 4 + 3], 0);
    }

    #[test]
    fn clahe_increases_dynamic_range_on_low_contrast_input() {
        // Build a 32×32 grayscale ramp clamped to mid-range [100, 150]
        // (low contrast). CLAHE should stretch the dynamic range so the
        // output spans a wider min/max gap.
        let mut src = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32u32 {
            for x in 0..32u32 {
                let v = 100u8 + (((x + y) % 50) as u8);
                src.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let mut dst = vec![0u8; src.len()];
        clahe(&src, 32, 32, 2.0, 4, &mut dst);
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in dst.chunks_exact(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        // Source spans 50 luma units; CLAHE should grow that.
        assert!(hi as i32 - lo as i32 >= 50, "CLAHE did not stretch range");
    }

    #[test]
    fn adjust_bcs_identity_leaves_pixels_within_one_lsb() {
        // The identity (b=0, c=1, s=0) should leave each channel within
        // 1 LSB of the input after a round trip through linear.
        let mut buf = solid(4, 4, [60, 130, 200]);
        let before = buf.clone();
        adjust_bcs(&mut buf, 0.0, 1.0, 0.0);
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn adjust_bcs_brightness_lifts_dark_pixels() {
        let mut buf = solid(4, 4, [20, 20, 20]);
        adjust_bcs(&mut buf, 0.5, 1.0, 0.0);
        // Expect each channel meaningfully brighter than 20.
        let r = buf[0];
        assert!(r > 60, "brightness did not lift dark pixel (got {r})");
    }

    #[test]
    fn adjust_bcs_saturation_minus_one_produces_grayscale() {
        let mut buf = solid(4, 4, [200, 50, 50]);
        adjust_bcs(&mut buf, 0.0, 1.0, -1.0);
        let r = buf[0];
        let g = buf[1];
        let b = buf[2];
        // After full desaturation, R == G == B (mod rounding).
        assert!(r.abs_diff(g) <= 1, "R/G drift after desat: {r} vs {g}");
        assert!(g.abs_diff(b) <= 1, "G/B drift after desat: {g} vs {b}");
    }

    #[test]
    fn adjust_bcs_skips_transparent_pixels() {
        let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
        adjust_bcs(&mut buf, 0.5, 1.0, 0.0);
        // First pixel is alpha=0 → RGB untouched.
        assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
        // Second pixel was lifted by brightness.
        assert!(buf[4] > 100);
    }

    #[test]
    fn auto_wb_balances_red_cast() {
        // Average sample: 200 R, 100 G, 100 B → mean grey ≈ 133. Gains
        // should drop R and lift G/B toward grey.
        let mut buf = solid(4, 4, [200, 100, 100]);
        auto_white_balance(&mut buf);
        let r = buf[0];
        let g = buf[1];
        let b = buf[2];
        assert!(r < 200, "R should drop after gray-world (got {r})");
        assert!(g > 100, "G should rise after gray-world (got {g})");
        assert!(b > 100, "B should rise after gray-world (got {b})");
    }

    #[test]
    fn auto_wb_skips_transparent_pixels() {
        // Single transparent pixel + one opaque pixel.
        let mut buf = vec![200u8, 100, 100, 0, 200, 100, 100, 255];
        auto_white_balance(&mut buf);
        assert_eq!(&buf[0..4], &[200, 100, 100, 0]);
        // Opaque pixel was rebalanced; it now sits closer to grey than
        // the input (200, 100, 100).
        let r = buf[4];
        let g = buf[5];
        let b = buf[6];
        assert!(r < 200 && g > 100 && b > 100);
    }

    #[test]
    fn auto_wb_noop_on_pure_grey() {
        let mut buf = solid(4, 4, [128, 128, 128]);
        let before = buf.clone();
        auto_white_balance(&mut buf);
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn run_pipeline_preserves_dimensions() {
        let src = solid(8, 8, [120, 80, 200]);
        let p = ColorEqualizationParams::default();
        let mut out = Vec::new();
        run_pipeline(&src, 8, 8, &p, &mut out);
        assert_eq!(out.len(), src.len());
    }

    #[test]
    fn run_pipeline_auto_wb_toggle_changes_output() {
        // Compose-level verification: with a red-cast input, toggling
        // auto-WB on must change the pipeline output relative to the same
        // pipeline with auto-WB off. (The pure auto-WB stage is exercised
        // by `auto_wb_balances_red_cast`; this test just confirms the
        // pipeline threads the flag through.)
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for y in 0..8u32 {
            for x in 0..8u32 {
                let jitter = ((x + y) % 16) as u8;
                src.extend_from_slice(&[200 - jitter / 2, 100 + jitter, 100 + jitter, 255]);
            }
        }
        let p_off = ColorEqualizationParams {
            auto_wb: false,
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            auto_wb: true,
            ..ColorEqualizationParams::default()
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(&src, 8, 8, &p_off, &mut out_off);
        run_pipeline(&src, 8, 8, &p_on, &mut out_on);
        assert_ne!(
            out_off, out_on,
            "auto-wb flag did not affect pipeline output"
        );
    }

    #[test]
    fn resize_bilinear_identity() {
        let src = solid(4, 4, [100, 150, 200]);
        let dst = resize_bilinear_rgba(&src, 4, 4, 4, 4);
        for (a, b) in dst.iter().zip(src.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn resize_bilinear_halves_dims() {
        let src = solid(8, 8, [100, 150, 200]);
        let dst = resize_bilinear_rgba(&src, 8, 8, 4, 4);
        assert_eq!(dst.len(), 4 * 4 * 4);
        // Solid colour → bilinear is identity-coloured.
        assert_eq!(&dst[..4], &[100, 150, 200, 255]);
    }

    #[test]
    fn aspect_fit_within_caps_larger_dimensions() {
        assert_eq!(aspect_fit_within(1024, 512, 512), (512, 256));
        assert_eq!(aspect_fit_within(512, 1024, 512), (256, 512));
        assert_eq!(aspect_fit_within(400, 300, 512), (400, 300));
    }
}
