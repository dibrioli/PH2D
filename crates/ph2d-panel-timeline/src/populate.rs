//! Timeline panel widget registration (called once at panel install).
//!
//! W2.E2 registers the transport row: Play/Pause + Prev/Next-frame buttons, the
//! seconds + frame number chips, and the Loop / Auto-key / Snap toggles. The
//! ruler, track list and key lanes (E3+) register their own ids later; the close
//! (X) button is painted straight on the chrome (hit-index only, not a store
//! widget).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState, ToggleState};

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn toggle(store: &mut WidgetStore, id: ph2d_a11y::NodeId, on: bool) {
    store.register(
        id,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on,
        },
    );
}

fn number(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: "0".to_string(),
            caret: 1,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    button(store, ids::TIMELINE_PREV_FRAME);
    button(store, ids::TIMELINE_PLAY);
    button(store, ids::TIMELINE_NEXT_FRAME);
    number(store, ids::TIMELINE_TIME_NUM);
    number(store, ids::TIMELINE_FRAME_NUM);
    toggle(store, ids::TIMELINE_LOOP, false);
    toggle(store, ids::TIMELINE_AUTOKEY, false);
    toggle(store, ids::TIMELINE_SNAP, true);
}
