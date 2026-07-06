//! Audio Mixer panel widget registration.

use crate::{AMIX_CLOSE, AMIX_FADER, AMIX_MASTER_MUTE};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState};

pub(crate) fn populate(store: &mut WidgetStore) {
    // Header close (X) + Master mute — plain Buttons so the panel's apply_event
    // branch fires on Click. Show/hide is also the TopBar `TOPBAR_AUDIO_MIXER`
    // pill. The dock drag/resize reuse the shared `INSP_*` handles (registered
    // by the Inspector), so they need no registration here.
    for id in [AMIX_CLOSE, AMIX_MASTER_MUTE] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Master fader — a vertical Slider (start at unity). The shared dispatch
    // maps a drag over its rect to `1.0 - (y - rect.y)/rect.h` and emits
    // `ValueChanged(AMIX_FADER)`, which apply_event turns into the master gain.
    store.register(
        AMIX_FADER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 1.0,
            orientation: SliderOrientation::Vertical,
        },
    );
}
