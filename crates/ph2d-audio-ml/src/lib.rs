#![forbid(unsafe_code)]
//! `ph2d-audio-ml` — native DeepFilterNet denoise (W7, ADR-0123).
//!
//! ## Why this is its own crate
//!
//! It is the **only** place the ML inference stack (`tract`, and the DeepFilterNet runner
//! `libDF`) lives. That is the same containment that keeps `realfft` inside
//! `ph2d-audio-spectral`, libvorbis inside `ph2d-audio-encode` and symphonia inside
//! `ph2d-audio-decode`: no heavy dependency is allowed to reach the RT mixer (`ph2d-audio`),
//! which must stay allocation-free and predictable. This crate runs on the **control thread**,
//! offline, so HR-3 (no-alloc) and HR-5 (no transcendentals) do not constrain it — it may
//! allocate and run a neural network freely.
//!
//! The whole crate is behind the shell's `audio-ml` feature (OFF by default): a build that does
//! not ask for AI denoise never compiles a line of `tract` and never pulls the 7.6 MB model.
//!
//! ## Why ML at all — the number that authorised it
//!
//! Our spectral denoise (W5, Ephraim-Malah) already passed its gate. DeepFilterNet is not a
//! marginal improvement over it: on the DeepFilterNet author's own 0 dB-SNR fixture it beats
//! the W5 by **+12 dB** of SI-SDR (ADR-0123 §3.5). The gain — not a wish — is what bought the
//! dependency.
//!
//! ## What DeepFilterNet needs, and what this crate does at the boundary
//!
//! The model is trained at a fixed **48 kHz** and consumes **hop-sized** blocks (160 samples).
//! It is causal-with-lookahead: its output lags the input by a fixed delay (STFT + model
//! lookahead), which the reference CLI compensates with `-D`. So the wrapper:
//!
//! 1. resamples the clip to 48 kHz at the boundary (and back to its own rate on the way out),
//! 2. de-interleaves to the channel-major layout the model wants,
//! 3. pads by the model delay so the tail is flushed, feeds it in hop-sized blocks,
//! 4. slices the delay back off so the output lands on the input's timeline,
//! 5. blends dry/wet by `amount` — `amount == 1.0` is the pure model output (CLI parity),
//!    `amount == 0.0` returns the input **untouched, byte for byte**, the neutral point every
//!    effect in the rack promises.
//!
//! ## Reporting progress without learning what a UI is
//!
//! A 3-minute take is ~5 s of inference, which is long enough that the editor wants a progress
//! bar ([`ph2d_editor_core::progress`]). So [`denoise_ml_with_progress`] takes a
//! **`&dyn Fn(f32)`** — not the editor's `Progress` handle, and not any other type from up
//! there. The callback is the narrowest contract that answers the question: the caller learns
//! how far along the run is, and this crate stays a DSP crate that a headless tool, a test or a
//! future CLI can use without linking an editor. The containment argument that put `tract` in
//! here in the first place cuts both ways — nothing heavy gets in, and no UI gets in either.

use df::tract::{DfParams, DfTract, ReduceMask, RuntimeParams};
// `slice_axis`, not the `s!` macro: `s!` expands to code carrying `#[allow(unsafe_code)]`,
// which this crate's `#![forbid(unsafe_code)]` refuses to downgrade (E0453). The methods are
// plain calls — the `unsafe` stays inside ndarray, where it belongs.
use ndarray::{Array2, Axis, Slice};
use ph2d_audio::{AudioFormat, SampleData};

/// The rate DeepFilterNet is trained at. A clip at any other rate is resampled to this on the
/// way in and back to its own rate on the way out — the model only ever sees 48 kHz.
const MODEL_SR: u32 = 48_000;

/// Suppress noise in `data` with DeepFilterNet3. `amount` runs 0 (bypass) to 1 (the full model
/// output). DeepFilterNet learns the noise itself — unlike the W5 denoise, there is no profile
/// to learn first.
///
/// **`amount == 0` returns the input untouched, byte for byte** — the same neutral-point
/// guarantee every effect in the rack makes. A tool whose "off" is a resynthesis is a tool you
/// cannot A/B.
///
/// Values between 0 and 1 are a dry/wet blend of the model's output with the (time-aligned)
/// input; `amount == 1.0` is the pure model output, which is what the reference CLI produces and
/// what the parity gate checks.
pub fn denoise_ml(data: &SampleData, amount: f32) -> SampleData {
    denoise_ml_with_progress(data, amount, &|_| {})
}

/// [`denoise_ml`], reporting how far along it is.
///
/// `on_progress` is called with a fraction in `0.0..=1.0` as the run advances, and is called
/// with `1.0` exactly once, at the end. Everything else about the function — every sample it
/// returns — is identical to [`denoise_ml`], which is this function with a callback that does
/// nothing. There is one code path, so there is no second implementation to drift.
///
/// **What the fraction measures.** The model pass, hop by hop. The resample at either boundary
/// is not in it: the model costs ~30 ms per second of audio while a resample costs well under
/// one, so weighting them would be inventing precision. A clip that is not already at 48 kHz
/// therefore sits at 0 % for a beat before the bar starts to move.
///
/// The callback runs **on the calling thread**, once per 160-sample hop. Make it cheap — the
/// intended body is a relaxed atomic store ([`ph2d_editor_core::progress::Progress::set`]).
pub fn denoise_ml_with_progress(
    data: &SampleData,
    amount: f32,
    on_progress: &dyn Fn(f32),
) -> SampleData {
    let amount = amount.clamp(0.0, 1.0);
    if amount <= 0.0 || data.is_empty() {
        return data.clone();
    }
    let fmt = data.format();
    let n_ch = fmt.channel_count().max(1);

    // 48 kHz at the boundary, keeping the channel layout (only the rate changes). A clip already
    // at 48 kHz is passed through byte-for-byte by `conform`, so the fixture path costs no resample.
    let target48 = AudioFormat::new(MODEL_SR, fmt.channels);
    let at48 = ph2d_audio_edit::conform(data, target48);
    let frames = at48.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let src = at48.samples();

    // De-interleave into the channel-major layout `DfTract::process` wants: shape [n_ch, frames].
    let mut planar = Array2::<f32>::zeros((n_ch, frames));
    for f in 0..frames {
        for c in 0..n_ch {
            planar[[c, f]] = src[f * n_ch + c];
        }
    }

    let enh = enhance_48k(&planar, n_ch, on_progress);

    // Dry/wet blend, re-interleaved. `wet == 1.0` (amount 1.0) is the pure model output.
    let wet = amount;
    let dry = 1.0 - amount;
    let out48 = SampleData::from_fn(frames * n_ch, target48, |i| {
        let f = i / n_ch;
        let c = i % n_ch;
        wet * enh[[c, f]] + dry * src[f * n_ch + c]
    });

    // Back to the clip's own rate (a no-op, byte-identical, when it was already 48 kHz).
    let out = ph2d_audio_edit::conform(&out48, fmt);
    // Say so once the work is really over — after the resample back, not after the last hop.
    // A bar that reads 100 % while the caller is still busy is the same lie as a bar that does
    // not move, told at the other end.
    on_progress(1.0);
    out
}

/// Run DeepFilterNet over a 48 kHz channel-major buffer and return the enhanced buffer, the same
/// length as the input and **on the input's timeline** (the model delay compensated).
///
/// The model is causal with a fixed lookahead: block `k` of its output is the enhanced version
/// of a block `delay` samples earlier. We pad the input by `delay` (rounded up to a whole number
/// of hops) so the last real samples flush through the lookahead, process every full hop, then
/// slice the leading `delay` back off — the mirror of the reference CLI's `-D`, except we keep
/// the tail the CLI drops.
///
/// Parameters mirror the reference CLI exactly (`RuntimeParams::default()` + its argument
/// defaults): no post-filter, `atten_lim_db = 100` (full reduction, no limit), thresholds
/// `(-15, 35, 35)`, `reduce_mask = MAX`. Only then is the output the CLI's, which the parity gate
/// requires.
///
/// `on_progress` is called once per hop with the fraction of hops consumed. The hop loop is the
/// natural home for it and the only honest one: it is where the seconds actually go, and it is
/// the one part of the run whose remaining cost is known (hops are uniform — the model does the
/// same work on every one).
fn enhance_48k(planar: &Array2<f32>, n_ch: usize, on_progress: &dyn Fn(f32)) -> Array2<f32> {
    let dfp = DfParams::default(); // embedded DFN3 weights (feature "default-model")
    let rp = RuntimeParams::default_with_ch(n_ch)
        .with_atten_lim(100.0)
        .with_thresholds(-15.0, 35.0, 35.0)
        .with_mask_reduce(ReduceMask::MAX);
    let mut model = DfTract::new(dfp, &rp).expect("DeepFilterNet3 model failed to initialise");

    let hop = model.hop_size;
    // STFT delay + model lookahead, in samples — the same expression the reference CLI uses.
    let delay = model.fft_size - hop + model.lookahead * hop;

    let frames = planar.ncols();
    // Round up so the input is a whole number of hops AND covers the delay tail.
    let padded = frames
        .next_multiple_of(hop)
        .max((frames + delay).next_multiple_of(hop));

    let mut noisy = Array2::<f32>::zeros((n_ch, padded));
    noisy
        .slice_axis_mut(Axis(1), Slice::from(0..frames))
        .assign(planar);
    let mut enh = Array2::<f32>::zeros((n_ch, padded));

    // `padded` is a whole number of hops by construction, so this is the exact count the loop
    // will run — the denominator is known before the first one, which is what makes the bar a
    // measurement and not a guess. (`max(1)` only guards the division; `padded >= hop` always.)
    let total_hops = (padded / hop).max(1);
    for (i, (ns, en)) in noisy
        .axis_chunks_iter(Axis(1), hop)
        .zip(enh.axis_chunks_iter_mut(Axis(1), hop))
        .enumerate()
    {
        if ns.len_of(Axis(1)) < hop {
            break;
        }
        model
            .process(ns, en)
            .expect("DeepFilterNet3 inference failed");
        // After the hop, not before: the fraction says what is *done*, and a bar that shows
        // work as finished the instant it is started arrives at 100 % with a hop still to run.
        on_progress((i + 1) as f32 / total_hops as f32);
    }

    // Compensate the delay: the enhanced sample for input frame `f` sits at `f + delay`.
    enh.slice_axis(Axis(1), Slice::from(delay..delay + frames))
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::ChannelLayout;

    fn mono48(s: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(s, AudioFormat::new(MODEL_SR, ChannelLayout::Mono))
    }

    /// **Zero is bypass, byte for byte** — the rack's non-negotiable invariant. An "off" that
    /// quietly runs the clip through a neural net is an "off" you cannot A/B.
    #[test]
    fn amount_zero_is_byte_identical() {
        let data = mono48((0..8000).map(|i| (i as f32 * 0.01).sin() * 0.3).collect());
        let out = denoise_ml(&data, 0.0);
        assert_eq!(data.samples(), out.samples());
    }

    /// An empty clip is returned untouched — no model is spun up for nothing.
    #[test]
    fn empty_is_byte_identical() {
        let data = mono48(Vec::new());
        let out = denoise_ml(&data, 1.0);
        assert!(out.is_empty());
        assert_eq!(data.samples(), out.samples());
    }
}
