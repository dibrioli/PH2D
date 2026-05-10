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
//! 6. **Palettes section**: Tabs (palette names) + grid of
//!    [`crate::widget::ColorSwatch`]es + add/remove buttons.
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
//! - `palette.rs` — palette tabs + swatch grid.

pub mod channels;
pub mod hex_field;
pub mod paint;
pub mod palette;
pub mod segmented;
pub mod state;
pub mod value_slider;
pub mod wheel;

#[cfg(test)]
mod tests;

pub use hex_field::parse_hex;
pub use paint::{paint_blender_color_picker, paint_blender_color_picker_with_store};
pub use state::{
    BlenderColorPicker, ChannelMode, ColorPalette, InterpolationMode, default_palette,
};
pub use value_slider::value_pick;
pub use wheel::wheel_pick;

use crate::interaction::WidgetStore;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::ColorValue;

/// Apply a click on the wheel sub-rect: translate `(px, py)` into
/// hue/sat via [`wheel_pick`], then mutate the BlenderPicker state
/// at `parent_id`. Caller must pass the wheel's registered rect.
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
    let Some((hue, sat)) = wheel_pick(rect, px, py) else {
        return false;
    };
    let chroma = sat as f64 * 0.4;
    let new_value = ColorValue::from_oklch(cur.oklch.0, chroma, hue as f64, cur.oklch.3);
    store.set_blender_value(parent_id, new_value);
    true
}

/// Apply a click on the value slider sub-rect: translate `py` into
/// OKLCH lightness via [`value_pick`], mutate the BlenderPicker.
pub fn apply_blender_value_pick(
    store: &mut WidgetStore,
    parent_id: NodeId,
    rect: Rect,
    _px: f32,
    py: f32,
) -> bool {
    let Some((cur, _, _, _)) = store.blender_picker(parent_id) else {
        return false;
    };
    let l = value_pick(rect, py) as f64;
    let new_value = ColorValue::from_oklch(l, cur.oklch.1, cur.oklch.2, cur.oklch.3);
    store.set_blender_value(parent_id, new_value);
    true
}
