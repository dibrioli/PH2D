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

/// One panel-originated edit. The shell pushes the generic
/// `EditorAction::ToolPanelEvent(PanelEvent::…)`; the tool's
/// `handle_panel_event` maps the NodeId back to one of these variants
/// and forwards through [`apply_ui_edit`].
///
/// Slider variants carry normalized `0.0..=1.0`; number-chip variants
/// carry the natural unit. Both paths converge in `apply_ui_edit` so
/// clamps live exactly once.
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1) — additive variants no
/// longer semver-break downstream `match`.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ColorEqualizationUiEdit {
    /// Clip limit slider moved (normalized).
    ClipLimitSlider(f32),
    /// Clip limit chip edited (natural unit).
    ClipLimit(f32),
    /// Tile grid slider moved (normalized).
    TileGridSlider(f32),
    /// Tile grid chip edited (natural unit, rounded to integer).
    TileGrid(u32),
    /// Exposure slider moved (normalized).
    ExposureSlider(f32),
    /// Exposure chip edited (EV stops, `-3..+3`).
    Exposure(f32),
    /// Temperature slider moved (normalized).
    TemperatureSlider(f32),
    /// Temperature chip edited (natural unit, `-1..+1`; +1 warm).
    Temperature(f32),
    /// Tint slider moved (normalized).
    TintSlider(f32),
    /// Tint chip edited (natural unit, `-1..+1`; +1 magenta).
    Tint(f32),
    /// Brightness slider moved (normalized).
    BrightnessSlider(f32),
    /// Brightness chip edited (natural unit, `-1..+1`).
    Brightness(f32),
    /// Contrast slider moved (normalized).
    ContrastSlider(f32),
    /// Contrast chip edited (natural unit, `0.5..2.0`).
    Contrast(f32),
    /// Vibrance slider moved (normalized).
    VibranceSlider(f32),
    /// Vibrance chip edited (natural unit, `-1..+1`).
    Vibrance(f32),
    /// Saturation slider moved (normalized).
    SaturationSlider(f32),
    /// Saturation chip edited (natural unit, `-1..+1`).
    Saturation(f32),
    /// Sharpen amount slider moved (normalized).
    SharpenAmountSlider(f32),
    /// Sharpen amount chip edited (natural unit, `0..2`).
    SharpenAmount(f32),
    /// Sharpen radius slider moved (normalized).
    SharpenRadiusSlider(f32),
    /// Sharpen radius chip edited (natural unit, `0.5..3`).
    SharpenRadius(f32),
    /// Auto Levels toggle flipped.
    ToggleAutoLevels,
    /// Auto Contrast toggle flipped.
    ToggleAutoContrast,
    /// Auto Colors toggle flipped.
    ToggleAutoColors,
    /// Auto-WB toggle flipped.
    ToggleAutoWb,
    /// Cycle LUT slot 1 to the next procedural preset (wraps).
    /// Kept alongside [`Self::SetLutPreset1`] for keyboard/cycle UX.
    CycleLutPreset1Next,
    /// Cycle LUT slot 1 to the previous preset (wraps).
    CycleLutPreset1Prev,
    /// Cycle LUT slot 2 to the next preset (wraps).
    CycleLutPreset2Next,
    /// Cycle LUT slot 2 to the previous preset (wraps).
    CycleLutPreset2Prev,
    /// LUT slot 1 set directly to `preset` (panel dropdown selection).
    SetLutPreset1(crate::lut_presets::LutPreset),
    /// LUT slot 2 set directly to `preset` (panel dropdown selection).
    SetLutPreset2(crate::lut_presets::LutPreset),
    /// Posterize levels set directly. `0` = off; valid `2..=16`.
    SetPosterizeLevels(u32),
    /// Toggle Floyd-Steinberg dithering in posterize.
    ToggleDithering,
    /// Dither strength slider moved (normalized `0..1`).
    DitherStrengthSlider(f32),
    /// Dither strength chip edited (natural `0..1`).
    DitherStrength(f32),
    /// Dither grain slider moved (normalized `0..1` → `1..=8`).
    DitherGrainSlider(f32),
    /// Dither grain chip edited (natural integer `1..=8`).
    DitherGrain(u32),
    /// K-Means++ quantize colour count. `0` = off; valid `2..=256`.
    SetQuantizeColors(u32),
    /// LUT intensity slider moved (normalized `0..1`).
    LutIntensitySlider(f32),
    /// LUT intensity chip edited (natural unit, `0..1`).
    LutIntensity(f32),
    /// LUT mix slider moved (normalized `0..1`; `0` = slot 1, `1` = slot 2).
    LutMixSlider(f32),
    /// LUT mix chip edited (natural unit, `0..1`).
    LutMix(f32),
    /// Apply pressed — bake at full resolution on every selected sprite.
    Apply,
    /// Reset every param to its default in one click. Fired by the
    /// panel's Reset button AND by the tool's `on_activate` so a
    /// reopened panel never inherits the previous session's slider
    /// positions.
    ResetAll,
}

/// Apply one UI edit against the live params with clamps centralized.
/// Returns `true` when a param actually changed (the tool uses this to
/// gate the preview re-run).
pub fn apply_ui_edit(params: &mut ColorEqualizationParams, edit: ColorEqualizationUiEdit) -> bool {
    let before = *params;
    match edit {
        ColorEqualizationUiEdit::ClipLimitSlider(v) => {
            params.clip_limit = slider_to_clip_limit(v);
        }
        ColorEqualizationUiEdit::ClipLimit(v) => {
            params.clip_limit = v.clamp(CLIP_LIMIT_MIN, CLIP_LIMIT_MAX);
        }
        ColorEqualizationUiEdit::TileGridSlider(v) => {
            params.tile_grid_size = slider_to_tile_grid(v);
        }
        ColorEqualizationUiEdit::TileGrid(n) => {
            params.tile_grid_size = n.clamp(TILE_GRID_MIN, TILE_GRID_MAX);
        }
        ColorEqualizationUiEdit::ExposureSlider(v) => {
            params.exposure = slider_to_exposure(v);
        }
        ColorEqualizationUiEdit::Exposure(v) => {
            params.exposure = v.clamp(EXPOSURE_MIN, EXPOSURE_MAX);
        }
        ColorEqualizationUiEdit::TemperatureSlider(v) => {
            params.temperature = slider_to_temperature(v);
        }
        ColorEqualizationUiEdit::Temperature(v) => {
            params.temperature = v.clamp(TEMPERATURE_MIN, TEMPERATURE_MAX);
        }
        ColorEqualizationUiEdit::TintSlider(v) => {
            params.tint = slider_to_tint(v);
        }
        ColorEqualizationUiEdit::Tint(v) => {
            params.tint = v.clamp(TINT_MIN, TINT_MAX);
        }
        ColorEqualizationUiEdit::BrightnessSlider(v) => {
            params.brightness = slider_to_brightness(v);
        }
        ColorEqualizationUiEdit::Brightness(v) => {
            params.brightness = v.clamp(BRIGHTNESS_MIN, BRIGHTNESS_MAX);
        }
        ColorEqualizationUiEdit::ContrastSlider(v) => {
            params.contrast = slider_to_contrast(v);
        }
        ColorEqualizationUiEdit::Contrast(v) => {
            params.contrast = v.clamp(CONTRAST_MIN, CONTRAST_MAX);
        }
        ColorEqualizationUiEdit::VibranceSlider(v) => {
            params.vibrance = slider_to_vibrance(v);
        }
        ColorEqualizationUiEdit::Vibrance(v) => {
            params.vibrance = v.clamp(VIBRANCE_MIN, VIBRANCE_MAX);
        }
        ColorEqualizationUiEdit::SaturationSlider(v) => {
            params.saturation = slider_to_saturation(v);
        }
        ColorEqualizationUiEdit::Saturation(v) => {
            params.saturation = v.clamp(SATURATION_MIN, SATURATION_MAX);
        }
        ColorEqualizationUiEdit::SharpenAmountSlider(v) => {
            params.sharpen_amount = slider_to_sharpen_amount(v);
        }
        ColorEqualizationUiEdit::SharpenAmount(v) => {
            params.sharpen_amount = v.clamp(SHARPEN_AMOUNT_MIN, SHARPEN_AMOUNT_MAX);
        }
        ColorEqualizationUiEdit::SharpenRadiusSlider(v) => {
            params.sharpen_radius = slider_to_sharpen_radius(v);
        }
        ColorEqualizationUiEdit::SharpenRadius(v) => {
            params.sharpen_radius = v.clamp(SHARPEN_RADIUS_MIN, SHARPEN_RADIUS_MAX);
        }
        ColorEqualizationUiEdit::ToggleAutoLevels => {
            params.auto_levels = !params.auto_levels;
        }
        ColorEqualizationUiEdit::ToggleAutoContrast => {
            params.auto_contrast = !params.auto_contrast;
        }
        ColorEqualizationUiEdit::ToggleAutoColors => {
            params.auto_colors = !params.auto_colors;
        }
        ColorEqualizationUiEdit::ToggleAutoWb => {
            params.auto_wb = !params.auto_wb;
        }
        ColorEqualizationUiEdit::CycleLutPreset1Next => {
            params.lut_preset_1 = params.lut_preset_1.next();
        }
        ColorEqualizationUiEdit::CycleLutPreset1Prev => {
            params.lut_preset_1 = params.lut_preset_1.prev();
        }
        ColorEqualizationUiEdit::CycleLutPreset2Next => {
            params.lut_preset_2 = params.lut_preset_2.next();
        }
        ColorEqualizationUiEdit::CycleLutPreset2Prev => {
            params.lut_preset_2 = params.lut_preset_2.prev();
        }
        ColorEqualizationUiEdit::SetLutPreset1(preset) => {
            params.lut_preset_1 = preset;
        }
        ColorEqualizationUiEdit::SetLutPreset2(preset) => {
            params.lut_preset_2 = preset;
        }
        ColorEqualizationUiEdit::SetPosterizeLevels(n) => {
            // `0` (off) passes through; valid range else is clamped to
            // the legacy panel's `2..=16` ceiling.
            params.posterize_levels = if n < crate::algorithm::POSTERIZE_LEVELS_MIN {
                0
            } else {
                n.min(crate::algorithm::POSTERIZE_LEVELS_MAX)
            };
        }
        ColorEqualizationUiEdit::ToggleDithering => {
            params.posterize_dithering = !params.posterize_dithering;
        }
        ColorEqualizationUiEdit::DitherStrengthSlider(v) => {
            params.posterize_dither_strength = slider_to_posterize_dither_strength(v);
        }
        ColorEqualizationUiEdit::DitherStrength(v) => {
            params.posterize_dither_strength =
                v.clamp(POSTERIZE_DITHER_STRENGTH_MIN, POSTERIZE_DITHER_STRENGTH_MAX);
        }
        ColorEqualizationUiEdit::DitherGrainSlider(v) => {
            params.posterize_dither_grain = slider_to_posterize_dither_grain(v);
        }
        ColorEqualizationUiEdit::DitherGrain(v) => {
            params.posterize_dither_grain =
                v.clamp(POSTERIZE_DITHER_GRAIN_MIN, POSTERIZE_DITHER_GRAIN_MAX);
        }
        ColorEqualizationUiEdit::SetQuantizeColors(n) => {
            params.quantize_colors = if n < crate::algorithm::QUANTIZE_COLORS_MIN {
                0
            } else {
                n.min(crate::algorithm::QUANTIZE_COLORS_MAX)
            };
        }
        ColorEqualizationUiEdit::LutIntensitySlider(v) => {
            params.lut_intensity = slider_to_lut_intensity(v);
        }
        ColorEqualizationUiEdit::LutIntensity(v) => {
            params.lut_intensity = v.clamp(LUT_INTENSITY_MIN, LUT_INTENSITY_MAX);
        }
        ColorEqualizationUiEdit::LutMixSlider(v) => {
            params.lut_mix = slider_to_lut_mix(v);
        }
        ColorEqualizationUiEdit::LutMix(v) => {
            params.lut_mix = v.clamp(LUT_MIX_MIN, LUT_MIX_MAX);
        }
        // Apply does not mutate params; the tool latches a separate
        // pending-apply flag.
        ColorEqualizationUiEdit::Apply => return false,
        // Reset every adjustment back to the default-constructed state
        // (CLAHE off, all tonal at identity, no LUT, no posterize /
        // quantize, every auto-* off). Returns `true` so the preview
        // rebuilds against the now-clean params on the next idle frame.
        ColorEqualizationUiEdit::ResetAll => {
            *params = ColorEqualizationParams::default();
        }
    }
    *params != before
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_identity() {
        let p = ColorEqualizationParams::default();
        assert_eq!(p.clip_limit, CLIP_LIMIT_DEFAULT);
        assert_eq!(p.tile_grid_size, TILE_GRID_DEFAULT);
        assert_eq!(p.brightness, BRIGHTNESS_DEFAULT);
        assert_eq!(p.contrast, CONTRAST_DEFAULT);
        assert_eq!(p.saturation, SATURATION_DEFAULT);
        assert!(!p.auto_wb);
    }

    #[test]
    fn slider_projections_round_trip() {
        let cases = [
            (CLIP_LIMIT_MIN, 0.0_f32),
            (
                CLIP_LIMIT_DEFAULT,
                (CLIP_LIMIT_DEFAULT - CLIP_LIMIT_MIN) / (CLIP_LIMIT_MAX - CLIP_LIMIT_MIN),
            ),
            (CLIP_LIMIT_MAX, 1.0_f32),
        ];
        for (v, expected) in cases {
            let t = clip_limit_to_slider(v);
            assert!(
                (t - expected).abs() < 1e-6,
                "clip {v} → {t} (want {expected})"
            );
            let back = slider_to_clip_limit(t);
            assert!((back - v).abs() < 1e-5, "round trip {v} → {t} → {back}");
        }
    }

    #[test]
    fn tile_grid_clamps_and_rounds() {
        let mut p = ColorEqualizationParams::default();
        assert!(apply_ui_edit(&mut p, ColorEqualizationUiEdit::TileGrid(2)));
        assert_eq!(p.tile_grid_size, TILE_GRID_MIN);
        assert!(apply_ui_edit(&mut p, ColorEqualizationUiEdit::TileGrid(99)));
        assert_eq!(p.tile_grid_size, TILE_GRID_MAX);
    }

    #[test]
    fn apply_ui_edit_returns_false_on_no_change() {
        let mut p = ColorEqualizationParams::default();
        assert!(!apply_ui_edit(
            &mut p,
            ColorEqualizationUiEdit::Brightness(BRIGHTNESS_DEFAULT)
        ));
    }

    #[test]
    fn apply_ui_edit_returns_false_on_apply() {
        let mut p = ColorEqualizationParams::default();
        assert!(!apply_ui_edit(&mut p, ColorEqualizationUiEdit::Apply));
        assert_eq!(p, ColorEqualizationParams::default());
    }

    #[test]
    fn auto_wb_toggles() {
        let mut p = ColorEqualizationParams::default();
        assert!(!p.auto_wb);
        assert!(apply_ui_edit(&mut p, ColorEqualizationUiEdit::ToggleAutoWb));
        assert!(p.auto_wb);
        assert!(apply_ui_edit(&mut p, ColorEqualizationUiEdit::ToggleAutoWb));
        assert!(!p.auto_wb);
    }

    #[test]
    fn slider_path_clamps_at_extremes() {
        let mut p = ColorEqualizationParams::default();
        assert!(apply_ui_edit(
            &mut p,
            ColorEqualizationUiEdit::BrightnessSlider(2.0)
        ));
        assert_eq!(p.brightness, BRIGHTNESS_MAX);
        assert!(apply_ui_edit(
            &mut p,
            ColorEqualizationUiEdit::BrightnessSlider(-2.0)
        ));
        assert_eq!(p.brightness, BRIGHTNESS_MIN);
    }

    #[test]
    fn uniform_clip_limit_is_pipeline_identity() {
        // `is_noop` reports identity-output params: CLAHE at clip 1.0 (no
        // contrast boost) + B/C/S identity + auto-WB off. The PANEL
        // default sits at clip 2.0 (canonical Zuiderveld), which DOES
        // change pixels — that is by design, not a bug.
        let p = ColorEqualizationParams {
            clip_limit: CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        assert!(p.is_noop());
    }

    #[test]
    fn changed_brightness_is_not_noop() {
        let p = ColorEqualizationParams {
            clip_limit: CLIP_LIMIT_MIN,
            brightness: 0.2,
            ..ColorEqualizationParams::default()
        };
        assert!(!p.is_noop());
    }
}
