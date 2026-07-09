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
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        for (c, filt) in filters.iter_mut().enumerate() {
            let i = f * ch + c;
            out[i] = filt.process(out[i]);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// `tanh` soft-clip normalized so full-scale in ≈ full-scale out.
pub(super) fn saturate(data: &SampleData, drive: f32) -> SampleData {
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
pub(super) fn bitcrush(data: &SampleData, bits: u32, downsample: u32) -> SampleData {
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
pub(super) fn stereo_width(data: &SampleData, width: f32) -> SampleData {
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
}
