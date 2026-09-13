//! **Uma edição vinda do painel, e onde ela aterra** — o [`ColorEqualizationUiEdit`] e o
//! [`apply_ui_edit`] que o aplica com os clamps num sítio só —, irmão de `params.rs` por tecto de
//! LOC. O caminho público não muda: `params.rs` re-exporta os dois.
//!
//! Corte mecânico: os dois itens saíram inteiros, verbatim.

use super::*;

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
