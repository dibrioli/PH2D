//! Testes de `params.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

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
