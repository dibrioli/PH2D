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
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
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
const SECTION_GAP: f32 = 12.0;
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

    // ColorSwatch (decorative — uses Plain so a click can be picked
    // up but no swatch-specific state is needed for the sample).
    store.register(ids::INSP_SAMPLE_SWATCH, InteractiveState::Plain);

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
    let inner_w = rect.w - BODY_PAD * 2.0;
    let body_top_y = content_top - scroll_y + 4.0;
    let mut y = body_top_y;

    // Inline the section sequence so each section gets a colored
    // separator + inter-section gap immediately after. Closures over
    // `&mut store` would conflict with the post-loop write to
    // `set_last_inspector_content_h` and `panel_max_scroll`.
    macro_rules! section {
        ($f:ident) => {
            let new_y = $f(scene, text_system, theme, hit_index, store, inner_x, inner_w, y);
            y = paint_section_separator(scene, theme, inner_x, inner_w, new_y) + SECTION_GAP;
        };
    }
    section!(paint_inputs_section);
    section!(paint_slider_section);
    section!(paint_switches_section);
    section!(paint_lists_section);
    section!(paint_vector_section);
    section!(paint_status_section);
    section!(paint_color_section);
    section!(paint_actions_section);
    section!(paint_identity_section);
    section!(paint_card_section);

    // Publish the total content height for the wheel dispatch to
    // clamp scroll offsets against. `y` is in screen-space WITH the
    // current scroll offset baked in, so `y + scroll_y` is the
    // virtual "bottom of all content" position relative to the
    // unscrolled origin.
    let content_h = (y + scroll_y - body_top_y).max(0.0);
    set_last_inspector_content_h(content_h);

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
    label: &str,
    count: u32,
) -> (f32, bool) {
    let r = Rect::new(x, y, w, SECTION_HEAD_H);
    let is_collapsed = store.is_collapsed(id);
    hit_index.register(id, r);
    let header = SectionHeader::new(id, label)
        .count(count)
        .collapsible(!is_collapsed);
    paint_section_header(&header, r, scene, text_system, theme);
    (y + SECTION_HEAD_H + 4.0, !is_collapsed)
}

/// Discreet colored separator painted at the end of each section's
/// content (when expanded). 2 px tall, `Accent` token, slightly inset
/// horizontally so it doesn't run into the panel's rounded chrome.
/// Skipped when the section is collapsed (the header already ends the
/// section visually).
fn paint_section_separator(
    scene: &mut VectorScene,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let pad_x = 16.0_f32;
    let line = Rect::new(x + pad_x, y + 4.0, (w - pad_x * 2.0).max(0.0), 2.0);
    fill_rounded_rect(scene, line, 1.0, resolve(ColorToken::Accent, theme));
    y + 4.0 + 2.0
}

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
    let mut tint = ColorSwatch::new(ids::INSP_SAMPLE_SWATCH, "Tint", [120, 60, 200, 255]);
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
