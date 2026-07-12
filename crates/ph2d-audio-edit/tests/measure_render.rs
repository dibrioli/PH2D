//! **A4 of ADR-0117** — a whole-clip render must allocate its output **once**.
//!
//! The output buffer is irreducible: rendering produces a new clip, and a new clip is N samples.
//! What was *not* irreducible was building it twice — a `Vec`, and then an `Arc<[f32]>` copied out
//! of it, because an `Arc` keeps its refcount inline before the data and can never adopt a `Vec`'s
//! allocation. Two blocks, twice the peak, for one buffer.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_render -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, Effect};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;
/// ADR-0117 §4 A4, frozen before the implementation existed: **one** allocation, not two.
const BAR_BLOCKS: u64 = 1;

fn clip(secs: usize) -> SampleData {
    let frames = secs * 48_000;
    SampleData::from_interleaved(
        (0..frames * 2)
            .map(|i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25)
            .collect(),
        AudioFormat {
            sample_rate: 48_000,
            channels: ChannelLayout::Stereo,
        },
    )
}

#[test]
fn a_whole_clip_render_allocates_its_output_once() {
    let secs = 60;
    let one = (secs * 48_000 * 2 * 4) as f64 / MB;

    let c = EditClip::new(clip(secs));
    let profiler = dhat::Profiler::builder().testing().build();
    let t = std::time::Instant::now();
    let out = c.render_effect(Effect::Saturate { drive: 1.5 });
    let elapsed = t.elapsed();
    let stats = dhat::HeapStats::get();
    drop(profiler);
    std::hint::black_box(&out);

    let peak = stats.max_bytes as f64 / MB;
    println!("\n=== ADR-0117 A4: one whole-clip render ===");
    println!("clip:        {secs}s stereo 48k = {one:.1} MB");
    println!(
        "allocated:   {:.1} MB across {} blocks   (was 43.9 MB / 2 blocks)",
        stats.total_bytes as f64 / MB,
        stats.total_blocks
    );
    println!("peak heap:   {peak:.1} MB   ({:.2}x the clip)", peak / one);
    println!("wall time:   {elapsed:?}   (was 18.5 ms)\n");

    assert_eq!(
        stats.total_blocks, BAR_BLOCKS,
        "A4: a whole-clip render must allocate exactly ONE buffer; it allocated {}",
        stats.total_blocks
    );
    // The peak is the output itself and nothing more. `1.05` leaves room for the allocator's
    // rounding, not for a second buffer.
    assert!(
        peak < one * 1.05,
        "A4: peak {peak:.1} MB is more than the output buffer ({one:.1} MB) — it is being copied"
    );
}
