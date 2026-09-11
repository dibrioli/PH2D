//! **The gate the progress bar rests on.**
//!
//! A bar is only worth its pixels if the number behind it moves *while the work happens*. Two
//! ways to get that wrong, and neither one crashes:
//!
//! - **It never reports.** The bar sits at 0 % for five seconds and then the clip changes. That
//!   is strictly worse than no bar: it says "frozen" and then contradicts itself.
//! - **It reports only at the ends.** `{0.0, 1.0}` passes any test that asks "did progress
//!   arrive?" — and paints exactly the same frozen bar. So the assertion here is not that
//!   progress *exists*, it is that progress is **spread across the run**.
//!
//! The clip is deliberately short (0.5 s): the claim is about the *shape* of the reporting, and
//! a longer clip would only make a debug-build test slower to say the same thing.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use std::sync::Mutex;

const SR: u32 = 48_000; // the model's own rate — no resample, so the run is all model.

/// Half a second of a voiced tone under hiss. Deterministic (splitmix64, no `rand`): a gate that
/// is flaky is a gate that gets ignored.
fn noisy_clip(secs: f32) -> SampleData {
    let tau = std::f32::consts::TAU;
    let mut state = 0x5EEDu64;
    let frames = (SR as f32 * secs) as usize;
    SampleData::from_fn(frames, AudioFormat::new(SR, ChannelLayout::Mono), |i| {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let hiss = (z >> 40) as f32 / 8_388_608.0 - 1.0;
        let t = i as f32 / SR as f32;
        ((tau * 150.0 * t).sin() * 0.2 + hiss * 0.2).clamp(-1.0, 1.0)
    })
}

fn collect_progress(data: &SampleData) -> Vec<f32> {
    let seen = Mutex::new(Vec::new());
    let _ = ph2d_audio_ml::denoise_ml_with_progress(data, 1.0, &|f| {
        seen.lock().expect("no panic while denoising").push(f)
    });
    seen.into_inner().expect("collected")
}

/// The number moves, in range, forwards, all the way across the run — and lands exactly on 1.
#[test]
fn progress_climbs_across_the_run_and_finishes_at_one() {
    let seen = collect_progress(&noisy_clip(0.5));

    assert!(
        !seen.is_empty(),
        "nothing reported: the bar would sit at 0 % for the whole run and then jump"
    );
    for f in &seen {
        assert!(
            (0.0..=1.0).contains(f),
            "reported {f}, which is not a fraction — a bar cannot draw it"
        );
    }
    assert!(
        seen.windows(2).all(|w| w[1] >= w[0]),
        "progress went backwards: {:?}",
        &seen[..seen.len().min(12)]
    );
    assert_eq!(
        seen.last().copied(),
        Some(1.0),
        "the run must end at exactly 1.0 — a bar that stops at 99 % outlives the work it \
         describes, and `tick` would drop it looking unfinished"
    );

    // **The load-bearing assertion.** Not "progress happened" (a `{0, 1}` step function passes
    // that) but "progress happened *during*". A 0.5 s clip is ~150 hops, so a healthy run
    // reports a hundred-odd readings strictly between the ends; a step function reports none.
    let middle = seen.iter().filter(|f| **f > 0.05 && **f < 0.95).count();
    assert!(
        middle >= 8,
        "only {middle} readings landed strictly between the ends, out of {} — this is a step \
         function, and it paints the same frozen bar as reporting nothing at all: {:?}",
        seen.len(),
        &seen[..seen.len().min(12)]
    );

    // And it is spread, not bunched: something is reported in each quarter of the run. Catches a
    // reporter that fires 100 times in the first hop and then goes quiet.
    for q in 0..4 {
        let (lo, hi) = (q as f32 * 0.25, (q + 1) as f32 * 0.25);
        assert!(
            seen.iter().any(|f| *f > lo && *f <= hi),
            "nothing reported in the {}–{} % stretch of the run",
            lo * 100.0,
            hi * 100.0
        );
    }
}

/// **Reporting changes no sample.** The callback is an observer; if it were in the signal path
/// the parity gate would be measuring a different function than the one the app runs.
///
/// This is the presence-sibling of the parity gate: that one proves the output is *right*, this
/// one proves the progress plumbing did not move it.
#[test]
fn watching_the_run_does_not_change_it() {
    let data = noisy_clip(0.2);
    let watched = ph2d_audio_ml::denoise_ml_with_progress(&data, 1.0, &|_| {});
    let plain = ph2d_audio_ml::denoise_ml(&data, 1.0);
    assert_eq!(
        watched.samples(),
        plain.samples(),
        "the reporting wrapper and the plain call must be the same function"
    );

    // `Cell`, because the callback is `Fn` and not `FnMut` — which is the right shape for the
    // one caller that matters: the shell's is `|f| progress.set(f)`, a relaxed atomic store
    // through a shared handle, and that needs no `&mut` of anything.
    let hits = std::cell::Cell::new(0);
    let counted =
        ph2d_audio_ml::denoise_ml_with_progress(&data, 1.0, &|_| hits.set(hits.get() + 1));
    assert!(hits.get() > 0, "the callback never fired");
    assert_eq!(
        counted.samples(),
        plain.samples(),
        "a callback that actually does something must still not perturb a single sample"
    );
}

/// `amount == 0` stays the rack's byte-identical no-op — it returns before the model, so it
/// reports nothing. A bar for work that is not happening is a bar that lies.
#[test]
fn amount_zero_reports_nothing_and_changes_nothing() {
    let data = noisy_clip(0.1);
    let seen = Mutex::new(Vec::new());
    let out = ph2d_audio_ml::denoise_ml_with_progress(&data, 0.0, &|f| {
        seen.lock().expect("no panic").push(f)
    });
    assert_eq!(out.samples(), data.samples(), "amount 0 must be bypass");
    assert!(
        seen.into_inner().expect("collected").is_empty(),
        "a bypass reported progress: there was no work to report on"
    );
}
