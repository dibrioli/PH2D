//! Editor **preview** transport (integration) — the dedicated preview voice's
//! play / seek / pause / hot-swap / pool-independence, exercised through the
//! public [`ph2d_audio::AudioEngine`] + [`ph2d_audio::AudioRenderer`] API.
//! (Moved out of `engine.rs` to keep that file under the workspace LOC cap.)

use ph2d_audio::{AudioEngine, AudioFormat, PlayParams, SampleData};

#[test]
fn preview_advances_seeks_and_finishes() {
    let (engine, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
    // 100-frame mono clip at the output rate → advance is exactly 1 frame per
    // output frame, so the published position is deterministic.
    let data = SampleData::from_interleaved(vec![0.2; 100], AudioFormat::mono(48_000));
    engine.play_preview(data, PlayParams::default()).unwrap();

    let mut out = [0.0f32; 64 * 2];
    r.render(&mut out, 32);
    assert!(engine.preview_playing(), "preview should be sounding");
    assert_eq!(engine.preview_frame(), 32, "advance 1.0 → 32 frames in");
    assert!(out[..64].iter().any(|&s| s.abs() > 0.0));

    // Seek near the end; the next block runs past it and the preview finishes.
    engine.seek_preview(90).unwrap();
    r.render(&mut out, 32);
    assert!(
        !engine.preview_playing(),
        "preview should finish after the end"
    );
}

#[test]
fn preview_data_hot_swap_keeps_position() {
    // An editor edit hot-swaps the sounding preview's buffer WITHOUT stopping it
    // or resetting the cursor.
    let (engine, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
    let quiet = SampleData::from_interleaved(vec![0.2; 200], AudioFormat::mono(48_000));
    engine.play_preview(quiet, PlayParams::default()).unwrap();
    let mut out = [0.0f32; 128];
    r.render(&mut out, 32);
    assert_eq!(engine.preview_frame(), 32);

    let loud = SampleData::from_interleaved(vec![0.8; 200], AudioFormat::mono(48_000));
    engine.set_preview_data(loud).unwrap();
    r.render(&mut out, 32);
    assert!(engine.preview_playing(), "swap must not stop the preview");
    assert_eq!(engine.preview_frame(), 64, "position continues, not reset");
    assert!(
        out[0].abs() > 0.5,
        "device now carries the swapped (louder) samples"
    );
}

#[test]
fn preview_pause_holds_position() {
    let (engine, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
    let data = SampleData::from_interleaved(vec![0.3; 1000], AudioFormat::mono(48_000));
    engine.play_preview(data, PlayParams::default()).unwrap();
    let mut out = [0.0f32; 128];
    r.render(&mut out, 64);
    let at = engine.preview_frame();
    assert_eq!(at, 64);
    engine.pause_preview(true).unwrap();
    r.render(&mut out, 64);
    assert_eq!(engine.preview_frame(), at, "paused preview holds position");
    assert!(engine.preview_playing(), "paused is still active");
    engine.pause_preview(false).unwrap();
    r.render(&mut out, 64);
    assert_eq!(engine.preview_frame(), at + 64);
}

#[test]
fn preview_is_independent_of_game_voices() {
    // A preview must not consume a game voice-pool slot.
    let (mut engine, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
    let data = SampleData::from_interleaved(vec![0.1; 200], AudioFormat::mono(48_000));
    engine
        .play_preview(data.clone(), PlayParams::default())
        .unwrap();
    let _v = engine.play(data, PlayParams::default()).unwrap();
    let mut out = [0.0f32; 128];
    r.render(&mut out, 64);
    assert_eq!(r.active_voices(), 1, "the game voice, not the preview");
    assert!(engine.preview_playing());
}
