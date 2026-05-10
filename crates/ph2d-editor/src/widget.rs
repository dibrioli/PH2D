//! Widget primitives. Each follows the Button pattern: data struct +
//! state enum + token-resolved colors + AccessKit Node builder + a
//! `paint_X` helper colocated in the widget file (not paint.rs —
//! keeps paint.rs as the dispatch layer for compound widgets like
//! FloatingPanel and ToastQueue).

mod avatar;
mod button;
mod checkbox;
mod color_swatch;
mod divider;
mod number_input;
mod progress_bar;
mod radio_group;
mod slider;
mod spinner;
mod tag;
mod text_area;
mod text_input;
mod toggle;

pub use avatar::{Avatar, AvatarShape, AvatarState, paint_avatar};
pub use button::{Button, ButtonKind, ButtonState, ICON_BUTTON_SIZE_PX, paint_button};
pub use checkbox::{CHECKBOX_BOX_PX, Checkbox, CheckboxState, CheckboxValue, paint_checkbox};
pub use color_swatch::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};
pub use divider::{Divider, DividerOrientation, paint_divider};
pub use number_input::{NumberInput, paint_number_input};
pub use progress_bar::{ProgressBar, ProgressMode, paint_progress_bar};
pub use radio_group::{RadioGroup, RadioOption, RadioOrientation, paint_radio_group};
pub use slider::{Slider, SliderOrientation, SliderState, paint_slider};
pub use spinner::{Spinner, paint_spinner};
pub use tag::{Tag, TagState, TagTone, paint_tag};
pub use text_area::{TextArea, min_height as text_area_min_height, paint_text_area};
pub use text_input::{TextInput, TextInputState, paint_text_input};
pub use toggle::{Toggle, ToggleState, paint_toggle};
