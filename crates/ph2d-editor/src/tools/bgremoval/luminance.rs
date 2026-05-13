//! Luminance-difference mask.
//!
//! Good for monochromatic backgrounds (paper texture, single-tone
//! studio). Cheaper than `colorkey` (no per-pixel OKLab) and often
//! enough.
//!
//! Mask convention: `0.0` = background, `1.0` = foreground.

/// BT.601 luminance of an sRGB byte triplet, normalized to 0..=1.
#[inline]
fn luma_bt601(r: u8, g: u8, b: u8) -> f32 {
    (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0
}

/// 5th-order smoothstep used in the soft band (same curve as
/// `colorkey::smootherstep`, kept local to avoid a public module
/// dependency for one tiny inlined function).
#[inline]
fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Compute the average BT.601 luminance of the image border band.
/// Sampled cheaply — single pass over edge pixels, skip transparent.
pub fn detect_border_luminance(rgba: &[u8], w: u32, h: u32) -> f32 {
    let (w, h) = (w as usize, h as usize);
    let border = ((w.min(h) as f32 * 0.02) as usize).clamp(2, 10);
    let mut sum = 0.0_f32;
    let mut count = 0u32;

    let mut add = |x: usize, y: usize| {
        let idx = (y * w + x) * 4;
        if idx + 3 >= rgba.len() {
            return;
        }
        let a = rgba[idx + 3];
        if a < 200 {
            return;
        }
        sum += luma_bt601(rgba[idx], rgba[idx + 1], rgba[idx + 2]);
        count += 1;
    };

    for x in 0..w {
        for dy in 0..border {
            add(x, dy);
            if h > dy {
                add(x, h - 1 - dy);
            }
        }
    }
    for y in border..h.saturating_sub(border) {
        for dx in 0..border {
            add(dx, y);
            if w > dx {
                add(w - 1 - dx, y);
            }
        }
    }

    if count == 0 { 1.0 } else { sum / count as f32 }
}

/// Fill `mask` with the luminance-difference result.
pub fn luminance_mask(rgba: &[u8], w: u32, h: u32, tolerance: f32, mask: &mut [f32]) {
    let total = (w as usize) * (h as usize);
    debug_assert_eq!(mask.len(), total);

    let bg_lum = detect_border_luminance(rgba, w, h);
    let tol = tolerance.clamp(0.0, 100.0) / 100.0;
    let half = tol * 0.5;

    for (i, m) in mask.iter_mut().enumerate().take(total) {
        let idx = i * 4;
        let a = rgba[idx + 3];
        if a < 10 {
            *m = 0.0;
            continue;
        }
        let lum = luma_bt601(rgba[idx], rgba[idx + 1], rgba[idx + 2]);
        let diff = (lum - bg_lum).abs();

        if diff <= half {
            // `<=` for exact-match with tolerance=0 case (see
            // `colorkey::colorkey_mask` for the same reasoning).
            *m = 0.0;
        } else if diff < tol {
            let t = (diff - half) / (tol - half).max(f32::EPSILON);
            *m = smootherstep(t);
        } else {
            *m = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_background_detects_high_luminance() {
        let mut buf = vec![0u8; 16 * 16 * 4];
        for i in 0..(16 * 16) {
            buf[i * 4] = 255;
            buf[i * 4 + 1] = 255;
            buf[i * 4 + 2] = 255;
            buf[i * 4 + 3] = 255;
        }
        let lum = detect_border_luminance(&buf, 16, 16);
        assert!((lum - 1.0).abs() < 0.01);
    }

    #[test]
    fn black_background_detects_zero_luminance() {
        let mut buf = vec![0u8; 16 * 16 * 4];
        for i in 0..(16 * 16) {
            buf[i * 4 + 3] = 255;
        }
        let lum = detect_border_luminance(&buf, 16, 16);
        assert!(lum < 0.01);
    }

    #[test]
    fn transparent_border_falls_back_to_one() {
        let buf = vec![0u8; 16 * 16 * 4];
        let lum = detect_border_luminance(&buf, 16, 16);
        assert_eq!(lum, 1.0);
    }

    #[test]
    fn dark_subject_on_light_bg_gets_full_alpha() {
        // 8×8 white image with black 4×4 center.
        let (w, h) = (8u32, 8u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            buf[i * 4 + 3] = 255;
        }
        for y in 2..6 {
            for x in 2..6 {
                let idx = (y * w as usize + x) * 4;
                buf[idx] = 0;
                buf[idx + 1] = 0;
                buf[idx + 2] = 0;
            }
        }
        let mut mask = vec![0.0; (w * h) as usize];
        luminance_mask(&buf, w, h, 30.0, &mut mask);
        // Corner = background.
        assert!(mask[0] < 0.1);
        // Center = foreground.
        let center = (3 * w + 3) as usize;
        assert!(mask[center] > 0.9);
    }

    #[test]
    fn zero_tolerance_clamps_to_full_mask_above_zero_diff() {
        let mut buf = vec![255u8; 4 * 4 * 4];
        for i in 0..16 {
            buf[i * 4 + 3] = 255;
        }
        // One center pixel slightly off: (x=1, y=1) → linear index 5.
        let off_idx = (4 + 1) * 4;
        buf[off_idx] = 250;
        buf[off_idx + 1] = 250;
        buf[off_idx + 2] = 250;
        let mut mask = vec![0.0; 16];
        luminance_mask(&buf, 4, 4, 0.0, &mut mask);
        // Any non-zero diff with tolerance=0 falls into the else branch → 1.0.
        assert_eq!(mask[5], 1.0);
    }
}
