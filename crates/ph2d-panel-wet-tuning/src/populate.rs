//! Widget registration — walked from [`crate::rows`], so a painted row cannot
//! be an unregistered one (an id the store never heard of is not focusable
//! and its click is dropped in silence — the wiring-parity class of bug).

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
        // Track (0..1) ↔ chip (real value): scale/offset from the def range.
        store.link_slider_number_mapped(
            row.slider,
            row.chip,
            (row.max - row.min) as f32,
            row.min as f32,
        );
        // ⚠️ Not optional: without a registered range+step the chip derives
        // its drag step from the buffer text and scrubs ~50 units per pixel
        // (min↔max switch; typing still works, which hides it).
        store.set_number_range(row.chip, row.min, row.max, row.step);
        button(store, row.reset);
    }
    for section in rows::SECTIONS {
        button(store, section.header);
        button(store, section.reset);
    }
    button(store, ids::WET_TUNING_GROUP_HEADERS[5]); // Experimental fold
    button(store, ids::WET_TUNING_PAPER_EYE);
    button(store, ids::WET_TUNING_KM_MIXING);
    button(store, ids::WET_TUNING_KM_GLAZE);
    button(store, ids::WET_TUNING_CLOSE);
}
