//! Audio Editor panel event routing.

use crate::AEDIT_LOOP_BAKE;
use crate::state::AudioEditorState;
use crate::tool_state::{self, EditTool};
use crate::{
    AEDIT_BATCH_LUFS, AEDIT_CLOSE, AEDIT_COPY, AEDIT_CUT, AEDIT_CUTS_CLEAR, AEDIT_DC, AEDIT_EXPORT,
    AEDIT_EXPORT_PIECES, AEDIT_FADE_IN, AEDIT_FADE_OUT, AEDIT_FX_ADD, AEDIT_FX_APPLY,
    AEDIT_FX_BYPASS, AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_NEXT, AEDIT_FX_PARAMS, AEDIT_FX_PREV,
    AEDIT_FX_REMOVE, AEDIT_FX_RESET, AEDIT_FX_STAGE_ONS, AEDIT_FX_STAGES, AEDIT_FX_UP,
    AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_INVERT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_LOOP_CLEAR,
    AEDIT_LOOP_SET, AEDIT_LOOP_XFADE, AEDIT_MARK_ADD, AEDIT_MARK_DEL, AEDIT_MONO, AEDIT_NORM_LUFS,
    AEDIT_NORMALIZE, AEDIT_PASTE, AEDIT_PLAY, AEDIT_PRESET_APPLY, AEDIT_PRESET_LOAD,
    AEDIT_PRESET_NEXT, AEDIT_PRESET_PREV, AEDIT_PRESET_SAVE, AEDIT_REDO, AEDIT_REVERSE,
    AEDIT_SILENCE, AEDIT_SPEC_AMOUNT, AEDIT_SPEC_DENOISE, AEDIT_SPEC_LEARN, AEDIT_SPEC_REPAIR,
    AEDIT_SPEC_VIEW, AEDIT_SPLIT, AEDIT_SPLIT_PLAYHEAD, AEDIT_STOP, AEDIT_TOOL_MOVE,
    AEDIT_TOOL_SCALE, AEDIT_TOOL_SELECT, AEDIT_TRIM, AEDIT_UNDO, AudioEditCmd, AudioEditorPanel,
    loop_state, presets, snapshot, spectral_state, variation_state,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

/// Loop-points clicks (W6). Returns `Some(Consumed)` when `id` is a loop control. Set
/// adopts the SELECTION (refuse without one); Clear needs an existing loop — the panel
/// dims them and the seam refuses too, since a dim is only cosmetic. The loop plays
/// via the transport's Loop toggle + Play, so there is no Audition control here.
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
    None
}

/// Edit-toolbar tool clicks: arm one of the three. A **group**, not three toggles — clicking the
/// armed one re-arms it rather than turning it off, because a pointer that means nothing over the
/// waveform is a pointer that does nothing.
///
/// Move is refused without a second piece to trade places with: the panel dims it, but a dim is
/// cosmetic, and a tool armed with no legal gesture is a mode the user cannot get out of by
/// dragging.
fn tool_click(id: NodeId) -> Option<EventOutcome> {
    let t = if id == AEDIT_TOOL_SELECT {
        EditTool::Select
    } else if id == AEDIT_TOOL_MOVE {
        if tool_state::pieces() < 2 {
            return Some(EventOutcome::Consumed);
        }
        EditTool::Move
    } else if id == AEDIT_TOOL_SCALE {
        EditTool::Scale
    } else {
        return None;
    };
    tool_state::set_tool(t);
    Some(EventOutcome::Consumed)
}

/// Spectral clicks (W5). The view toggle is free; the three tools each need something
/// selected, and **the seam refuses without it** — the panel dims them, but a dim is
/// cosmetic, and the cost of getting this wrong is not a no-op: Learn against a stretch of
/// *signal* would teach the denoiser that the voice is the noise, and then remove it.
fn spectral_click(id: NodeId) -> Option<EventOutcome> {
    if id == AEDIT_SPEC_VIEW {
        spectral_state::toggle_view();
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_SPEC_REPAIR {
        // Needs a time-AND-frequency box, which only exists in the spectrogram.
        if spectral_state::has_band() {
            snapshot::request_edit(AudioEditCmd::SpectralRepair);
        }
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_SPEC_LEARN {
        if snapshot::has_selection() {
            snapshot::request_edit(AudioEditCmd::LearnNoise);
        }
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_SPEC_DENOISE {
        if spectral_state::has_profile() {
            snapshot::request_edit(AudioEditCmd::Denoise);
        }
        return Some(EventOutcome::Consumed);
    }
    None
}

/// W6 asset-prep clicks that arm a `loop_state` intent (Batch LUFS · force-mono toggle ·
/// add / delete marker). Returns `Some(Consumed)` when handled — extracted, like
/// [`loop_click`], to keep `apply_event` under the panel fn-LOC cap.
fn asset_click(id: NodeId) -> Option<EventOutcome> {
    if id == AEDIT_BATCH_LUFS {
        loop_state::request_batch_lufs();
    } else if id == AEDIT_MONO {
        loop_state::request_toggle_mono();
    } else if id == AEDIT_MARK_ADD {
        loop_state::request_add_marker();
    } else if id == AEDIT_MARK_DEL {
        // Delete needs some markers (the panel dims it; the seam refuses it too).
        if loop_state::marker_count() > 0 {
            loop_state::request_del_marker();
        }
    } else {
        return None;
    }
    Some(EventOutcome::Consumed)
}

/// Variation-container clicks (W6). Returns `Some(Consumed)` when `id` is a variation
/// control. Selecting a row, cycling the strategy, and Add / Load are always live; Play
/// / Remove / Weight / Save need a variation to exist (the panel dims them, and — a dim
/// being cosmetic — the seam refuses them too, mirroring the range ops). Extracted, like
/// [`asset_click`], to keep `apply_event` under the panel fn-LOC cap.
fn variation_click(id: NodeId) -> Option<EventOutcome> {
    use crate::{
        AEDIT_VAR_ADD, AEDIT_VAR_ADD_FOLDER, AEDIT_VAR_ENABLED, AEDIT_VAR_LOAD, AEDIT_VAR_PLAY,
        AEDIT_VAR_REMOVE, AEDIT_VAR_ROWS, AEDIT_VAR_SAVE, AEDIT_VAR_STRATEGY_NEXT,
        AEDIT_VAR_STRATEGY_PREV, AEDIT_VAR_WEIGHT_DOWN, AEDIT_VAR_WEIGHT_UP,
    };
    if let Some(i) = AEDIT_VAR_ROWS.iter().position(|r| *r == id) {
        variation_state::select(i);
        return Some(EventOutcome::Consumed);
    }
    let has_any = variation_state::count() > 0;
    if id == AEDIT_VAR_ADD {
        variation_state::request_add();
    } else if id == AEDIT_VAR_ADD_FOLDER {
        variation_state::request_add_folder();
    } else if id == AEDIT_VAR_LOAD {
        variation_state::request_load();
    } else if id == crate::AEDIT_CODEC_PREV {
        crate::delivery_state::cycle_codec(-1);
    } else if id == crate::AEDIT_CODEC_NEXT {
        crate::delivery_state::cycle_codec(1);
    } else if id == AEDIT_VAR_STRATEGY_PREV {
        variation_state::cycle_strategy(-1);
    } else if id == AEDIT_VAR_STRATEGY_NEXT {
        variation_state::cycle_strategy(1);
    } else if id == AEDIT_VAR_PLAY {
        if has_any {
            variation_state::request_play();
        }
    } else if id == AEDIT_VAR_ENABLED {
        // Take the entry out of the pick (or put it back) without deleting it. Needs an entry,
        // like every other per-entry action — and the seam refuses without one, because the
        // panel's dim is cosmetic.
        if has_any {
            variation_state::request_toggle_enabled();
        }
    } else if id == AEDIT_VAR_REMOVE {
        if has_any {
            variation_state::request_remove();
        }
    } else if id == AEDIT_VAR_SAVE {
        if has_any {
            variation_state::request_save();
        }
    } else if id == AEDIT_VAR_WEIGHT_DOWN {
        if has_any {
            variation_state::bump_weight(-1);
        }
    } else if id == AEDIT_VAR_WEIGHT_UP {
        if has_any {
            variation_state::bump_weight(1);
        }
    } else {
        return None;
    }
    Some(EventOutcome::Consumed)
}

/// Map an edit-op / rack-commit button id to its one-shot [`AudioEditCmd`] (`None` for
/// any other id). Extracted from `apply_event` so it stays under the panel fn-LOC cap;
/// the selection guard + `request_edit` stay in `apply_event` (they read snapshot).
fn edit_cmd_for(id: NodeId) -> Option<AudioEditCmd> {
    if id == AEDIT_UNDO {
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
    } else if id == AEDIT_SPLIT {
        Some(AudioEditCmd::SplitAtMarkers)
    } else if id == AEDIT_SPLIT_PLAYHEAD {
        Some(AudioEditCmd::SplitAtPlayhead)
    } else if id == AEDIT_CUTS_CLEAR {
        Some(AudioEditCmd::ClearCuts)
    } else if id == AEDIT_LOOP_BAKE {
        // The panel dims it without a pre-roll, but a dim is cosmetic: a bake on a loop that starts
        // at frame 0 has nothing to fade from and would land a do-nothing step on the undo timeline.
        loop_state::can_bake().then_some(AudioEditCmd::BakeLoopCrossfade)
    } else if id == AEDIT_EXPORT_PIECES {
        // Not an AudioEditCmd: the pieces become FILES, so the shell has to pick a folder. The
        // panel never touches the filesystem.
        snapshot::request_export_pieces();
        None
    } else if id == AEDIT_COPY {
        Some(AudioEditCmd::Copy)
    } else if id == AEDIT_PASTE {
        Some(AudioEditCmd::Paste)
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
    }
}

pub(crate) fn apply_event(
    _state: &mut AudioEditorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // A rapid second click on the same button arrives as `DoubleClick` (the dispatcher
    // upgrades a 2nd Down within 350 ms). NO panel button has double-click semantics —
    // every one is a discrete action — so a `DoubleClick` must behave like a `Click`,
    // else the 2nd press is silently dropped and the op "sometimes does nothing"
    // (2026-07-11 multi-agent audit). Normalizing here covers every branch below,
    // including the `loop_click`/`asset_click`/`variation_click` sub-handlers. Matches
    // `ph2d-panel-motion-graph` / `ph2d-panel-timeline`, which unify `Click|DoubleClick`.
    let ev = match ev {
        WidgetEvent::DoubleClick(id) => WidgetEvent::Click(id),
        other => other,
    };
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
        // W6 asset-prep clicks (Batch LUFS · force-mono toggle · markers) — extracted
        // so `apply_event` stays under the panel fn-LOC cap.
        if let Some(outcome) = asset_click(id) {
            return outcome;
        }
        // Effects-rack chrome (selector · reset · add/remove/move · bypass) — extracted so
        // `apply_event` stays under the panel fn-LOC cap (HR-18). It is the cluster that grows
        // every time the rack does, so it is the one that should own a function.
        if let Some(outcome) = rack_click(id) {
            return outcome;
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
        // Variation containers (W6) — extracted like the above.
        if let Some(outcome) = variation_click(id) {
            return outcome;
        }
        // Spectral (W5) — the view toggle and the two repair tools.
        if let Some(outcome) = tool_click(id) {
            return outcome;
        }
        if let Some(outcome) = spectral_click(id) {
            return outcome;
        }
        // The Convolution Reverb's room. Only reachable when the panel painted the button,
        // which it only does for the stage that needs one.
        if id == crate::AEDIT_FX_LOAD_IR {
            snapshot::request_load_ir();
            return EventOutcome::Consumed;
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
        if let Some(cmd) = edit_cmd_for(id) {
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
    // The denoise Amount slider — the shell reads it when Denoise is clicked.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == AEDIT_SPEC_AMOUNT
    {
        let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
        spectral_state::set_amount(v);
        return EventOutcome::Consumed;
    }
    // The Ogg quality slider — the shell re-prices the asset when it moves.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == crate::AEDIT_OGG_QUALITY
    {
        let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
        crate::delivery_state::set_quality_norm(v);
        return EventOutcome::Consumed;
    }
    // The variation pitch/gain jitter sliders — the shell reads them each frame.
    if let WidgetEvent::ValueChanged(id) = ev
        && (id == crate::AEDIT_VAR_PITCH || id == crate::AEDIT_VAR_GAIN)
    {
        let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
        if id == crate::AEDIT_VAR_PITCH {
            variation_state::set_pitch_norm(v);
        } else {
            variation_state::set_gain_norm(v);
        }
        return EventOutcome::Consumed;
    }
    EventOutcome::Ignored
}

/// Effects-rack chrome clicks: the kind selector, Reset, Add/Remove, Up/Down, and the global
/// Bypass. `None` for any other id.
///
/// Extracted from `apply_event` under the panel fn-LOC cap, and mirrors the `asset_click` /
/// `spectral_click` / `tool_click` handlers beside it.
fn rack_click(id: NodeId) -> Option<EventOutcome> {
    // Effects rack selector: cycle the SELECTED stage's kind. Its parameters are
    // re-seeded with the new effect's neutral defaults, so the stage is a no-op
    // again until a slider moves.
    if id == AEDIT_FX_PREV {
        snapshot::cycle_fx_kind(-1);
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_FX_NEXT {
        snapshot::cycle_fx_kind(1);
        return Some(EventOutcome::Consumed);
    }
    // Reset the SELECTED effect to its neutral defaults (icon beside the name).
    if id == AEDIT_FX_RESET {
        snapshot::reset_fx_params();
        return Some(EventOutcome::Consumed);
    }
    // Chain editing (W3 block 3b). Add/Remove/Up/Down act on the SELECTED stage.
    if id == AEDIT_FX_ADD {
        snapshot::add_fx_stage();
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_FX_REMOVE {
        // The chain never empties (`remove_fx_stage` re-seeds a neutral stage),
        // but the panel dims Remove at one stage — refuse it here too, so the
        // seam agrees with the dim (2026-07-09 audit).
        if snapshot::fx_stage_count() > 1 {
            snapshot::remove_fx_stage();
        }
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_FX_UP {
        snapshot::move_fx_stage(-1);
        return Some(EventOutcome::Consumed);
    }
    if id == AEDIT_FX_DOWN {
        snapshot::move_fx_stage(1);
        return Some(EventOutcome::Consumed);
    }
    // Global A/B: mute the whole chain and hear the dry clip, keeping the chain.
    if id == AEDIT_FX_BYPASS {
        snapshot::toggle_fx_bypass();
        return Some(EventOutcome::Consumed);
    }
    None
}
