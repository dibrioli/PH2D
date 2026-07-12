//! **Formant Shift** — move the vocal tract, not the pitch.
//!
//! The one voice transform a granular pitch shifter *cannot* do. In
//! [`super::pitch`] the formants ride along with the pitch, which is why a shifted
//! voice sounds like a chipmunk rather than like a child: a child's larynx is faster
//! **and** their head is smaller, but those are two independent facts, and a resampler
//! only knows how to change both at once. Separating them is the source-filter model
//! (Fant, *Acoustic Theory of Speech Production*, 1960):
//!
//! - the **filter** is the vocal tract — the spectral envelope, the formants, the size
//!   of the head. An LPC fit ([`super::lpc`]) recovers it per block.
//! - the **residual** left over is the excitation — the glottal pulse train, which is
//!   where the pitch lives.
//!
//! Warp the filter, re-drive it with the untouched residual, and the speaker changes
//! size at a constant pitch. Slide it down and the same performance comes out of a
//! giant; up, and out of something small — with the melody dead still, which is what
//! makes it sound like a *creature* and not like a tape speed change.
//!
//! **Warping without an FFT.** Scaling a filter's impulse response in time scales its
//! spectrum in frequency by the inverse (`h(t/c) ↔ |c|·H(cω)`) — so reading the
//! all-pole impulse response `α` times faster moves every formant up by `α`. That is
//! the whole trick: [`super::lpc::impulse_response`], resampled, convolved back over
//! the residual. No FFT, no dependency, and the result is an FIR — it cannot ring
//! away like a re-derived pole might.
//!
//! Blocks are analysed and re-synthesised through a Hann window at 50% overlap and
//! divided by the accumulated window energy (weighted overlap-add), so the seams —
//! **including the first and last block** — come out at unity gain. Without that
//! division the region's edges would fade in and out of a selection.
//!
//! Control thread only.

use std::f64::consts::TAU;

use ph2d_audio::SampleData;

use super::lpc::{autocorrelation, impulse_response, levinson, residual};
use crate::ops::channels;

/// AR order. A vocal tract shows ~1 formant per kHz, so 24 poles comfortably covers
/// the 4–5 formants that carry speech, plus the spectral tilt.
const ORDER: usize = 24;

/// Analysis block (~21 ms at 48 kHz) and its 50%-overlap hop. Short enough to track a
/// diphthong, long enough for a stable fit at [`ORDER`]. `BLOCK` doubles as the
/// effect's pre-roll, so the region's first window is fitted to real audio.
pub(super) const BLOCK: usize = 1_024;
const HOP: usize = BLOCK / 2;

/// Length of the envelope's impulse response before warping. The response of a stable
/// order-24 model has decayed well inside this.
const IR_LEN: usize = 192;

/// Ceiling on the warped response: shifting *down* stretches it, and the convolution
/// is `O(BLOCK × taps)`. At the slider's floor (−12 st) the stretch is 2×, so this is
/// slack, not a truncation the user can reach.
const IR_MAX: usize = 512;

/// A window that carries no energy contributes nothing to the seam — below this its
/// slot falls back to the dry sample rather than dividing by ~0.
const WSUM_EPS: f64 = 1e-9;

/// Shift the spectral envelope by `semitones` (±) while leaving the excitation — and
/// therefore the pitch — alone. `mix` blends dry into wet. Same length out.
///
/// Called only off the neutral point (`semitones` past the bypass *and* `mix` above
/// 0); the caller returns the input untouched otherwise.
pub(super) fn formant_shift(data: &SampleData, semitones: f32, mix: f32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    if frames == 0 || ch == 0 {
        return data.clone();
    }
    let mix = f64::from(mix.clamp(0.0, 1.0));
    let alpha = (f64::from(semitones) / 12.0).exp2();
    let hann: Vec<f64> = (0..BLOCK)
        .map(|i| 0.5 - 0.5 * (TAU * i as f64 / BLOCK as f64).cos())
        .collect();

    let src = data.samples();
    let mut out = src.to_vec();
    for c in 0..ch {
        let x: Vec<f64> = (0..frames).map(|f| f64::from(src[f * ch + c])).collect();
        let mut acc = vec![0.0f64; frames];
        let mut wsum = vec![0.0f64; frames];

        let mut start = 0;
        while start < frames {
            let len = BLOCK.min(frames - start);
            let w: Vec<f64> = (0..BLOCK)
                .map(|i| if i < len { x[start + i] * hann[i] } else { 0.0 })
                .collect();

            // A silent block has no envelope: it still owns its slice of the window
            // sum (so the seam stays unity) but contributes no audio.
            if let Some((a, _)) = levinson(&autocorrelation(&w, ORDER)) {
                let excitation = residual(&w, &a);
                let envelope = warp(&impulse_response(&a, IR_LEN), alpha);
                let mut y = convolve(&excitation, &envelope);
                match_energy(&mut y, &w);
                for i in 0..len {
                    acc[start + i] += y[i] * hann[i];
                }
            }
            for i in 0..len {
                wsum[start + i] += hann[i] * hann[i];
            }
            start += HOP;
        }

        for f in 0..frames {
            let wet = if wsum[f] > WSUM_EPS {
                acc[f] / wsum[f]
            } else {
                x[f]
            };
            let v = (1.0 - mix) * x[f] + mix * wet;
            out[f * ch + c] = (v as f32).clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Resample the envelope's impulse response by `alpha` — `g[n] = h[n·α]` — which
/// scales its spectrum by `α`: every formant lands `α` times higher.
///
/// Shifting **down** (`α < 1`) stretches the response, so the output is longer than
/// the input; it is grown to fit (capped at [`IR_MAX`]) instead of truncating the
/// tail, which would smear the very resonances being moved.
fn warp(h: &[f64], alpha: f64) -> Vec<f64> {
    let src_len = h.len();
    let out_len = ((src_len as f64 / alpha).ceil() as usize).clamp(1, IR_MAX);
    (0..out_len)
        .map(|n| {
            let pos = n as f64 * alpha;
            let i = pos.floor() as usize;
            let frac = pos - i as f64;
            let a = h.get(i).copied().unwrap_or(0.0);
            let b = h.get(i + 1).copied().unwrap_or(0.0);
            a * (1.0 - frac) + b * frac
        })
        .collect()
}

/// Drive the warped envelope with the excitation: a plain FIR convolution, truncated
/// to the block (the tail past it belongs to the next window's own pass).
fn convolve(excitation: &[f64], envelope: &[f64]) -> Vec<f64> {
    (0..excitation.len())
        .map(|n| {
            envelope
                .iter()
                .enumerate()
                .take_while(|(k, _)| *k <= n)
                .map(|(k, &g)| g * excitation[n - k])
                .sum()
        })
        .collect()
}

/// Match the re-synthesised block's energy to the windowed input's. Warping the
/// envelope changes the filter's total gain (a compressed impulse response integrates
/// to less), and an un-normalised shift would read as a volume ride rather than as a
/// change of speaker.
fn match_energy(y: &mut [f64], w: &[f64]) {
    let target: f64 = w.iter().map(|v| v * v).sum();
    let got: f64 = y.iter().map(|v| v * v).sum();
    if got > 1e-20 && target > 0.0 {
        let g = (target / got).sqrt();
        for v in y.iter_mut() {
            *v *= g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;
    /// The synthetic vowel's pitch (glottal pulse rate) and formant.
    const PITCH_HZ: f64 = 120.0;
    const FORMANT_HZ: f64 = 800.0;

    /// A synthetic vowel: a pulse train at [`PITCH_HZ`] (the excitation — the pitch)
    /// driven through a 2-pole resonator at [`FORMANT_HZ`] (the vocal tract — the
    /// formant). Exactly the model the effect claims to invert, so a shift must move
    /// the resonance and leave the pulse rate alone.
    fn vowel(n: usize) -> SampleData {
        let period = (f64::from(SR) / PITCH_HZ) as usize;
        // Resonator: two poles at radius r, angle w0.
        let r = 0.97;
        let w0 = TAU * FORMANT_HZ / f64::from(SR);
        let (a1, a2) = (2.0 * r * w0.cos(), -r * r);
        let mut y = vec![0.0f64; n];
        for i in 0..n {
            let drive = if i % period == 0 { 1.0 } else { 0.0 };
            let p1 = if i >= 1 { a1 * y[i - 1] } else { 0.0 };
            let p2 = if i >= 2 { a2 * y[i - 2] } else { 0.0 };
            y[i] = drive + p1 + p2;
        }
        let peak = y.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1e-9);
        let x: Vec<f32> = y.iter().map(|v| (v / peak * 0.7) as f32).collect();
        SampleData::from_interleaved(x, AudioFormat::mono(SR))
    }

    /// Energy at one frequency — a single DFT bin, computed directly (no FFT, no
    /// dependency). The oracle comes from the DEFINITION of a formant (energy at a
    /// frequency), not from anything the implementation does.
    fn energy_at(x: &[f32], hz: f64) -> f64 {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for (n, &v) in x.iter().enumerate() {
            let phase = TAU * hz * n as f64 / f64::from(SR);
            re += f64::from(v) * phase.cos();
            im -= f64::from(v) * phase.sin();
        }
        (re * re + im * im).sqrt() / x.len() as f64
    }

    /// Where the spectral envelope PEAKS — which is what a formant *is*.
    ///
    /// Read only at the excitation's own harmonics, because those are the only places
    /// a pulse train puts any energy: sampling the envelope between them (at a round
    /// 800 or 1600 Hz, say) measures spectral leakage from the neighbours, not the
    /// resonance. The peak can therefore only land ON a harmonic — 120 Hz is the
    /// quantum of this measurement, and the tolerances below say so.
    fn formant_peak_hz(x: &[f32]) -> f64 {
        (1..=40)
            .map(|k| k as f64 * PITCH_HZ)
            .max_by(|&a, &b| energy_at(x, a).total_cmp(&energy_at(x, b)))
            .expect("the harmonic sweep is not empty")
    }

    /// Periodicity at the pitch period: the normalised autocorrelation at lag
    /// `SR/PITCH`. High means the glottal pulse train is still ticking at its
    /// original rate — the property a formant shift must NOT disturb.
    fn periodicity(x: &[f32]) -> f64 {
        let lag = (f64::from(SR) / PITCH_HZ) as usize;
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for n in lag..x.len() {
            num += f64::from(x[n]) * f64::from(x[n - lag]);
            den += f64::from(x[n]) * f64::from(x[n]);
        }
        if den > 0.0 { num / den } else { 0.0 }
    }

    /// THE contract, and the reason this effect exists: the FORMANT moves and the
    /// PITCH does not. Shift up an octave and the envelope's peak must climb from
    /// 800 Hz to 1600 Hz while the 120 Hz periodicity survives intact.
    ///
    /// A granular pitch shifter fails the second half (it drags the pitch along); a
    /// plain resampler fails both. Nothing else in the rack can pass this.
    #[test]
    fn the_formant_moves_and_the_pitch_stays() {
        let d = vowel(24_000);
        let up = formant_shift(&d, 12.0, 1.0);
        // Skip the first block: its window ramps in from silence.
        let (dry, wet) = (&d.samples()[BLOCK..], &up.samples()[BLOCK..]);

        let peak_dry = formant_peak_hz(dry);
        let peak_wet = formant_peak_hz(wet);
        assert!(
            (peak_dry - FORMANT_HZ).abs() <= PITCH_HZ,
            "the fixture's formant is not where it should be: {peak_dry} Hz"
        );
        assert!(
            (peak_wet - FORMANT_HZ * 2.0).abs() <= PITCH_HZ,
            "the formant did not move up an octave: {peak_dry} Hz dry, {peak_wet} Hz wet"
        );

        // ...and the glottal pulse train keeps its rate.
        let (p_dry, p_wet) = (periodicity(dry), periodicity(wet));
        assert!(p_dry > 0.5, "the fixture is not periodic: {p_dry}");
        assert!(
            p_wet > p_dry * 0.9,
            "the pitch was disturbed: periodicity {p_dry} dry, {p_wet} wet"
        );
    }

    /// Shifting down is the mirror: the resonance drops. This is the "giant" — the
    /// same performance, at the same pitch, out of a bigger head.
    #[test]
    fn shifting_down_lowers_the_formant() {
        let d = vowel(24_000);
        let down = formant_shift(&d, -12.0, 1.0);
        let peak = formant_peak_hz(&down.samples()[BLOCK..]);
        assert!(
            (peak - FORMANT_HZ / 2.0).abs() <= PITCH_HZ,
            "the formant did not move down an octave: {peak} Hz"
        );
    }

    /// Zero shift reconstructs the signal: the residual driven back through its own
    /// (unwarped) envelope is the audio it came from. The rack bypasses this case
    /// before it ever calls in, but if the identity drifts, every shift is coloured by
    /// whatever the drift is.
    #[test]
    fn a_zero_shift_reconstructs_the_input() {
        let d = vowel(12_000);
        let out = formant_shift(&d, 0.0, 1.0);
        // Past the first window, which ramps in from silence.
        let worst = out.samples()[BLOCK..]
            .iter()
            .zip(&d.samples()[BLOCK..])
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 0.02, "reconstruction drifted by {worst}");
    }

    /// The seam must not fade: weighted overlap-add divides by the accumulated window
    /// energy, so the LAST block comes out at unity gain like every other. Without the
    /// division the tail of a selection would duck — an audible dip at its edge.
    #[test]
    fn the_last_block_does_not_fade_out() {
        let n = 12_000;
        let d = vowel(n);
        let out = formant_shift(&d, 5.0, 1.0);
        let rms = |s: &[f32]| (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt();
        let mid = rms(&out.samples()[n / 2 - 500..n / 2 + 500]);
        let tail = rms(&out.samples()[n - 1_000..]);
        assert!(
            tail > mid * 0.5,
            "the tail faded: rms {mid} mid, {tail} at the end"
        );
    }

    #[test]
    fn formant_shift_preserves_length_and_stays_bounded() {
        let d = vowel(9_600);
        let out = formant_shift(&d, 7.0, 1.0);
        assert_eq!(out.frame_count(), 9_600);
        assert!(out.samples().iter().all(|s| s.abs() <= 1.0));
    }

    /// `mix` 0 returns the dry signal exactly — the blend, pinned.
    #[test]
    fn mix_zero_is_dry() {
        let d = vowel(4_800);
        assert_eq!(formant_shift(&d, 7.0, 0.0).samples(), d.samples());
    }
}
