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
        // W4 BATCH-1 — Color Balance (per-channel tonal-range-weighted shift).
        (AdjustmentKind::ColorBalance, AdjustmentParams::ColorBalance(p)) => {
            apply_color_balance(p, acc)
        }
        // W4 BATCH-1 — Channel Mixer (3×4 display-space matrix + monochrome).
        (AdjustmentKind::ChannelMixer, AdjustmentParams::ChannelMixer(p)) => {
            apply_channel_mixer(p, acc)
        }
        // W4 BATCH-1 — Black & White (6-hue luminance mix + optional tint).
        (AdjustmentKind::BlackAndWhite, AdjustmentParams::BlackAndWhite(p)) => {
            apply_black_and_white(p, acc)
        }
        // W4 BATCH-2 — Gradient Map (luma → gradient color, 256→RGB LUT).
        (AdjustmentKind::GradientMap, AdjustmentParams::GradientMap(p)) => {
            apply_gradient_map(p, acc)
        }
        // W4 BATCH-2 — Selective Color (9 color-group CMYK adjustment).
        (AdjustmentKind::SelectiveColor, AdjustmentParams::SelectiveColor(p)) => {
            apply_selective_color(p, acc)
        }
        // W4 close — Color Lookup (built-in cinematic look, per-pixel grade).
        (AdjustmentKind::ColorLookupLut, AdjustmentParams::ColorLookupLut(p)) => {
            super::lut::apply_color_lookup(p, acc)
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

/// The tonal-range weight for display value `s` (`0..1`) under `scope` — how
/// strongly a Color-Balance shift applies at that tone. Shadows fall off toward
/// white (`(1-s)²`), Highlights rise toward white (`s²`), Midtones hump at mid
/// (`1-(2s-1)²`). All in `0..=1`, smooth, zero at the off-end so the shift never
/// touches the opposite tonal extreme.
fn colorbalance_weight(s: f32, scope: ToneScope) -> f32 {
    let s = s.clamp(0.0, 1.0);
    match scope {
        ToneScope::Shadows => (1.0 - s) * (1.0 - s),
        ToneScope::Highlights => s * s,
        ToneScope::Midtones => {
            let d = 2.0 * s - 1.0;
            1.0 - d * d
        }
    }
}

/// Per-channel DISPLAY-space shift-transfer LUTs for [`ColorBalanceParams`]
/// (`[R, G, B]`). `lut_c[i]` = display input `i/255` biased by
/// `shift_c · weight(i/255, scope)` (clamped). This is the GPU-mandate
/// deliverable's math: the compositor's `adj_luts` binding uploads exactly these
/// (the same 3×256 machinery the Curves `ADJ_CURVES` case samples), so the
/// real-time GPU path reuses Curves' transfer-LUT sampling. The
/// preserve-luminosity renorm is the per-pixel step ON TOP (CPU below; a shader
/// flag for the GPU — see the W4 handoff §GPU-COORD).
#[must_use]
pub fn colorbalance_display_luts(p: &ColorBalanceParams) -> [[f32; DISPLAY_LUT_N]; 3] {
    // Full-slider strength on a fully-weighted tone (a moderate, Photoshop-ish
    // shift — a ±1 slider moves a fully-weighted display value by up to this).
    const K: f32 = 0.5;
    let shifts = [p.cyan_red, p.magenta_green, p.yellow_blue];
    core::array::from_fn(|c| {
        let shift = shifts[c].clamp(-1.0, 1.0) * K;
        build_display_lut(|s| (s + shift * colorbalance_weight(s, p.scope)).clamp(0.0, 1.0))
    })
}

/// `true` for a Color-Balance params that is an exact identity (all three shifts
/// neutral), so [`apply_color_balance`] can early-return before the per-pixel
/// sRGB round-trip — the neutral hot path while dragging another layer.
fn colorbalance_is_neutral(p: &ColorBalanceParams) -> bool {
    p.cyan_red == 0.0 && p.magenta_green == 0.0 && p.yellow_blue == 0.0
}

/// Color Balance — Photoshop-style per-channel tonal-range-weighted color shift
/// in DISPLAY space. The Red-Cyan / Magenta-Green / Yellow-Blue sliders bias the
/// R / G / B channel toward the warm end (`+`) or its complement (`-`), masked by
/// `scope`'s tonal weight ([`colorbalance_weight`]); `preserve_luminosity`
/// renormalizes each pixel's display luma so the shift moves color WITHOUT
/// changing brightness — **within gamut**: a channel that saturates is clamped to
/// 0..1 individually, which breaks the luma invariant for that pixel (Photoshop's
/// own behaviour; audit 2026-06-18 note — `apply_photo_filter` does the same renorm
/// but WITHOUT the final clamp, so the two diverge at the gamut boundary). Builds the per-channel LUTs
/// ([`colorbalance_display_luts`], the same tables the GPU binds) once, then maps
/// each pixel via an sRGB round-trip. `acc` is straight LINEAR f32 RGBA (alpha
/// preserved). Neutral shifts early-return an exact identity.
pub(crate) fn apply_color_balance(p: &ColorBalanceParams, acc: &mut [[f32; 4]]) {
    if colorbalance_is_neutral(p) {
        return;
    }
    let luts = colorbalance_display_luts(p);
    // Rec.601 display luma (matches the Threshold kernel + Photoshop's luma).
    const LW: [f32; 3] = [0.299, 0.587, 0.114];
    for px in acc.iter_mut() {
        let s = [
            linear_to_srgb_f32(px[0]),
            linear_to_srgb_f32(px[1]),
            linear_to_srgb_f32(px[2]),
        ];
        let mut o = [
            sample_display_lut(&luts[0], s[0]),
            sample_display_lut(&luts[1], s[1]),
            sample_display_lut(&luts[2], s[2]),
        ];
        if p.preserve_luminosity {
            let l_in = LW[0] * s[0] + LW[1] * s[1] + LW[2] * s[2];
            let l_out = LW[0] * o[0] + LW[1] * o[1] + LW[2] * o[2];
            if l_out > 1e-6 {
                let k = l_in / l_out;
                for v in &mut o {
                    *v = (*v * k).clamp(0.0, 1.0);
                }
            }
        }
        for (c, ov) in o.iter().enumerate() {
            px[c] = srgb_to_linear_f32(*ov);
        }
    }
}

/// `true` for a Channel-Mixer params that is the identity (R/G/B pass through
/// unmixed, no constant, not monochrome), so [`apply_channel_mixer`] can
/// early-return before the per-pixel sRGB round-trip — the neutral hot path.
fn channel_mixer_is_neutral(p: &ChannelMixerParams) -> bool {
    !p.monochromatic
        && p.red_out == [1.0, 0.0, 0.0, 0.0]
        && p.green_out == [0.0, 1.0, 0.0, 0.0]
        && p.blue_out == [0.0, 0.0, 1.0, 0.0]
}

/// Channel Mixer — a 3×4 matrix recombining the source channels, in DISPLAY
/// space (Photoshop applies the mix to the gamma-encoded channel values). Each
/// output row is `[r, g, b, constant]`: `out = r·R + g·G + b·B + constant`
/// (display, clamped). With `monochromatic` the `red_out` row is the single GRAY
/// mix written to all three channels (a weighted B&W conversion). `acc` is
/// straight LINEAR f32 RGBA (alpha preserved). The identity matrix early-returns
/// an exact identity.
pub(crate) fn apply_channel_mixer(p: &ChannelMixerParams, acc: &mut [[f32; 4]]) {
    if channel_mixer_is_neutral(p) {
        return;
    }
    let mix = |row: [f32; 4], r: f32, g: f32, b: f32| {
        (row[0] * r + row[1] * g + row[2] * b + row[3]).clamp(0.0, 1.0)
    };
    for px in acc.iter_mut() {
        let (r, g, b) = (
            linear_to_srgb_f32(px[0]),
            linear_to_srgb_f32(px[1]),
            linear_to_srgb_f32(px[2]),
        );
        if p.monochromatic {
            let gray = srgb_to_linear_f32(mix(p.red_out, r, g, b));
            px[0] = gray;
            px[1] = gray;
            px[2] = gray;
        } else {
            px[0] = srgb_to_linear_f32(mix(p.red_out, r, g, b));
            px[1] = srgb_to_linear_f32(mix(p.green_out, r, g, b));
            px[2] = srgb_to_linear_f32(mix(p.blue_out, r, g, b));
        }
    }
}

/// The 4 slider params (`R / G / B source weights + Constant`, each `(label,
/// value01)`) of Channel-Mixer output row `output` (0 = Red or Gray when
/// monochrome, 1 = Green, 2 = Blue) — what the bespoke Channel-Mixer editor
/// renders for the active output tab. Source weights map `-2..2 → 0..1`, the
/// constant `-1..1 → 0..1`. Inverse of [`set_channel_mixer_param`].
#[must_use]
pub fn channel_mixer_slider_params(
    p: &ChannelMixerParams,
    output: usize,
) -> Vec<(&'static str, f32)> {
    let row = match output {
        1 => p.green_out,
        2 => p.blue_out,
        _ => p.red_out,
    };
    let w = |v: f32| (v.clamp(-2.0, 2.0) + 2.0) * 0.25;
    vec![
        ("Red", w(row[0])),
        ("Green", w(row[1])),
        ("Blue", w(row[2])),
        ("Const", (row[3].clamp(-1.0, 1.0) + 1.0) * 0.5),
    ]
}

/// Set slider `slot` of Channel-Mixer output row `output` from a normalized
/// `0..1` value (inverse of [`channel_mixer_slider_params`]). Source weights map
/// `0..1 → -2..2`, the constant `0..1 → -1..1`. Out-of-range slots no-op.
pub fn set_channel_mixer_param(
    p: &mut ChannelMixerParams,
    output: usize,
    slot: usize,
    value01: f32,
) {
    let v = value01.clamp(0.0, 1.0);
    let row = match output {
        1 => &mut p.green_out,
        2 => &mut p.blue_out,
        _ => &mut p.red_out,
    };
    match slot {
        0 => row[0] = v * 4.0 - 2.0,
        1 => row[1] = v * 4.0 - 2.0,
        2 => row[2] = v * 4.0 - 2.0,
        3 => row[3] = v * 2.0 - 1.0,
        _ => {}
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

/// A single gradient stop's color (`[u8;4]` sRGB) → linear RGB.
fn stop_linear(color: [u8; 4]) -> [f32; 3] {
    [
        srgb_to_linear_f32(color[0] as f32 / 255.0),
        srgb_to_linear_f32(color[1] as f32 / 255.0),
        srgb_to_linear_f32(color[2] as f32 / 255.0),
    ]
}

/// Sample a gradient (`stops` ASCENDING by offset) at `offset` (`0..=1`) →
/// linear RGB. Outside the stop span the endpoints extend flat; `Smooth` applies
/// a smoothstep to the inter-stop `t`. Empty stops fall back to a black→white
/// ramp (so the LUT is well-defined even for a degenerate gradient).
fn gradient_sample(stops: &[ColorStop], interp: GradientInterp, offset: f32) -> [f32; 3] {
    if stops.is_empty() {
        let v = srgb_to_linear_f32(offset.clamp(0.0, 1.0));
        return [v, v, v];
    }
    let n = stops.len();
    if offset <= stops[0].offset {
        return stop_linear(stops[0].color);
    }
    if offset >= stops[n - 1].offset {
        return stop_linear(stops[n - 1].color);
    }
    let mut i = 0;
    while i + 1 < n && stops[i + 1].offset < offset {
        i += 1;
    }
    let (s0, s1) = (&stops[i], &stops[i + 1]);
    let span = s1.offset - s0.offset;
    let mut t = if span > 1e-6 {
        ((offset - s0.offset) / span).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if matches!(interp, GradientInterp::Smooth) {
        t = t * t * (3.0 - 2.0 * t);
    }
    let (c0, c1) = (stop_linear(s0.color), stop_linear(s1.color));
    core::array::from_fn(|ch| c0[ch] + (c1[ch] - c0[ch]) * t)
}

/// The 256-entry luma→linear-RGB table a [`GradientMapParams`] resolves to — the
/// real-time strategy (handoff §2.5): build the gradient ONCE, then the per-pixel
/// inner loop is a luma + table lookup. **The GPU-mandate deliverable's math**:
/// this is an RGB-OUTPUT LUT (3 channels from ONE luma input), NOT the per-channel
/// `adj_luts` transfer Curves uses, so the GPU needs a new 256×RGB binding mode
/// (Coord — see the W4 handoff §GPU-COORD-GM). Stops are sorted here so the table
/// is correct regardless of authoring order.
#[must_use]
pub fn gradient_map_lut(p: &GradientMapParams) -> [[f32; 3]; 256] {
    let mut stops = p.stops.clone();
    stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    core::array::from_fn(|i| gradient_sample(&stops, p.interpolation, i as f32 / 255.0))
}

/// The 3 RGB sliders (`(label, value01)`, `0..255 → 0..1`) of gradient `stop` —
/// what the bespoke editor renders for the selected stop. Out-of-range stops
/// return black. Inverse of [`set_gradient_stop_color_param`].
#[must_use]
pub fn gradient_stop_color_params(p: &GradientMapParams, stop: usize) -> Vec<(&'static str, f32)> {
    let c = p.stops.get(stop).map(|s| s.color).unwrap_or([0, 0, 0, 255]);
    vec![
        ("Red", c[0] as f32 / 255.0),
        ("Green", c[1] as f32 / 255.0),
        ("Blue", c[2] as f32 / 255.0),
    ]
}

/// Set RGB slider `slot` (0 = R, 1 = G, 2 = B) of gradient `stop` from a
/// normalized `0..1` value. Inverse of [`gradient_stop_color_params`]. Out-of-range
/// stops/slots no-op.
pub fn set_gradient_stop_color_param(
    p: &mut GradientMapParams,
    stop: usize,
    slot: usize,
    value01: f32,
) {
    let byte = (value01.clamp(0.0, 1.0) * 255.0).round() as u8;
    if let Some(s) = p.stops.get_mut(stop)
        && slot < 3
    {
        s.color[slot] = byte;
    }
}

/// Move gradient `stop` to `offset` (clamped `0..=1`). The stops keep their Vec
/// order (a stable index per editor handle, so a drag never re-binds to a
/// different stop); [`gradient_map_lut`] sorts a copy at sample time, so stops may
/// cross freely. No-op for an out-of-range index.
pub fn move_gradient_stop(p: &mut GradientMapParams, stop: usize, offset: f32) {
    if let Some(s) = p.stops.get_mut(stop) {
        s.offset = offset.clamp(0.0, 1.0);
    }
}

/// Insert a stop at the midpoint of the widest offset gap, its color sampled ON
/// the current gradient (so the rendered map is unchanged until the new stop is
/// recolored). Returns the inserted index, or `None` at the ≤16-stop cap or for a
/// degenerate (<1-stop) gradient. Mirror of `add_curve_point`.
pub fn add_gradient_stop(p: &mut GradientMapParams) -> Option<usize> {
    const MAX_STOPS: usize = 16;
    let n = p.stops.len();
    if !(1..MAX_STOPS).contains(&n) {
        return None;
    }
    // Widest gap between adjacent (sorted) stops, else after the last stop.
    let mut stops = p.stops.clone();
    stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    let (mut best_gap, mut new_off) = (-1.0_f32, 0.5_f32);
    for w in stops.windows(2) {
        let gap = w[1].offset - w[0].offset;
        if gap > best_gap {
            best_gap = gap;
            new_off = (w[0].offset + w[1].offset) * 0.5;
        }
    }
    let lin = gradient_sample(&stops, p.interpolation, new_off);
    let color = [
        (linear_to_srgb_f32(lin[0]) * 255.0).round() as u8,
        (linear_to_srgb_f32(lin[1]) * 255.0).round() as u8,
        (linear_to_srgb_f32(lin[2]) * 255.0).round() as u8,
        255,
    ];
    p.stops.push(ColorStop {
        offset: new_off,
        color,
    });
    Some(p.stops.len() - 1)
}

/// Remove gradient `stop`. No-op when only two stops remain (a gradient needs ≥2)
/// or `stop` is out of range. Mirror of `remove_curve_point`.
pub fn remove_gradient_stop(p: &mut GradientMapParams, stop: usize) {
    if p.stops.len() > 2 && stop < p.stops.len() {
        p.stops.remove(stop);
    }
}

/// Gradient Map — remaps each pixel's DISPLAY-space luma (Rec.601, like Threshold)
/// to a color along the gradient ([`gradient_map_lut`], the same table the GPU
/// binds). Builds the LUT once, then the per-pixel loop is a luma + lerped lookup.
/// `acc` is straight LINEAR f32 RGBA (alpha preserved). Always applies (a fresh
/// Gradient Map is a visible remap, like Posterize).
pub(crate) fn apply_gradient_map(p: &GradientMapParams, acc: &mut [[f32; 4]]) {
    let lut = gradient_map_lut(p);
    let encode = build_lut(linear_to_srgb_f32); // luma is computed in display space
    for px in acc.iter_mut() {
        let luma = 0.299 * sample_lut(&encode, px[0])
            + 0.587 * sample_lut(&encode, px[1])
            + 0.114 * sample_lut(&encode, px[2]);
        let t = luma.clamp(0.0, 1.0) * 255.0;
        let i = t as usize;
        let frac = t - i as f32;
        let a = lut[i.min(255)];
        let b = lut[(i + 1).min(255)];
        for ch in 0..3 {
            px[ch] = a[ch] + (b[ch] - a[ch]) * frac;
        }
    }
}

/// The CMYK adjustment of Selective-Color color-group `bucket` (0 = Reds,
/// 1 = Yellows, 2 = Greens, 3 = Cyans, 4 = Blues, 5 = Magentas, 6 = Whites,
/// 7 = Neutrals, 8 = Blacks).
fn selcolor_bucket(p: &SelectiveColorParams, bucket: usize) -> CmykAdjust {
    match bucket {
        0 => p.reds,
        1 => p.yellows,
        2 => p.greens,
        3 => p.cyans,
        4 => p.blues,
        5 => p.magentas,
        6 => p.whites,
        7 => p.neutrals,
        _ => p.blacks,
    }
}

/// Mutable [`selcolor_bucket`].
fn selcolor_bucket_mut(p: &mut SelectiveColorParams, bucket: usize) -> &mut CmykAdjust {
    match bucket {
        0 => &mut p.reds,
        1 => &mut p.yellows,
        2 => &mut p.greens,
        3 => &mut p.cyans,
        4 => &mut p.blues,
        5 => &mut p.magentas,
        6 => &mut p.whites,
        7 => &mut p.neutrals,
        _ => &mut p.blacks,
    }
}

/// The 9 Selective-Color group labels (the bucket-selector order).
pub const SELCOLOR_BUCKETS: [&str; 9] = [
    "Reds", "Yellows", "Greens", "Cyans", "Blues", "Magentas", "Whites", "Neutrals", "Blacks",
];

/// Selective Color — a CMYK adjustment applied per color group in DISPLAY space.
/// Each pixel is weighted into the 6 chromatic groups (the RGB hue hexagon, like
/// Black & White) plus 3 achromatic groups (Whites/Neutrals/Blacks via luma tonal
/// masks, biased toward low-chroma pixels); the matching groups' CMYK shifts are
/// accumulated and applied (C/M/Y subtract R/G/B, K darkens all). `Relative`
/// scales the shift by the channel's existing value; `Absolute` is a flat shift.
/// `acc` is straight LINEAR f32 RGBA (alpha preserved). All-zero groups
/// early-return an exact identity.
pub(crate) fn apply_selective_color(p: &SelectiveColorParams, acc: &mut [[f32; 4]]) {
    let buckets: [CmykAdjust; 9] = core::array::from_fn(|i| selcolor_bucket(p, i));
    if buckets
        .iter()
        .all(|c| c.cyan == 0.0 && c.magenta == 0.0 && c.yellow == 0.0 && c.black == 0.0)
    {
        return;
    }
    let relative = matches!(p.method, SelectiveMethod::Relative);
    for px in acc.iter_mut() {
        let (r, g, b) = (
            linear_to_srgb_f32(px[0]),
            linear_to_srgb_f32(px[1]),
            linear_to_srgb_f32(px[2]),
        );
        let m = r.min(g).min(b);
        let chroma = r.max(g).max(b) - m;
        let (rr, gg, bb) = (r - m, g - m, b - m);
        // 6 chromatic group weights (hue-hexagon decomposition).
        let (reds, yellows, greens, cyans, blues, magentas) = if b <= r && b <= g {
            let y = rr.min(gg);
            (rr - y, y, gg - y, 0.0, 0.0, 0.0)
        } else if r <= g && r <= b {
            let c = gg.min(bb);
            (0.0, 0.0, gg - c, c, bb - c, 0.0)
        } else {
            let mg = bb.min(rr);
            (rr - mg, 0.0, 0.0, 0.0, bb - mg, mg)
        };
        // 3 achromatic group weights (luma tonal masks, biased to low chroma).
        let luma = 0.299 * r + 0.587 * g + 0.114 * b;
        let achroma = 1.0 - chroma;
        let blacks = (1.0 - luma) * (1.0 - luma) * achroma;
        let whites = luma * luma * achroma;
        let mid = 2.0 * luma - 1.0;
        let neutrals = (1.0 - mid * mid) * achroma;
        let w = [
            reds, yellows, greens, cyans, blues, magentas, whites, neutrals, blacks,
        ];
        let (mut tc, mut tm, mut ty, mut tk) = (0.0, 0.0, 0.0, 0.0);
        for (wi, bc) in w.iter().zip(buckets.iter()) {
            tc += wi * bc.cyan;
            tm += wi * bc.magenta;
            ty += wi * bc.yellow;
            tk += wi * bc.black;
        }
        let (nr, ng, nb) = if relative {
            (r - (tc + tk) * r, g - (tm + tk) * g, b - (ty + tk) * b)
        } else {
            (r - (tc + tk), g - (tm + tk), b - (ty + tk))
        };
        px[0] = srgb_to_linear_f32(nr.clamp(0.0, 1.0));
        px[1] = srgb_to_linear_f32(ng.clamp(0.0, 1.0));
        px[2] = srgb_to_linear_f32(nb.clamp(0.0, 1.0));
    }
}

/// The 4 CMYK sliders (`Cyan/Magenta/Yellow/Black`, each `(label, value01)`,
/// `-1..1 → 0..1`) of Selective-Color group `bucket` — what the bespoke editor
/// renders for the active bucket tab. Inverse of [`set_selective_color_param`].
#[must_use]
pub fn selective_color_slider_params(
    p: &SelectiveColorParams,
    bucket: usize,
) -> Vec<(&'static str, f32)> {
    let c = selcolor_bucket(p, bucket);
    let s = |v: f32| (v.clamp(-1.0, 1.0) + 1.0) * 0.5;
    vec![
        ("Cyan", s(c.cyan)),
        ("Mag", s(c.magenta)),
        ("Yel", s(c.yellow)),
        ("Blk", s(c.black)),
    ]
}

/// Set CMYK slider `slot` (0 = C, 1 = M, 2 = Y, 3 = K) of Selective-Color group
/// `bucket` from a normalized `0..1` value (`→ -1..1`). Inverse of
/// [`selective_color_slider_params`]. Out-of-range slots no-op.
pub fn set_selective_color_param(
    p: &mut SelectiveColorParams,
    bucket: usize,
    slot: usize,
    value01: f32,
) {
    let v = value01.clamp(0.0, 1.0) * 2.0 - 1.0;
    let c = selcolor_bucket_mut(p, bucket);
    match slot {
        0 => c.cyan = v,
        1 => c.magenta = v,
        2 => c.yellow = v,
        3 => c.black = v,
        _ => {}
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
        // Color Balance: the 3 bipolar color-axis shifts (centered; 0.5 = neutral).
        // `scope` is a segment (see `adjustment_segment_params`) and
        // `preserve_luminosity` a toggle (see `adjustment_toggle_params`).
        AdjustmentParams::ColorBalance(p) => vec![
            ("C/R", (p.cyan_red.clamp(-1.0, 1.0) + 1.0) * 0.5),
            ("M/G", (p.magenta_green.clamp(-1.0, 1.0) + 1.0) * 0.5),
            ("Y/B", (p.yellow_blue.clamp(-1.0, 1.0) + 1.0) * 0.5),
        ],
        // Black & White: 6 per-hue weights (`-2..3 → 0..1`); the Tint toggle adds
        // the Hue + Tint-amount sliders (slots 6/7) only while a tint is set.
        AdjustmentParams::BlackAndWhite(p) => {
            let weight = |v: f32| (v.clamp(-2.0, 3.0) + 2.0) / 5.0;
            let mut v = vec![
                ("Reds", weight(p.reds)),
                ("Yellows", weight(p.yellows)),
                ("Greens", weight(p.greens)),
                ("Cyans", weight(p.cyans)),
                ("Blues", weight(p.blues)),
                ("Magentas", weight(p.magentas)),
            ];
            if let Some(t) = p.tint_color {
                v.push(("Hue", t.h.rem_euclid(360.0) / 360.0));
                v.push(("Tint", p.tint_amount.clamp(0.0, 1.0)));
            }
            v
        }
        // Gradient Map has a bespoke N-stop editor (preview bar + draggable stops
        // + the SELECTED stop's RGB sliders, see `gradient_stop_color_params`), so
        // it exposes no generic slider rack here.
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
        // W4 spatial mesh — the GPU pass-graph kinds. Radius/Distance 0..SPATIAL_PX,
        // Angle a full turn, Sharpen amount 0..2 (unsharp coef). Ranges are the
        // inverse of `set_adjustment_slider_param`.
        AdjustmentParams::GaussianBlur(p) => {
            vec![("Radius", (p.radius / SPATIAL_PX_MAX).clamp(0.0, 1.0))]
        }
        AdjustmentParams::MotionBlur(p) => vec![
            ("Distance", (p.distance / SPATIAL_PX_MAX).clamp(0.0, 1.0)),
            ("Angle", angle_to_slider(p.angle)),
        ],
        AdjustmentParams::Sharpen(p) => vec![
            ("Amount", (p.amount / SHARPEN_AMOUNT_MAX).clamp(0.0, 1.0)),
            ("Radius", (p.radius / SHARPEN_RADIUS_MAX).clamp(0.0, 1.0)),
        ],
        // Chromatic Aberration: 3 bipolar per-channel shifts (centered; 0.5 =
        // none). `falloff_center` is RESERVED (no slider — linear-radial model).
        AdjustmentParams::ChromaticAberration(p) => vec![
            ("Red", shift_to_slider(p.red_shift)),
            ("Green", shift_to_slider(p.green_shift)),
            ("Blue", shift_to_slider(p.blue_shift)),
        ],
        // Noise amount 0..1; the distribution (`kind`) is a segment + `mono` a
        // toggle (see the respective accessors).
        AdjustmentParams::Noise(p) => vec![("Amount", p.amount.clamp(0.0, 1.0))],
        // Halftone dot size 1..HALFTONE_DOT_MAX px + screen angle; the cell shape
        // is a segment.
        AdjustmentParams::Halftone(p) => vec![
            (
                "Dot Size",
                ((p.dot_size - 1.0) / (HALFTONE_DOT_MAX - 1.0)).clamp(0.0, 1.0),
            ),
            ("Angle", angle_to_slider(p.angle)),
        ],
        // Color Lookup: scrub the built-in look (handle = preset index, quantized
        // on the preset grid) + the intensity. A named popover is a UI follow-up.
        AdjustmentParams::ColorLookupLut(p) => vec![
            ("Look", preset_to_slider(p.lut_3d.0)),
            ("Amount", p.intensity.clamp(0.0, 1.0)),
        ],
        // Bloom: bright-pass threshold + glow intensity (0..2) + blur radius +
        // soft-knee falloff.
        AdjustmentParams::Bloom(p) => vec![
            ("Threshold", p.threshold.clamp(0.0, 1.0)),
            (
                "Intensity",
                (p.intensity / BLOOM_INTENSITY_MAX).clamp(0.0, 1.0),
            ),
            ("Radius", (p.radius / SPATIAL_PX_MAX).clamp(0.0, 1.0)),
            ("Falloff", p.falloff.clamp(0.0, 1.0)),
        ],
        // Shadows/Highlights: 8 params — shadows (amount/width/radius), highlights
        // (amount/width/radius), color correction (bipolar), midtone contrast
        // (bipolar). Fills the 8-slot generic rack exactly.
        AdjustmentParams::ShadowsHighlights(p) => vec![
            ("Shad Amt", p.shadows_amount.clamp(0.0, 1.0)),
            ("Shad Wid", p.shadows_tonal_width.clamp(0.0, 1.0)),
            (
                "Shad Rad",
                (p.shadows_radius / SPATIAL_PX_MAX).clamp(0.0, 1.0),
            ),
            ("High Amt", p.highlights_amount.clamp(0.0, 1.0)),
            ("High Wid", p.highlights_tonal_width.clamp(0.0, 1.0)),
            (
                "High Rad",
                (p.highlights_radius / SPATIAL_PX_MAX).clamp(0.0, 1.0),
            ),
            ("Color", (p.color_correction.clamp(-1.0, 1.0) + 1.0) * 0.5),
            (
                "Contrast",
                (p.midtone_contrast.clamp(-1.0, 1.0) + 1.0) * 0.5,
            ),
        ],
        _ => Vec::new(),
    }
}

/// Max bloom glow intensity exposed on the slider (the additive-glow multiplier).
const BLOOM_INTENSITY_MAX: f32 = 2.0;

/// Built-in Color Lookup preset index → `0..1` slider (quantized on the preset
/// grid; index 0 = None at the far left).
#[inline]
fn preset_to_slider(handle: u64) -> f32 {
    let last = (super::lut::LUT_PRESET_COUNT - 1).max(1) as f32;
    (handle.min(super::lut::LUT_PRESET_COUNT as u64 - 1) as f32 / last).clamp(0.0, 1.0)
}

/// `0..1` slider → built-in Color Lookup preset index (inverse of
/// [`preset_to_slider`]; snaps to the nearest preset).
#[inline]
fn slider_to_preset(v: f32) -> u64 {
    let last = (super::lut::LUT_PRESET_COUNT - 1) as f32;
    (v.clamp(0.0, 1.0) * last).round() as u64
}

// ── W4 spatial/coordinate slider ranges (UI ↔ param mapping, single source) ──
//
// The panel slider thumb is a normalized `0..1`; these constants are the physical
// extents `adjustment_slider_params` / `set_adjustment_slider_param` map to/from.

/// Max blur radius / motion distance exposed on the slider (px). The kernel cap
/// (`MAX_BLUR_HALF = 256`) is far past any interactive use; 100 px is a generous
/// editable range with a usable thumb resolution.
const SPATIAL_PX_MAX: f32 = 100.0;
/// Max unsharp-mask amount (the `base + amount·(base−blur)` coefficient).
const SHARPEN_AMOUNT_MAX: f32 = 2.0;
/// Max sharpen blur radius (px) — sharpening uses a small support.
const SHARPEN_RADIUS_MAX: f32 = 20.0;
/// Half-range of a chromatic-aberration per-channel shift (px at the canvas
/// corner); the slider is bipolar `−MAX..+MAX` (0.5 = no shift).
const CHROMA_SHIFT_MAX: f32 = 10.0;
/// Max halftone cell size (px); the minimum is 1 px (a single-pixel screen).
const HALFTONE_DOT_MAX: f32 = 32.0;

/// Radians angle → `0..1` slider (one full turn).
#[inline]
fn angle_to_slider(angle: f32) -> f32 {
    angle.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU
}

/// `0..1` slider → radians (inverse of [`angle_to_slider`]).
#[inline]
fn slider_to_angle(v: f32) -> f32 {
    v * std::f32::consts::TAU
}

/// Bipolar px shift `−MAX..+MAX` → `0..1` slider (0.5 = no shift).
#[inline]
fn shift_to_slider(shift: f32) -> f32 {
    ((shift / CHROMA_SHIFT_MAX).clamp(-1.0, 1.0) + 1.0) * 0.5
}

/// `0..1` slider → bipolar px shift (inverse of [`shift_to_slider`]).
#[inline]
fn slider_to_shift(v: f32) -> f32 {
    (v * 2.0 - 1.0) * CHROMA_SHIFT_MAX
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
        AdjustmentParams::ColorBalance(p) => match slot {
            0 => p.cyan_red = v * 2.0 - 1.0,      // -1..1
            1 => p.magenta_green = v * 2.0 - 1.0, // -1..1
            2 => p.yellow_blue = v * 2.0 - 1.0,   // -1..1
            _ => {}
        },
        AdjustmentParams::BlackAndWhite(p) => {
            let weight = v * 5.0 - 2.0; // 0..1 → -2..3
            match slot {
                0 => p.reds = weight,
                1 => p.yellows = weight,
                2 => p.greens = weight,
                3 => p.cyans = weight,
                4 => p.blues = weight,
                5 => p.magentas = weight,
                // Tint Hue / amount (only meaningful while a tint is set).
                6 => {
                    if let Some(t) = &mut p.tint_color {
                        t.h = v * 360.0;
                    }
                }
                7 => p.tint_amount = v,
                _ => {}
            }
        }
        AdjustmentParams::Levels(p) => match slot {
            0 => p.black_point = v,
            1 => p.gamma = levels_slider_to_gamma(v),
            2 => p.white_point = v,
            3 => p.output_black = v,
            4 => p.output_white = v,
            _ => {}
        },
        // W4 spatial mesh — inverse of `adjustment_slider_params`.
        AdjustmentParams::GaussianBlur(p) if slot == 0 => p.radius = v * SPATIAL_PX_MAX,
        AdjustmentParams::MotionBlur(p) => match slot {
            0 => p.distance = v * SPATIAL_PX_MAX,
            1 => p.angle = slider_to_angle(v),
            _ => {}
        },
        AdjustmentParams::Sharpen(p) => match slot {
            0 => p.amount = v * SHARPEN_AMOUNT_MAX,
            1 => p.radius = v * SHARPEN_RADIUS_MAX,
            _ => {}
        },
        AdjustmentParams::ChromaticAberration(p) => match slot {
            0 => p.red_shift = slider_to_shift(v),
            1 => p.green_shift = slider_to_shift(v),
            2 => p.blue_shift = slider_to_shift(v),
            _ => {}
        },
        AdjustmentParams::Noise(p) if slot == 0 => p.amount = v,
        AdjustmentParams::Halftone(p) => match slot {
            0 => p.dot_size = 1.0 + v * (HALFTONE_DOT_MAX - 1.0),
            1 => p.angle = slider_to_angle(v),
            _ => {}
        },
        AdjustmentParams::ColorLookupLut(p) => match slot {
            0 => p.lut_3d = LutHandle(slider_to_preset(v)),
            1 => p.intensity = v,
            _ => {}
        },
        AdjustmentParams::Bloom(p) => match slot {
            0 => p.threshold = v,
            1 => p.intensity = v * BLOOM_INTENSITY_MAX,
            2 => p.radius = v * SPATIAL_PX_MAX,
            3 => p.falloff = v,
            _ => {}
        },
        AdjustmentParams::ShadowsHighlights(p) => match slot {
            0 => p.shadows_amount = v,
            1 => p.shadows_tonal_width = v,
            2 => p.shadows_radius = v * SPATIAL_PX_MAX,
            3 => p.highlights_amount = v,
            4 => p.highlights_tonal_width = v,
            5 => p.highlights_radius = v * SPATIAL_PX_MAX,
            6 => p.color_correction = v * 2.0 - 1.0, // -1..1
            7 => p.midtone_contrast = v * 2.0 - 1.0, // -1..1
            _ => {}
        },
        // Gradient Map has a bespoke editor (its stop colors go through
        // `set_gradient_stop_color_param`), so no generic slider slot here.
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
        AdjustmentParams::ColorBalance(p) => vec![("Preserve Lum.", p.preserve_luminosity)],
        AdjustmentParams::ChannelMixer(p) => vec![("Monochrome", p.monochromatic)],
        AdjustmentParams::BlackAndWhite(p) => vec![("Tint", p.tint_color.is_some())],
        // Noise: monochrome = one luma grain vs independent per-channel. (Sharpen's
        // `mask_edges` is intentionally NOT exposed yet — its gate is a deferred
        // joint CPU+GPU follow-up, so a toggle here would be a no-op affordance.)
        AdjustmentParams::Noise(p) => vec![("Monochrome", p.monochromatic)],
        _ => Vec::new(),
    }
}

/// Set toggle `slot` of an adjustment (inverse of [`adjustment_toggle_params`]).
/// Out-of-range slots / non-toggle kinds no-op.
pub fn set_adjustment_toggle_param(params: &mut AdjustmentParams, slot: usize, on: bool) {
    match params {
        AdjustmentParams::PhotoFilter(p) if slot == 0 => p.preserve_luminosity = on,
        AdjustmentParams::ColorBalance(p) if slot == 0 => p.preserve_luminosity = on,
        AdjustmentParams::ChannelMixer(p) if slot == 0 => p.monochromatic = on,
        AdjustmentParams::Noise(p) if slot == 0 => p.monochromatic = on,
        // Black & White Tint: enabling seeds a classic warm sepia + a visible
        // amount (so the toggle has an immediate effect); the Hue/Tint sliders
        // then refine it. Disabling drops the tint back to a plain grayscale.
        AdjustmentParams::BlackAndWhite(p) if slot == 0 => {
            if on {
                p.tint_color = Some(OklchColor::opaque(0.7, 0.1, 70.0));
                if p.tint_amount == 0.0 {
                    p.tint_amount = 0.5;
                }
            } else {
                p.tint_color = None;
            }
        }
        _ => {}
    }
}

/// The single segmented (1-of-N, N ≤ 3) param of an adjustment, as
/// `(options, selected)` — what the layers panel renders as a segment-button row
/// (mirror of the Curves channel tabs). The inverse setter is
/// [`set_adjustment_segment_param`]. Kinds with no segmented param return `None`.
/// Currently the Color-Balance tonal range (Shadows / Midtones / Highlights).
#[must_use]
pub fn adjustment_segment_params(params: &AdjustmentParams) -> Option<(Vec<&'static str>, usize)> {
    match params {
        AdjustmentParams::ColorBalance(p) => Some((
            vec!["Shadows", "Midtones", "Highlights"],
            match p.scope {
                ToneScope::Shadows => 0,
                ToneScope::Midtones => 1,
                ToneScope::Highlights => 2,
            },
        )),
        AdjustmentParams::GradientMap(p) => Some((
            vec!["Linear", "Smooth"],
            match p.interpolation {
                GradientInterp::Linear => 0,
                GradientInterp::Smooth => 1,
            },
        )),
        AdjustmentParams::SelectiveColor(p) => Some((
            vec!["Relative", "Absolute"],
            match p.method {
                SelectiveMethod::Relative => 0,
                SelectiveMethod::Absolute => 1,
            },
        )),
        // W4 — Noise distribution + Halftone cell shape.
        AdjustmentParams::Noise(p) => Some((
            vec!["Gaussian", "Uniform"],
            match p.kind {
                NoiseKind::Gaussian => 0,
                NoiseKind::Uniform => 1,
            },
        )),
        AdjustmentParams::Halftone(p) => Some((
            vec!["Dot", "Line", "Circle"],
            match p.shape {
                HalftoneShape::Dot => 0,
                HalftoneShape::Line => 1,
                HalftoneShape::Circle => 2,
            },
        )),
        _ => None,
    }
}

/// Select option `option` of an adjustment's segmented param (inverse of
/// [`adjustment_segment_params`]). Out-of-range options clamp to the nearest
/// valid; non-segmented kinds no-op.
pub fn set_adjustment_segment_param(params: &mut AdjustmentParams, option: usize) {
    match params {
        AdjustmentParams::ColorBalance(p) => {
            p.scope = match option {
                0 => ToneScope::Shadows,
                2 => ToneScope::Highlights,
                _ => ToneScope::Midtones,
            };
        }
        AdjustmentParams::GradientMap(p) => {
            p.interpolation = if option == 1 {
                GradientInterp::Smooth
            } else {
                GradientInterp::Linear
            };
        }
        AdjustmentParams::SelectiveColor(p) => {
            p.method = if option == 1 {
                SelectiveMethod::Absolute
            } else {
                SelectiveMethod::Relative
            };
        }
        AdjustmentParams::Noise(p) => {
            p.kind = if option == 1 {
                NoiseKind::Uniform
            } else {
                NoiseKind::Gaussian
            };
        }
        AdjustmentParams::Halftone(p) => {
            p.shape = match option {
                1 => HalftoneShape::Line,
                2 => HalftoneShape::Circle,
                _ => HalftoneShape::Dot,
            };
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
