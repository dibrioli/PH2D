//! **De-esser**: a compressor that only ever touches the sibilance band.
//!
//! A broadband compressor keyed off an "ess" ducks the whole voice — the vowel that
//! carries the S goes with it, and the result pumps. This one drives a **dynamic
//! high shelf** instead: the sidechain listens to the sibilance band, and its gain
//! reduction becomes the shelf's gain.
//!
//! ## Why a shelf and not a band split
//!
//! The obvious construction is `high = highpass(x)`, `low = x − high`, and then
//! `out = low + g·high`. It reconstructs the input exactly at `g = 1`, and it is
//! **wrong**: `x − highpass(x)` is not a lowpass. A 2nd-order high-pass at 5 kHz
//! passes 9 kHz at |H| ≈ 0.96 *with a phase shift*, so the complement still carries
//! a rotated copy of the high band. Turning `g` down does not remove that band, it
//! combs it — measured, the high-band energy fell from 890 to 435 at `ratio` 2 and
//! then climbed back to 444 at `ratio` 12. Ducking harder made the "S" louder.
//!
//! A shelf attacks the magnitude response directly, so a deeper cut is always a
//! deeper cut, and the band below the corner is left alone by construction.
//!
//! Control thread only.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs};

use super::dynamics::{frame_peak, time_coeff};
use crate::ops::channels;

/// Slope of the shelf and of the sidechain's high-pass. `1/√2` is the maximally
/// flat corner — a resonant one would put a bump right where the sibilance lives.
/// `Effect::warmup_frames` sizes the pre-roll from it, so it lives here.
pub(super) const SPLIT_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;
/// Sibilance is a fast transient. A user-facing attack knob here just invites
/// settings that let the first millisecond of every "S" through.
const ATTACK_SECS: f32 = 0.001;
/// Deepest cut the shelf will make. Past this it stops being a de-esser and starts
/// being a missing top end.
const MAX_CUT_DB: f32 = -24.0;
/// Re-derive the shelf's coefficients only once the gain has moved this far. The
/// `sin_cos`/`powf` in `BiquadCoeffs::highshelf` is ~30× the cost of one filtered
/// sample, and a gain riding a 5 ms release does not move meaningfully per sample.
const RECALC_STEP_DB: f32 = 0.05;

/// Duck the band above `freq` whenever its own level exceeds `threshold`.
///
/// The sidechain is the high-passed signal's **stereo-linked** peak (an "S" panned
/// left would otherwise drag the image with it), smoothed with a fast attack and a
/// user-set release. The gain reduction it asks for becomes a high-shelf cut.
///
/// `ratio == 1` gives unity gain at every level — a 0 dB shelf, which is the neutral
/// point the rack's bypass relies on.
pub(super) fn deess(
    data: &SampleData,
    freq: f32,
    threshold: f32,
    ratio: f32,
    release_secs: f32,
) -> SampleData {
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let src = data.samples();

    // Sidechain: what the shelf's band actually contains. Its phase is irrelevant —
    // only |level| is read — so a plain biquad high-pass is the right tool here.
    let hp = BiquadCoeffs::highpass(sr, freq, SPLIT_Q);
    let mut detectors: Vec<Biquad> = (0..ch).map(|_| Biquad::new(hp)).collect();
    let mut band = vec![0.0f32; src.len()];
    for f in 0..frames {
        for (c, filt) in detectors.iter_mut().enumerate() {
            let i = f * ch + c;
            band[i] = filt.process(src[i]);
        }
    }

    let attack = time_coeff(ATTACK_SECS, sr);
    let release = time_coeff(release_secs, sr);
    let threshold = threshold.max(1e-6);
    let exponent = 1.0 - 1.0 / ratio.max(1.0);

    // The shelf, one instance per channel so their delay states stay independent.
    // Retuned in place (`set_coeffs` keeps the state), so a moving gain never clicks.
    let mut shelves: Vec<Biquad> = (0..ch)
        .map(|_| Biquad::new(BiquadCoeffs::identity()))
        .collect();
    let mut coeff_db = f32::INFINITY; // force the first recalc

    // Primed on the first frame, so a region that opens on an "S" is caught rather
    // than faded into.
    let mut env = frame_peak(&band, ch, 0);
    // One allocation (ADR-0117 D2): same length, and every sample is written at the index it
    // was read from. The sidechain reads the separate `band` buffer, never the output.
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            let level = frame_peak(&band, ch, f);
            let coeff = if level > env { attack } else { release };
            env += (level - env) * coeff;

            let shelf_db = if env > threshold {
                // The compressor law, in dB: cut = (1 − 1/ratio) × the overshoot.
                let cut = 20.0 * (threshold / env).log10() * exponent;
                cut.max(MAX_CUT_DB)
            } else {
                0.0
            };
            if (shelf_db - coeff_db).abs() > RECALC_STEP_DB {
                let c = BiquadCoeffs::highshelf(sr, freq, SPLIT_Q, shelf_db);
                for s in &mut shelves {
                    s.set_coeffs(c);
                }
                coeff_db = shelf_db;
            }
            for (c, s) in shelves.iter_mut().enumerate() {
                let i = f * ch + c;
                out[i] = s.process(src[i]);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::super::tone::biquad_all;
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: f32 = 48_000.0;
    /// Skip each measurement filter's own settling.
    const SETTLED: usize = 2_000;

    /// A steady low tone plus a loud high tone — a caricature of a vowel with an "S"
    /// riding on it.
    fn voice_with_sibilance(low_hz: f32, high_hz: f32, high_amp: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..9_600)
            .map(|n| {
                let t = n as f32 / SR;
                0.4 * (tau * low_hz * t).sin() + high_amp * (tau * high_hz * t).sin()
            })
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(48_000))
    }

    /// Energy of `d` above / below `split` Hz.
    fn band_energy(d: &SampleData, split: f32, high: bool) -> f32 {
        let c = if high {
            BiquadCoeffs::highpass(SR, split, SPLIT_Q)
        } else {
            BiquadCoeffs::lowpass(SR, split, SPLIT_Q)
        };
        biquad_all(d, c).samples()[SETTLED..]
            .iter()
            .map(|v| v * v)
            .sum()
    }

    /// THE contract: the sibilance goes down, the voice does not. A broadband
    /// compressor keyed off the same "S" would take the vowel with it.
    #[test]
    fn the_deesser_ducks_the_high_band_and_leaves_the_voice_alone() {
        let split = 5_000.0;
        let d = voice_with_sibilance(200.0, 9_000.0, 0.5);
        let out = deess(&d, split, 0.05, 6.0, 0.05);

        let hi_in = band_energy(&d, split, true);
        let hi_out = band_energy(&out, split, true);
        assert!(
            hi_out < hi_in * 0.5,
            "the sibilance survived: {hi_out} vs {hi_in}"
        );

        let lo_in = band_energy(&d, 1_000.0, false);
        let lo_out = band_energy(&out, 1_000.0, false);
        assert!(
            (lo_out - lo_in).abs() < lo_in * 0.05,
            "the voice was ducked with the S: {lo_out} vs {lo_in}"
        );
    }

    /// A clip with nothing in the sibilance band must come out (essentially)
    /// untouched — the de-esser only ever reacts to its own band.
    #[test]
    fn a_clip_without_sibilance_passes_through() {
        let d = voice_with_sibilance(200.0, 9_000.0, 0.0); // pure 200 Hz
        let out = deess(&d, 5_000.0, 0.05, 6.0, 0.05);
        let worst = d
            .samples()
            .iter()
            .zip(out.samples())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(worst < 1e-3, "a sibilance-free clip moved by {worst}");
    }

    /// Ducking harder must duck harder. This is the invariant the first design broke:
    /// with `low = x − highpass(x)`, the high-band energy fell to 435 at `ratio` 2 and
    /// climbed back to 444 by `ratio` 12 — the complement is not a lowpass, so a lower
    /// gain combed the band instead of removing it. A shelf cannot do that.
    #[test]
    fn a_heavier_ratio_always_ducks_harder() {
        let split = 5_000.0;
        let d = voice_with_sibilance(200.0, 9_000.0, 0.5);
        let hi = |x: &SampleData| band_energy(x, split, true);

        let mut last = hi(&d);
        for ratio in [2.0f32, 4.0, 8.0, 12.0] {
            let e = hi(&deess(&d, split, 0.05, ratio, 0.05));
            assert!(
                e < last,
                "ratio {ratio} ducked less than the ratio before it: {e} !< {last}"
            );
            last = e;
        }
    }

    /// `ratio == 1` is exactly unity gain — a 0 dB shelf — which is the neutral point
    /// the rack's bypass relies on.
    #[test]
    fn ratio_one_is_a_zero_db_shelf() {
        let d = voice_with_sibilance(200.0, 9_000.0, 0.5);
        let unity = deess(&d, 5_000.0, 0.05, 1.0, 0.05);
        let worst = d
            .samples()
            .iter()
            .zip(unity.samples())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(worst < 1e-4, "ratio 1 must be unity gain, moved by {worst}");
    }

    /// The gain must be stereo-linked: an "S" that lives on one channel has to duck
    /// both, or the image lurches on every sibilant. Compared as per-channel energy
    /// ratios — a sample-wise quotient divides by the sine's zero crossings.
    #[test]
    fn the_deesser_links_the_stereo_gain() {
        let tau = std::f32::consts::TAU;
        // Left carries the sibilance; right carries the same tone 20 dB down.
        let x: Vec<f32> = (0..9_600)
            .flat_map(|n| {
                let t = n as f32 / SR;
                let s = (tau * 9_000.0 * t).sin();
                [0.6 * s, 0.06 * s]
            })
            .collect();
        let d = SampleData::from_interleaved(x, AudioFormat::stereo(48_000));
        let out = deess(&d, 5_000.0, 0.05, 8.0, 0.05);

        let energy = |x: &SampleData, c: usize| -> f32 {
            x.samples()[SETTLED * 2..]
                .iter()
                .skip(c)
                .step_by(2)
                .map(|v| v * v)
                .sum()
        };
        let ratio_l = energy(&out, 0) / energy(&d, 0);
        let ratio_r = energy(&out, 1) / energy(&d, 1);
        assert!(
            (ratio_l - ratio_r).abs() < 0.02 * ratio_l,
            "channels were ducked by different amounts: {ratio_l} vs {ratio_r}"
        );
        assert!(
            ratio_l < 0.5,
            "the loud channel was not ducked at all: {ratio_l}"
        );
    }
}
