//! **ADR-0124, the deterministic half.** The timing gate (`measure_range_edit.rs`) states the claim
//! the user feels; this one states the same claim in a number that cannot flake.
//!
//! A range edit used to move ~138 MB of memory to touch 0.4 MB of audio: the splice rebuilt the
//! whole buffer, the history rescanned both copies of it, and the peak cache rebuilt the whole
//! waveform. Bytes allocated is not a proxy for that — it **is** that, and unlike a stopwatch it
//! reads the same on a busy CI box as on an idle desk.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_range_edit_alloc -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::EditClip;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const SR: usize = 48_000;
const MB: f64 = 1_048_576.0;

fn edit_clip(secs: usize, sel: std::ops::Range<usize>) -> EditClip {
    // Moved in, never cloned: a clone is an `Arc` bump and the doc would not own its buffer.
    let mut c = EditClip::new(SampleData::from_fn(
        secs * SR,
        AudioFormat::new(SR as u32, ChannelLayout::Mono),
        |i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25,
    ));
    c.set_selection(Some(sel));
    c
}

/// Bytes allocated by one `apply_gain` on a 100 ms selection, in a clip of `secs`.
fn gain_bytes(secs: usize) -> f64 {
    let mut c = edit_clip(secs, SR..SR + SR / 10);
    // Warm: the first edit on a fresh clip settles the history's allocations, and we are measuring
    // the steady state the user is actually repeating.
    c.apply_gain(1.0001);

    let profiler = dhat::Profiler::builder().testing().build();
    let before = dhat::HeapStats::get().total_bytes;
    c.apply_gain(1.0001);
    let after = dhat::HeapStats::get().total_bytes;
    drop(profiler);
    (after - before) as f64 / MB
}

/// **The gate.** Editing 100 ms costs 100 ms worth of memory — in a 22 s clip and in a 180 s one.
#[test]
fn a_range_edit_allocates_for_the_range_not_for_the_clip() {
    let small = gain_bytes(22);
    let big = gain_bytes(180);
    let sel_mb = (SR / 10 * 4) as f64 / MB;
    let clip_mb = (180 * SR * 4) as f64 / MB;

    println!("\n=== ADR-0124: what one gain nudge allocates ===");
    println!("the selection itself:  {sel_mb:.3} MB  (100 ms mono)");
    println!("the 180 s clip:        {clip_mb:.1} MB");
    println!(" 22 s clip -> {small:.3} MB");
    println!("180 s clip -> {big:.3} MB   (8x the audio)\n");

    // The edit must not allocate a clip. The old path allocated one whole buffer for the splice and
    // another for the rebuilt waveform; either alone would blow this bar by two orders of magnitude.
    const BAR_MB: f64 = 1.0;
    assert!(
        big < BAR_MB,
        "one gain nudge on a 100 ms selection allocated {big:.3} MB in a 180 s clip — the range is \
         {sel_mb:.3} MB. Something is rebuilding a whole-clip buffer (ADR-0124)."
    );
    // ...and 8x the clip must not cost more. This is the claim, stated in the only number that does
    // not depend on how busy the machine is.
    assert!(
        big <= small * 1.5 + 0.01,
        "the same 100 ms edit allocated {small:.3} MB in a 22 s clip and {big:.3} MB in a 180 s one: \
         a range edit is scaling with the clip again (ADR-0124)."
    );
}
