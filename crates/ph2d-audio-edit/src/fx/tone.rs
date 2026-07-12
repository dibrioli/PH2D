//! Tone + character processors for the rack: the biquad driver (low/high-pass,
//! peaking bell, shelves) and the memoryless character effects.
//!
//! Control thread only — allocates and uses `tanh` freely.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs};

use crate::ops::channels;

/// Run an independent [`Biquad`] per channel over the interleaved buffer.
pub(super) fn biquad_all(data: &SampleData, coeffs: BiquadCoeffs) -> SampleData {
    let ch = channels(data);
    let mut filters: Vec<Biquad> = (0..ch).map(|_| Biquad::new(coeffs)).collect();
    let frames = data.frame_count();
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            for (c, filt) in filters.iter_mut().enumerate() {
                let i = f * ch + c;
                out[i] = filt.process(out[i]);
            }
        }
    })
}

/// Full-depth mains-hum attenuation, in dB — a deep, narrow peaking cut is a de-facto
/// notch (a true zero would need a `notch()` in the RT core; a −60 dB cut is inaudible
/// residue and reuses the existing `peak`).
const HUM_MAX_CUT_DB: f32 = 60.0;
/// Q of each hum cut — narrow enough that only the ~`freq/Q` Hz around the mains line
/// is touched, leaving the voice around it intact.
pub(super) const HUM_Q: f32 = 30.0;
/// Cap on how many harmonics De-Hum chases (fundamental counts as the first).
const HUM_MAX_HARMONICS: u32 = 8;

/// **De-Hum**: a per-channel cascade of deep narrow peaking cuts at the mains
/// fundamental `freq` (50 EU / 60 US) and its harmonics, killing electrical buzz +
/// overtones. `depth` (0..1) scales the cut from 0 dB (identity — the neutral point) to
/// `−HUM_MAX_CUT_DB`. Length-preserving. Ported from the RBJ peaking filter (no invented
/// constant); presentation math, HR-5 exempt (control thread).
pub(super) fn dehum(data: &SampleData, freq: f32, depth: f32, harmonics: u32) -> SampleData {
    let ch = channels(data);
    let sr = data.format().sample_rate as f32;
    let nyquist = sr * 0.5;
    let cut_db = -depth.clamp(0.0, 1.0) * HUM_MAX_CUT_DB;
    // One deep narrow cut per harmonic (incl. the fundamental) that sits below Nyquist.
    let bands: Vec<BiquadCoeffs> = (1..=harmonics.clamp(1, HUM_MAX_HARMONICS))
        .map(|h| freq.max(1.0) * h as f32)
        .filter(|&f| f < nyquist)
        .map(|f| BiquadCoeffs::peak(sr, f, HUM_Q, cut_db))
        .collect();
    let mut chains: Vec<Vec<Biquad>> = (0..ch)
        .map(|_| bands.iter().map(|&c| Biquad::new(c)).collect())
        .collect();
    let frames = data.frame_count();
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            for (c, chain) in chains.iter_mut().enumerate() {
                let i = f * ch + c;
                let mut s = out[i];
                for filt in chain.iter_mut() {
                    s = filt.process(s);
                }
                out[i] = s;
            }
        }
    })
}

/// `tanh` soft-clip normalized so full-scale in ≈ full-scale out.
pub(super) fn saturate(data: &SampleData, drive: f32) -> SampleData {
    let k = drive.max(0.1);
    let norm = 1.0 / k.tanh();
    SampleData::map_in_place(data, |out| {
        for s in out.iter_mut() {
            let x = *s;
            *s = (k * x).tanh() * norm;
        }
    })
}

/// Quantize to `bits` levels + hold each channel's value for `downsample` frames.
pub(super) fn bitcrush(data: &SampleData, bits: u32, downsample: u32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let hold = downsample.max(1) as usize;
    // Levels for a signed range [-1, 1]: 2^bits steps.
    let levels = (1u32 << bits.clamp(1, 24)) as f32;
    let quant = |x: f32| ((x * 0.5 + 0.5) * levels).round() / levels * 2.0 - 1.0;
    let src = data.samples();
    let mut held = vec![0.0f32; ch];
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            for c in 0..ch {
                if f % hold == 0 {
                    held[c] = quant(src[f * ch + c]);
                }
                out[f * ch + c] = held[c].clamp(-1.0, 1.0);
            }
        }
    })
}

/// Mid/Side width on stereo clips; mono is returned unchanged.
pub(super) fn stereo_width(data: &SampleData, width: f32) -> SampleData {
    let ch = channels(data);
    if ch < 2 {
        return data.clone();
    }
    let frames = data.frame_count();
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            let base = f * ch;
            let (l, r) = (out[base], out[base + 1]);
            let mid = (l + r) * 0.5;
            let side = (l - r) * 0.5 * width;
            out[base] = (mid + side).clamp(-1.0, 1.0);
            out[base + 1] = (mid - side).clamp(-1.0, 1.0);
        }
    })
}

/// Distortion's darkest / brightest post-filter cutoff (Hz), swept by `tone`.
/// The min is `pub(super)` so the warmup pre-roll can size on the longest-ringing case.
pub(super) const DISTORT_TONE_MIN_HZ: f32 = 800.0;
const DISTORT_TONE_MAX_HZ: f32 = 16_000.0;
/// Pre-gain at full `drive` (drive 1 → ×31 into the clipper): heavy, square-ish grit.
const DISTORT_MAX_PRE: f32 = 30.0;

/// **Distortion**: a cubic hard-clipper — grittier than [`saturate`]'s `tanh` — with a
/// post low-pass **Tone** control. `drive` (0..1) blends dry→clipped *and* drives the
/// signal harder into the clip, so the onset off 0 is smooth (no level jump) while full
/// drive is a loud, clipped square. `tone` sweeps a low-pass that tames the fizz. This
/// is called only for `drive > 0`; at 0 the rack bypasses it.
pub(super) fn distortion(data: &SampleData, drive: f32, tone: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let drive = drive.clamp(0.0, 1.0);
    let pre = 1.0 + drive * DISTORT_MAX_PRE;
    // Cubic soft-clip scaled by 1.5 so its ±2/3 plateau reaches full scale — a loud,
    // saturated output rather than an attenuated one.
    let shaped = SampleData::map_in_place(data, |out| {
        for s in out.iter_mut() {
            let x = *s;
            let d = pre * x;
            let f = if d >= 1.0 {
                2.0 / 3.0
            } else if d <= -1.0 {
                -2.0 / 3.0
            } else {
                d - d * d * d / 3.0
            };
            // Blend by drive: at drive→0 this is the dry signal (a smooth onset).
            *s = (1.0 - drive) * x + drive * (f * 1.5);
        }
    });
    let cutoff = DISTORT_TONE_MIN_HZ * (DISTORT_TONE_MAX_HZ / DISTORT_TONE_MIN_HZ).powf(tone);
    // The low-pass rings past ±1 on the clipped square (Gibbs overshoot); clamp back —
    // a distortion hitting the ceiling is expected, not a bug.
    let lp = biquad_all(&shaped, BiquadCoeffs::lowpass(sr, cutoff, 0.707));
    SampleData::map_in_place(&lp, |out| {
        for v in out.iter_mut() {
            *v = v.clamp(-1.0, 1.0);
        }
    })
}

/// **Haas widener**: delays the right channel by `delay_ms` relative to the left. Under
/// ~30 ms the ear fuses the two into one *wider* source (the precedence effect) rather
/// than hearing an echo. A mono clip has nothing to widen and passes through; `delay_ms`
/// 0 leaves the right channel unshifted — the neutral point. The right channel's last
/// few ms are pushed out (length is preserved), which is inaudible at these delays.
pub(super) fn haas(data: &SampleData, delay_ms: f32) -> SampleData {
    let ch = channels(data);
    let sr = data.format().sample_rate as f32;
    let d = (delay_ms.max(0.0) * 0.001 * sr).round() as usize;
    if ch < 2 || d == 0 {
        return data.clone();
    }
    let frames = data.frame_count();
    let src = data.samples();
    // `out` starts as a copy of `src` (left channel already correct) and is a *distinct*
    // buffer, so the reads below still see the pristine input — the delay line must never
    // feed on its own output.
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            // Right channel reads `d` frames back (silence before the start).
            out[f * ch + 1] = if f >= d { src[(f - d) * ch + 1] } else { 0.0 };
        }
    })
}

/// Makeup on the exciter's generated harmonics — they come out well below the band
/// they were derived from, so a little gain buys audible sparkle before the clamp.
const EXCITER_DRIVE: f32 = 2.5;

/// **Exciter / enhancer**: adds high-frequency sparkle by generating harmonics of the
/// signal's own high band and mixing them back in. The band above `freq` is isolated
/// with a high-pass, shaped by `x·|x|` (a soft squaring that raises overtones above it),
/// and added back scaled by `amount`. Because the harmonics sit *above* the crossover,
/// the effect reads as air/presence, not distortion. Neutral at `amount` 0. Control
/// thread only.
pub(super) fn exciter(data: &SampleData, freq: f32, amount: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    // The high band whose overtones we will generate.
    let hb = biquad_all(data, BiquadCoeffs::highpass(sr, freq, 0.707));
    SampleData::map_in_place(data, |out| {
        for (s, &h) in out.iter_mut().zip(hb.samples()) {
            let x = *s;
            // `h·|h|`: sign-preserving, so it adds no DC for a symmetric band, and its
            // energy lands in overtones above the crossover.
            let harm = h * h.abs();
            *s = (x + amount * EXCITER_DRIVE * harm).clamp(-1.0, 1.0);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn stereo(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    #[test]
    fn saturate_is_monotonic_and_bounded() {
        let d = stereo(vec![0.0, 0.1, 0.5, 0.9, -0.5, -0.9]);
        let s = saturate(&d, 4.0);
        // Never exceeds unity, and a bigger input stays bigger (monotone).
        assert!(s.samples().iter().all(|x| x.abs() <= 1.0 + 1e-4));
        assert!(s.samples()[2] < s.samples()[3], "0.5 -> < 0.9 mapping");
        // Sign preserved.
        assert!(s.samples()[3] > 0.0 && s.samples()[5] < 0.0);
    }

    #[test]
    fn stereo_width_zero_is_mono_sum() {
        // width 0 collapses to mid on both channels.
        let d = stereo(vec![1.0, 0.0, 0.0, 1.0]);
        let m = stereo_width(&d, 0.0);
        assert_eq!(m.samples(), &[0.5, 0.5, 0.5, 0.5]);
        // width 1 is a passthrough.
        let p = stereo_width(&d, 1.0);
        assert_eq!(p.samples(), d.samples());
    }

    #[test]
    fn bitcrush_quantizes_and_holds() {
        // 1-bit crush snaps to the rail; downsample holds frame 0 across frame 1.
        let d = SampleData::from_interleaved(vec![0.7, 0.1], AudioFormat::mono(48_000));
        let c = bitcrush(&d, 1, 2);
        assert_eq!(c.samples()[0], c.samples()[1], "held across the decimation");
    }

    #[test]
    fn lowpass_attenuates_nyquist_more_than_dc() {
        // A DC-ish ramp vs. an alternating (near-Nyquist) signal: the low-pass must
        // keep DC energy and kill the alternation.
        let alt: Vec<f32> = (0..64)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let d = SampleData::from_interleaved(alt, AudioFormat::mono(48_000));
        let lp = biquad_all(&d, BiquadCoeffs::lowpass(48_000.0, 1_000.0, 0.707));
        let energy: f32 = lp.samples().iter().skip(16).map(|x| x * x).sum();
        assert!(energy < 8.0, "near-Nyquist content strongly attenuated");
    }

    #[test]
    fn dehum_kills_the_mains_line_and_spares_the_voice() {
        let sr = 48_000.0;
        let n = 9600; // 0.2 s
        let tone = |hz: f32| -> Vec<f32> {
            (0..n)
                .map(|i| (i as f32 * std::f32::consts::TAU * hz / sr).sin())
                .collect()
        };
        let (hum, voice) = (tone(60.0), tone(1000.0));
        let mixed: Vec<f32> = hum
            .iter()
            .zip(&voice)
            .map(|(h, v)| h * 0.5 + v * 0.5)
            .collect();
        let d = SampleData::from_interleaved(mixed, AudioFormat::mono(48_000));
        let cleaned = dehum(&d, 60.0, 1.0, 1);
        // Correlate the cleaned signal against each pure component (skip the filter
        // warm-up). The 60 Hz line must be crushed; the 1 kHz "voice" must survive.
        let corr = |b: &[f32]| -> f32 {
            cleaned
                .samples()
                .iter()
                .skip(2000)
                .zip(b.iter().skip(2000))
                .map(|(x, y)| x * y)
                .sum::<f32>()
                .abs()
        };
        let (hum_left, voice_left) = (corr(&hum), corr(&voice));
        assert!(
            hum_left < voice_left * 0.1,
            "hum not suppressed: {hum_left} vs voice {voice_left}"
        );
    }

    #[test]
    fn dehum_at_zero_depth_is_identity() {
        // depth 0 → 0 dB peaking cuts, which are algebraically a pass-through — the
        // rack's neutral point (the `apply` guard also short-circuits to a clone).
        let d = stereo(vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6]);
        let out = dehum(&d, 50.0, 0.0, 4);
        for (a, b) in d.samples().iter().zip(out.samples()) {
            assert!((a - b).abs() < 1e-6, "0-depth de-hum altered the signal");
        }
    }

    /// A peaking bell boosts its own band and leaves DC and Nyquist alone — that is
    /// what makes it a *bell* rather than a shelf. Checked on the transfer function
    /// at `z = 1` (DC) and `z = -1` (Nyquist).
    #[test]
    fn a_peaking_bell_leaves_dc_and_nyquist_alone() {
        let c = BiquadCoeffs::peak(48_000.0, 1_000.0, 1.0, 12.0);
        let dc = (c.b0 + c.b1 + c.b2) / (1.0 + c.a1 + c.a2);
        let nyq = (c.b0 - c.b1 + c.b2) / (1.0 - c.a1 + c.a2);
        assert!((dc - 1.0).abs() < 0.02, "bell moved DC: {dc}");
        assert!((nyq - 1.0).abs() < 0.02, "bell moved Nyquist: {nyq}");
    }

    fn sine_300() -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..4_800)
            .map(|n| 0.6 * (tau * 300.0 * n as f32 / 48_000.0).sin())
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(48_000))
    }

    /// Crest factor (peak / RMS) — a clean sine sits near √2 ≈ 1.414; clipping
    /// flattens the wave toward a square, dropping it toward 1.
    fn crest(s: &[f32]) -> f32 {
        let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        let rms = (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt();
        peak / rms.max(1e-9)
    }

    /// Distortion flattens the wave (clipping drops the crest factor) and never
    /// exceeds full scale.
    #[test]
    fn distortion_clips_and_stays_bounded() {
        let d = sine_300();
        let out = distortion(&d, 1.0, 1.0);
        assert!(
            out.samples().iter().all(|v| v.abs() <= 1.0 + 1e-4),
            "distortion exceeded full scale"
        );
        // Skip the low-pass's settling at the head.
        let cd = crest(&d.samples()[500..]);
        let co = crest(&out.samples()[500..]);
        assert!(co < cd, "distortion did not flatten the wave: {co} vs {cd}");
    }

    /// `drive` is a real knob: more drive clips harder, so the crest keeps dropping.
    #[test]
    fn more_drive_flattens_more() {
        let d = sine_300();
        let light = crest(&distortion(&d, 0.3, 1.0).samples()[500..]);
        let heavy = crest(&distortion(&d, 1.0, 1.0).samples()[500..]);
        assert!(
            heavy < light,
            "heavier drive must flatten more: {heavy} vs {light}"
        );
    }

    /// THE exciter contract: it manufactures overtones *above* the signal's band. A
    /// clean 3 kHz tone gains real energy above 6 kHz that was not there before.
    #[test]
    fn exciter_adds_energy_above_the_band() {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..4_800)
            .map(|n| 0.5 * (tau * 3_000.0 * n as f32 / 48_000.0).sin())
            .collect();
        let d = SampleData::from_interleaved(x, AudioFormat::mono(48_000));
        let out = exciter(&d, 2_000.0, 1.0);
        let above_6k = |s: &SampleData| {
            biquad_all(s, BiquadCoeffs::highpass(48_000.0, 6_000.0, 0.707)).samples()[500..]
                .iter()
                .map(|v| v * v)
                .sum::<f32>()
        };
        let hi_in = above_6k(&d);
        let hi_out = above_6k(&out);
        assert!(
            hi_out > hi_in * 2.0,
            "exciter added no high harmonics: {hi_out} vs {hi_in}"
        );
        assert!(
            out.samples().iter().all(|v| v.abs() <= 1.0 + 1e-4),
            "exciter clipped"
        );
    }

    /// The Haas widener delays only the right channel; the left is untouched.
    #[test]
    fn haas_delays_only_the_right_channel() {
        // 1 kHz SR → 10 ms = 10 samples. L ramps up, R ramps down (distinct channels).
        let n = 64;
        let mut v = vec![0.0f32; n * 2];
        for i in 0..n {
            v[i * 2] = i as f32 * 0.01;
            v[i * 2 + 1] = 1.0 - i as f32 * 0.01;
        }
        let d = SampleData::from_interleaved(v, AudioFormat::stereo(1_000));
        let out = haas(&d, 10.0); // 10 samples
        let (s, src) = (out.samples(), d.samples());
        for f in 0..n {
            assert!((s[f * 2] - src[f * 2]).abs() < 1e-6, "left moved at {f}");
        }
        assert!(
            s[1].abs() < 1e-6 && s[19].abs() < 1e-6,
            "right not silent pre-delay"
        );
        for f in 10..n {
            assert!(
                (s[f * 2 + 1] - src[(f - 10) * 2 + 1]).abs() < 1e-6,
                "right not delayed by 10 at {f}"
            );
        }
    }

    /// A mono clip cannot be widened, and delay 0 is a no-op — both pass through.
    #[test]
    fn haas_is_identity_on_mono_and_at_zero() {
        let m = SampleData::from_interleaved(vec![0.1, -0.2, 0.3], AudioFormat::mono(48_000));
        assert_eq!(haas(&m, 12.0).samples(), m.samples());
        let s = stereo(vec![0.1, -0.2, 0.3, -0.4]);
        assert_eq!(haas(&s, 0.0).samples(), s.samples());
    }
}
