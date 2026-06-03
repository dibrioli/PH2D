//! Stage 2 — tonal pipeline (Phase 1).
//!
//! Each stage primitive operates on a **linear-sRGB triple `[R, G, B]`** (or
//! **OKLab triple `[L, a, b]`** for the Vibrance / Saturation pair). They are
//! `pub` so tests + future WGSL parity work can call them in isolation; the
//! production path is [`adjust_tonal`], which fuses them inside one
//! `sRGB → linear → … → linear → sRGB` round-trip per pixel.

use crate::color_utils::{
    bradford_matrix_for_kelvin, linear_rgb_to_oklab, linear_to_srgb_u8, mat3_mul_vec,
    oklab_to_linear_rgb, srgb_to_linear_u8,
};
use crate::params::ColorEqualizationParams;

/// Apply exposure (EV stops) in linear-light sRGB. `m = pow(2, ev)` followed
/// by a soft-knee highlight compression above `0.8` that gradually rolls
/// values off toward `1.0` instead of hard-clipping. `ev` is the EV stop
/// count (`-3..+3`); `0` is identity.
pub fn apply_exposure_linear(rgb: &mut [f32; 3], ev: f32) {
    if ev == 0.0 {
        return;
    }
    let m = (2.0_f32).powf(ev);
    for c in rgb.iter_mut() {
        let v = *c * m;
        *c = soft_knee(v);
    }
}

/// Soft-knee compression above `0.8`: linear identity below the knee, then
/// `0.8 + 0.2 · (1 − exp(-(v - 0.8) · 2))` above. Asymptotic to `1.0` —
/// prevents harsh hard-clip on bright values. Mirrors the legacy `softKnee`.
fn soft_knee(v: f32) -> f32 {
    if v <= 0.8 {
        v
    } else {
        0.8 + 0.2 * (1.0 - (-(v - 0.8) * 2.0).exp())
    }
}

/// Apply pre-computed Bradford temperature matrix in linear sRGB. Build the
/// matrix once outside the per-pixel loop via
/// [`crate::color_utils::bradford_matrix_for_kelvin`] using
/// [`temperature01_to_kelvin`] to project a `-1..+1` slider value onto the
/// `2000K..10000K` target range (photographer convention: positive = warm).
pub fn apply_temperature_linear(rgb: &mut [f32; 3], matrix: &[f32; 9]) {
    let out = mat3_mul_vec(matrix, *rgb);
    *rgb = out;
}

/// Map the `-1..+1` slider value onto a target Kelvin for the Bradford
/// adaptation. Photographer convention: positive = warm (low Kelvin / orange
/// cast), negative = cool (high Kelvin / blue cast); `0` = D65 neutral.
pub fn temperature01_to_kelvin(t: f32) -> f32 {
    let t = t.clamp(-1.0, 1.0);
    if t >= 0.0 {
        // 0 → 6500K (D65); +1 → 2000K (tungsten, warm).
        6500.0 - (6500.0 - 2000.0) * t
    } else {
        // 0 → 6500K; -1 → 10000K (overcast / blue).
        6500.0 + (10000.0 - 6500.0) * (-t)
    }
}

/// Apply tint (green ↔ magenta) in linear-light sRGB. `tint ∈ [-1, +1]`:
/// positive shifts toward magenta (drops G, lifts R/B in luminance-preserving
/// proportions); negative shifts toward green. Mirrors the legacy `applyTint`.
pub fn apply_tint_linear(rgb: &mut [f32; 3], tint: f32) {
    if tint == 0.0 {
        return;
    }
    let t = tint.clamp(-1.0, 1.0);
    // Green shifts; R/B counter-shift weighted by their luminance
    // contribution so the overall Y stays roughly constant (BT.709).
    let g_shift = -t * 0.05;
    let r_comp = t * 0.05 * 0.7152 / 0.2126;
    let b_comp = t * 0.05 * 0.7152 / 0.0722;
    rgb[0] *= 1.0 + r_comp;
    rgb[1] *= 1.0 + g_shift;
    rgb[2] *= 1.0 + b_comp;
}

/// Apply brightness in linear-light sRGB. `brightness ∈ [-1, +1]` — applied
/// multiplicatively as `m = 1 + brightness`, so `0` is identity, `-1`
/// collapses to black, `+1` doubles. Multiplicative (not additive) preserves
/// blacks: a pure-black pixel stays black instead of being lifted to grey.
/// Mirrors the legacy `applyBrightness`.
pub fn apply_brightness_linear(rgb: &mut [f32; 3], brightness: f32) {
    if brightness == 0.0 {
        return;
    }
    let m = 1.0 + brightness.clamp(-1.0, 1.0);
    for c in rgb.iter_mut() {
        *c *= m;
    }
}

/// Apply contrast in linear-light sRGB with an S-curve around the
/// perceptual midpoint (`0.18`, "18 % grey"). `contrast ∈ [0.5, 2.0]`,
/// `1.0` is identity. Above `1.0` steepens midtones (S-curve); below `1.0`
/// flattens them. Mirrors the legacy `applyContrast` (more nuanced than a
/// simple multiply around 0.5 — preserves shadows).
pub fn apply_contrast_linear(rgb: &mut [f32; 3], contrast: f32) {
    if (contrast - 1.0).abs() < f32::EPSILON {
        return;
    }
    let strength = (contrast.clamp(0.5, 2.0) - 1.0) * 2.0;
    let pivot = 0.18;
    for c in rgb.iter_mut() {
        let centered = *c - pivot;
        let sign = if centered >= 0.0 { 1.0 } else { -1.0 };
        let abs = centered.abs();
        let curved = if contrast > 1.0 {
            abs * (1.0 + strength * (1.0 - abs))
        } else {
            abs * (1.0 + strength * abs)
        };
        *c = (pivot + sign * curved).clamp(0.0, 1.0);
    }
}

/// Apply vibrance (smart saturation) in OKLab. `vibrance ∈ [-1, +1]`:
/// boosts chroma INVERSELY proportional to current chroma, so already-vivid
/// regions (skin tones, sky) get less boost than muted regions. The
/// chroma-norm threshold `0.15` matches the legacy reference. Mirrors
/// `applyVibrance` (without explicit `cos(hue)` / `sin(hue)` — chroma
/// scaling preserves hue trivially when both `a` and `b` are scaled).
pub fn apply_vibrance_oklab(lab: &mut [f32; 3], vibrance: f32) {
    if vibrance == 0.0 {
        return;
    }
    let vn = vibrance.clamp(-1.0, 1.0);
    let chroma = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    if chroma <= 0.0 {
        return;
    }
    // Skin-tone protection: less boost when chroma is already high.
    let chroma_norm = (chroma / 0.15).min(1.0);
    let boost = vn * (1.0 - chroma_norm * chroma_norm);
    let factor = (1.0 + boost).max(0.0);
    lab[1] *= factor;
    lab[2] *= factor;
}

/// Apply uniform saturation in OKLab. `saturation ∈ [-1, +1]`, `0` is
/// identity; `-1` desaturates fully (grayscale), `+1` doubles chroma.
/// Mirrors the legacy `applySaturation` in OKLab. Scales `a` and `b`
/// directly (= scaling chroma while keeping hue, since `a + ib = chroma · e^(iθ)`).
pub fn apply_saturation_oklab(lab: &mut [f32; 3], saturation: f32) {
    if saturation == 0.0 {
        return;
    }
    let sat_mult = (1.0 + saturation.clamp(-1.0, 1.0)).max(0.0);
    lab[1] *= sat_mult;
    lab[2] *= sat_mult;
}

/// Apply the full Phase 1 tonal pipeline in place over straight-alpha
/// RGBA8. Performs ONE sRGB → linear and (when Vibrance/Saturation
/// non-identity) ONE OKLab round-trip per pixel — instead of cascading
/// each stage with its own conversion. Transparent pixels (`alpha == 0`)
/// are skipped (RGB undefined per straight-alpha convention).
///
/// Order matches the legacy reference: Exposure → Temperature → Tint →
/// Brightness → Contrast → Vibrance → Saturation. Stage primitives are
/// also exposed `pub` for standalone tests.
pub fn adjust_tonal(rgba: &mut [u8], params: &ColorEqualizationParams) {
    // Precompute the Bradford temperature matrix once outside the per-pixel
    // loop — it depends only on the target Kelvin.
    let temp_matrix: Option<[f32; 9]> = if params.temperature != 0.0 {
        Some(bradford_matrix_for_kelvin(temperature01_to_kelvin(
            params.temperature,
        )))
    } else {
        None
    };
    let needs_oklab = params.vibrance != 0.0 || params.saturation != 0.0;

    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let mut rgb = [
            srgb_to_linear_u8(px[0]),
            srgb_to_linear_u8(px[1]),
            srgb_to_linear_u8(px[2]),
        ];

        // Linear-sRGB stages.
        apply_exposure_linear(&mut rgb, params.exposure);
        if let Some(ref m) = temp_matrix {
            apply_temperature_linear(&mut rgb, m);
        }
        apply_tint_linear(&mut rgb, params.tint);
        apply_brightness_linear(&mut rgb, params.brightness);
        apply_contrast_linear(&mut rgb, params.contrast);

        // OKLab stages — single conversion for the pair.
        if needs_oklab {
            let mut lab = linear_rgb_to_oklab(rgb[0], rgb[1], rgb[2]);
            apply_vibrance_oklab(&mut lab, params.vibrance);
            apply_saturation_oklab(&mut lab, params.saturation);
            rgb = oklab_to_linear_rgb(lab[0], lab[1], lab[2]);
        }

        px[0] = linear_to_srgb_u8(rgb[0]);
        px[1] = linear_to_srgb_u8(rgb[1]);
        px[2] = linear_to_srgb_u8(rgb[2]);
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

    // ── Phase 1 — tonal stage primitives ─────────────────────────────

    #[test]
    fn exposure_zero_is_identity() {
        let mut rgb = [0.3_f32, 0.5, 0.7];
        let before = rgb;
        apply_exposure_linear(&mut rgb, 0.0);
        assert_eq!(rgb, before);
    }

    #[test]
    fn exposure_plus_one_ev_doubles_below_knee() {
        // 0.3 doubles to 0.6 (below soft-knee threshold of 0.8 → no
        // compression).
        let mut rgb = [0.3_f32, 0.3, 0.3];
        apply_exposure_linear(&mut rgb, 1.0);
        for c in rgb {
            assert!((c - 0.6).abs() < 1e-5, "got {c}");
        }
    }

    #[test]
    fn exposure_soft_knee_caps_below_one() {
        // 0.6 × 2^2 = 2.4 — would clip hard to 1.0 without soft knee.
        // Soft knee: 0.8 + 0.2·(1 - exp(-3.2)) ≈ 0.8 + 0.2·0.959 ≈ 0.99.
        let mut rgb = [0.6_f32, 0.6, 0.6];
        apply_exposure_linear(&mut rgb, 2.0);
        for c in rgb {
            assert!(
                c < 1.0 && c > 0.97,
                "soft knee should approach 1 from below: got {c}"
            );
        }
    }

    #[test]
    fn temperature_zero_is_identity_within_floats() {
        // Bradford D65→D65 is near-identity; with input == output passing
        // through `apply_temperature_linear` with the identity matrix
        // should hardly move the pixel.
        let m = bradford_matrix_for_kelvin(6500.0);
        let mut rgb = [0.4_f32, 0.55, 0.7];
        let before = rgb;
        apply_temperature_linear(&mut rgb, &m);
        for i in 0..3 {
            assert!(
                (rgb[i] - before[i]).abs() < 0.02,
                "channel {i} drifted: before {} after {}",
                before[i],
                rgb[i]
            );
        }
    }

    #[test]
    fn temperature_warm_lifts_red() {
        // Positive temperature (warm) — apply on neutral grey, R should
        // rise, B should drop.
        let m = bradford_matrix_for_kelvin(temperature01_to_kelvin(0.7));
        let mut rgb = [0.5_f32, 0.5, 0.5];
        apply_temperature_linear(&mut rgb, &m);
        assert!(rgb[0] > 0.5, "warm should boost R: got {}", rgb[0]);
        assert!(rgb[2] < 0.5, "warm should drop B: got {}", rgb[2]);
    }

    #[test]
    fn temperature_cool_lifts_blue() {
        let m = bradford_matrix_for_kelvin(temperature01_to_kelvin(-0.7));
        let mut rgb = [0.5_f32, 0.5, 0.5];
        apply_temperature_linear(&mut rgb, &m);
        assert!(rgb[0] < 0.5, "cool should drop R: got {}", rgb[0]);
        assert!(rgb[2] > 0.5, "cool should lift B: got {}", rgb[2]);
    }

    #[test]
    fn tint_positive_shifts_toward_magenta() {
        // Tint > 0 drops G and lifts R/B in compensation.
        let mut rgb = [0.5_f32, 0.5, 0.5];
        apply_tint_linear(&mut rgb, 0.5);
        assert!(rgb[1] < 0.5, "G should drop with magenta tint");
        assert!(rgb[0] > 0.5, "R should rise with magenta tint");
        assert!(rgb[2] > 0.5, "B should rise with magenta tint");
    }

    #[test]
    fn brightness_multiplicative_preserves_black() {
        // Critical legacy semantic: brightness is multiplicative, so pure
        // black stays pure black (no lift to mid-grey).
        let mut rgb = [0.0_f32, 0.0, 0.0];
        apply_brightness_linear(&mut rgb, 0.8);
        assert_eq!(rgb, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn brightness_lifts_midtones() {
        let mut rgb = [0.3_f32, 0.3, 0.3];
        apply_brightness_linear(&mut rgb, 0.5);
        // m = 1.5 → 0.3 × 1.5 = 0.45.
        for c in rgb {
            assert!((c - 0.45).abs() < 1e-5);
        }
    }

    #[test]
    fn contrast_one_is_identity() {
        let mut rgb = [0.3_f32, 0.5, 0.7];
        let before = rgb;
        apply_contrast_linear(&mut rgb, 1.0);
        assert_eq!(rgb, before);
    }

    #[test]
    fn contrast_above_one_pushes_pixels_away_from_pivot() {
        // Pivot is 0.18; contrast > 1 pushes pixels above pivot UP and
        // pixels below pivot DOWN.
        let mut above = [0.5_f32, 0.5, 0.5];
        apply_contrast_linear(&mut above, 1.5);
        assert!(above[0] > 0.5, "pixel above pivot should rise: {above:?}");

        let mut below = [0.1_f32, 0.1, 0.1];
        apply_contrast_linear(&mut below, 1.5);
        assert!(below[0] < 0.1, "pixel below pivot should drop: {below:?}");
    }

    #[test]
    fn vibrance_zero_is_identity() {
        let mut lab = [0.5_f32, 0.1, 0.05];
        let before = lab;
        apply_vibrance_oklab(&mut lab, 0.0);
        assert_eq!(lab, before);
    }

    #[test]
    fn vibrance_boosts_low_chroma_more_than_high_chroma() {
        // Two pixels: low chroma (a=0.02, b=0.01) and high chroma
        // (a=0.20, b=0.10). Same vibrance value (+0.5). The low-chroma
        // pixel should gain proportionally more.
        let mut low = [0.5_f32, 0.02, 0.01];
        let mut hi = [0.5_f32, 0.20, 0.10];
        let chroma_low_before = (low[1].powi(2) + low[2].powi(2)).sqrt();
        let chroma_hi_before = (hi[1].powi(2) + hi[2].powi(2)).sqrt();
        apply_vibrance_oklab(&mut low, 0.5);
        apply_vibrance_oklab(&mut hi, 0.5);
        let chroma_low_after = (low[1].powi(2) + low[2].powi(2)).sqrt();
        let chroma_hi_after = (hi[1].powi(2) + hi[2].powi(2)).sqrt();
        let ratio_low = chroma_low_after / chroma_low_before;
        let ratio_hi = chroma_hi_after / chroma_hi_before;
        assert!(
            ratio_low > ratio_hi,
            "low-chroma pixel should grow more (got {ratio_low} vs {ratio_hi})"
        );
    }

    #[test]
    fn saturation_minus_one_zeroes_chroma() {
        let mut lab = [0.5_f32, 0.2, -0.1];
        apply_saturation_oklab(&mut lab, -1.0);
        // sat_mult = 0 → chroma collapses to 0.
        assert!(lab[1].abs() < 1e-6);
        assert!(lab[2].abs() < 1e-6);
    }

    #[test]
    fn saturation_plus_one_doubles_chroma() {
        let mut lab = [0.5_f32, 0.1, -0.05];
        apply_saturation_oklab(&mut lab, 1.0);
        // sat_mult = 2 → chroma doubles.
        assert!((lab[1] - 0.2).abs() < 1e-6);
        assert!((lab[2] - -0.1).abs() < 1e-6);
    }

    #[test]
    fn temperature01_to_kelvin_endpoints() {
        assert_eq!(temperature01_to_kelvin(0.0), 6500.0);
        // Photographer convention: +1 → warm = low Kelvin.
        assert_eq!(temperature01_to_kelvin(1.0), 2000.0);
        assert_eq!(temperature01_to_kelvin(-1.0), 10000.0);
        // Out-of-range clamps.
        assert_eq!(temperature01_to_kelvin(99.0), 2000.0);
        assert_eq!(temperature01_to_kelvin(-99.0), 10000.0);
    }

    // ── adjust_tonal (combined batch) ────────────────────────────────

    #[test]
    fn adjust_tonal_identity_leaves_pixels_within_one_lsb() {
        let mut buf = solid(4, 4, [60, 130, 200]);
        let before = buf.clone();
        adjust_tonal(&mut buf, &ColorEqualizationParams::default());
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 1, "drift {a} vs {b}");
        }
    }

    #[test]
    fn adjust_tonal_brightness_preserves_black() {
        // Critical: multiplicative brightness MUST keep pure black at 0.
        let mut buf = vec![0u8, 0, 0, 255];
        let p = ColorEqualizationParams {
            brightness: 0.8,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        assert_eq!(&buf[..3], &[0, 0, 0]);
    }

    #[test]
    fn adjust_tonal_saturation_minus_one_grayscales() {
        let mut buf = solid(4, 4, [200, 50, 50]);
        let p = ColorEqualizationParams {
            saturation: -1.0,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        let r = buf[0];
        let g = buf[1];
        let b = buf[2];
        // OKLab's perceptual luma differs from BT.709, so the grey value
        // won't exactly match input R; just assert channels collapsed to
        // the same value (within 2 LSB given OKLab cube-root rounding).
        assert!(r.abs_diff(g) <= 2, "R/G drift after desat: {r} vs {g}");
        assert!(g.abs_diff(b) <= 2, "G/B drift after desat: {g} vs {b}");
    }

    #[test]
    fn adjust_tonal_skips_transparent_pixels() {
        let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
        let p = ColorEqualizationParams {
            brightness: 0.5,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
        assert!(buf[4] != 100, "opaque pixel should have been adjusted");
    }

    #[test]
    fn adjust_tonal_exposure_brightens() {
        let mut buf = solid(4, 4, [80, 80, 80]);
        let p = ColorEqualizationParams {
            exposure: 1.0, // +1 EV stop
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        assert!(buf[0] > 80, "+1 EV should brighten (got {})", buf[0]);
    }

    #[test]
    fn adjust_tonal_vibrance_increases_chroma() {
        let mut buf = solid(4, 4, [120, 100, 100]); // very mild red cast
        let before_r = buf[0] as i32;
        let p = ColorEqualizationParams {
            vibrance: 1.0,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        // Low-chroma input → vibrance pumps it up — R should now be
        // visibly higher than G/B.
        assert!(
            (buf[0] as i32 - before_r) > 5,
            "vibrance did not pump low chroma: got delta {}",
            buf[0] as i32 - before_r
        );
    }
}
