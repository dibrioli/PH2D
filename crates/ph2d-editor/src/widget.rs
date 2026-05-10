//! Widget primitives. Each follows the Button pattern: data struct +
//! state enum + token-resolved colors + AccessKit Node builder + a
//! `paint_X` helper colocated in the widget file (not paint.rs —
//! keeps paint.rs as the dispatch layer for compound widgets like
//! FloatingPanel and ToastQueue).

mod avatar;
mod blender_color_picker;
mod button;
mod card;
mod checkbox;
mod color_picker;
mod color_swatch;
mod combobox;
mod context_menu;
mod divider;
mod dropdown;
mod list_item;
mod modal;
mod number_input;
mod pill_group;
mod popover;
mod progress_bar;
mod radio_group;
mod section_header;
mod slider;
mod slider_with_chip;
mod spinner;
mod status_bar;
mod tabs;
mod tag;
mod text_area;
mod text_input;
mod toggle;
mod tool_rail;
mod tooltip;
mod tree_view;
mod vector3_editor;

pub use avatar::{Avatar, AvatarShape, AvatarState, paint_avatar};
pub use blender_color_picker::{
    BlenderColorPicker, BlenderSubIds, ChannelMode, ColorPalette, InterpolationMode,
    apply_blender_value_pick, apply_blender_wheel_pick, default_palette, hsv_to_rgba8,
    paint_blender_color_picker, paint_blender_color_picker_with_store,
    paint_blender_color_picker_with_store_compat, parse_hex, rgba_to_hsv, value_pick, wheel_pick,
};
pub use button::{Button, ButtonKind, ButtonState, ICON_BUTTON_SIZE_PX, paint_button};
pub use card::{Card, paint_card};
pub use checkbox::{CHECKBOX_BOX_PX, Checkbox, CheckboxState, CheckboxValue, paint_checkbox};
pub use color_picker::{ColorPicker, ColorPickerMode, paint_color_picker};
pub use color_swatch::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};
pub use combobox::{Combobox, ComboboxOption, ComboboxState, paint_combobox};
pub use context_menu::{ContextMenu, ContextMenuEntry, paint_context_menu};
pub use divider::{Divider, DividerOrientation, paint_divider};
pub use dropdown::{Dropdown, DropdownOption, DropdownState, paint_dropdown};
pub use list_item::{ListItem, ListItemState, paint_list_item};
pub use modal::{Modal, paint_modal};
pub use number_input::{NumberInput, paint_number_input, paint_number_input_with_buffer};
pub use pill_group::{PILL_PADDING_PX, PillGroup, paint_pill_group};
pub use popover::{Popover, paint_popover};
pub use progress_bar::{ProgressBar, ProgressMode, paint_progress_bar};
pub use radio_group::{
    RadioGroup, RadioOption, RadioOrientation, paint_radio_group, paint_radio_group_with_labels,
};
pub use section_header::{SectionHeader, paint_section_header};
pub use slider::{Slider, SliderOrientation, SliderState, paint_slider};
pub use slider_with_chip::{
    DEFAULT_CHIP_W, DEFAULT_LABEL_W, paint_number_chip, paint_slider_with_chip,
    paint_slider_with_chip_layout,
};
pub use spinner::{Spinner, paint_spinner};
pub use status_bar::{SegmentTone, StatusBar, StatusSegment, paint_status_bar};
pub use tabs::{TabItem, Tabs, TabsVariant, paint_tabs};
pub use tag::{Tag, TagState, TagTone, paint_tag};
pub use text_area::{TextArea, min_height as text_area_min_height, paint_text_area};
pub use text_input::{TextInput, TextInputState, paint_text_input, paint_text_input_with_buffer};
pub use toggle::{Toggle, ToggleState, paint_toggle};
pub use tool_rail::{
    COMPOUND_TOTAL_H_PX, DIVIDER_GAP_PX, TOOL_CHIP_PX, TOOL_RAIL_WIDTH_PX, ToolRail, ToolRailEntry,
    paint_tool_rail,
};
pub use tooltip::{Tooltip, paint_tooltip};
pub use tree_view::{TreeNode, TreeView, paint_tree_view};
pub use vector3_editor::{Vector3Editor, paint_vector3_editor};
