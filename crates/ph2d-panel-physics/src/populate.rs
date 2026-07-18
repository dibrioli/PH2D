//! Widget registration — walked from [`crate::rows::rows`], so a painted row
//! cannot be an unregistered one.
//!
//! A widget that `paint` hit-indexes but nobody registers is `is_focusable() ==
//! false`, and its click is dropped **in silence** — no compile error, no
//! warning, just a control that does nothing (the vector-pills class of bug
//! `architecture_panel_wiring_parity` exists to catch). Deriving this list from
//! the same table `paint` walks is how that stops being a thing anyone can
//! forget.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

use crate::rows;

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub fn populate(store: &mut WidgetStore) {
    for row in rows::rows() {
        store.register(
            row.slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            row.chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(row.slider, row.chip, row.scale(), row.offset());
        // ⚠️ The range is registered HERE, and it is not optional: without it
        // the chip derives its drag step from the buffer text and scrubs ~50
        // units per pixel, so one pixel of drag hits the ceiling and the chip
        // becomes a min↔max switch. Typing still works, which is why this class
        // of bug survives review.
        store.set_number_range(row.chip, f64::from(row.min), f64::from(row.max), row.step);
    }

    // Section headers are interactive (the chevron folds them), so they are
    // registered like any other control — a painted chevron that nobody
    // registered is an affordance that does nothing.
    for section in rows::SECTIONS {
        button(store, section.id);
    }
    button(store, ids::PHYSICS_SEC_DEBUG);

    // Commands. Registered as Buttons — including "Show Colliders", which LOOKS
    // like a checkbox and is not one: a `Checkbox` emits `Toggled`, which this
    // panel's `event.rs` does not forward, so it would be registered and dead
    // (the painter-layers sculpt segments carry the same warning).
    button(store, ids::PHYSICS_SHOW_COLLIDERS);
    button(store, ids::PHYSICS_RESET_DEFAULTS);
    button(store, ids::PHYSICS_CLOSE);
}
