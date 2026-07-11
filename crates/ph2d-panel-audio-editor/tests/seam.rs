//! Behavioral SEAM test for the Audio Editor panel (DIRETIVA §5 DoD).
//!
//! Unit tests prove `snapshot` round-trips and `populate` registers widgets —
//! but NEITHER proves the `event.rs` wire from a real `WidgetEvent` to the
//! transport intent is intact. A forgotten arm leaves a button painted,
//! clickable and SILENTLY DEAD while every unit test stays green. This drives
//! the full path the shell runs, headless: apply_event → snapshot intent.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_panel_audio_editor::state::AudioEditorState;
use ph2d_panel_audio_editor::{
    AEDIT_BATCH_LUFS, AEDIT_FADE_IN, AEDIT_FX_ADD, AEDIT_FX_APPLY, AEDIT_FX_BYPASS,
    AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_NEXT, AEDIT_FX_P0, AEDIT_FX_PREV, AEDIT_FX_REMOVE,
    AEDIT_FX_RESET, AEDIT_FX_S0_ON, AEDIT_FX_S1, AEDIT_FX_UP, AEDIT_LOAD, AEDIT_LOOP,
    AEDIT_LOOP_CLEAR, AEDIT_LOOP_SET, AEDIT_MARK_ADD, AEDIT_MARK_DEL, AEDIT_MONO, AEDIT_NORMALIZE,
    AEDIT_PLAY, AEDIT_PRESET_APPLY, AEDIT_PRESET_LOAD, AEDIT_PRESET_NEXT, AEDIT_PRESET_PREV,
    AEDIT_PRESET_SAVE, AEDIT_SILENCE, AEDIT_STOP, AEDIT_TRIM, AudioEditCmd, AudioEditorPanel,
    MAX_FX_STAGES, clear_fx_dirty, fx_bypass, fx_chain, fx_dirty, fx_sel, fx_sel_stage, looping,
    preset_sel, reset_fx_chain, set_fx_kind_defaults, set_fx_kind_names, set_has_selection,
    set_loop_span, set_marker_count, set_preset_names, take_add_marker, take_apply_preset,
    take_batch_lufs, take_clear_loop, take_del_marker, take_edit_cmd, take_load, take_load_preset,
    take_play_pause, take_save_preset, take_set_loop, take_stop, take_toggle_mono,
};
use ph2d_panel_audio_editor::{
    AEDIT_VAR_ADD, AEDIT_VAR_ADD_FOLDER, AEDIT_VAR_GAIN, AEDIT_VAR_LOAD, AEDIT_VAR_PITCH,
    AEDIT_VAR_PLAY, AEDIT_VAR_REMOVE, AEDIT_VAR_ROWS, AEDIT_VAR_SAVE, AEDIT_VAR_STRAT_NEXT,
    AEDIT_VAR_STRAT_PREV, AEDIT_VAR_WEIGHT_DOWN, AEDIT_VAR_WEIGHT_UP, gain_jitter_norm,
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
        WidgetEvent::Click(AEDIT_VAR_STRAT_NEXT),
    );
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_STRAT_NEXT),
    );
    host.apply_panel_event::<AudioEditorPanel>(
        &mut state,
        WidgetEvent::Click(AEDIT_VAR_STRAT_PREV),
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
