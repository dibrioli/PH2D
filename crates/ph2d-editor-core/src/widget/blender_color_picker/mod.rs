//! [`BlenderColorPicker`] — Blender-style color picker widget.
//!
//! Layout (top to bottom):
//!
//! 1. **Wheel** (HSV disc) + **vertical value slider** at right.
//! 2. **Linear / Perceptual** segmented toggle (interpolation hint).
//! 3. **RGB / HSV** segmented toggle (which channel triple appears
//!    in the sliders below).
//! 4. **4 horizontal sliders** (R+G+B+A or H+S+V+A) — each row has a
//!    label, a slider track, and a NumberInput-style value chip.
//! 5. **Hex field** + **eyedropper button**.
//! 6. **Palettes section**: a dropdown of named palettes (select +
//!    New / Rename / Delete) + grid of [`crate::widget::ColorSwatch`]es
//!    + add/remove + Import/Export.
//!
//! Output value is a [`ph2d_tokens::ColorValue`] (rgba + oklch in
//! sync). Theme tokens drive every chrome color; the wheel + value
//! slider show the user content.
//!
//! Module split (per `docs/plans/2026-05-color-picker-fix.md`):
//! - `state.rs` — data model.
//! - `paint.rs` — top-level layout orchestrator + sizing constants.
//! - `wheel.rs` — color wheel disc + cursor.
//! - `value_slider.rs` — vertical value slider.
//! - `segmented.rs` — Linear/Perceptual + RGB/HSV toggles.
//! - `channels.rs` — 4 channel rows + `rgba_to_hsv` helper.
//! - `hex_field.rs` — hex `#RRGGBBAA` field + eyedropper.
//! - `palette.rs` — palette dropdown header + swatch grid.
//! - `sub_ids.rs` — the `BlenderSubIds` hit-id bundle.

pub mod channels;
pub mod hex_field;
pub mod paint;
pub mod palette;
pub mod segmented;
pub mod state;
pub mod sub_ids;
pub mod value_slider;
pub mod wheel;

#[cfg(test)]
mod tests;

pub use channels::{hsv_to_rgba8, oklch_norm_channels, oklch_set_channel, rgba_to_hsv};
pub use hex_field::parse_hex;
pub use paint::{
    paint_blender_color_picker, paint_blender_color_picker_with_store,
    paint_blender_color_picker_with_store_compat,
};
pub use state::{
    BlenderColorPicker, ChannelMode, ColorPalette, InterpolationMode, default_palette,
};
pub use sub_ids::BlenderSubIds;
pub use value_slider::value_pick;
pub use wheel::wheel_pick;

use crate::interaction::WidgetStore;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::ColorValue;

/// Apply a click on the SV-rectangle sub-rect: translate `(px, py)`
/// into HSV saturation + value via [`wheel_pick`], then rewrite the
/// parent picker's RGBA preserving the current hue and alpha.
/// Web-standard model: the rectangle controls S + V at constant H;
/// the horizontal hue strip below controls H.
pub fn apply_blender_wheel_pick(
    store: &mut WidgetStore,
    parent_id: NodeId,
    rect: Rect,
    px: f32,
    py: f32,
) -> bool {
    let Some((cur, _, _, _)) = store.blender_picker(parent_id) else {
        return false;
    };
    let Some((s, v)) = wheel_pick(rect, px, py) else {
        return false;
    };
    // Use the retained hue anchor instead of deriving from RGBA — at
    // V→0 the RGBA collapses to (0,0,0) and `rgba_to_hsv` returns
    // H=0 (red), which would silently rotate the user's hue choice.
    let (retained_h, _) = store.blender_hsv_anchor(parent_id).unwrap_or((0.0, 1.0));
    let (_, _, _, a) = channels::rgba_to_hsv(cur.rgba);
    let rgba = channels::hsv_to_rgba8(retained_h, s, v, a);
    let new_value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
    store.set_blender_value_with_hsv(parent_id, new_value, retained_h, s);
    true
}

/// Apply a click on the hue-strip sub-rect: translate `px` into a
/// HSV hue via [`value_pick`], preserving the current S + V + A.
/// Left edge of the strip = hue 0 (red), right edge = wrap-around.
pub fn apply_blender_value_pick(
    store: &mut WidgetStore,
    parent_id: NodeId,
    rect: Rect,
    px: f32,
    _py: f32,
) -> bool {
    let Some((cur, _, _, _)) = store.blender_picker(parent_id) else {
        return false;
    };
    let h = value_pick(rect, px);
    // Use retained S so dragging the hue strip while at V=0 (or at
    // pure white where S=0) still preserves the user's saturation
    // choice. V is recoverable from RGBA so we read it directly.
    let (_, retained_s) = store.blender_hsv_anchor(parent_id).unwrap_or((0.0, 1.0));
    let (_, _, v, a) = channels::rgba_to_hsv(cur.rgba);
    let rgba = channels::hsv_to_rgba8(h, retained_s, v, a);
    let new_value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
    store.set_blender_value_with_hsv(parent_id, new_value, h, retained_s);
    true
}
