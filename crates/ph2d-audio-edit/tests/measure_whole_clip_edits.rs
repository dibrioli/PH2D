//! **A2 of ADR-0117** — the irreducible case, and the one the byte cap exists for.
//!
//! Sixty-four **whole-clip** edits: every sample changes every time, so no delta can be small.
//! Each step is honestly one clip, and 64 of them is 4.2 GB of audio that cannot all be kept. This
//! is where a count-based cap (`MAX_HISTORY = 64`) was a *multiplier* rather than a bound, and
//! where the byte budget earns its keep: undo depth becomes adaptive, and the application survives.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_whole_clip_edits -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, Effect};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;
/// ADR-0117 §4 A2, frozen before the implementation existed.
const BAR_MB: f64 = 512.0;

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
fn sixty_four_whole_clip_edits_stay_inside_the_budget() {
    let secs = 180;
    let one = (secs * 48_000 * 2 * 4) as f64 / MB;
    let profiler = dhat::Profiler::builder().testing().build();

    let mut c = EditClip::new(clip(secs));
    for i in 0..64 {
        // No selection: the target is the whole clip, and Saturate moves every sample.
        c.apply_effect(Effect::Saturate {
            drive: 1.0 + 0.01 * (i as f32 + 1.0),
        });
    }
    let could_undo = c.can_undo();

    let stats = dhat::HeapStats::get();
    drop(profiler);
    let peak = stats.max_bytes as f64 / MB;

    println!("\n=== ADR-0117 A2: 64 whole-clip edits ===");
    println!("clip:        {secs}s stereo 48k = {one:.1} MB");
    println!("peak heap:   {peak:.1} MB   (was 4351 MB — 64 whole clips)");
    println!("bar:         {BAR_MB:.0} MB\n");

    assert!(
        peak <= BAR_MB,
        "A2: peak {peak:.1} MB exceeds the {BAR_MB:.0} MB bar"
    );
    // A budget that silently disables Undo is not a budget, it is a feature removal. The user
    // must still be able to take back what they just did.
    assert!(
        could_undo,
        "A2: the budget ate the whole timeline — Undo is dead"
    );
}
