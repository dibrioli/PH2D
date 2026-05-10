//! Top-level layout: orchestrates wheel + value slider + segmented
//! toggles + 4 channel rows + hex field + palettes.

use super::channels::{paint_slider_row, rgba_to_hsv};
use super::hex_field::{paint_eyedropper, paint_hex_field};
use super::palette::{paint_palettes, paint_palettes_with_hits};
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

/// All sub-control [`NodeId`]s that
/// [`paint_blender_color_picker_with_store`] needs to register in the
/// [`HitIndex`]. Stack-allocated; passed by reference so callers don't
/// pay for extra arguments. `NodeId::ZERO` slots are skipped
/// (no hit rect registered).
///
/// Build via [`BlenderSubIds::new`] or fill individual fields after
/// `Default::default()`.
#[derive(Clone, Copy, Debug)]
pub struct BlenderSubIds {
    pub parent: NodeId,
    pub wheel: NodeId,
    pub value_slider: NodeId,
    pub interp_linear: NodeId,
    pub interp_perceptual: NodeId,
    pub channel_rgb: NodeId,
    pub channel_hsv: NodeId,
    /// Channel slider ids 0..4 (R/H, G/S, B/V, A).
    pub channels: [NodeId; 4],
    /// Hex TextInput id.
    pub hex: NodeId,
    /// Palette swatch ids (up to 12). Entries with id == 0 are skipped.
    pub swatches: [NodeId; 12],
}

impl BlenderSubIds {
    /// Construct with all zero (disabled) ids.
    pub const fn zeroed() -> Self {
        Self {
            parent: NodeId(0),
            wheel: NodeId(0),
            value_slider: NodeId(0),
            interp_linear: NodeId(0),
            interp_perceptual: NodeId(0),
            channel_rgb: NodeId(0),
            channel_hsv: NodeId(0),
            channels: [NodeId(0); 4],
            hex: NodeId(0),
            swatches: [NodeId(0); 12],
        }
    }
}

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
/// entry at `ids.parent`. Reads `value`/`channel_mode`/`interpolation`/
/// `active_palette` from the store, applies them onto a local
/// [`BlenderColorPicker`] clone, paints, and registers click hit
/// rects for every sub-control so dispatch can route Down events
/// back into store mutations.
///
/// `NodeId(0)` entries inside `ids` are skipped (no hit rect).
/// See [`BlenderSubIds`] for the full list of registerable sub-rects.
#[allow(clippy::too_many_arguments)]
pub fn paint_blender_color_picker_with_store(
    cp: &BlenderColorPicker,
    rect: Rect,
    ids: &BlenderSubIds,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let parent_id = ids.parent;
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
    if ids.wheel.0 != 0 {
        hit_index.register(ids.wheel, wheel_rect);
    }
    let value_rect = Rect::new(
        wheel_x + WHEEL_SIZE + Spacing::Md.px(),
        y,
        VALUE_SLIDER_W,
        WHEEL_SIZE,
    );
    paint_value_slider(&local, value_rect, scene, theme);
    if ids.value_slider.0 != 0 {
        hit_index.register(ids.value_slider, value_rect);
    }
    if parent_id.0 != 0 {
        hit_index.register(parent_id, rect);
    }
    y += WHEEL_SIZE + ROW_GAP;

    // Interpolation toggle (Linear / Perceptual).
    let interp_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_interpolation_toggle(&local, interp_rect, scene, text_system, theme);
    if ids.interp_linear.0 != 0 || ids.interp_perceptual.0 != 0 {
        let half_w = interp_rect.w * 0.5;
        if ids.interp_linear.0 != 0 {
            hit_index.register(
                ids.interp_linear,
                Rect::new(interp_rect.x, interp_rect.y, half_w, interp_rect.h),
            );
        }
        if ids.interp_perceptual.0 != 0 {
            hit_index.register(
                ids.interp_perceptual,
                Rect::new(interp_rect.x + half_w, interp_rect.y, half_w, interp_rect.h),
            );
        }
    }
    y += TOGGLE_H + ROW_GAP;

    // Channel mode toggle (RGB / HSV).
    let chan_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_channel_toggle(&local, chan_rect, scene, text_system, theme);
    if ids.channel_rgb.0 != 0 || ids.channel_hsv.0 != 0 {
        let half_w = chan_rect.w * 0.5;
        if ids.channel_rgb.0 != 0 {
            hit_index.register(
                ids.channel_rgb,
                Rect::new(chan_rect.x, chan_rect.y, half_w, chan_rect.h),
            );
        }
        if ids.channel_hsv.0 != 0 {
            hit_index.register(
                ids.channel_hsv,
                Rect::new(chan_rect.x + half_w, chan_rect.y, half_w, chan_rect.h),
            );
        }
    }
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
        // Register the full row as the channel slider hit rect. The
        // dispatch normalises px relative to this rect to get 0..1.
        let ch_id = ids.channels.get(i).copied().unwrap_or(NodeId(0));
        if ch_id.0 != 0 {
            hit_index.register(ch_id, row_rect);
        }
    }
    y += (SLIDER_ROW_H + 4.0) * 4.0 + ROW_GAP;

    let hex_rect = Rect::new(rect.x + pad, y, inner_w - 32.0, HEX_ROW_H);
    let eye_rect = Rect::new(hex_rect.x + hex_rect.w + 4.0, y, HEX_ROW_H, HEX_ROW_H);
    paint_hex_field(&local.hex, hex_rect, scene, text_system, theme);
    paint_eyedropper(eye_rect, scene, theme);
    if ids.hex.0 != 0 {
        hit_index.register(ids.hex, hex_rect);
    }
    y += HEX_ROW_H + ROW_GAP;

    let palette_h = (rect.y + rect.h - y - pad).max(0.0);
    let palette_rect = Rect::new(rect.x + pad, y, inner_w, palette_h);
    if palette_h > 60.0 {
        paint_palettes_with_hits(
            &local,
            palette_rect,
            &ids.swatches,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }
}

/// Backward-compatible wrapper that only registers the wheel and
/// value-slider hit rects. Used by callers that predate [`BlenderSubIds`].
#[allow(clippy::too_many_arguments)]
pub fn paint_blender_color_picker_with_store_compat(
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
    let mut ids = BlenderSubIds::zeroed();
    ids.parent = parent_id;
    ids.wheel = wheel_id;
    ids.value_slider = value_slider_id;
    paint_blender_color_picker_with_store(
        cp,
        rect,
        &ids,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}
