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
        // W4 bespoke — Curves (per-channel display-space tone curves, LUT-baked).
        (AdjustmentKind::Curves, AdjustmentParams::Curves(p)) => apply_curves(p, acc),
        // W4 bespoke — Levels (display-space black/gamma/white + output remap).
        (AdjustmentKind::Levels, AdjustmentParams::Levels(p)) => apply_levels(p, acc),
        // W4 BATCH-1 — Photo Filter (warm/cool gel: linear multiply + luma preserve).
        (AdjustmentKind::PhotoFilter, AdjustmentParams::PhotoFilter(p)) => {
            apply_photo_filter(p, acc)
        }
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

// ───────────────── display-space 1-D transfer LUTs (Curves / Levels) ──────────
//
// Curves and Levels are both per-channel 1-D transfers DEFINED IN DISPLAY (sRGB)
// space — `out_ch = f(in_ch)` — so they bake to a per-channel table indexed in
// display space. This is the real-time strategy (handoff §3): a `[f32; 256]`
// table per channel is what the GPU compositor's `adj_luts` binding samples
// (`adj_luts[base + ch*256 + round(s*255)]`), turning a curve that would cost a
// spline eval / `powf` per pixel into a single L1 lookup. The CPU kernels below
// sample the SAME tables, so GPU↔CPU parity is "do they read the same table"
// (within the ±tolerance the parity gate allows for the GPU's nearest vs the
// CPU's lerp lookup). The exporters are `pub` so the tool's GPU flatten can
// build the buffer (`ph2d-render` stays decoupled — the tool feeds it the bytes).

/// Width of a display-space transfer LUT (one entry per 8-bit display value).
/// The GPU `adj_luts` storage buffer uses this as its per-channel stride.
pub const DISPLAY_LUT_N: usize = 256;

/// Build a 256-entry display-space transfer table: `lut[i]` is the output for
/// display input `i / 255`. `f` maps display `0..=1` → display `0..=1`.
fn build_display_lut<F: Fn(f32) -> f32>(f: F) -> [f32; DISPLAY_LUT_N] {
    core::array::from_fn(|i| f(i as f32 / (DISPLAY_LUT_N - 1) as f32))
}

/// Sample a [`build_display_lut`] table at display `s` (`0..=1`) with linear
/// interpolation. The GPU samples the same table with a nearest lookup; the
/// difference is bounded by one 8-bit step and absorbed by the ±tolerance
/// GPU↔CPU parity gate.
#[inline]
fn sample_display_lut(lut: &[f32; DISPLAY_LUT_N], s: f32) -> f32 {
    let t = s.clamp(0.0, 1.0) * (DISPLAY_LUT_N - 1) as f32;
    let i = t as usize;
    let frac = t - i as f32;
    let a = lut[i];
    let b = lut[(i + 1).min(DISPLAY_LUT_N - 1)];
    a + (b - a) * frac
}

/// Fritsch–Carlson monotone tangent at control point `i` of a tone curve — the
/// Hermite slope used by the segments on either side. Clamped against the
/// adjacent secants so the spline stays MONOTONE (never overshoots past the
/// control points — a tone curve must not wiggle out of the points' range).
fn monotone_tangent(points: &[[f32; 2]], i: usize) -> f32 {
    let n = points.len();
    let secant = |a: usize, b: usize| {
        let dx = points[b][0] - points[a][0];
        if dx.abs() <= 1e-9 {
            0.0
        } else {
            (points[b][1] - points[a][1]) / dx
        }
    };
    // Raw tangent: endpoints use the one-sided secant, interior the average.
    let mut m = if i == 0 {
        secant(0, 1)
    } else if i == n - 1 {
        secant(n - 2, n - 1)
    } else {
        0.5 * (secant(i - 1, i) + secant(i, i + 1))
    };
    // Monotonicity clamp (Fritsch–Carlson sufficient condition |m| ≤ 3|d| per
    // adjacent secant; a flat or sign-flipping secant forces a flat tangent).
    let neighbors = [
        (i > 0).then(|| secant(i - 1, i)),
        (i + 1 < n).then(|| secant(i, i + 1)),
    ];
    for d in neighbors.into_iter().flatten() {
        if d == 0.0 {
            m = 0.0;
        } else {
            let r = m / d;
            if r < 0.0 {
                m = 0.0;
            } else if r > 3.0 {
                m = 3.0 * d;
            }
        }
    }
    m
}

/// Evaluate a tone curve `points` (normalized `[x, y]` in `0..=1`, sorted by x)
/// at display-space input `x`. Empty → the identity (`y = x`); one point → a
/// constant; two or more → a monotone cubic-Hermite spline ([`monotone_tangent`]).
/// Outside the point span the endpoints extend flat (Photoshop's clamp).
fn eval_curve(points: &[[f32; 2]], x: f32) -> f32 {
    match points.len() {
        0 => return x.clamp(0.0, 1.0),
        1 => return points[0][1].clamp(0.0, 1.0),
        _ => {}
    }
    let x = x.clamp(0.0, 1.0);
    let n = points.len();
    if x <= points[0][0] {
        return points[0][1].clamp(0.0, 1.0);
    }
    if x >= points[n - 1][0] {
        return points[n - 1][1].clamp(0.0, 1.0);
    }
    let mut i = 0;
    while i + 1 < n && points[i + 1][0] < x {
        i += 1;
    }
    let (x0, y0) = (points[i][0], points[i][1]);
    let (x1, y1) = (points[i + 1][0], points[i + 1][1]);
    let h = x1 - x0;
    if h <= 1e-9 {
        return y1.clamp(0.0, 1.0); // coincident control points — take the later
    }
    let m0 = monotone_tangent(points, i);
    let m1 = monotone_tangent(points, i + 1);
    let t = (x - x0) / h;
    let (t2, t3) = (t * t, t * t * t);
    // Cubic Hermite basis.
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    (h00 * y0 + h10 * h * m0 + h01 * y1 + h11 * h * m1).clamp(0.0, 1.0)
}

/// Sample a tone curve `points` (normalized `[x, y]`, ascending x) at `x`,
/// returning the output `y`. Public entry to the monotone spline eval
/// ([`eval_curve`]) for the tool — e.g. placing a newly-inserted control point ON
/// the current curve so adding it leaves the output unchanged.
#[must_use]
pub fn curve_value_at(points: &[[f32; 2]], x: f32) -> f32 {
    eval_curve(points, x)
}

/// Photoshop-style Levels transfer in DISPLAY space: display input `s` (`0..=1`)
/// → display output. Input black/white points clip+stretch the range, `gamma`
/// (the midtone slider, effective neutral `1.0`) reshapes it, and the output
/// black/white compress into a target range. Neutral params are an exact identity.
fn levels_transfer(s: f32, p: &LevelsParams) -> f32 {
    let s = s.clamp(0.0, 1.0);
    let bp = p.black_point.clamp(0.0, 1.0);
    let wp = p.white_point.clamp(0.0, 1.0);
    // Input remap: stretch [bp, wp] → [0, 1]; a degenerate span (wp ≤ bp) is a
    // hard step at bp.
    let span = wp - bp;
    let t = if span > 1e-6 {
        ((s - bp) / span).clamp(0.0, 1.0)
    } else if s >= bp {
        1.0
    } else {
        0.0
    };
    // Midtone gamma (PS: out = t^(1/γ); γ > 1 brightens). Neutral γ = 1.
    let g = if p.gamma > 1e-3 {
        t.powf(1.0 / p.gamma)
    } else {
        t
    };
    // Output remap: compress [0, 1] → [output_black, output_white].
    let ob = p.output_black.clamp(0.0, 1.0);
    let ow = p.output_white.clamp(0.0, 1.0);
    ob + g * (ow - ob)
}

/// Per-channel display-space transfer LUTs for a [`CurvesParams`] — `[R, G, B]`,
/// each a 256-entry display→display table baking the master (RGB) curve composed
/// over the per-channel curve (`out = master(channel(in))`, Photoshop order).
/// **The GPU-mandate deliverable's math**: the compositor's `adj_luts` binding
/// uploads exactly these tables and the WGSL `ADJ_CURVES` case samples them, so
/// CPU [`apply_curves`] and the GPU read the SAME function.
#[must_use]
pub fn curves_display_luts(p: &CurvesParams) -> [[f32; DISPLAY_LUT_N]; 3] {
    let chans = [&p.points_r, &p.points_g, &p.points_b];
    core::array::from_fn(|c| {
        build_display_lut(|s| eval_curve(&p.points_rgb.points, eval_curve(&chans[c].points, s)))
    })
}

/// Channel-uniform display-space transfer LUT for a [`LevelsParams`] (the same
/// table applies to R/G/B). The GPU `ADJ_LEVELS` case samples this 256-entry
/// table; CPU [`apply_levels`] samples the same one.
#[must_use]
pub fn levels_display_lut(p: &LevelsParams) -> [f32; DISPLAY_LUT_N] {
    build_display_lut(|s| levels_transfer(s, p))
}

/// `true` for a Levels params that is an exact identity (so [`apply_levels`] can
/// early-return without the per-pixel sRGB round-trip — the neutral hot path).
fn levels_is_neutral(p: &LevelsParams) -> bool {
    p.black_point == 0.0
        && p.white_point == 1.0
        && p.gamma == 1.0
        && p.output_black == 0.0
        && p.output_white == 1.0
}

/// Curves — per-channel tone curve in DISPLAY space. Builds the per-channel LUTs
/// ([`curves_display_luts`], the same tables the GPU samples) once, then maps each
/// pixel via an sRGB round-trip. `acc` is straight LINEAR f32 RGBA (alpha
/// preserved). All-empty curves (the neutral default) early-return an exact
/// identity (skipping the round-trip — the hot-path win, mirror of [`apply_hsb`]).
pub(crate) fn apply_curves(p: &CurvesParams, acc: &mut [[f32; 4]]) {
    if p.points_rgb.points.is_empty()
        && p.points_r.points.is_empty()
        && p.points_g.points.is_empty()
        && p.points_b.points.is_empty()
    {
        return;
    }
    let luts = curves_display_luts(p);
    for px in acc.iter_mut() {
        for (ch, v) in px.iter_mut().take(3).enumerate() {
            let s = linear_to_srgb_f32(*v);
            *v = srgb_to_linear_f32(sample_display_lut(&luts[ch], s));
        }
    }
}

/// Levels — Photoshop-style black/gamma/white input remap + output remap in
/// DISPLAY space. Builds the channel-uniform LUT ([`levels_display_lut`]) once,
/// then maps each pixel via an sRGB round-trip. `acc` is straight LINEAR f32 RGBA
/// (alpha preserved). Neutral params early-return an exact identity.
pub(crate) fn apply_levels(p: &LevelsParams, acc: &mut [[f32; 4]]) {
    if levels_is_neutral(p) {
        return;
    }
    let lut = levels_display_lut(p);
    for px in acc.iter_mut() {
        for v in px.iter_mut().take(3) {
            let s = linear_to_srgb_f32(*v);
            *v = srgb_to_linear_f32(sample_display_lut(&lut, s));
        }
    }
}

/// Levels gamma ↔ slider: log-symmetric so the neutral γ=1 sits at the slider
/// midpoint and the usable range is γ ∈ [0.1, 10].
fn levels_gamma_to_slider(gamma: f32) -> f32 {
    (gamma.max(1e-3).log10() / 2.0 + 0.5).clamp(0.0, 1.0)
}

/// Inverse of [`levels_gamma_to_slider`].
fn levels_slider_to_gamma(s: f32) -> f32 {
    10.0_f32.powf((s.clamp(0.0, 1.0) - 0.5) * 2.0)
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
        // Photo Filter: Temperature -1..1 (centered; 0.5 = neutral) + Density 0..1.
        // `preserve_luminosity` is a toggle (see `adjustment_toggle_params`).
        AdjustmentParams::PhotoFilter(p) => vec![
            ("Temp", (p.temperature.clamp(-1.0, 1.0) + 1.0) * 0.5),
            ("Density", p.density.clamp(0.0, 1.0)),
        ],
        // Levels maps cleanly onto the generic slider rack (5 ≤ 6 slots): input
        // black/gamma/white + output black/white. (Curves needs the bespoke
        // curve canvas — handoff §4 — so it returns no generic sliders.)
        AdjustmentParams::Levels(p) => vec![
            ("Black", p.black_point.clamp(0.0, 1.0)),
            ("Gamma", levels_gamma_to_slider(p.gamma)),
            ("White", p.white_point.clamp(0.0, 1.0)),
            ("Out Lo", p.output_black.clamp(0.0, 1.0)),
            ("Out Hi", p.output_white.clamp(0.0, 1.0)),
        ],
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
        AdjustmentParams::PhotoFilter(p) => match slot {
            0 => p.temperature = v * 2.0 - 1.0, // -1..1
            1 => p.density = v,                 // 0..1
            _ => {}
        },
        AdjustmentParams::Levels(p) => match slot {
            0 => p.black_point = v,
            1 => p.gamma = levels_slider_to_gamma(v),
            2 => p.white_point = v,
            3 => p.output_black = v,
            4 => p.output_white = v,
            _ => {}
        },
        // Curves has no generic sliders — its bespoke editor drives free 2-D point
        // drags through `PainterTool::set_curve_point` (W4 §3), not this slot path.
        _ => {}
    }
}

/// The boolean (toggle) params of an adjustment, as `(label, on)` in slot order
/// — the toggle-rack twin of [`adjustment_slider_params`]. The layers panel
/// renders each as a small switch row under the slider rack; the inverse setter
/// is [`set_adjustment_toggle_param`]. Kinds with no toggles return empty. Adding
/// a toggle-bearing kind needs ZERO panel change (the rack iterates this).
#[must_use]
pub fn adjustment_toggle_params(params: &AdjustmentParams) -> Vec<(&'static str, bool)> {
    match params {
        AdjustmentParams::PhotoFilter(p) => vec![("Preserve Lum.", p.preserve_luminosity)],
        _ => Vec::new(),
    }
}

/// Set toggle `slot` of an adjustment (inverse of [`adjustment_toggle_params`]).
/// Out-of-range slots / non-toggle kinds no-op.
pub fn set_adjustment_toggle_param(params: &mut AdjustmentParams, slot: usize, on: bool) {
    match params {
        AdjustmentParams::PhotoFilter(p) if slot == 0 => p.preserve_luminosity = on,
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
