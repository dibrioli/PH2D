//! **The multiband's own A4** — the rack's hungriest effect, measured rather than assumed.
//!
//! ADR-0117 amended HR-13: whoever declares a budget owns a gate that MEASURES one. The
//! multiband is the effect that most deserves it, because it is the only one that has to
//! *materialise* intermediate audio — you cannot compress a band without first having the
//! band. A naive build holds all of it at once (three band buffers, three compressed copies,
//! an accumulator: seven clip-sized blocks live, which on a 3-minute clip is 900 MB), and
//! nothing in the rack's own gates would say a word about it.
//!
//! What the implementation actually does is one band at a time, summing into the output:
//! the output IS the accumulator, and only one band is alive at any moment. So the peak is
//! **three** clip-sized buffers (output + the band + its compressed copy), not seven, and it
//! does not grow with the number of bands.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_multiband -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, Effect};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;

/// Output + one band + that band's compressed copy. The `0.2` is the allocator's rounding,
/// not room for a fourth buffer — a build that held every band at once would land at 7x.
const BAR_CLIPS: f64 = 3.2;

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
fn a_multiband_render_holds_one_band_at_a_time() {
    let secs = 60;
    let one = (secs * 48_000 * 2 * 4) as f64 / MB;

    let c = EditClip::new(clip(secs));
    let profiler = dhat::Profiler::builder().testing().build();
    let out = c.render_effect(Effect::Multiband {
        threshold: 0.3,
        ratio: 8.0,
        attack_secs: 0.005,
        release_secs: 0.1,
    });
    let stats = dhat::HeapStats::get();
    drop(profiler);
    std::hint::black_box(&out);

    let peak = stats.max_bytes as f64 / MB;
    println!("\n=== the multiband's peak heap ===");
    println!("clip:        {secs}s stereo 48k = {one:.1} MB");
    println!(
        "allocated:   {:.1} MB across {} blocks",
        stats.total_bytes as f64 / MB,
        stats.total_blocks
    );
    println!(
        "peak heap:   {peak:.1} MB   ({:.2}x the clip; the bar is {BAR_CLIPS}x)",
        peak / one
    );
    println!("             a build holding every band at once would be 7x\n");

    assert!(
        peak < one * BAR_CLIPS,
        "the multiband is holding {:.2} clips at once (bar: {BAR_CLIPS}) — it is keeping \
         every band alive instead of summing them one at a time",
        peak / one
    );
    // ...and it really did render three bands, rather than bailing out early.
    assert!(
        peak > one * 2.0,
        "peak {peak:.1} MB is under two clips — did the bands actually get materialised?"
    );
}
