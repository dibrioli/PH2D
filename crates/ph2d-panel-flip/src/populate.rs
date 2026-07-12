//! Flip Style panel widget registration (called once at panel install).
//!
//! Registers the FIXED widgets: the three mode buttons (Select/Draw/Erase), the
//! four Brush sliders + chips (Size/Hardness/Opacity/Smoothing), the three Erase
//! sub-mode buttons, and the Layers toolbar (Add/Delete). Each brush slider is
//! seeded at the tool's default so its knob renders correctly before the first
//! drag (the slider is absolute-position — the store value drives the render AND
//! the drag baseline). The per-LAYER row widgets are dynamic (`register_if_absent`
//! at paint time in `paint_layers`); the Stroke swatch needs no store entry (its
//! Down is the generic `is_picker_swatch` dispatch).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_flip::{
    DEFAULT_HARDNESS, DEFAULT_OPACITY, DEFAULT_SMOOTHING, DEFAULT_WIDTH_PX, GAP_MAX_PX, GROW_MAX,
    GROW_MIN, OPACITY_SLIDER_SCALE, PRECISION_MAX, PRECISION_MIN, WIDTH_SLIDER_OFFSET,
    WIDTH_SLIDER_SCALE, px_to_slider,
};

/// Register a plain action Button in the Normal state.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Register a slider + its linked value chip, seeded at `track` / `display`.
fn slider_chip(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
) {
    store.register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: display,
            buffer: format!("{display}"),
            caret: 0,
            last_committed: display,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(slider, chip, scale, offset);
}

/// Registra os widgets fixos do painel Flip.
pub fn populate(store: &mut WidgetStore) {
    // Mode row (Select / Draw / Erase).
    button(store, ids::FLIP_MODE_SELECT);
    button(store, ids::FLIP_MODE_DRAW);
    button(store, ids::FLIP_MODE_ERASE);

    // Brush sliders — seeded at the tool defaults.
    slider_chip(
        store,
        ids::FLIP_SIZE,
        ids::FLIP_SIZE_NUM,
        px_to_slider(DEFAULT_WIDTH_PX),
        DEFAULT_WIDTH_PX,
        WIDTH_SLIDER_SCALE,
        WIDTH_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::FLIP_HARDNESS,
        ids::FLIP_HARDNESS_NUM,
        DEFAULT_HARDNESS,
        f64::from(DEFAULT_HARDNESS),
        1.0, // track 0..1 → 0..1 (identity)
        0.0,
    );
    slider_chip(
        store,
        ids::FLIP_OPACITY,
        ids::FLIP_OPACITY_NUM,
        DEFAULT_OPACITY,
        f64::from(DEFAULT_OPACITY) * 100.0, // LITERAL-PX-OK: fraction→percent display
        OPACITY_SLIDER_SCALE,
        0.0,
    );
    slider_chip(
        store,
        ids::FLIP_SMOOTHING,
        ids::FLIP_SMOOTHING_NUM,
        DEFAULT_SMOOTHING,
        f64::from(DEFAULT_SMOOTHING),
        1.0,
        0.0,
    );

    // Fill (W4): modo, sub-modos do balde e os 3 sliders. Registrados sempre (só
    // PINTADOS no modo Fill) — um widget não-registrado não pode ser clicado.
    button(store, ids::FLIP_MODE_FILL);
    button(store, ids::FLIP_FILL_PAINT);
    button(store, ids::FLIP_FILL_BEHIND);
    button(store, ids::FLIP_FILL_UNPAINT);
    slider_chip(
        store,
        ids::FLIP_GAP,
        ids::FLIP_GAP_NUM,
        0.0,
        0.0,
        GAP_MAX_PX as f32,
        0.0,
    );
    slider_chip(
        store,
        ids::FLIP_GROW,
        ids::FLIP_GROW_NUM,
        // O default (+2px) na faixa [-8, +8].
        ((2.0 - GROW_MIN) / (GROW_MAX - GROW_MIN)) as f32,
        2.0,
        (GROW_MAX - GROW_MIN) as f32,
        GROW_MIN as f32,
    );
    slider_chip(
        store,
        ids::FLIP_PRECISION,
        ids::FLIP_PRECISION_NUM,
        ((1.0 - PRECISION_MIN) / (PRECISION_MAX - PRECISION_MIN)) as f32,
        1.0,
        (PRECISION_MAX - PRECISION_MIN) as f32,
        PRECISION_MIN as f32,
    );

    // Erase sub-mode buttons (painted only in Erase mode, registered always).
    button(store, ids::FLIP_ERASE_SOFT);
    button(store, ids::FLIP_ERASE_HARD);
    button(store, ids::FLIP_ERASE_STROKE);

    // Layers toolbar.
    button(store, ids::FLIP_LAYER_ADD);
    button(store, ids::FLIP_LAYER_DELETE);

    // Close (X) chrome button.
    button(store, ids::FLIP_CLOSE);
}
