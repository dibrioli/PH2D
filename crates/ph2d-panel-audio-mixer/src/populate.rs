//! Audio Mixer panel widget registration.

use crate::fader::FADER_UNITY_POS;
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_DUCK, AMIX_DUCK_DEPTH, AMIX_FADER, AMIX_LIMITER, AMIX_LOWCUT,
    AMIX_MASTER_METER, AMIX_MASTER_MUTE, AMIX_PAN, AMIX_PLAY, AMIX_REVERB, AMIX_REVERB_MIX,
    AMIX_REVERB_SIZE, SUB_FADER, SUB_LOWCUT, SUB_METER, SUB_MUTE, SUB_PAN, SUB_SOLO, SUB_TONE,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState};

pub(crate) fn populate(store: &mut WidgetStore) {
    // Header close (X) + every mute (Master + one per sub-bus) + every sub-bus
    // solo — plain Buttons so the panel's apply_event branch fires on Click.
    // Show/hide is also the TopBar `TOPBAR_AUDIO_MIXER` pill. The dock
    // drag/resize reuse the shared `INSP_*` handles (registered by the
    // Inspector), so they need none here.
    let button = || InteractiveState::Button {
        state: ButtonState::Normal,
    };
    store.register(AMIX_CLOSE, button());
    store.register(AMIX_MASTER_MUTE, button());
    store.register(AMIX_PLAY, button());
    store.register(AMIX_LIMITER, button());
    store.register(AMIX_REVERB, button());
    store.register(AMIX_DUCK, button());
    for id in SUB_MUTE {
        store.register(id, button());
    }
    for id in SUB_SOLO {
        store.register(id, button());
    }
    // Meters are click-to-clear (the latched clip indicator) — register as
    // Buttons so the click dispatches, even though they paint as meters.
    store.register(AMIX_MASTER_METER, button());
    for id in SUB_METER {
        store.register(id, button());
    }

    // Every fader — a vertical Slider starting at the unity position (0 dB, per
    // the dB taper). The shared dispatch maps a drag over its rect to
    // `1.0 - (y - rect.y)/rect.h` and emits `ValueChanged(id)`, which
    // apply_event turns into that strip's gain through the taper.
    let vfader = || InteractiveState::Slider {
        state: SliderState::Normal,
        value: FADER_UNITY_POS,
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

    // Every tone (low-pass cutoff) — a horizontal Slider, open at 1.0. Master's
    // is `AMIX_CUTOFF`; each sub-bus has its own. apply_event log-maps 0..1 to
    // 20 Hz..20 kHz.
    let tone = || InteractiveState::Slider {
        state: SliderState::Normal,
        value: 1.0,
        orientation: SliderOrientation::Horizontal,
    };
    store.register(AMIX_CUTOFF, tone());
    for id in SUB_TONE {
        store.register(id, tone());
    }

    // Every low-cut (high-pass cutoff) — a horizontal Slider, off at 0.0 (full
    // left). Master's is `AMIX_LOWCUT`; each sub-bus has its own. apply_event
    // log-maps 0..1 to 20 Hz..20 kHz (20 Hz = off).
    let lowcut = || InteractiveState::Slider {
        state: SliderState::Normal,
        value: 0.0,
        orientation: SliderOrientation::Horizontal,
    };
    store.register(AMIX_LOWCUT, lowcut());
    for id in SUB_LOWCUT {
        store.register(id, lowcut());
    }

    // Master reverb Size (decay) + Mix (wet/dry) — horizontal Sliders.
    store.register(
        AMIX_REVERB_SIZE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        AMIX_REVERB_MIX,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.3, // LITERAL-PX-OK: default reverb wet/dry mix (audio param)
            orientation: SliderOrientation::Horizontal,
        },
    );

    // Ducking depth — how much Music/SFX drop under the Voice bus.
    store.register(
        AMIX_DUCK_DEPTH,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.7, // LITERAL-PX-OK: default duck depth (audio param)
            orientation: SliderOrientation::Horizontal,
        },
    );
}
