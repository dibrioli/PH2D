//! **De-plosive**: a compressor that only ever touches the low "pop" band.
//!
//! A plosive — the `p`/`b` burst of breath hitting the mic — is a fast spike of
//! low-frequency energy under ~150 Hz. Ducking the whole voice when one lands would
//! pump the vowel; this drives a **dynamic low shelf** instead, the exact mirror of the
//! de-esser: the sidechain listens to the low band, and its gain reduction becomes the
//! shelf's cut. A shelf (not a `x − highpass(x)` band split) so a deeper cut is always a
//! deeper cut — see [`super::deess`] for why the complement-filter construction combs
//! instead of removing.
//!
//! Control thread only.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs};

use super::deess::SPLIT_Q;
use super::dynamics::{frame_peak, time_coeff};
use crate::ops::channels;

/// A plosive is a transient — catch its onset fast, or the first thump escapes.
const ATTACK_SECS: f32 = 0.002;
/// Deepest cut the shelf will make; past this it stops de-popping and starts thinning
/// the voice's body.
const MAX_CUT_DB: f32 = -24.0;
/// Re-derive the shelf coefficients only once the gain has moved this far (the
/// `sin_cos`/`powf` in `lowshelf` costs ~30 filtered samples; a gain on a release does
/// not move meaningfully per sample). Same rationale as the de-esser.
const RECALC_STEP_DB: f32 = 0.05;

/// Duck the band below `freq` whenever its own level exceeds `threshold`.
///
/// The sidechain is the low-passed signal's **stereo-linked** peak, smoothed with a
/// fast attack and a user-set release; the gain reduction it asks for becomes a
/// low-shelf cut. `ratio == 1` gives unity gain at every level — a 0 dB shelf, the
/// neutral point the rack's bypass relies on.
pub(super) fn deplosive(
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

    // Sidechain: what the shelf's band actually contains. Only |level| is read, so a
    // plain biquad low-pass is the right detector.
    let lp = BiquadCoeffs::lowpass(sr, freq, SPLIT_Q);
    let mut detectors: Vec<Biquad> = (0..ch).map(|_| Biquad::new(lp)).collect();
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

    // The shelf, one per channel (independent delay state); retuned in place so a
    // moving gain never clicks.
    let mut shelves: Vec<Biquad> = (0..ch)
        .map(|_| Biquad::new(BiquadCoeffs::identity()))
        .collect();
    let mut coeff_db = f32::INFINITY; // force the first recalc

    // Primed on the first frame, so a region that opens on a pop is caught, not faded.
    let mut env = frame_peak(&band, ch, 0);
    // One allocation (ADR-0117 D2): same length, and every sample is written at the index it
    // was read from. The sidechain reads the separate `band` buffer, never the output.
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            let level = frame_peak(&band, ch, f);
            let coeff = if level > env { attack } else { release };
            env += (level - env) * coeff;

            let shelf_db = if env > threshold {
                let cut = 20.0 * (threshold / env).log10() * exponent;
                cut.max(MAX_CUT_DB)
            } else {
                0.0
            };
            if (shelf_db - coeff_db).abs() > RECALC_STEP_DB {
                let c = BiquadCoeffs::lowshelf(sr, freq, SPLIT_Q, shelf_db);
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

    /// A steady mid "voice" tone plus a loud low tone — a caricature of speech with a
    /// plosive pop under it.
    fn voice_with_pop(voice_hz: f32, pop_hz: f32, pop_amp: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..9_600)
            .map(|n| {
                let t = n as f32 / SR;
                0.4 * (tau * voice_hz * t).sin() + pop_amp * (tau * pop_hz * t).sin()
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

    /// THE contract: the pop's low band goes down, the voice above it does not.
    #[test]
    fn the_deplosive_ducks_the_low_band_and_leaves_the_voice_alone() {
        let split = 120.0;
        let d = voice_with_pop(800.0, 60.0, 0.6);
        let out = deplosive(&d, split, 0.05, 6.0, 0.08);

        let lo_in = band_energy(&d, split, false);
        let lo_out = band_energy(&out, split, false);
        assert!(
            lo_out < lo_in * 0.5,
            "the pop survived: {lo_out} vs {lo_in}"
        );

        let hi_in = band_energy(&d, 400.0, true);
        let hi_out = band_energy(&out, 400.0, true);
        assert!(
            (hi_out - hi_in).abs() < hi_in * 0.05,
            "the voice was ducked with the pop: {hi_out} vs {hi_in}"
        );
    }

    /// Ducking harder must duck harder (the invariant the complement-filter design
    /// broke; a shelf cannot comb the band).
    #[test]
    fn a_heavier_ratio_always_ducks_harder() {
        let split = 120.0;
        let d = voice_with_pop(800.0, 60.0, 0.6);
        let lo = |x: &SampleData| band_energy(x, split, false);
        let mut last = lo(&d);
        for ratio in [2.0f32, 4.0, 8.0, 12.0] {
            let e = lo(&deplosive(&d, split, 0.05, ratio, 0.08));
            assert!(
                e < last,
                "ratio {ratio} ducked less than before: {e} !< {last}"
            );
            last = e;
        }
    }

    /// `ratio == 1` is exactly unity gain — a 0 dB shelf — the rack's neutral point.
    #[test]
    fn ratio_one_is_a_zero_db_shelf() {
        let d = voice_with_pop(800.0, 60.0, 0.6);
        let unity = deplosive(&d, 120.0, 0.05, 1.0, 0.08);
        let worst = d
            .samples()
            .iter()
            .zip(unity.samples())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(worst < 1e-4, "ratio 1 must be unity gain, moved by {worst}");
    }
}
