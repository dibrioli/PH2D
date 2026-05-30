//! Inspector panel `populate` — pre-allocates Inspector-only widget
//! state slots in the `WidgetStore`. Called once at host boot via
//! `Panel::populate`.
//!
//! Other widgets historically registered in the same module (gallery
//! showcase samples, blender color picker, hierarchy chrome handles,
//! global context-menu items, scrollbars) remain in
//! `ph2d_editor_core::screens::hero::pre_populate` because they are
//! shared across panels / chrome layers and are not Inspector-specific.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, DropdownState, SliderOrientation, SliderState,
    TextInputState,
};

pub fn populate(store: &mut WidgetStore) {
    populate_transform_editor(store);
    populate_visibility_editor(store);
    populate_render_strategy(store);
    populate_region(store);
    populate_sprite_flip(store);
    populate_color_tint(store);
    populate_sprite_sheet(store);
    populate_name_editor(store);
    populate_ordering(store);
    populate_sampling(store);
}

/// Register the W3 segmented-tab + dropdown-option ids as `Button`s so
/// the pointer dispatcher routes their clicks (an unregistered hit id is
/// rejected by `is_focusable` and never emits `Click`). The selected
/// visual is snapshot-driven via `Tabs::selected`; these states exist
/// purely so the click reaches the event handler. §7 Sort Point tabs,
/// §7 Sorting Layer dropdown options, and the §9 Sampling tabs.
fn register_button_ids(store: &mut WidgetStore, ids: &[ph2d_a11y::NodeId]) {
    for &id in ids {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

fn populate_sampling(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_SAMPLE_FILTER);
    register_button_ids(store, &ids::INSP_SAMPLE_REPEAT);
    register_button_ids(
        store,
        &[
            ids::INSP_ORDER_SP_CENTER,
            ids::INSP_ORDER_SP_PIVOT,
            ids::INSP_ORDER_SP_CUSTOM,
        ],
    );
    register_button_ids(store, &ids::INSP_ORDER_LAYER_OPT);
}

/// W3 Sprite Inspector v2 §7 Ordering / Sorting: 7 toggles + 2 integer
/// NumberInputs. Defaults match the optional-component "absent" state
/// (everything off, Z as Relative on per Godot). Live values sync from
/// the snapshot.
fn populate_ordering(store: &mut WidgetStore) {
    for (id, on) in [
        (ids::INSP_ORDER_Z_OVERRIDE, false),
        (ids::INSP_ORDER_Z_RELATIVE, true),
        (ids::INSP_ORDER_SHOW_BEHIND, false),
        (ids::INSP_ORDER_YSORT_ENABLED, false),
        (ids::INSP_ORDER_SORTING_GROUP, false),
        (ids::INSP_ORDER_SORT_AT_ROOT, false),
        (ids::INSP_ORDER_TOP_LEVEL, false),
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            },
        );
    }
    for id in [
        ids::INSP_ORDER_Z_INDEX,
        ids::INSP_ORDER_ORDER_IN_LAYER,
        ids::INSP_ORDER_AXIS_X,
        ids::INSP_ORDER_AXIS_Y,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    // Sorting Layer dropdown (default = "Default" layer index 2).
    store.register(
        ids::INSP_ORDER_SORTING_LAYER,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(2),
        },
    );
}

/// W2 Sprite Inspector v2 Sprite Sheet grid: Centered toggle (default
/// on) + Offset X/Y (default 0) + HFrames / VFrames (default 1) + Frame
/// (default 0). Live values sync from the snapshot.
fn populate_sprite_sheet(store: &mut WidgetStore) {
    store.register(
        ids::INSP_SPRITE_CENTERED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    for id in [ids::INSP_SPRITE_OFFSET_X, ids::INSP_SPRITE_OFFSET_Y] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    for (id, value) in [
        (ids::INSP_SPRITE_HFRAMES, 1.0_f64),
        (ids::INSP_SPRITE_VFRAMES, 1.0_f64),
        (ids::INSP_SPRITE_FRAME, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format!("{value:.0}"),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
}

/// W2 Sprite Inspector v2 Color & Tint controls: Opacity Slider (0..1
/// storage, default 1.0) with a linked percent chip (0..100), + Tint Fill
/// checkbox (default off). Live values sync from the snapshot.
fn populate_color_tint(store: &mut WidgetStore) {
    // Opacity Slider 0..1 + linked chip showing 0..100 % (spec §3.6).
    store.register(
        ids::INSP_SPRITE_OPACITY,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 1.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SPRITE_OPACITY_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 100.0, // LITERAL-PX-OK: opacity percent scale (1.0 → 100 %), not a design token
            buffer: format_number(100.0), // LITERAL-PX-OK: opacity percent scale
            caret: 0,
            last_committed: 100.0, // LITERAL-PX-OK: opacity percent scale
            selection_anchor: None,
        },
    );
    // chip_display = slider_storage * 100 (+0); integer-snapped so the
    // chip is whole percents while the slider track stays continuous.
    store.link_slider_number_mapped_integer(
        ids::INSP_SPRITE_OPACITY,
        ids::INSP_SPRITE_OPACITY_CHIP,
        100.0, // LITERAL-PX-OK: opacity percent scale (slider 0..1 → chip 0..100)
        0.0,
    );
    store.register(
        ids::INSP_SPRITE_TINT_FILL,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    // Tint / Self Tint + 4 per-corner color swatches. Registered as
    // `Plain` (like the section color-dots in `pre_populate` and
    // grid-snap's swatch) so `is_focusable` is true and the pointer
    // dispatch arms `active` on Down → emits `Click` on Up. Without this
    // leg the click is silently dropped and the picker never opens (the
    // swatch carries no value of its own — its color lives in the
    // `widget_colors` side-table).
    for id in [
        ids::INSP_SPRITE_TINT_SWATCH,
        ids::INSP_SPRITE_SELF_TINT_SWATCH,
        ids::INSP_SPRITE_CORNER_TL,
        ids::INSP_SPRITE_CORNER_TR,
        ids::INSP_SPRITE_CORNER_BL,
        ids::INSP_SPRITE_CORNER_BR,
    ] {
        store.register(id, InteractiveState::Plain);
    }
    // "Equalize corners" button (copies TL → the other three).
    store.register(
        ids::INSP_SPRITE_CORNER_EQUALIZE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // Sub-tabs (spec §3.0 D11): segmented Button group, exactly one
    // `Pressed`. Tint is the default-open sub-tab.
    for (id, pressed) in [
        (ids::INSP_COLOR_TAB_TINT, true),
        (ids::INSP_COLOR_TAB_SELF, false),
        (ids::INSP_COLOR_TAB_CORNER, false),
        (ids::INSP_COLOR_TAB_EFFECTS, false),
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: if pressed {
                    ButtonState::Pressed
                } else {
                    ButtonState::Normal
                },
            },
        );
    }
}

/// W2 Sprite Inspector v2: Flip H / Flip V checkboxes. Default
/// Unchecked (the Sprite default `flip_x = flip_y = false`); the live
/// value is synced from the snapshot each frame in `sync.rs`.
fn populate_sprite_flip(store: &mut WidgetStore) {
    for id in [ids::INSP_SPRITE_FLIP_X, ids::INSP_SPRITE_FLIP_Y] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
}

/// W2 Sprite Inspector v2 — Region sampling (Render Source section,
/// spec §3.3): enable toggle (default off) + 4 px NumberInputs (x/y/w/h,
/// default 0) + filter-clip toggle (default ON, the Atlas anti-bleed
/// default). Live values sync from the snapshot.
fn populate_region(store: &mut WidgetStore) {
    store.register(
        ids::INSP_REGION_ENABLED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    for id in [
        ids::INSP_REGION_X,
        ids::INSP_REGION_Y,
        ids::INSP_REGION_W,
        ids::INSP_REGION_H,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    store.register(
        ids::INSP_REGION_FILTER_CLIP,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

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
    store.register(
        ids::INSP_RENDER_FORMAT_RGBA8,
        InteractiveState::Button {
            state: ButtonState::Pressed,
        },
    );
    store.register(
        ids::INSP_RENDER_FORMAT_RGBA16,
        InteractiveState::Button {
            state: ButtonState::Disabled,
        },
    );
    store.register(
        ids::INSP_RENDER_SOURCE_REIMPORT,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn populate_visibility_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_VISIBILITY_CHECK,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

fn populate_transform_editor(store: &mut WidgetStore) {
    let identity_pairs = [
        (ids::INSP_TRANSFORM_POS_X, 0.0_f64),
        (ids::INSP_TRANSFORM_POS_Y, 0.0_f64),
        (ids::INSP_TRANSFORM_ROT, 0.0_f64),
        (ids::INSP_TRANSFORM_SCALE_X, 1.0_f64),
        (ids::INSP_TRANSFORM_SCALE_Y, 1.0_f64),
        (ids::INSP_TRANSFORM_SKEW_X, 0.0_f64),
        (ids::INSP_TRANSFORM_SKEW_Y, 0.0_f64),
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
