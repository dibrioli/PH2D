//! Inspector painter.
//!
//! Header: panel title "Inspector" + a sub-row showing the
//! currently-selected entity's name (or "(none)").
//!
//! Body: placeholder until the pilot project wires real per-component
//! editors. The 10-section widget showcase that used to live here
//! (Inputs / Slider / Switches / Lists / Vector / Status / Color /
//! Actions / Identity / Card + notes) now lives in the floating
//! Widget Gallery panel (toggle via the palette pill in the TopBar);
//! the section painters are reached from [`paint_showcase_body`].
//! The `#[allow(dead_code)]` at file scope covers helpers that
//! aren't directly called by the live `paint_inspector` body but
//! ARE used by `paint_showcase_body`.
//!
//! The floating [`crate::widget::BlenderColorPicker`] (handled by
//! [`super::color_picker_demo`]) is not duplicated here — its
//! retained state still lives on this panel under
//! [`ids::INSP_BLENDER_PICKER`], which is registered in
//! [`populate`].
#![allow(dead_code)]

use super::HeroLayout;
use super::HeroSelection;
use super::ids;
use super::style::{
    PANEL_HEAD_PAD, paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect,
    panel_resize_handle_rect,
};
use super::{InspectorSpriteInfo, InspectorSpriteSource};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, NoteData, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_title, rect_to_vello, resolve};
use crate::widget::Dropdown;
use crate::widget::DropdownOption;
use crate::widget::{
    Avatar, AvatarShape, Button, ButtonKind, ButtonState, Card, ChannelMode, Checkbox,
    CheckboxState, CheckboxValue, ColorSwatch, Combobox, ComboboxOption, ComboboxState,
    DropdownState, InterpolationMode, ListItem, ListItemState, NumberInput, ProgressBar,
    RadioGroup, RadioOption, RadioOrientation, SectionHeader, SliderOrientation, SliderState,
    Spinner, SwatchSize, TabItem, Tabs, TabsVariant, Tag, TagState, TagTone, TextArea, TextInput,
    TextInputState, Toggle, ToggleState, TreeNode, TreeView, Vector3Editor, paint_avatar,
    paint_button, paint_card, paint_checkbox, paint_color_swatch, paint_combobox_with_state,
    paint_list_item, paint_number_input_with_buffer, paint_progress_bar,
    paint_radio_group_with_labels, paint_section_header, paint_slider_with_chip, paint_spinner,
    paint_tabs, paint_tag, paint_text_area_with_state, paint_text_input_with_buffer, paint_toggle,
    paint_tree_view,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const BODY_PAD: f32 = 10.0;
const ROW_GAP: f32 = 6.0;
const SECTION_HEAD_H: f32 = 28.0;
const FIELD_H: f32 = 32.0;

/// Stable id list for every collapsible section header in the
/// Inspector. Order matches `paint_inspector` paint order so the
/// `apply_event` lookup and `populate` registration walk the same
/// sequence.
const SECTION_IDS: [ph2d_a11y::NodeId; 10] = [
    ids::INSP_SECTION_INPUTS,
    ids::INSP_SECTION_SLIDER,
    ids::INSP_SECTION_SWITCHES,
    ids::INSP_SECTION_LISTS,
    ids::INSP_SECTION_VECTOR,
    ids::INSP_SECTION_STATUS,
    ids::INSP_SECTION_COLOR,
    ids::INSP_SECTION_ACTIONS,
    ids::INSP_SECTION_IDENTITY,
    ids::INSP_SECTION_CARD,
];

/// Color-circle hit ids — one per section header, in the same
/// order as `SECTION_IDS`. Clicking any of these opens the global
/// color picker editing `widget_colors[circle_id]`.
const SECTION_COLOR_IDS: [ph2d_a11y::NodeId; 10] = [
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

/// Ids of the three Radio buttons that form the Switches sample's
/// segmented "Low / Mid / High" group. Same trick used for the
/// "Edit / Play / Debug" tabs — exactly one button is `Pressed` at
/// a time, the painter reads the active index from that flag.
const RADIO_GROUP_IDS: [ph2d_a11y::NodeId; 3] = [
    ids::INSP_SAMPLE_RADIO_A,
    ids::INSP_SAMPLE_RADIO_B,
    ids::INSP_SAMPLE_RADIO_C,
];
const TAB_GROUP_IDS: [ph2d_a11y::NodeId; 3] = [
    ids::INSP_SAMPLE_TAB_A,
    ids::INSP_SAMPLE_TAB_B,
    ids::INSP_SAMPLE_TAB_C,
];
const TREE_LEAF_IDS: [ph2d_a11y::NodeId; 2] =
    [ids::INSP_SAMPLE_TREE_LEAF_A, ids::INSP_SAMPLE_TREE_LEAF_B];
/// Hit-slot ids for the 12 possible notes per panel. The painter
/// assigns slots by position (slot 0 = first painted note, etc.).
pub(super) const NOTE_SLOT_IDS: [ph2d_a11y::NodeId; 12] = [
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
pub(super) const NOTE_TITLE_IDS: [ph2d_a11y::NodeId; 12] = [
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
pub(super) const NOTE_BODY_IDS: [ph2d_a11y::NodeId; 12] = [
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

// Thread-local stash for the open Dropdown's chip rect, captured by
// `paint_lists_section` and consumed by `paint_inspector` AFTER the
// section loop so the open list paints above every other section
// (single dropdown per panel — see `docs/UI_Bugs/README.md` §9.16).
thread_local! {
    static PENDING_DROPDOWN_CHIP: std::cell::RefCell<Option<(usize, Rect)>> =
        const { std::cell::RefCell::new(None) };
    /// Content height measured during the previous paint pass. The
    /// wheel dispatch reads this via [`last_inspector_content_h`] to
    /// clamp `scroll_y` to `[0, content_h - visible_h]`. One frame of
    /// staleness is invisible since paint runs every frame.
    static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Exact visible body height (`content_bottom - content_top`) of
    /// the inspector's last paint. Used together with content_h to
    /// derive max_scroll. Bypasses the rough `panel.h - 60` heuristic
    /// which over-estimated visible_h and clamped the scroll too
    /// early — last few px of new notes weren't reachable.
    static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Body-relative top-Y of each painted section's header, indexed
    /// by section position in `SECTION_IDS`. The right-click dispatch
    /// uses this to pick which section a new note should be inserted
    /// ABOVE (the user's "nota deve ser inserida acima do objeto
    /// selecionado"). Body-relative so it stays stable across
    /// scroll offsets — the lookup converts the click's screen y
    /// into body-y via `event.y - body_top_screen + scroll_y`.
    static LAST_SECTION_TOPS_Y: std::cell::RefCell<Vec<f32>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Body-relative top-Y in screen coords reference, captured each
    /// frame so callers (the hero) can convert screen-y → body-y.
    static LAST_BODY_TOP_SCREEN_Y: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// M14.5 inspector phase (6.4/§9): live snapshot the host
    /// publishes each frame so `paint_inspector` can render the
    /// Render Source section + Reimport button without crossing the
    /// ADR-0021 / HR-8 boundary into SimWorld. `None` when nothing is
    /// selected or selection isn't a sprite.
    static CURRENT_INSPECTOR_SPRITE: std::cell::RefCell<Option<InspectorSpriteInfo>> =
        const { std::cell::RefCell::new(None) };
    /// M14.A: same shape as `CURRENT_INSPECTOR_SPRITE` for the live
    /// `Transform` editor section. `paint_hero_screen` publishes this
    /// before `paint_inspector` and clears it after so a stale
    /// snapshot can't leak into the next frame.
    static CURRENT_INSPECTOR_TRANSFORM: std::cell::RefCell<Option<super::InspectorTransformInfo>> =
        const { std::cell::RefCell::new(None) };
    /// M14.D: same pattern for the Visibility checkbox row. Held as
    /// a `Cell` (struct is `Copy`) so the painter's read is
    /// allocation-free.
    static CURRENT_INSPECTOR_VISIBILITY:
        std::cell::Cell<Option<super::InspectorVisibilityInfo>> =
        const { std::cell::Cell::new(None) };
    /// M14.E: editable entity-name snapshot. `RefCell` because the
    /// inner `InspectorNameInfo` carries an owned `String` (entity
    /// names can be longer than `Copy` is convenient for).
    static CURRENT_INSPECTOR_NAME: std::cell::RefCell<Option<super::InspectorNameInfo>> =
        const { std::cell::RefCell::new(None) };
    /// Mirror of `LAST_CONTENT_H` / `LAST_VISIBLE_H` for the floating
    /// Widget Gallery panel painted by [`paint_showcase_body`]. Tracked
    /// independently so the gallery and Inspector scroll without
    /// aliasing each other's clamp bound.
    static LAST_GALLERY_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    static LAST_GALLERY_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// Set the inspector sprite snapshot for the current paint. Hero
/// publishes this before `paint_inspector` runs and clears it after,
/// matching the [[hierarchy_live_entries]] thread-local pattern.
pub(super) fn set_current_inspector_sprite(info: Option<InspectorSpriteInfo>) {
    CURRENT_INSPECTOR_SPRITE.with(|c| *c.borrow_mut() = info);
}

fn current_inspector_sprite() -> Option<InspectorSpriteInfo> {
    CURRENT_INSPECTOR_SPRITE.with(|c| c.borrow().clone())
}

/// M14.A: paired with [`set_current_inspector_sprite`] for the
/// Transform live-binding section. `paint_hero_screen` is the only
/// publisher.
pub(super) fn set_current_inspector_transform(info: Option<super::InspectorTransformInfo>) {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow_mut() = info);
}

fn current_inspector_transform() -> Option<super::InspectorTransformInfo> {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow())
}

/// M14.D: same as `set_current_inspector_transform` for the
/// Visibility checkbox row.
pub(super) fn set_current_inspector_visibility(info: Option<super::InspectorVisibilityInfo>) {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.set(info));
}

fn current_inspector_visibility() -> Option<super::InspectorVisibilityInfo> {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.get())
}

/// M14.E: same shape for the editable entity-name field.
pub(super) fn set_current_inspector_name(info: Option<super::InspectorNameInfo>) {
    CURRENT_INSPECTOR_NAME.with(|c| *c.borrow_mut() = info);
}

/// Audit #2 fix (LOW, clone elision): presence check without cloning
/// the inner `String`. The painter reads the live `name` from the
/// store buffer (the host writes it during the selection-change reset
/// in `paint_hero_screen`); the snapshot is only consulted to decide
/// whether to paint the row at all.
fn current_inspector_name_is_some() -> bool {
    CURRENT_INSPECTOR_NAME.with(|c| c.borrow().is_some())
}

fn set_pending_dropdown_chip(chip: Option<(usize, Rect)>) {
    PENDING_DROPDOWN_CHIP.with(|c| *c.borrow_mut() = chip);
}

fn take_pending_dropdown_chip() -> Option<(usize, Rect)> {
    PENDING_DROPDOWN_CHIP.with(|c| c.borrow_mut().take())
}

/// Last-known total content height of the inspector body (sum of all
/// section heights + gaps). Used by `dispatch_wheel` to clamp the
/// scroll offset so the user can't scroll past the last element.
pub(super) fn last_inspector_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

fn set_last_inspector_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

pub(super) fn last_inspector_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

fn set_last_inspector_visible_h(h: f32) {
    LAST_VISIBLE_H.with(|c| c.set(h));
}

/// Gallery counterparts of `last_inspector_content_h` / `last_inspector_visible_h`.
/// Read by the host after [`paint_showcase_body`] to clamp the
/// wheel-scroll bound on `GAL_PANEL`.
pub(super) fn last_gallery_content_h() -> f32 {
    LAST_GALLERY_CONTENT_H.with(|c| c.get())
}

pub(super) fn last_gallery_visible_h() -> f32 {
    LAST_GALLERY_VISIBLE_H.with(|c| c.get())
}

fn set_last_gallery_content_h(h: f32) {
    LAST_GALLERY_CONTENT_H.with(|c| c.set(h));
}

fn set_last_gallery_visible_h(h: f32) {
    LAST_GALLERY_VISIBLE_H.with(|c| c.set(h));
}

/// Find the section index whose body the given body-relative y
/// lies INSIDE. Returns `Some(i)` so callers know a new note
/// should be inserted above `SECTION_IDS[i]` (i.e. above the
/// section the user right-clicked into). Returns `None` when y is
/// past the last section's content (note appends to the bottom).
///
/// Previous version returned "the first section whose top > y" —
/// which for clicks INSIDE section A returned the index of
/// section B, so the new note went BELOW A's separator instead of
/// above A's header (user reported "note created below the separator
/// of the section the right-click landed in").
pub(super) fn section_index_below_body_y(body_y: f32) -> Option<u8> {
    LAST_SECTION_TOPS_Y.with(|tops| {
        let tops = tops.borrow();
        // Walk pairs (top[i], top[i+1]); the click is "inside"
        // section i when top[i] <= y < top[i+1]. The last section
        // has no successor — clicks past its top fall through to
        // `None` (trailing note).
        for i in 0..tops.len() {
            let top = tops[i];
            let next = tops.get(i + 1).copied().unwrap_or(f32::INFINITY);
            if body_y >= top && body_y < next {
                return Some(i as u8);
            }
            // Click ABOVE the very first section's top → insert
            // before that section.
            if i == 0 && body_y < top {
                return Some(0);
            }
        }
        None
    })
}

pub(super) fn last_body_top_screen_y() -> f32 {
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.get())
}

fn push_section_top_y(tops: &mut Vec<f32>, body_y: f32) {
    tops.push(body_y);
}

/// Register every sample widget + the floating BlenderColorPicker's
/// retained state. Called once at screen construction time.
pub fn populate(store: &mut WidgetStore) {
    populate_blender_picker(store);
    populate_samples(store);
    populate_transform_editor(store);
    populate_visibility_editor(store);
    populate_render_strategy(store);
    populate_name_editor(store);
}

/// M14.E: register the editable entity-name TextInput. Buffer starts
/// empty; the host seeds it from `InspectorNameInfo` on the first
/// frame an entity is selected.
fn populate_name_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_ENTITY_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
}

/// M14.C: register the 3 segmented Strategy buttons in the Render
/// Source section. Default `Normal`; the painter re-pins the
/// matching button to `Pressed` each frame from the snapshot, so the
/// stored state only matters for hover/idle transitions on inactive
/// options.
fn populate_render_strategy(store: &mut WidgetStore) {
    for id in [
        ids::INSP_RENDER_STRATEGY_ATLAS,
        ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
        ids::INSP_RENDER_STRATEGY_HANDPACKED,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// M14.D: register the Visibility checkbox state. Default Checked
/// matches the canonical absence-equals-visible invariant for newly
/// spawned entities (`ph2d_ecs::Visibility` doc-string).
fn populate_visibility_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_VISIBILITY_CHECK,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

/// M14.A: register the 5 NumberInput states + the Reset button used
/// by [`paint_transform_section`]. Identity defaults seed each field
/// (`0` / `0` / `0` / `1` / `1`); the host overwrites these via
/// [`WidgetStore::set_number_value`] when a fresh
/// [`super::InspectorTransformInfo`] snapshot lands. Per the
/// `set_number_value` focus-guard rule, an in-progress edit on a
/// focused field survives a host snapshot republish.
fn populate_transform_editor(store: &mut WidgetStore) {
    let identity_pairs = [
        (ids::INSP_TRANSFORM_POS_X, 0.0_f64),
        (ids::INSP_TRANSFORM_POS_Y, 0.0_f64),
        (ids::INSP_TRANSFORM_ROT, 0.0_f64),
        (ids::INSP_TRANSFORM_SCALE_X, 1.0_f64),
        (ids::INSP_TRANSFORM_SCALE_Y, 1.0_f64),
    ];
    for (id, value) in identity_pairs {
        let buffer = format!("{value}");
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer,
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    store.register(
        ids::INSP_TRANSFORM_RESET,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn populate_blender_picker(store: &mut WidgetStore) {
    store.register(
        ids::INSP_BLENDER_PICKER,
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(0, 0, 0, 255),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
            hsv_h: 0.0,
            hsv_s: 0.0,
        },
    );
    store.init_blender_palette(
        ids::INSP_BLENDER_PICKER,
        crate::widget::default_palette().swatches.clone(),
    );
    for (id, kind) in [
        (
            ids::BLENDER_ADD_SWATCH,
            crate::interaction::BlenderHitKind::AddSwatch,
        ),
        (
            ids::BLENDER_EYEDROPPER,
            crate::interaction::BlenderHitKind::Eyedropper,
        ),
        (
            ids::BLENDER_DRAG_HANDLE,
            crate::interaction::BlenderHitKind::DragHandle,
        ),
        (
            ids::BLENDER_WHEEL,
            crate::interaction::BlenderHitKind::Wheel,
        ),
        (
            ids::BLENDER_VALUE_SLIDER,
            crate::interaction::BlenderHitKind::ValueSlider,
        ),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind,
            },
        );
    }
    for (id, idx) in [
        (ids::BLENDER_CHANNEL_0, 0u8),
        (ids::BLENDER_CHANNEL_1, 1),
        (ids::BLENDER_CHANNEL_2, 2),
        (ids::BLENDER_CHANNEL_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: crate::interaction::BlenderHitKind::ChannelSlider(idx),
            },
        );
    }
    for (id, idx) in [
        (ids::BLENDER_NUM_0, 0u8),
        (ids::BLENDER_NUM_1, 1),
        (ids::BLENDER_NUM_2, 2),
        (ids::BLENDER_NUM_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_blender_channel(ids::INSP_BLENDER_PICKER, id, idx);
    }
    store.register(
        ids::BLENDER_HEX,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "#E7E7E7FF".to_string(),
            caret: 9,
            selection_anchor: None,
        },
    );
    store.link_blender_hex(ids::INSP_BLENDER_PICKER, ids::BLENDER_HEX);
    for (id, kind) in [
        (
            ids::BLENDER_INTERP_LINEAR,
            crate::interaction::BlenderHitKind::InterpolationLinear,
        ),
        (
            ids::BLENDER_INTERP_PERCEPTUAL,
            crate::interaction::BlenderHitKind::InterpolationPerceptual,
        ),
        (
            ids::BLENDER_CHANNEL_RGB,
            crate::interaction::BlenderHitKind::ChannelRgb,
        ),
        (
            ids::BLENDER_CHANNEL_HSV,
            crate::interaction::BlenderHitKind::ChannelHsv,
        ),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind,
            },
        );
    }
    for (id, swatch_idx) in [
        (ids::BLENDER_SWATCH_0, 0u8),
        (ids::BLENDER_SWATCH_1, 1),
        (ids::BLENDER_SWATCH_2, 2),
        (ids::BLENDER_SWATCH_3, 3),
        (ids::BLENDER_SWATCH_4, 4),
        (ids::BLENDER_SWATCH_5, 5),
        (ids::BLENDER_SWATCH_6, 6),
        (ids::BLENDER_SWATCH_7, 7),
        (ids::BLENDER_SWATCH_8, 8),
        (ids::BLENDER_SWATCH_9, 9),
        (ids::BLENDER_SWATCH_10, 10),
        (ids::BLENDER_SWATCH_11, 11),
        (ids::BLENDER_SWATCH_12, 12),
        (ids::BLENDER_SWATCH_13, 13),
        (ids::BLENDER_SWATCH_14, 14),
        (ids::BLENDER_SWATCH_15, 15),
        (ids::BLENDER_SWATCH_16, 16),
        (ids::BLENDER_SWATCH_17, 17),
        (ids::BLENDER_SWATCH_18, 18),
        (ids::BLENDER_SWATCH_19, 19),
        (ids::BLENDER_SWATCH_20, 20),
        (ids::BLENDER_SWATCH_21, 21),
        (ids::BLENDER_SWATCH_22, 22),
        (ids::BLENDER_SWATCH_23, 23),
        (ids::BLENDER_SWATCH_24, 24),
        (ids::BLENDER_SWATCH_25, 25),
        (ids::BLENDER_SWATCH_26, 26),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: crate::interaction::BlenderHitKind::PaletteSwatch(swatch_idx),
            },
        );
    }
}

fn populate_samples(store: &mut WidgetStore) {
    // Text input — pre-filled with "Player".
    store.register(
        ids::INSP_SAMPLE_TEXT,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Player".to_string(),
            caret: 6,
            selection_anchor: None,
        },
    );
    // Multi-line area — placeholder until the user types.
    store.register(
        ids::INSP_SAMPLE_TEXTAREA,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Notes about this entity…\nLine two.".to_string(),
            caret: 0,
            selection_anchor: None,
        },
    );
    // Combobox — empty query, 3 options.
    store.register(
        ids::INSP_SAMPLE_COMBO,
        InteractiveState::Combobox {
            state: ComboboxState::Normal,
            open: false,
            query: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    // NumberInput chip.
    store.register(
        ids::INSP_SAMPLE_NUMBER,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 42.0,
            buffer: "42".to_string(),
            caret: 0,
            last_committed: 42.0,
            selection_anchor: None,
        },
    );
    // Slider × chip pair (62 % default).
    store.register(
        ids::INSP_SAMPLE_SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.62,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SAMPLE_SLIDER_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.62,
            buffer: crate::interaction::format_number(0.62),
            caret: 0,
            last_committed: 0.62,
            selection_anchor: None,
        },
    );
    store.link_slider_number(ids::INSP_SAMPLE_SLIDER, ids::INSP_SAMPLE_SLIDER_CHIP);

    // Checkbox + Toggle.
    store.register(
        ids::INSP_SAMPLE_CHECKBOX,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    store.register(
        ids::INSP_SAMPLE_TOGGLE,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: true,
        },
    );
    // RadioGroup options — each registered as a Plain hit so the
    // dispatcher emits Click events; selection lives on the painter
    // side via a single Button on `INSP_SAMPLE_RADIO_A` carrying
    // the selected index in its Pressed flag (segmented variant
    // reads it directly).
    for id in [
        ids::INSP_SAMPLE_RADIO_A,
        ids::INSP_SAMPLE_RADIO_B,
        ids::INSP_SAMPLE_RADIO_C,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    if let Some(InteractiveState::Button { state }) = store.get_mut(ids::INSP_SAMPLE_RADIO_A) {
        *state = ButtonState::Pressed;
    }

    // Dropdown.
    store.register(
        ids::INSP_SAMPLE_DROPDOWN,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );

    // Tabs — three buttons, first pressed.
    for id in [
        ids::INSP_SAMPLE_TAB_A,
        ids::INSP_SAMPLE_TAB_B,
        ids::INSP_SAMPLE_TAB_C,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    if let Some(InteractiveState::Button { state }) = store.get_mut(ids::INSP_SAMPLE_TAB_A) {
        *state = ButtonState::Pressed;
    }

    // TreeView root + leaves. The root is `Plain` (its click flips
    // the panel's collapsed flag); leaves are `Button` so we can
    // reuse the same `pin_button_selection` trick used for the
    // Radio/Tabs samples (exactly one is `Pressed`, persistent
    // across frames).
    store.register(ids::INSP_SAMPLE_TREE_ROOT, InteractiveState::Plain);
    for id in [ids::INSP_SAMPLE_TREE_LEAF_A, ids::INSP_SAMPLE_TREE_LEAF_B] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    if let Some(InteractiveState::Button { state }) = store.get_mut(ids::INSP_SAMPLE_TREE_LEAF_A) {
        *state = ButtonState::Pressed;
    }

    // Vector3Editor — 3 chips.
    for (id, v) in [
        (ids::INSP_SAMPLE_V3_X, 1.0_f64),
        (ids::INSP_SAMPLE_V3_Y, 2.0),
        (ids::INSP_SAMPLE_V3_Z, 3.0),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: v,
                buffer: crate::interaction::format_number(v),
                caret: 0,
                last_committed: v,
                selection_anchor: None,
            },
        );
    }

    // ColorSwatch — Plain hit; the painter reads the live color
    // from `widget_colors[INSP_SAMPLE_SWATCH]` each frame, and the
    // global picker opens / writes back through the same slot.
    // Seed the default purple here so the initial visual + the
    // picker's first-open value both match.
    store.register(ids::INSP_SAMPLE_SWATCH, InteractiveState::Plain);
    store.set_widget_color(ids::INSP_SAMPLE_SWATCH, [120, 60, 200, 255]);

    // Buttons.
    for id in [
        ids::INSP_SAMPLE_BTN_PRIMARY,
        ids::INSP_SAMPLE_BTN_SECONDARY,
        ids::INSP_SAMPLE_BTN_DANGER,
        ids::INSP_SAMPLE_BTN_ICON,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // ListItem + removable Tag.
    store.register(
        ids::INSP_SAMPLE_LIST_ITEM,
        InteractiveState::ListItem {
            state: ListItemState::Normal,
            selected: false,
        },
    );
    store.register(
        ids::INSP_SAMPLE_TAG_REMOVE,
        InteractiveState::Tag {
            state: TagState::Normal,
        },
    );

    // Section headers — every section is collapsible. Registered
    // as Plain so the dispatch emits a `Click` event; the inspector
    // `apply_event` flips the store's collapsed flag.
    for id in SECTION_IDS {
        store.register(id, InteractiveState::Plain);
    }
    // Section color-circle hits. Click → opens the global picker
    // editing the circle's color. Registered as Plain (no own
    // visual state — the painter just draws the rgba stored in
    // `widget_colors[id]`).
    for id in SECTION_COLOR_IDS {
        store.register(id, InteractiveState::Plain);
    }

    // Context-menu items — must be registered for the dispatch to
    // mark them as focusable + active and emit `Click` on Up. With
    // no registration the menu paints but clicking does nothing
    // (the user's "Create Note ainda não cria nada" report).
    for id in [
        ids::CTX_MENU_CREATE_NOTE,
        ids::CTX_MENU_OUTLINE_NONE,
        ids::CTX_MENU_OUTLINE_0,
        ids::CTX_MENU_OUTLINE_1,
        ids::CTX_MENU_OUTLINE_2,
        ids::CTX_MENU_OUTLINE_3,
        ids::CTX_MENU_OUTLINE_4,
        ids::CTX_MENU_THEME_FORGE,
        ids::CTX_MENU_THEME_PAINT,
        ids::CTX_MENU_THEME_SUNSTONE,
        ids::CTX_MENU_THEME_BLUEPRINT,
        ids::CTX_MENU_RADIUS_SHARP,
        ids::CTX_MENU_RADIUS_DEFAULT,
        ids::CTX_MENU_RADIUS_ROUND,
        ids::CTX_MENU_MIRROR_UI,
        ids::CTX_MENU_SHOW_STATS,
        ids::CTX_MENU_SHOW_GRID,
        ids::CTX_MENU_SAVE,
        ids::CTX_MENU_SAVE_AS,
        ids::CTX_MENU_OPEN_PROJECT,
        ids::CTX_MENU_IMPORT,
        // M14.7 polish (6.3): Settings cascade entry + px/m submenu
        // entries. Without these the cascade click does nothing —
        // the menu paints but the dispatch's `is_focusable` gate
        // rejects them and no `Click` event ever fires.
        ids::CTX_MENU_SETTINGS_PPM,
        ids::CTX_MENU_PPM_16,
        ids::CTX_MENU_PPM_32,
        ids::CTX_MENU_PPM_100,
        ids::CTX_MENU_PPM_256,
        ids::CTX_MENU_PPM_1024,
        // M14.6 F: HierarchyRow per-entity menu entries. Same
        // registration requirement as the rest — otherwise the
        // dispatch's `set_active` skips them on Down and the Up
        // handler never calls `apply_click`.
        ids::CTX_MENU_HIER_RENAME,
        ids::CTX_MENU_HIER_DUPLICATE,
        ids::CTX_MENU_HIER_ADD_CHILD,
        ids::CTX_MENU_HIER_RESET_TRANSFORM,
        ids::CTX_MENU_HIER_DELETE,
        ids::CTX_SCENE_ROW_0,
        ids::CTX_SCENE_ROW_1,
        ids::CTX_SCENE_ROW_2,
        ids::CTX_SCENE_ROW_3,
        ids::CTX_SCENE_ROW_4,
        ids::CTX_SCENE_ROW_5,
        ids::CTX_SCENE_ROW_6,
        ids::CTX_SCENE_ROW_7,
    ] {
        store.register(id, InteractiveState::Plain);
    }
    // Pre-allocated note slot hits. Each painted note registers a
    // hit at one of these ids by position; right-clicking the
    // slot opens the `NoteBackground` menu for that index.
    for id in NOTE_SLOT_IDS {
        store.register(id, InteractiveState::Plain);
    }
    // Note title (TextInput) + body (TextInput multi-line via
    // TextArea) editing slots. Painter syncs `NoteData.title/body`
    // from these stores each frame, so `TextInput` is the live
    // truth; `NoteData` is the snapshot for serialization /
    // right-click menus.
    for id in NOTE_TITLE_IDS {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    for id in NOTE_BODY_IDS {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    // M14.5 inspector phase: pixel-format segmented picker. RGBA8 is
    // pressed by default; RGBA16 is disabled until the asset crate
    // supports 16-bit-channel decoding. Pre-registered so the
    // dispatch can mark them focusable + active and emit `Click` on
    // Up. The Reimport button reads which one is `Pressed` to choose
    // the target format at drain time.
    store.register(
        ids::INSP_RENDER_FORMAT_RGBA8,
        InteractiveState::Button {
            state: crate::widget::ButtonState::Pressed,
        },
    );
    store.register(
        ids::INSP_RENDER_FORMAT_RGBA16,
        InteractiveState::Button {
            state: crate::widget::ButtonState::Disabled,
        },
    );
    store.register(
        ids::INSP_RENDER_SOURCE_REIMPORT,
        InteractiveState::Button {
            state: crate::widget::ButtonState::Normal,
        },
    );
    // Scrollbar thumb hits — must be in the store so `is_focusable`
    // returns true and the dispatch's `set_active` block runs.
    // Without this the Down handler skips the scrollbar-drag-
    // anchor seed and the drag never starts (user's "não está
    // arrastando com o mouse").
    store.register(
        crate::widget::INSPECTOR_SCROLLBAR_ID,
        InteractiveState::Plain,
    );
    store.register(
        crate::widget::HIERARCHY_SCROLLBAR_ID,
        InteractiveState::Plain,
    );
    // Drag handles for movable Inspector / Hierarchy. Reuse the
    // BlenderColorPicker's panel-agnostic drag infrastructure
    // (`apply_blender_hit` → `begin_blender_drag` keyed by the
    // `parent` NodeId; the painter reads
    // `store.blender_picker_offset(parent)` to apply the offset).
    store.register(
        ids::INSP_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::INSP_PANEL,
            kind: crate::interaction::BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::HIER_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::HIER_PANEL,
            kind: crate::interaction::BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::INSP_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::INSP_PANEL,
            kind: crate::interaction::BlenderHitKind::ResizeHandle,
        },
    );
    store.register(
        ids::HIER_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::HIER_PANEL,
            kind: crate::interaction::BlenderHitKind::ResizeHandle,
        },
    );

    // Per-widget tooltips so the generic registry shows hints when
    // the user hovers. Demonstrates the §9.8 lesson in practice.
    for (id, text) in [
        (ids::INSP_SAMPLE_TEXT, "TextInput sample"),
        (ids::INSP_SAMPLE_TEXTAREA, "TextArea sample"),
        (ids::INSP_SAMPLE_COMBO, "Combobox sample"),
        (ids::INSP_SAMPLE_NUMBER, "NumberInput sample"),
        (ids::INSP_SAMPLE_SLIDER, "Slider × chip composite"),
        (ids::INSP_SAMPLE_CHECKBOX, "Checkbox sample"),
        (ids::INSP_SAMPLE_TOGGLE, "Toggle sample"),
        (ids::INSP_SAMPLE_DROPDOWN, "Dropdown sample"),
        (ids::INSP_SAMPLE_SWATCH, "ColorSwatch sample"),
        (ids::INSP_SAMPLE_BTN_PRIMARY, "Primary button"),
        (ids::INSP_SAMPLE_BTN_SECONDARY, "Secondary button"),
        (ids::INSP_SAMPLE_BTN_DANGER, "Danger button"),
        (ids::INSP_SAMPLE_BTN_ICON, "Icon button"),
        (ids::INSP_SAMPLE_LIST_ITEM, "ListItem sample"),
        (ids::INSP_SAMPLE_TAG_REMOVE, "Removable Tag"),
    ] {
        store.set_tooltip(id, text);
    }
}

/// Apply a [`WidgetEvent`] against Inspector widgets. Currently
/// handles:
///   - **Section headers**: clicking toggles the section's collapsed
///     flag on the store.
///   - **Radio group + Tabs**: clicking one option pins it as the
///     `Pressed` Button state and resets the siblings to `Normal`.
///     Without this, the default Button-Up handler reverts every
///     option to `Hovered`/`Normal` after release, losing selection.
pub fn apply_event(store: &mut WidgetStore, event: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = event {
        // Context menu items — route by id using the snapshot of
        // the most recently closed menu request (dispatch closes the
        // menu on the Down event; the Click arrives on Up after the
        // snapshot landed in `last_context_menu`). Returns true so
        // the orchestrator doesn't pass the Click to other panels.
        const OUTLINE_ITEMS: [(ph2d_a11y::NodeId, Option<u8>); 6] = [
            (ids::CTX_MENU_OUTLINE_NONE, None),
            (ids::CTX_MENU_OUTLINE_0, Some(0)),
            (ids::CTX_MENU_OUTLINE_1, Some(1)),
            (ids::CTX_MENU_OUTLINE_2, Some(2)),
            (ids::CTX_MENU_OUTLINE_3, Some(3)),
            (ids::CTX_MENU_OUTLINE_4, Some(4)),
        ];
        if id == ids::CTX_MENU_CREATE_NOTE {
            if let Some(req) = store.consume_last_context_menu()
                && let crate::interaction::ContextMenuKind::CreateNote { panel, .. } = req.kind
            {
                // Convert the click's screen y into body-relative y
                // and look up which section sits below it; the new
                // note slots in just above that section. Returns
                // `None` when the click was past the last section,
                // in which case the note appends at the bottom.
                let scroll_y = store.panel_scroll(panel);
                let body_top_screen = last_body_top_screen_y();
                let body_y = req.y - body_top_screen + scroll_y;
                let before = section_index_below_body_y(body_y);
                // Default to yellow (color_idx 0).
                let new_index = store.notes_for_panel(panel).len();
                store.notes_push(panel, 0, before);
                // Seed the editable title TextInput at the slot the
                // painter will assign to this note. The body starts
                // empty.
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
                    crate::interaction::ContextMenuKind::SectionOutline { section } => {
                        store.set_section_outline_color(section, *color_idx);
                    }
                    crate::interaction::ContextMenuKind::NoteBackground { panel, note_index } => {
                        if let Some(c) = color_idx {
                            store.note_set_color(panel, note_index as usize, *c);
                        }
                    }
                    _ => {}
                }
            }
            return true;
        }

        // Color circle on a section header OR the ColorSwatch
        // sample → open the global BlenderColorPicker editing this
        // widget's color. The picker's value seeds from the
        // widget's current color (defaulting to neutral gray for
        // unseeded targets).
        if SECTION_COLOR_IDS.contains(&id) || id == ids::INSP_SAMPLE_SWATCH {
            let seed = store.widget_color(id).unwrap_or([0x88, 0x88, 0x88, 0xFF]);
            store.set_widget_color(id, seed);
            store.set_picker_target(Some(id));
            // Seed the picker's retained color from the target so
            // the floating picker opens already showing the
            // current color.
            store.set_blender_value(
                ids::INSP_BLENDER_PICKER,
                ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
            );
            return true;
        }

        // Section header → flip collapse.
        if SECTION_IDS.contains(&id) {
            store.toggle_collapsed(id);
            return true;
        }
        // TreeView root chevron → flip child visibility. Uses the
        // same `is_collapsed` side-table as sections; the painter
        // calls `tree.expand(...)` only when `!collapsed`.
        if id == ids::INSP_SAMPLE_TREE_ROOT {
            store.toggle_collapsed(id);
            return true;
        }
        // Tree leaf selection — same pin-pressed trick as radio/tabs.
        if TREE_LEAF_IDS.contains(&id) {
            pin_button_selection(store, id, &TREE_LEAF_IDS);
            return true;
        }
        // Radio group selection lock — pin clicked, clear siblings.
        if RADIO_GROUP_IDS.contains(&id) {
            pin_button_selection(store, id, &RADIO_GROUP_IDS);
            return true;
        }
        // Tabs sample — same shape, different ids.
        if TAB_GROUP_IDS.contains(&id) {
            pin_button_selection(store, id, &TAB_GROUP_IDS);
            return true;
        }
    }
    false
}

/// Force `selected` into `ButtonState::Pressed` and every other id in
/// `group` into `ButtonState::Normal`. Used for "exactly one is
/// active" segmented controls (RadioGroup, Tabs) that model selection
/// as a single Pressed flag across N Button entries.
fn pin_button_selection(
    store: &mut WidgetStore,
    selected: ph2d_a11y::NodeId,
    group: &[ph2d_a11y::NodeId],
) {
    for id in group {
        if let Some(InteractiveState::Button { state }) = store.get_mut(*id) {
            *state = if *id == selected {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint_inspector(
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rect = layout.inspector;
    paint_panel_surface(rect, scene, theme);
    // Standard panel chrome hit zones — visual is in
    // `paint_panel_surface`; we just register hits against this
    // panel's NodeIds. Re-registered after the body to outrank any
    // scrolled widget that drifted into the chrome area.
    let drag_handle_rect = panel_drag_handle_rect(rect);
    let resize_handle_rect = panel_resize_handle_rect(rect);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);

    // Header: panel title "Inspector" + a sub-field showing the
    // currently-selected entity's name (or "(none)" when no entity
    // is selected). The pilot project's selection wiring drives the
    // sub-field; the panel title is constant.
    let title_y = rect.y + 18.0;
    paint_text_title(
        text_system,
        scene,
        "Inspector",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    // M14.E: header subtitle is now the sprite's world size (when a
    // sprite is selected). The entity name moved to the editable
    // TextInput at the top of the body (see `paint_entity_name_row`).
    // For non-sprite entities the subtitle is empty — Transform's own
    // section + the editable name row carry the identification.
    let sprite_for_header = current_inspector_sprite();
    let subtitle_owned;
    let subtitle: &str = match sprite_for_header.as_ref() {
        Some(info) => {
            subtitle_owned = format!("{:.3} × {:.3} m", info.world_size[0], info.world_size[1]);
            subtitle_owned.as_str()
        }
        None => "",
    };
    paint_text(
        text_system,
        scene,
        subtitle,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Sm.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    let div_y = title_y + TypeToken::Md.px() + TypeToken::Sm.px() + 16.0;
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Scroll-aware body. The clip covers the entire body region so
    // overflow stays inside the panel; `scroll_y` shifts the content
    // cursor up by however much the wheel has been scrolled.
    let content_top = div_y + Spacing::Sm.px();
    let content_bottom = rect.y + rect.h - 4.0;
    let scroll_y = store.panel_scroll(ids::INSP_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);

    let inner_x = rect.x + BODY_PAD;
    // Reserve room on the right for the scrollbar's track even if
    // the bar isn't visible this frame — keeps the content width
    // stable so widgets don't reflow when notes/section toggles
    // push content past the viewport. `SCROLLBAR_W + 6` covers the
    // track (10) + the 2 px outer gap + a 4 px breathing margin.
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + 6.0;
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + 4.0;
    // Body: placeholder until the pilot project wires real component
    // editors. The 10-section widget showcase + notes are now in
    // the floating Widget Gallery panel ([`paint_showcase_body`],
    // toggled via the palette pill in the TopBar). The working
    // Inspector here focuses on the actual editor's job — inspect
    // properties of the currently-selected entity. No selection →
    // instructional prompt.
    let section_tops_y: Vec<f32> = Vec::new();
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + 4.0));
    let transform_info = current_inspector_transform();
    let sprite_info = current_inspector_sprite();
    let visibility_info = current_inspector_visibility();
    let name_present = current_inspector_name_is_some();
    let any_section = transform_info.is_some()
        || sprite_info.is_some()
        || visibility_info.is_some()
        || name_present;
    let mut y = body_top_y + 4.0;
    // ── Entity name (M14.E) — editable TextInput at the very top
    // of the body. Replaces the read-only name displays that used to
    // live in the header subtitle (now world size) and the Render
    // Source "Name" row (now removed).
    if name_present {
        y = paint_entity_name_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
        );
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    // ── Visibility row (M14.D) — checkbox below the name field.
    // Drives the same `ph2d_ecs::Visibility` component as the
    // Hierarchy panel's eye toggle (M14.6 A) via
    // `EditorCommand::SetComponent`.
    if visibility_info.is_some() {
        y = paint_visibility_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
        );
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    // ── Transform section (M14.A) — first SECTION (below the
    // Visibility row), since Transform is the most fundamental
    // component. Matches Unity / Godot / Blender conventions where
    // Transform sits above all other components.
    if transform_info.is_some() {
        y = paint_transform_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
        );
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    // ── Render Source section (M14.5) — below Transform.
    if let Some(info) = sprite_info.as_ref() {
        y = paint_render_source_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            info,
        );
    }
    // ── Placeholder when nothing is selected ──
    if !any_section {
        let placeholder = if selection.is_some() {
            "No properties yet for the selected entity."
        } else {
            "Select an entity in the Hierarchy to inspect its properties."
        };
        let line_h = TypeToken::Sm.px() + 4.0;
        let center_y = content_top + (content_bottom - content_top) * 0.5 - line_h * 0.5;
        paint_text(
            text_system,
            scene,
            placeholder,
            inner_x + 8.0,
            center_y,
            TypeToken::Sm.px(),
            (inner_w - 16.0).max(80.0),
            resolve(ColorToken::Text3, theme),
        );
    }

    // Publish the total content height + the EXACT visible body
    // height for the wheel dispatch + hero clamp. visible_h must
    // match the actual content viewport, not a rough panel.h - 60
    // heuristic — the latter overestimated by ~20-30 px and
    // prevented scrolling to reach the last note (user reported the
    // scroll bound didn't adapt to a newly-added note).
    let content_h = (y - body_top_y).max(0.0);
    let visible_h = (content_bottom - content_top).max(0.0);
    set_last_inspector_content_h(content_h);
    set_last_inspector_visible_h(visible_h);
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);

    // Scrollbar (right edge of body). Hit-registered with the
    // canonical inspector scrollbar id; dispatch handles drag.
    if crate::widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, content_top, rect.w, visible_h);
        let track = crate::widget::scrollbar_track_rect(body);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::INSP_PANEL);
        crate::widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        // Register the thumb hit so Down on it starts a drag.
        hit_index.register(crate::widget::INSPECTOR_SCROLLBAR_ID, thumb);
    }

    // Second pass: open Dropdown popover paints on top of every
    // section that ran before it. We reconstruct the Dropdown
    // (cheap — fixed option list) and call the popover-only painter
    // at the chip rect captured during the Lists section.
    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = ["Front", "Side", "Top"];
        let selected_label = labels.get(sel_idx).copied().unwrap_or("Front");
        let dd = Dropdown::new(
            ids::INSP_SAMPLE_DROPDOWN,
            "View",
            vec![
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
            ],
        )
        .selected(selected_label)
        .open(true);
        crate::widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        // Re-register the option hits AFTER the popover paint so
        // they sit on top of the section hits painted earlier.
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    scene.pop_layer();

    // Standard panel chrome — painted AFTER the body so the
    // corner dot sits on top of widgets that may overlap the
    // bottom-right area. Same re-registration pattern keeps the
    // drag pill + resize handle hit zones above any scrolled
    // widget's rect.
    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);
}

/// Paint the canonical widget showcase at `rect`. Designed to be
/// called from the floating Widget Gallery panel (see
/// [`super::widget_gallery`]). The live `paint_inspector` uses its
/// own minimalist body; this entry point keeps the 10-section
/// showcase reachable inside the working app so peripheral agents
/// have a single in-app source of truth for UI decoration.
///
/// Re-uses the `paint_*_section` painters preserved as `dead_code`
/// after the Inspector switched to the live entity-binding model.
/// Uses `ids::GAL_*` for panel chrome so the gallery's drag /
/// resize state is independent of the live Inspector at
/// `ids::INSP_*`.
///
/// `rect` is the panel rect in viewport pixels. Content is clipped
/// to that rect; v1 is non-scrollable, so callers should size the
/// panel tall enough to fit all 10 sections (~700 px at default
/// theme).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_showcase_body(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    paint_panel_surface(rect, scene, theme);
    // Drag pill + resize gripper hit zones. Visuals are inside
    // `paint_panel_surface` / `paint_panel_corner_dot`; we register
    // the hits here against the gallery's own NodeIds so the
    // BlenderHit dispatch (`DragHandle` / `ResizeHandle`) drives
    // `GAL_PANEL` independently of the Inspector.
    let drag_handle_rect = panel_drag_handle_rect(rect);
    let resize_handle_rect = panel_resize_handle_rect(rect);
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);

    // Header: title + subtitle + divider, matching the reference
    // snapshot's Inspector header style.
    let title_y = rect.y + 18.0;
    paint_text_title(
        text_system,
        scene,
        "Widget Gallery",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0 - 40.0,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        text_system,
        scene,
        "Canonical widget showcase \u{00b7} reference for peripheral agents",
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    // Close (X) at top-right of the header strip.
    let close_size = 24.0_f32;
    let close_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - close_size,
        title_y - 2.0,
        close_size,
        close_size,
    );
    hit_index.register(ids::GAL_CLOSE, close_rect);
    crate::paint::paint_icon(
        scene,
        IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        1.5,
    );

    let div_y = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 16.0;
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Body clipped to panel rect with wheel-driven scroll offset
    // routed through `GAL_PANEL` (independent of `INSP_PANEL`).
    // Reserve room for the scrollbar even when it isn't visible so
    // the section content width is stable.
    let content_top = div_y + Spacing::Sm.px();
    let content_bottom = rect.y + rect.h - 4.0;
    let scroll_y = store.panel_scroll(ids::GAL_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);

    let inner_x = rect.x + BODY_PAD;
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + 6.0;
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + 4.0;
    let mut y = body_top_y;
    // Publish the body's screen-Y origin so the right-click dispatch
    // can convert screen-y → body-y when computing `before_section`
    // for a new note (`section_index_below_body_y`). Inspector's live
    // paint also writes this thread-local, but the gallery paints
    // AFTER inspector in `paint_hero_screen`, so the gallery's value
    // wins for the next dispatch tick — correct for clicks on the
    // gallery body.
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + 4.0));

    // Notes — read once and partition by `before_section`. Notes
    // tagged with `Some(i)` paint immediately above `SECTION_IDS[i]`;
    // notes with `None` paint at the tail after the last section.
    let all_notes = store.notes_for_panel(ids::GAL_PANEL).to_vec();
    let mut notes_per_section: [Vec<(usize, NoteData)>; 10] = Default::default();
    let mut trailing_notes: Vec<(usize, NoteData)> = Vec::new();
    for (idx, note) in all_notes.into_iter().enumerate() {
        match note.before_section {
            Some(i) if (i as usize) < notes_per_section.len() => {
                notes_per_section[i as usize].push((idx, note));
            }
            _ => trailing_notes.push((idx, note)),
        }
    }

    // Body-relative top-Y of each section header — captured so the
    // right-click dispatch can map a click to "which section the
    // user is targeting" for note insertion.
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(SECTION_IDS.len());
    let mut section_idx: usize = 0;
    macro_rules! paint_pending_notes {
        () => {
            for (slot, note) in &notes_per_section[section_idx] {
                paint_one_note(
                    scene,
                    text_system,
                    hit_index,
                    store,
                    inner_x,
                    inner_w,
                    &mut y,
                    note,
                    *slot,
                );
            }
        };
    }
    // Section macro: paints any notes anchored here, then the
    // section, then the colored outline (if the user picked one via
    // right-click → "Section outline"), then a separator. Each
    // iteration also records the section's body-relative top y so
    // `section_index_below_body_y` works.
    macro_rules! section {
        ($f:ident, $section_id:expr) => {
            paint_pending_notes!();
            let y_before = y;
            push_section_top_y(&mut section_tops_y, y_before - body_top_y);
            let new_y = $f(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            );
            if let Some(color_idx) = store.section_outline_color($section_id) {
                let rgba = crate::screens::hero::context_menu_overlay::HIGHLIGHTER_RGBA
                    [color_idx.min(4) as usize];
                let pad = 4.0_f32;
                let block = Rect::new(
                    inner_x - pad,
                    y_before - pad,
                    inner_w + pad * 2.0,
                    (new_y - y_before + pad * 2.0).max(0.0),
                );
                let outline_color =
                    ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
                crate::paint::stroke_rounded_rect(
                    scene,
                    block,
                    Radius::Md.px(),
                    2.0,
                    outline_color,
                );
            }
            y = paint_section_separator(scene, theme, inner_x, inner_w, new_y);
            #[allow(unused_assignments)]
            {
                section_idx += 1;
            }
        };
    }
    section!(paint_inputs_section, ids::INSP_SECTION_INPUTS);
    section!(paint_slider_section, ids::INSP_SECTION_SLIDER);
    section!(paint_switches_section, ids::INSP_SECTION_SWITCHES);
    section!(paint_lists_section, ids::INSP_SECTION_LISTS);
    section!(paint_vector_section, ids::INSP_SECTION_VECTOR);
    section!(paint_status_section, ids::INSP_SECTION_STATUS);
    section!(paint_color_section, ids::INSP_SECTION_COLOR);
    section!(paint_actions_section, ids::INSP_SECTION_ACTIONS);
    section!(paint_identity_section, ids::INSP_SECTION_IDENTITY);
    section!(paint_card_section, ids::INSP_SECTION_CARD);
    // Trailing notes (anchor = None or out-of-range section index)
    // paint at the bottom after all sections.
    for (slot, note) in &trailing_notes {
        paint_one_note(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            &mut y,
            note,
            *slot,
        );
    }
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);
    // Publish content + visible heights so the host can clamp the
    // wheel-scroll bound and so the scrollbar's thumb sizes itself
    // correctly. Mirror of the live Inspector's `set_last_inspector_*`
    // pair — kept separate so the two panels can scroll independently.
    let content_h = (y - body_top_y).max(0.0);
    let visible_h = (content_bottom - content_top).max(0.0);
    set_last_gallery_content_h(content_h);
    set_last_gallery_visible_h(visible_h);

    // Scrollbar — same widget as Inspector / Hierarchy, but routed
    // via `GALLERY_SCROLLBAR_ID` so `dispatch::scrollbar_panel_for_id`
    // sends drag-thumb moves to `GAL_PANEL`.
    if crate::widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, content_top, rect.w, visible_h);
        let track = crate::widget::scrollbar_track_rect(body);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::GAL_PANEL);
        crate::widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(crate::widget::GALLERY_SCROLLBAR_ID, thumb);
    }

    // Late-paint phase: open Dropdown popover sits on top of every
    // section that ran before it. `take_pending_dropdown_chip` is a
    // thread_local owned by the showcase; the live Inspector never
    // paints dropdowns so there's no contention.
    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = ["Front", "Side", "Top"];
        let selected_label = labels.get(sel_idx).copied().unwrap_or("Front");
        let dd = Dropdown::new(
            ids::INSP_SAMPLE_DROPDOWN,
            "View",
            vec![
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
            ],
        )
        .selected(selected_label)
        .open(true);
        crate::widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    scene.pop_layer();
    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// M14.D: paint the live Visibility checkbox row above the Transform
/// section. Mirrors the eye toggle that's already in the Hierarchy
/// panel (M14.6 A) — both surfaces drive the same underlying
/// `ph2d_ecs::Visibility` component via `EditorCommand::SetComponent`.
///
/// Layout: single row, Checkbox + "Visible" label. The store carries
/// the live `CheckboxValue`; `paint_hero_screen` writes it on the
/// frame the host publishes a fresh
/// [`super::InspectorVisibilityInfo`] snapshot so the displayed state
/// always tracks the underlying ECS.
///
/// Returns the y-coordinate of the bottom of the painted row.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_entity_name_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let row_h = 28.0_f32;
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_ENTITY_NAME, host);
    let (state, text, caret, anchor) = match store.get(ids::INSP_ENTITY_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(ids::INSP_ENTITY_NAME, "")
        .placeholder("Name\u{2026}")
        .state(state);
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    y + row_h + 6.0
}

/// M14.D: paint the live Visibility checkbox row above the Transform
/// section. Mirrors the eye toggle in the Hierarchy panel.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_visibility_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let row_h = 24.0_f32;
    let (state, value) = match store.checkbox(ids::INSP_VISIBILITY_CHECK) {
        Some(pair) => pair,
        // Fallback: render Checked as a sensible default if the
        // store hasn't been populated (e.g. early-paint smoke tests).
        None => (CheckboxState::Normal, CheckboxValue::Checked),
    };
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_VISIBILITY_CHECK, host);
    let checkbox = Checkbox::new(ids::INSP_VISIBILITY_CHECK, "Visible")
        .state(state)
        .value(value);
    paint_checkbox(&checkbox, host, scene, text_system, theme);
    y + row_h + 6.0
}

/// M14.A: paint the live `Transform` editor section. Shows Position
/// X/Y (meters), Rotation (degrees, rad ↔ deg conversion at the
/// paint/commit boundary), Scale X/Y (unitless), and a Reset-to-
/// Identity button in the section header. Z is intentionally absent
/// — `Transform` is 2D by design (SKILL §3 + ADR-0025).
///
/// Wiring: the section paints the canonical
/// [`crate::widget::paint_number_input_with_buffer`] (Widget Gallery
/// reference) for each of the 5 editable fields. Live values come
/// from the [`WidgetStore`]'s number-value cache; the host seeds
/// those via `set_number_value` whenever a new
/// [`super::InspectorTransformInfo`] snapshot lands (selection
/// change, gizmo drag, script mutation). Per
/// [`crate::interaction::WidgetStore::set_number_value`], focused
/// fields skip the rewrite so an in-progress edit isn't clobbered.
///
/// Commits flow through `WidgetEvent::ValueChanged` (Enter / blur)
/// in [`super::HeroScreen::apply_event`], which assembles a fresh
/// [`super::InspectorTransformInfo`] from the 5 store values and
/// publishes it via `pending_transform_edit` for the shell to push
/// to its `EditorCommandQueue` as
/// [`ph2d_ecs::scene::commands::EditorCommand::SetComponent`].
///
/// Returns the y-coordinate of the bottom of the painted section so
/// the caller can advance the body cursor.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_transform_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let field_h = 28.0_f32;
    let row_gap = 6.0_f32;
    let label_color = resolve(ColorToken::Text2, theme);

    // ── Section header: "Transform" title + Reset (Identity) button ──
    paint_text_title(
        text_system,
        scene,
        "Transform",
        x,
        y,
        TypeToken::Md.px(),
        w - 90.0,
        resolve(ColorToken::Text1, theme),
    );
    let reset_w = 80.0_f32;
    let reset_h = 24.0_f32;
    let reset_rect = Rect::new(x + w - reset_w, y - 2.0, reset_w, reset_h);
    let reset_state = store
        .button_state(ids::INSP_TRANSFORM_RESET)
        .unwrap_or(ButtonState::Normal);
    hit_index.register(ids::INSP_TRANSFORM_RESET, reset_rect);
    let reset_btn = Button::new(ids::INSP_TRANSFORM_RESET, "Reset")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset_btn, reset_rect, scene, text_system, theme);
    let mut cur_y = y + TypeToken::Md.px() + 10.0;
    cur_y = paint_section_separator(scene, theme, x, w, cur_y);

    // ── 5-column grid geometry ──────────────────────────────────────
    // | col 1: row label | col 2: X tag | col 3: X box | col 4: Y tag | col 5: Y box |
    // Col 2 and col 4 are a single-letter wide so the axis tags hug
    // their boxes. Col 1 fixed at the widest row label ("Rotation
    // (°)"); cols 3 + 5 split the remaining width evenly.
    //
    // Two gap sizes: the gap BEFORE each axis tag (col 1→2, col 3→4)
    // is the standard `col_gap` so columns breathe, but the gap
    // BETWEEN the tag and its own box (col 2→3, col 4→5) is tighter
    // (`tag_box_gap`) so the eye reads tag+box as a single unit.
    let col_gap = 8.0_f32;
    let tag_box_gap = 2.0_f32;
    let label_col_w = 78.0_f32;
    let axis_col_w = 12.0_f32;
    // Width consumed by the non-box columns: label + 2×(col_gap before
    // tag) + 2×(tag) + 2×(tag→box gap). The boxes share what's left.
    let non_box_w = label_col_w + col_gap * 2.0 + (axis_col_w + tag_box_gap) * 2.0;
    let box_col_w = ((w - non_box_w) * 0.5).max(40.0);
    let axis_label_font = TypeToken::Base.px();

    // ── Helper: paint one row of the grid ──────────────────────────
    // `right_id == None` means "row has only an X field" (Rotation
    // case) — col 4 + col 5 stay empty so the grid still aligns.
    let paint_row = |scene: &mut VectorScene,
                     text_system: &mut TextSystem,
                     hit_index: &mut HitIndex,
                     row_y: f32,
                     row_label: &str,
                     left_id: NodeId,
                     left_tag: &str,
                     left_color: ColorToken,
                     left_step: f64,
                     right: Option<(NodeId, &str, ColorToken, f64)>| {
        // Col 1: row label, vertically centered in the field row.
        paint_text(
            text_system,
            scene,
            row_label,
            x,
            row_y + (field_h - label_font) * 0.5,
            label_font,
            label_col_w,
            label_color,
        );
        // Col 2: X / left-axis tag.
        let left_tag_x = x + label_col_w + col_gap;
        paint_text(
            text_system,
            scene,
            left_tag,
            left_tag_x,
            row_y + (field_h - axis_label_font) * 0.5,
            axis_label_font,
            axis_col_w,
            resolve(left_color, theme),
        );
        // Col 3: left field, hugging the X tag (`tag_box_gap`, not
        // `col_gap` — see grid geometry comment). Reads full state
        // from the store so the canonical focus-guard semantics in
        // [`WidgetStore::set_number_value`] take effect — host
        // snapshot refreshes never clobber an in-progress edit.
        let left_box_x = left_tag_x + axis_col_w + tag_box_gap;
        let left_rect = Rect::new(left_box_x, row_y, box_col_w, field_h);
        hit_index.register(left_id, left_rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, left_id);
        let input = NumberInput::new(left_id, "", value)
            .step(left_step)
            .state(state);
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            left_rect,
            scene,
            text_system,
            theme,
        );
        // Col 4 + 5: right-axis tag + box, when present.
        if let Some((right_id, right_tag, right_color, right_step)) = right {
            let right_tag_x = left_box_x + box_col_w + col_gap;
            paint_text(
                text_system,
                scene,
                right_tag,
                right_tag_x,
                row_y + (field_h - axis_label_font) * 0.5,
                axis_label_font,
                axis_col_w,
                resolve(right_color, theme),
            );
            let right_box_x = right_tag_x + axis_col_w + tag_box_gap;
            let right_rect = Rect::new(right_box_x, row_y, box_col_w, field_h);
            hit_index.register(right_id, right_rect);
            let (r_state, r_value, r_buffer, r_caret, r_anchor) =
                read_number_input(store, right_id);
            let r_input = NumberInput::new(right_id, "", r_value)
                .step(right_step)
                .state(r_state);
            paint_number_input_with_buffer(
                &r_input,
                Some(r_buffer),
                r_caret,
                r_anchor,
                right_rect,
                scene,
                text_system,
                theme,
            );
        }
    };

    // ── Three rows, same grid ──
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Position",
        ids::INSP_TRANSFORM_POS_X,
        "X",
        ColorToken::Danger,
        0.01,
        Some((ids::INSP_TRANSFORM_POS_Y, "Y", ColorToken::Success, 0.01)),
    );
    cur_y += field_h + row_gap;
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Rotation (°)",
        ids::INSP_TRANSFORM_ROT,
        "",
        ColorToken::Text3,
        1.0,
        None,
    );
    cur_y += field_h + row_gap;
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Scale",
        ids::INSP_TRANSFORM_SCALE_X,
        "X",
        ColorToken::Danger,
        0.1,
        Some((ids::INSP_TRANSFORM_SCALE_Y, "Y", ColorToken::Success, 0.1)),
    );
    cur_y += field_h + 4.0;

    cur_y
}

/// M14.5 inspector phase (6.4/§9): paint the "Render Source" section
/// when a sprite entity is selected. Shows the entity name, world
/// size, source kind (Atlas / Hand-packed / Individual), source-image
/// pixels, and a "Reimport at current px/m" button that re-decodes
/// the source asset at the project's current `pixels_per_meter`.
///
/// Read-only display except for the Reimport button — the strategy
/// switcher is a later milestone (M14.5 follow-up); the picker shows
/// the current strategy without offering a swap so callers can already
/// see which storage backs each sprite.
#[allow(clippy::too_many_arguments)]
fn paint_render_source_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    let line_font = TypeToken::Sm.px();
    let label_font = TypeToken::Xs.px();
    let row_gap = 4.0_f32;
    let row_h = line_font + row_gap;
    // Section title.
    paint_text_title(
        text_system,
        scene,
        "Render Source",
        x,
        y,
        TypeToken::Md.px(),
        w,
        resolve(ColorToken::Text1, theme),
    );
    let mut cur_y = y + TypeToken::Md.px() + 8.0;
    // Separator under the title.
    cur_y = paint_section_separator(scene, theme, x, w, cur_y);

    // Helper: paint "label · value" two-line row.
    let paint_pair = |scene: &mut VectorScene,
                      text_system: &mut TextSystem,
                      label: &str,
                      value: &str,
                      mut yy: f32|
     -> f32 {
        paint_text(
            text_system,
            scene,
            label,
            x,
            yy,
            label_font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        yy += label_font + 2.0;
        paint_text(
            text_system,
            scene,
            value,
            x,
            yy,
            line_font,
            w,
            resolve(ColorToken::Text1, theme),
        );
        yy + row_h + row_gap
    };

    // M14.E: "Name" and "World size" rows previously lived here.
    // They moved to the editable name TextInput at the top of the
    // Inspector body and the header subtitle, respectively. Render
    // Source now focuses on the actual storage strategy + identifier.
    // M14.C: 3-segment Strategy switcher. Each button is `Pressed`
    // when its strategy matches the current source_kind; the painter
    // computes this from the snapshot each frame so the buttons
    // always agree with the underlying ECS (no in-progress edit
    // state to worry about — click → host swap → snapshot
    // republishes → painter re-pins). HandPacked stays clickable
    // but the host shows a toast and skips the swap in v1.
    paint_text(
        text_system,
        scene,
        "Strategy",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += label_font + 4.0;
    let strategy_btn_h = 28.0_f32;
    let strategy_gap = 6.0_f32;
    let strategy_btn_w = ((w - strategy_gap * 2.0) / 3.0).max(40.0);
    let strategy_buttons = [
        (
            ids::INSP_RENDER_STRATEGY_ATLAS,
            "Atlas",
            matches!(info.source_kind, InspectorSpriteSource::Atlas { .. }),
        ),
        (
            ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            "Individual",
            matches!(info.source_kind, InspectorSpriteSource::Individual { .. }),
        ),
        (
            ids::INSP_RENDER_STRATEGY_HANDPACKED,
            "Hand-packed",
            matches!(info.source_kind, InspectorSpriteSource::HandPacked),
        ),
    ];
    for (i, (id, label, pressed)) in strategy_buttons.into_iter().enumerate() {
        let bx = x + (strategy_btn_w + strategy_gap) * i as f32;
        let r = Rect::new(bx, cur_y, strategy_btn_w, strategy_btn_h);
        hit_index.register(id, r);
        // Driven from the snapshot — hover/normal/pressed otherwise
        // mirror the canonical button states.
        let state = if pressed {
            ButtonState::Pressed
        } else {
            store.button_state(id).unwrap_or(ButtonState::Normal)
        };
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .state(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    cur_y += strategy_btn_h + 8.0;
    // Storage detail (atlas key / texture id) — kept as a small
    // line under the switcher so the user can still see the
    // identifier without it cluttering the buttons.
    let storage_detail = match info.source_kind {
        InspectorSpriteSource::Atlas { key } => format!("Atlas key: {}", key),
        InspectorSpriteSource::Individual { texture_id } => {
            format!("Texture id: {}", texture_id)
        }
        InspectorSpriteSource::HandPacked => "Hand-packed (atlas asset)".to_string(),
    };
    cur_y = paint_pair(scene, text_system, "Storage", &storage_detail, cur_y);
    if let Some((pw, ph)) = info.source_pixels {
        let px_str = format!("{} × {} px", pw, ph);
        cur_y = paint_pair(scene, text_system, "Source", &px_str, cur_y);
    }

    // Pixel-format segmented picker — RGBA8 (default, supported) +
    // RGBA16 (disabled until the asset layer grows 16-bit storage).
    // Pressed = current choice; clicking the alternative flips the
    // pin via `pin_button_selection` in `apply_event`. Reimport
    // reads the pressed id at drain time.
    paint_text(
        text_system,
        scene,
        "Pixel format",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += label_font + 4.0;
    let btn_h = 28.0_f32;
    let gap = 6.0_f32;
    let half_w = (w - gap) * 0.5;
    let rgba8_rect = Rect::new(x, cur_y, half_w, btn_h);
    let rgba16_rect = Rect::new(x + half_w + gap, cur_y, half_w, btn_h);
    let rgba8_state = store
        .button_state(ids::INSP_RENDER_FORMAT_RGBA8)
        .unwrap_or(ButtonState::Pressed);
    // RGBA16 stays Disabled regardless of stored state until the
    // asset crate adds half-float decode — the click handler skips
    // pinning Disabled buttons, so the user can't even land on it.
    let rgba16_state = ButtonState::Disabled;
    hit_index.register(ids::INSP_RENDER_FORMAT_RGBA8, rgba8_rect);
    hit_index.register(ids::INSP_RENDER_FORMAT_RGBA16, rgba16_rect);
    let rgba8_btn = Button::new(ids::INSP_RENDER_FORMAT_RGBA8, "RGBA8")
        .kind(ButtonKind::Default)
        .state(rgba8_state);
    let rgba16_btn = Button::new(ids::INSP_RENDER_FORMAT_RGBA16, "RGBA16")
        .kind(ButtonKind::Default)
        .state(rgba16_state);
    paint_button(&rgba8_btn, rgba8_rect, scene, text_system, theme);
    paint_button(&rgba16_btn, rgba16_rect, scene, text_system, theme);
    cur_y += btn_h + 8.0;

    // Reimport button — disabled when the snapshot says the source
    // doesn't resolve to a re-decodable asset (procedural / lost).
    let reimport_h = 30.0_f32;
    let btn_rect = Rect::new(x, cur_y, w, reimport_h);
    let id = ids::INSP_RENDER_SOURCE_REIMPORT;
    let state = if !info.can_reimport {
        ButtonState::Disabled
    } else {
        store.button_state(id).unwrap_or(ButtonState::Normal)
    };
    hit_index.register(id, btn_rect);
    let btn = Button::new(id, "Reimport at current px/m")
        .kind(ButtonKind::Default)
        .state(state);
    paint_button(&btn, btn_rect, scene, text_system, theme);
    cur_y + reimport_h + 4.0
}

/// Paint a collapsible section header at `(x, y)` and register its
/// click hit. Returns `(next_y, is_open)` where `is_open` controls
/// whether the caller paints the section body or skips ahead.
///
/// Single source of truth for every section in the Inspector — the
/// chevron direction, hit rect, and collapsed-flag read all live
/// here so individual section painters stay focused on their content.
#[allow(clippy::too_many_arguments)]
fn paint_collapsible_header(
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
    (y + SECTION_HEAD_H + 4.0, !is_collapsed)
}

/// Paint a single sticky-note. Editable: the title + body each
/// have their own TextInput state in the store
/// (`NOTE_TITLE_IDS[slot]` + `NOTE_BODY_IDS[slot]`). Single click
/// on either focuses it; double-click selects all; typing edits.
/// Dark glyphs hardcoded `#212121` for contrast over the light
/// highlighter bg regardless of theme.
///
/// Registers three hit rects:
///   - The whole note slot (`NOTE_SLOT_IDS[slot]`) for right-click
///     → background-color menu. Registered FIRST so the more
///     specific title/body rects layered above win pointer hits
///     within them.
///   - The title sub-rect (`NOTE_TITLE_IDS[slot]`) for focus + edit.
///   - The body sub-rect (`NOTE_BODY_IDS[slot]`).
#[allow(clippy::too_many_arguments)]
fn paint_one_note(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: &mut f32,
    note: &NoteData,
    slot: usize,
) {
    // Font sizes MUST match the dispatch's hardcoded `TypeToken::
    // Base.px()` (13 px) so click→caret mapping uses the same
    // glyph width as the painter. Using `Sm`/`Xs` caused the
    // dispatch to measure prefixes at the wrong font size and put
    // the caret 1–3 chars off (the user's "mapeamento errado do
    // mouse"). Same lesson as docs/UI_Bugs §3.3.
    let pad = 8.0_f32;
    let title_font = TypeToken::Base.px();
    let body_font = TypeToken::Base.px();
    let title_h = title_font + 8.0;
    // Body holds ~3 lines. Painter starts at rect.y + NOTE_TEXT_PAD_Y
    // (=8) and uses line_h = body_font + 4. Total height needs both
    // the top inset and a small bottom buffer or the third line gets
    // clipped under the note's rounded bottom edge.
    let body_h = NOTE_TEXT_PAD_Y * 2.0 + (body_font + 4.0) * 3.0;
    let note_h = title_h + body_h + pad * 2.0;
    let r = Rect::new(x, *y, w, note_h);
    if let Some(slot_id) = NOTE_SLOT_IDS.get(slot) {
        hit_index.register(*slot_id, r);
    }
    let rgba = crate::screens::hero::context_menu_overlay::HIGHLIGHTER_RGBA
        [note.color_idx.min(4) as usize];
    let bg = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
    fill_rounded_rect(scene, r, Radius::Md.px(), bg);

    let dark = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0xFF);
    // Title row.
    let title_rect = Rect::new(r.x + pad, r.y + pad, r.w - pad * 2.0, title_h);
    if let Some(title_id) = NOTE_TITLE_IDS.get(slot) {
        hit_index.register(*title_id, title_rect);
        paint_note_editable_line(
            scene,
            text_system,
            store,
            *title_id,
            title_rect,
            title_font,
            dark,
            "Title",
        );
    }
    // Body region — multi-line below the title.
    let body_rect = Rect::new(r.x + pad, r.y + pad + title_h, r.w - pad * 2.0, body_h);
    if let Some(body_id) = NOTE_BODY_IDS.get(slot) {
        hit_index.register(*body_id, body_rect);
        paint_note_editable_multiline(
            scene,
            text_system,
            store,
            *body_id,
            body_rect,
            body_font,
            dark,
            "Notes…",
        );
    }
    *y += note_h + 8.0;
}

/// Paint an editable single-line text field with no chrome —
/// dark glyphs + caret + selection on a transparent background.
/// Reads the TextInput state at `id` from the store.
///
/// CRITICAL: text origin matches the TextInput dispatch contract
/// (`text_start_x = rect.x + 12`, `text_start_y = rect.y`) — without
/// this alignment, `byte_offset_from_click_xy` measures from a
/// different origin than the painter and click→caret + drag-select
/// land on the wrong byte. Vertical centering still happens, but
/// click→byte ignores y for single-line widgets.
#[allow(clippy::too_many_arguments)]
fn paint_note_editable_line(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    id: NodeId,
    rect: Rect,
    font_size: f32,
    fg: ph2d_vector::Color,
    placeholder: &str,
) {
    let (state, text, caret, anchor) = read_text_input(store, id);
    let focused = state == TextInputState::Focused;
    let text_x = rect.x + NOTE_TEXT_PAD_X;
    let text_w = (rect.w - NOTE_TEXT_PAD_X).max(0.0);
    let text_y = rect.y + (rect.h - font_size) * 0.5;
    // Selection highlight (drawn under glyphs).
    if focused
        && !text.is_empty()
        && let Some(a) = anchor
        && a != caret
    {
        let (s, e) = if a < caret { (a, caret) } else { (caret, a) };
        let s = s.min(text.len());
        let e = e.min(text.len());
        let prefix_w = text_system.prefix_width(&text[..s], font_size);
        let mid_w = if s == e {
            0.0
        } else {
            text_system.prefix_width(&text[s..e], font_size)
        };
        let sel_x = text_x + prefix_w;
        let sel_w = mid_w.min(text_x + text_w - sel_x).max(0.0);
        if sel_w > 0.0 {
            let sel = Rect::new(sel_x, rect.y + 2.0, sel_w, (rect.h - 4.0).max(2.0));
            // Translucent dark wash for the selection so the highlighter
            // bg still shows through.
            let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33);
            fill_rounded_rect(scene, sel, 1.0, sel_color);
        }
    }
    // Visible text (or placeholder if empty and not focused).
    let displayed: &str = if text.is_empty() && !focused {
        placeholder
    } else {
        text
    };
    let display_color = if text.is_empty() && !focused {
        ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80)
    } else {
        fg
    };
    paint_text(
        text_system,
        scene,
        displayed,
        text_x,
        text_y,
        font_size,
        text_w,
        display_color,
    );
    // Caret — only when focused.
    if focused {
        let caret_byte = caret.min(text.len());
        let prefix_w = text_system.prefix_width(&text[..caret_byte], font_size);
        let caret_rect = Rect::new(
            (text_x + prefix_w).min(text_x + text_w),
            rect.y + 2.0,
            1.5,
            (rect.h - 4.0).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg);
    }
}

/// Horizontal inset between a note's rect edge and where text drawn
/// inside it actually starts. Matches the non-hex `TextInput`
/// dispatch math in `byte_offset_from_click_xy` (`rect.x + 12.0`) so
/// click→caret + drag-select route to the byte under the visible
/// cursor.
const NOTE_TEXT_PAD_X: f32 = 12.0;

/// Vertical inset for multi-line note body painting. Mirrors the
/// `TextArea` dispatch math (`text_start_y = rect.y + 8.0`).
const NOTE_TEXT_PAD_Y: f32 = 8.0;

/// Paint an editable multi-line text region with no chrome. Splits
/// the text on `\n` and renders each line; caret + selection mirror
/// the same per-line math the `TextArea` widget uses.
#[allow(clippy::too_many_arguments)]
fn paint_note_editable_multiline(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    id: NodeId,
    rect: Rect,
    font_size: f32,
    fg: ph2d_vector::Color,
    placeholder: &str,
) {
    let (state, text, caret, anchor) = read_text_input(store, id);
    let focused = state == TextInputState::Focused;
    // line_h MUST match the dispatch's TextArea math:
    // `byte_offset_from_click_xy` uses `font_size + 4.0`. Drift
    // breaks click-y → line index. Origin offsets (text_x, text_y0)
    // also MUST match dispatch (`rect.x + 12`, `rect.y + 8`) so
    // click→caret lands at the visible glyph boundary.
    let line_h = font_size + 4.0;
    let text_x = rect.x + NOTE_TEXT_PAD_X;
    let text_y0 = rect.y + NOTE_TEXT_PAD_Y;
    let text_w = (rect.w - NOTE_TEXT_PAD_X).max(0.0);
    if text.is_empty() && !focused {
        paint_text(
            text_system,
            scene,
            placeholder,
            text_x,
            text_y0,
            font_size,
            text_w,
            ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80),
        );
        return;
    }
    // Selection highlight (drawn under glyphs). Supports spanning
    // multiple lines: paints one box per visible line covered by the
    // range. Mirrors `TextArea`'s widget selection math.
    if focused
        && let Some(a) = anchor
        && a != caret
    {
        let (s, e) = if a < caret { (a, caret) } else { (caret, a) };
        let s = s.min(text.len());
        let e = e.min(text.len());
        let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33);
        let mut line_start = 0_usize;
        for (i, line) in text.split('\n').enumerate() {
            let line_end = line_start + line.len();
            // Overlap of [s, e] with this line's byte range.
            let seg_s = s.max(line_start);
            let seg_e = e.min(line_end);
            if seg_s < seg_e {
                let local_s = seg_s - line_start;
                let local_e = seg_e - line_start;
                let prefix_w = text_system.prefix_width(&line[..local_s], font_size);
                let mid_w = text_system.prefix_width(&line[local_s..local_e], font_size);
                let sel_x = text_x + prefix_w;
                let sel_w = mid_w.min(text_x + text_w - sel_x).max(0.0);
                if sel_w > 0.0 {
                    let sel = Rect::new(sel_x, text_y0 + i as f32 * line_h, sel_w, line_h);
                    fill_rounded_rect(scene, sel, 1.0, sel_color);
                }
            }
            line_start = line_end + 1;
        }
    }
    for (i, line) in text.split('\n').enumerate() {
        paint_text(
            text_system,
            scene,
            line,
            text_x,
            text_y0 + i as f32 * line_h,
            font_size,
            text_w,
            fg,
        );
    }
    if focused {
        // Caret on the line containing the caret byte.
        let caret_byte = caret.min(text.len());
        let mut line_start = 0_usize;
        let mut line_idx = 0_usize;
        let mut line_text: &str = "";
        for line in text.split('\n') {
            let line_end = line_start + line.len();
            if caret_byte <= line_end {
                line_text = line;
                break;
            }
            line_start = line_end + 1;
            line_idx += 1;
        }
        let local = caret_byte.saturating_sub(line_start).min(line_text.len());
        let prefix_w = text_system.prefix_width(&line_text[..local], font_size);
        let caret_rect = Rect::new(
            (text_x + prefix_w).min(text_x + text_w),
            text_y0 + line_idx as f32 * line_h,
            1.5,
            (line_h - 2.0).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg);
    }
}

/// Discreet colored separator painted at the end of each section's
/// content (when expanded). 1 px tall, almost full-width (2 px
/// horizontal inset only — line spans the panel body edge to edge).
/// Balanced vertical padding (`SEPARATOR_PAD_Y` above + below) so
/// the line sits centered in its own gap, not lopsided against the
/// content above. Owns the entire inter-section spacing — callers
/// should NOT add `SECTION_GAP` on top.
fn paint_section_separator(scene: &mut VectorScene, theme: Theme, x: f32, w: f32, y: f32) -> f32 {
    let pad_x = 2.0_f32;
    let pad_y = SEPARATOR_PAD_Y;
    let thickness = 1.0_f32;
    let line = Rect::new(x + pad_x, y + pad_y, (w - pad_x * 2.0).max(0.0), thickness);
    fill_rounded_rect(scene, line, 0.5, resolve(ColorToken::Accent, theme));
    y + pad_y + thickness + pad_y
}

/// Vertical breathing room above AND below the section separator
/// line — same on both sides so the colored pill reads as centered
/// between two sections rather than glued to the previous one.
const SEPARATOR_PAD_Y: f32 = 8.0;

#[allow(clippy::too_many_arguments)]
fn paint_left_label(
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

#[allow(clippy::too_many_arguments)]
fn paint_inputs_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_INPUTS,
        ids::INSP_SECTION_INPUTS_COLOR,
        "Inputs",
        4,
    );
    if !open {
        return y;
    }

    // TextInput.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_TEXT, r);
    let (ti_state, ti_text, ti_caret, ti_anchor) = read_text_input(store, ids::INSP_SAMPLE_TEXT);
    let input = TextInput::new(ids::INSP_SAMPLE_TEXT, "Name")
        .placeholder("Entity name")
        .state(ti_state);
    paint_text_input_with_buffer(
        &input,
        Some(ti_text),
        Some(ti_caret),
        ti_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    y += FIELD_H + ROW_GAP;

    // TextArea (3 lines tall).
    let area_h = 60.0_f32;
    let r = Rect::new(x, y, w, area_h);
    hit_index.register(ids::INSP_SAMPLE_TEXTAREA, r);
    let (ta_state, ta_text, ta_caret, ta_anchor) =
        read_text_input(store, ids::INSP_SAMPLE_TEXTAREA);
    let mut ta = TextArea::new(ids::INSP_SAMPLE_TEXTAREA, "Notes")
        .placeholder("Notes…")
        .state(ta_state);
    ta.value = ta_text.to_string();
    paint_text_area_with_state(&ta, Some(ta_caret), ta_anchor, r, scene, text_system, theme);
    y += area_h + ROW_GAP;

    // Combobox.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_COMBO, r);
    let (cb_state, cb_open, cb_query, cb_caret, cb_anchor) =
        read_combobox(store, ids::INSP_SAMPLE_COMBO);
    let combo = Combobox::new(
        ids::INSP_SAMPLE_COMBO,
        "Asset",
        vec![
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_A, "spike.gltf"),
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_B, "block.gltf"),
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_C, "decal.png"),
        ],
    )
    .query(cb_query)
    .open(cb_open)
    .state(cb_state);
    paint_combobox_with_state(&combo, cb_caret, cb_anchor, r, scene, text_system, theme);
    y += FIELD_H + ROW_GAP;

    // NumberInput.
    let label_w = 80.0_f32;
    let chip_w = (w - label_w - 8.0).max(40.0);
    let r = Rect::new(x + label_w + 8.0, y, chip_w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_NUMBER, r);
    paint_left_label(scene, text_system, theme, x, "Value", label_w, y, FIELD_H);
    let (n_state, n_value, n_buffer, n_caret, n_anchor) =
        read_number_input(store, ids::INSP_SAMPLE_NUMBER);
    let num = NumberInput::new(ids::INSP_SAMPLE_NUMBER, "Value", n_value).state(n_state);
    paint_number_input_with_buffer(
        &num,
        Some(n_buffer),
        n_caret,
        n_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    y + FIELD_H
}

#[allow(clippy::too_many_arguments)]
fn paint_slider_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_SLIDER,
        ids::INSP_SECTION_SLIDER_COLOR,
        "Slider",
        1,
    );
    if !open {
        return y;
    }
    let (_, value) = store
        .slider(ids::INSP_SAMPLE_SLIDER)
        .unwrap_or((SliderState::Normal, 0.62));
    let r = Rect::new(x, y, w, FIELD_H);
    paint_slider_with_chip(
        r,
        "Speed",
        value,
        ids::INSP_SAMPLE_SLIDER,
        ids::INSP_SAMPLE_SLIDER_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += FIELD_H;
    y
}

#[allow(clippy::too_many_arguments)]
fn paint_switches_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_SWITCHES,
        ids::INSP_SECTION_SWITCHES_COLOR,
        "Switches",
        3,
    );
    if !open {
        return y;
    }

    // Checkbox.
    let r = Rect::new(x, y, w, 22.0);
    hit_index.register(ids::INSP_SAMPLE_CHECKBOX, r);
    let (cb_state, cb_value) = store
        .checkbox(ids::INSP_SAMPLE_CHECKBOX)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let cb = Checkbox::new(ids::INSP_SAMPLE_CHECKBOX, "Hot reload on save")
        .state(cb_state)
        .value(cb_value);
    paint_checkbox(&cb, r, scene, text_system, theme);
    y += 22.0 + ROW_GAP;

    // Toggle.
    let toggle_w = 44.0_f32;
    let row_h = 22.0_f32;
    let tr = Rect::new(x + w - toggle_w, y, toggle_w, row_h);
    hit_index.register(ids::INSP_SAMPLE_TOGGLE, tr);
    let (tg_state, tg_on) = store
        .toggle(ids::INSP_SAMPLE_TOGGLE)
        .unwrap_or((ToggleState::Normal, true));
    let toggle = Toggle::new(ids::INSP_SAMPLE_TOGGLE, "Snap to grid")
        .state(tg_state)
        .on(tg_on);
    paint_toggle(&toggle, tr, scene, theme);
    paint_left_label(
        scene,
        text_system,
        theme,
        x,
        "Snap to grid",
        w - toggle_w - 6.0,
        y,
        row_h,
    );
    y += row_h + ROW_GAP;

    // Segmented RadioGroup. We use the per-tab pressed-button trick
    // (mirrors the Inspector-tabs pattern in `paint_inspector_tabs`
    // pre-cleanup) so selection survives across frames without
    // adding a typed RadioGroup state to the store.
    let selected = active_index(
        store,
        &[
            ids::INSP_SAMPLE_RADIO_A,
            ids::INSP_SAMPLE_RADIO_B,
            ids::INSP_SAMPLE_RADIO_C,
        ],
    )
    .unwrap_or(0);
    let selected_label: &str = ["Low", "Mid", "High"][selected];
    let rg = RadioGroup::new(
        NodeId(0),
        "Quality",
        vec![
            RadioOption::new(ids::INSP_SAMPLE_RADIO_A, "Low".to_string(), "Low"),
            RadioOption::new(ids::INSP_SAMPLE_RADIO_B, "Mid".to_string(), "Mid"),
            RadioOption::new(ids::INSP_SAMPLE_RADIO_C, "High".to_string(), "High"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(selected_label.to_string());
    let r = Rect::new(x, y, w, 28.0);
    paint_radio_group_with_labels(&rg, r, scene, text_system, theme);
    // Register per-option hit rects.
    for (i, id) in [
        ids::INSP_SAMPLE_RADIO_A,
        ids::INSP_SAMPLE_RADIO_B,
        ids::INSP_SAMPLE_RADIO_C,
    ]
    .iter()
    .enumerate()
    {
        hit_index.register(*id, rg.option_rect(r, i));
    }
    y + 28.0
}

#[allow(clippy::too_many_arguments)]
fn paint_lists_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_LISTS,
        ids::INSP_SECTION_LISTS_COLOR,
        "Lists",
        4,
    );
    if !open {
        return y;
    }

    // Dropdown.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_DROPDOWN, r);
    let (dd_state, dd_open, dd_sel) = match store.get(ids::INSP_SAMPLE_DROPDOWN) {
        Some(InteractiveState::Dropdown {
            state,
            open,
            selected_index,
        }) => (*state, *open, *selected_index),
        _ => (DropdownState::Normal, false, Some(0)),
    };
    let labels = ["Front", "Side", "Top"];
    let selected_label = dd_sel
        .and_then(|i| labels.get(i).copied())
        .unwrap_or("Front");
    let dd = Dropdown::new(
        ids::INSP_SAMPLE_DROPDOWN,
        "View",
        vec![
            DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
            DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
            DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
        ],
    )
    .selected(selected_label)
    .open(dd_open)
    .state(dd_state);
    // Chip first; popover is painted at the END of paint_inspector
    // (after every other section) so it lands ABOVE every other
    // widget. Without this, sections painted later in the loop
    // (Vector/Status/Color) covered the open list. Stash the open
    // dropdown + its chip rect via a thread-local for the second
    // pass — paint_inspector reads it after the section loop and
    // before pop_layer.
    crate::widget::paint_dropdown_chip(&dd, r, scene, text_system, theme);
    if dd_open {
        set_pending_dropdown_chip(Some((dd_sel.unwrap_or(0), r)));
    }
    y += FIELD_H + ROW_GAP;

    // Tabs (segmented).
    let r = Rect::new(x, y, w, 28.0);
    let selected = active_index(
        store,
        &[
            ids::INSP_SAMPLE_TAB_A,
            ids::INSP_SAMPLE_TAB_B,
            ids::INSP_SAMPLE_TAB_C,
        ],
    )
    .unwrap_or(0);
    let tabs = Tabs::new(
        NodeId(0),
        "Mode",
        vec![
            TabItem::new(ids::INSP_SAMPLE_TAB_A, "Edit"),
            TabItem::new(ids::INSP_SAMPLE_TAB_B, "Play"),
            TabItem::new(ids::INSP_SAMPLE_TAB_C, "Debug"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected(selected);
    paint_tabs(&tabs, r, scene, text_system, theme);
    for (i, item) in tabs.items.iter().enumerate() {
        hit_index.register(item.id, tabs.tab_rect(r, i));
    }
    y += 28.0 + 4.0;

    // Tab body — distinct sample per selected tab so the user can
    // see the tab actually swapping content (vs. just visual
    // emphasis on the segmented control). Each body is painted in a
    // `BgElev` rounded panel for visual grouping with the tabs.
    let body_h = 36.0_f32;
    let body_rect = Rect::new(x, y, w, body_h);
    fill_rounded_rect(
        scene,
        body_rect,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let (caption, tone) = match selected {
        0 => ("Editing scene · pencil tool", ColorToken::Text1),
        1 => ("Running simulation · 60 fps", ColorToken::Success),
        _ => ("Logging · 124 events captured", ColorToken::Warn),
    };
    paint_text(
        text_system,
        scene,
        caption,
        body_rect.x + Spacing::Md.px(),
        body_rect.y + (body_rect.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        body_rect.w - Spacing::Md.px() * 2.0,
        resolve(tone, theme),
    );
    y += body_h + ROW_GAP;

    // TreeView (1 root + 2 leaves). Expand-state lives on the
    // store's `collapsed` side-table (same one that drives section
    // collapse) so clicking the root row truly hides/shows its
    // children. Selected leaf comes from the per-leaf `Pressed`
    // button state — `pin_button_selection` keeps exactly one
    // leaf Pressed across frames.
    let expanded = !store.is_collapsed(ids::INSP_SAMPLE_TREE_ROOT);
    let visible_rows = if expanded { 3.0 } else { 1.0 };
    let tree_h = 26.0_f32 * visible_rows;
    let r = Rect::new(x, y, w, tree_h);
    let mut tree = TreeView::new(
        NodeId(0),
        "Tree",
        vec![
            TreeNode::new(ids::INSP_SAMPLE_TREE_ROOT, "Group")
                .icon(IconId::Folder)
                .children(vec![
                    TreeNode::new(ids::INSP_SAMPLE_TREE_LEAF_A, "Item A").icon(IconId::Sprite),
                    TreeNode::new(ids::INSP_SAMPLE_TREE_LEAF_B, "Item B").icon(IconId::Sprite),
                ]),
        ],
    );
    if expanded {
        tree.expand(ids::INSP_SAMPLE_TREE_ROOT);
    }
    let selected_leaf = active_index(store, &TREE_LEAF_IDS).unwrap_or(0);
    tree.select(TREE_LEAF_IDS[selected_leaf]);
    paint_tree_view(&tree, r, scene, text_system, theme, 26.0);
    // Register hit rects for each visible row.
    for (i, (_depth, node)) in tree.visible_rows().iter().enumerate() {
        hit_index.register(node.id, tree.row_rect(r, i, 26.0));
    }
    y += tree_h + ROW_GAP;

    // ListItem.
    let li_h = 26.0_f32;
    let r = Rect::new(x, y, w, li_h);
    hit_index.register(ids::INSP_SAMPLE_LIST_ITEM, r);
    let li_state = match store.get(ids::INSP_SAMPLE_LIST_ITEM) {
        Some(InteractiveState::ListItem { state, .. }) => *state,
        _ => ListItemState::Normal,
    };
    let item = ListItem::new(ids::INSP_SAMPLE_LIST_ITEM, "Open file")
        .icon(IconId::Open)
        // ASCII shortcut — the Command glyph U+2318 (⌘) isn't in the
        // editor's font fallback chain (parley GenericFamily::SansSerif
        // resolves to system fonts that don't include the Unicode
        // technical-symbol block). Showing `Cmd+O` instead of the
        // tofu `□O` for the keyboard hint.
        .value("Cmd+O")
        .chevron(true)
        .state(li_state);
    paint_list_item(&item, r, scene, text_system, theme);
    y + li_h
}

#[allow(clippy::too_many_arguments)]
fn paint_vector_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_VECTOR,
        ids::INSP_SECTION_VECTOR_COLOR,
        "Vector",
        3,
    );
    if !open {
        return y;
    }
    let r = Rect::new(x, y, w, FIELD_H);
    // Pull each axis' live state from the store so focus/typing
    // actually drives the chip. Previously this rendered via the
    // static `paint_vector3_editor` which never reads the in-progress
    // edit buffer — clicking a chip "focused" it visually (border
    // accent) but every keystroke vanished because the painter kept
    // showing `input.value` regardless. See `docs/UI_Bugs/README.md`
    // §9.14.
    let (sx, vx, bx, cx, ax) = read_number_input(store, ids::INSP_SAMPLE_V3_X);
    let (sy, vy, by, cy, ay) = read_number_input(store, ids::INSP_SAMPLE_V3_Y);
    let (sz, vz, bz, cz, az) = read_number_input(store, ids::INSP_SAMPLE_V3_Z);
    let nx = NumberInput::new(ids::INSP_SAMPLE_V3_X, "X", vx).state(sx);
    let ny = NumberInput::new(ids::INSP_SAMPLE_V3_Y, "Y", vy).state(sy);
    let nz = NumberInput::new(ids::INSP_SAMPLE_V3_Z, "Z", vz).state(sz);
    let v3 = Vector3Editor::new(NodeId(0), "Position", nx, ny, nz);
    crate::widget::paint_vector3_editor_with_state(
        &v3,
        [Some(bx), Some(by), Some(bz)],
        [cx, cy, cz],
        [ax, ay, az],
        r,
        scene,
        text_system,
        theme,
    );
    let rects = v3.field_rects(r);
    for (id, fr) in [
        (ids::INSP_SAMPLE_V3_X, rects[0]),
        (ids::INSP_SAMPLE_V3_Y, rects[1]),
        (ids::INSP_SAMPLE_V3_Z, rects[2]),
    ] {
        hit_index.register(id, fr);
    }
    y += FIELD_H;
    y
}

#[allow(clippy::too_many_arguments)]
fn paint_status_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_STATUS,
        ids::INSP_SECTION_STATUS_COLOR,
        "Status",
        3,
    );
    if !open {
        return y;
    }

    // ProgressBar — determinate, 60 %.
    let bar = ProgressBar::new(NodeId(0), "Build")
        .determinate(0.6)
        .show_percent(true);
    let bar_rect = Rect::new(x, y, w, 12.0);
    paint_progress_bar(&bar, bar_rect, scene, text_system, theme);
    y += 12.0 + ROW_GAP;

    // Spinner + caption.
    let spin_rect = Rect::new(x, y, 20.0, 20.0);
    paint_spinner(&Spinner::new(NodeId(0), "Loading"), spin_rect, scene, theme);
    paint_left_label(
        scene,
        text_system,
        theme,
        x + 28.0,
        "Loading…",
        w - 28.0,
        y,
        20.0,
    );
    y += 20.0 + ROW_GAP;

    // Tag chips — Accent + Success + Warn.
    let chip_w = 50.0_f32;
    let chip_h = 18.0_f32;
    let gap = 6.0_f32;
    for (i, (label, tone)) in [
        ("PRF", TagTone::Accent),
        ("OK", TagTone::Success),
        ("HOT", TagTone::Warn),
    ]
    .iter()
    .enumerate()
    {
        let cr = Rect::new(x + (chip_w + gap) * i as f32, y, chip_w, chip_h);
        let tag = Tag::new(NodeId(0), *label).tone(*tone);
        paint_tag(&tag, cr, scene, text_system, theme);
    }
    y + chip_h
}

#[allow(clippy::too_many_arguments)]
fn paint_color_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_COLOR,
        ids::INSP_SECTION_COLOR_COLOR,
        "Color",
        1,
    );
    if !open {
        return y;
    }
    let sw_h = 28.0_f32;
    let label_w = 80.0_f32;
    paint_left_label(scene, text_system, theme, x, "Tint", label_w, y, sw_h);
    let sw_size = 32.0_f32;
    let sr = Rect::new(x + w - sw_size, y, sw_size, sw_h);
    hit_index.register(ids::INSP_SAMPLE_SWATCH, sr);
    // Read the swatch's live color from `widget_colors` so the
    // picker's edits propagate back through hero's per-frame
    // mirror. Fallback to the initial purple if nothing was seeded.
    let rgba = store
        .widget_color(ids::INSP_SAMPLE_SWATCH)
        .unwrap_or([120, 60, 200, 255]);
    let mut tint = ColorSwatch::new(ids::INSP_SAMPLE_SWATCH, "Tint", rgba);
    tint.size = SwatchSize::Md;
    paint_color_swatch(&tint, sr, scene, theme);
    y + sw_h
}

#[allow(clippy::too_many_arguments)]
fn paint_actions_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_ACTIONS,
        ids::INSP_SECTION_ACTIONS_COLOR,
        "Actions",
        4,
    );
    if !open {
        return y;
    }
    let btn_h = 30.0_f32;
    let gap = 6.0_f32;
    // Three labelled buttons.
    let trio_w = (w - gap * 2.0) / 3.0;
    let trio = [
        (ids::INSP_SAMPLE_BTN_PRIMARY, "Save", ButtonKind::Accent),
        (
            ids::INSP_SAMPLE_BTN_SECONDARY,
            "Cancel",
            ButtonKind::Default,
        ),
        (ids::INSP_SAMPLE_BTN_DANGER, "Delete", ButtonKind::Danger),
    ];
    for (i, (id, label, kind)) in trio.iter().enumerate() {
        let r = Rect::new(x + (trio_w + gap) * i as f32, y, trio_w, btn_h);
        hit_index.register(*id, r);
        let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(*id, *label).kind(*kind).state(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    y += btn_h + ROW_GAP;

    // Icon button + Tag (removable) on one row.
    let icon_size = 32.0_f32;
    let ir = Rect::new(x, y, icon_size, icon_size);
    hit_index.register(ids::INSP_SAMPLE_BTN_ICON, ir);
    let icon_state = store
        .button_state(ids::INSP_SAMPLE_BTN_ICON)
        .unwrap_or(ButtonState::Normal);
    let icon_btn = Button::new(ids::INSP_SAMPLE_BTN_ICON, "")
        .kind(ButtonKind::IconOnly {
            icon: IconId::Settings,
        })
        .state(icon_state);
    paint_button(&icon_btn, ir, scene, text_system, theme);

    let tag_w = 80.0_f32;
    let tag_h = 22.0_f32;
    let tr = Rect::new(
        x + icon_size + 8.0,
        y + (icon_size - tag_h) * 0.5,
        tag_w,
        tag_h,
    );
    let tag_state = match store.get(ids::INSP_SAMPLE_TAG_REMOVE) {
        Some(InteractiveState::Tag { state }) => *state,
        _ => TagState::Normal,
    };
    let tag = Tag::new(ids::INSP_SAMPLE_TAG_REMOVE, "filter")
        .tone(TagTone::Accent)
        .removable(true)
        .state(tag_state);
    paint_tag(&tag, tr, scene, text_system, theme);
    if let Some(close_r) = tag.close_rect(tr) {
        hit_index.register(ids::INSP_SAMPLE_TAG_REMOVE, close_r);
    }
    y + icon_size
}

#[allow(clippy::too_many_arguments)]
fn paint_identity_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_IDENTITY,
        ids::INSP_SECTION_IDENTITY_COLOR,
        "Identity",
        2,
    );
    if !open {
        return y;
    }
    let size = 36.0_f32;
    let gap = 8.0_f32;
    let circle = Rect::new(x, y, size, size);
    paint_avatar(
        &Avatar::new(NodeId(0), "Enio", 'E'),
        circle,
        scene,
        text_system,
        theme,
    );
    let square = Rect::new(x + size + gap, y, size, size);
    paint_avatar(
        &Avatar::new(NodeId(0), "Player", 'P').shape(AvatarShape::Square),
        square,
        scene,
        text_system,
        theme,
    );
    // Caption to the right.
    paint_left_label(
        scene,
        text_system,
        theme,
        x + size * 2.0 + gap * 2.0,
        "Avatar · circle + square",
        (w - size * 2.0 - gap * 2.0).max(0.0),
        y,
        size,
    );
    let _ = w; // suppress unused
    y + size
}

#[allow(clippy::too_many_arguments)]
fn paint_card_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_CARD,
        ids::INSP_SECTION_CARD_COLOR,
        "Card",
        1,
    );
    if !open {
        return y;
    }
    let card_h = 80.0_f32;
    let r = Rect::new(x, y, w, card_h);
    let card = Card::new(NodeId(0)).title("Quick actions");
    paint_card(&card, r, scene, text_system, theme);
    // Body caption — Card itself doesn't paint into body_rect, so we
    // write a single label line inside it as a static demo.
    let body = card.body_rect(r);
    paint_text(
        text_system,
        scene,
        "Card body slot — host content here.",
        body.x,
        body.y,
        TypeToken::Xs.px(),
        body.w,
        resolve(ColorToken::Text2, theme),
    );
    y + card_h
}

// ── Store accessor helpers ─────────────────────────────────────────────────

fn read_text_input(
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

fn read_combobox(
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

fn read_number_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

/// Find which id in `ids` is the active (`Pressed`) one. Used for
/// segmented button groups (Tabs, RadioGroup) that we model as N
/// independent Button states with exactly one in the Pressed state.
fn active_index(store: &WidgetStore, ids: &[NodeId]) -> Option<usize> {
    for (i, id) in ids.iter().enumerate() {
        if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
            return Some(i);
        }
    }
    None
}
