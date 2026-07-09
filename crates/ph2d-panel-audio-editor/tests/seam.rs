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
    AEDIT_LOAD, AEDIT_LOOP, AEDIT_NORMALIZE, AEDIT_PLAY, AEDIT_STOP, AEDIT_TRIM, AudioEditCmd,
    AudioEditorPanel, looping, take_edit_cmd, take_load, take_play_pause, take_stop,
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
