//! **True-peak** measurement: the peak of the *reconstructed* waveform, not of the
//! samples.
//!
//! Sample values undersell the signal. A sine at `fs/4` sampled 45° off-phase lands
//! on `±0.707·A` at every sample while the continuous waveform still swings to `±A`
//! — a **3 dB inter-sample peak** invisible to a `max(|x[n]|)` meter. That headroom
//! lie is what makes a "0 dBFS" master clip a consumer DAC, or a lossy encoder
//! overshoot on decode.
//!
//! ITU-R BS.1770 measures true peak by oversampling ×4 before taking the max. This
//! module does the same with a **windowed-sinc fractional-delay interpolator**
//! generated here, not the literal coefficient table from the recommendation's
//! attachment — so the shape is ours and the behaviour is the standard's. The extra
//! headroom the standard's own filter leaves (it is not brickwall either) is
//! absorbed by working ~0.3 dB conservatively; callers wanting a mastering ceiling
//! should ask for −1 dBTP like everyone else.
//!
//! Control thread only (HR-3/HR-5 do not apply — this allocates and uses `sin`).

use ph2d_audio::SampleData;

use crate::ops::channels;

/// Upsampling factor. BS.1770 specifies ×4 for rates up to 96 kHz.
pub const OVERSAMPLE: usize = 4;

/// Taps per polyphase branch. Each branch reaches `TAPS/2` input frames either way,
/// so the interpolator is exact for anything comfortably below Nyquist and the cost
/// stays linear in the clip length.
const TAPS: usize = 12;
/// Index of the tap sitting on the output frame itself (`u = 0` for phase 0).
const CENTER: isize = (TAPS / 2) as isize - 1;

/// One polyphase branch per fractional offset `p/OVERSAMPLE`.
type Kernel = [[f32; TAPS]; OVERSAMPLE];

/// `sin(πx)/(πx)`, continuous at 0.
fn sinc(x: f32) -> f32 {
    if x.abs() < 1e-6 {
        1.0
    } else {
        let pix = std::f32::consts::PI * x;
        pix.sin() / pix
    }
}

/// Blackman window over `|u| <= half`, zero outside.
fn blackman(u: f32, half: f32) -> f32 {
    if u.abs() > half {
        return 0.0;
    }
    let t = std::f32::consts::PI * u / half;
    0.42 + 0.5 * t.cos() + 0.08 * (2.0 * t).cos()
}

/// Build the interpolator.
///
/// Phase 0 comes out an exact unit impulse (`sinc` vanishes at every non-zero
/// integer), so the original samples pass through untouched and `true_peak` can
/// never read *below* the sample peak. Each branch is normalized to unity DC gain,
/// so a constant signal interpolates to itself rather than to a rippling one.
fn kernel() -> Kernel {
    let half = (TAPS / 2) as f32;
    let mut k = [[0.0f32; TAPS]; OVERSAMPLE];
    for (p, phase) in k.iter_mut().enumerate() {
        let frac = p as f32 / OVERSAMPLE as f32;
        for (i, tap) in phase.iter_mut().enumerate() {
            let u = (i as isize - CENTER) as f32 - frac;
            *tap = sinc(u) * blackman(u, half);
        }
        let sum: f32 = phase.iter().sum();
        if sum.abs() > f32::EPSILON {
            for tap in phase.iter_mut() {
                *tap /= sum;
            }
        }
    }
    k
}

/// Read frame `f`, channel `c`, replicating the edge samples past the clip. The
/// alternative — zero-padding — invents a discontinuity at the clip edge and reports
/// an inter-sample peak that is an artefact of the padding, not of the audio.
fn at(src: &[f32], frames: usize, ch: usize, f: isize, c: usize) -> f32 {
    let f = f.clamp(0, frames as isize - 1) as usize;
    src[f * ch + c]
}

/// Per-frame true peak: for each frame, the largest reconstructed magnitude across
/// every channel and every one of the `OVERSAMPLE` sub-sample positions starting at
/// that frame. Length = `data.frame_count()`.
pub(crate) fn true_peak_envelope(data: &SampleData) -> Vec<f32> {
    let frames = data.frame_count();
    if frames == 0 {
        return Vec::new();
    }
    let ch = channels(data);
    let src = data.samples();
    let k = kernel();

    let mut env = Vec::with_capacity(frames);
    for f in 0..frames {
        let mut peak = 0.0f32;
        for c in 0..ch {
            // Phase 0 is a unit impulse — read the sample instead of convolving it.
            peak = peak.max(src[f * ch + c].abs());
            for phase in k.iter().skip(1) {
                let mut acc = 0.0f32;
                for (i, tap) in phase.iter().enumerate() {
                    let idx = f as isize + i as isize - CENTER;
                    acc += tap * at(src, frames, ch, idx, c);
                }
                peak = peak.max(acc.abs());
            }
        }
        env.push(peak);
    }
    env
}

/// The clip's true peak (linear, `1.0` = 0 dBFS). Always ≥ the sample peak.
pub fn true_peak(data: &SampleData) -> f32 {
    true_peak_envelope(data)
        .iter()
        .fold(0.0f32, |m, &p| m.max(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn mono(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::mono(48_000))
    }

    /// Phase 0 must be an exact impulse: every original sample survives, so the true
    /// peak can never come out *below* the sample peak. If it could, a limiter built
    /// on this would happily let a clipped sample through.
    #[test]
    fn phase_zero_is_an_exact_impulse() {
        let k = kernel();
        for (i, tap) in k[0].iter().enumerate() {
            let expected = if i as isize == CENTER { 1.0 } else { 0.0 };
            assert!(
                (tap - expected).abs() < 1e-6,
                "phase 0 tap {i} = {tap}, expected {expected}"
            );
        }
    }

    /// Every branch must have unity DC gain, or a constant signal would interpolate
    /// into a ripple and report a phantom inter-sample peak.
    #[test]
    fn every_phase_has_unity_dc_gain() {
        for (p, phase) in kernel().iter().enumerate() {
            let sum: f32 = phase.iter().sum();
            assert!((sum - 1.0).abs() < 1e-5, "phase {p} sums to {sum}");
        }
    }

    /// A DC signal has no inter-sample peak: true peak == sample peak.
    #[test]
    fn a_constant_signal_has_no_intersample_peak() {
        let d = mono(vec![0.5; 200]);
        let tp = true_peak(&d);
        assert!((tp - 0.5).abs() < 1e-3, "DC true peak was {tp}");
    }

    /// THE case this module exists for. A full-scale sine at `fs/4`, sampled 45° off
    /// phase, lands on `±0.7071` at every single sample — yet the waveform it encodes
    /// swings to `±1.0`. A sample-peak meter reads −3 dBFS and calls it safe.
    #[test]
    fn a_45_degree_quarter_rate_sine_hides_a_3db_intersample_peak() {
        let amp = 0.999f32;
        let quarter = std::f32::consts::FRAC_PI_2; // fs/4 → π/2 rad per sample
        let phase = std::f32::consts::FRAC_PI_4; // 45°
        let x: Vec<f32> = (0..512)
            .map(|n| amp * (quarter * n as f32 + phase).cos())
            .collect();
        let d = mono(x);

        let sample_peak = crate::peak(&d);
        assert!(
            (sample_peak - amp * std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3,
            "the samples must sit on ±A/√2, got {sample_peak}"
        );

        let tp = true_peak(&d);
        assert!(
            tp > sample_peak * 1.3,
            "true peak {tp} did not see past the samples (peak {sample_peak})"
        );
        assert!(
            (tp - amp).abs() < 0.03,
            "true peak {tp} should recover the sine's amplitude {amp}"
        );
    }

    /// The envelope is per-frame and never under-reports the sample it sits on.
    #[test]
    fn the_envelope_is_per_frame_and_bounds_its_own_sample() {
        let x: Vec<f32> = (0..64).map(|n| if n == 30 { 0.9 } else { 0.0 }).collect();
        let d = mono(x.clone());
        let env = true_peak_envelope(&d);
        assert_eq!(env.len(), 64);
        for (n, &s) in x.iter().enumerate() {
            assert!(
                env[n] >= s.abs() - 1e-6,
                "frame {n} under-reports its sample"
            );
        }
        assert!(
            env[30] >= 0.9 - 1e-6,
            "the impulse frame must see the impulse"
        );
    }
}
