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

pub use paint::paint_blender_color_picker;
pub use state::{
    BlenderColorPicker, ChannelMode, ColorPalette, InterpolationMode, default_palette,
};
pub use value_slider::value_pick;
pub use wheel::wheel_pick;
