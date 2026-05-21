//! Padding panel `populate` — pre-registers the panel's widget slots in
//! the `WidgetStore` at host boot (once, via `Panel::populate`). The
//! four edge fields are canonical `NumberInput`s; the host overwrites
//! their values every frame from the live `PaddingUiSnapshot` (the paint
//! seeds the displayed value from the snapshot, falling back to the
//! stored value for in-progress keyboard edits — same contract as the
//! Inspector Transform fields).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

pub fn populate(store: &mut WidgetStore) {
    // Cancel / Apply buttons.
    for id in [ids::PAD_CANCEL, ids::PAD_APPLY] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Four signed per-edge fields, all seeded to 0 (a no-op spec).
    for id in [ids::PAD_TOP, ids::PAD_RIGHT, ids::PAD_BOTTOM, ids::PAD_LEFT] {
        store.register(
            id,
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
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        for id in [ids::PAD_CANCEL, ids::PAD_APPLY] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [ids::PAD_TOP, ids::PAD_RIGHT, ids::PAD_BOTTOM, ids::PAD_LEFT] {
            assert!(
                store.number_value(id).is_some(),
                "number field {id:?} missing"
            );
        }
    }
}
