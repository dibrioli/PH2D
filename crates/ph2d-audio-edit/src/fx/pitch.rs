//! **Pitch shift** — a time-domain granular shifter (no FFT, no dependency).
//!
//! Resamples the signal by `2^(semitones/12)` while holding the clip length, with the
//! classic **two-tap delay line + Hann cross-fade**: one read tap sweeps the delay at
//! the pitch rate, and when it would wrap past a grain boundary — a click — a second
//! tap, half a grain out of phase, is already faded in to cover it. Two Hann windows
//! offset by half a window sum to exactly 1, so the cross-fade is gain-flat.
//!
//! Formants shift *with* the pitch (this is not a formant-preserving vocoder) — which
//! is exactly what a monster / chipmunk / robot voice wants; a phase vocoder that held
//! the formants would sound auto-tuned, not transformed. `mix` blends dry→wet, for a
//! detuned double or an octave-down "sub" sitting under the dry voice.
//!
//! Control thread only.

use std::f32::consts::TAU;

use ph2d_audio::SampleData;

use super::modulation::frac_read;
use crate::ops::channels;

/// Grain length (samples): the delay window the two taps sweep. ~43 ms at 48 kHz —
/// long enough that the wrap is gentle, short enough that the smear stays subtle.
pub(super) const GRAIN: usize = 2048;

/// A shift this small is inaudible; below it the effect bypasses (and a
/// resample-then-cross-fade would still round every sample away from identity).
pub(super) const PITCH_BYPASS_ST: f32 = 1e-3;

/// Shift `data` by `semitones` (±), blended dry→wet by `mix`. Same length out.
///
/// Called only off the neutral point (`semitones` past [`PITCH_BYPASS_ST`] *and*
/// `mix` above 0); the caller returns the input untouched otherwise.
pub(super) fn pitch_shift(data: &SampleData, semitones: f32, mix: f32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let mix = mix.clamp(0.0, 1.0);
    let ratio = (semitones / 12.0).exp2();
    // The read delay drifts by `(1 - ratio)` per output sample: shrinking when we read
    // faster than we write (pitch up), growing when slower (pitch down). Wrapped into
    // one grain, the drift is what re-times the signal.
    let rate = 1.0 - ratio;
    let glen = GRAIN as f32;
    let half = glen * 0.5;
    let len = GRAIN + 4; // headroom for the fractional read past the deepest tap

    let mut bufs: Vec<Vec<f32>> = (0..ch).map(|_| vec![0.0f32; len]).collect();
    let mut write = 0usize;
    let src = data.samples();
    let mut out = src.to_vec();
    let mut phase = 0.0f32; // tap-1 delay, kept in [0, GRAIN)
    for f in 0..frames {
        // Tap 2 trails half a grain behind tap 1 (wrapped).
        let phase2 = if phase >= half {
            phase - half
        } else {
            phase + half
        };
        // Hann pair: w1 is 0 exactly where tap 1 wraps (delay 0 / GRAIN), so the wrap
        // discontinuity is masked; w2 = 1 - w1 covers it. Their sum is 1 → flat gain.
        let w1 = 0.5 - 0.5 * (TAU * phase / glen).cos();
        let w2 = 1.0 - w1;
        for (c, buf) in bufs.iter_mut().enumerate() {
            let x = src[f * ch + c];
            buf[write] = x;
            // +1: never ask for the just-written sample at delay 0.
            let wet =
                w1 * frac_read(buf, write, phase + 1.0) + w2 * frac_read(buf, write, phase2 + 1.0);
            out[f * ch + c] = (1.0 - mix) * x + mix * wet;
        }
        write = (write + 1) % len;
        phase += rate;
        while phase >= glen {
            phase -= glen;
        }
        while phase < 0.0 {
            phase += glen;
        }
    }
    SampleData::from_interleaved(out, data.format())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;

    /// A pure mono tone at `hz`, `n` frames.
    fn tone(hz: f32, n: usize) -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..n)
            .map(|i| 0.6 * (tau * hz * i as f32 / SR as f32).sin())
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(SR))
    }

    /// Sign changes over `s[from..]` — a dependency-free proxy for the dominant
    /// frequency of a tone: it rises with pitch and falls with it.
    fn zero_crossings(s: &[f32], from: usize) -> usize {
        s[from..]
            .windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count()
    }

    #[test]
    fn pitch_shift_preserves_length() {
        let d = tone(300.0, 9_600);
        assert_eq!(pitch_shift(&d, 7.0, 1.0).frame_count(), 9_600);
    }

    /// THE contract: shifting up raises the pitch, shifting down lowers it. Measured
    /// by zero-crossing rate past the delay-line fill, so no FFT is needed. An octave
    /// up roughly doubles the rate; an octave down roughly halves it.
    #[test]
    fn shifting_up_raises_pitch_and_down_lowers_it() {
        let d = tone(300.0, 24_000);
        let skip = 2 * GRAIN; // past the delay fill transient
        let dry = zero_crossings(d.samples(), skip);
        let up = zero_crossings(pitch_shift(&d, 12.0, 1.0).samples(), skip);
        let down = zero_crossings(pitch_shift(&d, -12.0, 1.0).samples(), skip);
        assert!(
            up as f32 > dry as f32 * 1.6,
            "octave up should ~double the rate: dry {dry} up {up}"
        );
        assert!(
            (down as f32) < dry as f32 * 0.7,
            "octave down should ~halve the rate: dry {dry} down {down}"
        );
    }

    /// Never exceeds full scale — a convex blend of bounded taps.
    #[test]
    fn pitch_shift_stays_bounded() {
        let d = tone(300.0, 12_000);
        for st in [-12.0f32, -5.0, 3.0, 12.0] {
            let out = pitch_shift(&d, st, 1.0);
            assert!(
                out.samples().iter().all(|s| s.abs() <= 1.0 + 1e-4),
                "clipped at {st} st"
            );
        }
    }

    /// `mix` 0 returns the dry signal exactly — the blend, pinned. (The rack bypasses
    /// this case before it ever calls in, but the math must still be honest.)
    #[test]
    fn mix_zero_is_dry() {
        let d = tone(300.0, 4_800);
        assert_eq!(pitch_shift(&d, 7.0, 0.0).samples(), d.samples());
    }
}
