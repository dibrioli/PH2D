//! Channel Mixer — a 3×4 display-space matrix recombining source channels (+
//! monochrome) and its bespoke per-output slider params. Split out of the former
//! monolithic `compute.rs` (pure move).

use super::*;

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
