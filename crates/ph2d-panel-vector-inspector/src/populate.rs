//! Vector inspector widget registration (called once at panel install).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub fn populate(store: &mut WidgetStore) {
    // Close (X) button. The fill swatch needs no store entry — its Down is
    // handled by the generic `is_picker_swatch` dispatch (pointer.rs), which
    // short-circuits before the normal widget-event path.
    store.register_if_absent(
        ph2d_editor_core::ids::VECTOR_INSPECTOR_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // Shape-kind picker options (§4.2). The generic RadioGroup dispatch is not
    // wired yet, so each option is a plain `Button` — its `Click` is mapped to a
    // pending kind-index in `apply_event`, drained by the shell bridge. Registered
    // unconditionally (one-time install); when the Shape tool is inactive `paint`
    // skips the hit-rects so these never receive a `Click`.
    for id in crate::ids::SHAPE_OPTION_IDS {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
