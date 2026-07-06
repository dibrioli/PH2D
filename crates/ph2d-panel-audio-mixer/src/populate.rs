//! Audio Mixer panel widget registration.

use crate::AMIX_MASTER_MUTE;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn populate(store: &mut WidgetStore) {
    // Master mute — a plain Button so the panel's apply_event branch fires on
    // Click (mute state is panel-owned, in the live snapshot). Show/hide is the
    // TopBar `TOPBAR_AUDIO_MIXER` pill, registered by the TopBar populate.
    store.register(
        AMIX_MASTER_MUTE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
