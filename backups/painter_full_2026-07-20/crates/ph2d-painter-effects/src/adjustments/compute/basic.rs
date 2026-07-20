//! Simple per-pixel adjustment kernels (no spatial neighbourhood, no bespoke
//! editor state): HSB / Exposure / Vibrance / Posterize / Threshold / Invert /
//! Brightness-Contrast / Photo Filter / Black & White. Split out of the former
//! monolithic `compute.rs` (pure move).

use super::*;

/// Exposure — `acc` is straight LINEAR f32 RGBA (alpha preserved). A photographic
/// exposure in stops (`exposure_ev`, a `2^ev` gain on linear light), then a
/// linear `offset` (lifts/drops the floor), then a `gamma_correction` applied as
/// a power in linear (stored as an offset from 1.0 so the all-zero `Default` is
/// an exact identity). Neutral `{0,0,0}` early-returns identity.
pub(crate) fn apply_exposure(p: &ExposureParams, acc: &mut [[f32; 4]]) {
    if p.exposure_ev == 0.0 && p.offset == 0.0 && p.gamma_correction == 0.0 {
        return;
    }
    let gain = 2.0_f32.powf(p.exposure_ev);
    let inv_gamma = 1.0 / (1.0 + p.gamma_correction).max(1e-3);
    // The whole chain is a 1-D function of the input channel — LUT it (the only
    // `powf` left is the per-call table build, not per pixel).
    let lut = build_lut(|v| (v * gain + p.offset).max(0.0).powf(inv_gamma));
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            *ch = sample_lut(&lut, *ch);
        }
    }
}

/// Vibrance — OKLab chroma scaling. `vibrance` boosts chroma MORE for
/// low-saturation pixels (and tapers off as a pixel approaches full chroma — the
/// "protect already-saturated colors / skin tones" behavior), while `saturation`
/// scales every pixel's chroma uniformly. Both `-1..1`, neutral 0. Rotation-free
/// (hue preserved) and gray-safe (zero chroma stays zero — no rainbow). Alpha
/// preserved; neutral early-returns identity.
pub(crate) fn apply_vibrance(p: &VibranceParams, acc: &mut [[f32; 4]]) {
    if p.vibrance == 0.0 && p.saturation == 0.0 {
        return;
    }
    let sat_mul = (1.0 + p.saturation).max(0.0);
    // OKLab chroma is ~0..0.4 for in-gamut sRGB; normalize against that so the
    // vibrance weight (1 - normalized_chroma) reads "how unsaturated is this".
    const CHROMA_NORM: f32 = 0.4;
    for px in acc.iter_mut() {
        let lab = OklabColor::from_linear(LinearRgba::new(px[0], px[1], px[2], 1.0));
        let chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();
        if chroma > 1e-6 {
            let nc = (chroma / CHROMA_NORM).min(1.0);
            let vib_mul = (1.0 + p.vibrance * (1.0 - nc)).max(0.0);
            let scale = sat_mul * vib_mul;
            let out = OklabColor::new(lab.l, lab.a * scale, lab.b * scale, 1.0).to_linear();
            px[0] = out.r();
            px[1] = out.g();
            px[2] = out.b();
        }
    }
}

/// Posterize — quantize each channel to `levels` (`2..=32`) evenly-spaced steps
/// in DISPLAY (sRGB) space (where the bands read as Photoshop's do), then convert
/// back to linear. `acc` is straight LINEAR f32 RGBA (alpha preserved). Always
/// applies (a freshly-created Posterize is a visible effect, like Photoshop).
pub(crate) fn apply_posterize(p: &PosterizeParams, acc: &mut [[f32; 4]]) {
    let levels = p.levels.clamp(2, 32);
    let steps = (levels - 1) as f32;
    // Encode (linear→sRGB) LUT picks the band; `band_out[k]` is the exact linear
    // value of band `k` (≤32 transcendentals total, none per pixel). The hard
    // `round()` stays exact so the quantization boundaries don't smear.
    let encode = build_lut(linear_to_srgb_f32);
    let band_out: [f32; 32] =
        core::array::from_fn(|k| srgb_to_linear_f32((k as f32 / steps).min(1.0)));
    let max_k = (levels - 1) as usize;
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            let s = sample_lut(&encode, *ch);
            let k = ((s * steps).round() as usize).min(max_k);
            *ch = band_out[k];
        }
    }
}

/// Threshold — every pixel becomes pure black or white by comparing its display-
/// space luma (Rec.601 weights on sRGB, matching Photoshop's Threshold) against
/// `threshold` (`0..=255`). `acc` is straight LINEAR f32 RGBA (alpha preserved).
pub(crate) fn apply_threshold(p: &ThresholdParams, acc: &mut [[f32; 4]]) {
    let cut = p.threshold as f32 / 255.0;
    let encode = build_lut(linear_to_srgb_f32); // luma is computed in display space
    for px in acc.iter_mut() {
        let luma = 0.299 * sample_lut(&encode, px[0])
            + 0.587 * sample_lut(&encode, px[1])
            + 0.114 * sample_lut(&encode, px[2]);
        let v = if luma >= cut { 1.0 } else { 0.0 };
        px[0] = v;
        px[1] = v;
        px[2] = v;
    }
}

/// Invert — a photographic negative: `1 - x` per channel in DISPLAY (sRGB) space
/// (a linear `1 - x` would skew midtones), converted back to linear. `acc` is
/// straight LINEAR f32 RGBA (alpha preserved).
pub(crate) fn apply_invert(acc: &mut [[f32; 4]]) {
    let lut = build_lut(|v| srgb_to_linear_f32(1.0 - linear_to_srgb_f32(v)));
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            *ch = sample_lut(&lut, *ch);
        }
    }
}

/// Brightness/Contrast. `acc` is straight LINEAR f32 RGBA (alpha preserved).
/// Contrast scales each channel around the perceptual mid-gray pivot; brightness
/// then lerps toward black (`-1`) / white (`+1`) — exact extremes (the same
/// brightness model as [`apply_hsb`], for consistency). Both `-1..1`, neutral 0.
pub(crate) fn apply_brightness_contrast(p: &BrightnessContrastParams, acc: &mut [[f32; 4]]) {
    if p.brightness == 0.0 && p.contrast == 0.0 {
        return;
    }
    // sRGB 0.5 in linear light — contrast pivots around perceptual mid-gray
    // (a linear-0.5 pivot would sit far too bright).
    const PIVOT: f32 = 0.214_041_14;
    let scale = 1.0 + p.contrast; // -1 → flat to pivot, +1 → 2×
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            // Contrast around the pivot, clamped so the brightness lerp below
            // stays well-defined (LDR; the compositor encode clamps anyway).
            let mut v = ((*ch - PIVOT) * scale + PIVOT).clamp(0.0, 1.0);
            if p.brightness > 0.0 {
                v += (1.0 - v) * p.brightness;
            } else if p.brightness < 0.0 {
                v *= 1.0 + p.brightness;
            }
            *ch = v;
        }
    }
}

/// Photo Filter — a colored gel over the image. A photographic filter is a
/// physical sheet of tinted glass in front of the lens, so it is a straight
/// LINEAR-light multiply (the space `acc` already lives in — no sRGB round-trip).
/// `temperature` (`-1..1`, neutral 0) picks a WARM (`>0`, passes red/green, cuts
/// blue) or COOL (`<0`, passes blue, cuts red) gel; `density` (`0..1`) is its
/// strength; with `preserve_luminosity` each pixel's luminance is renormalized
/// after the multiply so the filter shifts color WITHOUT darkening (Photoshop's
/// default). Alpha (= coverage) is preserved. Neutral (`density == 0` OR
/// `temperature == 0`, both ⇒ the unit gel) early-returns an exact identity — the
/// drag hot path, mirror of [`apply_hsb`].
pub(crate) fn apply_photo_filter(p: &PhotoFilterParams, acc: &mut [[f32; 4]]) {
    if p.density == 0.0 || p.temperature == 0.0 {
        return;
    }
    let t = p.temperature.clamp(-1.0, 1.0);
    let density = p.density.clamp(0.0, 1.0);
    // Linear-light gel transmittances at full strength (`|t| = 1`). Warm cuts the
    // blue channel; cool cuts red — the classic 85 (warming) / 80 (cooling) gels.
    const WARM: [f32; 3] = [1.0, 0.75, 0.45];
    const COOL: [f32; 3] = [0.55, 0.80, 1.0];
    let anchor = if t >= 0.0 { WARM } else { COOL };
    let mag = t.abs();
    // Effective per-channel gain: blend neutral white → anchor by |t|, then
    // white → that gel by density (both lerps fold into one factor per channel).
    let eff: [f32; 3] = core::array::from_fn(|c| {
        let gel = 1.0 + (anchor[c] - 1.0) * mag;
        1.0 + (gel - 1.0) * density
    });
    // Linear Rec.709 luma (acc is linear light) for the optional preserve-lum renorm.
    const LW: [f32; 3] = [0.2126, 0.7152, 0.0722];
    for px in acc.iter_mut() {
        let l_in = LW[0] * px[0] + LW[1] * px[1] + LW[2] * px[2];
        for (c, e) in eff.iter().enumerate() {
            px[c] *= e;
        }
        if p.preserve_luminosity {
            let l_out = LW[0] * px[0] + LW[1] * px[1] + LW[2] * px[2];
            if l_out > 1e-6 {
                let k = l_in / l_out;
                for c in px.iter_mut().take(3) {
                    *c *= k;
                }
            }
        }
    }
}

/// Black & White — a hue-aware grayscale conversion in DISPLAY space. Each pixel
/// is decomposed into its six hue components (reds/yellows/greens/cyans/blues/
/// magentas, the RGB hue hexagon), and the per-hue sliders weight how much each
/// contributes to the output gray (`-2..3`; the achromatic floor `min(r,g,b)`
/// always passes through). With a `tint_color` (+ `tint_amount`) the gray is then
/// colorized in OKLab — the tint's hue/chroma applied at each pixel's lightness
/// (Photoshop's "Tint"). `acc` is straight LINEAR f32 RGBA (alpha preserved).
/// Always applies (a fresh Black & White is a visible grayscale, like Posterize).
pub(crate) fn apply_black_and_white(p: &BlackAndWhiteParams, acc: &mut [[f32; 4]]) {
    let w = [p.reds, p.yellows, p.greens, p.cyans, p.blues, p.magentas];
    // Tint is active only with a color AND a non-zero amount; precompute its
    // OKLab chroma direction (hue in degrees → the (a, b) unit vector × chroma).
    let tint = p.tint_color.filter(|_| p.tint_amount != 0.0);
    let (ta, tb) = match tint {
        Some(t) => {
            let (s, c) = t.h.to_radians().sin_cos();
            (t.c * c * p.tint_amount, t.c * s * p.tint_amount)
        }
        None => (0.0, 0.0),
    };
    for px in acc.iter_mut() {
        let (r, g, b) = (
            linear_to_srgb_f32(px[0]),
            linear_to_srgb_f32(px[1]),
            linear_to_srgb_f32(px[2]),
        );
        let m = r.min(g).min(b);
        let (rr, gg, bb) = (r - m, g - m, b - m);
        // The chroma remainder (one channel is 0) falls in one hue sector between
        // two adjacent vertices; split it into the two pure-hue amounts.
        let (reds, yellows, greens, cyans, blues, magentas) = if b <= r && b <= g {
            let yellow = rr.min(gg); // R–G sector (blue is the achromatic floor)
            (rr - yellow, yellow, gg - yellow, 0.0, 0.0, 0.0)
        } else if r <= g && r <= b {
            let cyan = gg.min(bb); // G–B sector (red floor)
            (0.0, 0.0, gg - cyan, cyan, bb - cyan, 0.0)
        } else {
            let magenta = bb.min(rr); // B–R sector (green floor)
            (rr - magenta, 0.0, 0.0, 0.0, bb - magenta, magenta)
        };
        let gray = (m
            + reds * w[0]
            + yellows * w[1]
            + greens * w[2]
            + cyans * w[3]
            + blues * w[4]
            + magentas * w[5])
            .clamp(0.0, 1.0);
        let gray_lin = srgb_to_linear_f32(gray);
        if tint.is_some() {
            // Colorize: keep the gray's OKLab lightness, set the tint chroma vector.
            let l = OklabColor::from_linear(LinearRgba::new(gray_lin, gray_lin, gray_lin, 1.0)).l;
            let out = OklabColor::new(l, ta, tb, 1.0).to_linear();
            px[0] = out.r();
            px[1] = out.g();
            px[2] = out.b();
        } else {
            px[0] = gray_lin;
            px[1] = gray_lin;
            px[2] = gray_lin;
        }
    }
}

/// Hue / Saturation / Brightness in **OKLab** (the project's perceptual color
/// space — gold standard, and the brush engine's native space). `acc` is
/// straight LINEAR f32 RGBA; only RGB is transformed — the alpha (= coverage)
/// is preserved.
///
/// - **Hue** (`h`, in turns) is a RIGID rotation of the `(a, b)` chroma vector,
///   so chroma magnitude is preserved exactly and near-neutral pixels stay
///   neutral. HSL hue is numerically unstable for near-gray pixels (tiny chroma
///   → ill-defined hue), so rotating it scatters incoherent colors — the colored
///   speckle Enio hit on the gray background. An OKLab rotation has no such
///   instability.
/// - **Saturation** (`s`, `-1..1`) scales chroma (`-1` = grayscale, `+1` = 2×).
/// - **Brightness** (`b`, `-1..1`) lerps toward black (`-1`) / white (`+1`) in
///   linear light, so the extremes are EXACT black / white (matches Procreate).
///
/// Neutral `{0, 0, 0}` early-returns an EXACT identity (and skips the OKLab
/// round-trip — also the hot-path win while dragging).
pub(crate) fn apply_hsb(p: &HsbParams, acc: &mut [[f32; 4]]) {
    if p.h == 0.0 && p.s == 0.0 && p.b == 0.0 {
        return;
    }
    let hue_rad = p.h * std::f32::consts::TAU; // turns → radians
    let (hue_sin, hue_cos) = hue_rad.sin_cos();
    let chroma_scale = (1.0 + p.s).max(0.0); // -1 → 0 (gray), +1 → 2×
    for px in acc.iter_mut() {
        let lab = OklabColor::from_linear(LinearRgba::new(px[0], px[1], px[2], 1.0));
        // Hue rotation (rigid) + saturation (scale) of the chroma vector; L kept.
        let a = (lab.a * hue_cos - lab.b * hue_sin) * chroma_scale;
        let b = (lab.a * hue_sin + lab.b * hue_cos) * chroma_scale;
        let out = OklabColor::new(lab.l, a, b, 1.0).to_linear();
        let (mut r, mut g, mut bl) = (out.r(), out.g(), out.b());
        // Brightness: lerp toward black / white in linear (exact at the ends).
        if p.b > 0.0 {
            r += (1.0 - r) * p.b;
            g += (1.0 - g) * p.b;
            bl += (1.0 - bl) * p.b;
        } else if p.b < 0.0 {
            let k = 1.0 + p.b;
            r *= k;
            g *= k;
            bl *= k;
        }
        px[0] = r;
        px[1] = g;
        px[2] = bl;
    }
}
