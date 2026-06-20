//! Levels — Photoshop-style display-space black/gamma/white input remap + output
//! remap (channel-uniform LUT). Split out of the former monolithic `compute.rs`
//! (pure move).

use super::*;

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
pub(crate) fn levels_gamma_to_slider(gamma: f32) -> f32 {
    (gamma.max(1e-3).log10() / 2.0 + 0.5).clamp(0.0, 1.0)
}

/// Inverse of [`levels_gamma_to_slider`].
pub(crate) fn levels_slider_to_gamma(s: f32) -> f32 {
    10.0_f32.powf((s.clamp(0.0, 1.0) - 0.5) * 2.0)
}
