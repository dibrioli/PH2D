//! Adjustment computation: `apply_adjustment` dispatch + per-kind kernels +
//! LUT helpers + slider param accessors. Split out of the former
//! `adjustments.rs` (pure mechanical move).

use super::*;

/// Apply a non-destructive adjustment to a window of the compositor's
/// accumulator IN PLACE. `acc` is **straight, LINEAR f32 RGBA** (the same space
/// `ph2d_tool_painter::compositor` blends in) — operating on f32 keeps a stack
/// of adjustments band-free (no 8-bit round-trip in the per-frame composite).
///
/// Mask / opacity / blend-mode are handled by the compositor AROUND this call
/// (copy → `apply_adjustment` → blend by mask×opacity in the layer's blend
/// mode), so this fn is the pure `kind` + `params` → pixel transform. Kinds
/// conventionally defined in display space (Curves / Levels / Posterize)
/// convert linear↔sRGB internally.
///
/// **STUB (W4 T4.1/T4.2 Coord):** the hook signature + wiring are landed (the
/// compositor calls this for every `LayerKind::Adjustment`), but the per-kind
/// compute is the implementer's (T4.3+, HSB first for the Day-4 smoke). Replace
/// the no-op body with `match kind { … }`; an implemented arm goes live the next
/// frame.
pub fn apply_adjustment(kind: &AdjustmentKind, params: &AdjustmentParams, acc: &mut [[f32; 4]]) {
    debug_assert_eq!(
        params.kind(),
        *kind,
        "apply_adjustment: kind/params variant mismatch"
    );
    // The match grows an arm per kind as T4.x land; the remaining kinds stay
    // no-ops (identity) until theirs ships.
    match (kind, params) {
        // T4.3 — Hue/Saturation/Brightness (Day-4 smoke).
        (AdjustmentKind::HueSaturationBrightness, AdjustmentParams::HueSaturationBrightness(p)) => {
            apply_hsb(p, acc)
        }
        // T4.7 — Brightness/Contrast.
        (AdjustmentKind::BrightnessContrast, AdjustmentParams::BrightnessContrast(p)) => {
            apply_brightness_contrast(p, acc)
        }
        // T4.x — Exposure (linear gain + offset + gamma).
        (AdjustmentKind::Exposure, AdjustmentParams::Exposure(p)) => apply_exposure(p, acc),
        // T4.x — Vibrance (OKLab chroma, low-saturation-weighted).
        (AdjustmentKind::Vibrance, AdjustmentParams::Vibrance(p)) => apply_vibrance(p, acc),
        // T4.x — Posterize (display-space level quantization).
        (AdjustmentKind::Posterize, AdjustmentParams::Posterize(p)) => apply_posterize(p, acc),
        // T4.x — Threshold (display-space luma → black/white).
        (AdjustmentKind::Threshold, AdjustmentParams::Threshold(p)) => apply_threshold(p, acc),
        // T4.x — Invert (display-space photographic negative).
        (AdjustmentKind::Invert, AdjustmentParams::Invert(_)) => apply_invert(acc),
        _ => {}
    }
}

// ─────────────────────── sRGB transfer (display space) ───────────────────
//
// Continuous f32 sRGB ↔ linear transfer (IEC 61966), for kinds conventionally
// defined in display space (Invert / Posterize / Threshold). The `ph2d_color`
// crate exposes only the 8-bit byte transfer; these f32 twins avoid the
// quantization round-trip while staying byte-identical at the sample points.

/// linear-light intensity → sRGB-encoded `0..=1` (display space).
#[inline]
pub(crate) fn linear_to_srgb_f32(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB-encoded `0..=1` (display space) → linear-light intensity.
#[inline]
pub(crate) fn srgb_to_linear_f32(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

// ─────────────────────────── per-call 1-D LUT ───────────────────────────
//
// PERF (handoff §3 — the implementer's perf duty is "keep the compute cheap;
// avoid redundant transcendentals"). A stack of adjustments re-composites the
// whole canvas every drag frame (the structural `CompositorCache` cut-point is
// the Coord's W4 lever, not this), so a per-pixel `powf` dominates: the naive
// display-space kinds cost up to 6 `powf`/pixel (an sRGB round-trip per channel)
// — visibly worse than the OKLab kinds' single `cbrt` round-trip, which is the
// FPS Enio felt. Every per-channel display-space op here is a 1-D function of the
// input, so build its LUT ONCE per call (N evals) and make the per-pixel inner
// loop a clamp + index + lerp: ZERO transcendentals/pixel.

const LUT_N: usize = 1024;

/// Build a 1-D LUT sampling `f` uniformly over the input domain `0..=1`.
pub(crate) fn build_lut<F: Fn(f32) -> f32>(f: F) -> [f32; LUT_N] {
    core::array::from_fn(|i| f(i as f32 / (LUT_N - 1) as f32))
}

/// Sample a [`build_lut`] table at `v` (clamped to `0..=1`) with linear
/// interpolation between the two bracketing entries.
#[inline]
pub(crate) fn sample_lut(lut: &[f32; LUT_N], v: f32) -> f32 {
    let t = v.clamp(0.0, 1.0) * (LUT_N - 1) as f32;
    let i = t as usize; // floor (t ≥ 0); always in 0..=LUT_N-1
    let frac = t - i as f32;
    let a = lut[i];
    let b = lut[(i + 1).min(LUT_N - 1)];
    a + (b - a) * frac
}

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

/// The slider-editable params of an adjustment, as `(label, value01)` in slot
/// order — what the layers panel renders as a labeled slider per slot, and the
/// inverse of [`set_adjustment_slider_param`]. Kinds with bespoke controls
/// (Curves, Gradient Map, …) return empty here and get their own UI later.
#[must_use]
pub fn adjustment_slider_params(params: &AdjustmentParams) -> Vec<(&'static str, f32)> {
    match params {
        AdjustmentParams::HueSaturationBrightness(p) => vec![
            ("Hue", p.h.clamp(0.0, 1.0)),
            ("Sat", (p.s + 1.0) * 0.5),
            ("Bright", (p.b + 1.0) * 0.5),
        ],
        AdjustmentParams::BrightnessContrast(p) => vec![
            ("Bright", (p.brightness + 1.0) * 0.5),
            ("Contrast", (p.contrast + 1.0) * 0.5),
        ],
        // Exposure: EV -4..4, Offset -0.5..0.5, Gamma -0.9..0.9 (all centered).
        AdjustmentParams::Exposure(p) => vec![
            ("Expo", (p.exposure_ev + 4.0) / 8.0),
            ("Offset", p.offset + 0.5),
            ("Gamma", (p.gamma_correction + 0.9) / 1.8),
        ],
        AdjustmentParams::Vibrance(p) => vec![
            ("Vib", (p.vibrance + 1.0) * 0.5),
            ("Sat", (p.saturation + 1.0) * 0.5),
        ],
        // Posterize levels 2..=32; Threshold cutoff 0..=255.
        AdjustmentParams::Posterize(p) => {
            vec![("Levels", (p.levels.clamp(2, 32) as f32 - 2.0) / 30.0)]
        }
        AdjustmentParams::Threshold(p) => vec![("Level", p.threshold as f32 / 255.0)],
        _ => Vec::new(),
    }
}

/// Set slider `slot` of an adjustment from a normalized `0..1` value (inverse of
/// [`adjustment_slider_params`]). Out-of-range slots / non-slider kinds no-op.
pub fn set_adjustment_slider_param(params: &mut AdjustmentParams, slot: usize, value01: f32) {
    let v = value01.clamp(0.0, 1.0);
    match params {
        AdjustmentParams::HueSaturationBrightness(p) => match slot {
            0 => p.h = v,             // 0..1 turns
            1 => p.s = v * 2.0 - 1.0, // -1..1
            2 => p.b = v * 2.0 - 1.0, // -1..1
            _ => {}
        },
        AdjustmentParams::BrightnessContrast(p) => match slot {
            0 => p.brightness = v * 2.0 - 1.0,
            1 => p.contrast = v * 2.0 - 1.0,
            _ => {}
        },
        AdjustmentParams::Exposure(p) => match slot {
            0 => p.exposure_ev = v * 8.0 - 4.0,      // -4..4 EV
            1 => p.offset = v - 0.5,                 // -0.5..0.5
            2 => p.gamma_correction = v * 1.8 - 0.9, // -0.9..0.9 (effective γ 0.1..1.9)
            _ => {}
        },
        AdjustmentParams::Vibrance(p) => match slot {
            0 => p.vibrance = v * 2.0 - 1.0,
            1 => p.saturation = v * 2.0 - 1.0,
            _ => {}
        },
        AdjustmentParams::Posterize(p) if slot == 0 => {
            p.levels = (2.0 + v * 30.0).round().clamp(2.0, 32.0) as u8;
        }
        AdjustmentParams::Threshold(p) if slot == 0 => {
            p.threshold = (v * 255.0).round().clamp(0.0, 255.0) as u8;
        }
        _ => {}
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
