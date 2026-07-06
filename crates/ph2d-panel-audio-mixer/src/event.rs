//! Audio Mixer panel event routing.

use crate::state::AudioMixerState;
use crate::{AMIX_MASTER_MUTE, AudioMixerPanel, snapshot};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    _state: &mut AudioMixerState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let WidgetEvent::Click(id) = ev {
        // TopBar pill toggles the panel (mirrors the Widget Gallery pattern).
        if id == ids::TOPBAR_AUDIO_MIXER {
            let next = !host.panel_visible(AudioMixerPanel::ID);
            host.set_panel_visible(AudioMixerPanel::ID, next);
            return EventOutcome::Consumed;
        }
        if id == AMIX_MASTER_MUTE {
            snapshot::toggle_muted();
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}
