//! **A3 of ADR-0117** — editing one second of a three-minute clip.
//!
//! The work is 1/180th of the clip. The old splice cost **133.3 MB in 6 blocks** to touch 0.37 MB
//! of audio (364×), because it built the whole clip in a `Vec` and then let `Arc::from(Vec)` build
//! it a second time. Every untouched sample was copied **twice, for the privilege of not being
//! touched**.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_selection -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, Effect};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;
/// ADR-0117 §4 A3, frozen before the implementation existed.
const BAR_MB: f64 = 70.0;
const BAR_BLOCKS: u64 = 3;

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
fn editing_one_second_of_a_three_minute_clip() {
    let secs = 180;
    let sel_mb = (48_000 * 2 * 4) as f64 / MB;

    let mut c = EditClip::new(clip(secs));
    c.set_selection(Some(60 * 48_000..61 * 48_000));

    // Profile the render only: the clip and its peak cache are already built and paid for.
    let profiler = dhat::Profiler::builder().testing().build();
    let t = std::time::Instant::now();
    let out = c.render_effect(Effect::Saturate { drive: 1.5 });
    let elapsed = t.elapsed();
    let stats = dhat::HeapStats::get();
    drop(profiler);
    std::hint::black_box(&out);

    let alloc = stats.total_bytes as f64 / MB;
    println!("\n=== ADR-0117 A3: a 1s selection in a 180s clip ===");
    println!("selection:   {sel_mb:.2} MB  (0.56% of the clip)");
    println!(
        "allocated:   {alloc:.1} MB across {} blocks   (was 133.3 MB / 6 blocks)",
        stats.total_blocks
    );
    println!("wall time:   {elapsed:?}   (was 26.3 ms)");
    println!("bar:         <= {BAR_MB:.0} MB / <= {BAR_BLOCKS} blocks\n");

    assert!(
        alloc <= BAR_MB,
        "A3: allocated {alloc:.1} MB exceeds the {BAR_MB:.0} MB bar"
    );
    assert!(
        stats.total_blocks <= BAR_BLOCKS,
        "A3: {} blocks exceeds the {BAR_BLOCKS}-block bar — something is building a Vec and then \
         copying it into an Arc again",
        stats.total_blocks
    );
}
