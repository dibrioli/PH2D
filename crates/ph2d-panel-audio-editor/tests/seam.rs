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
    AEDIT_FX_APPLY, AEDIT_FX_CANCEL, AEDIT_FX_NEXT, AEDIT_FX_P0, AEDIT_FX_PREV, AEDIT_LOAD,
    AEDIT_LOOP, AEDIT_NORMALIZE, AEDIT_PLAY, AEDIT_STOP, AEDIT_TRIM, AudioEditCmd,
    AudioEditorPanel, clear_fx_dirty, clear_fx_touched, fx_dirty, fx_kind, fx_touched, looping,
    set_fx_kind_count, take_edit_cmd, take_load, take_play_pause, take_stop,
};
use ph2d_ui_testkit::MockPanelHost;

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

/// The live-audition contract. `fx_dirty` is what makes the shell render the
/// effect into the sounding preview, so it must fire on **user input only** —
/// merely opening the panel (which re-seeds the sliders with the preset) cannot
/// start auditioning an effect nobody asked for.
#[test]
fn audition_starts_on_user_input_only_and_cancel_arms_its_command() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    clear_fx_dirty();
    let _ = take_edit_cmd();
    set_fx_kind_count(3);

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

/// `fx_touched` is what tells the shell to **stack** the active effect onto the
/// live chain when the user switches effects. It must fire on a slider drag and
/// NOT on the arrows — otherwise browsing the effect list would silently pile up
/// presets. The shell (not the panel) consumes the flag once it has stacked it.
#[test]
fn only_a_tuned_effect_is_marked_for_stacking() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    clear_fx_dirty(); // also clears `touched`
    set_fx_kind_count(3);
    assert!(!fx_touched(), "a fresh stage is untouched");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert!(
        !fx_touched(),
        "browsing with the arrows must not mark a stage as tuned"
    );

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::ValueChanged(AEDIT_FX_P0));
    assert!(fx_touched(), "dragging a parameter tunes the stage");

    // Switching does NOT clear it here — the shell reads the flag to decide whether
    // to stack the stage, and clears it afterwards.
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert!(
        fx_touched(),
        "the shell consumes the tuned flag, not the panel"
    );
    clear_fx_touched();
    assert!(!fx_touched());
}

/// The rack's selector must actually move the kind index through the seam — a
/// dead arrow leaves the panel painted, clickable and stuck on one effect.
#[test]
fn fx_selector_cycles_the_kind_and_wraps() {
    let mut host = MockPanelHost::with_panel::<AudioEditorPanel>();
    let mut state = AudioEditorState;
    // The shell publishes how many kinds exist; without it the selector is inert.
    set_fx_kind_count(3);

    let start = fx_kind();
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_NEXT));
    assert_eq!(fx_kind(), (start + 1) % 3, "Next did not advance the kind");

    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_PREV));
    assert_eq!(fx_kind(), start, "Prev did not step back");

    // Prev from 0 wraps to the last kind rather than underflowing (usize would panic).
    host.apply_panel_event::<AudioEditorPanel>(&mut state, WidgetEvent::Click(AEDIT_FX_PREV));
    assert_eq!(
        fx_kind(),
        (start + 2) % 3,
        "Prev at 0 must wrap, not underflow"
    );
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
