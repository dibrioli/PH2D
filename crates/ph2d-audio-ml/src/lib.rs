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

    let enh = enhance_48k(&planar, n_ch);

    // Dry/wet blend, re-interleaved. `wet == 1.0` (amount 1.0) is the pure model output.
    let wet = amount;
    let dry = 1.0 - amount;
    let out48 = SampleData::from_fn(frames * n_ch, target48, |i| {
        let f = i / n_ch;
        let c = i % n_ch;
        wet * enh[[c, f]] + dry * src[f * n_ch + c]
    });

    // Back to the clip's own rate (a no-op, byte-identical, when it was already 48 kHz).
    ph2d_audio_edit::conform(&out48, fmt)
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
fn enhance_48k(planar: &Array2<f32>, n_ch: usize) -> Array2<f32> {
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

    for (ns, en) in noisy
        .axis_chunks_iter(Axis(1), hop)
        .zip(enh.axis_chunks_iter_mut(Axis(1), hop))
    {
        if ns.len_of(Axis(1)) < hop {
            break;
        }
        model
            .process(ns, en)
            .expect("DeepFilterNet3 inference failed");
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
