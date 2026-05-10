//! Top-level layout: orchestrates wheel + value slider + segmented
//! toggles + 4 channel rows + hex field + palettes.

use super::channels::{paint_slider_row, rgba_to_hsv};
use super::hex_field::{paint_eyedropper, paint_hex_field};
use super::palette::paint_palettes;
use super::segmented::{paint_channel_toggle, paint_interpolation_toggle};
use super::state::{BlenderColorPicker, ChannelMode};
use super::value_slider::paint_value_slider;
use super::wheel::paint_color_wheel;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

pub const WHEEL_SIZE: f32 = 232.0;
pub const VALUE_SLIDER_W: f32 = 24.0;
pub const ROW_GAP: f32 = 8.0;
pub const TOGGLE_H: f32 = 28.0;
pub const SLIDER_ROW_H: f32 = 22.0;
pub const HEX_ROW_H: f32 = 28.0;

pub fn paint_blender_color_picker(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let pad = Spacing::Lg.px();
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad;

    let wheel_block_w = WHEEL_SIZE + Spacing::Md.px() + VALUE_SLIDER_W;
    let wheel_x = rect.x + (rect.w - wheel_block_w) * 0.5;
    let wheel_rect = Rect::new(wheel_x, y, WHEEL_SIZE, WHEEL_SIZE);
    paint_color_wheel(cp, wheel_rect, scene);
    let value_rect = Rect::new(
        wheel_x + WHEEL_SIZE + Spacing::Md.px(),
        y,
        VALUE_SLIDER_W,
        WHEEL_SIZE,
    );
    paint_value_slider(cp, value_rect, scene, theme);
    y += WHEEL_SIZE + ROW_GAP;

    let interp_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_interpolation_toggle(cp, interp_rect, scene, text_system, theme);
    y += TOGGLE_H + ROW_GAP;

    let chan_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_channel_toggle(cp, chan_rect, scene, text_system, theme);
    y += TOGGLE_H + ROW_GAP;

    let labels = match cp.channel_mode {
        ChannelMode::Rgb => ["Red", "Green", "Blue", "Alpha"],
        ChannelMode::Hsv => ["Hue", "Saturation", "Value", "Alpha"],
    };
    let values = match cp.channel_mode {
        ChannelMode::Rgb => [
            cp.value.rgba[0] as f32 / 255.0,
            cp.value.rgba[1] as f32 / 255.0,
            cp.value.rgba[2] as f32 / 255.0,
            cp.value.rgba[3] as f32 / 255.0,
        ],
        ChannelMode::Hsv => {
            let (h, s, v, a) = rgba_to_hsv(cp.value.rgba);
            [h, s, v, a]
        }
    };
    for (i, (label, val)) in labels.iter().zip(values.iter()).enumerate() {
        let row_y = y + (SLIDER_ROW_H + 4.0) * i as f32;
        let row_rect = Rect::new(rect.x + pad, row_y, inner_w, SLIDER_ROW_H);
        paint_slider_row(label, *val, row_rect, scene, text_system, theme);
    }
    y += (SLIDER_ROW_H + 4.0) * 4.0 + ROW_GAP;

    let hex_rect = Rect::new(rect.x + pad, y, inner_w - 32.0, HEX_ROW_H);
    let eye_rect = Rect::new(hex_rect.x + hex_rect.w + 4.0, y, HEX_ROW_H, HEX_ROW_H);
    paint_hex_field(&cp.hex, hex_rect, scene, text_system, theme);
    paint_eyedropper(eye_rect, scene, theme);
    y += HEX_ROW_H + ROW_GAP;

    let palette_h = (rect.y + rect.h - y - pad).max(0.0);
    let palette_rect = Rect::new(rect.x + pad, y, inner_w, palette_h);
    if palette_h > 60.0 {
        paint_palettes(cp, palette_rect, scene, text_system, theme);
    }
}
