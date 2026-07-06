//! Sub-bus routing + per-bus fader are independent: muting the Music bus
//! silences only its voices, the SFX bus keeps sounding, and each strip's
//! post-fader meter reflects its own bus. This is the behavioral proof that
//! `PlayParams.bus` routes and `set_bus_gain` attenuates the right sub-mix.

use ph2d_audio::{AudioEngine, AudioFormat, BusId, PlayParams, SampleData};

/// A steady full-scale mono sample (loops so the bus keeps producing signal
/// across the whole warm-up, letting the smoothed fader settle).
fn steady() -> SampleData {
    SampleData::from_interleaved(vec![0.8; 960], AudioFormat::mono(48_000))
}

fn on(bus: BusId) -> PlayParams {
    PlayParams {
        looping: true,
        bus,
        ..PlayParams::default()
    }
}

/// Render enough warm blocks for any smoothed fader ramp to fully settle, then
/// return the master peak `[L, R]` of the final (steady) block.
fn steady_master_peak(renderer: &mut ph2d_audio::AudioRenderer) -> [f32; 2] {
    const FRAMES: usize = 512;
    let mut out = vec![0.0f32; FRAMES * 2];
    for _ in 0..200 {
        renderer.render(&mut out, FRAMES);
    }
    let mut pl = 0.0f32;
    let mut pr = 0.0f32;
    for f in 0..FRAMES {
        pl = pl.max(out[2 * f].abs());
        pr = pr.max(out[2 * f + 1].abs());
    }
    [pl, pr]
}

#[test]
fn muting_one_bus_leaves_the_other_sounding() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    engine.play(steady(), on(BusId::Music)).unwrap();
    engine.play(steady(), on(BusId::Sfx)).unwrap();

    // Both buses open → the master carries both voices.
    let both = steady_master_peak(&mut renderer);
    assert!(
        both[0] > 0.1 && both[1] > 0.1,
        "both buses should sound: {both:?}"
    );

    // Mute Music (fold to gain 0) → only SFX remains, so the master is still
    // audible but quieter than with both open.
    engine.set_bus_gain(BusId::Music, 0.0).unwrap();
    let sfx_only = steady_master_peak(&mut renderer);
    assert!(
        sfx_only[0] > 0.1,
        "SFX must keep sounding after Music is muted: {sfx_only:?}"
    );
    assert!(
        sfx_only[0] < both[0] - 1e-3,
        "muting Music must drop the master peak (both {both:?} → sfx {sfx_only:?})"
    );

    // Per-bus meters: Music post-fader ≈ silent, SFX still hot.
    let bus = engine.bus_levels();
    let [music, sfx] = [bus[0], bus[1]];
    assert!(
        music[0] < 1e-3,
        "muted Music bus meter must read silence: {music:?}"
    );
    assert!(sfx[0] > 0.1, "SFX bus meter must stay hot: {sfx:?}");
}

#[test]
fn panning_a_sub_bus_hard_left_empties_the_right_channel() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    engine.play(steady(), on(BusId::Music)).unwrap();

    // Centered → both channels carry the bus.
    let centered = steady_master_peak(&mut renderer);
    assert!(
        centered[0] > 0.1 && centered[1] > 0.1,
        "centered bus should fill both channels: {centered:?}"
    );

    // Hard-left → the right channel goes silent, the left keeps its level.
    engine.set_bus_pan(BusId::Music, -1.0).unwrap();
    let left = steady_master_peak(&mut renderer);
    assert!(
        left[1] < 1e-3,
        "hard-left pan must empty the right channel: {left:?}"
    );
    assert!(
        (left[0] - centered[0]).abs() < 1e-3,
        "hard-left pan must keep the left channel at its centered level ({centered:?} → {left:?})"
    );

    // The strip meter is pre-pan, so it must not drop when panned.
    let music_meter = engine.bus_levels()[0];
    assert!(
        music_meter[0] > 0.1 && music_meter[1] > 0.1,
        "the pre-pan bus meter must stay hot on both channels when panned: {music_meter:?}"
    );
}

#[test]
fn rms_is_positive_and_never_exceeds_peak() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    engine.play(steady(), on(BusId::Music)).unwrap();
    let mut out = vec![0.0f32; 512 * 2];
    for _ in 0..50 {
        renderer.render(&mut out, 512);
    }

    let (peak, rms) = (engine.levels(), engine.rms());
    assert!(rms[0] > 0.0, "master RMS must be positive with signal: {rms:?}");
    assert!(
        rms[0] <= peak[0] + 1e-4,
        "RMS must never exceed peak (master): rms {rms:?} peak {peak:?}"
    );
    let (bpeak, brms) = (engine.bus_levels()[0], engine.bus_rms()[0]);
    assert!(brms[0] > 0.0, "Music bus RMS must be positive: {brms:?}");
    assert!(
        brms[0] <= bpeak[0] + 1e-4,
        "RMS must never exceed peak (bus): rms {brms:?} peak {bpeak:?}"
    );
}

#[test]
fn master_limiter_tames_a_boosted_over_unity_mix() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    engine.play(steady(), on(BusId::Music)).unwrap();
    // Boost the master well past full scale so the raw mix would clip.
    engine.set_master_gain(2.5).unwrap();

    // Read the engine's *pre-clamp* peak meter (the device buffer is clamped, so
    // it can't show over-unity); render enough for the gain ramp to settle.
    let mut out = vec![0.0f32; 512 * 2];
    for _ in 0..200 {
        renderer.render(&mut out, 512);
    }
    let clipping = engine.levels();
    let rms_off = engine.rms();
    assert!(
        clipping[0] > 1.0,
        "precondition: the boosted mix must exceed full scale, got {clipping:?}"
    );

    // Engage the limiter → the master peak is pulled below the clip ceiling.
    engine.set_master_limiter(true).unwrap();
    for _ in 0..200 {
        renderer.render(&mut out, 512);
    }
    let limited = engine.levels();
    let rms_on = engine.rms();
    assert!(
        limited[0] <= 1.0,
        "the limiter must keep the master under full scale, got {limited:?}"
    );
    assert!(limited[0] > 0.5, "…without gutting the signal, got {limited:?}");
    // The audible bit: gain reduction actually lowers the sustained level, not
    // just the peak tips (a static soft-clip would barely move the RMS).
    assert!(
        rms_on[0] < rms_off[0] - 0.05,
        "the limiter must audibly reduce the sustained level: rms {rms_off:?} → {rms_on:?}"
    );
}

#[test]
fn sub_bus_lowpass_attenuates_high_frequency_content() {
    // A near-Nyquist tone on the Music bus: a low cutoff must knock it down.
    let sr = 48_000u32;
    let fmt = AudioFormat::stereo(sr);
    // Alternating ±0.8 = the highest representable frequency (Nyquist).
    let hi: Vec<f32> = (0..960).map(|i| if i % 2 == 0 { 0.8 } else { -0.8 }).collect();
    let nyquist = SampleData::from_interleaved(hi, AudioFormat::mono(sr));

    let (mut engine, mut renderer) = AudioEngine::new(fmt);
    engine
        .play(
            nyquist,
            PlayParams {
                looping: true,
                bus: BusId::Music,
                ..PlayParams::default()
            },
        )
        .unwrap();

    let mut out = vec![0.0f32; 512 * 2];
    for _ in 0..20 {
        renderer.render(&mut out, 512);
    }
    let open = engine.bus_levels()[0];
    assert!(open[0] > 0.3, "precondition: the open bus passes the tone: {open:?}");

    // Close the Music bus filter to 500 Hz → the Nyquist tone is heavily cut.
    engine.set_bus_cutoff(BusId::Music, 500.0).unwrap();
    for _ in 0..40 {
        renderer.render(&mut out, 512);
    }
    let filtered = engine.bus_levels()[0];
    assert!(
        filtered[0] < open[0] * 0.5,
        "the low-pass must attenuate the near-Nyquist tone (open {open:?} → filtered {filtered:?})"
    );
}

#[test]
fn master_reverb_leaves_a_decaying_tail_after_the_sound_ends() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    engine.set_reverb(true, 0.9, 0.6).unwrap();
    // A short one-shot burst (480 frames < one 512-frame block).
    let burst = SampleData::from_interleaved(vec![0.6; 480], AudioFormat::mono(48_000));
    engine.play(burst, PlayParams::default()).unwrap();

    let mut out = vec![0.0f32; 512 * 2];
    for _ in 0..30 {
        renderer.render(&mut out, 512);
    }
    // The dry burst ended after the first block; only the wet tail remains.
    let tail = engine.levels();
    assert!(
        tail[0] > 1e-4,
        "reverb must leave a wet tail after the sound ends: {tail:?}"
    );
}

#[test]
fn without_reverb_output_is_silent_after_the_sound_ends() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    let burst = SampleData::from_interleaved(vec![0.6; 480], AudioFormat::mono(48_000));
    engine.play(burst, PlayParams::default()).unwrap();

    let mut out = vec![0.0f32; 512 * 2];
    for _ in 0..30 {
        renderer.render(&mut out, 512);
    }
    let tail = engine.levels();
    assert!(
        tail[0] < 1e-5,
        "without reverb, output must be silent after the sound ends: {tail:?}"
    );
}

#[test]
fn master_direct_voice_ignores_sub_bus_faders() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    // A voice on Master (default) is unaffected by a muted sub-bus.
    engine
        .play(
            steady(),
            PlayParams {
                looping: true,
                ..PlayParams::default()
            },
        )
        .unwrap();
    engine.set_bus_gain(BusId::Music, 0.0).unwrap();
    engine.set_bus_gain(BusId::Sfx, 0.0).unwrap();

    let peak = steady_master_peak(&mut renderer);
    assert!(
        peak[0] > 0.1,
        "a Master-routed voice survives muted sub-buses: {peak:?}"
    );
}
