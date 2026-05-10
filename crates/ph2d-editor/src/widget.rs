//! Widget primitives. Each follows the Button pattern: data struct +
//! state enum + token-resolved colors + AccessKit Node builder + a
//! `paint_X` helper colocated in the widget file (not paint.rs —
//! keeps paint.rs as the dispatch layer for compound widgets like
//! FloatingPanel and ToastQueue).

mod button;
mod color_swatch;
mod radio_group;
mod slider;
mod toggle;

pub use button::{Button, ButtonKind, ButtonState, ICON_BUTTON_SIZE_PX, paint_button};
pub use color_swatch::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};
pub use radio_group::{RadioGroup, RadioOption, RadioOrientation, paint_radio_group};
pub use slider::{Slider, SliderOrientation, SliderState, paint_slider};
pub use toggle::{Toggle, ToggleState, paint_toggle};
