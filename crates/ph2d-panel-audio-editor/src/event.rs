//! Audio Editor panel event routing.

use crate::state::AudioEditorState;
use crate::{
    AEDIT_CLOSE, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_PLAY, AEDIT_STOP, AudioEditorPanel,
    snapshot,
};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    _state: &mut AudioEditorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let WidgetEvent::Click(id) = ev {
        // TopBar pill toggles the panel (mirrors the Audio Mixer pattern).
        if id == ids::TOPBAR_AUDIO_EDITOR {
            let next = !host.panel_visible(AudioEditorPanel::ID);
            host.set_panel_visible(AudioEditorPanel::ID, next);
            return EventOutcome::Consumed;
        }
        // Header close (X) hides the dock.
        if id == AEDIT_CLOSE {
            host.set_panel_visible(AudioEditorPanel::ID, false);
            return EventOutcome::Consumed;
        }
        if id == AEDIT_PLAY {
            snapshot::request_play_pause();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_STOP {
            snapshot::request_stop();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_LOOP {
            snapshot::toggle_looping();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_LOAD {
            snapshot::request_load();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_EXPORT {
            snapshot::request_export();
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}
