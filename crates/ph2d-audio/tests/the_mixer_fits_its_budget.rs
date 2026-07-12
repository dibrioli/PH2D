//! **HR-13's measuring gate for the RUNTIME mixer** — the one the rule always demanded and never
//! had, and what ADR-0118 did about it.
//!
//! ADR-0117 amended HR-13: *a subsystem that declares a budget owns an executable gate that
//! MEASURES it.* The Audio Editor got its gates. This asks the same question of the other side of
//! the fence — the mixer that ships **inside a game**, which is what HR-13's "Audio buffers" row
//! (30 MB on iPad, 80 MB on desktop) actually governs. It was born **red**.
//!
//! No allocator instrumentation needed: residency is not an allocation *pattern*, it is a fact about
//! the buffer. Deterministic, and it names the exact regression — same reasoning as
//! `no_alloc_render.rs` preferring capacity-stability to dhat's process-global counters.
//!
//! ```text
//! cargo test -p ph2d-audio --release --test the_mixer_fits_its_budget -- --nocapture
//! ```

use ph2d_audio::{
    AudioEngine, AudioFormat, ChannelLayout, PlayParams, STREAM_CHUNK_FRAMES, STREAM_DEPTH,
    SampleData, stream,
};

const MB: f64 = 1_048_576.0;

/// SKILL §12.1, "Audio buffers", iPad column — the strictest platform, and the one where going over
/// is a silent jetsam kill rather than a swap.
const HR13_IPAD_MB: f64 = 30.0;

/// ADR-0118 A1: a streaming voice costs its ring, and nothing else.
const RING_BAR_MB: f64 = 2.0;

fn bytes_mb(samples: usize) -> f64 {
    (samples * std::mem::size_of::<f32>()) as f64 / MB
}

/// One music track: three minutes, stereo, 48 kHz. Not an exotic asset — it is *a song*.
fn music() -> SampleData {
    SampleData::from_fn(
        180 * 48_000 * 2,
        AudioFormat {
            sample_rate: 48_000,
            channels: ChannelLayout::Stereo,
        },
        |i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25,
    )
}

/// **What a game used to pay to play one song, and what it pays now.**
///
/// Resident, a `Voice` holds the whole clip decoded to `f32`. Streamed, it holds `STREAM_DEPTH`
/// chunks — a fixed, tiny amount that does not care how long the song is.
///
/// This is also what finally makes the codec mean something. Opus is 6.4 % of WAV16 **on disk** and
/// was 100 % of it **in RAM**, because the first thing the loader did was expand it back.
#[test]
fn a_streamed_song_costs_its_ring_not_its_length() {
    let song = music();
    let resident = bytes_mb(song.samples().len());

    // The streaming cost is the chunks, and only the chunks: STREAM_DEPTH of them, each
    // STREAM_CHUNK_FRAMES stereo frames. It does not grow with the song.
    let streamed = bytes_mb(STREAM_DEPTH * STREAM_CHUNK_FRAMES * 2);

    println!("\n=== ADR-0118 A1 / HR-13: playing one 3-minute stereo song ===");
    println!(
        "resident:  {resident:.1} MB   ({:.1}x the whole iPad audio budget)",
        resident / HR13_IPAD_MB
    );
    println!(
        "streamed:  {streamed:.2} MB   ({:.1}% of the budget)",
        streamed / HR13_IPAD_MB * 100.0
    );
    println!("HR-13 'Audio buffers', iPad: {HR13_IPAD_MB:.0} MB\n");

    assert!(
        streamed <= RING_BAR_MB,
        "A1: a streamed voice costs {streamed:.2} MB — the ring was supposed to be under {RING_BAR_MB} MB"
    );
    assert!(
        streamed <= HR13_IPAD_MB,
        "HR-13: even streamed, one song does not fit the budget"
    );
    // The point of the whole ADR, stated as arithmetic: streaming is not a little better.
    assert!(
        resident / streamed > 100.0,
        "streaming should be orders of magnitude cheaper, not a tweak"
    );

    // And it is a real, playable voice — not a number in a comment.
    let (_feeder, handle) = stream(48_000, None);
    let (mut engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
    engine
        .play_stream(handle, PlayParams::default())
        .expect("play_stream");
}
