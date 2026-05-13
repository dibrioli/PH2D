//! Color-key mask in OKLab space with soft smootherstep edges.
//!
//! Per-pixel: convert RGB → OKLab, find minimum squared distance to
//! any target color (sampled or k-means auto-detected), compare
//! against `tolerance_to_oklab_threshold_sq`. Inside the soft band,
//! falloff is smootherstep (5th-order, zero derivatives at the ends)
//! → cleaner edges than the legacy linear ramp.
//!
//! Mask convention: `0.0` = background (transparent), `1.0` =
//! foreground (opaque).

use super::oklab::{Oklab, oklab_dist_sq, srgb_to_oklab, tolerance_to_oklab_threshold_sq};
use super::params::RgbColor;

/// Fill `mask` (length `w*h`) with the color-key result.
///
/// `targets` must be non-empty. Caller pre-allocates `mask` —
/// satisfies HR-3 in tight loops.
pub fn colorkey_mask(
    rgba: &[u8],
    w: u32,
    h: u32,
    tolerance: f32,
    targets: &[RgbColor],
    mask: &mut [f32],
) {
    let total = (w as usize) * (h as usize);
    debug_assert_eq!(mask.len(), total);
    debug_assert!(!targets.is_empty(), "colorkey needs ≥1 target");

    let tol_sq = tolerance_to_oklab_threshold_sq(tolerance);
    let half_sq = tol_sq * 0.25; // (tol * 0.5)² in distance² space

    // Pre-convert targets to OKLab once.
    let mut tgt: [Oklab; 16] = [Oklab::default(); 16];
    let n_tgt = targets.len().min(16);
    for (i, t) in targets.iter().take(16).enumerate() {
        tgt[i] = srgb_to_oklab(t.r, t.g, t.b);
    }

    for (i, m) in mask.iter_mut().enumerate().take(total) {
        let idx = i * 4;
        let a = rgba[idx + 3];
        // Already transparent → keep as background.
        if a < 10 {
            *m = 0.0;
            continue;
        }
        let r = rgba[idx];
        let g = rgba[idx + 1];
        let b = rgba[idx + 2];
        let lab = srgb_to_oklab(r, g, b);

        let mut min_d = f32::INFINITY;
        for c in tgt.iter().take(n_tgt) {
            let d = oklab_dist_sq(lab, *c);
            if d < min_d {
                min_d = d;
            }
        }

        if min_d <= half_sq {
            // `<=` so that an exact-match (min_d == 0) with
            // tolerance == 0 still classifies as background.
            *m = 0.0;
        } else if min_d < tol_sq {
            // Smootherstep falloff in the soft band, 5th-order.
            let t = (min_d - half_sq) / (tol_sq - half_sq);
            *m = smootherstep(t);
        } else {
            *m = 1.0;
        }
    }
}

/// 5th-order smoothstep (zero 1st and 2nd derivatives at endpoints).
/// Less banding than the legacy 3rd-order `smoothstep` when alpha is
/// inspected at high zoom.
#[inline]
fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4×4 sprite: 2-px white border, 2×2 red center.
    fn red_on_white() -> (Vec<u8>, u32, u32) {
        let (w, h) = (4u32, 4u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let idx = (y * w as usize + x) * 4;
                let is_center = (1..=2).contains(&x) && (1..=2).contains(&y);
                if is_center {
                    buf[idx] = 220;
                    buf[idx + 1] = 30;
                    buf[idx + 2] = 30;
                } else {
                    buf[idx] = 255;
                    buf[idx + 1] = 255;
                    buf[idx + 2] = 255;
                }
                buf[idx + 3] = 255;
            }
        }
        (buf, w, h)
    }

    #[test]
    fn background_pixels_get_zero_mask() {
        let (img, w, h) = red_on_white();
        let mut mask = vec![0.0; (w * h) as usize];
        colorkey_mask(&img, w, h, 30.0, &[RgbColor::new(255, 255, 255)], &mut mask);
        // Corner pixel (0,0) is white = background.
        assert_eq!(mask[0], 0.0);
        // Center pixel (1,1) is red = foreground.
        let center = (w + 1) as usize;
        assert!(mask[center] > 0.9);
    }

    #[test]
    fn transparent_pixels_stay_transparent() {
        let (w, h) = (2u32, 2u32);
        let mut img = vec![0u8; (w * h * 4) as usize];
        // First pixel transparent, second opaque red.
        img[3] = 0;
        img[4] = 255;
        img[5] = 0;
        img[6] = 0;
        img[7] = 255;
        let mut mask = vec![0.0; (w * h) as usize];
        colorkey_mask(&img, w, h, 30.0, &[RgbColor::new(255, 255, 255)], &mut mask);
        assert_eq!(mask[0], 0.0); // already transparent
        assert!(mask[1] > 0.9); // red foreground
    }

    #[test]
    fn zero_tolerance_keeps_only_exact_matches() {
        let (img, w, h) = red_on_white();
        let mut mask = vec![0.0; (w * h) as usize];
        colorkey_mask(&img, w, h, 0.0, &[RgbColor::new(255, 255, 255)], &mut mask);
        // With zero tolerance, exact white drops to 0; red stays 1.
        assert_eq!(mask[0], 0.0);
        let center = (w + 1) as usize;
        assert_eq!(mask[center], 1.0);
    }

    #[test]
    fn higher_tolerance_widens_soft_band_for_near_target_pixels() {
        // Near-white (245, 245, 245) should fall into the soft band
        // at tolerance 30 (mask between 0 and 1) — and into the hard
        // background region at tolerance 100.
        let img = vec![245u8, 245, 245, 255];
        let mut mask_low = vec![0.5; 1];
        let mut mask_high = vec![0.5; 1];
        colorkey_mask(
            &img,
            1,
            1,
            30.0,
            &[RgbColor::new(255, 255, 255)],
            &mut mask_low,
        );
        colorkey_mask(
            &img,
            1,
            1,
            100.0,
            &[RgbColor::new(255, 255, 255)],
            &mut mask_high,
        );
        // Higher tolerance pushes the same pixel toward 0 (background).
        assert!(mask_high[0] <= mask_low[0]);
    }

    #[test]
    fn multi_target_picks_closest() {
        // Pixel is red. Targets: white + red. Should classify as bg
        // because red target is closer.
        let img = vec![220, 30, 30, 255];
        let mut mask = vec![1.0; 1];
        colorkey_mask(
            &img,
            1,
            1,
            30.0,
            &[RgbColor::new(255, 255, 255), RgbColor::new(220, 30, 30)],
            &mut mask,
        );
        assert!(mask[0] < 0.1);
    }
}
