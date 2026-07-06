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
    assert!(both[0] > 0.1 && both[1] > 0.1, "both buses should sound: {both:?}");

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
    assert!(music[0] < 1e-3, "muted Music bus meter must read silence: {music:?}");
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
fn master_direct_voice_ignores_sub_bus_faders() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    // A voice on Master (default) is unaffected by a muted sub-bus.
    engine.play(steady(), PlayParams { looping: true, ..PlayParams::default() }).unwrap();
    engine.set_bus_gain(BusId::Music, 0.0).unwrap();
    engine.set_bus_gain(BusId::Sfx, 0.0).unwrap();

    let peak = steady_master_peak(&mut renderer);
    assert!(
        peak[0] > 0.1,
        "a Master-routed voice survives muted sub-buses: {peak:?}"
    );
}
