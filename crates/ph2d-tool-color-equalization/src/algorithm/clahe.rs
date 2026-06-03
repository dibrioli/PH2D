//! Stage 1 — Contrast-Limited Adaptive Histogram Equalization (CLAHE).
//!
//! Zuiderveld 1994, *Graphics Gems IV* pp. 474-485. Per-tile contrast-
//! limited histogram equalization on the BT.709 luminance channel, with
//! bilinear interpolation of the per-tile LUTs at the pixel level.

use super::util::clamp8;

/// Number of bins in the per-tile luminance histogram. 8-bit luminance
/// means a one-to-one mapping; the LUT step turns this into a per-tile
/// 256-byte table.
const HISTOGRAM_BINS: usize = 256;

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
/// Hue / chroma preservation runs in **BT.709 YCbCr (full range, sRGB
/// gamma-encoded)** — mirrors OpenCV `cvtColor BGR2YCrCb + equalize Y +
/// merge` / Krita's CLAHE pipeline. The histogram + LUT are built over
/// `Y` only; `Cb` and `Cr` (cached at original-pixel precision) are
/// reattached after the equalized `Y` is interpolated. The legacy
/// `scale = new_L / max(old_L, 1)` reconstruction shifted hue in dark
/// pixels (`l_in ∈ {1, 2}` → scale > 50× → one channel saturating
/// before the others) and bled tile-CDF differences into chroma in soft
/// areas — the YCbCr split removes both. Alpha passes through unchanged.
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

    // 1. Y / Cb / Cr buffers (BT.709 full-range, sRGB gamma).
    //    Y as u8 because the histogram + LUT live in [0, 255]; Cb / Cr
    //    as f32 to retain sub-LSB chroma precision through reattach.
    let mut luma = vec![0u8; n_px];
    let mut cb = vec![0.0_f32; n_px];
    let mut cr = vec![0.0_f32; n_px];
    for (i, px) in src.chunks_exact(4).enumerate() {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        luma[i] = clamp8(y);
        cb[i] = (b - y) / 1.8556;
        cr[i] = (r - y) / 1.5748;
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
            let new_y = top + wy * (bot - top);

            // BT.709 full-range inverse: reattach the original Cb / Cr.
            // Tile-CDF differences live only in `new_y`; hue stays put.
            let cb_v = cb[idx];
            let cr_v = cr[idx];
            let r_out = new_y + 1.5748 * cr_v;
            let g_out = new_y - 0.187_324 * cb_v - 0.468_124 * cr_v;
            let b_out = new_y + 1.8556 * cb_v;

            let src_off = idx * 4;
            dst[src_off] = clamp8(r_out);
            dst[src_off + 1] = clamp8(g_out);
            dst[src_off + 2] = clamp8(b_out);
            dst[src_off + 3] = src[src_off + 3];
        }
    }
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
    fn clahe_clip_limit_min_does_not_panic_and_still_normalizes_to_255() {
        // clip_limit=1.0 is the slider MIN. `build_clahe_lut` line 311
        // computes `clip = max(1.0, 1.0) * mean = mean`, so bins at or
        // below the mean don't get clipped — excess only accumulates from
        // bins above the mean. The CDF normalization still maps the top
        // bin to 255 regardless of clip strength. Regression cover
        // (Agent C audit, §6 BAIXA): no test gated this floor before.
        let src = solid(8, 8, [128, 128, 128]);
        let mut dst = vec![0u8; src.len()];
        // Should not panic; uniform input maps the (now single) populated
        // bin to 255 just like the larger clip_limit cases.
        clahe(&src, 8, 8, 1.0, 4, &mut dst);
        for px in dst.chunks_exact(4) {
            assert_eq!(px[0], 255);
            assert_eq!(px[3], 255);
        }
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
}
