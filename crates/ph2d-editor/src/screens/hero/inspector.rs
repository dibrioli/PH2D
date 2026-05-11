//! Inspector painter — gallery of canonical widget samples.
//!
//! The placeholder fixture (fake Player params, Behavior section)
//! was retired in the showcase teardown commit. This panel now hosts
//! exactly one instance of each widget in [`crate::widget`], wired
//! through the [`WidgetStore`] so the user can interact with samples
//! and visually verify the standardized chrome.
//!
//! The floating [`crate::widget::BlenderColorPicker`] (handled by
//! [`super::color_picker_demo`]) is not duplicated here — its
//! retained state still lives on this panel under
//! [`ids::INSP_BLENDER_PICKER`], which is registered in
//! [`populate`].
//!
//! Layout: every section uses [`crate::widget::SectionHeader`] as a
//! divider; rows below are widget samples padded inside the panel
//! body. Body is wrapped in a `push_clip` so wheel-scroll can shift
//! the entire content cursor up/down without painting outside the
//! panel chrome.

use super::HeroLayout;
use super::HeroSelection;
use super::ids;
use super::style::{PANEL_HEAD_PAD, paint_panel_surface};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, NoteData, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve};
use crate::widget::{
    Avatar, AvatarShape, Button, ButtonKind, ButtonState, Card, ChannelMode, Checkbox,
    CheckboxState, CheckboxValue, ColorSwatch, Combobox, ComboboxOption, ComboboxState,
    DropdownState, InterpolationMode, ListItem, ListItemState, NumberInput, ProgressBar,
    RadioGroup, RadioOption, RadioOrientation, SectionHeader, SliderOrientation, SliderState,
    Spinner, SwatchSize, TabItem, Tabs, TabsVariant, Tag, TagState, TagTone, TextArea, TextInput,
    TextInputState, Toggle, ToggleState, TreeNode, TreeView, Vector3Editor, paint_avatar,
    paint_button, paint_card, paint_checkbox, paint_color_swatch, paint_combobox_with_state,
    paint_list_item, paint_number_input_with_buffer, paint_progress_bar,
    paint_radio_group_with_labels, paint_section_header, paint_slider_with_chip,
    paint_spinner, paint_tabs, paint_tag, paint_text_area_with_state,
    paint_text_input_with_buffer, paint_toggle, paint_tree_view,
};
use crate::widget::Dropdown;
use crate::widget::DropdownOption;
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
const TREE_LEAF_IDS: [ph2d_a11y::NodeId; 2] = [
    ids::INSP_SAMPLE_TREE_LEAF_A,
    ids::INSP_SAMPLE_TREE_LEAF_B,
];
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

/// Find the section index whose body the given body-relative y
/// lies INSIDE. Returns `Some(i)` so callers know a new note
/// should be inserted above `SECTION_IDS[i]` (i.e. above the
/// section the user right-clicked into). Returns `None` when y is
/// past the last section's content (note appends to the bottom).
///
/// Previous version returned "the first section whose top > y" —
/// which for clicks INSIDE section A returned the index of
/// section B, so the new note went BELOW A's separator instead of
/// above A's header (user's "nota foi criada abaixo do separador
/// da sessão onde o componente foi escolhido").
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
        (ids::BLENDER_ADD_SWATCH, crate::interaction::BlenderHitKind::AddSwatch),
        (ids::BLENDER_EYEDROPPER, crate::interaction::BlenderHitKind::Eyedropper),
        (ids::BLENDER_DRAG_HANDLE, crate::interaction::BlenderHitKind::DragHandle),
        (ids::BLENDER_WHEEL, crate::interaction::BlenderHitKind::Wheel),
        (ids::BLENDER_VALUE_SLIDER, crate::interaction::BlenderHitKind::ValueSlider),
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
        (ids::BLENDER_INTERP_LINEAR, crate::interaction::BlenderHitKind::InterpolationLinear),
        (ids::BLENDER_INTERP_PERCEPTUAL, crate::interaction::BlenderHitKind::InterpolationPerceptual),
        (ids::BLENDER_CHANNEL_RGB, crate::interaction::BlenderHitKind::ChannelRgb),
        (ids::BLENDER_CHANNEL_HSV, crate::interaction::BlenderHitKind::ChannelHsv),
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
    // Scrollbar thumb hits — must be in the store so `is_focusable`
    // returns true and the dispatch's `set_active` block runs.
    // Without this the Down handler skips the scrollbar-drag-
    // anchor seed and the drag never starts (user's "não está
    // arrastando com o mouse").
    store.register(crate::widget::INSPECTOR_SCROLLBAR_ID, InteractiveState::Plain);
    store.register(crate::widget::HIERARCHY_SCROLLBAR_ID, InteractiveState::Plain);

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
                    crate::interaction::ContextMenuKind::NoteBackground {
                        panel,
                        note_index,
                    } => {
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
        if SECTION_COLOR_IDS.iter().any(|c| *c == id) || id == ids::INSP_SAMPLE_SWATCH {
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
        if SECTION_IDS.iter().any(|s| *s == id) {
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
        if TREE_LEAF_IDS.iter().any(|l| *l == id) {
            pin_button_selection(store, id, &TREE_LEAF_IDS);
            return true;
        }
        // Radio group selection lock — pin clicked, clear siblings.
        if RADIO_GROUP_IDS.iter().any(|r| *r == id) {
            pin_button_selection(store, id, &RADIO_GROUP_IDS);
            return true;
        }
        // Tabs sample — same shape, different ids.
        if TAB_GROUP_IDS.iter().any(|t| *t == id) {
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

    // Header: title + subtitle + divider line.
    let title = selection
        .map(|s| s.label.as_str())
        .unwrap_or("(no selection)");
    let title_y = rect.y + 18.0;
    paint_text(
        text_system,
        scene,
        title,
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        text_system,
        scene,
        "Widget samples",
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    let div_y = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 16.0;
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
    let mut y = body_top_y;
    // Capture each section's body-relative top y so the right-click
    // dispatch can compute `before_section` for new notes (the user
    // wants notes inserted ABOVE the section the right-click landed
    // in, not at the bottom).
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(SECTION_IDS.len());
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + 4.0));

    // Inline the section sequence so each section gets a colored
    // separator + inter-section gap immediately after. Closures over
    // `&mut store` would conflict with the post-loop write to
    // `set_last_inspector_content_h` and `panel_max_scroll`.
    // Walk notes once, partitioning by `before_section`. A note
    // with `before_section: Some(i)` paints just before
    // `SECTION_IDS[i]`; notes with `None` queue for the tail.
    // Each note also carries its original index in the panel's
    // notes list (the "slot") so the painter can register the
    // right `NOTE_SLOT_IDS[slot]` hit — which is what the
    // right-click dispatch then maps to `note_index` on the
    // `NoteBackground` context menu.
    let all_notes = store.notes_for_panel(ids::INSP_PANEL).to_vec();
    let mut notes_per_section: [Vec<(usize, &NoteData)>; 10] = Default::default();
    let mut trailing_notes: Vec<(usize, &NoteData)> = Vec::new();
    for (idx, note) in all_notes.iter().enumerate() {
        match note.before_section {
            Some(i) if (i as usize) < notes_per_section.len() => {
                notes_per_section[i as usize].push((idx, note));
            }
            _ => trailing_notes.push((idx, note)),
        }
    }
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
    macro_rules! section {
        ($f:ident, $section_id:expr) => {
            paint_pending_notes!();
            let y_before = y;
            push_section_top_y(&mut section_tops_y, y_before - body_top_y);
            let new_y = $f(scene, text_system, theme, hit_index, store, inner_x, inner_w, y);
            // Outline: when the user picked an outline color via the
            // right-click menu on this section header, stroke a
            // colored rect spanning the header + body before the
            // separator. Indexes the 5-color highlighter palette.
            if let Some(color_idx) = store.section_outline_color($section_id) {
                let rgba = crate::screens::hero::context_menu_overlay::HIGHLIGHTER_RGBA
                    [color_idx.min(4) as usize];
                // Inflate the block by 4 px so the outline doesn't
                // hug the section content — gives the highlight a
                // breathing margin (user's "outline precisa de
                // padding"). The stroke sits OUTSIDE the section's
                // hit rects so it never intercepts clicks.
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
            // The last increment is unused (no section after Card),
            // but bumping unconditionally keeps the loop body
            // uniform. Allow the warning here.
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
    // Trailing notes (those without an explicit anchor).
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

    // Publish the total content height + the EXACT visible body
    // height for the wheel dispatch + hero clamp. visible_h must
    // match the actual content viewport, not a rough panel.h - 60
    // heuristic — the latter overestimated by ~20-30 px and
    // prevented scrolling to reach the last note (user's
    // "limite do scroll não se adaptou a nota nova").
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
}

// ── Helpers ────────────────────────────────────────────────────────────────

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
    let rgba = store.widget_color(color_id).unwrap_or([0x88, 0x88, 0x88, 0xFF]);
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
    let pad = 8.0_f32;
    let title_h = TypeToken::Sm.px() + 8.0;
    let body_h = TypeToken::Xs.px() * 3.0 + 12.0; // ~3 lines worth
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
            TypeToken::Sm.px(),
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
            TypeToken::Xs.px(),
            dark,
            "Notes…",
        );
    }
    *y += note_h + 8.0;
}

/// Paint an editable single-line text field with no chrome —
/// dark glyphs + caret + selection on a transparent background.
/// Reads the TextInput state at `id` from the store. Used by the
/// note painter so the colored note bg shows through.
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
        let prefix_w = if s == 0 {
            0.0
        } else {
            text_system.layout(&text[..s], font_size, f32::INFINITY).width()
        };
        let mid_w = if s == e {
            0.0
        } else {
            text_system
                .layout(&text[s..e], font_size, f32::INFINITY)
                .width()
        };
        let sel = Rect::new(
            rect.x + prefix_w,
            rect.y + 2.0,
            mid_w.min(rect.w - prefix_w),
            (rect.h - 4.0).max(2.0),
        );
        // Translucent dark wash for the selection so the highlighter
        // bg still shows through.
        let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33);
        fill_rounded_rect(scene, sel, 1.0, sel_color);
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
    paint_text(text_system, scene, displayed, rect.x, text_y, font_size, rect.w, display_color);
    // Caret — only when focused.
    if focused {
        let caret_byte = caret.min(text.len());
        let prefix_w = if caret_byte == 0 {
            0.0
        } else {
            text_system
                .layout(&text[..caret_byte], font_size, f32::INFINITY)
                .width()
        };
        let caret_rect = Rect::new(
            (rect.x + prefix_w).min(rect.x + rect.w),
            rect.y + 2.0,
            1.5,
            (rect.h - 4.0).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg);
    }
}

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
    let (state, text, caret, _anchor) = read_text_input(store, id);
    let focused = state == TextInputState::Focused;
    let line_h = font_size + 3.0;
    if text.is_empty() && !focused {
        paint_text(
            text_system,
            scene,
            placeholder,
            rect.x,
            rect.y,
            font_size,
            rect.w,
            ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80),
        );
        return;
    }
    for (i, line) in text.split('\n').enumerate() {
        paint_text(
            text_system,
            scene,
            line,
            rect.x,
            rect.y + i as f32 * line_h,
            font_size,
            rect.w,
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
        let prefix_w = if local == 0 {
            0.0
        } else {
            text_system
                .layout(&line_text[..local], font_size, f32::INFINITY)
                .width()
        };
        let caret_rect = Rect::new(
            (rect.x + prefix_w).min(rect.x + rect.w),
            rect.y + line_idx as f32 * line_h,
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
fn paint_section_separator(
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

/// Vertical breathing room above AND below the section separator
/// line — same on both sides so the colored pill reads as centered
/// between two sections rather than glued to the previous one.
const SEPARATOR_PAD_Y: f32 = 8.0;

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
    let (ta_state, ta_text, ta_caret, ta_anchor) = read_text_input(store, ids::INSP_SAMPLE_TEXTAREA);
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
    paint_left_label(scene, text_system, theme, x, "Snap to grid", w - toggle_w - 6.0, y, row_h);
    y += row_h + ROW_GAP;

    // Segmented RadioGroup. We use the per-tab pressed-button trick
    // (mirrors the Inspector-tabs pattern in `paint_inspector_tabs`
    // pre-cleanup) so selection survives across frames without
    // adding a typed RadioGroup state to the store.
    let selected = active_index(store, &[
        ids::INSP_SAMPLE_RADIO_A,
        ids::INSP_SAMPLE_RADIO_B,
        ids::INSP_SAMPLE_RADIO_C,
    ])
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
    let selected = active_index(store, &[
        ids::INSP_SAMPLE_TAB_A,
        ids::INSP_SAMPLE_TAB_B,
        ids::INSP_SAMPLE_TAB_C,
    ])
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
                    TreeNode::new(ids::INSP_SAMPLE_TREE_LEAF_A, "Item A")
                        .icon(IconId::Sprite),
                    TreeNode::new(ids::INSP_SAMPLE_TREE_LEAF_B, "Item B")
                        .icon(IconId::Sprite),
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
        (ids::INSP_SAMPLE_BTN_SECONDARY, "Cancel", ButtonKind::Default),
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
    let tr = Rect::new(x + icon_size + 8.0, y + (icon_size - tag_h) * 0.5, tag_w, tag_h);
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
