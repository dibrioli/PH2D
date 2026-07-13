//! Behavioral SEAM test for the Audio Editor panel (DIRETIVA §5 DoD).
//!
//! Unit tests prove `snapshot` round-trips and `populate` registers widgets —
//! but NEITHER proves the `event.rs` wire from a real `WidgetEvent` to the
//! transport intent is intact. A forgotten arm leaves a button painted,
//! clickable and SILENTLY DEAD while every unit test stays green. This drives
//! the full path the shell runs, headless: apply_event → snapshot intent.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_panel_audio_editor::loop_state;
use ph2d_panel_audio_editor::state::AudioEditorState;
use ph2d_panel_audio_editor::tool_state::{self, EditTool};
use ph2d_panel_audio_editor::{
    AEDIT_BATCH_LUFS, AEDIT_COPY, AEDIT_CUT, AEDIT_CUTS_CLEAR, AEDIT_EXPORT_PIECES, AEDIT_FADE_IN,
    AEDIT_FX_ADD, AEDIT_FX_APPLY, AEDIT_FX_BYPASS, AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_NEXT,
    AEDIT_FX_P0, AEDIT_FX_PREV, AEDIT_FX_REMOVE, AEDIT_FX_RESET, AEDIT_FX_S0_ON, AEDIT_FX_S1,
    AEDIT_FX_UP, AEDIT_LOAD, AEDIT_LOOP, AEDIT_LOOP_BAKE, AEDIT_LOOP_CLEAR, AEDIT_LOOP_SET,
    AEDIT_MARK_ADD, AEDIT_MARK_DEL, AEDIT_MONO, AEDIT_NORMALIZE, AEDIT_PASTE, AEDIT_PLAY,
    AEDIT_PRESET_APPLY, AEDIT_PRESET_LOAD, AEDIT_PRESET_NEXT, AEDIT_PRESET_PREV, AEDIT_PRESET_SAVE,
    AEDIT_SILENCE, AEDIT_SPLIT, AEDIT_SPLIT_PLAYHEAD, AEDIT_STOP, AEDIT_TOOL_MOVE,
    AEDIT_TOOL_SCALE, AEDIT_TOOL_SELECT, AEDIT_TRIM, AudioEditCmd, AudioEditorPanel, MAX_FX_STAGES,
    clear_fx_dirty, fx_bypass, fx_chain, fx_dirty, fx_sel, fx_sel_stage, looping, preset_sel,
    reset_fx_chain, set_fx_kind_defaults, set_fx_kind_names, set_has_clipboard, set_has_selection,
    set_loop_span, set_marker_count, set_preset_names, take_add_marker, take_apply_preset,
    take_batch_lufs, take_clear_loop, take_del_marker, take_edit_cmd, take_export_pieces,
    take_load, take_load_preset, take_play_pause, take_save_preset, take_set_loop, take_stop,
    take_toggle_mono,
};
use ph2d_panel_audio_editor::{
    AEDIT_FX_LOAD_IR, AEDIT_SPEC_AMOUNT, AEDIT_SPEC_DENOISE, AEDIT_SPEC_LEARN, AEDIT_SPEC_REPAIR,
    AEDIT_SPEC_VIEW, AEDIT_VAR_ENABLED, spectral_state, take_load_ir, take_toggle_enabled,
};
use ph2d_panel_audio_editor::{
    AEDIT_VAR_ADD, AEDIT_VAR_ADD_FOLDER, AEDIT_VAR_GAIN, AEDIT_VAR_LOAD, AEDIT_VAR_PITCH,
    AEDIT_VAR_PLAY, AEDIT_VAR_REMOVE, AEDIT_VAR_ROWS, AEDIT_VAR_SAVE, AEDIT_VAR_STRATEGY_NEXT,
    AEDIT_VAR_STRATEGY_PREV, AEDIT_VAR_WEIGHT_DOWN, AEDIT_VAR_WEIGHT_UP, gain_jitter_norm,
    pitch_jitter_norm, set_strategy_name, set_variation_names, take_add_variation,
    take_add_variation_folder, take_load_variation_set, take_play_variation, take_remove_variation,
    take_save_variation_set, take_strategy_step, take_weight_step, variation_sel,
};
use ph2d_ui_testkit::MockPanelHost;

/// The shell publishes the effect-kind table each frame; without it the rack has no
/// kinds to cycle and no neutral point to seed a fresh stage with, so it stays inert.
/// Three kinds with distinct neutral points are enough to drive every seam below.
fn publish_kind_table() {
    set_fx_kind_names(&["Low-Pass", "High-Pass", "Reverb"]);
    set_fx_kind_defaults(&[
        [1.0, 0.25, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.7, 0.5, 0.0, 0.5],
    ]);
}

/// A rack on a known footing: kind table published, chain back to one neutral stage,
/// nothing auditioning, no armed command.
fn fresh_rack() {
    publish_kind_table();
    reset_fx_chain();
    clear_fx_dirty();
    let _ = take_edit_cmd();
}

/// Clicking Play must reach the play/pause transport intent through the seam.
#[test]
fn play_click_reaches_the_transport_intent() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;

    // Drain any stale intent first (thread-local is per-test-thread but be safe).
    let _ = take_play_pause();

    let outcome =
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_PLAY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Play click ignored — `event.rs` arm for AEDIT_PLAY is missing"
    );
    assert!(
        take_play_pause(),
        "Play click never set the play/pause intent — the panel→shell seam is dead"
    );
}

/// Stop + Load clicks must each reach their one-shot intent.
#[test]
fn stop_and_load_clicks_reach_their_intents() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_stop();
    let _ = take_load();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_STOP));
    assert!(take_stop(), "Stop click never set the stop intent");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOAD));
    assert!(take_load(), "Load click never set the load intent");
}

/// Edit-op clicks must arm the matching one-shot `AudioEditCmd` through the seam.
#[test]
fn edit_clicks_reach_the_edit_command() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();
    set_has_selection(true); // Trim below is a range op; it needs one.

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_NORMALIZE));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::NormalizePeak),
        "Normalize click never armed the edit command"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_TRIM));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::Trim),
        "Trim click never armed the edit command"
    );

    // The effects rack's Apply rides the same seam (W3 block 3a).
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_APPLY));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::ApplyFx),
        "Apply Effect click never armed the effect command"
    );
}

/// Range ops must NOT fire without a selection. `EditClip::target()` silently
/// falls back to the whole clip, so a stray click on the dimmed `Silence` would
/// zero the ENTIRE buffer. The panel dims them, but a dim is cosmetic — found by
/// the 2026-07-09 audit, when disabled buttons still registered their hit rect.
#[test]
fn range_ops_refuse_to_fire_without_a_selection() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();

    set_has_selection(false);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SILENCE));
    assert_eq!(
        take_edit_cmd(),
        None,
        "Silence without a selection would zero the whole clip"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FADE_IN));
    assert_eq!(
        take_edit_cmd(),
        None,
        "Fade In without a selection must not fire"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_TRIM));
    assert_eq!(
        take_edit_cmd(),
        None,
        "Trim without a selection must not fire"
    );

    // With a selection they arm normally.
    set_has_selection(true);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SILENCE));
    assert_eq!(take_edit_cmd(), Some(AudioEditCmd::Silence));
}

/// The live-audition contract. `fx_dirty` is what makes the shell render the chain
/// into the sounding preview, so it must fire on **user input only** — merely
/// opening the panel (which materializes a neutral first stage) cannot start
/// auditioning an effect nobody asked for.
#[test]
fn audition_starts_on_user_input_only_and_cancel_arms_its_command() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();

    assert!(
        !fx_dirty(),
        "a freshly populated rack must not be auditioning"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert!(fx_dirty(), "cycling the effect must start an audition");

    clear_fx_dirty();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    assert!(fx_dirty(), "dragging a parameter must start an audition");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_CANCEL));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::CancelFx),
        "Cancel click never armed the discard command"
    );
}

/// Switching a stage's kind must re-seed its parameters with the NEW kind's neutral
/// point. Carrying the old kind's normals over would audition the new effect with
/// the previous one's settings — an audible glitch, and a stage that reads as
/// "untouched" while it is not.
#[test]
fn switching_kind_reseeds_the_stage_on_the_new_neutral_point() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();

    // Low-Pass is neutral at the TOP of the cutoff range, High-Pass at the bottom.
    assert_eq!(fx_sel_stage(), (0, [1.0, 0.25, 0.0, 0.0]));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert_eq!(
        fx_sel_stage(),
        (1, [0.0, 0.25, 0.0, 0.0]),
        "the stage kept the previous kind's parameters"
    );
}

/// The per-effect Reset icon must put the SELECTED stage back on its kind's neutral
/// defaults through the seam, leaving the rest of the chain alone.
#[test]
fn reset_returns_the_selected_stage_to_its_neutral_defaults() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();
    let neutral = [1.0, 0.25, 0.0, 0.0]; // Low-Pass

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    assert_ne!(fx_sel_stage().1, neutral, "the drag moved a parameter");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_RESET));
    assert_eq!(
        fx_sel_stage().1,
        neutral,
        "Reset did not restore the neutral defaults"
    );
}

/// The rack's selector must actually move the SELECTED stage's kind through the
/// seam — a dead arrow leaves the panel painted, clickable and stuck on one effect.
#[test]
fn fx_selector_cycles_the_kind_and_wraps() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();

    assert_eq!(fx_sel_stage().0, 0);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert_eq!(fx_sel_stage().0, 1, "Next did not advance the kind");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_PREV));
    assert_eq!(fx_sel_stage().0, 0, "Prev did not step back");

    // Prev from 0 wraps to the last kind rather than underflowing (usize would panic).
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_PREV));
    assert_eq!(fx_sel_stage().0, 2, "Prev at 0 must wrap, not underflow");
}

/// Add appends a stage **after** the selected one and selects it, seeded on its
/// neutral point — so growing the chain never changes the sound by itself. Remove
/// drops the selected stage and keeps the selection inside the chain.
#[test]
fn add_and_remove_grow_and_shrink_the_chain_around_the_selection() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();
    assert_eq!(fx_chain().len(), 1, "the rack always has a stage to edit");

    // Tune stage 0, then add: the new stage must be neutral, selected, and last.
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    let tuned = fx_chain()[0].norms;
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_ADD));
    assert_eq!(fx_chain().len(), 2);
    assert_eq!(fx_sel(), 1, "Add selects the stage it created");
    assert_eq!(fx_chain()[0].norms, tuned, "Add disturbed the tuned stage");
    assert_eq!(
        fx_chain()[1].norms,
        [1.0, 0.25, 0.0, 0.0],
        "a fresh stage must be a neutral no-op"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_REMOVE));
    assert_eq!(fx_chain().len(), 1);
    assert_eq!(fx_sel(), 0, "the selection followed the removal");
    assert_eq!(fx_chain()[0].norms, tuned, "Remove dropped the wrong stage");

    // The last stage cannot be removed: the rack would have nothing to edit. The
    // panel dims Remove there, and the seam refuses it too (a dim is cosmetic).
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_REMOVE));
    assert_eq!(fx_chain().len(), 1, "the chain must never empty");
    assert_eq!(fx_chain()[0].norms, tuned, "the last stage was cleared");
}

/// Add stops at `MAX_FX_STAGES` — the chain list is sized for exactly that many
/// rows, so an unbounded Add would paint stages nobody can reach or click.
#[test]
fn add_stops_at_the_chain_capacity() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();

    for _ in 0..MAX_FX_STAGES + 3 {
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_ADD));
    }
    assert_eq!(fx_chain().len(), MAX_FX_STAGES);
    assert!(fx_sel() < MAX_FX_STAGES, "the selection stayed in range");
}

/// Order matters: a filter before a reverb is not the same as after. Up/Down move
/// the SELECTED stage and the selection travels with it, so a second click keeps
/// moving the same effect rather than swapping two others.
#[test]
fn reordering_moves_the_selected_stage_and_the_selection_follows() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();

    // Stage 0 = Low-Pass (tuned), stage 1 = High-Pass (fresh, then switched).
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_ADD));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert_eq!(fx_chain()[1].kind, 1, "stage 1 is High-Pass");
    assert_eq!(fx_sel(), 1);

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_UP));
    assert_eq!(fx_chain()[0].kind, 1, "High-Pass did not move up");
    assert_eq!(fx_chain()[1].kind, 0, "Low-Pass did not move down");
    assert_eq!(fx_sel(), 0, "the selection must travel with the stage");

    // Up at the top is a no-op, not a wrap or an underflow.
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_UP));
    assert_eq!(fx_chain()[0].kind, 1);
    assert_eq!(fx_sel(), 0);

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_DOWN));
    assert_eq!(fx_chain()[1].kind, 1, "Down did not move it back");
    assert_eq!(fx_sel(), 1);
}

/// Clicking a chain row selects it (the selector + sliders follow); the row's eye
/// takes the stage out of the render **without dropping it** — the per-stage A/B.
#[test]
fn a_row_selects_its_stage_and_the_eye_bypasses_it_in_place() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_ADD));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert_eq!(fx_sel(), 1);

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_S1));
    assert_eq!(fx_sel(), 1, "row 1 selects stage 1");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_S0_ON));
    assert!(!fx_chain()[0].enabled, "the eye did not bypass the stage");
    assert_eq!(fx_chain().len(), 2, "the eye must not remove the stage");
    assert_eq!(fx_sel(), 1, "toggling an eye must not move the selection");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_S0_ON));
    assert!(fx_chain()[0].enabled, "the eye did not restore the stage");
}

/// The global A/B: Bypass swaps the dry clip back in without touching the chain, so
/// releasing it must return exactly the chain that was there.
#[test]
fn global_bypass_keeps_the_chain_and_does_not_start_an_audition() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    let chain = fx_chain();
    clear_fx_dirty();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_BYPASS));
    assert!(fx_bypass(), "Bypass click never engaged the A/B");
    assert_eq!(fx_chain(), chain, "Bypass must not disturb the chain");
    assert!(
        !fx_dirty(),
        "Bypass is a monitor switch — it renders nothing new"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_BYPASS));
    assert!(!fx_bypass());
    assert_eq!(fx_chain(), chain);
}

/// After Apply/Cancel the shell bakes (or drops) the chain, so `reset_fx_chain` must
/// leave a single neutral stage behind. Re-rendering a chain that is already baked
/// into the clip would double every effect on the next audition.
#[test]
fn reset_leaves_one_neutral_stage_and_releases_the_bypass() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    fresh_rack();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_ADD));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_BYPASS));
    assert_eq!(fx_chain().len(), 2);

    reset_fx_chain();
    assert_eq!(fx_chain().len(), 1);
    assert_eq!(fx_sel(), 0);
    assert!(!fx_bypass(), "the A/B must not survive a commit");
    assert!(!fx_dirty(), "a reset rack is idle");
    assert_eq!(fx_chain()[0].norms, [1.0, 0.25, 0.0, 0.0], "not neutral");
}

/// The Loop toggle must flip the persistent looping flag through the seam.
#[test]
fn loop_click_toggles_looping() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;

    let before = looping();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP));
    assert_ne!(
        looping(),
        before,
        "Loop click never flipped the looping flag — the AEDIT_LOOP arm is dead"
    );
}

/// Loop points (W6). "Set Loop" adopts the SELECTION, so — like the range ops — it
/// must refuse to fire without one (a dim is cosmetic; `EditClip` would otherwise set
/// a loop from a phantom range).
#[test]
fn loop_set_arms_only_with_a_selection() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_set_loop();

    set_has_selection(false);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP_SET));
    assert!(
        !take_set_loop(),
        "Set Loop without a selection must not fire"
    );

    set_has_selection(true);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP_SET));
    assert!(take_set_loop(), "Set Loop never armed the set-loop intent");
}

/// Force-to-mono is a NON-destructive toggle — its click arms the mono-toggle intent
/// (NOT a destructive edit command). Batch LUFS is a folder op with its own one-shot.
#[test]
fn force_mono_toggle_and_batch_lufs_reach_their_intents() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();
    let _ = take_toggle_mono();
    let _ = take_batch_lufs();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_MONO));
    assert!(
        take_toggle_mono(),
        "Force Mono click never armed the mono-toggle intent"
    );
    assert_eq!(
        take_edit_cmd(),
        None,
        "Force Mono must NOT arm a destructive edit command"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_BATCH_LUFS));
    assert!(
        take_batch_lufs(),
        "Batch LUFS click never armed the folder-op intent"
    );
    assert!(!take_batch_lufs(), "the intent is one-shot");
}

/// Markers (W6): Add arms its intent unconditionally; Delete needs some markers to
/// exist (the shell publishes the count) — the panel dims it and the seam refuses it.
#[test]
fn marker_add_fires_and_delete_needs_markers() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_add_marker();
    let _ = take_del_marker();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_MARK_ADD));
    assert!(take_add_marker(), "Add Marker never armed its intent");

    // No markers yet: Delete is inert.
    set_marker_count(0);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_MARK_DEL));
    assert!(!take_del_marker(), "Delete fired with no markers");

    // Once the shell reports markers, Delete comes alive.
    set_marker_count(2);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_MARK_DEL));
    assert!(take_del_marker(), "Delete never armed its intent");
}

/// Clear acts on an EXISTING loop — the shell publishes whether one is set
/// (`set_loop_span`), and the seam must refuse it (the button stays dim) until there
/// is a loop, then wire through once there is. (Snap folded into Set; Audition removed
/// — the loop plays via Loop + Play.)
#[test]
fn loop_clear_needs_a_loop_then_wires_through() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_clear_loop();

    // No loop yet: Clear is inert.
    set_loop_span(None);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP_CLEAR));
    assert!(!take_clear_loop(), "Clear fired with no loop");

    // Once the shell reports a loop, Clear comes alive.
    set_loop_span(Some((0.5, 1.5)));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP_CLEAR));
    assert!(take_clear_loop(), "Clear never armed its intent");
}

/// The preset selector must cycle over the factory names the shell published, and
/// Apply must arm the "load this preset" one-shot — WITHOUT the panel touching the
/// chain itself (the shell owns the factory table).
#[test]
fn preset_selector_cycles_and_apply_arms_its_intent() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    set_preset_names(&["Voice Cleanup", "Telephone", "Master Bus"]);
    let _ = take_apply_preset();

    let start = preset_sel();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_PRESET_NEXT));
    assert_eq!(
        preset_sel(),
        (start + 1) % 3,
        "Next did not advance the preset"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_PRESET_PREV));
    assert_eq!(preset_sel(), start, "Prev did not step back");

    // Browsing must NOT arm an apply — otherwise arrowing the list would keep
    // overwriting the chain.
    assert!(!take_apply_preset(), "cycling armed an apply");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_PRESET_APPLY));
    assert!(
        take_apply_preset(),
        "Apply click never armed the load-preset intent"
    );
    assert!(!take_apply_preset(), "the intent is one-shot");
}

/// Save and Load must arm their file-dialog one-shots through the seam — a dead arm
/// leaves the button painted and the user's chain unsaveable.
#[test]
fn preset_save_and_load_arm_their_file_intents() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_save_preset();
    let _ = take_load_preset();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_PRESET_SAVE));
    assert!(take_save_preset(), "Save click never armed the save intent");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_PRESET_LOAD));
    assert!(take_load_preset(), "Load click never armed the load intent");
}

/// Variation containers (W6). Add, Add Folder and Load are always live (Add builds the
/// set one clip at a time, Add Folder imports a folder by convention, Load reads a
/// manifest); their clicks must reach the file/folder-picker one-shots.
#[test]
fn variation_add_and_load_arm_their_intents() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_add_variation();
    let _ = take_add_variation_folder();
    let _ = take_load_variation_set();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_ADD));
    assert!(take_add_variation(), "Add click never armed the add intent");
    assert!(!take_add_variation(), "the intent is one-shot");

    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_ADD_FOLDER),
    );
    assert!(
        take_add_variation_folder(),
        "Add Folder click never armed the import intent"
    );
    assert!(!take_add_variation_folder(), "the intent is one-shot");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_LOAD));
    assert!(
        take_load_variation_set(),
        "Load click never armed its intent"
    );
}

/// Play / Remove / Save act on the set, so — like the range ops — they must refuse to
/// fire until a variation exists (the shell publishes the row labels; the panel dims
/// them, and a dim being cosmetic, the seam refuses too).
#[test]
fn variation_play_remove_save_need_a_variation() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_play_variation();
    let _ = take_remove_variation();
    let _ = take_save_variation_set();

    // Empty set: all three are inert.
    set_variation_names(&[]);
    for id in [AEDIT_VAR_PLAY, AEDIT_VAR_REMOVE, AEDIT_VAR_SAVE] {
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(id));
    }
    assert!(!take_play_variation(), "Play fired with no variations");
    assert!(!take_remove_variation(), "Remove fired with no variations");
    assert!(!take_save_variation_set(), "Save fired with no variations");

    // Once the shell reports a clip, they come alive.
    set_variation_names(&["step_01.wav  \u{00d7}1.0".into()]);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_PLAY));
    assert!(take_play_variation(), "Play never armed its intent");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_REMOVE));
    assert!(take_remove_variation(), "Remove never armed its intent");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_SAVE));
    assert!(take_save_variation_set(), "Save never armed its intent");
}

/// Clicking a list row selects it through the seam (the shell reads `variation_sel`
/// to resolve Remove / Weight); the selection clamps to the published count.
#[test]
fn variation_row_click_selects_and_clamps() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    set_variation_names(&["a".into(), "b".into(), "c".into()]);

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_ROWS[2]));
    assert_eq!(variation_sel(), 2, "row 2 click did not select stage 2");

    // A row past the count is registered (fixed id array) but the selection clamps.
    set_variation_names(&["a".into()]);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_ROWS[2]));
    assert_eq!(
        variation_sel(),
        0,
        "selection must clamp to the shrunken set"
    );
}

/// The strategy selector must accumulate signed cycle steps through the seam, and
/// browsing it must NOT arm any other one-shot.
#[test]
fn variation_strategy_selector_accumulates_steps() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_strategy_step();
    set_strategy_name("Shuffle");

    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_STRATEGY_NEXT),
    );
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_STRATEGY_NEXT),
    );
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_STRATEGY_PREV),
    );
    assert_eq!(
        take_strategy_step(),
        1,
        "net cycle steps did not reach the shell"
    );
    assert_eq!(take_strategy_step(), 0, "the accumulator resets on drain");
}

/// Weight ÷2 / ×2 bump the selected entry through the seam, but only with a variation
/// present (net doubling steps; positive = ×2, negative = ÷2).
#[test]
fn variation_weight_bumps_need_a_variation() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_weight_step();

    set_variation_names(&[]);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_WEIGHT_UP));
    assert_eq!(
        take_weight_step(),
        0,
        "weight bump fired with no variations"
    );

    set_variation_names(&["a".into()]);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_WEIGHT_UP));
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_WEIGHT_UP));
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_WEIGHT_DOWN),
    );
    assert_eq!(
        take_weight_step(),
        1,
        "net weight steps did not reach the shell"
    );
}

/// Dragging the pitch / gain jitter sliders must publish their normalized positions
/// through the seam (the shell reads them each frame to build the container jitter).
#[test]
fn variation_jitter_sliders_publish_their_positions() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;

    host.set_slider_value(AEDIT_VAR_PITCH, 0.75);
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(AEDIT_VAR_PITCH),
    );
    assert!(
        (pitch_jitter_norm() - 0.75).abs() < 1e-4,
        "the pitch-jitter slider never reached the shell"
    );

    host.set_slider_value(AEDIT_VAR_GAIN, 0.4);
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(AEDIT_VAR_GAIN),
    );
    assert!(
        (gain_jitter_norm() - 0.4).abs() < 1e-4,
        "the gain-jitter slider never reached the shell"
    );
}

/// A rapid second click on the SAME button arrives as `WidgetEvent::DoubleClick` (the
/// dispatcher upgrades a 2nd Down within 350 ms). The panel has no double-click
/// semantics — every button is a discrete action — so a `DoubleClick` must behave like
/// a `Click`. If it doesn't, the 2nd press is silently dropped and the op "sometimes
/// does nothing" (intermittent by timing). Reproduces the 2026-07-11 multi-agent audit
/// finding; guards the normalization at the top of `apply_event`. Drives both an edit
/// button (Normalize) and a transport button (Play) to prove the fix is global, not
/// per-branch.
#[test]
fn a_double_click_is_treated_as_a_click() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();
    let _ = take_play_pause();

    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::DoubleClick(AEDIT_NORMALIZE),
    );
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::NormalizePeak),
        "a fast double-click on an edit button was swallowed — DoubleClick isn't treated as Click"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::DoubleClick(AEDIT_PLAY));
    assert!(
        take_play_pause(),
        "a double-click on Play was swallowed — the normalization must cover the whole panel"
    );
}

// ---------------------------------------------------------------------------------
// Delivery (W6 asset-prep) — the codec selector and the quality slider.
// ---------------------------------------------------------------------------------

/// The codec selector must reach the shell, because the codec is what decides both the
/// price the panel shows AND the file the Export button writes. A selector that painted
/// but did not dispatch would leave the two disagreeing — the readout would price one
/// format while the export wrote another, which is the worst possible failure for a
/// tool whose whole job is to tell you what you are about to ship.
#[test]
fn the_codec_selector_reaches_the_shell() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    // The shell publishes the table each frame; without it the selector cannot wrap.
    ph2d_panel_audio_editor::delivery_state::set_codec_info(4, "WAV 16-bit", false, "Quality");

    let start = ph2d_panel_audio_editor::delivery_state::codec();
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(ph2d_panel_audio_editor::AEDIT_CODEC_NEXT),
    );
    let next = ph2d_panel_audio_editor::delivery_state::codec();
    assert_ne!(next, start, "clicking the codec arrow did nothing");

    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(ph2d_panel_audio_editor::AEDIT_CODEC_PREV),
    );
    assert_eq!(
        ph2d_panel_audio_editor::delivery_state::codec(),
        start,
        "the selector must step back the way it stepped forward"
    );
}

/// ...and it WRAPS on the table the shell published, rather than walking off the end
/// into an index the shell would have to clamp.
#[test]
fn the_codec_selector_wraps() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    ph2d_panel_audio_editor::delivery_state::set_codec_info(4, "WAV 16-bit", false, "Quality");

    // Step back from the first codec: it must land on the last, not on -1.
    while ph2d_panel_audio_editor::delivery_state::codec() != 0 {
        host.apply_panel_event::<AudioEditorPanel>(
            &mut state,
            WidgetEvent::Click(ph2d_panel_audio_editor::AEDIT_CODEC_PREV),
        );
    }
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(ph2d_panel_audio_editor::AEDIT_CODEC_PREV),
    );
    assert_eq!(
        ph2d_panel_audio_editor::delivery_state::codec(),
        3,
        "stepping back off the front must wrap to the last codec (Opus, since ADR-0116)"
    );
}

/// The quality slider must reach the shell, or the size on screen would be the size of
/// a file nobody is going to write.
#[test]
fn the_quality_slider_reaches_the_shell() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;

    host.set_slider_value(ph2d_panel_audio_editor::AEDIT_OGG_QUALITY, 0.9);
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ph2d_panel_audio_editor::AEDIT_OGG_QUALITY),
    );
    let q = ph2d_panel_audio_editor::delivery_state::quality();
    assert!(
        (q - 0.9).abs() < 1e-4,
        "the quality slider did not reach the shell: {q}"
    );
}

// ---------------------------------------------------------------------------------
// Collapsible sections — the panel's spine.
// ---------------------------------------------------------------------------------

/// **Every section header must be registered as collapsible.** The fold is done by the
/// editor-core dispatch, not by the panel: a click on `id` only folds it if `id` is in
/// the store's `collapsible_sections` set. A header that is painted (chevron and all)
/// but never `mark_collapsible_section`-registered is a chevron that does nothing —
/// the exact "painted but not wired" failure the panel gates exist to stop.
#[test]
fn every_section_header_is_registered_as_collapsible() {
    let host = MockPanelHost::with_panel::<AudioEditorPanel>();
    for id in [
        ph2d_panel_audio_editor::AEDIT_SEC_TRANSPORT,
        ph2d_panel_audio_editor::AEDIT_SEC_EDIT,
        ph2d_panel_audio_editor::AEDIT_SEC_FX,
        ph2d_panel_audio_editor::AEDIT_SEC_LOOP,
        ph2d_panel_audio_editor::AEDIT_SEC_MARKERS,
        ph2d_panel_audio_editor::AEDIT_SEC_VARIATIONS,
        ph2d_panel_audio_editor::AEDIT_SEC_DELIVERY,
    ] {
        assert!(
            host.store().is_collapsible_section(id),
            "a section header the dispatch will not fold: {id:?}"
        );
    }
}

/// ...and the asset-prep half starts FOLDED, so the panel opens on the three blocks you
/// actually work in rather than on the whole wall.
#[test]
fn the_asset_prep_sections_start_folded() {
    let host = MockPanelHost::with_panel::<AudioEditorPanel>();
    for (id, name) in [
        (ph2d_panel_audio_editor::AEDIT_SEC_LOOP, "Loop"),
        (ph2d_panel_audio_editor::AEDIT_SEC_MARKERS, "Markers"),
        (ph2d_panel_audio_editor::AEDIT_SEC_VARIATIONS, "Variations"),
        (ph2d_panel_audio_editor::AEDIT_SEC_DELIVERY, "Delivery"),
    ] {
        assert!(host.store().is_collapsed(id), "{name} should start folded");
    }
    for (id, name) in [
        (ph2d_panel_audio_editor::AEDIT_SEC_TRANSPORT, "Transport"),
        (ph2d_panel_audio_editor::AEDIT_SEC_EDIT, "Edit"),
        (ph2d_panel_audio_editor::AEDIT_SEC_FX, "Effects"),
    ] {
        assert!(!host.store().is_collapsed(id), "{name} should start open");
    }
}

// ── Spectral (W5, ADR-0115) ─────────────────────────────────────────────────────────

/// The view toggle is the precondition for the whole section: the box that Repair needs
/// can only be drawn in the spectrogram, so if this click does not reach the overlay,
/// nothing else here can be reached at all.
#[test]
fn the_view_toggle_flips_the_overlay() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let before = spectral_state::view();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_VIEW));
    assert_ne!(
        spectral_state::view(),
        before,
        "the Spectrogram toggle never reached the overlay"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_VIEW));
    assert_eq!(
        spectral_state::view(),
        before,
        "the toggle does not toggle back"
    );
}

/// **Repair refuses without a time-frequency band, and Denoise refuses without a profile.**
///
/// Both buttons are dimmed without them — and a dim is cosmetic (the 2026-07-09 audit
/// found disabled buttons still registering their hit rects). The failure they guard is not
/// a no-op either: `SpectralRepair` with no band would fall through to whatever the shell
/// made of an empty region, and `Denoise` with no profile is a tool aimed at nothing.
#[test]
fn the_spectral_tools_refuse_to_fire_without_what_they_need() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();

    spectral_state::set_ready(false, false, "");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_REPAIR));
    assert_eq!(
        take_edit_cmd(),
        None,
        "Repair armed with no time-frequency band selected"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_DENOISE));
    assert_eq!(
        take_edit_cmd(),
        None,
        "Denoise armed with no noise profile learned"
    );

    // …and with what they need, they fire.
    spectral_state::set_ready(true, true, "");
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_REPAIR));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::SpectralRepair),
        "Repair never armed its command, even with a band"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_DENOISE));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::Denoise),
        "Denoise never armed its command, even with a profile"
    );
}

/// **Learn refuses without a selection** — and this is the one that could do real harm.
/// Learning from a stretch of *signal* teaches the denoiser that the voice is the noise,
/// and Denoise then dutifully removes the voice. `EditClip::target()` falls back to the
/// whole clip when there is no selection, so an unguarded Learn would do exactly that.
#[test]
fn learn_refuses_without_a_selection() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();

    set_has_selection(false);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_LEARN));
    assert_eq!(
        take_edit_cmd(),
        None,
        "Learn armed with no selection — it would have learned the whole clip as 'noise'"
    );

    set_has_selection(true);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPEC_LEARN));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::LearnNoise),
        "Learn never armed its command, even with a selection"
    );
}

/// The Spectral section's controls are all REGISTERED — a control that paints but was
/// never registered in `populate` is a dead item: it looks live, and clicking it does
/// nothing. (The gate that catches the fan-out's most common wiring miss.)
#[test]
fn every_spectral_control_is_registered() {
    let host = MockPanelHost::with_panel::<AudioEditorPanel>();
    for (id, name) in [
        (AEDIT_SPEC_VIEW, "Spectrogram toggle"),
        (AEDIT_SPEC_REPAIR, "Repair"),
        (AEDIT_SPEC_LEARN, "Learn Noise"),
        (AEDIT_SPEC_DENOISE, "Denoise"),
        (AEDIT_SPEC_AMOUNT, "Amount slider"),
    ] {
        assert!(
            host.store().get(id).is_some(),
            "{name} is painted but never registered in populate — it is a dead control"
        );
    }
}

// ── Convolution reverb: the room ────────────────────────────────────────────────────

/// **The Load IR button reaches the shell** — and it is registered, so it is not a painted
/// corpse. (The fan-out's most common wiring miss: a control that paints, hit-tests, and does
/// nothing, because nobody registered it in `populate`.)
#[test]
fn the_load_ir_button_asks_the_shell_for_a_room() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_load_ir();

    assert!(
        host.store().get(AEDIT_FX_LOAD_IR).is_some(),
        "Load IR is painted but never registered in populate — it is a dead control"
    );
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_LOAD_IR));
    assert!(
        take_load_ir(),
        "clicking Load IR never asked the shell to open the picker"
    );
    assert!(!take_load_ir(), "the request is a one-shot; it fired twice");
}

/// **The Enabled toggle reaches the shell, and refuses without an entry.**
///
/// The flag was always in the model — the picker skips disabled entries, the row paints
/// `(off)`, the manifest round-trips it — and nothing in the UI could turn it. This is the
/// seam that finally connects them, so it is the seam worth gating.
#[test]
fn the_enabled_toggle_reaches_the_shell() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_toggle_enabled();

    assert!(
        host.store().get(AEDIT_VAR_ENABLED).is_some(),
        "the Enabled toggle is painted but never registered — it is a dead control"
    );

    // No variations: a per-entry action has no entry to act on, and the seam refuses (the
    // panel dims it, but a dim is cosmetic).
    set_variation_names(&[]);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_ENABLED));
    assert!(
        !take_toggle_enabled(),
        "the toggle fired with no variation to toggle"
    );

    set_variation_names(&["step_01.wav".to_string()]);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_VAR_ENABLED));
    assert!(
        take_toggle_enabled(),
        "clicking Enabled never reached the shell"
    );
    assert!(
        !take_toggle_enabled(),
        "the request is a one-shot; it fired twice"
    );
}

/// **The clipboard reaches the shell** (W2).
///
/// A button that is painted and hit-indexed but never dispatched is the classic dead item on this
/// panel — it looks alive, it highlights, and it does nothing. Cut/Copy/Paste each have to arm
/// their command through the seam, or the editor grows three buttons that lie.
#[test]
fn clipboard_clicks_reach_the_edit_command() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();
    set_has_selection(true);
    set_has_clipboard(true);

    for (id, want, what) in [
        (AEDIT_CUT, AudioEditCmd::Cut, "Cut"),
        (AEDIT_COPY, AudioEditCmd::Copy, "Copy"),
        (AEDIT_PASTE, AudioEditCmd::Paste, "Paste"),
    ] {
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            take_edit_cmd(),
            Some(want),
            "{what} click never armed the edit command — the button is dead"
        );
    }
}

/// **Split at Markers is an EDIT** — it splits the clip, and that is all it does.
///
/// It used to arm a file-writing one-shot: the button called "Split" encoded the pieces to disk and
/// adopted them as a variation set. Emitting files is a delivery verb, and it moved to one
/// (`Export Pieces`, below). This gate is the difference, and it fails if they are ever rejoined.
#[test]
fn split_at_markers_is_an_edit_and_writes_no_files() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_export_pieces();
    let _ = take_edit_cmd();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_SPLIT));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::SplitAtMarkers),
        "Split at Markers click never armed the edit command — the button is dead"
    );
    assert!(
        !take_export_pieces(),
        "splitting must not write files: that is Export Pieces, in Delivery"
    );
}

/// **Export Pieces reaches the shell** (Delivery).
///
/// It does not ride `AudioEditCmd`: the pieces become FILES, so the shell has to pick a folder, and
/// the panel never touches the filesystem. It arms its own one-shot instead — and that one-shot has
/// to actually fire, exactly once.
#[test]
fn export_pieces_click_reaches_the_export_request() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_export_pieces();

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_EXPORT_PIECES));
    assert!(
        take_export_pieces(),
        "Export Pieces click never armed the request — the button is dead"
    );
    // A one-shot: it must not still be armed on the next frame, or one click exports forever.
    assert!(!take_export_pieces(), "the export request did not drain");
}

/// The Edit toolbar's three tools are a **group**: clicking one arms it, and the other two go
/// quiet. Clicking the armed one leaves it armed — a tool group with an "off" state is a pointer
/// that means nothing over the waveform.
#[test]
fn the_tool_buttons_are_a_group_with_no_off_state() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    tool_state::set_pieces(3); // Move needs somewhere to drop a piece
    tool_state::set_tool(EditTool::Select);

    for (id, want) in [
        (AEDIT_TOOL_MOVE, EditTool::Move),
        (AEDIT_TOOL_SCALE, EditTool::Scale),
        (AEDIT_TOOL_SELECT, EditTool::Select),
    ] {
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(tool_state::tool(), want, "the click did not arm the tool");
        // Clicking it again does not turn it off.
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(tool_state::tool(), want, "an armed tool must stay armed");
    }
}

/// **Move is refused with nothing to trade places with.** The panel dims it, but a dim is
/// cosmetic: arming Move on an uncut clip is a mode the user cannot drag their way out of.
#[test]
fn move_cannot_be_armed_on_an_uncut_clip() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    tool_state::set_pieces(1); // uncut — one piece, nowhere to move it
    tool_state::set_tool(EditTool::Select);

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_TOOL_MOVE));
    assert_eq!(
        tool_state::tool(),
        EditTool::Select,
        "Move armed itself with a single piece — there is nowhere to drop it"
    );
    tool_state::set_pieces(1);
}

/// Split at the playhead and Clear Cuts are ordinary edit commands (the document changes; no
/// dialog, no filesystem).
#[test]
fn the_structure_buttons_arm_their_edit_commands() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    tool_state::set_pieces(2); // Clear Cuts refuses on an uncut clip

    for (id, want, what) in [
        (AEDIT_SPLIT_PLAYHEAD, AudioEditCmd::SplitAtPlayhead, "Split"),
        (AEDIT_CUTS_CLEAR, AudioEditCmd::ClearCuts, "Clear Cuts"),
    ] {
        let _ = take_edit_cmd();
        host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            take_edit_cmd(),
            Some(want),
            "{what} click never armed the edit command — the button is dead"
        );
    }
    tool_state::set_pieces(1);
}

/// **Move disarms itself when the pieces go away.** Refusing to *arm* it on an uncut clip is not
/// enough: arm it with three pieces, then Clear Cuts, and the tool is still held over a clip where
/// dragging can do nothing at all. A pointer with no legal gesture reads as a broken editor.
#[test]
fn move_disarms_itself_when_the_last_cut_is_cleared() {
    tool_state::set_pieces(3);
    tool_state::set_tool(EditTool::Move);
    assert_eq!(tool_state::tool(), EditTool::Move);

    // Clear Cuts (or Load, or an undo of the last split) — the shell republishes the count.
    tool_state::set_pieces(1);
    assert_eq!(
        tool_state::tool(),
        EditTool::Select,
        "Move stayed armed over a clip with one piece — the pointer does nothing and says nothing"
    );
}

/// **Crossfade Loop is an edit** (ADR-0119 A6) — and it refuses when there is nothing to fade from.
///
/// The panel dims it without a pre-roll, but a dim is cosmetic: a bake on a loop that starts at
/// frame 0 has no audio before it to blend with, and firing anyway would land a do-nothing step on
/// the undo timeline.
#[test]
fn the_crossfade_bake_refuses_without_a_pre_roll() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    let _ = take_edit_cmd();

    loop_state::set_can_bake(false);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP_BAKE));
    assert_eq!(
        take_edit_cmd(),
        None,
        "a bake with no pre-roll must not fire — there is nothing to crossfade with"
    );

    loop_state::set_can_bake(true);
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_LOOP_BAKE));
    assert_eq!(
        take_edit_cmd(),
        Some(AudioEditCmd::BakeLoopCrossfade),
        "Crossfade Loop click never armed the edit command — the button is dead"
    );
    loop_state::set_can_bake(false);
}
