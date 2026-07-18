//! Timeline panel widget registration (called once at panel install).
//!
//! W2.E2 registers the transport row: Play/Pause + Prev/Next-frame buttons, the
//! seconds + frame number chips, and the Loop / Auto-key / Snap toggles. The
//! ruler, track list and key lanes (E3+) register their own ids later.
//!
//! **The close (X) is a store widget too** (Enio, 2026-07-16). It used to be painted
//! straight on the chrome — hit-index only — and that made it a DEAD button: the
//! chrome painted it, the hit index found it, `event.rs` had the handler written and
//! waiting, and no `Click` was ever produced to reach it, because `dispatch` only
//! raises one for a widget the store knows. Registered ≠ dispatched, the same bug the
//! rail's Undo/Redo had. The sibling panels (`painter-layers`, `audio-mixer`) register
//! theirs; the timeline was the odd one out.

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
    // The X. Its handler has always been in `event.rs`; this line is what lets a
    // click reach it.
    button(store, ids::TIMELINE_CLOSE);
    button(store, ids::TIMELINE_GO_START);
    button(store, ids::TIMELINE_PREV_FRAME);
    button(store, ids::TIMELINE_PLAY);
    button(store, ids::TIMELINE_NEXT_FRAME);
    button(store, ids::TIMELINE_GO_END);
    button(store, ids::TIMELINE_ADD_TRACK);
    // The clip stack (ADR-0115). EVERY lane slot is registered, not just the ones a
    // fresh document uses: the store is populated ONCE at install, and a lane added
    // later would otherwise paint a button that was never registered — painted but
    // inert, the same trap the clip options already document.
    button(store, ids::TIMELINE_ADD_LANE);
    for id in ids::TIMELINE_LANE_MUTE {
        button(store, id);
    }
    for id in ids::TIMELINE_LANE_ADD_STRIP {
        button(store, id);
    }
    button(store, ids::TIMELINE_ADD_MARKER);
    for (id, _) in ids::ADDPROP_BUTTONS {
        button(store, id);
    }
    number(store, ids::TIMELINE_TIME_NUM);
    number(store, ids::TIMELINE_FRAME_NUM);
    // Every lane's weight field, not just the lanes a fresh document has: the
    // store is populated ONCE at install, so a lane added later would paint a
    // field that was never registered — pintado mas inerte (the same reason every
    // clip option below is registered up front).
    for id in ids::TIMELINE_LANE_WEIGHT {
        number(store, id);
    }
    // The lane LABEL surfaces (the right-click target). The slot has to exist
    // before the first paint; the paint then replaces each with the
    // `TimelineSurface` that carries its lane index.
    for id in ids::TIMELINE_LANE_ROW {
        store.register(id, InteractiveState::Plain);
    }
    toggle(store, ids::TIMELINE_LOOP, false);
    toggle(store, ids::TIMELINE_PINGPONG, false);
    toggle(store, ids::TIMELINE_PHYSICS, false);
    toggle(store, ids::TIMELINE_AUTOKEY, false);
    toggle(store, ids::TIMELINE_RECORD, false);
    toggle(store, ids::TIMELINE_SNAP, true);
    toggle(store, ids::TIMELINE_SPEED, false);

    // The view tabs. Each is a plain Button — which is what makes its Click reach
    // `apply_event` at all: `dispatch_pointer`'s Down only makes a hit ACTIVE when
    // it is focusable, and an id absent from the store is not. Paint it, register
    // its rect, route it in `event.rs` — every gate green — and it is still stone
    // dead under the mouse without this line
    // ([[feedback_widget_is_done_when_a_test_clicks_it]]).
    //
    // Walks `tab::TABS`, the same table the strip paints and the router matches, so
    // a third tab is a row there and nothing else.
    for (id, _) in crate::tab::TABS {
        button(store, id);
    }
    button(store, ids::TIMELINE_DUP_CLIP);
    button(store, ids::TIMELINE_REVERSE_CLIP);

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
