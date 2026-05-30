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
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};

pub fn populate(store: &mut WidgetStore) {
    populate_transform_editor(store);
    populate_visibility_editor(store);
    populate_render_strategy(store);
    populate_sprite_flip(store);
    populate_name_editor(store);
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
