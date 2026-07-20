//! Color Balance — Photoshop-style per-channel tonal-range-weighted color shift
//! in display space (per-channel transfer LUTs + optional luma renorm). Split out
//! of the former monolithic `compute.rs` (pure move).

use super::*;

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
