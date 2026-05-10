//! Smoke + state-round-trip tests for the picker.

use super::channels::rgba_to_hsv;
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
    assert!(!cp.palettes[0].editable);
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
    assert!(!p.editable);
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
    let mut text = TextSystem::new();
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
    smoke(BlenderColorPicker::new(NodeId(1), "x"), Theme::ForgeSdf);
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
        Theme::PaintStudio,
    );
}
