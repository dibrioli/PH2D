//! Timeline panel widget registration (called once at panel install).
//!
//! W2.E2 registers the transport row: Play/Pause + Prev/Next-frame buttons, the
//! seconds + frame number chips, and the Loop / Auto-key / Snap toggles. The
//! ruler, track list and key lanes (E3+) register their own ids later; the close
//! (X) button is painted straight on the chrome (hit-index only, not a store
//! widget).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, DropdownState, SliderOrientation, SliderState, TextInputState, ToggleState,
};

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
    // The ruler is scrubbed as a horizontal slider (1D drag over its strip; the
    // panel paints the ticks/playhead itself and reads the slider value on drag).
    store.register(
        ids::TIMELINE_RULER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    button(store, ids::TIMELINE_GO_START);
    button(store, ids::TIMELINE_PREV_FRAME);
    button(store, ids::TIMELINE_PLAY);
    button(store, ids::TIMELINE_NEXT_FRAME);
    button(store, ids::TIMELINE_GO_END);
    button(store, ids::TIMELINE_ADD_TRACK);
    button(store, ids::TIMELINE_ADD_MARKER);
    for (id, _) in ids::ADDPROP_BUTTONS {
        button(store, id);
    }
    number(store, ids::TIMELINE_TIME_NUM);
    number(store, ids::TIMELINE_FRAME_NUM);
    toggle(store, ids::TIMELINE_LOOP, false);
    toggle(store, ids::TIMELINE_PINGPONG, false);
    toggle(store, ids::TIMELINE_AUTOKEY, false);
    toggle(store, ids::TIMELINE_RECORD, false);
    toggle(store, ids::TIMELINE_SNAP, true);
    toggle(store, ids::TIMELINE_SPEED, false);

    // Clip selector (W5). The chip is a Dropdown (the generic dispatch flips
    // `open` on a click and emits no event — the paint reads it back); each option
    // is a plain Button, which is what makes its Click reach `apply_event`.
    //
    // Every option id is registered, not just the ones a fresh document uses: the
    // store is populated ONCE at install, and a clip added later would otherwise
    // paint an option that was never registered — pintado mas inerte, the exact
    // failure this registration exists to prevent.
    store.register(
        ids::TIMELINE_CLIP_DD,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );
    for id in ids::TIMELINE_CLIP_OPT {
        button(store, id);
    }
    button(store, ids::TIMELINE_ADD_CLIP);
    button(store, ids::TIMELINE_RENAME_CLIP);
    button(store, ids::TIMELINE_DELETE_CLIP);
}
