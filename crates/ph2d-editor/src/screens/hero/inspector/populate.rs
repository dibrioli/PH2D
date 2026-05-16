//! Inspector `register`-time setup: pre-allocate every retained
//! widget state slot before the first paint pass.
//!
//! Extracted from [`super`] (Track C2). HR-3 (zero alloc on the hot
//! path) requires the [`WidgetStore`] to be sized up-front via
//! `register` — `insert` panics during hot-path use. This module is
//! the one and only place where Inspector widgets are seeded;
//! [`populate`] is called from `HeroScreen::new`.

use super::ids;
use super::{NOTE_BODY_IDS, NOTE_SLOT_IDS, NOTE_TITLE_IDS, SECTION_COLOR_IDS, SECTION_IDS};
use crate::interaction::{InteractiveState, WidgetStore};
use crate::widget::{
    ButtonState, ChannelMode, CheckboxState, CheckboxValue, ComboboxState, DropdownState,
    InterpolationMode, ListItemState, SliderOrientation, SliderState, TagState, TextInputState,
    ToggleState,
};
use ph2d_tokens::ColorValue;

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
/// by [`super::paint_transform_section`]. Identity defaults seed each
/// field (`0` / `0` / `0` / `1` / `1`); the host overwrites these via
/// [`WidgetStore::set_number_value`] when a fresh
/// [`super::super::InspectorTransformInfo`] snapshot lands. Per the
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
            text: "#E7E7E7FF".to_string(), // LITERAL-COLOR-OK: default value shown in the picker's hex input
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
