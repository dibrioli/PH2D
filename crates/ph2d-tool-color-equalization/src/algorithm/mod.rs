//! Color Equalization — pure CPU pipeline.
//!
//! `std`-only, no editor / ECS / external image deps. Operates on
//! straight-alpha RGBA8 (`w*h*4` bytes, row-major) and produces a fresh
//! RGBA8 buffer of the same dimensions. Stages, in pipeline order:
//!
//! 1. [`clahe`] — Contrast-Limited Adaptive Histogram Equalization on the
//!    BT.709 luminance channel (Zuiderveld 1994, *Graphics Gems IV*
//!    pp. 474-485).
//! 2. [`adjust_tonal`] — combined Phase 1 tonal pipeline batched in a
//!    single sRGB ↔ linear ↔ OKLab round-trip per pixel. Stages in order:
//!    Exposure → Temperature (Bradford) → Tint → Brightness → Contrast →
//!    Vibrance (OKLab) → Saturation (OKLab). Each stage is also exposed
//!    as a pure primitive (`apply_*_linear` / `apply_*_oklab`) for
//!    standalone tests and future WGSL parity.
//!    Phase 3 LUT color grading then runs in-line (procedural presets
//!    via [`crate::lut_presets`] → [`crate::lut::apply_lut3d`]; dual-slot
//!    blend by `lut_mix` + intensity attenuation; skipped when both
//!    slots are `None`).
//! 3. [`sharpen_laplacian`] (radius ≤ 1) or [`sharpen_unsharp`] (radius
//!    \> 1) — Phase 2 detail enhancement. Denoise stage was evaluated
//!    (Bilateral, NLM, Guided Filter, À-Trous, Domain Transform,
//!    Anisotropic Diffusion, TV-Chambolle, Wavelet Shrinkage) and
//!    removed 2026-05-27 — none met the visual bar.
//! 4. [`auto_levels`] / [`auto_contrast`] / [`auto_colors`] — optional
//!    post-tonal normalization toggles (Phase 2).
//! 5. [`auto_white_balance`] — Gray-World channel gains in sRGB.
//!
//! [`compute_histogram`] returns the per-channel + luma distribution and
//! powers both the panel's visual overlay and the auto-* percentile
//! analysis. [`run_pipeline`] threads everything together; each stage is
//! also usable standalone for tests / future GPU parity work.
//!
//! ## GPU port plan (follow-up)
//!
//! Every per-pixel stage in this module is embarrassingly parallel. A
//! single WGSL compute pass can fuse the Phase 1 tonal pipeline (the
//! seven stages + OKLab smart-sat pair) plus the CLAHE LUT apply step
//! plus auto-WB into one shader, doing exactly one sRGB → linear and
//! one OKLab round-trip per pixel. The legacy engine demonstrates the pattern in 799 LOC of
//! WebGL2 (`ceq-webgl.ts`). Histogram + Bradford-matrix precompute stay
//! CPU (atomic contention vs. sequential setup); the per-pixel apply is
//! GPU. Parity test (ε = 0.5 / 255) compares this CPU path against the
//! shader output on the same input.
//!
//! ## Module layout
//!
//! Split by pipeline stage (mechanical, no behaviour change). All public
//! items are re-exported flat at the `crate::algorithm::*` path so the
//! tool, panel, and GPU mirror crates keep their existing import paths:
//!
//! - [`clahe`] — Stage 1 CLAHE (`clahe`, per-tile LUT build).
//! - [`tonal`] — Stage 2 tonal primitives + batched [`adjust_tonal`].
//! - [`auto`] — Stage 3/4 auto-WB, histogram + auto-levels/contrast/colors.
//! - [`sharpen`] — Stage 5 Laplacian / Unsharp + Gaussian kernel.
//! - [`posterize_quantize`] — Stage 7 Posterize (Floyd-Steinberg) +
//!   Quantize (K-Means++ OKLab).
//! - [`util`] — shared `clamp8`, aspect-fit, bilinear resize.

mod auto;
mod clahe;
mod posterize_quantize;
mod sharpen;
mod tonal;
mod util;

pub use auto::{
    HistogramData, auto_colors, auto_contrast, auto_levels, auto_white_balance, compute_histogram,
};
pub use clahe::clahe;
pub use posterize_quantize::{
    POSTERIZE_LEVELS_MAX, POSTERIZE_LEVELS_MIN, QUANTIZE_COLORS_MAX, QUANTIZE_COLORS_MIN,
    posterize, quantize,
};
pub use sharpen::{gaussian_kernel_1d, sharpen_laplacian, sharpen_unsharp};
pub use tonal::{
    adjust_tonal, apply_brightness_linear, apply_contrast_linear, apply_exposure_linear,
    apply_saturation_oklab, apply_temperature_linear, apply_tint_linear, apply_vibrance_oklab,
    temperature01_to_kelvin,
};
pub use util::{aspect_fit_within, resize_bilinear_rgba};

use crate::lut::{DEFAULT_LUT_SIZE, apply_lut3d, blend_luts};
use crate::lut_presets::generate_preset_lut;
use crate::params::ColorEqualizationParams;

/// Run the full Color Equalization pipeline against `rgba` (straight-alpha
/// RGBA8, `w * h * 4` bytes) with `params`, writing into `out` (resized
/// to match). Stages run in order: CLAHE → brightness/contrast/saturation
/// → optional auto-WB.
///
/// The output buffer is reused across runs (HR-3) — the caller owns it.
/// Caller may pass an empty `Vec<u8>` on first call; subsequent calls
/// reuse the allocation.
pub fn run_pipeline(
    pixels: &[ph2d_color::SrgbRgba],
    w: u32,
    h: u32,
    params: &ColorEqualizationParams,
    out: &mut Vec<u8>,
) {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    let expected = (w as usize) * (h as usize) * 4;
    assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
    out.clear();
    out.resize(expected, 0);
    if w == 0 || h == 0 {
        return;
    }

    // Fast-path: identity params → copy source through, skip every
    // stage. Mirrors the GPU chain's zero-dispatch shortcut
    // (`chain_identity_params_short_circuits_to_no_dispatches`).
    if params.is_noop() {
        out.copy_from_slice(rgba);
        return;
    }

    // Stage 1 — CLAHE (writes through into `out`). Skipped at the
    // identity clip limit so the per-tile CDF reconstruction can't
    // tint the image when CLAHE is effectively off.
    if params.clip_limit > crate::params::CLIP_LIMIT_MIN {
        clahe(rgba, w, h, params.clip_limit, params.tile_grid_size, out);
    } else {
        out.copy_from_slice(rgba);
    }

    // Stage 2 — combined Phase 1 tonal pipeline in a single sRGB↔linear
    // (and optional OKLab) round-trip per pixel. Skipped when ALL params
    // are at identity to keep the no-op cheap.
    if !params.tonal_is_identity() {
        adjust_tonal(out, params);
    }

    // Stage 2.5 — Phase 3 LUT color grading. Procedural presets are
    // materialised here on-demand (≈ 5-15 ms per active preset at the
    // default 17³ size; bypassed entirely when both slots are `None`
    // or `lut_intensity` is `0`). Dual-LUT case pre-blends the two
    // LUTs by `lut_mix` so the per-pixel apply pass only samples one
    // cube. A wgpu compute follow-up replaces this CPU loop with one
    // `textureSampleLevel(lut3d, ...)` per pixel.
    if !params.lut_is_identity() {
        let lut1 = generate_preset_lut(params.lut_preset_1, DEFAULT_LUT_SIZE);
        let lut2 = generate_preset_lut(params.lut_preset_2, DEFAULT_LUT_SIZE);
        match (lut1, lut2) {
            (Some(a), Some(b)) => {
                let blended = blend_luts(&a, &b, params.lut_mix);
                apply_lut3d(out, &blended, params.lut_intensity);
            }
            (Some(a), None) => apply_lut3d(out, &a, params.lut_intensity),
            (None, Some(b)) => apply_lut3d(out, &b, params.lut_intensity),
            (None, None) => {}
        }
    }

    // Stage 3 — Phase 2 sharpen. Small radius (≤ 1) takes the fast
    // Laplacian 3×3; larger radius takes Unsharp Mask (Gaussian blur).
    if params.sharpen_amount > 0.0 {
        if params.sharpen_radius <= 1.0 {
            sharpen_laplacian(out, w, h, params.sharpen_amount);
        } else {
            sharpen_unsharp(out, w, h, params.sharpen_amount, params.sharpen_radius);
        }
    }

    // Stage 4 — Phase 2 optional automatic adjustments. Each is a toggle
    // applied AFTER tonal so it normalises the user's adjustments rather
    // than fighting them.
    if params.auto_levels {
        auto_levels(out);
    }
    if params.auto_contrast {
        auto_contrast(out);
    }
    if params.auto_colors {
        auto_colors(out);
    }

    // Stage 5 — Gray-World auto white balance (also in place over `out`).
    if params.auto_wb {
        auto_white_balance(out);
    }

    // Stage 6 — Posterize (Floyd-Steinberg dithering optional). Always
    // CPU — the error-diffusion sweep is strict raster-scan. Runs after
    // all colour-shift stages so it operates on the final palette.
    if params.posterize_levels >= POSTERIZE_LEVELS_MIN {
        posterize(
            out,
            w,
            h,
            params.posterize_levels,
            params.posterize_dithering,
            params.posterize_dither_strength,
            params.posterize_dither_grain,
        );
    }

    // Stage 7 — Quantize (K-Means++ in OKLab). Always CPU. Runs LAST —
    // every prior stage feeds into the colour set that gets clustered.
    if params.quantize_colors >= QUANTIZE_COLORS_MIN {
        quantize(out, w, h, params.quantize_colors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ColorEqualizationParams;

    /// 4×4 RGBA8 with a single solid colour + opaque alpha.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn run_pipeline_preserves_dimensions() {
        let src = solid(8, 8, [120, 80, 200]);
        let p = ColorEqualizationParams::default();
        let mut out = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p, &mut out);
        assert_eq!(out.len(), src.len());
    }

    #[test]
    fn run_pipeline_identity_round_trip_exact() {
        // Phase 1 audit (2026-05): with `ColorEqualizationParams::default()`
        // the pipeline must produce the source byte-for-byte. Defaults are
        // engineered identity (`CLIP_LIMIT_DEFAULT = CLIP_LIMIT_MIN`,
        // every tonal knob at its identity value, every Phase 2 toggle
        // off, no LUT preset, no posterize / quantize). The fast-path
        // guard in `run_pipeline` short-circuits on `is_noop()`; this
        // test pins the guarantee so a future stage author can't break
        // the "activating the tool with no edits is a no-op" invariant.
        //
        // Source spans every alpha state (opaque + semi-transparent +
        // fully transparent) and four primary hues so any stage that
        // sneaks in a unilateral mutation would diverge here.
        let p = ColorEqualizationParams::default();
        assert!(
            p.is_noop(),
            "test precondition: default params must be is_noop()"
        );
        let mut src = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16u8 {
            for x in 0..16u8 {
                let r = x.wrapping_mul(17);
                let g = y.wrapping_mul(17);
                let b = (x ^ y).wrapping_mul(17);
                let a = match (x % 4, y % 4) {
                    (0, _) => 0,
                    (1, _) => 64,
                    (2, _) => 128,
                    _ => 255,
                };
                src.extend_from_slice(&[r, g, b, a]);
            }
        }
        let mut out = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 16, 16, &p, &mut out);
        assert_eq!(out, src, "default params must round-trip identity");
    }

    #[test]
    fn run_pipeline_auto_wb_toggle_changes_output() {
        // Compose-level verification: with a red-cast input, toggling
        // auto-WB on must change the pipeline output relative to the same
        // pipeline with auto-WB off. (The pure auto-WB stage is exercised
        // by `auto_wb_balances_red_cast`; this test just confirms the
        // pipeline threads the flag through.)
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for y in 0..8u32 {
            for x in 0..8u32 {
                let jitter = ((x + y) % 16) as u8;
                src.extend_from_slice(&[200 - jitter / 2, 100 + jitter, 100 + jitter, 255]);
            }
        }
        let p_off = ColorEqualizationParams {
            auto_wb: false,
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            auto_wb: true,
            ..ColorEqualizationParams::default()
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_off, &mut out_off);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_on, &mut out_on);
        assert_ne!(
            out_off, out_on,
            "auto-wb flag did not affect pipeline output"
        );
    }

    #[test]
    fn run_pipeline_lut_preset_toggle_changes_output() {
        // Activate Sepia in slot 1 — output should diverge from the
        // neutral CLAHE baseline (warm cast collapses chroma toward
        // sepia tones).
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            src.extend_from_slice(&[
                (i * 3 % 256) as u8,
                (i * 5 % 256) as u8,
                (i * 7 % 256) as u8,
                255,
            ]);
        }
        let p_off = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Sepia,
            ..p_off
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_off, &mut out_off);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_on, &mut out_on);
        assert_ne!(out_off, out_on, "LUT preset toggle did not change output");
    }

    #[test]
    fn run_pipeline_dual_lut_blend_changes_output_at_midpoint() {
        // With slot 1 = Warm + slot 2 = Cool + mix = 0.5, the output
        // should sit between the two preset extremes (neither pure warm
        // nor pure cool).
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            src.extend_from_slice(&[128 + (i * 2 % 64) as u8, 128, 128 + (i * 3 % 64) as u8, 255]);
        }
        let base = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        let p_warm = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Warm,
            ..base
        };
        let p_cool = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Cool,
            ..base
        };
        let p_blend = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Warm,
            lut_preset_2: crate::lut_presets::LutPreset::Cool,
            lut_mix: 0.5,
            ..base
        };
        let mut warm = Vec::new();
        let mut cool = Vec::new();
        let mut blend = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_warm, &mut warm);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_cool, &mut cool);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_blend, &mut blend);
        assert_ne!(blend, warm, "blend should not equal pure-warm");
        assert_ne!(blend, cool, "blend should not equal pure-cool");
    }

    #[test]
    fn run_pipeline_lut_intensity_zero_is_noop_relative_to_baseline() {
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            src.extend_from_slice(&[
                (i * 3 % 256) as u8,
                (i * 5 % 256) as u8,
                (i * 7 % 256) as u8,
                255,
            ]);
        }
        let base = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        let p_zero = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Cinematic,
            lut_intensity: 0.0,
            ..base
        };
        let mut baseline = Vec::new();
        let mut zero = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &base, &mut baseline);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_zero, &mut zero);
        assert_eq!(
            baseline, zero,
            "intensity=0 should short-circuit the LUT stage entirely"
        );
    }

    #[test]
    fn run_pipeline_auto_levels_toggle_changes_output() {
        // Build a low-range input (R ∈ [80, 180]); Auto Levels should
        // stretch it noticeably.
        let mut src = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let r = 80u8 + (((x + y) % 100) as u8);
                src.extend_from_slice(&[r, 128, 128, 255]);
            }
        }
        let p_off = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN, // neutral CLAHE
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            auto_levels: true,
            ..p_off
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 16, 16, &p_off, &mut out_off);
        run_pipeline(bytemuck::cast_slice(&src), 16, 16, &p_on, &mut out_on);
        assert_ne!(out_off, out_on, "auto_levels toggle did not change output");
    }
}
