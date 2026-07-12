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
    let warm_bus_scratch = renderer.bus_scratch_capacity();
    let warm_send_scratch = renderer.send_scratch_capacity();
    let warm_delay_send_scratch = renderer.delay_send_scratch_capacity();
    let voice_cap = renderer.voice_capacity();
    assert!(
        warm_scratch >= FRAMES * 2,
        "warm-up under-sized the scratch"
    );
    assert!(
        warm_bus_scratch >= FRAMES * 2,
        "warm-up under-sized the per-bus scratch"
    );
    assert!(
        warm_send_scratch >= FRAMES * 2,
        "warm-up under-sized the reverb-send scratch"
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
        renderer.bus_scratch_capacity(),
        warm_bus_scratch,
        "HR-3 violation: the per-bus scratch reallocated in the hot path"
    );
    assert_eq!(
        renderer.send_scratch_capacity(),
        warm_send_scratch,
        "HR-3 violation: the reverb-send scratch reallocated in the hot path"
    );
    assert_eq!(
        renderer.delay_send_scratch_capacity(),
        warm_delay_send_scratch,
        "HR-3 violation: the delay-send scratch reallocated in the hot path"
    );
    assert_eq!(
        renderer.voice_capacity(),
        voice_cap,
        "the voice pool must never grow"
    );
    assert_eq!(renderer.active_voices(), 8, "looping voices keep sounding");
}

/// **A3 of ADR-0118.** A *streaming* voice must not allocate on the audio thread either — and it is
/// the one that most easily could, because it handles buffers rather than just indexing one.
///
/// The trap it guards is specific: a chunk the audio thread has finished with must go **back through
/// the recycling ring**, never simply be dropped. Dropping it there is a `free()`, and a `free()` is
/// an allocation running backwards. If someone ever "simplifies" `StreamHandle::pull` by letting the
/// spent chunk fall out of scope, the audio thread starts calling the allocator every 43 ms — the
/// classic source of a click nobody can reproduce.
///
/// dhat cannot see this (its counters are process-global, and the producer thread allocates freely
/// and legitimately). The property that CAN be checked, deterministically, is the one that matters:
/// the chunks that exist are the same chunks throughout — **none is created, none is destroyed**,
/// they only go round. `STREAM_DEPTH` chunks in, `STREAM_DEPTH` chunks out, forever.
#[test]
fn a_streaming_voice_recycles_its_chunks_instead_of_freeing_them() {
    use ph2d_audio::{ChannelLayout, STREAM_CHUNK_FRAMES, STREAM_DEPTH, stream};

    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    let (feeder, handle) = stream(48_000, None);
    engine
        .play_stream(handle, PlayParams::default())
        .expect("play_stream");

    const FRAMES: usize = 512;
    let mut out = vec![0.0f32; FRAMES * 2];

    // Run long enough that the ring must turn over many times: 400 blocks of 512 frames is
    // 204 800 frames, i.e. ~100 chunks through a 4-chunk ring.
    let mut recycled = 0usize;
    for _ in 0..400 {
        // The producer: take back every chunk the audio thread returned, refill it, submit it. If
        // the audio thread were *dropping* spent chunks instead of recycling them, this loop would
        // run dry — there would be nothing to take.
        while let Some(mut chunk) = feeder.take_empty() {
            let buf = chunk.buffer_mut();
            for (k, s) in buf.iter_mut().enumerate() {
                *s = ((k % 101) as f32 / 101.0) * 0.4 - 0.2;
            }
            chunk.set_frames(STREAM_CHUNK_FRAMES);
            if feeder.submit(chunk).is_err() {
                break;
            }
            recycled += 1;
        }
        renderer.render(&mut out, FRAMES);
    }

    assert!(
        recycled > STREAM_DEPTH * 10,
        "the ring stopped turning over after {recycled} chunks — the audio thread is not returning \
         them, which means it is FREEING them (HR-3)"
    );
    assert!(
        out.iter().any(|s| *s != 0.0),
        "the streamed voice rendered nothing, so this proved nothing"
    );
    // The format is untouched by any of it — no growth, no reallocation of the mixer's own buffers.
    assert_eq!(engine.format().channels, ChannelLayout::Stereo);
}
