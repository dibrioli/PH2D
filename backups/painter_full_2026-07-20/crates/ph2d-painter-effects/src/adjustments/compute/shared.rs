//! Shared compute helpers: f32 sRGB↔linear transfer + per-call 1-D / display-space
//! LUT machinery. Used by every adjustment family below (and by `lut.rs` /
//! `spatial.rs` via `super::compute::*`). Split out of the former monolithic
//! `compute.rs` (pure mechanical move).

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
pub(crate) fn build_display_lut<F: Fn(f32) -> f32>(f: F) -> [f32; DISPLAY_LUT_N] {
    core::array::from_fn(|i| f(i as f32 / (DISPLAY_LUT_N - 1) as f32))
}

/// Sample a [`build_display_lut`] table at display `s` (`0..=1`) with linear
/// interpolation. The GPU samples the same table with a nearest lookup; the
/// difference is bounded by one 8-bit step and absorbed by the ±tolerance
/// GPU↔CPU parity gate.
#[inline]
pub(crate) fn sample_display_lut(lut: &[f32; DISPLAY_LUT_N], s: f32) -> f32 {
    let t = s.clamp(0.0, 1.0) * (DISPLAY_LUT_N - 1) as f32;
    let i = t as usize;
    let frac = t - i as f32;
    let a = lut[i];
    let b = lut[(i + 1).min(DISPLAY_LUT_N - 1)];
    a + (b - a) * frac
}
