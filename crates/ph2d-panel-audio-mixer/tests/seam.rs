//! Behavioral SEAM test for the Audio Mixer panel's `apply_event` (DIRETIVA §5
//! DoD). Populate + the wiring gates prove the buttons are registered, but only
//! this proves the `apply_event → set_panel_visible` / mute wire is intact:
//! it drives the exact `Click` events the TopBar pill and the Mute button emit,
//! headless, and asserts the observable effect.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_panel_audio_mixer::state::AudioMixerState;
use ph2d_panel_audio_mixer::{
    AMIX_CUTOFF, AMIX_FADER, AMIX_MASTER_MUTE, AudioMixerPanel, master_cutoff_target,
    master_gain_target, master_muted,
};
use ph2d_ui_testkit::MockPanelHost;

/// The TopBar "MIX" pill toggles the panel: from hidden, the click opens it;
/// a second click hides it — proving the `TOPBAR_AUDIO_MIXER` arm drives the
/// canonical visibility flag both ways.
#[test]
fn topbar_pill_click_toggles_visibility() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    assert!(
        !host.panel_visible(AudioMixerPanel::ID),
        "precondition: mixer starts hidden"
    );

    let outcome = host.apply_panel_event::<AudioMixerPanel>(
        &mut state,
        WidgetEvent::Click(ids::TOPBAR_AUDIO_MIXER),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the TOPBAR_AUDIO_MIXER click — the event.rs arm is missing"
    );
    assert!(
        host.panel_visible(AudioMixerPanel::ID),
        "pill click was consumed but visibility never flipped on — the seam is dead"
    );

    // Toggle again → hides.
    host.apply_panel_event::<AudioMixerPanel>(
        &mut state,
        WidgetEvent::Click(ids::TOPBAR_AUDIO_MIXER),
    );
    assert!(
        !host.panel_visible(AudioMixerPanel::ID),
        "second pill click must hide the panel again"
    );
}

/// Clicking the Master mute button flips the panel-owned muted state (which the
/// shell reads back to zero the engine's master gain).
#[test]
fn mute_click_toggles_master_muted() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = master_muted();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_MASTER_MUTE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the mute click — the AMIX_MASTER_MUTE arm is missing"
    );
    assert_ne!(
        master_muted(),
        before,
        "mute click was consumed but the panel-owned muted state never flipped"
    );
}

/// Dragging the Master fader (the shared vertical-slider dispatch writes the new
/// value into the store, then emits `ValueChanged(AMIX_FADER)`) must publish
/// that value as the master gain the shell reads to drive the engine.
#[test]
fn fader_drag_publishes_master_gain() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    // What a drag to 30% writes into the store before the ValueChanged fires.
    host.set_slider_value(AMIX_FADER, 0.3);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_FADER));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the fader ValueChanged — the AMIX_FADER arm is missing"
    );
    assert!(
        (master_gain_target() - 0.3).abs() < 1e-5,
        "fader drag was consumed but the published master gain never updated"
    );
}

/// Dragging the Cutoff slider to the bottom publishes the lowest cutoff
/// (~20 Hz) — the log map 0..1 → 20 Hz..20 kHz, read by the shell to drive the
/// master low-pass filter.
#[test]
fn cutoff_drag_publishes_hz() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(AMIX_CUTOFF, 0.0);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_CUTOFF));

    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        (master_cutoff_target() - 20.0).abs() < 0.5,
        "cutoff slider at 0 must map to ~20 Hz, got {}",
        master_cutoff_target()
    );
}
