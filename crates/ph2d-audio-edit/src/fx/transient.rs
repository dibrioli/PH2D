//! **Transient shaper** — reshapes a hit's attack and sustain independently.
//!
//! Two amplitude followers track the signal: a **fast** one that leaps onto an onset
//! and then tracks the signal down, and a **slow** one that lags onto the onset and,
//! crucially, **releases slowly** so it stays elevated through the decay. Their *ratio*
//! `fast / slow` is the shape detector — above 1 while the fast follower leads (the
//! **attack**), below 1 through the tail where the slow one lingers above the decaying
//! fast one (the **body / sustain**). A ratio (not a difference) makes it
//! **level-independent**: a quiet hit is shaped like a loud one, and steady tone —
//! where the two followers sit together at ratio 1 — is left alone. Only transients
//! move.
//!
//! Control thread only. Reuses the dynamics kit's stereo-linked peak and one-pole
//! coefficient so a stereo hit keeps a coherent gain across both channels.

use ph2d_audio::SampleData;

use super::dynamics::{frame_peak, time_coeff};
use crate::ops::channels;

/// Fast follower attack: catches the leading edge of an onset.
const FAST_ATTACK_SECS: f32 = 0.002;
/// Fast follower release: tracks the signal down through the decay fairly closely.
const FAST_RELEASE_SECS: f32 = 0.020;
/// Slow follower attack: deliberately lags, so it trails the fast one on a hit.
const SLOW_ATTACK_SECS: f32 = 0.030;
/// Slow follower release: **long**, so through the decay tail it lingers above the
/// fast follower (ratio < 1) — that gap is the sustain the knob shapes. Without it the
/// two would converge a few ms after the peak and Sustain would do almost nothing.
const SLOW_RELEASE_SECS: f32 = 0.300;
/// Below this level there is no hit to shape, only noise — hold the gain at unity so a
/// silent gap between hits is never amplified.
const NOISE_FLOOR: f32 = 1e-3;
/// Gain clamp (±12 dB): a shaper drives peaks, but it must stay bounded.
const GAIN_MAX: f32 = 4.0;
const GAIN_MIN: f32 = 0.25;

/// Shifts this small leave the gain at unity — the neutral point the rack bypasses.
pub(super) const TRANSIENT_BYPASS: f32 = 1e-4;

/// Shape `data`'s transients: `attack` punches (+) or softens (−) onsets, `sustain`
/// lengthens (+) or tightens (−) the body. Same length out.
///
/// Called only off the neutral point (either knob past [`TRANSIENT_BYPASS`]); the
/// caller returns the input untouched otherwise.
pub(super) fn transient_shape(data: &SampleData, attack: f32, sustain: f32) -> SampleData {
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let src = data.samples();

    let fast_atk = time_coeff(FAST_ATTACK_SECS, sr);
    let slow_atk = time_coeff(SLOW_ATTACK_SECS, sr);
    let fast_rel = time_coeff(FAST_RELEASE_SECS, sr);
    let slow_rel = time_coeff(SLOW_RELEASE_SECS, sr);

    // Prime both followers on the first frame, so a region that opens on a hit starts
    // at ratio 1 (no edge jolt) and only a *rise* past that reads as an attack.
    let first = frame_peak(src, ch, 0);
    let mut ef = first;
    let mut es = first;

    let mut out = src.to_vec();
    for f in 0..frames {
        let a = frame_peak(src, ch, f);
        ef += (a - ef) * if a > ef { fast_atk } else { fast_rel };
        es += (a - es) * if a > es { slow_atk } else { slow_rel };

        let g = if es < NOISE_FLOOR {
            1.0
        } else {
            let r = ef / es;
            if r >= 1.0 {
                1.0 + attack * (r - 1.0) // fast leads → onset
            } else {
                1.0 + sustain * (1.0 - r) // slow leads → body
            }
        }
        .clamp(GAIN_MIN, GAIN_MAX);

        for c in 0..ch {
            let i = f * ch + c;
            out[i] = (src[i] * g).clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: f32 = 48_000.0;

    /// Silence, then a sharp-onset, exponentially-decaying 300 Hz tone — a caricature
    /// of a drum hit: a hard attack followed by a sustain tail.
    fn hit() -> SampleData {
        let tau = std::f32::consts::TAU;
        let onset = 2_400; // 50 ms of silence first
        let x: Vec<f32> = (0..12_000)
            .map(|n| {
                if n < onset {
                    return 0.0;
                }
                let t = (n - onset) as f32 / SR;
                0.6 * (-8.0 * t).exp() * (tau * 300.0 * t).sin()
            })
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(48_000))
    }

    fn peak(s: &[f32], from: usize, to: usize) -> f32 {
        s[from..to].iter().fold(0.0f32, |m, v| m.max(v.abs()))
    }
    fn energy(s: &[f32], from: usize) -> f32 {
        s[from..].iter().map(|v| v * v).sum()
    }

    #[test]
    fn transient_shape_preserves_length() {
        assert_eq!(transient_shape(&hit(), 1.0, 0.0).frame_count(), 12_000);
    }

    /// Both knobs at 0 is a byte-identical no-op — the neutral point.
    #[test]
    fn zero_is_identity() {
        let d = hit();
        assert_eq!(transient_shape(&d, 0.0, 0.0).samples(), d.samples());
    }

    /// THE attack contract: punching the attack raises the onset peak, softening it
    /// lowers it. Measured in a window right after the onset.
    #[test]
    fn attack_punches_and_softens_the_onset() {
        let d = hit();
        let (from, to) = (2_400, 2_400 + 480); // 10 ms after the onset
        let punch = peak(transient_shape(&d, 1.0, 0.0).samples(), from, to);
        let soften = peak(transient_shape(&d, -1.0, 0.0).samples(), from, to);
        let dry = peak(d.samples(), from, to);
        assert!(
            punch > dry * 1.2,
            "attack +1 did not punch: {punch} vs {dry}"
        );
        assert!(
            soften < dry * 0.8,
            "attack −1 did not soften: {soften} vs {dry}"
        );
    }

    /// THE sustain contract: raising sustain leaves more energy in the body than
    /// tightening it does. Attack held at 0 so only the body region differs.
    #[test]
    fn sustain_fills_and_tightens_the_body() {
        let d = hit();
        let body = 2_400 + 1_200; // past the onset, into the decay
        let fill = energy(transient_shape(&d, 0.0, 1.0).samples(), body);
        let tight = energy(transient_shape(&d, 0.0, -1.0).samples(), body);
        assert!(
            fill > tight,
            "sustain +1 must hold more than −1: {fill} vs {tight}"
        );
    }
}
