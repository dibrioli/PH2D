//! **Auto-wah** — a resonant band-pass whose centre frequency follows the signal's
//! own envelope, so louder passages open the filter (the vowel-like "wah" that tracks
//! playing dynamics).
//!
//! An envelope follower on the stereo-linked peak drives the cut-off up from `base`
//! by `sens` octaves; a resonant band-pass at that moving centre is blended with the
//! dry signal by `mix`. Neutral at `mix` 0. The band-pass is **retuned in place**
//! (state preserved) only when the cut-off has moved appreciably, so a smooth sweep
//! never clicks and the per-sample `sin_cos` cost stays bounded.
//!
//! Control thread only.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs};

use super::dynamics::{frame_peak, time_coeff};
use crate::ops::channels;

/// Envelope follower times: catch a transient fast, fall back gently.
const WAH_ATTACK_SECS: f32 = 0.005;
const WAH_RELEASE_SECS: f32 = 0.080;
/// Band-pass resonance — sharp enough to sing, not so sharp it whistles.
/// `pub(super)` so the warmup pre-roll can size the band-pass's settling.
pub(super) const WAH_Q: f32 = 3.0;
/// At `sens` 1 and a full envelope the cut-off climbs this many octaves above `base`.
const WAH_SWEEP_OCT: f32 = 4.0;
/// Absolute cut-off ceiling (also clamped below Nyquist).
const WAH_MAX_HZ: f32 = 8_000.0;
/// Retune the band-pass only once the cut-off has moved this fraction — the coeff
/// `sin_cos` costs far more than a filtered sample, and a sweep barely moves per frame.
const WAH_RECALC_RATIO: f32 = 0.02;

/// Sweep `data` through an envelope-driven band-pass rooted at `base_freq`, `sens`
/// (0..1) setting the sweep depth, blended dry→wet by `mix`.
pub(super) fn auto_wah(data: &SampleData, base_freq: f32, sens: f32, mix: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mix = mix.clamp(0.0, 1.0);
    let sens = sens.clamp(0.0, 1.0);
    let attack = time_coeff(WAH_ATTACK_SECS, sr);
    let release = time_coeff(WAH_RELEASE_SECS, sr);
    let max_hz = WAH_MAX_HZ.min(sr * 0.45);

    let src = data.samples();
    let mut filters: Vec<Biquad> = (0..ch)
        .map(|_| Biquad::new(BiquadCoeffs::bandpass(sr, base_freq, WAH_Q)))
        .collect();
    // Prime the envelope on the first frame so a region that opens loud is caught.
    let mut env = frame_peak(src, ch, 0);
    let mut last_cut = 0.0f32; // force the first retune

    // One allocation (ADR-0117 D2): same length, and every sample is written at the index it
    // was read from. The envelope reads the pristine `src`, never the output.
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            let a = frame_peak(src, ch, f);
            env += (a - env) * if a > env { attack } else { release };
            let cut = (base_freq * (sens * WAH_SWEEP_OCT * env).exp2()).clamp(base_freq, max_hz);
            if (cut - last_cut).abs() > last_cut * WAH_RECALC_RATIO {
                let c = BiquadCoeffs::bandpass(sr, cut, WAH_Q);
                for filt in &mut filters {
                    filt.set_coeffs(c);
                }
                last_cut = cut;
            }
            for (c, filt) in filters.iter_mut().enumerate() {
                let i = f * ch + c;
                let x = src[i];
                let wet = filt.process(x);
                out[i] = ((1.0 - mix) * x + mix * wet).clamp(-1.0, 1.0);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: f32 = 48_000.0;

    /// A steady mono tone at `hz`, amplitude `amp`.
    fn tone(hz: f32, amp: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..9_600)
            .map(|n| amp * (tau * hz * n as f32 / SR).sin())
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(48_000))
    }

    /// Mix 0 is a byte-identical no-op.
    #[test]
    fn auto_wah_at_zero_mix_is_identity() {
        let d = tone(1_000.0, 0.5);
        assert_eq!(auto_wah(&d, 500.0, 0.5, 0.0).samples(), d.samples());
    }

    /// THE contract: a louder input opens the filter higher, so a tone above the base
    /// passes with more gain when it is loud than when it is quiet. Measured as the
    /// band-pass's effective gain (out energy / in energy), which removes the raw
    /// amplitude difference and leaves only the filter's doing.
    #[test]
    fn louder_input_opens_the_filter() {
        let gain = |amp: f32| {
            let d = tone(3_000.0, amp);
            let out = auto_wah(&d, 500.0, 1.0, 1.0);
            let e_in: f32 = d.samples()[1_000..].iter().map(|v| v * v).sum();
            let e_out: f32 = out.samples()[1_000..].iter().map(|v| v * v).sum();
            e_out / e_in.max(1e-12)
        };
        let quiet = gain(0.15);
        let loud = gain(0.9);
        assert!(
            loud > quiet * 2.0,
            "the envelope did not open the filter: loud {loud} vs quiet {quiet}"
        );
    }
}
