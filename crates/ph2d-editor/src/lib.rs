#![forbid(unsafe_code)]
//! ph2d-editor — Procreate-style canvas-first editor (M12, ADR-0023).
//!
//! Foundation:
//! - [`zones::Layout`] — 4-zone canonical positioning (top-left
//!   creates / top-right edits / sidebar modulates / center 100 %
//!   canvas). Per ADR-0023 §3.
//! - [`floating_panel::FloatingPanel`] — Procreate-style draggable
//!   tool drawer primitive. Per ADR-0023 §5.
//! - [`widget`] — primitives (Button, Slider, Toggle, RadioGroup,
//!   ColorSwatch). Each follows the same pattern: data + state enum
//!   + tokens + a11y::Node + colocated `paint_X` helper.
//! - [`tool::Tool`] + [`tool::ToolRegistry`] — canonical contract
//!   every editor tool implements (id / label / icon / build_panel
//!   / activate hooks). Registry tracks the active tool.
//! - [`tools`] — seed implementations ([`tools::BrushTool`],
//!   [`tools::MoveTool`]) proving the trait shape.
//! - [`zen::ZenMode`] — Tab-toggle workspace state. Per ADR-0023 §2.
//! - [`toast::ToastQueue`] — non-modal notification stream. Per
//!   ADR-0023 §2 ("Notificações flutuantes não-modais").
//! - [`paint`] — Vello lowering (`Paint` trait + `paint_text`
//!   helper). M11 widget paint pass + this PR's text rendering.
//!
//! Out of scope (M13+):
//! - QuickMenu radial (ADR-0023 §6)
//! - Gesture-mapping editor UI (ADR-0023 §4)
//! - Single-Touch Companion overlay

pub mod floating_panel;
pub mod gizmo;
pub mod grid;
pub mod grid_snap;
pub mod icons;
pub mod image_edit;
pub mod interaction;
pub mod paint;
pub mod project;
pub mod screens;
pub mod toast;
pub mod tool;
pub mod tools;
pub mod widget;
pub mod zen;
pub mod zones;

pub use floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelControl, PanelTab, ToolId};
pub use gizmo::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoView,
    TransformSnapshot, compute_gizmo_transform, gizmo_kind_for_id, is_gizmo_handle_id,
    paint_sprite_gizmo,
};
pub use grid::{GridConfig, GridLineCounts, GridView, count_visible_lines, paint_grid};
pub use icons::{IconCmd, IconId, cmd_to_path};
pub use image_edit::{PixelBounds, recenter_after_crop};
pub use interaction::{
    HitIndex, InteractiveState, WidgetEvent, WidgetStore, dispatch_key, dispatch_pointer,
    dispatch_text_input, dispatch_tick,
};
pub use paint::{
    Paint, PaintCtx, fill_rounded_rect, paint_icon, paint_text, paint_text_centered,
    paint_text_title, paint_tool_palette_icons, resolve, stroke_rect, stroke_rounded_rect,
};
pub use project::{
    DEFAULT_PIXELS_PER_METER, MAX_PIXELS_PER_METER, MIN_PIXELS_PER_METER, ProjectSettings,
};
pub use screens::{
    BottomHudStats, HeroScreen, HeroSelection, InspectorNameInfo, InspectorSpriteInfo,
    InspectorSpriteSource, InspectorTransformInfo, InspectorVisibilityInfo,
    RequestedSpriteStrategy, ViewFocusKind, paint_hero_screen, set_live_component_count,
};
pub use toast::{Toast, ToastQueue, ToastSeverity};
pub use tool::{PanelEvent, Tool, ToolRegistry};
// Re-export so the shell can name the dragging node id without
// taking a direct ph2d-a11y dep just for one type.
pub use ph2d_a11y::NodeId;
pub use tools::{
    Bounds, BrushTool, MakeSquareResult, MoveTool, TrimResult, crop_bezpath, make_square,
    square_bezpath, trim_transparency,
};
pub use widget::{
    Avatar, AvatarShape, AvatarState, Button, ButtonKind, ButtonState, CHECKBOX_BOX_PX, Card,
    Checkbox, CheckboxState, CheckboxValue, ColorPicker, ColorPickerMode, ColorSwatch, Combobox,
    ComboboxOption, ComboboxState, ContextMenu, ContextMenuEntry, Divider, DividerOrientation,
    Dropdown, DropdownOption, DropdownState, ICON_BUTTON_SIZE_PX, ListItem, ListItemState, Modal,
    NumberInput, Popover, ProgressBar, ProgressMode, RadioGroup, RadioOption, RadioOrientation,
    Slider, SliderOrientation, SliderState, Spinner, SwatchSize, SwatchState, TabItem, Tabs,
    TabsVariant, Tag, TagState, TagTone, TextArea, TextInput, TextInputState, Toggle, ToggleState,
    Tooltip, TreeNode, TreeView, Vector3Editor, paint_avatar, paint_button, paint_card,
    paint_checkbox, paint_color_picker, paint_color_swatch, paint_combobox, paint_context_menu,
    paint_divider, paint_dropdown, paint_list_item, paint_modal, paint_number_input, paint_popover,
    paint_progress_bar, paint_radio_group, paint_slider, paint_spinner, paint_tabs, paint_tag,
    paint_text_area, paint_text_input, paint_toggle, paint_tooltip, paint_tree_view,
    paint_vector3_editor,
};
pub use zen::ZenMode;
pub use zones::{Layout, Zone};
