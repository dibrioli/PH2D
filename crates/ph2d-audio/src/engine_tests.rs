use super::*;

fn _assert_send<T: Send>() {}

#[test]
fn renderer_is_send() {
    // The renderer must move to the audio thread.
    _assert_send::<AudioRenderer>();
}

/// **The Mobile export, played through the WHOLE preview path.** `editor_toggle_play` calls
/// `play_preview`; the renderer drains that command onto its dedicated preview voice and mixes
/// it into `out`. A 24 kHz mono clip (the Mobile target) against a 48 kHz device must come out
/// of `render` audible — the report is that it does not. Every prior gate stops at a return
/// value or a stream-vs-resident diff; this drives the real door end to end.
#[test]
fn a_24k_mono_preview_is_audible_through_the_full_renderer() {
    let tau = std::f32::consts::TAU;
    let (engine, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
    let src: Vec<f32> = (0..12_000)
        .map(|i| 0.5 * (tau * 220.0 * (i as f32 / 24_000.0)).sin())
        .collect();
    let clip = SampleData::from_interleaved(src, AudioFormat::mono(24_000));
    engine.play_preview(clip, PlayParams::default()).unwrap();

    let mut peak = 0.0f32;
    let mut out = [0.0f32; 256];
    for _ in 0..250 {
        out.fill(0.0);
        r.render(&mut out, 128);
        for &s in &out {
            peak = peak.max(s.abs());
        }
    }
    println!("full renderer, 24k mono preview @ 48k: peak {peak:.4}");
    assert!(
        peak > 0.1,
        "the 24 kHz mono Mobile clip came out of the FULL renderer silent (peak {peak:.4}) -- \
             this is what the Enio hears when he loads .mobile.ogg back and presses Play"
    );
}

#[test]
fn play_returns_distinct_handles() {
    let (mut engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
    let data = SampleData::from_interleaved(vec![0.0; 8], AudioFormat::mono(48_000));
    let a = engine.play(data.clone(), PlayParams::default()).unwrap();
    let b = engine.play(data, PlayParams::default()).unwrap();
    assert_ne!(a, b);
    assert!(!a.is_none() && !b.is_none());
}

#[test]
fn open_cutoff_is_true_bypass_across_sample_rates() {
    // The Tone slider's fully-open top is OPEN_CUTOFF_HZ (20 kHz). At 48 kHz
    // the Nyquist guard (sr*0.5*0.9 = 21.6 kHz) alone would NOT bypass it, so
    // the ceiling guard must — at 48 kHz and 96 kHz alike. (Regression: the
    // "open" default used to apply a real 20 kHz low-pass at 48 kHz+.)
    for &sr in &[44_100, 48_000, 96_000] {
        let (engine, _r) = AudioEngine::new(AudioFormat::stereo(sr));
        assert_eq!(
            engine.lowpass_coeffs(OPEN_CUTOFF_HZ),
            BiquadCoeffs::identity(),
            "fully-open Tone must be a true bypass at {sr} Hz"
        );
    }
    // A cutoff below the ceiling is still a real low-pass (not bypassed).
    let (engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
    assert_ne!(
        engine.lowpass_coeffs(1_000.0),
        BiquadCoeffs::identity(),
        "a 1 kHz cutoff must filter, not bypass"
    );
}

#[test]
fn lowcut_off_is_true_bypass_across_sample_rates() {
    // The Low Cut slider's fully-off bottom is FLOOR_CUTOFF_HZ (20 Hz); a
    // high-pass there must be an exact bypass at every sample rate (symmetric
    // to the low-pass "open" ceiling). Above the floor it filters for real.
    for &sr in &[44_100, 48_000, 96_000] {
        let (engine, _r) = AudioEngine::new(AudioFormat::stereo(sr));
        assert_eq!(
            engine.highpass_coeffs(FLOOR_CUTOFF_HZ),
            BiquadCoeffs::identity(),
            "fully-off Low Cut must be a true bypass at {sr} Hz"
        );
    }
    let (engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
    assert_ne!(
        engine.highpass_coeffs(500.0),
        BiquadCoeffs::identity(),
        "a 500 Hz low-cut must filter, not bypass"
    );
}
