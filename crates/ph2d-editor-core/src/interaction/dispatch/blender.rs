//! BlenderColorPicker dispatch helpers.
//!
//! Extracted from [`super`] (Track A5). The picker has many
//! sub-controls (wheel, value slider, channel chips, hex field,
//! palette swatches, drag handle, resize gripper, eyedropper) — each
//! registers a [`super::super::InteractiveState::BlenderHit`] entry
//! with the dispatcher pointing back at the parent picker's NodeId.
//! [`apply_blender_hit`] is the picker-level click router that takes
//! a Down hit on any sub-control and applies it to the parent's
//! state. [`derive_blender_channel_value`] / [`apply_blender_channel_value`]
//! are the read/write primitives used when a channel chip's
//! NumberInput gains/loses focus.

use super::super::{InteractiveState, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Read a single channel value (0..=1) from the parent picker's
/// current `value` + `channel_mode`. Used when seeding a chip's
/// edit buffer on focus arrival.
pub(super) fn derive_blender_channel_value(store: &WidgetStore, parent: NodeId, idx: u8) -> f64 {
    use crate::widget::{ChannelMode, oklch_norm_channels, rgba_to_hsv};
    let Some((cur, mode, _, _)) = store.blender_picker(parent) else {
        return 0.0;
    };
    match mode {
        ChannelMode::Rgb => cur.rgba[idx as usize] as f64 / 255.0,
        ChannelMode::Hsv => {
            let (h, s, v, a) = rgba_to_hsv(cur.rgba);
            [h, s, v, a][idx as usize] as f64
        }
        ChannelMode::Oklch => oklch_norm_channels(cur.oklch)[idx as usize] as f64,
    }
}

/// Rewrite the parent BlenderPicker's color value with `new_norm`
/// (0..=1) at channel index `idx`. Honors the parent's current
/// `channel_mode`: in RGB mode `idx` maps to R/G/B/A, in HSV mode
/// `idx` maps to H/S/V/A (then converted to RGBA via
/// [`crate::widget::hsv_to_rgba8`]).
pub(super) fn apply_blender_channel_value(
    store: &mut WidgetStore,
    parent: NodeId,
    idx: u8,
    new_norm: f32,
) {
    use crate::widget::{ChannelMode, hsv_to_rgba8, rgba_to_hsv};
    use ph2d_tokens::ColorValue;
    let Some((cur, mode, _, _)) = store.blender_picker(parent) else {
        return;
    };
    let n = new_norm.clamp(0.0, 1.0);
    match mode {
        ChannelMode::Rgb => {
            let mut rgba = cur.rgba;
            rgba[idx as usize] = (n * 255.0).round() as u8;
            let new_value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
            store.set_blender_value(parent, new_value);
        }
        ChannelMode::Hsv => {
            // Use the retained (h, s) anchor as the canonical HSV
            // basis — RGBA→HSV would collapse H/S on V=0 / S=0
            // states and silently rotate the user's hue back to red
            // when they edit V or A. Only the channel being changed
            // gets overwritten with `n`; the others stay retained.
            let (retained_h, retained_s) = store.blender_hsv_anchor(parent).unwrap_or((0.0, 1.0));
            let (_, _, v_rgba, a_rgba) = rgba_to_hsv(cur.rgba);
            let mut h = retained_h;
            let mut s = retained_s;
            let mut v = v_rgba;
            let mut a = a_rgba;
            match idx {
                0 => h = n,
                1 => s = n,
                2 => v = n,
                3 => a = n,
                _ => {}
            }
            let rgba = hsv_to_rgba8(h, s, v, a);
            let new_value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
            store.set_blender_value_with_hsv(parent, new_value, h, s);
        }
        ChannelMode::Oklch => {
            // Edit one OKLCH channel off the value's STORED oklch and
            // rebuild via `from_oklch` (round-trip-free), so chroma/hue
            // persist across edits even at chroma≈0 / gamut clamp.
            let new_value = crate::widget::oklch_set_channel(cur.oklch, idx, n);
            store.set_blender_value(parent, new_value);
        }
    }
}

/// If `id` is a [`super::super::InteractiveState::BlenderHit`]
/// sub-control, apply the click to its parent
/// [`super::super::InteractiveState::BlenderPicker`] and return the
/// parent's id (so the caller can emit `ValueChanged`). Returns
/// `None` for non-picker hits.
pub(super) fn apply_blender_hit(
    store: &mut WidgetStore,
    id: NodeId,
    rect: Rect,
    px: f32,
    py: f32,
    button: ph2d_host::PointerButton,
) -> Option<NodeId> {
    use crate::interaction::BlenderHitKind;
    use crate::widget::{
        ChannelMode, InterpolationMode, apply_blender_value_pick, apply_blender_wheel_pick,
    };
    let (parent, kind) = match store.get(id)? {
        InteractiveState::BlenderHit { parent, kind } => (*parent, *kind),
        _ => return None,
    };
    match kind {
        BlenderHitKind::Wheel => {
            apply_blender_wheel_pick(store, parent, rect, px, py).then_some(parent)
        }
        BlenderHitKind::ValueSlider => {
            apply_blender_value_pick(store, parent, rect, px, py).then_some(parent)
        }
        BlenderHitKind::InterpolationLinear => {
            store.set_blender_interpolation(parent, InterpolationMode::Linear);
            Some(parent)
        }
        BlenderHitKind::InterpolationPerceptual => {
            store.set_blender_interpolation(parent, InterpolationMode::Perceptual);
            Some(parent)
        }
        BlenderHitKind::ChannelRgb => {
            store.set_blender_channel_mode(parent, ChannelMode::Rgb);
            Some(parent)
        }
        BlenderHitKind::ChannelHsv => {
            store.set_blender_channel_mode(parent, ChannelMode::Hsv);
            Some(parent)
        }
        BlenderHitKind::ChannelOklch => {
            store.set_blender_channel_mode(parent, ChannelMode::Oklch);
            Some(parent)
        }
        BlenderHitKind::ChannelSlider(idx) => {
            // The hit rect is now the slider track itself (the
            // painter registers only the inner track region, not
            // the full row). Direct normalisation against rect.x
            // and rect.w gives the value cleanly.
            let norm = if rect.w > 0.0 {
                ((px - rect.x) / rect.w).clamp(0.0, 1.0)
            } else {
                0.0
            };
            store.set_blender_channel(parent, idx, norm);
            Some(parent)
        }
        BlenderHitKind::Hex => {
            // `Hex` hits redirect focus to the hex TextInput sibling
            // registered with the same parent id hierarchy. The
            // BlenderHit NodeId IS the hex TextInput id in our
            // registration scheme (BLENDER_HEX = NodeId 604 which is
            // registered as TextInput, not as BlenderHit). This arm
            // is a no-op: focus was already set by the Down handler.
            // Return None so no spurious ValueChanged fires.
            None
        }
        BlenderHitKind::PaletteSwatch(swatch_idx) => {
            // Right-click: remove the swatch from the picker's
            // palette. Left/middle: pick its color.
            if button == ph2d_host::PointerButton::Secondary {
                if store.blender_palette_remove(parent, swatch_idx as usize) {
                    Some(parent)
                } else {
                    None
                }
            } else {
                let color = store
                    .blender_palette(parent)
                    .and_then(|p| p.get(swatch_idx as usize).copied());
                if let Some(color) = color {
                    store.set_blender_value(parent, color);
                    Some(parent)
                } else {
                    None
                }
            }
        }
        BlenderHitKind::AddSwatch => {
            // Append the picker's current value to the palette.
            let (cur, _, _, _) = store.blender_picker(parent)?;
            store.blender_palette_push(parent, cur);
            Some(parent)
        }
        BlenderHitKind::PaletteTab(idx) => {
            store.blender_select_palette(parent, idx as usize);
            store.sync_blender_palette_name_buffer(parent);
            Some(parent)
        }
        BlenderHitKind::NewPalette => {
            store.blender_new_palette(parent);
            store.sync_blender_palette_name_buffer(parent);
            Some(parent)
        }
        BlenderHitKind::DeletePalette => {
            store.blender_delete_active_palette(parent);
            store.sync_blender_palette_name_buffer(parent);
            Some(parent)
        }
        BlenderHitKind::ImportPalette => {
            // Flag the host to open a load dialog + add a NEW palette (the picker can't do file I/O).
            store.set_palette_io_pending(parent, crate::interaction::PaletteIoKind::Import);
            Some(parent)
        }
        BlenderHitKind::ExportPalette => {
            store.set_palette_io_pending(parent, crate::interaction::PaletteIoKind::Export);
            Some(parent)
        }
        BlenderHitKind::Eyedropper => {
            // Toggle eyedropper "pending" mode. While pending, the
            // next pointer Down anywhere except this button is
            // intercepted by the Down handler and emitted as
            // `WidgetEvent::EyedropperPick` for the host to perform
            // the GPU pixel readback. Clicking the button a second
            // time (still pending → same parent) cancels the mode.
            let already_pending = store.eyedropper_pending() == Some(parent);
            store.set_eyedropper_pending(if already_pending { None } else { Some(parent) });
            None
        }
        BlenderHitKind::DragHandle => {
            // Down on the drag bar — snapshot the cursor + current
            // offset so Move events can apply (cursor − down_cursor)
            // as the new delta. Up clears the anchor.
            store.begin_blender_drag(parent, px, py);
            None
        }
        BlenderHitKind::ResizeHandle => {
            // Down on the bottom-right gripper — anchor the cursor so
            // subsequent Moves apply incremental delta to the panel's
            // stored (dw, dh). Up clears the anchor.
            store.begin_panel_resize(parent, px, py);
            None
        }
        BlenderHitKind::ResizeHandleBl => {
            // Down on the bottom-LEFT gripper — anchor the cursor so
            // subsequent Moves shift the panel offset AND grow the
            // resize delta in opposite directions, keeping the right
            // edge stationary. Up clears the anchor (shared end via
            // `end_panel_resize`).
            store.begin_panel_resize_bl(parent, px, py);
            None
        }
        BlenderHitKind::VisibilityToggle => {
            // M14.6A: not dispatched through this `BlenderHit` arm —
            // the hierarchy eye uses an offset-NodeId companion
            // (see `hier_eye_companion` in `screens/hero/ids.rs`)
            // and routes via the regular `WidgetEvent::Click` path
            // in `HeroScreen::apply_event`. This match arm exists
            // only to keep the enum exhaustive; left as no-op.
            None
        }
    }
}
