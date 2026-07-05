//! A full pool steals the oldest-quietest voice instead of growing, and the
//! displaced voice's sample is returned to the control thread to be dropped.

use ph2d_audio::{AudioEngine, AudioFormat, MAX_VOICES, PlayParams, SampleData};

#[test]
fn steals_when_full_and_never_exceeds_capacity() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    // 0.1 s sample — longer than a block, so nothing finishes on its own here.
    let data = SampleData::from_interleaved(vec![0.0; 4_800], AudioFormat::mono(48_000));

    for _ in 0..(MAX_VOICES + 1) {
        engine.play(data.clone(), PlayParams::default()).unwrap();
    }

    // Drain the commands (the overflow play steals during this render).
    let mut out = vec![0.0f32; 256];
    renderer.render(&mut out, 128);

    assert_eq!(renderer.voice_capacity(), MAX_VOICES);
    assert_eq!(
        renderer.active_voices(),
        MAX_VOICES,
        "the pool caps at MAX_VOICES; the 65th play steals, not grows"
    );

    // The stolen voice's sample rode the return ring back for off-thread drop.
    assert!(
        engine.pending_returns() >= 1,
        "stealing must return the displaced sample"
    );
    engine.collect_returns();
    assert_eq!(engine.pending_returns(), 0);
}
