//! Smoke + state-round-trip tests for the picker.

use super::channels::{max_in_gamut_chroma, oklch_norm_channels, oklch_set_channel, rgba_to_hsv};
use super::paint::paint_blender_color_picker;
use super::state::{
    BlenderColorPicker, ChannelMode, ColorPalette, InterpolationMode, default_palette,
};
use crate::zones::Rect;
use ph2d_a11y::{NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorValue, Theme};
use ph2d_vector::VectorScene;

#[test]
fn defaults_match_spec() {
    let cp = BlenderColorPicker::new(NodeId(1), "Color");
    assert_eq!(cp.interpolation, InterpolationMode::Perceptual);
    assert_eq!(cp.channel_mode, ChannelMode::Rgb);
    assert_eq!(cp.palettes.len(), 1);
    assert_eq!(cp.palettes[0].name, "Default");
    assert!(cp.palettes[0].editable);
    assert_eq!(cp.value.rgba, [231, 231, 231, 255]);
    assert!(cp.hex.starts_with('#'));
}

#[test]
fn set_value_resyncs_hex() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    cp.set_value(ColorValue::from_rgba8(255, 0, 0, 255));
    assert_eq!(cp.hex, "#FF0000FF");
}

#[test]
fn channel_mode_swap_does_not_change_value() {
    let cp = BlenderColorPicker::new(NodeId(1), "x")
        .channel_mode(ChannelMode::Hsv)
        .value(ColorValue::from_rgba8(120, 200, 80, 255));
    assert_eq!(cp.value.rgba, [120, 200, 80, 255]);
}

#[test]
fn interpolation_setter_round_trips() {
    let cp = BlenderColorPicker::new(NodeId(1), "x").interpolation(InterpolationMode::Linear);
    assert_eq!(cp.interpolation, InterpolationMode::Linear);
}

#[test]
fn rgba_to_hsv_red() {
    let (h, s, v, _) = rgba_to_hsv([255, 0, 0, 255]);
    assert!(h.abs() < 1e-3 || (h - 1.0).abs() < 1e-3);
    assert!((s - 1.0).abs() < 1e-3);
    assert!((v - 1.0).abs() < 1e-3);
}

#[test]
fn default_palette_has_12_swatches() {
    let p = default_palette();
    assert_eq!(p.swatches.len(), 12);
    // Default palette is now editable so the user can append to it
    // via the "+ swatch" button and remove via right-click.
    assert!(p.editable);
}

#[test]
fn palette_constructor_defaults_editable() {
    let p = ColorPalette::new("Custom", vec![ColorValue::WHITE]);
    assert!(p.editable);
    assert_eq!(p.name, "Custom");
}

#[test]
fn a11y_role_is_group() {
    let cp = BlenderColorPicker::new(NodeId(1), "Color");
    let node = cp.build_a11y(0.0, 0.0, 280.0, 540.0);
    assert_eq!(node.role(), Role::Group);
}

fn smoke(cp: BlenderColorPicker, theme: Theme) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_blender_color_picker(
        &cp,
        Rect::new(0.0, 0.0, 280.0, 560.0),
        &mut scene,
        &mut text,
        theme,
    );
}

#[test]
fn paint_smoke_default() {
    smoke(BlenderColorPicker::new(NodeId(1), "x"), Theme::Forge);
}

#[test]
fn paint_smoke_hsv_mode() {
    smoke(
        BlenderColorPicker::new(NodeId(1), "x")
            .channel_mode(ChannelMode::Hsv)
            .interpolation(InterpolationMode::Linear),
        Theme::Sunstone,
    );
}

#[test]
fn paint_smoke_oklch_mode() {
    smoke(
        BlenderColorPicker::new(NodeId(1), "x")
            .channel_mode(ChannelMode::Oklch)
            .value(ColorValue::from_rgba8(120, 200, 80, 255)),
        Theme::Forge,
    );
}

#[test]
fn oklch_mode_swap_does_not_change_value() {
    let cp = BlenderColorPicker::new(NodeId(1), "x")
        .channel_mode(ChannelMode::Oklch)
        .value(ColorValue::from_rgba8(120, 200, 80, 255));
    assert_eq!(cp.value.rgba, [120, 200, 80, 255]);
}

#[test]
fn oklch_lcha_builds_oklch_color() {
    // The picker emits an L/C/H/A tuple (editor-core keeps ph2d-color
    // dev-only); the consumer builds OklchColor from it. White → L≈1,
    // C≈0.
    let cp = BlenderColorPicker::new(NodeId(1), "x").value(ColorValue::from_rgba8(255, 255, 255, 255));
    let (l, c, _h, a) = cp.oklch_lcha();
    let oklch = ph2d_color::OklchColor::new(l, c, _h, a);
    assert!((oklch.l - 1.0).abs() < 0.02, "white L≈1, got {}", oklch.l);
    assert!(oklch.c < 0.02, "white chroma≈0, got {}", oklch.c);
    assert!((oklch.a - 1.0).abs() < 1e-3);
}

#[test]
fn oklch_norm_channels_in_unit_range() {
    for rgba in [[255, 0, 0, 255], [0, 255, 0, 128], [0, 0, 0, 255], [255, 255, 255, 255]] {
        let cv = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
        for v in oklch_norm_channels(cv.oklch) {
            assert!((0.0..=1.0).contains(&v), "channel {v} out of 0..1 for {rgba:?}");
        }
    }
}

#[test]
fn chroma_slider_top_is_reachable() {
    // "Chroma quase nunca chega a 1": with gamut-relative normalization,
    // pushing chroma to norm=1 produces a value whose displayed chroma
    // reads back at ~1 (the gamut boundary), not pinned far below.
    let base = ColorValue::from_rgba8(120, 80, 60, 255);
    let maxed = oklch_set_channel(base.oklch, 1, 1.0);
    let displayed_c = oklch_norm_channels(maxed.oklch)[1];
    assert!(
        displayed_c > 0.9,
        "chroma slider should reach ~1 (gamut-relative), got {displayed_c}"
    );
}

#[test]
fn oklch_channel_edits_are_round_trip_free() {
    // Smoke report "can't move Chroma/Hue until I change Lightness": the
    // edited value's STORED oklch must reflect the norm exactly, so the
    // slider thumb tracks (no snap-back) and hue persists at chroma≈0.
    // Near-white gray (high L, chroma≈0) — the problematic default.
    let gray = ColorValue::from_rgba8(231, 231, 231, 255);
    // Drag Hue to ~0.5 turn: even though the color stays ~gray, the
    // stored hue must read back at ~0.5 (not collapse to 0).
    let hue_set = oklch_set_channel(gray.oklch, 2, 0.5);
    assert!(
        (oklch_norm_channels(hue_set.oklch)[2] - 0.5).abs() < 0.02,
        "hue must persist at chroma≈0, got {}",
        oklch_norm_channels(hue_set.oklch)[2]
    );
    // Drag Chroma to 0.6: the displayed chroma reads back at ~0.6 (no
    // 8-bit snap-back), because we store the exact oklch.
    let chroma_set = oklch_set_channel(hue_set.oklch, 1, 0.6);
    assert!(
        (oklch_norm_channels(chroma_set.oklch)[1] - 0.6).abs() < 0.02,
        "chroma thumb must track (no snap-back), got {}",
        oklch_norm_channels(chroma_set.oklch)[1]
    );
}

#[test]
fn max_in_gamut_chroma_zero_at_lightness_extremes() {
    // Pure black / white can't carry chroma.
    assert_eq!(max_in_gamut_chroma(0.0, 30.0), 0.0);
    assert_eq!(max_in_gamut_chroma(1.0, 30.0), 0.0);
    // A mid lightness has representable chroma.
    assert!(max_in_gamut_chroma(0.6, 30.0) > 0.05);
}

#[test]
fn oklch_set_channel_lightness_monotone() {
    // Raising normalized lightness on a mid color should not decrease
    // the resulting perceptual luminance proxy (sum of RGB).
    let base = ColorValue::from_rgba8(120, 80, 60, 255);
    let darker = oklch_set_channel(base.oklch, 0, 0.2);
    let lighter = oklch_set_channel(base.oklch, 0, 0.9);
    let sum = |c: ColorValue| c.rgba[0] as u32 + c.rgba[1] as u32 + c.rgba[2] as u32;
    assert!(sum(lighter) > sum(darker), "higher L should brighten");
    // Alpha channel edit leaves the sRGB color intact.
    let a_edit = oklch_set_channel(base.oklch, 3, 0.5);
    assert_eq!(
        [a_edit.rgba[0], a_edit.rgba[1], a_edit.rgba[2]],
        [base.rgba[0], base.rgba[1], base.rgba[2]]
    );
}

#[test]
fn paint_smoke_red_value() {
    smoke(
        BlenderColorPicker::new(NodeId(1), "x").value(ColorValue::from_rgba8(220, 40, 40, 255)),
        Theme::Blueprint,
    );
}

#[test]
fn paint_smoke_with_alpha() {
    smoke(
        BlenderColorPicker::new(NodeId(1), "x").value(ColorValue::from_rgba8(40, 40, 220, 128)),
        Theme::Workshop,
    );
}

#[test]
fn pick_swatch_updates_value_and_active() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    let custom = ColorPalette::new(
        "Custom",
        vec![
            ColorValue::from_rgba8(10, 20, 30, 255),
            ColorValue::from_rgba8(40, 50, 60, 255),
        ],
    );
    cp.palettes.push(custom);
    assert!(cp.pick_swatch(1, 1));
    assert_eq!(cp.value.rgba, [40, 50, 60, 255]);
    assert_eq!(cp.active_palette, 1);
}

#[test]
fn pick_swatch_oob_is_noop() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    let original = cp.value.rgba;
    assert!(!cp.pick_swatch(99, 0));
    assert!(!cp.pick_swatch(0, 99));
    assert_eq!(cp.value.rgba, original);
}

#[test]
fn try_set_hex_commits_valid() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    assert!(cp.try_set_hex("#101820"));
    assert_eq!(cp.value.rgba, [0x10, 0x18, 0x20, 0xFF]);
    assert_eq!(cp.hex, "#101820FF");
}

#[test]
fn try_set_hex_rejects_invalid_keeps_value() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    let original = cp.value.rgba;
    assert!(!cp.try_set_hex("nope"));
    assert_eq!(cp.value.rgba, original);
}

#[test]
fn add_swatch_only_when_palette_editable() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    // Default palette is now editable — add returns Some.
    let initial = cp.palettes[0].swatches.len();
    let idx = cp.add_swatch(ColorValue::WHITE).unwrap();
    assert_eq!(idx, initial);
    assert_eq!(cp.palettes[0].swatches.len(), initial + 1);
    // A read-only palette still rejects.
    cp.palettes
        .push(ColorPalette::new("Locked", vec![]).read_only());
    cp.active_palette = 1;
    assert!(cp.add_swatch(ColorValue::WHITE).is_none());
}

#[test]
fn remove_swatch_from_editable_palette() {
    let mut cp = BlenderColorPicker::new(NodeId(1), "x");
    cp.palettes.push(ColorPalette::new(
        "Custom",
        vec![ColorValue::WHITE, ColorValue::BLACK],
    ));
    cp.active_palette = 1;
    assert!(cp.remove_swatch(0));
    assert_eq!(cp.palettes[1].swatches.len(), 1);
    assert_eq!(cp.palettes[1].swatches[0].rgba, ColorValue::BLACK.rgba);
}
