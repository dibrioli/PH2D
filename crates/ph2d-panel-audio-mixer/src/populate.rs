//! Audio Mixer panel widget registration.

use crate::{AMIX_CLOSE, AMIX_MASTER_MUTE};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn populate(store: &mut WidgetStore) {
    // Close (X) and the Master mute — plain Buttons so the panel's apply_event
    // branch fires on Click (mute state is panel-owned, in the live snapshot).
    store.register(
        AMIX_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    store.register(
        AMIX_MASTER_MUTE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
