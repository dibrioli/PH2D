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
}
