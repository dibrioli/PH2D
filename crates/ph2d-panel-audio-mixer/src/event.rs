//! Audio Mixer panel event routing.

use crate::state::AudioMixerState;
use crate::{AMIX_CLOSE, AMIX_FADER, AMIX_MASTER_MUTE, AudioMixerPanel, snapshot};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    _state: &mut AudioMixerState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) => {
            // TopBar pill toggles the panel (mirrors the Widget Gallery pattern).
            if id == ids::TOPBAR_AUDIO_MIXER {
                let next = !host.panel_visible(AudioMixerPanel::ID);
                host.set_panel_visible(AudioMixerPanel::ID, next);
                return EventOutcome::Consumed;
            }
            // Header close (X) hides the dock.
            if id == AMIX_CLOSE {
                host.set_panel_visible(AudioMixerPanel::ID, false);
                return EventOutcome::Consumed;
            }
            if id == AMIX_MASTER_MUTE {
                snapshot::toggle_muted();
                return EventOutcome::Consumed;
            }
        }
        // Master fader dragged — the shared slider dispatch already wrote the
        // new value into the store; publish it as the master gain for the shell.
        WidgetEvent::ValueChanged(id) if id == AMIX_FADER => {
            let gain = host.store().slider(AMIX_FADER).map(|(_, v)| v).unwrap_or(1.0);
            snapshot::set_master_gain(gain);
            return EventOutcome::Consumed;
        }
        _ => {}
    }
    EventOutcome::Ignored
}
