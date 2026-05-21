//! Padding panel `populate` — pre-registers the panel's widget slots in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Each edge is a bipolar **slider** (`PAD_*`, normalized `0..1` with
//! `0.5` = neutral) plus a px-valued **NumberInput chip** (`PAD_*_NUM`).
//! They are NOT wired with `link_slider_number` — that helper couples
//! both widgets in the same `0..1` space, but the chip must show / accept
//! PIXELS. Instead [`crate::event`] keeps them in sync manually (slider
//! drag → chip px; chip type → slider track), so the chip stays a true
//! px field while the slider stays a smooth `0..1` track. The host
//! overwrites both every frame from the live `PaddingUiSnapshot`.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub fn populate(store: &mut WidgetStore) {
    // Cancel / Apply + the pivot-mode toggle (a Button painted as an
    // accent toggle, like Bg Removal's Show-Mask).
    for id in [ids::PAD_CANCEL, ids::PAD_APPLY, ids::PAD_PIVOT_RECENTER] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Four bipolar sliders (seeded neutral = 0.5 = 0 px) + their px chips
    // (seeded 0). NO `link_slider_number` — see module docs.
    for (slider_id, chip_id) in [
        (ids::PAD_TOP, ids::PAD_TOP_NUM),
        (ids::PAD_RIGHT, ids::PAD_RIGHT_NUM),
        (ids::PAD_BOTTOM, ids::PAD_BOTTOM_NUM),
        (ids::PAD_LEFT, ids::PAD_LEFT_NUM),
    ] {
        store.register(
            slider_id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip_id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(16);
        populate(&mut store);
        for id in [ids::PAD_CANCEL, ids::PAD_APPLY, ids::PAD_PIVOT_RECENTER] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [ids::PAD_TOP, ids::PAD_RIGHT, ids::PAD_BOTTOM, ids::PAD_LEFT] {
            assert!(store.slider(id).is_some(), "slider {id:?} missing");
        }
        for id in [
            ids::PAD_TOP_NUM,
            ids::PAD_RIGHT_NUM,
            ids::PAD_BOTTOM_NUM,
            ids::PAD_LEFT_NUM,
        ] {
            assert!(store.number_value(id).is_some(), "chip {id:?} missing");
        }
    }
}
