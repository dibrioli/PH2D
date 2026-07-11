//! Audio Editor panel event routing.

use crate::state::AudioEditorState;
use crate::{
    AEDIT_CLOSE, AEDIT_CUT, AEDIT_DC, AEDIT_EXPORT, AEDIT_FADE_IN, AEDIT_FADE_OUT, AEDIT_FX_ADD,
    AEDIT_FX_APPLY, AEDIT_FX_BYPASS, AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_NEXT,
    AEDIT_FX_PARAMS, AEDIT_FX_PREV, AEDIT_FX_REMOVE, AEDIT_FX_RESET, AEDIT_FX_STAGE_ONS,
    AEDIT_FX_STAGES, AEDIT_FX_UP, AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_INVERT, AEDIT_LOAD,
    AEDIT_LOOP, AEDIT_LOOP_AUDITION, AEDIT_LOOP_CLEAR, AEDIT_LOOP_SET, AEDIT_LOOP_SNAP,
    AEDIT_LOOP_XFADE, AEDIT_NORM_LUFS, AEDIT_NORMALIZE, AEDIT_PLAY, AEDIT_PRESET_APPLY,
    AEDIT_PRESET_LOAD, AEDIT_PRESET_NEXT, AEDIT_PRESET_PREV, AEDIT_PRESET_SAVE, AEDIT_REDO,
    AEDIT_REVERSE, AEDIT_SILENCE, AEDIT_STOP, AEDIT_TRIM, AEDIT_UNDO, AudioEditCmd,
    AudioEditorPanel, loop_state, presets, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

/// Loop-points clicks (W6). Returns `Some(Consumed)` when `id` is a loop control, so
/// `apply_event` stays under the panel's 200-LOC function cap. Set adopts the
/// SELECTION (refuse without one); Clear/Snap/Audition need an existing loop — the
/// panel dims them and the seam refuses too, since a dim is only cosmetic.
fn loop_click(id: NodeId) -> Option<EventOutcome> {
    if id == AEDIT_LOOP_SET {
        if snapshot::has_selection() {
            loop_state::request_set_loop();
        }
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_LOOP_CLEAR {
        if loop_state::has_loop() {
            loop_state::request_clear_loop();
        }
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_LOOP_SNAP {
        if loop_state::has_loop() {
            loop_state::request_snap_loop();
        }
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_LOOP_AUDITION {
        if loop_state::has_loop() {
            loop_state::toggle_loop_audition();
        }
        return Some(EventOutcome::Consumed);
    }
    None
}

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
        // Effects rack selector: cycle the SELECTED stage's kind. Its parameters are
        // re-seeded with the new effect's neutral defaults, so the stage is a no-op
        // again until a slider moves.
        if id == AEDIT_FX_PREV {
            snapshot::cycle_fx_kind(-1);
            return EventOutcome::Consumed;
        }
        if id == AEDIT_FX_NEXT {
            snapshot::cycle_fx_kind(1);
            return EventOutcome::Consumed;
        }
        // Reset the SELECTED effect to its neutral defaults (icon beside the name).
        if id == AEDIT_FX_RESET {
            snapshot::reset_fx_params();
            return EventOutcome::Consumed;
        }
        // Chain editing (W3 block 3b). Add/Remove/Up/Down act on the SELECTED stage.
        if id == AEDIT_FX_ADD {
            snapshot::add_fx_stage();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_FX_REMOVE {
            // The chain never empties (`remove_fx_stage` re-seeds a neutral stage),
            // but the panel dims Remove at one stage — refuse it here too, so the
            // seam agrees with the dim (2026-07-09 audit).
            if snapshot::fx_stage_count() > 1 {
                snapshot::remove_fx_stage();
            }
            return EventOutcome::Consumed;
        }
        if id == AEDIT_FX_UP {
            snapshot::move_fx_stage(-1);
            return EventOutcome::Consumed;
        }
        if id == AEDIT_FX_DOWN {
            snapshot::move_fx_stage(1);
            return EventOutcome::Consumed;
        }
        // Global A/B: mute the whole chain and hear the dry clip, keeping the chain.
        if id == AEDIT_FX_BYPASS {
            snapshot::toggle_fx_bypass();
            return EventOutcome::Consumed;
        }
        // Chain presets. The selector just browses; Apply/Save/Load arm one-shots the
        // shell drains (it owns the factory table + the effect-name ↔ kind mapping).
        if id == AEDIT_PRESET_PREV {
            presets::cycle_preset(-1);
            return EventOutcome::Consumed;
        }
        if id == AEDIT_PRESET_NEXT {
            presets::cycle_preset(1);
            return EventOutcome::Consumed;
        }
        if id == AEDIT_PRESET_APPLY {
            presets::request_apply_preset();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_PRESET_SAVE {
            presets::request_save_preset();
            return EventOutcome::Consumed;
        }
        if id == AEDIT_PRESET_LOAD {
            presets::request_load_preset();
            return EventOutcome::Consumed;
        }
        // Loop points (W6) — extracted so `apply_event` stays under the panel fn-LOC cap.
        if let Some(outcome) = loop_click(id) {
            return outcome;
        }
        // Chain rows: the eye toggles a stage in/out of the render, the row selects it.
        if let Some(i) = AEDIT_FX_STAGE_ONS.iter().position(|s| *s == id) {
            snapshot::toggle_fx_stage_enabled(i);
            return EventOutcome::Consumed;
        }
        if let Some(i) = AEDIT_FX_STAGES.iter().position(|s| *s == id) {
            snapshot::select_fx_stage(i);
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
        } else if id == AEDIT_FX_APPLY {
            Some(AudioEditCmd::ApplyFx)
        } else if id == AEDIT_FX_CANCEL {
            Some(AudioEditCmd::CancelFx)
        } else {
            None
        };
        if let Some(cmd) = edit {
            // Range ops act on the SELECTION. `target()` silently falls back to the
            // whole clip, so an unguarded Silence with no selection would zero the
            // entire buffer. The panel dims them, but a dim is cosmetic — refuse to
            // arm them here too (2026-07-09 audit).
            let needs_selection = matches!(
                cmd,
                AudioEditCmd::Trim
                    | AudioEditCmd::Cut
                    | AudioEditCmd::Silence
                    | AudioEditCmd::FadeIn
                    | AudioEditCmd::FadeOut
            );
            if needs_selection && !snapshot::has_selection() {
                return EventOutcome::Consumed;
            }
            snapshot::request_edit(cmd);
            return EventOutcome::Consumed;
        }
    }
    // Dragging a parameter slider republishes its normalized position; the shell
    // reformats the readout next frame and reads these on Apply.
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(slot) = AEDIT_FX_PARAMS.iter().position(|p| *p == id)
    {
        let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
        snapshot::set_fx_norm(slot, v);
        return EventOutcome::Consumed;
    }
    // The loop crossfade slider — the shell reads its position to build the audition.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == AEDIT_LOOP_XFADE
    {
        let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
        loop_state::set_xfade_norm(v);
        return EventOutcome::Consumed;
    }
    EventOutcome::Ignored
}
