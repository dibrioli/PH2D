//! Widget Gallery showcase — the canonical visual reference for
//! every primitive widget. Peripheral agents copy section layout +
//! state-machine wiring from here.
//!
//! Wave 8 Phase 2.A moved the showcase tree from
//! `ph2d_editor::screens::hero::inspector::showcase` to editor-core
//! so the `ph2d-panel-widget-gallery` panel crate can consume
//! `paint_showcase_body` + helpers without depending on `ph2d-editor`
//! (audit S1 + A3 closure).
//!
//! Wave 8 Phase 3 (audit A1): `#[doc(hidden)]` — items are pub for
//! panel crates but excluded from rustdoc / unstable across 0.x.y.

#![allow(dead_code)]
#![doc(hidden)]

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{HitIndex, InteractiveState, NoteData, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve};
use crate::widget::panel_chrome::{
    HIGHLIGHTER_RGBA, PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
};
use crate::widget::{
    Avatar, AvatarShape, Button, ButtonKind, ButtonState, Card, Checkbox, CheckboxState,
    CheckboxValue, ColorSwatch, Combobox, ComboboxOption, ComboboxState, Dropdown, DropdownOption,
    DropdownState, ListItem, ListItemState, NumberInput, ProgressBar, RadioGroup, RadioOption,
    RadioOrientation, SectionHeader, SliderState, Spinner, SwatchSize, TabItem, Tabs, TabsVariant,
    Tag, TagState, TagTone, TextArea, TextInput, TextInputState, Toggle, ToggleState, TreeNode,
    TreeView, color_circle_hit_rect, paint_avatar, paint_button, paint_card,
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
    // Canon (2026-05-24): all field labels follow the Grid Settings
    // pattern — Base font (13 px) + Text1 + left-align via
    // `paint_text`. Was Xs/Sm at various points; user pinned it on
    // Grid Settings as the source of truth so this stays in sync.
    let font = TypeToken::Base.px();
    let label_y = y + (row_h - font) * 0.5;
    paint_text(
        text_system,
        scene,
        label,
        x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text1, theme),
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

/// ADR-0029 Phase C.1: host-level event router for showcase-shared
/// clicks (section header collapse, radio/tab/tree leaf pinning,
/// section outline / note background color picks, "Create note"
/// context-menu item, color-circle picker open). Previously lived
/// in `inspector::apply_event` (editor-core) where it served BOTH
/// the live Inspector AND the Widget Gallery — both panels reuse
/// the same `INSP_SAMPLE_*` showcase widgets + the global
/// `CTX_MENU_OUTLINE_*` / `CTX_MENU_CREATE_NOTE` ids. After
/// Inspector migrated to its own crate, this logic became
/// host-level so the Gallery (still on the legacy registry) keeps
/// working without depending on the typed Inspector being
/// installed.
///
/// Returns `true` when `event` was consumed (host should stop
/// further dispatch).
pub fn apply_showcase_event(
    store: &mut WidgetStore,
    event: crate::interaction::WidgetEvent,
) -> bool {
    use crate::interaction::{ContextMenuKind, InteractiveState, WidgetEvent};
    if let WidgetEvent::Click(id) = event {
        const OUTLINE_ITEMS: [(NodeId, Option<u8>); 6] = [
            (crate::ids::CTX_MENU_OUTLINE_NONE, None),
            (crate::ids::CTX_MENU_OUTLINE_0, Some(0)),
            (crate::ids::CTX_MENU_OUTLINE_1, Some(1)),
            (crate::ids::CTX_MENU_OUTLINE_2, Some(2)),
            (crate::ids::CTX_MENU_OUTLINE_3, Some(3)),
            (crate::ids::CTX_MENU_OUTLINE_4, Some(4)),
        ];
        if id == crate::ids::CTX_MENU_CREATE_NOTE {
            if let Some(req) = store.consume_last_context_menu()
                && let ContextMenuKind::CreateNote { panel, .. } = req.kind
            {
                let scroll_y = store.panel_scroll(panel);
                let body_top_screen = last_body_top_screen_y();
                let body_y = req.y - body_top_screen + scroll_y;
                let before = section_index_below_body_y(body_y);
                let new_index = store.notes_for_panel(panel).len();
                store.notes_push(panel, 0, before);
                if let Some(title_id) = NOTE_TITLE_IDS.get(new_index)
                    && let Some(InteractiveState::TextInput { text, caret, .. }) =
                        store.get_mut(*title_id)
                {
                    *text = format!("Note {}", new_index + 1);
                    *caret = text.len();
                }
                if let Some(body_id) = NOTE_BODY_IDS.get(new_index)
                    && let Some(InteractiveState::TextInput { text, caret, .. }) =
                        store.get_mut(*body_id)
                {
                    text.clear();
                    *caret = 0;
                }
            }
            return true;
        }
        if let Some((_, color_idx)) = OUTLINE_ITEMS.iter().find(|(item_id, _)| *item_id == id) {
            if let Some(req) = store.consume_last_context_menu() {
                match req.kind {
                    ContextMenuKind::SectionOutline { section } => {
                        store.set_section_outline_color(section, *color_idx);
                    }
                    ContextMenuKind::NoteBackground { panel, note_index } => {
                        if let Some(c) = color_idx {
                            store.note_set_color(panel, note_index as usize, *c);
                        }
                    }
                    _ => {}
                }
            }
            return true;
        }
        if SECTION_COLOR_IDS.contains(&id) || id == crate::ids::INSP_SAMPLE_SWATCH {
            let seed = store.widget_color(id).unwrap_or([0x88, 0x88, 0x88, 0xFF]);
            store.set_widget_color(id, seed);
            store.set_picker_target(Some(id));
            store.set_blender_value(
                crate::ids::INSP_BLENDER_PICKER,
                ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
            );
            return true;
        }
        if crate::ids::SECTION_IDS.contains(&id) {
            store.toggle_collapsed(id);
            return true;
        }
        if id == crate::ids::INSP_SAMPLE_TREE_ROOT {
            store.toggle_collapsed(id);
            return true;
        }
        if TREE_LEAF_IDS.contains(&id) {
            pin_showcase_button_selection(store, id, &TREE_LEAF_IDS);
            return true;
        }
        if RADIO_GROUP_IDS.contains(&id) {
            pin_showcase_button_selection(store, id, &RADIO_GROUP_IDS);
            return true;
        }
        if TAB_GROUP_IDS.contains(&id) {
            pin_showcase_button_selection(store, id, &TAB_GROUP_IDS);
            return true;
        }
    }
    false
}

fn pin_showcase_button_selection(store: &mut WidgetStore, selected: NodeId, group: &[NodeId]) {
    for id in group {
        if let Some(crate::interaction::InteractiveState::Button { state }) = store.get_mut(*id) {
            *state = if *id == selected {
                crate::widget::ButtonState::Pressed
            } else {
                crate::widget::ButtonState::Normal
            };
        }
    }
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
