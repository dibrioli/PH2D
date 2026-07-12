//! **Comb resonator** — a tuned feedback comb that rings at a chosen pitch.
//!
//! `y[n] = x[n] + g·y[n − D]` with `D = sample_rate / freq`: a feedback comb whose
//! resonant peaks sit at `freq` and its harmonics, the "plucked tube" colour. The wet
//! signal is scaled by `1 − g` so the resonant peak lands near unity instead of the
//! `1/(1 − g)` a raw feedback comb would reach, then blended with the dry by `mix`.
//! Neutral at `mix` 0.
//!
//! Control thread only. Length-preserving (the ring builds from rest inside the region,
//! so there is no edge click and no tail to extend).

use ph2d_audio::SampleData;

use crate::ops::channels;

/// The tuned frequency is clamped here before it becomes a delay length.
const COMB_MIN_HZ: f32 = 40.0;
const COMB_MAX_HZ: f32 = 2_000.0;
/// Feedback ceiling: at 1 the comb self-oscillates forever.
const COMB_MAX_FEEDBACK: f32 = 0.95;

/// Resonate `data` at `freq` with `feedback` (0..1), blended dry→wet by `mix`.
pub(super) fn comb(data: &SampleData, freq: f32, feedback: f32, mix: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mix = mix.clamp(0.0, 1.0);
    let g = feedback.clamp(0.0, COMB_MAX_FEEDBACK);
    let d = ((sr / freq.clamp(COMB_MIN_HZ, COMB_MAX_HZ)).round() as usize).max(2);
    let norm = 1.0 - g; // hold the resonant peak near unity

    let mut bufs: Vec<Vec<f32>> = (0..ch).map(|_| vec![0.0f32; d]).collect();
    let mut idx = 0usize;
    let src = data.samples();
    // One allocation (ADR-0117 D2): same length, and each sample is written at the index it
    // was read from. The ring lives in `bufs`, not in the output, so nothing reads ahead.
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            for (c, buf) in bufs.iter_mut().enumerate() {
                let delayed = buf[idx]; // y[n − D]
                let x = src[f * ch + c];
                let y = x + g * delayed;
                buf[idx] = y;
                out[f * ch + c] = ((1.0 - mix) * x + mix * (y * norm)).clamp(-1.0, 1.0);
            }
            idx = (idx + 1) % d;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    /// Mix 0 is a byte-identical no-op — the neutral point.
    #[test]
    fn comb_at_zero_mix_is_identity() {
        let d =
            SampleData::from_interleaved(vec![0.1, -0.2, 0.3, -0.4], AudioFormat::stereo(48_000));
        assert_eq!(comb(&d, 200.0, 0.8, 0.0).samples(), d.samples());
    }

    /// THE resonator contract: an impulse rings at the tuned period, decaying by the
    /// feedback each cycle. 1 kHz makes freq 100 Hz exactly a 10-sample period.
    #[test]
    fn comb_rings_at_the_tuned_period() {
        let mut x = vec![0.0f32; 64];
        x[0] = 1.0;
        let d = SampleData::from_interleaved(x, AudioFormat::mono(1_000));
        let out = comb(&d, 100.0, 0.8, 1.0); // D = 10, g = 0.8
        let s = out.samples();
        assert!(s[10].abs() > 0.1, "no ring at the period: {}", s[10]);
        assert!(
            s[20].abs() > 0.05 && s[20].abs() < s[10].abs(),
            "ring not decaying: {} then {}",
            s[10],
            s[20]
        );
        assert!(s.iter().all(|v| v.abs() <= 1.0 + 1e-4), "comb clipped");
    }
}
