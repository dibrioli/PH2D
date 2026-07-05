//! HR-3: once warm, [`AudioRenderer::render`] must not allocate. Like the
//! `layers_no_alloc_hot_compose` gate in `ph2d-render`, we assert this by
//! **buffer-capacity stability** (deterministic), not by diffing dhat's global
//! `total_blocks` counter (process-wide → flaky). The renderer's only growable
//! buffer is the mix scratch (`clear` + `resize` to a fixed block); the voice
//! pool is fixed at construction and the rings are bounded. So "scratch + pool
//! capacity unchanged across a warm loop" is a complete, deterministic proxy for
//! "zero allocation when warm", and it names the exact regression (a realloc).

use ph2d_audio::{AudioEngine, AudioFormat, PlayParams, SampleData};

#[test]
fn warm_render_does_not_reallocate() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));

    // A handful of looping voices so the mix path runs every block.
    let data = SampleData::from_interleaved(vec![0.2; 960], AudioFormat::mono(48_000));
    for _ in 0..8 {
        engine
            .play(
                data.clone(),
                PlayParams {
                    looping: true,
                    ..PlayParams::default()
                },
            )
            .unwrap();
    }

    const FRAMES: usize = 512;
    let mut out = vec![0.0f32; FRAMES * 2];

    // Warm: the one and only scratch realloc happens on the first block.
    renderer.render(&mut out, FRAMES);
    let warm_scratch = renderer.scratch_capacity();
    let voice_cap = renderer.voice_capacity();
    assert!(
        warm_scratch >= FRAMES * 2,
        "warm-up under-sized the scratch"
    );

    // 200 warm blocks must reuse scratch + pool with zero reallocation.
    for _ in 0..200 {
        renderer.render(&mut out, FRAMES);
    }
    assert_eq!(
        renderer.scratch_capacity(),
        warm_scratch,
        "HR-3 violation: the mix scratch reallocated in the hot path"
    );
    assert_eq!(
        renderer.voice_capacity(),
        voice_cap,
        "the voice pool must never grow"
    );
    assert_eq!(renderer.active_voices(), 8, "looping voices keep sounding");
}
