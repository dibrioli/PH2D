//! Audio Mixer panel widget registration.

use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_FADER, AMIX_MASTER_MUTE, AMIX_PAN, SUB_FADER, SUB_MUTE, SUB_PAN,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState};

pub(crate) fn populate(store: &mut WidgetStore) {
    // Header close (X) + every mute (Master + one per sub-bus) — plain Buttons
    // so the panel's apply_event branch fires on Click. Show/hide is also the
    // TopBar `TOPBAR_AUDIO_MIXER` pill. The dock drag/resize reuse the shared
    // `INSP_*` handles (registered by the Inspector), so they need none here.
    let button = || InteractiveState::Button {
        state: ButtonState::Normal,
    };
    store.register(AMIX_CLOSE, button());
    store.register(AMIX_MASTER_MUTE, button());
    for id in SUB_MUTE {
        store.register(id, button());
    }

    // Every fader — a vertical Slider starting at unity. The shared dispatch
    // maps a drag over its rect to `1.0 - (y - rect.y)/rect.h` and emits
    // `ValueChanged(id)`, which apply_event turns into that strip's gain.
    let vfader = || InteractiveState::Slider {
        state: SliderState::Normal,
        value: 1.0,
        orientation: SliderOrientation::Vertical,
    };
    store.register(AMIX_FADER, vfader());
    for id in SUB_FADER {
        store.register(id, vfader());
    }

    // Every pan — a horizontal Slider centered at 0.5 (→ pan 0.0). The shared
    // dispatch maps a drag to the 0..1 value; apply_event remaps to -1..1.
    let pan = || InteractiveState::Slider {
        state: SliderState::Normal,
        value: 0.5,
        orientation: SliderOrientation::Horizontal,
    };
    store.register(AMIX_PAN, pan());
    for id in SUB_PAN {
        store.register(id, pan());
    }

    // Master low-pass cutoff — a horizontal Slider (start open at 1.0).
    store.register(
        AMIX_CUTOFF,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 1.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
}
