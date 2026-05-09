//! Widget primitives — pure data + AccessKit nodes. Rendering moved
//! to the shell (egui owns paint after the egui-migration pivot).

mod button;
mod color_swatch;
mod radio_group;
mod slider;
mod toggle;

pub use button::{Button, ButtonState};
pub use color_swatch::{ColorSwatch, SwatchState};
pub use radio_group::{RadioGroup, RadioOption, RadioOrientation};
pub use slider::{Slider, SliderState};
pub use toggle::{Toggle, ToggleState};
