//! Widget Gallery showcase — the canonical visual reference for
//! every primitive widget. Peripheral agents copy section layout +
//! state-machine wiring from here.
//!
//! Wave 8 Phase 2.A moved the showcase tree from
//! `ph2d_editor::screens::hero::inspector::showcase` to editor-core
//! so the `ph2d-panel-widget-gallery` panel crate can consume
//! `paint_showcase_body` + helpers without depending on `ph2d-editor`
//! (audit S1 + A3 closure).

#![allow(dead_code)]

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{HitIndex, InteractiveState, NoteData, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_title, rect_to_vello, resolve};
use crate::widget::panel_chrome::{
    HIGHLIGHTER_RGBA, PANEL_HEAD_PAD, paint_panel_corner_dot, paint_panel_surface,
    panel_drag_handle_rect, panel_resize_handle_rect,
};
use crate::widget::{
    Avatar, AvatarShape, Button, ButtonKind, ButtonState, Card, Checkbox, CheckboxState,
    CheckboxValue, ColorSwatch, Combobox, ComboboxOption, ComboboxState, Dropdown, DropdownOption,
    DropdownState, ListItem, ListItemState, NumberInput, ProgressBar, RadioGroup, RadioOption,
    RadioOrientation, SectionHeader, SliderState, Spinner, SwatchSize, TabItem, Tabs, TabsVariant,
    Tag, TagState, TagTone, TextArea, TextInput, TextInputState, Toggle, ToggleState, TreeNode,
    TreeView, Vector3Editor, color_circle_hit_rect, paint_avatar, paint_button, paint_card,
    paint_checkbox, paint_color_swatch, paint_combobox_with_state, paint_list_item,
    paint_number_input_with_buffer, paint_progress_bar, paint_radio_group_with_labels,
    paint_section_header, paint_slider_with_chip, paint_spinner, paint_tabs, paint_tag,
    paint_text_area_with_state, paint_text_input_with_buffer, paint_toggle, paint_tree_view,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, Density, ICON_BTN_SIZE_PX, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

pub mod state;
pub use state::{
    LAST_BODY_TOP_SCREEN_Y, LAST_GALLERY_CONTENT_H, LAST_GALLERY_VISIBLE_H, LAST_SECTION_TOPS_Y,
    PENDING_DROPDOWN_CHIP, last_body_top_screen_y, last_gallery_content_h, last_gallery_visible_h,
    push_section_top_y, section_index_below_body_y, set_last_gallery_content_h,
    set_last_gallery_visible_h, set_pending_dropdown_chip, take_pending_dropdown_chip,
};

mod notes;
pub use notes::paint_one_note;

pub use crate::ids::SECTION_IDS;

/// TreeView leaf NodeIds — the two leaves under the Lists section's
/// "Layers" tree.
pub const TREE_LEAF_IDS: [NodeId; 2] = [ids::INSP_SAMPLE_TREE_LEAF_A, ids::INSP_SAMPLE_TREE_LEAF_B];

/// Hit-slot NodeIds for the 12 possible notes per panel.
pub const NOTE_SLOT_IDS: [NodeId; 12] = [
    ids::INSP_NOTE_SLOT_0,
    ids::INSP_NOTE_SLOT_1,
    ids::INSP_NOTE_SLOT_2,
    ids::INSP_NOTE_SLOT_3,
    ids::INSP_NOTE_SLOT_4,
    ids::INSP_NOTE_SLOT_5,
    ids::INSP_NOTE_SLOT_6,
    ids::INSP_NOTE_SLOT_7,
    ids::INSP_NOTE_SLOT_8,
    ids::INSP_NOTE_SLOT_9,
    ids::INSP_NOTE_SLOT_10,
    ids::INSP_NOTE_SLOT_11,
];
pub const NOTE_TITLE_IDS: [NodeId; 12] = [
    ids::INSP_NOTE_TITLE_0,
    ids::INSP_NOTE_TITLE_1,
    ids::INSP_NOTE_TITLE_2,
    ids::INSP_NOTE_TITLE_3,
    ids::INSP_NOTE_TITLE_4,
    ids::INSP_NOTE_TITLE_5,
    ids::INSP_NOTE_TITLE_6,
    ids::INSP_NOTE_TITLE_7,
    ids::INSP_NOTE_TITLE_8,
    ids::INSP_NOTE_TITLE_9,
    ids::INSP_NOTE_TITLE_10,
    ids::INSP_NOTE_TITLE_11,
];
pub const NOTE_BODY_IDS: [NodeId; 12] = [
    ids::INSP_NOTE_BODY_0,
    ids::INSP_NOTE_BODY_1,
    ids::INSP_NOTE_BODY_2,
    ids::INSP_NOTE_BODY_3,
    ids::INSP_NOTE_BODY_4,
    ids::INSP_NOTE_BODY_5,
    ids::INSP_NOTE_BODY_6,
    ids::INSP_NOTE_BODY_7,
    ids::INSP_NOTE_BODY_8,
    ids::INSP_NOTE_BODY_9,
    ids::INSP_NOTE_BODY_10,
    ids::INSP_NOTE_BODY_11,
];

/// Color-circle hit NodeIds — one per section header, in the same
/// order as `SECTION_IDS`.
pub const SECTION_COLOR_IDS: [NodeId; 10] = [
    ids::INSP_SECTION_INPUTS_COLOR,
    ids::INSP_SECTION_SLIDER_COLOR,
    ids::INSP_SECTION_SWITCHES_COLOR,
    ids::INSP_SECTION_LISTS_COLOR,
    ids::INSP_SECTION_VECTOR_COLOR,
    ids::INSP_SECTION_STATUS_COLOR,
    ids::INSP_SECTION_COLOR_COLOR,
    ids::INSP_SECTION_ACTIONS_COLOR,
    ids::INSP_SECTION_IDENTITY_COLOR,
    ids::INSP_SECTION_CARD_COLOR,
];

pub const RADIO_GROUP_IDS: [NodeId; 3] = [
    ids::INSP_SAMPLE_RADIO_A,
    ids::INSP_SAMPLE_RADIO_B,
    ids::INSP_SAMPLE_RADIO_C,
];
pub const TAB_GROUP_IDS: [NodeId; 3] = [
    ids::INSP_SAMPLE_TAB_A,
    ids::INSP_SAMPLE_TAB_B,
    ids::INSP_SAMPLE_TAB_C,
];

pub(super) const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: showcase body inset
pub(super) const ROW_GAP: f32 = Spacing::Sm.px();
pub(super) const SECTION_HEAD_H: f32 = ROW_H_PX;
pub(super) const FIELD_H: f32 = Spacing::Xl3.px();
const SEPARATOR_PAD_Y: f32 = Spacing::Md.px();

// ── Helpers (formerly inspector::mod.rs) ───────────────────────────────────

/// Paint a collapsible section header at `(x, y)` and register its
/// click hit. Returns `(next_y, is_open)`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_collapsible_header(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    color_id: NodeId,
    label: &str,
    _count: u32,
) -> (f32, bool) {
    let r = Rect::new(x, y, w, SECTION_HEAD_H);
    let is_collapsed = store.is_collapsed(id);
    hit_index.register(id, r);
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xFF]);
    let header = SectionHeader::new(id, label)
        .color(rgba)
        .collapsible(!is_collapsed);
    paint_section_header(&header, r, scene, text_system, theme);
    if let Some(circle_rect) = color_circle_hit_rect(&header, r) {
        hit_index.register(color_id, circle_rect);
    }
    (y + SECTION_HEAD_H + Spacing::Xs.px(), !is_collapsed)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_left_label(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    label: &str,
    label_w: f32,
    y: f32,
    row_h: f32,
) {
    let label_y = y + (row_h - TypeToken::Xs.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        label,
        x,
        label_y,
        TypeToken::Xs.px(),
        label_w,
        resolve(ColorToken::Text2, theme),
    );
}

/// Discreet colored separator painted at the end of each section's
/// content. 1 px tall, almost full-width.
pub fn paint_section_separator(
    scene: &mut VectorScene,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let pad_x = 2.0_f32;
    let pad_y = SEPARATOR_PAD_Y;
    let thickness = 1.0_f32;
    let line = Rect::new(x + pad_x, y + pad_y, (w - pad_x * 2.0).max(0.0), thickness);
    fill_rounded_rect(scene, line, 0.5, resolve(ColorToken::Accent, theme));
    y + pad_y + thickness + pad_y
}

#[allow(clippy::too_many_arguments)]
pub fn read_text_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, &str, usize, Option<usize>) {
    match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.as_str(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, "", 0, None),
    }
}

pub fn read_combobox(
    store: &WidgetStore,
    id: NodeId,
) -> (ComboboxState, bool, &str, usize, Option<usize>) {
    match store.get(id) {
        Some(InteractiveState::Combobox {
            state,
            open,
            query,
            caret,
            selection_anchor,
        }) => (*state, *open, query.as_str(), *caret, *selection_anchor),
        _ => (ComboboxState::Normal, false, "", 0, None),
    }
}

pub fn read_number_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

/// Find which id in `ids` is the active (`Pressed`) one. Used for
/// segmented button groups (Tabs, RadioGroup) that model selection
/// as N independent Button states with exactly one in the Pressed
/// state.
pub fn active_index(store: &WidgetStore, ids: &[NodeId]) -> Option<usize> {
    for (i, id) in ids.iter().enumerate() {
        if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
            return Some(i);
        }
    }
    None
}

mod actions;
mod card;
mod color;
mod identity;
mod inputs;
mod lists;
mod slider;
mod status;
mod switches;
mod vector;

use actions::paint_actions_section;
use card::paint_card_section;
use color::paint_color_section;
use identity::paint_identity_section;
use inputs::paint_inputs_section;
use lists::paint_lists_section;
use slider::paint_slider_section;
use status::paint_status_section;
use switches::paint_switches_section;
use vector::paint_vector_section;

mod body;
pub use body::paint_showcase_body;
