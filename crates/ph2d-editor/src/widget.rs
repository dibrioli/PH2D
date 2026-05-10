//! Widget primitives. Each follows the Button pattern: data struct +
//! state enum + token-resolved colors + AccessKit Node builder + a
//! `paint_X` helper colocated in the widget file (not paint.rs —
//! keeps paint.rs as the dispatch layer for compound widgets like
//! FloatingPanel and ToastQueue).

mod avatar;
mod button;
mod card;
mod checkbox;
mod color_swatch;
mod combobox;
mod context_menu;
mod divider;
mod dropdown;
mod list_item;
mod number_input;
mod progress_bar;
mod radio_group;
mod slider;
mod spinner;
mod tabs;
mod tag;
mod text_area;
mod text_input;
mod toggle;
mod tooltip;
mod vector3_editor;

pub use avatar::{Avatar, AvatarShape, AvatarState, paint_avatar};
pub use button::{Button, ButtonKind, ButtonState, ICON_BUTTON_SIZE_PX, paint_button};
pub use card::{Card, paint_card};
pub use checkbox::{CHECKBOX_BOX_PX, Checkbox, CheckboxState, CheckboxValue, paint_checkbox};
pub use color_swatch::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};
pub use combobox::{Combobox, ComboboxOption, ComboboxState, paint_combobox};
pub use context_menu::{ContextMenu, ContextMenuEntry, paint_context_menu};
pub use divider::{Divider, DividerOrientation, paint_divider};
pub use dropdown::{Dropdown, DropdownOption, DropdownState, paint_dropdown};
pub use list_item::{ListItem, ListItemState, paint_list_item};
pub use number_input::{NumberInput, paint_number_input};
pub use progress_bar::{ProgressBar, ProgressMode, paint_progress_bar};
pub use radio_group::{RadioGroup, RadioOption, RadioOrientation, paint_radio_group};
pub use slider::{Slider, SliderOrientation, SliderState, paint_slider};
pub use spinner::{Spinner, paint_spinner};
pub use tabs::{TabItem, Tabs, TabsVariant, paint_tabs};
pub use tag::{Tag, TagState, TagTone, paint_tag};
pub use text_area::{TextArea, min_height as text_area_min_height, paint_text_area};
pub use text_input::{TextInput, TextInputState, paint_text_input};
pub use toggle::{Toggle, ToggleState, paint_toggle};
pub use tooltip::{Tooltip, paint_tooltip};
pub use vector3_editor::{Vector3Editor, paint_vector3_editor};
