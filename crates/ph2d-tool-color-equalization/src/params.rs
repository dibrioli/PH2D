//! Parameters + UI projection for the stateful Color Equalization tool.
//!
//! Single source of truth for clamps + slider↔value mapping. The tool's
//! `handle_panel_event` routes panel NodeIds into [`ColorEqualizationUiEdit`]
//! variants and forwards them through [`apply_ui_edit`], where the clamp
//! lives exactly once.
//!
//! The panel crate paints normalized slider tracks (`0.0..=1.0`) and reads
//! a [`ColorEqualizationUiSnapshot`] published by the host once per frame
//! while the tool is active. The snapshot mirrors the params projected to
//! the same normalized space the sliders use.

/// Full-scale clip limit (Zuiderveld). Slider track `0..1` maps onto
/// `1.0..=CLIP_LIMIT_MAX` (the canonical CLAHE range). `1.0` collapses
/// to a uniform redistribution (no contrast boost — CLAHE off);
/// `2.0` matches the OpenCV reference; anything above ≈ 4.0 starts
/// to amplify noise harshly.
///
/// Default = `1.0` (identity / off). Audit fix: a default of `2.0`
/// (OpenCV reference) caused the tool to silently alter the image the
/// moment it activated, before the user touched any slider — and the
/// CLAHE chroma reconstruction in RGB amplifies that into visible
/// blotches in soft areas. Identity-by-default lets the user opt in.
pub const CLIP_LIMIT_MIN: f32 = 1.0;
pub const CLIP_LIMIT_MAX: f32 = 4.0;
pub const CLIP_LIMIT_DEFAULT: f32 = 1.0;

/// Tile grid size — image is partitioned into `N×N` square tiles.
/// `N = 8` is the canonical default (Zuiderveld); below `4` the
/// per-tile histogram becomes too sparse, above `16` interpolation
/// artefacts dominate. Slider integer in `[4, 16]`.
pub const TILE_GRID_MIN: u32 = 4;
pub const TILE_GRID_MAX: u32 = 16;
pub const TILE_GRID_DEFAULT: u32 = 8;

/// Phase 1 tonal stage ranges. All in normalized natural units; the
/// pipeline projects them onto the right working space (linear sRGB or
/// OKLab) inside [`crate::algorithm::adjust_tonal`].
///
/// - **Exposure** (EV stops, `[-3, +3]`, `0` = identity) — multiplicative
///   `pow(2, ev)` in linear sRGB with soft-knee highlight compression.
/// - **Temperature** (`[-1, +1]`) — photographer convention: `+1` warm
///   (target 2000K Bradford), `-1` cool (target 10000K), `0` D65 neutral.
/// - **Tint** (`[-1, +1]`) — green/magenta shift, luminance-preserving in
///   linear sRGB. `+1` magenta, `-1` green.
/// - **Brightness** (`[-1, +1]`) — multiplicative `m = 1 + b` in linear
///   sRGB. Pure black stays black (mirrors legacy semantics, distinct
///   from additive offset which lifts blacks to grey).
/// - **Contrast** (`[0.5, 2.0]`, `1.0` = identity) — S-curve around the
///   perceptual midpoint `0.18` (18 % grey) in linear sRGB.
/// - **Vibrance** (`[-1, +1]`) — smart saturation in OKLab; boosts chroma
///   inversely to current chroma (skin-tone protection).
/// - **Saturation** (`[-1, +1]`) — uniform chroma scale in OKLab.
pub const EXPOSURE_MIN: f32 = -3.0;
pub const EXPOSURE_MAX: f32 = 3.0;
pub const EXPOSURE_DEFAULT: f32 = 0.0;

pub const TEMPERATURE_MIN: f32 = -1.0;
pub const TEMPERATURE_MAX: f32 = 1.0;
pub const TEMPERATURE_DEFAULT: f32 = 0.0;

pub const TINT_MIN: f32 = -1.0;
pub const TINT_MAX: f32 = 1.0;
pub const TINT_DEFAULT: f32 = 0.0;

pub const BRIGHTNESS_MIN: f32 = -1.0;
pub const BRIGHTNESS_MAX: f32 = 1.0;
pub const BRIGHTNESS_DEFAULT: f32 = 0.0;

pub const CONTRAST_MIN: f32 = 0.5;
pub const CONTRAST_MAX: f32 = 2.0;
pub const CONTRAST_DEFAULT: f32 = 1.0;

pub const VIBRANCE_MIN: f32 = -1.0;
pub const VIBRANCE_MAX: f32 = 1.0;
pub const VIBRANCE_DEFAULT: f32 = 0.0;

pub const SATURATION_MIN: f32 = -1.0;
pub const SATURATION_MAX: f32 = 1.0;
pub const SATURATION_DEFAULT: f32 = 0.0;

// ── Phase 2 (Effects: sharpen) ────────────────────────────────────
//
// - **Sharpen amount** (`[0, 2]`): strength of the Laplacian / Unsharp
//   mask kernel. `0` = identity, `1` = canonical, `2` = aggressive.
// - **Sharpen radius** (`[0.5, 3.0]`): kernel reach. `≤ 1` uses the fast
//   Laplacian 3×3; `> 1` switches to Unsharp Mask (Gaussian blur).
pub const SHARPEN_AMOUNT_MIN: f32 = 0.0;
pub const SHARPEN_AMOUNT_MAX: f32 = 2.0;
pub const SHARPEN_AMOUNT_DEFAULT: f32 = 0.0;

pub const SHARPEN_RADIUS_MIN: f32 = 0.5;
pub const SHARPEN_RADIUS_MAX: f32 = 3.0;
pub const SHARPEN_RADIUS_DEFAULT: f32 = 1.0;

// ── Phase 3 (LUT color grading) ──────────────────────────────────
//
// Two slots for procedural LUT presets (`lut_preset_1`, `lut_preset_2`),
// blended by `lut_mix` (`0` = preset 1 only, `1` = preset 2 only) and
// then applied at `lut_intensity` (`0` = original, `1` = full LUT). When
// both presets are `LutPreset::None` the stage is skipped entirely.
pub const LUT_INTENSITY_MIN: f32 = 0.0;
pub const LUT_INTENSITY_MAX: f32 = 1.0;
pub const LUT_INTENSITY_DEFAULT: f32 = 1.0;

pub const LUT_MIX_MIN: f32 = 0.0;
pub const LUT_MIX_MAX: f32 = 1.0;
pub const LUT_MIX_DEFAULT: f32 = 0.5;

// ── Phase 5 (Posterize dither knobs) — Enio 2026-05-26 ────────────
pub const POSTERIZE_DITHER_STRENGTH_MIN: f32 = 0.0;
pub const POSTERIZE_DITHER_STRENGTH_MAX: f32 = 1.0;
pub const POSTERIZE_DITHER_STRENGTH_DEFAULT: f32 = 1.0;

pub const POSTERIZE_DITHER_GRAIN_MIN: u32 = 1;
pub const POSTERIZE_DITHER_GRAIN_MAX: u32 = 8;
pub const POSTERIZE_DITHER_GRAIN_DEFAULT: u32 = 1;

// ── Slider ↔ value projection helpers ─────────────────────────────
//
// The panel paints normalized track `0..1`; the tool stores the natural
// unit. Single mapping site keeps `ui_snapshot` (forward) and
// `apply_ui_edit` (inverse) in lock-step.

pub fn project01(v: f32, min: f32, max: f32) -> f32 {
    ((v - min) / (max - min)).clamp(0.0, 1.0)
}

fn unproject01(track: f32, min: f32, max: f32) -> f32 {
    min + track.clamp(0.0, 1.0) * (max - min)
}

pub fn clip_limit_to_slider(v: f32) -> f32 {
    project01(v, CLIP_LIMIT_MIN, CLIP_LIMIT_MAX)
}

pub fn slider_to_clip_limit(track: f32) -> f32 {
    unproject01(track, CLIP_LIMIT_MIN, CLIP_LIMIT_MAX)
}

pub fn tile_grid_to_slider(n: u32) -> f32 {
    project01(n as f32, TILE_GRID_MIN as f32, TILE_GRID_MAX as f32)
}

pub fn slider_to_tile_grid(track: f32) -> u32 {
    unproject01(track, TILE_GRID_MIN as f32, TILE_GRID_MAX as f32).round() as u32
}

pub fn exposure_to_slider(v: f32) -> f32 {
    project01(v, EXPOSURE_MIN, EXPOSURE_MAX)
}

pub fn slider_to_exposure(track: f32) -> f32 {
    unproject01(track, EXPOSURE_MIN, EXPOSURE_MAX)
}

pub fn temperature_to_slider(v: f32) -> f32 {
    project01(v, TEMPERATURE_MIN, TEMPERATURE_MAX)
}

pub fn slider_to_temperature(track: f32) -> f32 {
    unproject01(track, TEMPERATURE_MIN, TEMPERATURE_MAX)
}

pub fn tint_to_slider(v: f32) -> f32 {
    project01(v, TINT_MIN, TINT_MAX)
}

pub fn slider_to_tint(track: f32) -> f32 {
    unproject01(track, TINT_MIN, TINT_MAX)
}

pub fn brightness_to_slider(v: f32) -> f32 {
    project01(v, BRIGHTNESS_MIN, BRIGHTNESS_MAX)
}

pub fn slider_to_brightness(track: f32) -> f32 {
    unproject01(track, BRIGHTNESS_MIN, BRIGHTNESS_MAX)
}

pub fn contrast_to_slider(v: f32) -> f32 {
    project01(v, CONTRAST_MIN, CONTRAST_MAX)
}

pub fn slider_to_contrast(track: f32) -> f32 {
    unproject01(track, CONTRAST_MIN, CONTRAST_MAX)
}

pub fn vibrance_to_slider(v: f32) -> f32 {
    project01(v, VIBRANCE_MIN, VIBRANCE_MAX)
}

pub fn slider_to_vibrance(track: f32) -> f32 {
    unproject01(track, VIBRANCE_MIN, VIBRANCE_MAX)
}

pub fn saturation_to_slider(v: f32) -> f32 {
    project01(v, SATURATION_MIN, SATURATION_MAX)
}

pub fn slider_to_saturation(track: f32) -> f32 {
    unproject01(track, SATURATION_MIN, SATURATION_MAX)
}

pub fn sharpen_amount_to_slider(v: f32) -> f32 {
    project01(v, SHARPEN_AMOUNT_MIN, SHARPEN_AMOUNT_MAX)
}

pub fn slider_to_sharpen_amount(track: f32) -> f32 {
    unproject01(track, SHARPEN_AMOUNT_MIN, SHARPEN_AMOUNT_MAX)
}

pub fn sharpen_radius_to_slider(v: f32) -> f32 {
    project01(v, SHARPEN_RADIUS_MIN, SHARPEN_RADIUS_MAX)
}

pub fn slider_to_sharpen_radius(track: f32) -> f32 {
    unproject01(track, SHARPEN_RADIUS_MIN, SHARPEN_RADIUS_MAX)
}

pub fn lut_intensity_to_slider(v: f32) -> f32 {
    project01(v, LUT_INTENSITY_MIN, LUT_INTENSITY_MAX)
}

pub fn slider_to_lut_intensity(track: f32) -> f32 {
    unproject01(track, LUT_INTENSITY_MIN, LUT_INTENSITY_MAX)
}

pub fn lut_mix_to_slider(v: f32) -> f32 {
    project01(v, LUT_MIX_MIN, LUT_MIX_MAX)
}

pub fn slider_to_lut_mix(track: f32) -> f32 {
    unproject01(track, LUT_MIX_MIN, LUT_MIX_MAX)
}

pub fn posterize_dither_strength_to_slider(v: f32) -> f32 {
    project01(
        v,
        POSTERIZE_DITHER_STRENGTH_MIN,
        POSTERIZE_DITHER_STRENGTH_MAX,
    )
}

pub fn slider_to_posterize_dither_strength(track: f32) -> f32 {
    unproject01(
        track,
        POSTERIZE_DITHER_STRENGTH_MIN,
        POSTERIZE_DITHER_STRENGTH_MAX,
    )
}

pub fn posterize_dither_grain_to_slider(v: u32) -> f32 {
    project01(
        v as f32,
        POSTERIZE_DITHER_GRAIN_MIN as f32,
        POSTERIZE_DITHER_GRAIN_MAX as f32,
    )
}

pub fn slider_to_posterize_dither_grain(track: f32) -> u32 {
    let raw = unproject01(
        track,
        POSTERIZE_DITHER_GRAIN_MIN as f32,
        POSTERIZE_DITHER_GRAIN_MAX as f32,
    );
    raw.round().clamp(
        POSTERIZE_DITHER_GRAIN_MIN as f32,
        POSTERIZE_DITHER_GRAIN_MAX as f32,
    ) as u32
}

/// Authoritative parameter bag fed into [`crate::algorithm::run_pipeline`].
///
/// **Denoise stage removed (2026-05-27)** — Bilateral / NLM / Guided
/// Filter / À-Trous / Domain Transform / Anisotropic Diffusion / Total
/// Variation / Wavelet Shrinkage all evaluated by Enio and rejected on
/// visual quality grounds. CE no longer ships a denoise stage; users
/// rely on the source image being clean (or run an external denoiser
/// before importing).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorEqualizationParams {
    pub clip_limit: f32,
    pub tile_grid_size: u32,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub vibrance: f32,
    pub saturation: f32,
    // ── Phase 2 effects ───────────────────────────────────────────
    pub sharpen_amount: f32,
    pub sharpen_radius: f32,
    // ── Phase 2 automatic adjustments (toggles) ───────────────────
    pub auto_levels: bool,
    pub auto_contrast: bool,
    pub auto_colors: bool,
    pub auto_wb: bool,
    // ── Phase 3 LUT color grading ─────────────────────────────────
    pub lut_preset_1: crate::lut_presets::LutPreset,
    pub lut_preset_2: crate::lut_presets::LutPreset,
    pub lut_intensity: f32,
    pub lut_mix: f32,
    // ── Phase 5 Posterize / Quantize ──────────────────────────────
    /// Posterize levels per channel. `0` (default) is off; valid range
    /// is `2..=16` (mirror of the legacy panel's discrete options
    /// `2, 3, 4, 6, 8, 16`).
    pub posterize_levels: u32,
    /// Floyd-Steinberg dithering during posterize. Default `true` —
    /// dithered output reads cleaner on gradients (the on-by-default
    /// behaviour of the legacy panel).
    pub posterize_dithering: bool,
    /// Dither strength (Enio 2026-05-26): `0.0` = plain quantization,
    /// `1.0` = full FS dither output. Lerp between plain and dithered
    /// per-pixel results. Only effective when `posterize_dithering` is
    /// `true`. Default `1.0` preserves legacy behaviour.
    pub posterize_dither_strength: f32,
    /// Dither grain (Enio 2026-05-26): `1` = per-pixel dither, `N` =
    /// block-averaged dither on `NxN` tiles (chunky pixel-art look).
    /// Valid `1..=8`. Default `1` preserves legacy behaviour.
    pub posterize_dither_grain: u32,
    /// K-Means++ quantize colour count. `0` (default) is off; valid
    /// range is `2..=256` (legacy panel offers `4, 8, 16, 32, 64, 128,
    /// 256`).
    pub quantize_colors: u32,
}

impl Default for ColorEqualizationParams {
    fn default() -> Self {
        Self {
            clip_limit: CLIP_LIMIT_DEFAULT,
            tile_grid_size: TILE_GRID_DEFAULT,
            exposure: EXPOSURE_DEFAULT,
            temperature: TEMPERATURE_DEFAULT,
            tint: TINT_DEFAULT,
            brightness: BRIGHTNESS_DEFAULT,
            contrast: CONTRAST_DEFAULT,
            vibrance: VIBRANCE_DEFAULT,
            saturation: SATURATION_DEFAULT,
            sharpen_amount: SHARPEN_AMOUNT_DEFAULT,
            sharpen_radius: SHARPEN_RADIUS_DEFAULT,
            auto_levels: false,
            auto_contrast: false,
            auto_colors: false,
            auto_wb: false,
            lut_preset_1: crate::lut_presets::LutPreset::None,
            lut_preset_2: crate::lut_presets::LutPreset::None,
            lut_intensity: LUT_INTENSITY_DEFAULT,
            lut_mix: LUT_MIX_DEFAULT,
            posterize_levels: 0,
            posterize_dithering: true,
            posterize_dither_strength: POSTERIZE_DITHER_STRENGTH_DEFAULT,
            posterize_dither_grain: POSTERIZE_DITHER_GRAIN_DEFAULT,
            quantize_colors: 0,
        }
    }
}

impl ColorEqualizationParams {
    /// True when every adjustment is a no-op: CLAHE redistribution at
    /// `clip_limit=1` (uniform CDF), every Phase 1 tonal param at its
    /// identity value, every Phase 2 effect at `0`, all auto-* toggles
    /// off, Posterize / Quantize disabled. Callers can skip the bake
    /// + undo entry.
    pub fn is_noop(self) -> bool {
        (self.clip_limit - CLIP_LIMIT_MIN).abs() < f32::EPSILON
            && self.tonal_is_identity()
            && self.sharpen_amount == 0.0
            && !self.auto_levels
            && !self.auto_contrast
            && !self.auto_colors
            && !self.auto_wb
            && self.lut_is_identity()
            && self.posterize_levels < crate::algorithm::POSTERIZE_LEVELS_MIN
            && self.quantize_colors < crate::algorithm::QUANTIZE_COLORS_MIN
    }

    /// True when the Phase 3 LUT stage is a no-op: both preset slots
    /// are `None`, OR `lut_intensity` is `0` (forced bypass).
    pub fn lut_is_identity(self) -> bool {
        use crate::lut_presets::LutPreset;
        self.lut_intensity <= 0.0
            || (self.lut_preset_1 == LutPreset::None && self.lut_preset_2 == LutPreset::None)
    }

    /// True when the Phase 1 tonal stages produce identity output (so
    /// `adjust_tonal` can be skipped entirely). All seven knobs at their
    /// default ⇒ identity. Separate from [`Self::is_noop`] because CLAHE
    /// and auto-WB can still be active while the tonal stack is neutral.
    pub fn tonal_is_identity(self) -> bool {
        self.exposure == 0.0
            && self.temperature == 0.0
            && self.tint == 0.0
            && self.brightness == 0.0
            && (self.contrast - 1.0).abs() < f32::EPSILON
            && self.vibrance == 0.0
            && self.saturation == 0.0
    }
}

/// Projection of the tool's params for the typed
/// `ph2d-panel-color-equalization` to paint. All slider-bound fields land
/// in normalized `0.0..=1.0`; the panel paints these directly as slider
/// track positions. The host publishes a fresh snapshot each frame via
/// `ph2d_panel_color_equalization::set_current_snapshot`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorEqualizationUiSnapshot {
    // Normalized slider positions (0..1).
    pub clip_limit01: f32,
    pub tile_grid01: f32,
    pub exposure01: f32,
    pub temperature01: f32,
    pub tint01: f32,
    pub brightness01: f32,
    pub contrast01: f32,
    pub vibrance01: f32,
    pub saturation01: f32,
    pub sharpen_amount01: f32,
    pub sharpen_radius01: f32,
    pub lut_intensity01: f32,
    pub lut_mix01: f32,
    pub auto_levels: bool,
    pub auto_contrast: bool,
    pub auto_colors: bool,
    pub auto_wb: bool,
    pub lut_preset_1: crate::lut_presets::LutPreset,
    pub lut_preset_2: crate::lut_presets::LutPreset,
    /// Posterize levels per channel — `0` = off, else `2..=16`. Panel
    /// paints as a dropdown of discrete options.
    pub posterize_levels: u32,
    pub posterize_dithering: bool,
    /// Dither knobs (Enio 2026-05-26). Strength/Grain como slider 01 +
    /// raw pra chip painters.
    pub posterize_dither_strength01: f32,
    pub posterize_dither_strength: f32,
    pub posterize_dither_grain01: f32,
    pub posterize_dither_grain: u32,
    /// Quantize colour count — `0` = off, else `2..=256`. Panel paints
    /// as a dropdown of discrete options.
    pub quantize_colors: u32,
    // Raw values mirrored so chips can paint the natural unit
    // (EV stops, ±1 brightness, etc.) without recomputing inverse
    // projections in the panel.
    pub clip_limit: f32,
    pub tile_grid_size: u32,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub vibrance: f32,
    pub saturation: f32,
    pub sharpen_amount: f32,
    pub sharpen_radius: f32,
    pub lut_intensity: f32,
    pub lut_mix: f32,
}

impl Default for ColorEqualizationUiSnapshot {
    fn default() -> Self {
        let p = ColorEqualizationParams::default();
        Self {
            clip_limit01: clip_limit_to_slider(p.clip_limit),
            tile_grid01: tile_grid_to_slider(p.tile_grid_size),
            exposure01: exposure_to_slider(p.exposure),
            temperature01: temperature_to_slider(p.temperature),
            tint01: tint_to_slider(p.tint),
            brightness01: brightness_to_slider(p.brightness),
            contrast01: contrast_to_slider(p.contrast),
            vibrance01: vibrance_to_slider(p.vibrance),
            saturation01: saturation_to_slider(p.saturation),
            sharpen_amount01: sharpen_amount_to_slider(p.sharpen_amount),
            sharpen_radius01: sharpen_radius_to_slider(p.sharpen_radius),
            lut_intensity01: lut_intensity_to_slider(p.lut_intensity),
            lut_mix01: lut_mix_to_slider(p.lut_mix),
            auto_levels: p.auto_levels,
            auto_contrast: p.auto_contrast,
            auto_colors: p.auto_colors,
            auto_wb: p.auto_wb,
            lut_preset_1: p.lut_preset_1,
            lut_preset_2: p.lut_preset_2,
            posterize_levels: p.posterize_levels,
            posterize_dithering: p.posterize_dithering,
            posterize_dither_strength01: posterize_dither_strength_to_slider(
                p.posterize_dither_strength,
            ),
            posterize_dither_strength: p.posterize_dither_strength,
            posterize_dither_grain01: posterize_dither_grain_to_slider(p.posterize_dither_grain),
            posterize_dither_grain: p.posterize_dither_grain,
            quantize_colors: p.quantize_colors,
            clip_limit: p.clip_limit,
            tile_grid_size: p.tile_grid_size,
            exposure: p.exposure,
            temperature: p.temperature,
            tint: p.tint,
            brightness: p.brightness,
            contrast: p.contrast,
            vibrance: p.vibrance,
            saturation: p.saturation,
            sharpen_amount: p.sharpen_amount,
            sharpen_radius: p.sharpen_radius,
            lut_intensity: p.lut_intensity,
            lut_mix: p.lut_mix,
        }
    }
}

#[path = "params_edit.rs"]
mod edit;
pub use edit::{ColorEqualizationUiEdit, apply_ui_edit};
#[cfg(test)]
#[path = "params_tests.rs"]
mod tests;
