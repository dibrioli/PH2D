//! Color Lookup — built-in cinematic "looks" applied per pixel.
//!
//! The frozen contract carries a [`LutHandle`](super::LutHandle) (a `u64`) + an
//! `intensity` + a [`LutProfile`](super::LutProfile). v1 resolves the handle to
//! one of the built-in [`LUT_PRESETS`] (analytic colour functions authored in
//! normalised **sRGB**) and applies it directly — exact, no 3-D-LUT sampling
//! error, and no LUT cache to thread through the compute. Handle `0` is the
//! neutral pass-through (`None`), so a freshly-created Color Lookup layer is an
//! identity until the user picks a look (matches the other neutral defaults).
//!
//! **Follow-ups (noted to the Coord, out of this crate):**
//! - **Load a `.cube` file** — the handle would then resolve to a cached
//!   `Lut3d` (trilinear sample) instead of a preset index. The engine slots in
//!   behind the same [`apply_color_lookup`] entry point; the blocker is the
//!   native file dialog (shell), not the math. (`ph2d-tool-color-equalization`
//!   already has a display-space `LUT3D` + 15 presets a future satellite crate
//!   could share — a Coord-coordinated refactor, not a solo painter edit.)
//! - **Named preset dropdown** — v1 scrubs presets via the generic "Look"
//!   slider (the live preview is the feedback); a named popover (mirroring the
//!   "+ Adjustment" `Dropdown`) is a UI-polish follow-up.

use super::ColorLookupLutParams;
use super::compute::{linear_to_srgb_f32, srgb_to_linear_f32};

/// Rec.709 display luma of a normalised-sRGB triple.
#[inline]
fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[inline]
fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

// ── Preset colour functions (normalised sRGB `[0,1]` → `[0,1]`) ───────────────

/// Neutral pass-through (handle 0). Never actually invoked (the dispatch
/// short-circuits handle 0), but present so the index space starts at "None".
fn look_none(c: [f32; 3]) -> [f32; 3] {
    c
}

/// Warm — lift red, ease blue (golden-hour cast).
fn look_warm(c: [f32; 3]) -> [f32; 3] {
    [clamp01(c[0] * 1.05 + 0.03), c[1], clamp01(c[2] * 0.92)]
}

/// Cool — the inverse of [`look_warm`] (shade / overcast cast).
fn look_cool(c: [f32; 3]) -> [f32; 3] {
    [clamp01(c[0] * 0.92), c[1], clamp01(c[2] * 1.05 + 0.03)]
}

/// Teal & Orange — the blockbuster split-tone: shadows toward teal, highlights
/// toward orange, pivoting on luma.
fn look_teal_orange(c: [f32; 3]) -> [f32; 3] {
    let t = luma(c) - 0.5; // <0 shadows, >0 highlights
    [
        clamp01(c[0] + t * 0.18),
        clamp01(c[1] + t * 0.04),
        clamp01(c[2] - t * 0.18),
    ]
}

/// Vintage — lifted blacks + reduced contrast + a faint warm wash (faded film).
fn look_vintage(c: [f32; 3]) -> [f32; 3] {
    let fade = |v: f32| v * 0.82 + 0.09;
    [
        clamp01(fade(c[0]) + 0.02),
        clamp01(fade(c[1])),
        clamp01(fade(c[2]) - 0.02),
    ]
}

/// Noir — high-contrast black & white.
fn look_noir(c: [f32; 3]) -> [f32; 3] {
    let l = clamp01((luma(c) - 0.5) * 1.45 + 0.5);
    [l, l, l]
}

/// Sepia — warm monochrome (toned print).
fn look_sepia(c: [f32; 3]) -> [f32; 3] {
    let l = luma(c);
    [
        clamp01(l * 1.07 + 0.04),
        clamp01(l * 0.92),
        clamp01(l * 0.72),
    ]
}

/// Vivid — push every channel away from its luma (saturation boost).
fn look_vivid(c: [f32; 3]) -> [f32; 3] {
    let m = luma(c);
    [
        clamp01(m + (c[0] - m) * 1.35),
        clamp01(m + (c[1] - m) * 1.35),
        clamp01(m + (c[2] - m) * 1.35),
    ]
}

/// A built-in look transform: normalised **sRGB (display)** triple → sRGB triple.
/// (Audit 2026-06-18: looks operate in DISPLAY space — `apply_color_lookup` does
/// the linear→sRGB encode before calling and decodes after, per the module header
/// — NOT linear-RGB. Calibrate new looks in sRGB.)
pub type LookFn = fn([f32; 3]) -> [f32; 3];

/// The built-in looks the "Look" control scrubs through. Index 0 is the neutral
/// pass-through; the [`LutHandle`](super::LutHandle) stores this index. Labels are
/// English ([[feedback-app-ui-english-only]]) and feed a future named dropdown.
pub const LUT_PRESETS: &[(&str, LookFn)] = &[
    ("None", look_none),
    ("Warm", look_warm),
    ("Cool", look_cool),
    ("Teal & Orange", look_teal_orange),
    ("Vintage", look_vintage),
    ("Noir", look_noir),
    ("Sepia", look_sepia),
    ("Vivid", look_vivid),
];

/// Number of built-in looks (including `None`) — the "Look" slider's grid size.
pub const LUT_PRESET_COUNT: usize = LUT_PRESETS.len();

/// Apply the Color Lookup adjustment in place. The preset is authored in display
/// (sRGB) space, so each pixel round-trips linear→sRGB→linear around the look;
/// `intensity` blends toward it (0 = identity, 1 = full look). Coverage (alpha)
/// is preserved — a colour grade does not change shape. Handle 0 / out-of-range /
/// zero intensity is a no-op.
///
/// `profile` is honoured for a future loaded `.cube` (a `Linear`-authored LUT
/// would skip the sRGB round-trip); the built-in presets are sRGB, so v1 always
/// grades in display space.
pub fn apply_color_lookup(p: &ColorLookupLutParams, acc: &mut [[f32; 4]]) {
    let idx = p.lut_3d.0 as usize;
    let amount = p.intensity.clamp(0.0, 1.0);
    if idx == 0 || idx >= LUT_PRESETS.len() || amount <= 0.0 {
        return;
    }
    let look = LUT_PRESETS[idx].1;
    // Per-pixel + independent → parallel (the grade is 6 transcendentals/pixel).
    super::spatial::par_pixels(acc, |px| {
        let d = [
            linear_to_srgb_f32(px[0]),
            linear_to_srgb_f32(px[1]),
            linear_to_srgb_f32(px[2]),
        ];
        let graded = look(d);
        for c in 0..3 {
            let out_d = d[c] + (clamp01(graded[c]) - d[c]) * amount;
            px[c] = srgb_to_linear_f32(out_d);
        }
    });
}
