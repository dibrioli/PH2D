//! Offline, **length-preserving** effect processors for the editor (W3 §6 rack).
//!
//! Each [`Effect`] renders a fresh buffer from a clip — control-thread only, so
//! HR-3/HR-5 do not apply (they allocate and use `tanh`/`exp` freely). Because
//! every effect here keeps the frame count, they route through
//! [`crate::in_range`] just like the W2 ops, so applying to a selection processes
//! only the selected samples (filter/compressor state starts fresh at the region
//! edge). Tail-extending effects (reverb/delay) are a separate block — they can't
//! use the length-preserving splice.
//!
//! Filters and dynamics reuse the shared `ph2d_audio::dsp` kit (Biquad,
//! Compressor); character effects (saturate/bitcrush/width) are local pure math.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs, Compressor};

/// Channel count of the clip (≥1).
fn channels(data: &SampleData) -> usize {
    data.format().channel_count().max(1)
}

/// A single offline effect with fixed parameters. The UI picks a curated preset
/// per button (mirror of the W2 fixed-gain buttons); parametric control + chain +
/// presets are a later block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// 2nd-order low-pass (muffle / warm) at `cutoff` Hz, `q` resonance.
    LowPass { cutoff: f32, q: f32 },
    /// 2nd-order high-pass (thin / de-rumble) at `cutoff` Hz, `q` resonance.
    HighPass { cutoff: f32, q: f32 },
    /// Feed-forward compressor (glue / level) + linear `makeup` gain.
    Compress {
        /// Level (linear 0..1) above which reduction starts.
        threshold: f32,
        /// Ratio (≥1).
        ratio: f32,
        /// Attack time (seconds).
        attack_secs: f32,
        /// Release time (seconds).
        release_secs: f32,
        /// Post-compression make-up gain (linear).
        makeup: f32,
    },
    /// `tanh` soft-clip saturation (warmth / drive); `drive` ≥ ~0.1.
    Saturate { drive: f32 },
    /// Lo-fi bit-depth reduction to `bits` + sample-hold decimation by
    /// `downsample` (≥1 = no decimation).
    Bitcrush { bits: u32, downsample: u32 },
    /// Mid/Side stereo width (`1.0` = unchanged, `>1` wider, `0` mono). No-op on
    /// mono clips.
    StereoWidth { width: f32 },
}

impl Effect {
    /// Render `data` through this effect, returning a fresh, **same-length** clip.
    pub fn apply(&self, data: &SampleData) -> SampleData {
        let sr = data.format().sample_rate as f32;
        match *self {
            Effect::LowPass { cutoff, q } => biquad_all(data, BiquadCoeffs::lowpass(sr, cutoff, q)),
            Effect::HighPass { cutoff, q } => {
                biquad_all(data, BiquadCoeffs::highpass(sr, cutoff, q))
            }
            Effect::Compress {
                threshold,
                ratio,
                attack_secs,
                release_secs,
                makeup,
            } => compress(data, threshold, ratio, attack_secs, release_secs, makeup),
            Effect::Saturate { drive } => saturate(data, drive),
            Effect::Bitcrush { bits, downsample } => bitcrush(data, bits, downsample),
            Effect::StereoWidth { width } => stereo_width(data, width),
        }
    }
}

/// Run an independent [`Biquad`] per channel over the interleaved buffer.
fn biquad_all(data: &SampleData, coeffs: BiquadCoeffs) -> SampleData {
    let ch = channels(data);
    let mut filters: Vec<Biquad> = (0..ch).map(|_| Biquad::new(coeffs)).collect();
    let frames = data.frame_count();
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        for (c, filt) in filters.iter_mut().enumerate() {
            let i = f * ch + c;
            out[i] = filt.process(out[i]);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// One-shot / per-sample coefficient for a `secs` time-constant at `sr` Hz.
fn time_coeff(secs: f32, sr: f32) -> f32 {
    if secs <= 0.0 {
        return 1.0;
    }
    1.0 - (-1.0 / (secs * sr)).exp()
}

/// Stereo-linked compression (mono is duplicated then re-collapsed) + make-up.
fn compress(
    data: &SampleData,
    threshold: f32,
    ratio: f32,
    attack_secs: f32,
    release_secs: f32,
    makeup: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mut comp = Compressor::default();
    comp.set_params(
        true,
        threshold,
        ratio,
        time_coeff(attack_secs, sr),
        time_coeff(release_secs, sr),
    );
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        let base = f * ch;
        if ch >= 2 {
            let (l, r) = comp.process(out[base], out[base + 1]);
            out[base] = (l * makeup).clamp(-1.0, 1.0);
            out[base + 1] = (r * makeup).clamp(-1.0, 1.0);
        } else {
            let (l, _) = comp.process(out[base], out[base]);
            out[base] = (l * makeup).clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// `tanh` soft-clip normalized so full-scale in ≈ full-scale out.
fn saturate(data: &SampleData, drive: f32) -> SampleData {
    let k = drive.max(0.1);
    let norm = 1.0 / k.tanh();
    let out: Vec<f32> = data
        .samples()
        .iter()
        .map(|&x| (k * x).tanh() * norm)
        .collect();
    SampleData::from_interleaved(out, data.format())
}

/// Quantize to `bits` levels + hold each channel's value for `downsample` frames.
fn bitcrush(data: &SampleData, bits: u32, downsample: u32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let hold = downsample.max(1) as usize;
    // Levels for a signed range [-1, 1]: 2^bits steps.
    let levels = (1u32 << bits.clamp(1, 24)) as f32;
    let quant = |x: f32| ((x * 0.5 + 0.5) * levels).round() / levels * 2.0 - 1.0;
    let src = data.samples();
    let mut out = src.to_vec();
    let mut held = vec![0.0f32; ch];
    for f in 0..frames {
        for c in 0..ch {
            if f % hold == 0 {
                held[c] = quant(src[f * ch + c]);
            }
            out[f * ch + c] = held[c].clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Mid/Side width on stereo clips; mono is returned unchanged.
fn stereo_width(data: &SampleData, width: f32) -> SampleData {
    let ch = channels(data);
    if ch < 2 {
        return data.clone();
    }
    let frames = data.frame_count();
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        let base = f * ch;
        let (l, r) = (out[base], out[base + 1]);
        let mid = (l + r) * 0.5;
        let side = (l - r) * 0.5 * width;
        out[base] = (mid + side).clamp(-1.0, 1.0);
        out[base + 1] = (mid - side).clamp(-1.0, 1.0);
    }
    SampleData::from_interleaved(out, data.format())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn stereo(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    #[test]
    fn effects_preserve_length() {
        let d = stereo(vec![0.3, -0.4, 0.5, -0.6, 0.7, -0.8]); // 3 frames
        for fx in [
            Effect::LowPass {
                cutoff: 3_000.0,
                q: 0.707,
            },
            Effect::HighPass {
                cutoff: 150.0,
                q: 0.707,
            },
            Effect::Compress {
                threshold: 0.3,
                ratio: 4.0,
                attack_secs: 0.005,
                release_secs: 0.1,
                makeup: 1.5,
            },
            Effect::Saturate { drive: 3.0 },
            Effect::Bitcrush {
                bits: 6,
                downsample: 4,
            },
            Effect::StereoWidth { width: 1.6 },
        ] {
            assert_eq!(
                fx.apply(&d).frame_count(),
                3,
                "{fx:?} must keep the frame count"
            );
        }
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
}
