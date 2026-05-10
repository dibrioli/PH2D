//! Top-level layout: orchestrates wheel + value slider + segmented
//! toggles + 4 channel rows + hex field + palettes.

use super::channels::{paint_slider_row, rgba_to_hsv};
use super::hex_field::{paint_eyedropper, paint_hex_field};
use super::palette::paint_palettes;
use super::segmented::{paint_channel_toggle, paint_interpolation_toggle};
use super::state::{BlenderColorPicker, ChannelMode};
use super::value_slider::paint_value_slider;
use super::wheel::paint_color_wheel;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
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

/// Paint the picker driven by retained state from a [`WidgetStore`]
/// entry at `parent_id`. Reads `value`/`channel_mode`/`interpolation`/
/// `active_palette` from the store, applies them onto a local
/// [`BlenderColorPicker`] clone, paints, and registers click hit
/// rects for the wheel + value slider sub-controls so dispatch can
/// route Down events back into store mutations.
///
/// `wheel_id` and `value_slider_id` are the per-sub-control ids the
/// caller wants the [`HitIndex`] to know about — they don't need to
/// be pre-registered in the store; the dispatch helpers
/// `apply_blender_wheel_pick` / `apply_blender_value_pick` look the
/// parent state up by `parent_id`.
#[allow(clippy::too_many_arguments)]
pub fn paint_blender_color_picker_with_store(
    cp: &BlenderColorPicker,
    rect: Rect,
    parent_id: NodeId,
    wheel_id: NodeId,
    value_slider_id: NodeId,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let mut local = cp.clone();
    if let Some((value, channel_mode, interpolation, active_palette)) =
        store.blender_picker(parent_id)
    {
        local.value = value;
        local.channel_mode = channel_mode;
        local.interpolation = interpolation;
        local.active_palette = active_palette;
        local.sync_hex();
    }
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let pad = Spacing::Lg.px();
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad;

    let wheel_block_w = WHEEL_SIZE + Spacing::Md.px() + VALUE_SLIDER_W;
    let wheel_x = rect.x + (rect.w - wheel_block_w) * 0.5;
    let wheel_rect = Rect::new(wheel_x, y, WHEEL_SIZE, WHEEL_SIZE);
    paint_color_wheel(&local, wheel_rect, scene);
    hit_index.register(wheel_id, wheel_rect);
    let value_rect = Rect::new(
        wheel_x + WHEEL_SIZE + Spacing::Md.px(),
        y,
        VALUE_SLIDER_W,
        WHEEL_SIZE,
    );
    paint_value_slider(&local, value_rect, scene, theme);
    hit_index.register(value_slider_id, value_rect);
    hit_index.register(parent_id, rect);
    y += WHEEL_SIZE + ROW_GAP;

    let interp_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_interpolation_toggle(&local, interp_rect, scene, text_system, theme);
    y += TOGGLE_H + ROW_GAP;

    let chan_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_channel_toggle(&local, chan_rect, scene, text_system, theme);
    y += TOGGLE_H + ROW_GAP;

    let labels = match local.channel_mode {
        ChannelMode::Rgb => ["Red", "Green", "Blue", "Alpha"],
        ChannelMode::Hsv => ["Hue", "Saturation", "Value", "Alpha"],
    };
    let values = match local.channel_mode {
        ChannelMode::Rgb => [
            local.value.rgba[0] as f32 / 255.0,
            local.value.rgba[1] as f32 / 255.0,
            local.value.rgba[2] as f32 / 255.0,
            local.value.rgba[3] as f32 / 255.0,
        ],
        ChannelMode::Hsv => {
            let (h, s, v, a) = rgba_to_hsv(local.value.rgba);
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
    paint_hex_field(&local.hex, hex_rect, scene, text_system, theme);
    paint_eyedropper(eye_rect, scene, theme);
    y += HEX_ROW_H + ROW_GAP;

    let palette_h = (rect.y + rect.h - y - pad).max(0.0);
    let palette_rect = Rect::new(rect.x + pad, y, inner_w, palette_h);
    if palette_h > 60.0 {
        paint_palettes(&local, palette_rect, scene, text_system, theme);
    }
}
