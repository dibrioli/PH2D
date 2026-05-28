//! Asserts the Color Equalization panel's chip↔slider pairs are wired
//! with the affine mapping that matches the tool's `*_to_slider`
//! projection. Without these gates, drift between the panel's chip
//! seed and the tool's projection re-introduces the 2026-05-27 BGRemoval
//! Grow bug class (type X, commit Y) silently.
//!
//! The mapping under test: `display = storage * (max - min) + min`,
//! mirroring `unproject01` in `ph2d-tool-color-equalization::params`.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::Panel;
use ph2d_panel_color_equalization::ColorEqualizationPanel;
use ph2d_tool_color_equalization::params::{
    BRIGHTNESS_DEFAULT, BRIGHTNESS_MAX, BRIGHTNESS_MIN, CLIP_LIMIT_DEFAULT, CLIP_LIMIT_MAX,
    CLIP_LIMIT_MIN, CONTRAST_DEFAULT, CONTRAST_MAX, CONTRAST_MIN, EXPOSURE_DEFAULT, EXPOSURE_MAX,
    EXPOSURE_MIN, POSTERIZE_DITHER_GRAIN_DEFAULT, POSTERIZE_DITHER_GRAIN_MAX,
    POSTERIZE_DITHER_GRAIN_MIN, SHARPEN_RADIUS_DEFAULT, SHARPEN_RADIUS_MAX, SHARPEN_RADIUS_MIN,
    TILE_GRID_DEFAULT, TILE_GRID_MAX, TILE_GRID_MIN,
};

use ph2d_panel_color_equalization::ids;

fn populated_store() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(64);
    ColorEqualizationPanel::populate(&mut store);
    store
}

/// Helper: assert the chip↔slider pair (id_slider, id_chip) is wired
/// with mapping `(max - min, min)` and that the chip's stored value
/// matches the natural default (NOT the 0..1 storage — that's the bug
/// this whole fix is about).
fn assert_pair(
    store: &WidgetStore,
    label: &str,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    default: f32,
    min: f32,
    max: f32,
) {
    let scale = max - min;
    let offset = min;
    let (got_scale, got_offset) = store.linked_slider_mapping(chip);
    let identity = (got_scale - 1.0).abs() < f32::EPSILON && got_offset.abs() < f32::EPSILON;
    let registered = (got_scale - scale).abs() < 1e-5 && (got_offset - offset).abs() < 1e-5;
    if scale == 1.0 && offset == 0.0 {
        // Identity-mapped rows (LUT Intensity, LUT Mix, Dither Strength)
        // — mapping table is intentionally empty for these, so
        // linked_slider_mapping returns identity.
        assert!(
            identity,
            "{label}: identity mapping expected (scale 1, offset 0), got ({got_scale}, {got_offset})"
        );
    } else {
        assert!(
            registered,
            "{label}: mapping mismatch — expected ({scale}, {offset}), got ({got_scale}, {got_offset})"
        );
    }
    // Chip's stored value must be the NATURAL default (display-space),
    // not `default_to_slider`. The buffer the user sees on focus is
    // formatted from this value.
    let chip_v = store.number_value(chip).expect("chip");
    assert!(
        (chip_v - default as f64).abs() < 1e-5,
        "{label}: chip seed must be natural default {default}, got {chip_v}"
    );
    // Slider track must be `project01(default, min, max)` so the track
    // position visually matches the chip readout.
    let (_, slider_v) = store.slider(slider).expect("slider");
    let expected_track = if (max - min).abs() < f32::EPSILON {
        0.0
    } else {
        ((default - min) / (max - min)).clamp(0.0, 1.0)
    };
    assert!(
        (slider_v - expected_track).abs() < 1e-5,
        "{label}: slider track {slider_v} != project01(default) {expected_track}"
    );
}

#[test]
fn clip_limit_pair_uses_correct_mapping() {
    let s = populated_store();
    assert_pair(
        &s,
        "CLIP_LIMIT",
        ids::CEQ_CLIP_LIMIT,
        ids::CEQ_CLIP_LIMIT_NUM,
        CLIP_LIMIT_DEFAULT,
        CLIP_LIMIT_MIN,
        CLIP_LIMIT_MAX,
    );
}

#[test]
fn tile_grid_pair_uses_correct_mapping() {
    let s = populated_store();
    assert_pair(
        &s,
        "TILE_GRID",
        ids::CEQ_TILE_GRID,
        ids::CEQ_TILE_GRID_NUM,
        TILE_GRID_DEFAULT as f32,
        TILE_GRID_MIN as f32,
        TILE_GRID_MAX as f32,
    );
}

#[test]
fn exposure_pair_uses_bipolar_mapping() {
    let s = populated_store();
    assert_pair(
        &s,
        "EXPOSURE",
        ids::CEQ_EXPOSURE,
        ids::CEQ_EXPOSURE_NUM,
        EXPOSURE_DEFAULT,
        EXPOSURE_MIN,
        EXPOSURE_MAX,
    );
}

#[test]
fn brightness_pair_uses_bipolar_mapping() {
    let s = populated_store();
    assert_pair(
        &s,
        "BRIGHTNESS",
        ids::CEQ_BRIGHTNESS,
        ids::CEQ_BRIGHTNESS_NUM,
        BRIGHTNESS_DEFAULT,
        BRIGHTNESS_MIN,
        BRIGHTNESS_MAX,
    );
}

#[test]
fn contrast_pair_uses_offset_mapping() {
    // Contrast min=0.5, max=2.0, default 1.0 → not centred on 0.
    let s = populated_store();
    assert_pair(
        &s,
        "CONTRAST",
        ids::CEQ_CONTRAST,
        ids::CEQ_CONTRAST_NUM,
        CONTRAST_DEFAULT,
        CONTRAST_MIN,
        CONTRAST_MAX,
    );
}

#[test]
fn sharpen_radius_pair_uses_offset_mapping() {
    // Sharpen radius min=0.5, max=3.0 — non-symmetric, non-zero offset.
    let s = populated_store();
    assert_pair(
        &s,
        "SHARPEN_RADIUS",
        ids::CEQ_SHARPEN_RADIUS,
        ids::CEQ_SHARPEN_RADIUS_NUM,
        SHARPEN_RADIUS_DEFAULT,
        SHARPEN_RADIUS_MIN,
        SHARPEN_RADIUS_MAX,
    );
}

#[test]
fn posterize_dither_grain_pair_uses_integer_mapping() {
    let s = populated_store();
    assert_pair(
        &s,
        "POSTERIZE_DITHER_GRAIN",
        ids::CEQ_POSTERIZE_DITHER_GRAIN,
        ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
        POSTERIZE_DITHER_GRAIN_DEFAULT as f32,
        POSTERIZE_DITHER_GRAIN_MIN as f32,
        POSTERIZE_DITHER_GRAIN_MAX as f32,
    );
}

#[test]
fn chip_seed_in_natural_space_not_storage_space() {
    // Direct regression for the bug pattern: chip's stored value MUST
    // match the natural default. The pre-fix populate seeded chip with
    // `track` (0..1), so e.g. brightness chip read 0.5 instead of 0.0,
    // and the focused buffer showed "0.500" while the unfocused
    // display_override showed "+0.00" — typing parsed text was then
    // written to the slider 1:1 as if it were 0..1 storage.
    let s = populated_store();
    let brightness_chip = s.number_value(ids::CEQ_BRIGHTNESS_NUM).expect("chip");
    assert!(
        (brightness_chip - BRIGHTNESS_DEFAULT as f64).abs() < 1e-5,
        "brightness chip must seed at natural default ({BRIGHTNESS_DEFAULT}), got {brightness_chip}"
    );
    let exposure_chip = s.number_value(ids::CEQ_EXPOSURE_NUM).expect("chip");
    assert!(
        (exposure_chip - EXPOSURE_DEFAULT as f64).abs() < 1e-5,
        "exposure chip must seed at natural default ({EXPOSURE_DEFAULT}), got {exposure_chip}"
    );
    let clip_chip = s.number_value(ids::CEQ_CLIP_LIMIT_NUM).expect("chip");
    assert!(
        (clip_chip - CLIP_LIMIT_DEFAULT as f64).abs() < 1e-5,
        "clip_limit chip must seed at natural default ({CLIP_LIMIT_DEFAULT}), got {clip_chip}"
    );
}
