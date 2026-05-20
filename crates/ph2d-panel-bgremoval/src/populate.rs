//! Background-Removal panel `populate` — pre-registers the panel's
//! widget slots in the `WidgetStore` at host boot (once, via
//! `Panel::populate`). Initial slider values are placeholders; the host
//! overwrites them every frame from the live `BgRemovalUiSnapshot` (the
//! paint reads the snapshot, not the stored slider value, for track
//! position — the stored value is what dispatch mutates on drag and what
//! [`crate::event`] reads on `ValueChanged`).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState};

pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::BGR_MODE_CHROMA,
        ids::BGR_MODE_GRABCUT,
        ids::BGR_APPLY,
        ids::BGR_CANCEL,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for (id, value) in [
        (ids::BGR_TOLERANCE, 0.10 / 0.30),
        (ids::BGR_FEATHER, 0.04 / 0.20),
        (ids::BGR_REFINE, 30.0 / 100.0),
    ] {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        // Buttons.
        for id in [ids::BGR_MODE_CHROMA, ids::BGR_MODE_GRABCUT, ids::BGR_APPLY] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        // Sliders with their seeded normalized values.
        for (id, expect) in [
            (ids::BGR_TOLERANCE, 0.10 / 0.30),
            (ids::BGR_FEATHER, 0.04 / 0.20),
            (ids::BGR_REFINE, 0.30),
        ] {
            let (_, v) = store.slider(id).expect("slider registered");
            assert!((v - expect).abs() < 1e-5, "slider {id:?}: {v} vs {expect}");
        }
    }
}
