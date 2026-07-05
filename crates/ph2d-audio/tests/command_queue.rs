//! The control→audio ring is bounded and lock-free: a full ring reports
//! `QueueFull` (never blocks/panics/allocates unboundedly), and once the audio
//! side drains it the control side can enqueue again.

use ph2d_audio::{AudioEngine, AudioError, AudioFormat, CMD_CAPACITY, PlayParams, SampleData};

#[test]
fn full_ring_reports_error_then_recovers() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    let data = SampleData::from_interleaved(vec![0.0; 8], AudioFormat::mono(48_000));

    // Fill the ring exactly (the renderer hasn't drained anything yet).
    for _ in 0..CMD_CAPACITY {
        engine.play(data.clone(), PlayParams::default()).unwrap();
    }

    // One more overflows → a clean error, and the dropped sample frees here.
    let err = engine
        .play(data.clone(), PlayParams::default())
        .unwrap_err();
    assert_eq!(err, AudioError::QueueFull(CMD_CAPACITY));

    // Drain on the audio side, then the control side can enqueue again.
    let mut out = vec![0.0f32; 128];
    renderer.render(&mut out, 64);
    assert!(engine.play(data, PlayParams::default()).is_ok());
}
