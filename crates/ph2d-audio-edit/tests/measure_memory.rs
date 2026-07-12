//! **A1 of ADR-0117** — the gate that would have caught the 4351 MB.
//!
//! Sixty-four *selection* edits on a three-minute stereo clip: the realistic long-clip session
//! (you fix a click, you denoise a passage — you do not saturate three minutes of ambience).
//!
//! One `#[test]` per binary, deliberately: dhat's counters are process-global, and `cargo test`
//! runs a binary's tests on threads. Two profilers in one process race.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_memory -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, Effect};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;

/// ADR-0117 §4 A1, as **amended** in §4.1.
///
/// The bar I froze — a flat 128 MB — was arithmetically impossible, and the amendment says so
/// rather than quietly widening it. **Two whole buffers are irreducible**: a new one must exist
/// before the old one can be released, and the product's path is always `render_effect` (the
/// preview the mixer *plays* before you commit) followed by `commit_rendered`. Two copies of a
/// 65.9 MB clip is 131.8 MB before a single byte of history.
///
/// So the bar is **structural, not absolute** — which is what it should have been from the start:
/// the editor holds the clip, one buffer under construction, and deltas. **Not N clips.** That is
/// the property the 4351 MB violated, and it holds at any clip length.
const BAR: fn(f64) -> f64 = |clip_mb| 2.0 * clip_mb + 32.0;

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
fn sixty_four_selection_edits_on_a_three_minute_clip() {
    let secs = 180;
    let one = (secs * 48_000 * 2 * 4) as f64 / MB;
    let profiler = dhat::Profiler::builder().testing().build();

    let mut c = EditClip::new(clip(secs));
    for i in 0..64 {
        // A one-second selection, walking down the clip. Each edit touches 0.56% of the audio.
        let at = (i % 120) * 48_000;
        c.set_selection(Some(at..at + 48_000));
        c.apply_effect(Effect::Saturate {
            drive: 1.0 + 0.01 * (i as f32 + 1.0),
        });
    }

    let stats = dhat::HeapStats::get();
    drop(profiler);
    let peak = stats.max_bytes as f64 / MB;
    let bar = BAR(one);

    println!("\n=== ADR-0117 A1: 64 selection edits ===");
    println!("clip:        {secs}s stereo 48k = {one:.1} MB");
    println!("peak heap:   {peak:.1} MB   (was 4351 MB with the snapshot timeline)");
    println!("bar:         {bar:.1} MB   (2x the clip + 32 MB of deltas)\n");

    assert!(
        peak <= bar,
        "A1: peak {peak:.1} MB exceeds the {bar:.1} MB bar — the timeline is holding clips again"
    );
    // And the floor: if this ever drops near ONE clip, the preview buffer stopped existing, which
    // would mean the audition path is gone. A gate that only has a ceiling can pass by breaking
    // the feature.
    assert!(
        peak > one,
        "A1: peak {peak:.1} MB is below one clip — did the edit actually run?"
    );
}
