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
/// `1.0..=CLIP_LIMIT_MAX` (the canonical CLAHE range; `2.0` is the
/// default and matches the OpenCV reference implementation). Anything
/// below `1.0` collapses to a uniform redistribution (no contrast
/// boost); anything above ≈ 4.0 starts to amplify noise harshly.
pub const CLIP_LIMIT_MIN: f32 = 1.0;
pub const CLIP_LIMIT_MAX: f32 = 4.0;
pub const CLIP_LIMIT_DEFAULT: f32 = 2.0;

/// Tile grid size — image is partitioned into `N×N` square tiles.
/// `N = 8` is the canonical default (Zuiderveld); below `4` the
/// per-tile histogram becomes too sparse, above `16` interpolation
/// artefacts dominate. Slider integer in `[4, 16]`.
pub const TILE_GRID_MIN: u32 = 4;
pub const TILE_GRID_MAX: u32 = 16;
pub const TILE_GRID_DEFAULT: u32 = 8;

/// Brightness / Contrast / Saturation slider ranges.
///
/// All three are applied in **linear-light sRGB** after CLAHE. Brightness
/// is an additive offset in `[−1, +1]` (linear units; clamped to `[0, 1]`
/// before delinearization). Contrast is a multiplicative scale around
/// `0.5` in `[0.5, 2.0]` (`1.0` = identity). Saturation is a mix between
/// linear-luma grayscale and the original, where `0` = identity, `−1` =
/// fully desaturated, `+1` = twice the chroma (mix factor `1 + v`).
pub const BRIGHTNESS_MIN: f32 = -1.0;
pub const BRIGHTNESS_MAX: f32 = 1.0;
pub const BRIGHTNESS_DEFAULT: f32 = 0.0;

pub const CONTRAST_MIN: f32 = 0.5;
pub const CONTRAST_MAX: f32 = 2.0;
pub const CONTRAST_DEFAULT: f32 = 1.0;

pub const SATURATION_MIN: f32 = -1.0;
pub const SATURATION_MAX: f32 = 1.0;
pub const SATURATION_DEFAULT: f32 = 0.0;

// ── Slider ↔ value projection helpers ─────────────────────────────
//
// The panel paints normalized track `0..1`; the tool stores the natural
// unit. Single mapping site keeps `ui_snapshot` (forward) and
// `apply_ui_edit` (inverse) in lock-step.

fn project01(v: f32, min: f32, max: f32) -> f32 {
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

pub fn saturation_to_slider(v: f32) -> f32 {
    project01(v, SATURATION_MIN, SATURATION_MAX)
}

pub fn slider_to_saturation(track: f32) -> f32 {
    unproject01(track, SATURATION_MIN, SATURATION_MAX)
}

/// Authoritative parameter bag fed into [`crate::algorithm::run_pipeline`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorEqualizationParams {
    pub clip_limit: f32,
    pub tile_grid_size: u32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub auto_wb: bool,
}

impl Default for ColorEqualizationParams {
    fn default() -> Self {
        Self {
            clip_limit: CLIP_LIMIT_DEFAULT,
            tile_grid_size: TILE_GRID_DEFAULT,
            brightness: BRIGHTNESS_DEFAULT,
            contrast: CONTRAST_DEFAULT,
            saturation: SATURATION_DEFAULT,
            auto_wb: false,
        }
    }
}

impl ColorEqualizationParams {
    /// True when every adjustment is a no-op (CLAHE redistribution at
    /// `clip_limit=1` is a uniform CDF that produces the source back; B/C/S
    /// at their identity defaults touch nothing; auto-WB off). Callers can
    /// skip the bake + undo entry.
    pub fn is_noop(self) -> bool {
        (self.clip_limit - CLIP_LIMIT_MIN).abs() < f32::EPSILON
            && self.brightness == 0.0
            && self.contrast == 1.0
            && self.saturation == 0.0
            && !self.auto_wb
    }
}

/// Projection of the tool's params for the typed
/// `ph2d-panel-color-equalization` to paint. All slider-bound fields land
/// in normalized `0.0..=1.0`; the panel paints these directly as slider
/// track positions. The host publishes a fresh snapshot each frame via
/// `ph2d_panel_color_equalization::set_current_snapshot`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorEqualizationUiSnapshot {
    pub clip_limit01: f32,
    pub tile_grid01: f32,
    pub brightness01: f32,
    pub contrast01: f32,
    pub saturation01: f32,
    pub auto_wb: bool,
    /// Raw values, mirrored so number chips can paint the natural unit
    /// (clip limit `2.0`, tile grid `8`, etc.) without recomputing the
    /// inverse projection inside the panel.
    pub clip_limit: f32,
    pub tile_grid_size: u32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

impl Default for ColorEqualizationUiSnapshot {
    fn default() -> Self {
        let p = ColorEqualizationParams::default();
        Self {
            clip_limit01: clip_limit_to_slider(p.clip_limit),
            tile_grid01: tile_grid_to_slider(p.tile_grid_size),
            brightness01: brightness_to_slider(p.brightness),
            contrast01: contrast_to_slider(p.contrast),
            saturation01: saturation_to_slider(p.saturation),
            auto_wb: p.auto_wb,
            clip_limit: p.clip_limit,
            tile_grid_size: p.tile_grid_size,
            brightness: p.brightness,
            contrast: p.contrast,
            saturation: p.saturation,
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColorEqualizationUiEdit {
    /// Clip limit slider moved (normalized).
    ClipLimitSlider(f32),
    /// Clip limit chip edited (natural unit).
    ClipLimit(f32),
    /// Tile grid slider moved (normalized).
    TileGridSlider(f32),
    /// Tile grid chip edited (natural unit, rounded to integer).
    TileGrid(u32),
    /// Brightness slider moved (normalized).
    BrightnessSlider(f32),
    /// Brightness chip edited (natural unit, `-1..+1`).
    Brightness(f32),
    /// Contrast slider moved (normalized).
    ContrastSlider(f32),
    /// Contrast chip edited (natural unit, `0.5..2.0`).
    Contrast(f32),
    /// Saturation slider moved (normalized).
    SaturationSlider(f32),
    /// Saturation chip edited (natural unit, `-1..+1`).
    Saturation(f32),
    /// Auto-WB toggle flipped.
    ToggleAutoWb,
    /// Apply pressed — bake at full resolution on every selected sprite.
    Apply,
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
        ColorEqualizationUiEdit::SaturationSlider(v) => {
            params.saturation = slider_to_saturation(v);
        }
        ColorEqualizationUiEdit::Saturation(v) => {
            params.saturation = v.clamp(SATURATION_MIN, SATURATION_MAX);
        }
        ColorEqualizationUiEdit::ToggleAutoWb => {
            params.auto_wb = !params.auto_wb;
        }
        // Apply does not mutate params; the tool latches a separate
        // pending-apply flag.
        ColorEqualizationUiEdit::Apply => return false,
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
