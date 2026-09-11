//! **ADR-0124: the cost of a range edit must not depend on the length of the clip.**
//!
//! The bug, as reported: *"with this big audio, common operations (like raising the gain) on small
//! selected ranges became slow. Everything here must be real-time."*
//!
//! Measured on the reported fixture — a 3-minute mono clip (34.5 MB), raising the gain on a 100 ms
//! selection:
//!
//! ```text
//! selection FIXED (100 ms), clip growing:
//!    4 s -> 0.76 ms |  30 s -> 5.77 ms |  60 s -> 12.02 ms | 180 s -> 22.37 ms   (linear in the CLIP)
//! clip FIXED (180 s), selection growing 1000x:
//!   10 ms -> 22.4 ms | 100 ms -> 22.4 ms | 1 s -> 22.4 ms | 10 s -> 22.4 ms      (FLAT)
//! ```
//!
//! 22 ms is past a 60 fps frame, on an operation the user repeats — and the selection, the only
//! thing the edit actually touches, did not matter at all.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_range_edit -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::EditClip;

const SR: usize = 48_000;

/// The reported fixture: mono, so 180 s is the 34.5 MB the bug report cites.
fn clip(secs: usize) -> SampleData {
    SampleData::from_fn(
        secs * SR,
        AudioFormat::new(SR as u32, ChannelLayout::Mono),
        |i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25,
    )
}

/// A clip that owns its buffer alone — the ordinary editing case (nothing is playing it).
///
/// The `SampleData` is **moved** in, never cloned: a clone is an `Arc` bump, the doc would not be
/// the sole owner, and every measurement below would silently be of the splice path.
fn edit_clip(secs: usize, sel: std::ops::Range<usize>) -> EditClip {
    let mut c = EditClip::new(clip(secs));
    c.set_selection(Some(sel));
    c
}

/// Time `f`, after warming it up (see `measure_preview.rs`: the first pass over a fresh buffer pays
/// its page faults, and whichever measurement ran first would absorb them).
fn time(label: &str, mut f: impl FnMut()) -> f64 {
    for _ in 0..5 {
        f();
    }
    let n = 50;
    let t = std::time::Instant::now();
    for _ in 0..n {
        f();
    }
    let ms = t.elapsed().as_secs_f64() * 1_000.0 / n as f64;
    println!("  {label:<40} {ms:>9.4} ms");
    ms
}

/// **The gate.** The same edit, on the same selection, in a clip **8× longer**.
///
/// The bar is a RATIO and deliberately so: `ci-test` builds at `opt-level = 1`, so a wall-clock bar
/// tuned in release is a CI landmine that measures the profile rather than the code (the lesson
/// `measure_preview.rs` paid for). A ratio is profile-robust — both sides run in the same build —
/// and it states the claim exactly: **a range edit does not know how long the clip is.**
///
/// Before ADR-0124 this ratio was ~8: the edit rebuilt the buffer, rescanned it to rediscover the
/// range, and rebuilt the whole waveform, three times over the whole clip.
#[test]
fn the_cost_of_a_range_edit_does_not_scale_with_the_clip() {
    // A 1-second selection: big enough to sit well clear of the timer's noise floor, and still
    // 1/180th of the long clip.
    let sel = SR..SR * 2;
    println!("\n=== ADR-0124: the same 1 s selection, in clips 8x apart ===");
    let mut small = edit_clip(22, sel.clone());
    let mut big = edit_clip(180, sel.clone());
    let t_small = time(" 22 s clip", || small.apply_gain(1.0001));
    let t_big = time("180 s clip (8x the audio)", || big.apply_gain(1.0001));

    let ratio = t_big / t_small.max(1e-9);
    println!("\n  ratio: {ratio:.2}x for 8x the clip   (before ADR-0124: ~8x)\n");

    // An honest margin: identical work, so the ratio is 1 plus whatever the machine is doing. Well
    // clear of the 8x that O(clip) would produce, and loose enough not to flake on a busy CI box.
    const BAR: f64 = 3.0;
    assert!(
        ratio < BAR,
        "raising the gain on the SAME selection cost {ratio:.2}x more in a clip 8x longer \
         ({t_small:.4} ms -> {t_big:.4} ms). A range edit is scaling with the clip again: something \
         downstream of the edit is rediscovering the range instead of being told it (ADR-0124)."
    );
}

/// The reported measurement, reproduced: one `apply_gain` on a FIXED selection, clips growing.
#[test]
fn cost_versus_clip_length() {
    println!("\n=== fixed selection (100 ms), clip growing ===");
    for &secs in &[4usize, 30, 60, 180] {
        let mut c = edit_clip(secs, SR..SR + SR / 10);
        time(&format!("{secs:>3} s clip"), || c.apply_gain(1.0001));
    }
}

/// The other half: a FIXED clip, selection growing 1000×. This is the one that should now scale —
/// the cost of an edit is allowed to depend on how much audio it edits, and on nothing else.
#[test]
fn cost_versus_selection_length() {
    println!("\n=== fixed clip (180 s), selection growing 1000x ===");
    for &(label, frames) in &[
        ("10 ms", SR / 100),
        ("100 ms", SR / 10),
        ("1 s", SR),
        ("10 s", SR * 10),
    ] {
        let mut c = edit_clip(180, SR..SR + frames);
        time(label, || c.apply_gain(1.0001));
    }
}
