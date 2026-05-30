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
    ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, TextInputState,
};

pub fn populate(store: &mut WidgetStore) {
    populate_transform_editor(store);
    populate_visibility_editor(store);
    populate_render_strategy(store);
    populate_sprite_flip(store);
    populate_color_tint(store);
    populate_sprite_sheet(store);
    populate_name_editor(store);
}

/// W2 Sprite Inspector v2 Sprite Sheet grid: HFrames / VFrames (default
/// 1) + Frame (default 0). Live values sync from the snapshot.
fn populate_sprite_sheet(store: &mut WidgetStore) {
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
            value: 100.0, // display space (percent)
            buffer: format_number(100.0),
            caret: 0,
            last_committed: 100.0,
            selection_anchor: None,
        },
    );
    // chip_display = slider_storage * 100 (+0); integer-snapped so the
    // chip is whole percents while the slider track stays continuous.
    store.link_slider_number_mapped_integer(
        ids::INSP_SPRITE_OPACITY,
        ids::INSP_SPRITE_OPACITY_CHIP,
        100.0,
        0.0,
    );
    store.register(
        ids::INSP_SPRITE_TINT_FILL,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    // Tint / Self Tint color swatches. Registered as `Plain` (like the
    // section color-dots in `pre_populate` and grid-snap's swatch) so
    // `is_focusable` is true and the pointer dispatch arms `active` on
    // Down → emits `Click` on Up. Without this leg the click is silently
    // dropped and the picker never opens (the swatch carries no value of
    // its own — its color lives in the `widget_colors` side-table).
    for id in [
        ids::INSP_SPRITE_TINT_SWATCH,
        ids::INSP_SPRITE_SELF_TINT_SWATCH,
    ] {
        store.register(id, InteractiveState::Plain);
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
