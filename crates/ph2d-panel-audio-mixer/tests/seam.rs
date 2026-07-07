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
    AMIX_CUTOFF, AMIX_DELAY, AMIX_DELAY_TIME, AMIX_DUCK, AMIX_DUCK_DEPTH, AMIX_DUCK_KEY,
    AMIX_EQ_LOW, AMIX_FADER, AMIX_LIMITER, AMIX_LOWCUT, AMIX_MASTER_METER, AMIX_MASTER_MUTE,
    AMIX_PAN, AMIX_PLAY, AMIX_REVERB, AMIX_REVERB_SIZE, AudioMixerPanel, FADER_UNITY_POS,
    SUB_DELAY_SEND, SUB_FADER, SUB_LOWCUT, SUB_MUTE, SUB_PAN, SUB_SEND, SUB_SOLO, SUB_TONE,
    delay_on, delay_time, duck_depth, ducking, ducking_key, fader_gain, limiter, master_clipped,
    master_cutoff_target, master_eq_target, master_gain_target, master_lowcut_target, master_muted,
    master_pan_target, play_test, reverb_on, reverb_size, set_levels, sub_delay_send_target,
    sub_gain_target, sub_lowcut_target, sub_muted, sub_pan_target, sub_send_target, sub_soloed,
    sub_tone_target,
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

/// Dragging the Low Cut slider up publishes a high cutoff in Hz; at its default
/// bottom (0.0) it maps to ~20 Hz (off) — read by the shell to drive the master
/// high-pass. Mirror of the Cutoff seam with the ends flipped.
#[test]
fn lowcut_drag_publishes_hz() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(AMIX_LOWCUT, 1.0);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_LOWCUT));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Low Cut ValueChanged — the AMIX_LOWCUT arm is missing"
    );
    assert!(
        (master_lowcut_target() - 20_000.0).abs() < 1.0,
        "low-cut slider at 1.0 must map to ~20 kHz, got {}",
        master_lowcut_target()
    );

    // Back to the bottom → ~20 Hz (off / true bypass).
    host.set_slider_value(AMIX_LOWCUT, 0.0);
    host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_LOWCUT));
    assert!(
        (master_lowcut_target() - 20.0).abs() < 0.5,
        "low-cut slider at 0 must map to ~20 Hz (off), got {}",
        master_lowcut_target()
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
    assert_eq!(
        master_clipped(),
        [false, false],
        "meter click must clear the clip latch"
    );
}

/// The Ducking toggle flips the enable flag, and dragging Depth publishes the
/// raw 0..1 value — both read by the shell to duck Music/SFX under the Voice bus.
#[test]
fn ducking_toggle_and_depth_publish() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = ducking();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_DUCK));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Ducking click — the AMIX_DUCK arm is missing"
    );
    assert_ne!(ducking(), before, "Ducking click must flip the enable flag");

    host.set_slider_value(AMIX_DUCK_DEPTH, 0.9);
    host.apply_panel_event::<AudioMixerPanel>(
        &mut state,
        WidgetEvent::ValueChanged(AMIX_DUCK_DEPTH),
    );
    assert!(
        (duck_depth() - 0.9).abs() < 1e-5,
        "Depth drag must publish the raw value, got {}",
        duck_depth()
    );
}

/// Clicking the sidechain Key selector advances the key sub-bus (wrapping),
/// proving the AMIX_DUCK_KEY arm drives the shell's duck key.
#[test]
fn duck_key_click_cycles_the_key_bus() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = ducking_key();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_DUCK_KEY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Key click — the AMIX_DUCK_KEY arm is missing"
    );
    assert_eq!(
        ducking_key(),
        (before + 1) % 4,
        "Key click must advance to the next sub-bus (wraps at 4)"
    );
}

/// The Delay toggle flips the enable flag, dragging Time publishes the raw 0..1
/// (seconds), and a sub-bus Delay send publishes into its own slot only.
#[test]
fn delay_toggle_time_and_send_publish() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = delay_on();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_DELAY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Delay click — the AMIX_DELAY arm is missing"
    );
    assert_ne!(delay_on(), before, "Delay click must flip the enable flag");

    host.set_slider_value(AMIX_DELAY_TIME, 0.5);
    host.apply_panel_event::<AudioMixerPanel>(
        &mut state,
        WidgetEvent::ValueChanged(AMIX_DELAY_TIME),
    );
    assert!(
        (delay_time() - 0.5).abs() < 1e-5,
        "Time drag must publish the raw value, got {}",
        delay_time()
    );

    host.set_slider_value(SUB_DELAY_SEND[0], 0.7);
    host.apply_panel_event::<AudioMixerPanel>(
        &mut state,
        WidgetEvent::ValueChanged(SUB_DELAY_SEND[0]),
    );
    let sends = sub_delay_send_target();
    assert!(
        (sends[0] - 0.7).abs() < 1e-5,
        "delay send must publish 0.7, got {}",
        sends[0]
    );
    assert!(sends[1].abs() < 1e-5, "the other bus delay send stays dry");
}

/// The Reverb toggle flips the enable flag, and dragging Size publishes the raw
/// 0..1 value — both read by the shell to drive the master reverb.
#[test]
fn reverb_toggle_and_size_publish() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    let before = reverb_on();
    let outcome =
        host.apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::Click(AMIX_REVERB));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Reverb click — the AMIX_REVERB arm is missing"
    );
    assert_ne!(
        reverb_on(),
        before,
        "Reverb click must flip the enable flag"
    );

    host.set_slider_value(AMIX_REVERB_SIZE, 0.8);
    host.apply_panel_event::<AudioMixerPanel>(
        &mut state,
        WidgetEvent::ValueChanged(AMIX_REVERB_SIZE),
    );
    assert!(
        (reverb_size() - 0.8).abs() < 1e-5,
        "Size drag must publish the raw value, got {}",
        reverb_size()
    );
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
    assert_ne!(
        limiter(),
        before,
        "Limiter click must flip the limiter flag"
    );
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
    assert_ne!(
        play_test(),
        before,
        "Play Test click must flip the play flag"
    );
}

/// Dragging a sub-bus Tone slider down publishes a low cutoff in Hz (~20 Hz at
/// the bottom) into that bus's slot only — the shell drives its low-pass filter.
#[test]
fn sub_bus_tone_drag_publishes_cutoff_hz() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(SUB_TONE[0], 0.0);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(SUB_TONE[0]));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the sub-bus Tone ValueChanged — the SUB_TONE arm is missing"
    );
    let tones = sub_tone_target();
    assert!(
        (tones[0] - 20.0).abs() < 0.5,
        "Tone at 0 must map to ~20 Hz, got {}",
        tones[0]
    );
    assert!(
        (tones[1] - 20_000.0).abs() < 1.0,
        "the other sub-bus tone must stay open (~20 kHz), got {}",
        tones[1]
    );
}

/// Dragging a sub-bus Low Cut slider up publishes a high cutoff in Hz into that
/// bus's slot only — the shell drives its high-pass filter. The others stay at
/// ~20 Hz (off), proving the SUB_LOWCUT arm routes to the right bus.
#[test]
fn sub_bus_lowcut_drag_publishes_cutoff_hz() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(SUB_LOWCUT[0], 1.0);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(SUB_LOWCUT[0]));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the sub-bus Low Cut ValueChanged — the SUB_LOWCUT arm is missing"
    );
    let lowcuts = sub_lowcut_target();
    assert!(
        (lowcuts[0] - 20_000.0).abs() < 1.0,
        "Low Cut at 1.0 must map to ~20 kHz, got {}",
        lowcuts[0]
    );
    assert!(
        (lowcuts[1] - 20.0).abs() < 0.5,
        "the other sub-bus low-cut must stay off (~20 Hz), got {}",
        lowcuts[1]
    );
}

/// Dragging a sub-bus reverb Send slider publishes that bus's aux-send amount
/// (0..1) into its own slot only — the shell routes it to that bus's reverb
/// send. The others stay dry (0), proving the SUB_SEND arm routes per bus.
#[test]
fn sub_bus_send_drag_publishes_amount() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(SUB_SEND[0], 0.6);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(SUB_SEND[0]));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the sub-bus Send ValueChanged — the SUB_SEND arm is missing"
    );
    let sends = sub_send_target();
    assert!(
        (sends[0] - 0.6).abs() < 1e-5,
        "Send at 0.6 must publish 0.6, got {}",
        sends[0]
    );
    assert!(
        sends[1].abs() < 1e-5,
        "the other sub-bus send must stay dry (0), got {}",
        sends[1]
    );
}

/// Dragging a master EQ band publishes its raw 0..1 position into that band's
/// slot only — the shell maps it to ±12 dB. The other bands stay flat (0.5),
/// proving the EQ arm routes per band.
#[test]
fn eq_drag_publishes_position() {
    let mut host = MockPanelHost::with_panel::<AudioMixerPanel>();
    let mut state = AudioMixerState;

    host.set_slider_value(AMIX_EQ_LOW, 0.8);
    let outcome = host
        .apply_panel_event::<AudioMixerPanel>(&mut state, WidgetEvent::ValueChanged(AMIX_EQ_LOW));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the EQ Low ValueChanged — the AMIX_EQ arm is missing"
    );
    let eq = master_eq_target();
    assert!(
        (eq[0] - 0.8).abs() < 1e-5,
        "EQ Low must publish 0.8, got {}",
        eq[0]
    );
    assert!(
        (eq[1] - 0.5).abs() < 1e-5,
        "the Mid band must stay flat (0.5), got {}",
        eq[1]
    );
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
