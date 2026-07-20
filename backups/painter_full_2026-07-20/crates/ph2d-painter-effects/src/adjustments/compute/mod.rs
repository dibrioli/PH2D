//! Adjustment computation: `apply_adjustment` dispatch + per-kind kernels +
//! LUT helpers + slider param accessors. Originally split out of the former
//! `adjustments.rs` (pure mechanical move); further split BY FAMILY into the
//! sibling submodules below (god-file decomposition, 2026-06-20; pure move):
//!
//! - [`shared`] — sRGB↔linear transfer + 1-D / display-space LUT machinery.
//! - [`basic`] — simple per-pixel kernels (HSB / Exposure / Vibrance / Posterize
//!   / Threshold / Invert / Brightness-Contrast / Photo Filter / Black & White).
//! - [`curves`] / [`levels`] / [`color_balance`] / [`channel_mixer`] /
//!   [`gradient_map`] / [`selective_color`] — the bespoke adjustment families.
//! - [`params`] — the generic slider / toggle / segment UI param plumbing.
//!
//! Every family submodule's public + crate-visible symbols are re-exported here
//! (the glob `pub(crate) use … ::*` below), so the external surface is unchanged:
//! `super::compute::*` (used by `lut.rs` / `spatial.rs` / `tests.rs`) and the
//! `adjustments/mod.rs` re-export list still resolve to the SAME paths.

use super::*;

mod basic;
mod channel_mixer;
mod color_balance;
mod curves;
mod gradient_map;
mod levels;
mod params;
mod selective_color;
mod shared;

// Re-export every family's symbols at the `compute` module root so the existing
// external paths (`super::compute::X`, `compute::*`, the `adjustments/mod.rs`
// `pub use` list) are byte-for-byte unchanged. A `pub use … ::*` glob re-exports
// each item at the MINIMUM of `pub` and the item's own visibility — so the
// family `pub fn`/`pub const` API stays `pub` (re-exportable from the crate) and
// the `pub(crate)` kernels / helpers stay `pub(crate)`.
pub(crate) use basic::*; // family is all `pub(crate)` kernels (no `pub` API to re-export)
pub use channel_mixer::*;
pub use color_balance::*;
pub use curves::*;
pub use gradient_map::*;
pub use levels::*;
pub use params::*;
pub use selective_color::*;
pub use shared::*;

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
