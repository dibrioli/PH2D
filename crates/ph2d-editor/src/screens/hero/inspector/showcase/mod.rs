//! Dead-code Widget Gallery showcase painters.
//!
//! Extracted from [`super`] (Track C5). These are the demo / reference
//! section painters that powered the original Inspector body before
//! the M14 active sections (Name / Visibility / Transform / Render
//! Source) replaced them. They now live in the floating Widget Gallery
//! panel (toggled via the palette pill in the TopBar) and remain the
//! canonical visual reference for every primitive widget — peripheral
//! agents copy section layout / state-machine wiring from here.
//!
//! The `#![allow(dead_code)]` at the inspector module level covers
//! these.

use super::super::style::{
    PANEL_HEAD_PAD, paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect,
    panel_resize_handle_rect,
};
use super::ids;
use super::state::{
    LAST_BODY_TOP_SCREEN_Y, LAST_SECTION_TOPS_Y, push_section_top_y, set_last_gallery_content_h,
    set_last_gallery_visible_h, set_pending_dropdown_chip, take_pending_dropdown_chip,
};
use super::{
    SECTION_IDS, TREE_LEAF_IDS, active_index, paint_one_note, paint_section_separator,
    read_combobox, read_number_input, read_text_input,
};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, NoteData, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_title, rect_to_vello, resolve};
use crate::widget::Dropdown;
use crate::widget::DropdownOption;
use crate::widget::{
    Avatar, AvatarShape, Button, ButtonKind, ButtonState, Card, Checkbox, CheckboxState,
    CheckboxValue, ColorSwatch, Combobox, ComboboxOption, DropdownState, ListItem, ListItemState,
    NumberInput, ProgressBar, RadioGroup, RadioOption, RadioOrientation, SectionHeader,
    SliderState, Spinner, SwatchSize, TabItem, Tabs, TabsVariant, Tag, TagState, TagTone, TextArea,
    TextInput, Toggle, ToggleState, TreeNode, TreeView, Vector3Editor, paint_avatar, paint_button,
    paint_card, paint_checkbox, paint_color_swatch, paint_combobox_with_state, paint_list_item,
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

pub(super) const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: showcase body inset (between Spacing::Md and Lg; chrome-specific)
pub(super) const ROW_GAP: f32 = Spacing::Sm.px();
pub(super) const SECTION_HEAD_H: f32 = ROW_H_PX;
pub(super) const FIELD_H: f32 = Spacing::Xl3.px();
// ── Helpers ────────────────────────────────────────────────────────────────

/// Paint a collapsible section header at `(x, y)` and register its
/// click hit. Returns `(next_y, is_open)` where `is_open` controls
/// whether the caller paints the section body or skips ahead.
///
/// Single source of truth for every section in the Inspector — the
/// chevron direction, hit rect, and collapsed-flag read all live
/// here so individual section painters stay focused on their content.
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
    // Color circle replaces the legacy count chip. Default to a
    // neutral mid-gray when the user hasn't picked a color yet.
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xFF]);
    let header = SectionHeader::new(id, label)
        .color(rgba)
        .collapsible(!is_collapsed);
    paint_section_header(&header, r, scene, text_system, theme);
    // Register the circle hit zone AFTER the header rect so the
    // circle "wins" the back-to-front HitIndex lookup. Clicking the
    // circle opens the picker for this section; clicking elsewhere
    // on the header toggles collapse.
    if let Some(circle_rect) = crate::widget::color_circle_hit_rect(&header, r) {
        hit_index.register(color_id, circle_rect);
    }
    (y + SECTION_HEAD_H + Spacing::Xs.px(), !is_collapsed)
}

/// Discreet colored separator painted at the end of each section's
/// content (when expanded). 1 px tall, almost full-width (2 px
/// horizontal inset only — line spans the panel body edge to edge).
/// Balanced vertical padding (`SEPARATOR_PAD_Y` above + below) so
/// the line sits centered in its own gap, not lopsided against the
/// content above. Owns the entire inter-section spacing — callers
/// should NOT add `SECTION_GAP` on top.
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
pub(in crate::screens::hero) use body::paint_showcase_body;
