//! sRGB ↔ OKLab color space conversion (Björn Ottosson, 2020).
//!
//! Why OKLab and not RGB Euclidean: distance in OKLab approximates
//! perceptual difference. Two pinks that look identical have small
//! OKLab distance even when their RGB Euclidean distance is large
//! (and vice versa for two distant grays). This makes `tolerance`
//! intuitive across hues and lightness ranges — the legacy engine's
//! RGB-distance model required different tolerance values for each
//! sprite.
//!
//! Reference: <https://bottosson.github.io/posts/oklab/>

/// One OKLab sample (linear lightness + perceptual chroma a/b).
#[derive(Copy, Clone, Debug, Default)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

/// sRGB byte → linear-light f32 (0..=1). Uses the standard sRGB EOTF
/// (gamma 2.4 piecewise, linear ramp under 0.04045).
#[inline]
fn srgb_to_linear(c: u8) -> f32 {
    let x = c as f32 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB byte triplet → OKLab. Pure function, branch-free past the
/// EOTF lookup.
#[inline]
pub fn srgb_to_oklab(r: u8, g: u8, b: u8) -> Oklab {
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    // Linear sRGB → LMS (Ottosson §3 matrix M1).
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

    let l = l.cbrt();
    let m = m.cbrt();
    let s = s.cbrt();

    // LMS' → OKLab (matrix M2).
    Oklab {
        l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        b: 0.025_904_037 * l + 0.782_771_8 * m - 0.808_675_77 * s,
    }
}

/// Squared Euclidean distance in OKLab. Squared form is enough for
/// thresholding (avoids the sqrt per pixel) — callers compare against
/// `tol * tol` instead of `tol`.
#[inline]
pub fn oklab_dist_sq(a: Oklab, b: Oklab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    dl * dl + da * da + db * db
}

/// Maps the user-facing `tolerance` slider (0..=100) to a squared
/// OKLab distance threshold. The constant 0.012 was picked so that
/// tolerance=30 (legacy default) cleanly separates near-white from
/// near-light-gray in OKLab — matches the legacy "feels right"
/// baseline for cartoon sprites with off-white backgrounds.
#[inline]
pub fn tolerance_to_oklab_threshold_sq(tolerance: f32) -> f32 {
    let t = tolerance.clamp(0.0, 100.0) / 100.0;
    // Quadratic ramp — small tolerances stay tight, large ones grow
    // visibly. `0.35` is the upper-bound OKLab radius beyond which
    // most distinct colors fall inside (empirically calibrated).
    let r = t * 0.35;
    r * r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn black_maps_to_zero_lightness() {
        let c = srgb_to_oklab(0, 0, 0);
        assert!(approx_eq(c.l, 0.0, 1e-4));
        assert!(approx_eq(c.a, 0.0, 1e-3));
        assert!(approx_eq(c.b, 0.0, 1e-3));
    }

    #[test]
    fn white_maps_to_unit_lightness() {
        let c = srgb_to_oklab(255, 255, 255);
        // OKLab L for sRGB white is exactly 1.
        assert!(approx_eq(c.l, 1.0, 1e-3));
        assert!(approx_eq(c.a, 0.0, 1e-3));
        assert!(approx_eq(c.b, 0.0, 1e-3));
    }

    #[test]
    fn perceptually_close_pinks_have_small_distance() {
        // Two near-pink samples that legacy RGB Euclidean would
        // separate but OKLab keeps close.
        let pink_a = srgb_to_oklab(240, 180, 200);
        let pink_b = srgb_to_oklab(235, 175, 195);
        let d = oklab_dist_sq(pink_a, pink_b);
        // ~0.0001..0.005 — well inside the tolerance=30 threshold.
        assert!(d < tolerance_to_oklab_threshold_sq(30.0));
    }

    #[test]
    fn perceptually_far_pinks_exceed_tolerance() {
        let pink = srgb_to_oklab(240, 180, 200);
        let cyan = srgb_to_oklab(60, 200, 220);
        let d = oklab_dist_sq(pink, cyan);
        assert!(d > tolerance_to_oklab_threshold_sq(30.0));
    }

    #[test]
    fn tolerance_threshold_monotonic_in_tolerance() {
        let t0 = tolerance_to_oklab_threshold_sq(0.0);
        let t30 = tolerance_to_oklab_threshold_sq(30.0);
        let t100 = tolerance_to_oklab_threshold_sq(100.0);
        assert!(t0 <= t30);
        assert!(t30 < t100);
        assert_eq!(t0, 0.0);
    }

    #[test]
    fn tolerance_threshold_clamps_negative() {
        assert_eq!(tolerance_to_oklab_threshold_sq(-50.0), 0.0);
    }

    #[test]
    fn tolerance_threshold_clamps_above_100() {
        assert_eq!(
            tolerance_to_oklab_threshold_sq(500.0),
            tolerance_to_oklab_threshold_sq(100.0)
        );
    }
}
