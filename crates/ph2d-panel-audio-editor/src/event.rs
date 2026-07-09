//! Audio Editor panel event routing.

use crate::state::AudioEditorState;
use crate::{
    AEDIT_BITCRUSH, AEDIT_CLOSE, AEDIT_COMPRESS, AEDIT_CUT, AEDIT_DC, AEDIT_EXPORT, AEDIT_FADE_IN,
    AEDIT_FADE_OUT, AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_HIGHPASS, AEDIT_INVERT, AEDIT_LOAD,
    AEDIT_LOOP, AEDIT_LOWPASS, AEDIT_NORM_LUFS, AEDIT_NORMALIZE, AEDIT_PLAY, AEDIT_REDO,
    AEDIT_REVERSE, AEDIT_SATURATE, AEDIT_SILENCE, AEDIT_STOP, AEDIT_TRIM, AEDIT_UNDO, AEDIT_WIDEN,
    AudioEditCmd, AudioEditorPanel, snapshot,
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
        // Edit ops → arm the matching one-shot command for the shell.
        let edit = if id == AEDIT_UNDO {
            Some(AudioEditCmd::Undo)
        } else if id == AEDIT_REDO {
            Some(AudioEditCmd::Redo)
        } else if id == AEDIT_NORMALIZE {
            Some(AudioEditCmd::NormalizePeak)
        } else if id == AEDIT_NORM_LUFS {
            Some(AudioEditCmd::NormalizeLufs)
        } else if id == AEDIT_REVERSE {
            Some(AudioEditCmd::Reverse)
        } else if id == AEDIT_DC {
            Some(AudioEditCmd::RemoveDc)
        } else if id == AEDIT_INVERT {
            Some(AudioEditCmd::Invert)
        } else if id == AEDIT_GAIN_DOWN {
            Some(AudioEditCmd::GainDown)
        } else if id == AEDIT_GAIN_UP {
            Some(AudioEditCmd::GainUp)
        } else if id == AEDIT_TRIM {
            Some(AudioEditCmd::Trim)
        } else if id == AEDIT_CUT {
            Some(AudioEditCmd::Cut)
        } else if id == AEDIT_SILENCE {
            Some(AudioEditCmd::Silence)
        } else if id == AEDIT_FADE_IN {
            Some(AudioEditCmd::FadeIn)
        } else if id == AEDIT_FADE_OUT {
            Some(AudioEditCmd::FadeOut)
        } else if id == AEDIT_LOWPASS {
            Some(AudioEditCmd::LowPass)
        } else if id == AEDIT_HIGHPASS {
            Some(AudioEditCmd::HighPass)
        } else if id == AEDIT_COMPRESS {
            Some(AudioEditCmd::Compress)
        } else if id == AEDIT_SATURATE {
            Some(AudioEditCmd::Saturate)
        } else if id == AEDIT_BITCRUSH {
            Some(AudioEditCmd::Bitcrush)
        } else if id == AEDIT_WIDEN {
            Some(AudioEditCmd::StereoWiden)
        } else {
            None
        };
        if let Some(cmd) = edit {
            snapshot::request_edit(cmd);
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}
