//! Stage 3 — auto-WB (Gray-World) + Stage 4 — histogram & automatic
//! adjustments (Phase 2).
//!
//! Histogram + automatic adjustments. Each is pure / deterministic /
//! `std`-only. CPU implementations target the Apply path (one-shot per
//! sprite). [`auto_white_balance`] (Gray-World) runs in linear sRGB;
//! [`auto_levels`] / [`auto_colors`] stretch per-channel sRGB histograms;
//! [`auto_contrast`] stretches BT.709 linear-light luminance.

use super::util::clamp8;

/// Gray-World auto white balance applied in place over straight-alpha
/// RGBA8 — runs in **linear sRGB**, not gamma-encoded.
///
/// Why linear: averaging luminance is a physical operation (mean of
/// light), and sRGB is a perceptual encoding that compresses shadows.
/// Averaging gamma-encoded values pulls the mean toward the dark end and
/// biases the gains — visible as drifted WB in high-contrast scenes
/// (sun + shadow). Decoding to linear before averaging restores the
/// photon-space invariant gray-world depends on.
///
/// Averages linear R / G / B independently over opaque pixels (`alpha >
/// 0`), then computes `gain = mean_gray / mean_channel` per channel and
/// rescales every pixel in linear space before re-encoding sRGB.
/// Transparent pixels are skipped (no contribution to the mean, no
/// rescale).
///
/// Falls back to a no-op when there are no opaque pixels or any channel
/// mean is zero (a fully black or single-channel image — no information to
/// balance against).
pub fn auto_white_balance(rgba: &mut [u8]) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    let mut sum_r = 0.0_f64;
    let mut sum_g = 0.0_f64;
    let mut sum_b = 0.0_f64;
    let mut count: u64 = 0;
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        sum_r += srgb_to_linear_u8(px[0]) as f64;
        sum_g += srgb_to_linear_u8(px[1]) as f64;
        sum_b += srgb_to_linear_u8(px[2]) as f64;
        count += 1;
    }
    if count == 0 {
        return;
    }
    let mean_r = (sum_r / count as f64) as f32;
    let mean_g = (sum_g / count as f64) as f32;
    let mean_b = (sum_b / count as f64) as f32;
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
        let r_lin = srgb_to_linear_u8(px[0]) * gain_r;
        let g_lin = srgb_to_linear_u8(px[1]) * gain_g;
        let b_lin = srgb_to_linear_u8(px[2]) * gain_b;
        px[0] = linear_to_srgb_u8(r_lin);
        px[1] = linear_to_srgb_u8(g_lin);
        px[2] = linear_to_srgb_u8(b_lin);
    }
}

/// Per-channel 256-bin histogram (R, G, B, and BT.709 luma) plus the count
/// of opaque pixels that contributed. Built by [`compute_histogram`] from
/// a straight-alpha RGBA8 buffer; consumed by [`auto_levels`] /
/// [`auto_contrast`] / [`auto_colors`] and by the panel's overlay
/// visualizer. Skips fully transparent pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistogramData {
    pub r: [u32; 256],
    pub g: [u32; 256],
    pub b: [u32; 256],
    /// BT.709 luma — `Y = 0.2126·R + 0.7152·G + 0.0722·B`. Matches the
    /// sRGB primaries CLAHE / `luma_srgb` rely on; the older BT.601
    /// (`0.299, 0.587, 0.114`) constants were for analog NTSC encoding
    /// and don't reflect modern sRGB luminance.
    pub l: [u32; 256],
    /// Pixels with `alpha > 0` counted across all channels.
    pub opaque_count: u32,
}

impl Default for HistogramData {
    fn default() -> Self {
        Self {
            r: [0; 256],
            g: [0; 256],
            b: [0; 256],
            l: [0; 256],
            opaque_count: 0,
        }
    }
}

/// Compute per-channel + luma histograms from a straight-alpha RGBA8
/// buffer. Skips fully transparent pixels.
///
/// CPU-only by design: atomic histogram updates on GPU suffer contention
/// and the analytical passes already dominate the per-image cost in CLAHE
/// when the histogram is needed there. Linear-scan CPU is ~4 ns per pixel
/// in release — under 5 ms for 1024².
pub fn compute_histogram(pixels: &[ph2d_color::SrgbRgba]) -> HistogramData {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    let mut h = HistogramData::default();
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        h.r[px[0] as usize] += 1;
        h.g[px[1] as usize] += 1;
        h.b[px[2] as usize] += 1;
        // BT.709 luma — sRGB primaries. Coefficients applied to sRGB
        // gamma-encoded values (matches the inline Y in `clahe`), not
        // linear — fine for histogram bucketing.
        let luma = (0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32) as usize;
        h.l[luma.min(255)] += 1;
        h.opaque_count += 1;
    }
    h
}

/// Build a 256-byte LUT that stretches `[min, max]` linearly onto
/// `[0, 255]`. Pixels at the extremes saturate. Used by the auto-* stages.
fn stretch_lut(min: u8, max: u8) -> [u8; 256] {
    let mut lut = [0u8; 256];
    let range = (max as i32 - min as i32).max(1) as f32;
    for (i, v) in lut.iter_mut().enumerate() {
        let stretched = ((i as i32 - min as i32) as f32 / range) * 255.0;
        *v = clamp8(stretched);
    }
    lut
}

/// Find the `[min, max]` channel range that excludes the bottom and top
/// `cutoff_fraction` percentiles. `cutoff_fraction` is in `[0, 1]` —
/// typical values are `0.005` (Auto Levels) or `0.01` (Auto Colors).
fn percentile_range(hist: &[u32; 256], total: u32, cutoff_fraction: f32) -> (u8, u8) {
    if total == 0 {
        return (0, 255);
    }
    let cutoff = (total as f32 * cutoff_fraction).floor() as u32;
    let mut count: u32 = 0;
    let mut lo = 0u8;
    for (v, &c) in hist.iter().enumerate() {
        count += c;
        if count > cutoff {
            lo = v as u8;
            break;
        }
    }
    count = 0;
    let mut hi = 255u8;
    for (v, &c) in hist.iter().enumerate().rev() {
        count += c;
        if count > cutoff {
            hi = v as u8;
            break;
        }
    }
    (lo, hi)
}

/// Auto Levels — per-channel histogram stretching with 0.5 % outlier
/// trimming. Same `findRange` shape as the legacy `autoLevels`.
pub fn auto_levels(rgba: &mut [u8]) {
    let hist = compute_histogram(bytemuck::cast_slice(rgba));
    if hist.opaque_count == 0 {
        return;
    }
    let (r_lo, r_hi) = percentile_range(&hist.r, hist.opaque_count, 0.005);
    let (g_lo, g_hi) = percentile_range(&hist.g, hist.opaque_count, 0.005);
    let (b_lo, b_hi) = percentile_range(&hist.b, hist.opaque_count, 0.005);
    let lut_r = stretch_lut(r_lo, r_hi);
    let lut_g = stretch_lut(g_lo, g_hi);
    let lut_b = stretch_lut(b_lo, b_hi);
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        px[0] = lut_r[px[0] as usize];
        px[1] = lut_g[px[1] as usize];
        px[2] = lut_b[px[2] as usize];
    }
}

/// Auto Colors — per-channel stretching with 1 % outlier trimming.
/// Softer than Auto Levels; matches the legacy `autoColors`.
pub fn auto_colors(rgba: &mut [u8]) {
    let hist = compute_histogram(bytemuck::cast_slice(rgba));
    if hist.opaque_count == 0 {
        return;
    }
    let (r_lo, r_hi) = percentile_range(&hist.r, hist.opaque_count, 0.01);
    let (g_lo, g_hi) = percentile_range(&hist.g, hist.opaque_count, 0.01);
    let (b_lo, b_hi) = percentile_range(&hist.b, hist.opaque_count, 0.01);
    let lut_r = stretch_lut(r_lo, r_hi);
    let lut_g = stretch_lut(g_lo, g_hi);
    let lut_b = stretch_lut(b_lo, b_hi);
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        px[0] = lut_r[px[0] as usize];
        px[1] = lut_g[px[1] as usize];
        px[2] = lut_b[px[2] as usize];
    }
}

/// Auto Contrast — stretches BT.709 **linear-light** luminance via a
/// uniform ratio scale on linear-sRGB RGB (preserves hue). Uses 5 %/95 %
/// percentile cut.
///
/// Linear-light Y is the correct lightness measure here: pure red
/// (255,0,0) and pure blue (0,0,255) have HSL L = 0.5 each, but their
/// linear luminances differ by ~3× (`Y_red ≈ 0.21`, `Y_blue ≈ 0.07`).
/// HSL L would treat them as equivalent and the per-channel scale by
/// `new_L/L` would push saturated pixels past 1.0 in one channel before
/// the others — manifest as hue drift. BT.709 linear keeps the scale
/// physically meaningful.
pub fn auto_contrast(rgba: &mut [u8]) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    // 1. Linear-luma histogram (256 bins over `[0, 1]`).
    let mut hist_l = [0u32; 256];
    let mut total: u32 = 0;
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        let rl = srgb_to_linear_u8(px[0]);
        let gl = srgb_to_linear_u8(px[1]);
        let bl = srgb_to_linear_u8(px[2]);
        let y = 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
        let bin = (y.clamp(0.0, 1.0) * 255.0).round() as usize;
        hist_l[bin.min(255)] += 1;
        total += 1;
    }
    if total == 0 {
        return;
    }
    let (lo, hi) = percentile_range(&hist_l, total, 0.05);
    let lo_n = lo as f32 / 255.0;
    let range = ((hi as f32 - lo as f32) / 255.0).max(f32::EPSILON);

    // 2. Per-pixel: stretch linear Y, scale linear RGB by the ratio, encode.
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let rl = srgb_to_linear_u8(px[0]);
        let gl = srgb_to_linear_u8(px[1]);
        let bl = srgb_to_linear_u8(px[2]);
        let y = 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
        if y <= 0.0 || y >= 1.0 {
            continue;
        }
        let new_y = ((y - lo_n) / range).clamp(0.0, 1.0);
        let ratio = new_y / y;
        px[0] = linear_to_srgb_u8(rl * ratio);
        px[1] = linear_to_srgb_u8(gl * ratio);
        px[2] = linear_to_srgb_u8(bl * ratio);
    }
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

    // ── Phase 2 ──────────────────────────────────────────────────────

    #[test]
    fn histogram_skips_transparent_and_counts_opaque() {
        let buf = vec![
            10u8, 20, 30, 0, // transparent — skipped
            10, 20, 30, 255, 200, 100, 50, 255,
        ];
        let h = compute_histogram(bytemuck::cast_slice(&buf));
        assert_eq!(h.opaque_count, 2);
        assert_eq!(h.r[10], 1);
        assert_eq!(h.r[200], 1);
        assert_eq!(h.r[20], 0, "alpha=0 should not contribute");
    }

    #[test]
    fn histogram_total_equals_opaque_count() {
        let mut buf = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            buf.extend_from_slice(&[
                (i % 256) as u8,
                ((i * 2) % 256) as u8,
                ((i * 3) % 256) as u8,
                255,
            ]);
        }
        let h = compute_histogram(bytemuck::cast_slice(&buf));
        assert_eq!(h.opaque_count, 64);
        let r_total: u32 = h.r.iter().sum();
        let g_total: u32 = h.g.iter().sum();
        let b_total: u32 = h.b.iter().sum();
        let l_total: u32 = h.l.iter().sum();
        assert_eq!(r_total, 64);
        assert_eq!(g_total, 64);
        assert_eq!(b_total, 64);
        assert_eq!(l_total, 64);
    }

    #[test]
    fn auto_levels_stretches_compressed_range() {
        // Build a buffer whose R channel only occupies [80, 180].
        let mut buf = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32u32 {
            for x in 0..32u32 {
                let r = 80u8 + (((x + y) % 100) as u8);
                buf.extend_from_slice(&[r, 128, 128, 255]);
            }
        }
        auto_levels(&mut buf);
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in buf.chunks_exact(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        // R channel now spans close to full range.
        assert!(lo <= 10, "auto_levels did not pull min down (got {lo})");
        assert!(hi >= 245, "auto_levels did not push max up (got {hi})");
    }

    #[test]
    fn auto_colors_preserves_uniform_distribution() {
        // Build a 64×64 image where each channel hits every value 0..255
        // multiple times (uniform distribution). 1 % cutoff (40 pixels at
        // each tail) won't shift min/max past 0 / 255, so auto_colors is
        // effectively identity.
        let mut buf = Vec::with_capacity(64 * 64 * 4);
        for i in 0..(64 * 64) {
            let v = (i % 256) as u8;
            buf.extend_from_slice(&[v, v, v, 255]);
        }
        let before = buf.clone();
        auto_colors(&mut buf);
        let mut max_drift = 0u8;
        for (a, b) in buf.iter().zip(before.iter()) {
            max_drift = max_drift.max(a.abs_diff(*b));
        }
        assert!(
            max_drift <= 2,
            "auto_colors on uniform-distribution buffer drifted by {max_drift}"
        );
    }

    #[test]
    fn auto_contrast_lifts_compressed_lightness() {
        // All pixels at L ≈ 0.4..0.6 (compressed mid-range).
        let mut buf = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let v = 102u8 + (((x + y) % 50) as u8); // 102..152 ≈ L 0.4..0.6
                buf.extend_from_slice(&[v, v, v, 255]);
            }
        }
        auto_contrast(&mut buf);
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in buf.chunks_exact(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        assert!(
            hi as i32 - lo as i32 >= 100,
            "auto_contrast did not stretch"
        );
    }

    #[test]
    fn auto_levels_skips_transparent_pixels() {
        let mut buf = vec![10u8, 10, 10, 0, 50, 80, 120, 255, 200, 220, 240, 255];
        auto_levels(&mut buf);
        assert_eq!(&buf[0..4], &[10, 10, 10, 0]); // untouched
    }

    #[test]
    fn percentile_range_finds_endpoints() {
        let mut hist = [0u32; 256];
        hist[10] = 100;
        hist[200] = 100;
        let (lo, hi) = percentile_range(&hist, 200, 0.005);
        // 0.5 % of 200 = 1, so cutoff lifts past index 10's 100 → lo = 10.
        assert_eq!(lo, 10);
        assert_eq!(hi, 200);
    }
}
