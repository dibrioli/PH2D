//! Top-level layout: orchestrates wheel + value slider + segmented
//! toggles + 4 channel rows + hex field + palettes.

use super::channels::{oklch_norm_channels, paint_slider_row, rgba_to_hsv};
use super::hex_field::{paint_eyedropper, paint_hex_field};
use super::palette::{paint_palette_section, paint_palettes};
use super::segmented::paint_channel_toggle;
use super::state::{BlenderColorPicker, ChannelMode, ColorPalette};
use super::sub_ids::BlenderSubIds;
use super::value_slider::paint_value_slider;
use super::wheel::paint_color_wheel;
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

pub const SV_RECT_H: f32 = 150.0;
pub const HUE_STRIP_H: f32 = 16.0;
pub const ROW_GAP: f32 = 8.0;
pub const TOGGLE_H: f32 = 28.0;
pub const SLIDER_ROW_H: f32 = 22.0;
pub const HEX_ROW_H: f32 = 28.0;
pub const PREVIEW_H: f32 = 24.0;

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
        ChannelMode::Oklch => ["Lightness", "Chroma", "Hue", "Alpha"],
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
        ChannelMode::Oklch => oklch_norm_channels(cp.value.oklch),
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
    local.harmony = store.blender_harmony(parent_id);
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Full-rect hit BARRIER, registered FIRST so the sub-controls below
    // (registered AFTER → win the back-to-front walk) keep their clicks
    // while dead space hits the picker, not the panel beneath it. The
    // picker paints after every panel, so this outranks them (fixes
    // click-through, 2026-05-31). The picker container is non-focusable
    // (`is_focusable`), so the barrier blocks without becoming active.
    if parent_id.0 != 0 {
        hit_index.register(parent_id, rect);
    }

    let pad = Spacing::Lg.px();
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad;

    // Top bar: a slim drag handle (three dot grips, click+drag to move) on the left and
    // a "×" close button on the right. The drag bar is shrunk to reserve the close slot;
    // the close hit registers AFTER the drag handle so it wins where they abut.
    let drag_h = 14.0_f32;
    let top_gap = Spacing::Xs.px();
    let close_sz = drag_h;
    let drag_rect = Rect::new(
        rect.x + pad,
        y,
        (inner_w - close_sz - top_gap).max(0.0),
        drag_h,
    );
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
    let close_rect = Rect::new(rect.x + pad + inner_w - close_sz, y, close_sz, drag_h);
    paint_icon(
        scene,
        IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );
    if ids.close.0 != 0 {
        hit_index.register(ids.close, close_rect);
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
    // (`parent_id` barrier was registered at the top of this fn.)
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

    // Channel mode toggle (RGB / HSV / OKLCH) — three equal segments,
    // matching the 3-option RadioGroup painted in `paint_channel_toggle`.
    let chan_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    paint_channel_toggle(&local, chan_rect, scene, text_system, theme);
    if ids.channel_rgb.0 != 0 || ids.channel_hsv.0 != 0 || ids.channel_oklch.0 != 0 {
        let seg_w = chan_rect.w / 3.0;
        for (i, id) in [ids.channel_rgb, ids.channel_hsv, ids.channel_oklch]
            .into_iter()
            .enumerate()
        {
            if id.0 != 0 {
                hit_index.register(
                    id,
                    Rect::new(
                        chan_rect.x + seg_w * i as f32,
                        chan_rect.y,
                        seg_w,
                        chan_rect.h,
                    ),
                );
            }
        }
    }
    y += TOGGLE_H + ROW_GAP;

    let labels = match local.channel_mode {
        ChannelMode::Rgb => ["Red", "Green", "Blue", "Alpha"],
        ChannelMode::Hsv => ["Hue", "Saturation", "Value", "Alpha"],
        ChannelMode::Oklch => ["Lightness", "Chroma", "Hue", "Alpha"],
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
        // OKLCH channels derive directly from the sRGB value (no
        // retained anchor): L/C/H normalized to 0..1 for the uniform
        // slider model. Gray collapses hue to 0 — acceptable, as the
        // hue strip above stays HSV-spatial and OKLCH rows are a
        // numeric alt-view.
        ChannelMode::Oklch => oklch_norm_channels(local.value.oklch),
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
        "Hex",
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

    // Color Harmonies — the scheme selector + derived partner swatches. Because this is the ONE
    // shared picker every module opens, adding it here is the "global tool" (Enio 2026-07-25): Painter,
    // Vector, Inspector all get it. The section returns the height it drew (selector only when Off).
    let harmony_rect = Rect::new(rect.x + pad, y, inner_w, super::harmony::HARMONY_SEL_H);
    let harmony_used = super::harmony::paint_harmony_section(
        &local,
        harmony_rect,
        ids,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += harmony_used + ROW_GAP;

    // Rebuild the local picker's palette set from the store (the runtime source of truth): every named
    // palette's name + swatches, so the dropdown + swatch grid reflect every CRUD / import edit.
    // The active index already came from the store state (above); clamp it onto the rebuilt set.
    if let Some(set) = store.blender_palette_set(parent_id) {
        local.palettes = set
            .iter()
            .map(|p| ColorPalette::new(p.name.clone(), p.swatches.clone()))
            .collect();
        local.active_palette = local
            .active_palette
            .min(local.palettes.len().saturating_sub(1));
    }
    // Palette section — dropdown header (select + New/Rename/Delete) + optional inline rename field +
    // swatch grid + Import/Export + the deferred dropdown popover. Owns its own store reads (rename /
    // dropdown open flags, name-field buffer); see [`paint_palette_section`].
    let palette_h = (rect.y + rect.h - y - pad).max(0.0);
    if palette_h > 60.0 {
        let palette_rect = Rect::new(rect.x + pad, y, inner_w, palette_h);
        paint_palette_section(
            &local,
            palette_rect,
            rect,
            ids,
            store,
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
