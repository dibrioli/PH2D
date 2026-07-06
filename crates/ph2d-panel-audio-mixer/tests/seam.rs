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
    AMIX_CUTOFF, AMIX_FADER, AMIX_LIMITER, AMIX_MASTER_METER, AMIX_MASTER_MUTE, AMIX_PAN, AMIX_PLAY,
    AudioMixerPanel, FADER_UNITY_POS, SUB_FADER, SUB_MUTE, SUB_PAN, SUB_SOLO, fader_gain, limiter,
    master_clipped, master_cutoff_target, master_gain_target, master_muted, master_pan_target,
    play_test, set_levels, sub_gain_target, sub_muted, sub_pan_target, sub_soloed,
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
/// 0..1 position into the store, then emits `ValueChanged(AMIX_FADER)`) must
/// publish the **dB-tapered** gain — not the raw position — as the master gain.
#[test]
fn fader_drag_publishes_tapered_master_gain() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    // A drag to 30% of the travel → the taper's gain for that position.
    host.set_slider_value(AMIX_FADER, 0.3);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_FADER));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the fader ValueChanged — the AMIX_FADER arm is missing"
    );
    assert!(
        (master_gain_target() - fader_gain(0.3)).abs() < 1e-5,
        "fader must publish the tapered gain fader_gain(0.3)={}, got {}",
        fader_gain(0.3),
        master_gain_target()
    );

    // At the unity position the taper is exactly 0 dB → gain 1.0.
    host.set_slider_value(AMIX_FADER, FADER_UNITY_POS);
    host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_FADER));
    assert!(
        (master_gain_target() - 1.0).abs() < 1e-4,
        "unity fader position must publish unity gain, got {}",
        master_gain_target()
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

/// Dragging a sub-bus fader publishes that bus's gain into its own slot (index
/// 0 = the first sub-bus), leaving the other sub-buses untouched — proving the
/// per-strip `SUB_FADER` arm routes to the right bus.
#[test]
fn sub_bus_fader_drag_publishes_that_bus_gain() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(SUB_FADER[0], 0.4);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(SUB_FADER[0]));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the sub-bus fader ValueChanged — the SUB_FADER arm is missing"
    );
    let gains = sub_gain_target();
    assert!(
        (gains[0] - fader_gain(0.4)).abs() < 1e-5,
        "sub-bus 0 gain must be the tapered fader_gain(0.4)={}, got {}",
        fader_gain(0.4),
        gains[0]
    );
    assert!(
        (gains[1] - 1.0).abs() < 1e-5,
        "the other sub-bus must stay at unity gain, got {}",
        gains[1]
    );
}

/// A master peak over full scale latches the clip indicator; clicking the meter
/// clears it. (`set_levels` is the shell→panel publish that also latches clip.)
#[test]
fn clip_latches_on_over_unity_peak_and_meter_click_clears_it() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    // Clear any latch left on this thread, then publish a clipping peak.
    host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_MASTER_METER));
    set_levels([1.4, 0.2], [0.5, 0.1]);
    assert_eq!(
        master_clipped(),
        [true, false],
        "a left peak > 1.0 must latch the left clip flag only"
    );

    // A later quiet frame must NOT un-latch it (latch sticks until cleared).
    set_levels([0.1, 0.1], [0.05, 0.05]);
    assert_eq!(master_clipped(), [true, false], "clip latch must stick");

    // Clicking the meter clears the latch.
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_MASTER_METER));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the meter click — the AMIX_MASTER_METER arm is missing"
    );
    assert_eq!(master_clipped(), [false, false], "meter click must clear the clip latch");
}

/// Clicking the Limiter button toggles the limiter flag the shell reads to
/// engage/disengage the master soft-clip limiter.
#[test]
fn limiter_click_toggles_flag() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = limiter();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_LIMITER));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Limiter click — the AMIX_LIMITER arm is missing"
    );
    assert_ne!(limiter(), before, "Limiter click must flip the limiter flag");
}

/// Clicking the footer Play Test button toggles the play flag the shell reads
/// to start/stop the built-in test signal.
#[test]
fn play_test_click_toggles_flag() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = play_test();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_PLAY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Play Test click — the AMIX_PLAY arm is missing"
    );
    assert_ne!(play_test(), before, "Play Test click must flip the play flag");
}

/// Clicking a sub-bus solo button flips only that bus's solo flag (the shell
/// reads it back to mute the non-soloed buses).
#[test]
fn sub_bus_solo_click_toggles_only_that_bus() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = sub_soloed();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(SUB_SOLO[0]));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the solo click — the SUB_SOLO arm is missing"
    );
    let after = sub_soloed();
    assert_ne!(after[0], before[0], "sub-bus 0 solo must flip");
    assert_eq!(after[1], before[1], "sub-bus 1 solo must be untouched");
}

/// Dragging a pan slider remaps the 0..1 value to a `-1..1` balance: the
/// Master pan hard-left (0.0) publishes -1.0, and a sub-bus pan to the far
/// right (1.0) publishes +1.0 into that bus's slot only.
#[test]
fn pan_sliders_publish_remapped_balance() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    // Master pan hard-left.
    host.set_slider_value(AMIX_PAN, 0.0);
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_PAN));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Master pan ValueChanged — the AMIX_PAN arm is missing"
    );
    assert!(
        (master_pan_target() + 1.0).abs() < 1e-5,
        "Master pan at 0.0 must map to -1.0 (hard left), got {}",
        master_pan_target()
    );

    // Sub-bus 0 pan far-right → +1.0 in slot 0 only.
    host.set_slider_value(SUB_PAN[0], 1.0);
    host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(SUB_PAN[0]));
    let pans = sub_pan_target();
    assert!(
        (pans[0] - 1.0).abs() < 1e-5,
        "sub-bus 0 pan at 1.0 must map to +1.0, got {}",
        pans[0]
    );
    assert!(
        pans[1].abs() < 1e-5,
        "the other sub-bus pan must stay centered (0.0), got {}",
        pans[1]
    );
}

/// Clicking a sub-bus mute button flips only that bus's mute flag (the shell
/// reads it back to zero that bus's gain).
#[test]
fn sub_bus_mute_click_toggles_only_that_bus() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = sub_muted();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(SUB_MUTE[0]));

    assert_eq!(outcome, EventOutcome::Consumed);
    let after = sub_muted();
    assert_ne!(after[0], before[0], "sub-bus 0 mute must flip");
    assert_eq!(after[1], before[1], "sub-bus 1 mute must be untouched");
}
