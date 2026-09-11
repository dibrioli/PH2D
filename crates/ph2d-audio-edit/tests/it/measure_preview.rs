//! **What a knob drag actually costs** (ADR-0117 §5, the number that was registered rather than
//! hidden).
//!
//! Dragging a slider re-renders the effect **every frame**. The rack applies to a SELECTION, so
//! the DSP is O(selection) already — but the result has to be a contiguous `SampleData` for the
//! mixer to play, so every frame still allocates and copies a whole-clip buffer.
//!
//! This measures the two separately, on a 3-minute clip, for a **1-second selection**. If the
//! copy dominates, O(selection) preview is a real feature and the ADR's double-buffer scratch is
//! the way to get it. If the DSP dominates, the copy is not the problem and the ADR would be
//! solving the wrong thing.
//!
//! ```text
//! cargo test -p ph2d-audio-edit --release --test measure_preview -- --nocapture
//! ```

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, Effect};

const SR: usize = 48_000;
/// The clip the ADR was worried about.
const SECS: usize = 180;

fn clip() -> SampleData {
    SampleData::from_fn(
        SECS * SR * 2,
        AudioFormat::new(SR as u32, ChannelLayout::Stereo),
        |i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25,
    )
}

/// Time `f`, after warming it up.
///
/// The warm-up is not a nicety: the first pass over a fresh 65 MB buffer pays the page faults for
/// it, and without a warm-up whichever measurement runs FIRST absorbs them and comes out slowest.
/// That is how the same copy measured 11 ms in one run and 23 ms in the next, and how "the DSP
/// costs 0 ms" appeared — a subtraction going negative. The measurement was reporting the order it
/// ran in.
fn time(label: &str, mut f: impl FnMut()) -> f64 {
    for _ in 0..3 {
        f();
    }
    let n = 10;
    let t = std::time::Instant::now();
    for _ in 0..n {
        f();
    }
    let ms = t.elapsed().as_secs_f64() * 1_000.0 / n as f64;
    println!("  {label:<44} {ms:>8.2} ms");
    ms
}

#[test]
fn what_one_frame_of_a_knob_drag_costs() {
    let data = clip();
    let mb = (data.samples().len() * 4) as f64 / 1_048_576.0;
    println!("\n=== ADR-0117 §5: the cost of ONE frame of a knob drag ===");
    println!("clip: {SECS}s stereo 48k = {mb:.1} MB\n");

    // The floor: a whole-clip copy, doing nothing at all. This is what the preview pays for being
    // contiguous, and it is what an O(selection) preview would remove.
    let copy = time("whole-clip copy (the contiguity tax)", || {
        let out = SampleData::map_in_place(&data, |_| {});
        std::hint::black_box(&out);
    });

    // A ONE-SECOND selection out of three minutes: the DSP touches 0.55 % of the clip.
    let mut sel = EditClip::new(data.clone());
    sel.set_selection(Some(SR..SR * 2));
    let fx = Effect::Compress {
        threshold: 0.3,
        ratio: 8.0,
        attack_secs: 0.005,
        release_secs: 0.1,
    };
    let one_sec = time("render a 1 s selection (copy + DSP)", || {
        let out = sel.render_effect(fx);
        std::hint::black_box(&out);
    });

    // ...and the whole clip, for scale: this is the DSP actually doing 180x the work.
    let mut all = EditClip::new(data.clone());
    all.set_selection(None);
    let whole = time("render the WHOLE clip (copy + DSP)", || {
        let out = all.render_effect(fx);
        std::hint::black_box(&out);
    });

    // The DSP itself, derived from the whole-clip render: 180 s of it costs `whole - copy`, so one
    // second costs a 180th of that. NOT `one_sec - copy` -- that difference is the SPLICE (a
    // structured copy, more expensive than a raw one), and calling it "the DSP" would have put a
    // number 28x too big on the screen.
    let dsp_per_sec = (whole - copy).max(0.0) / SECS as f64;
    println!("\n  the DSP on 1 s of audio (derived):           {dsp_per_sec:>8.2} ms");
    println!(
        "  the whole-clip work a render pays anyway:    {copy:>8.2} ms  ({:.0}% of the frame)",
        copy / one_sec * 100.0
    );

    // ...and the fast path (ADR-0120): rewrite the region of a buffer we already own.
    let mut scratch = SampleData::map_in_place(&data, |_| {});
    let incremental = time("ADR-0120: rewrite the region in place", || {
        let (r, region) = sel.render_effect_region(fx).expect("length-preserving");
        let out = scratch.get_mut().expect("sole owner here");
        out[r.start * 2..r.end * 2].copy_from_slice(region.samples());
        std::hint::black_box(&scratch);
    });
    println!(
        "\n  knob drag, before:  {one_sec:.2} ms   after: {incremental:.2} ms   ({:.1}x)\n",
        one_sec / incremental.max(1e-9)
    );
    assert!(
        incremental < one_sec * 0.5,
        "the incremental preview ({incremental:.2} ms) is not meaningfully faster than the full \
         render ({one_sec:.2} ms) -- ADR-0120 buys nothing"
    );

    // **The only bar here is a RATIO, and deliberately so.**
    //
    // A wall-clock bar in this suite is a CI landmine: `ci-test` builds the workspace at
    // `opt-level = 1`, so the DSP runs an order of magnitude slower than in release while the
    // memcpy — a libc intrinsic — does not budge. A bar tuned on the numbers above went red on the
    // very first `nextest` run, and it was measuring the PROFILE, not the code. (The rack's own
    // fingerprint test refuses pinned values for the same reason.)
    //
    // What IS profile-robust: the incremental path does strictly less work — it never copies the
    // whole clip. That inequality holds at any optimisation level, on any machine, and it is the
    // entire claim of ADR-0120. The claims that must never silently break (byte-identity, and the
    // fast path actually firing) are gated deterministically elsewhere, where they belong.
    assert!(
        incremental < one_sec * 0.5,
        "the incremental preview ({incremental:.2} ms) is not meaningfully faster than the full \
         render ({one_sec:.2} ms) -- ADR-0120 buys nothing"
    );
}
