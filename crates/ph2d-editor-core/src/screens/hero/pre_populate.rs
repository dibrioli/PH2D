//! Pre-paint widget registrations that are NOT owned by any single
//! panel.
//!
//! ADR-0029 Phase C.1 split: the legacy `inspector::populate` module
//! historically registered Inspector widgets together with
//! widget-gallery showcase samples, the floating BlenderColorPicker,
//! global context-menu items, scrollbars, and Hierarchy drag/resize
//! handles. After the Inspector panel migrated to `ph2d-panel-inspector`,
//! the Inspector-specific 4 functions moved with it (via
//! `Panel::populate` on the new registry); the rest stays here because
//! it is either shared across panels (showcase samples, picker) or
//! owned by chrome (context menus, scrollbars, Hierarchy handles —
//! the latter follow Hierarchy into its own crate in C.2).

use crate::ids;
use crate::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use crate::widget::showcase::{
    NOTE_BODY_IDS, NOTE_SLOT_IDS, NOTE_TITLE_IDS, RADIO_GROUP_IDS, SECTION_COLOR_IDS,
    TAB_GROUP_IDS, TREE_LEAF_IDS,
};
use crate::widget::{
    ButtonState, ChannelMode, CheckboxState, CheckboxValue, ComboboxState, DropdownState,
    HIERARCHY_SCROLLBAR_ID, INSPECTOR_SCROLLBAR_ID, InterpolationMode, ListItemState,
    SliderOrientation, SliderState, TagState, TextInputState, ToggleState,
};
use ph2d_tokens::ColorValue;

/// Register every shared widget slot the editor needs before the
/// first paint. Called once from `HeroScreen::pre_populate_store`.
pub fn populate_shared(store: &mut WidgetStore) {
    populate_blender_picker(store);
    populate_samples(store);
    populate_global_context_menu(store);
    populate_scrollbars(store);
    populate_hierarchy_chrome(store);
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
        (ids::BLENDER_ADD_SWATCH, BlenderHitKind::AddSwatch),
        (ids::BLENDER_EYEDROPPER, BlenderHitKind::Eyedropper),
        (ids::BLENDER_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::BLENDER_WHEEL, BlenderHitKind::Wheel),
        (ids::BLENDER_VALUE_SLIDER, BlenderHitKind::ValueSlider),
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
                kind: BlenderHitKind::ChannelSlider(idx),
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
            BlenderHitKind::InterpolationLinear,
        ),
        (
            ids::BLENDER_INTERP_PERCEPTUAL,
            BlenderHitKind::InterpolationPerceptual,
        ),
        (ids::BLENDER_CHANNEL_RGB, BlenderHitKind::ChannelRgb),
        (ids::BLENDER_CHANNEL_HSV, BlenderHitKind::ChannelHsv),
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
                kind: BlenderHitKind::PaletteSwatch(swatch_idx),
            },
        );
    }
}

fn populate_samples(store: &mut WidgetStore) {
    store.register(
        ids::INSP_SAMPLE_TEXT,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Player".to_string(),
            caret: 6,
            selection_anchor: None,
        },
    );
    store.register(
        ids::INSP_SAMPLE_TEXTAREA,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Notes about this entity…\nLine two.".to_string(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.mark_multiline_text(ids::INSP_SAMPLE_TEXTAREA);
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
    store.register(
        ids::INSP_SAMPLE_NUMBER,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 42.0, // LITERAL-PX-OK: NumberInput demo seed
            buffer: "42".to_string(),
            caret: 0,
            last_committed: 42.0, // LITERAL-PX-OK: NumberInput demo seed
            selection_anchor: None,
        },
    );
    store.register(
        ids::INSP_SAMPLE_SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.62, // LITERAL-PX-OK: slider demo seed (62%)
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SAMPLE_SLIDER_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.62, // LITERAL-PX-OK: slider demo seed (62%)
            buffer: crate::interaction::format_number(0.62), // LITERAL-PX-OK: slider demo seed (62%)
            caret: 0,
            last_committed: 0.62, // LITERAL-PX-OK: slider demo seed (62%)
            selection_anchor: None,
        },
    );
    store.link_slider_number(ids::INSP_SAMPLE_SLIDER, ids::INSP_SAMPLE_SLIDER_CHIP);

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

    store.register(
        ids::INSP_SAMPLE_DROPDOWN,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );

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

    for (id, v) in [
        (ids::INSP_SAMPLE_V3_X, 1.0_f64),
        (ids::INSP_SAMPLE_V3_Y, 2.0),
        (ids::INSP_SAMPLE_V3_Z, 3.0), // LITERAL-PX-OK: Vector3 demo seed values (1, 2, 3)
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

    store.register(ids::INSP_SAMPLE_SWATCH, InteractiveState::Plain);
    store.set_widget_color(ids::INSP_SAMPLE_SWATCH, [120, 60, 200, 255]);

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

    for id in ids::SECTION_IDS {
        store.register(id, InteractiveState::Plain);
    }
    for id in SECTION_COLOR_IDS {
        store.register(id, InteractiveState::Plain);
    }
    let _ = (RADIO_GROUP_IDS, TAB_GROUP_IDS, TREE_LEAF_IDS); // imports keep namespaces aligned with event.rs

    for id in NOTE_SLOT_IDS {
        store.register(id, InteractiveState::Plain);
    }
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
        store.mark_multiline_text(id);
    }

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

fn populate_global_context_menu(store: &mut WidgetStore) {
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
        ids::CTX_MENU_SETTINGS_PPM,
        ids::CTX_MENU_PPM_16,
        ids::CTX_MENU_PPM_32,
        ids::CTX_MENU_PPM_100,
        ids::CTX_MENU_PPM_256,
        ids::CTX_MENU_PPM_1024,
        ids::CTX_MENU_SETTINGS_UNIT,
        ids::CTX_MENU_UNIT_METERS,
        ids::CTX_MENU_UNIT_PIXELS,
        ids::CTX_MENU_SETTINGS_FILTER,
        ids::CTX_MENU_FILTER_PIXELART,
        ids::CTX_MENU_FILTER_SMOOTH,
        ids::CTX_MENU_SETTINGS_DISPLAY,
        ids::CTX_MENU_DISPLAY_VSYNC,
        ids::CTX_MENU_DISPLAY_IMMEDIATE,
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
}

fn populate_scrollbars(store: &mut WidgetStore) {
    store.register(INSPECTOR_SCROLLBAR_ID, InteractiveState::Plain);
    store.register(HIERARCHY_SCROLLBAR_ID, InteractiveState::Plain);
}

fn populate_hierarchy_chrome(store: &mut WidgetStore) {
    // Inspector chrome (drag/resize handles) registered here while
    // Inspector still consumes INSP_PANEL as its NodeId — these are
    // panel-agnostic infrastructure tied to the panel rect; they
    // move to the panel's own `populate` if/when the panel crate
    // owns its NodeId allocation.
    store.register(
        ids::INSP_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::INSP_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::HIER_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::HIER_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::INSP_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::INSP_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    store.register(
        ids::HIER_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::HIER_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    // Inspector close (X) button. UI canon post-2026-05-24: every
    // floating panel except Hierarchy carries a close X. Registered as
    // a plain Button so the dispatch's is_focusable + apply_click
    // fire on the X's hit rect.
    store.register(
        ids::INSP_CLOSE,
        InteractiveState::Button {
            state: crate::widget::ButtonState::Normal,
        },
    );

    // Inspector live-section color-dot hit ids — registered as Plain
    // so is_focusable returns true and apply_click can fire the
    // section's color-picker handler (Inspector apply_event seeds
    // INSP_BLENDER_PICKER). Same canon Widget Gallery's
    // SECTION_COLOR_IDS use.
    for id in [
        ids::INSP_LIVE_NAME_COLOR,
        ids::INSP_LIVE_VISIBILITY_COLOR,
        ids::INSP_LIVE_TRANSFORM_COLOR,
        ids::INSP_LIVE_RENDER_COLOR,
    ] {
        store.register(id, InteractiveState::Plain);
    }

    // Bottom-LEFT resize handles (post-2026-05-24 chrome canon — every
    // floating panel resizable from EITHER bottom corner).
    store.register(
        ids::INSP_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ids::INSP_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );
    store.register(
        ids::HIER_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ids::HIER_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );

    // Every section header that the Inspector + Widget Gallery paint
    // is collapse-toggle eligible. UI canon post-2026-05-24: every
    // section is collapsible (vide
    // `docs/UI_Padrao/components/section_header.md`). Each id is
    // marked here once; the dispatch flips collapse on left-click.
    for id in [
        // Inspector live sections.
        ids::INSP_LIVE_NAME_SECTION,
        ids::INSP_LIVE_VISIBILITY_SECTION,
        ids::INSP_LIVE_TRANSFORM_SECTION,
        ids::INSP_LIVE_RENDER_SECTION,
        // Widget Gallery showcase sections.
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
    ] {
        store.mark_collapsible_section(id);
    }
}
