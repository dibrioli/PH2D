//! Top-level layout: orchestrates wheel + value slider + segmented
//! toggles + 4 channel rows + hex field + palettes.

use super::channels::{paint_slider_row, rgba_to_hsv};
use super::hex_field::{paint_eyedropper, paint_hex_field};
use super::palette::{paint_palettes, paint_palettes_with_hits};
use super::segmented::paint_channel_toggle;
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

pub const SV_RECT_H: f32 = 150.0;
pub const HUE_STRIP_H: f32 = 16.0;
pub const ROW_GAP: f32 = 8.0;
pub const TOGGLE_H: f32 = 28.0;
pub const SLIDER_ROW_H: f32 = 22.0;
pub const HEX_ROW_H: f32 = 28.0;
pub const PREVIEW_H: f32 = 24.0;

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
    /// Channel value chip `NumberInput` ids 0..4 (mirror channels).
    pub channels_num: [NodeId; 4],
    /// Hex TextInput id.
    pub hex: NodeId,
    /// "+ swatch" button id (appends current value to palette).
    pub add_swatch: NodeId,
    /// Eyedropper button id.
    pub eyedropper: NodeId,
    /// Drag-handle bar id (at top of picker — drag to reposition).
    pub drag_handle: NodeId,
    /// Palette swatch ids (up to 27). Entries with id == 0 are
    /// skipped. The first 12 cover the default palette; the rest
    /// cover user "+ swatch" additions (capped to keep the array
    /// fixed-size).
    pub swatches: [NodeId; 27],
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
            channels_num: [NodeId(0); 4],
            hex: NodeId(0),
            add_swatch: NodeId(0),
            eyedropper: NodeId(0),
            drag_handle: NodeId(0),
            swatches: [NodeId(0); 27],
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

    let sv_rect = Rect::new(rect.x + pad, y, inner_w, SV_RECT_H);
    paint_color_wheel(cp, sv_rect, scene);
    y += SV_RECT_H + ROW_GAP;

    let hue_rect = Rect::new(rect.x + pad, y, inner_w, HUE_STRIP_H);
    paint_value_slider(cp, hue_rect, scene, theme);
    y += HUE_STRIP_H + ROW_GAP;

    let preview_rect = Rect::new(rect.x + pad, y, inner_w, PREVIEW_H);
    paint_color_preview(cp, preview_rect, scene, theme);
    y += PREVIEW_H + ROW_GAP;

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
            // Same retained-anchor read as the with-store variant
            // (see comment there) — H/S survive V→0 collapse.
            let (_, _, v, a) = rgba_to_hsv(cp.value.rgba);
            [cp.hsv_h, cp.hsv_s, v, a]
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
    if let Some((h, s)) = store.blender_hsv_anchor(parent_id) {
        local.hsv_h = h;
        local.hsv_s = s;
    }
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let pad = Spacing::Lg.px();
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad;

    // Drag handle — slim bar across the top with three dot grips.
    // Click+drag to move the picker (host updates rect via stored
    // offset before calling this painter).
    let drag_h = 14.0_f32;
    let drag_rect = Rect::new(rect.x + pad, y, inner_w, drag_h);
    fill_rounded_rect(
        scene,
        drag_rect,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let dot_y = drag_rect.y + drag_rect.h * 0.5 - 1.5;
    let dot_color = resolve(ColorToken::Text3, theme);
    for i in 0..3i32 {
        let dot_x = drag_rect.x + drag_rect.w * 0.5 + (i - 1) as f32 * 6.0 - 1.5;
        let dot_rect = Rect::new(dot_x, dot_y, 3.0, 3.0);
        fill_rounded_rect(scene, dot_rect, 1.5, dot_color);
    }
    if ids.drag_handle.0 != 0 {
        hit_index.register(ids.drag_handle, drag_rect);
    }
    y += drag_h + ROW_GAP;

    // Web-standard SV rectangle (replaces the HSV disc). Full
    // picker width, fixed height — saturation runs left→right,
    // value runs top→bottom (top = bright, bottom = black).
    let sv_rect = Rect::new(rect.x + pad, y, inner_w, SV_RECT_H);
    paint_color_wheel(&local, sv_rect, scene);
    if ids.wheel.0 != 0 {
        hit_index.register(ids.wheel, sv_rect);
    }
    y += SV_RECT_H + ROW_GAP;

    // Horizontal hue strip (replaces the vertical V slider). Click
    // anywhere along it sets the hue while preserving S + V.
    let hue_rect = Rect::new(rect.x + pad, y, inner_w, HUE_STRIP_H);
    paint_value_slider(&local, hue_rect, scene, theme);
    if ids.value_slider.0 != 0 {
        hit_index.register(ids.value_slider, hue_rect);
    }
    // Intentionally NOT registering `parent_id` (the outer picker
    // rect) in the hit index: it would shadow the SV + hue strip
    // sub-rects since `HitIndex::hit` walks back-to-front and would
    // pick the most-recently-registered rect (the outer one) for
    // any click that lands inside it.
    y += HUE_STRIP_H + ROW_GAP;

    // Resulting-color preview swatch — full-width strip showing the
    // current `value.rgba` so the user can verify the pick before
    // committing it elsewhere.
    let preview_rect = Rect::new(rect.x + pad, y, inner_w, PREVIEW_H);
    paint_color_preview(&local, preview_rect, scene, theme);
    y += PREVIEW_H + ROW_GAP;

    // Linear / Perceptual interpolation toggle removed by design —
    // OKLCH-perceptual mixing was a Blender-specific concept the
    // simplified picker doesn't expose. The state field still exists
    // (defaults to Perceptual) but the toggle row is no longer
    // painted and `interp_linear`/`interp_perceptual` ids are not
    // registered in the hit index.
    let _ = ids.interp_linear;
    let _ = ids.interp_perceptual;

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
            // H + S come from the retained anchor — RGBA→HSV would
            // collapse H to 0 (red) on V=0 / S=0 colors and report
            // 0.000 in the chip even though the hue strip stays at
            // the user's chosen position. V + A are recoverable
            // from RGBA.
            let (_, _, v, a) = rgba_to_hsv(local.value.rgba);
            [local.hsv_h, local.hsv_s, v, a]
        }
    };
    for (i, (label, val)) in labels.iter().zip(values.iter()).enumerate() {
        let row_y = y + (SLIDER_ROW_H + 4.0) * i as f32;
        let row_rect = Rect::new(rect.x + pad, row_y, inner_w, SLIDER_ROW_H);
        let ch_id = ids.channels.get(i).copied().unwrap_or(NodeId(0));
        let chip_id = ids.channels_num.get(i).copied().unwrap_or(NodeId(0));
        // Canonical slider+chip composite — the rest of the app uses
        // this same painter (Inspector field rows etc.). Both ids
        // register their own hit rects inside.
        crate::widget::paint_slider_with_chip(
            row_rect,
            label,
            *val,
            ch_id,
            chip_id,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }
    y += (SLIDER_ROW_H + 4.0) * 4.0 + ROW_GAP;

    let hex_rect = Rect::new(rect.x + pad, y, inner_w - 32.0, HEX_ROW_H);
    let eye_rect = Rect::new(hex_rect.x + hex_rect.w + 4.0, y, HEX_ROW_H, HEX_ROW_H);
    // Read live buffer/caret/state from the WidgetStore entry for
    // the hex TextInput so typing is visible (caret + buffer +
    // focus border). Falls back to `local.hex` when not registered.
    let (hex_state, hex_buffer, hex_caret, hex_anchor) = if ids.hex.0 != 0 {
        match store.get(ids.hex) {
            Some(crate::interaction::InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
            _ => (crate::widget::TextInputState::Normal, None, 0, None),
        }
    } else {
        (crate::widget::TextInputState::Normal, None, 0, None)
    };
    super::hex_field::paint_hex_field_with_state(
        &local.hex,
        hex_buffer,
        hex_caret,
        hex_anchor,
        hex_state,
        hex_rect,
        scene,
        text_system,
        theme,
    );
    let eyedropper_active = store.eyedropper_pending() == Some(parent_id);
    super::hex_field::paint_eyedropper_with_state(eye_rect, eyedropper_active, scene, theme);
    if ids.hex.0 != 0 {
        hit_index.register(ids.hex, hex_rect);
    }
    if ids.eyedropper.0 != 0 {
        hit_index.register(ids.eyedropper, eye_rect);
    }
    y += HEX_ROW_H + ROW_GAP;

    // Patch the local picker's active palette swatches with the
    // store-side mutable palette (init_blender_palette + push/remove).
    if let Some(swatches) = store.blender_palette(parent_id)
        && let Some(palette) = local.palettes.get_mut(local.active_palette)
    {
        palette.swatches = swatches.to_vec();
        palette.editable = true;
    }
    let palette_h = (rect.y + rect.h - y - pad).max(0.0);
    let palette_rect = Rect::new(rect.x + pad, y, inner_w, palette_h);
    if palette_h > 60.0 {
        paint_palettes_with_hits(
            &local,
            palette_rect,
            &ids.swatches,
            ids.add_swatch,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }
}

/// Paint the resulting-color preview swatch — full-width strip
/// showing the current `value.rgba` over a checkerboard so partial
/// alpha is legible (light/dark squares show through translucent
/// colors, matching what every other paint app does).
fn paint_color_preview(cp: &BlenderColorPicker, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    // Light backdrop covering the whole rect (the "white" cells of
    // the checker).
    fill_rounded_rect(
        scene,
        rect,
        radius,
        ph2d_vector::Color::from_rgba8(220, 220, 220, 255),
    );
    // Darker squares — every other cell. Inset by `radius` on all
    // sides so the checker stays inside the rounded outline; with
    // a translucent overlay the corners would otherwise show tiny
    // squares poking past the curve.
    let cell = 6.0_f32;
    let chk_x = rect.x + radius;
    let chk_y = rect.y + radius;
    let chk_w = (rect.w - radius * 2.0).max(0.0);
    let chk_h = (rect.h - radius * 2.0).max(0.0);
    let cols = (chk_w / cell).ceil() as i32;
    let rows = (chk_h / cell).ceil() as i32;
    let dark = ph2d_vector::Color::from_rgba8(170, 170, 170, 255);
    for j in 0..rows {
        for i in 0..cols {
            if (i + j) % 2 == 0 {
                continue;
            }
            let cx = chk_x + (i as f32) * cell;
            let cy = chk_y + (j as f32) * cell;
            let w = cell.min(chk_x + chk_w - cx);
            let h = cell.min(chk_y + chk_h - cy);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            let kr = ph2d_vector::Rect::new(cx as f64, cy as f64, (cx + w) as f64, (cy + h) as f64);
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(dark),
                None,
                &kr,
            );
        }
    }
    // Color overlay (with whatever alpha the picker carries).
    let [r, g, b, a] = cp.value.rgba;
    fill_rounded_rect(
        scene,
        rect,
        radius,
        ph2d_vector::Color::from_rgba8(r, g, b, a),
    );
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
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
