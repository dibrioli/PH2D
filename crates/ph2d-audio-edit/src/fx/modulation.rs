//! Modulation effects for the rack: chorus, flanger, phaser, tremolo, ring mod.
//!
//! All are **length-preserving** (they route through [`crate::in_range_warm`] like
//! the filters). Chorus and flanger share a modulated fractional-delay line; the
//! phaser is a modulated all-pass cascade; tremolo is sub-audio amplitude modulation;
//! ring mod is amplitude modulation by an *audio-rate* carrier. The first four ride a
//! **sine LFO**; the ring modulator's sine is the carrier itself.
//!
//! Control thread only — allocates and uses `sin`/`tan` freely.

use std::f32::consts::TAU;

use ph2d_audio::SampleData;

use crate::ops::channels;

/// The stereo LFO phase offset between L and R: a quarter cycle. Zero would make a
/// stereo chorus collapse to a mono one (both sides sweep identically); 90° gives the
/// classic wide, drifting image.
const STEREO_PHASE: f32 = TAU / 4.0;

/// Read `buf` at a **fractional** sample offset behind `write`, wrapping. Linear
/// interpolation between the two neighbouring samples — a modulated delay lands
/// between samples on almost every frame, and rounding to the nearest one zippers.
fn frac_read(buf: &[f32], write: usize, delay: f32) -> f32 {
    let len = buf.len();
    let mut rp = write as f32 - delay;
    while rp < 0.0 {
        rp += len as f32;
    }
    let i0 = rp.floor() as usize % len;
    let i1 = (i0 + 1) % len;
    let frac = rp - rp.floor();
    buf[i0] * (1.0 - frac) + buf[i1] * frac
}

/// A modulated delay line — the core of both **chorus** and **flanger**. The read
/// tap sweeps between `base_ms` and `base_ms + depth_ms` at `rate_hz`; `feedback`
/// routes the delayed signal back in (a flanger's resonant whoosh); `mix` crossfades
/// dry→wet. Stereo channels sweep a quarter-cycle apart for width.
///
/// Chorus = a longer base delay, no feedback (a detuned doubling); flanger = a very
/// short base delay with feedback (a sweeping comb). One function, two presets.
pub(super) fn modulated_delay(
    data: &SampleData,
    base_ms: f32,
    depth_ms: f32,
    rate_hz: f32,
    feedback: f32,
    mix: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mix = mix.clamp(0.0, 1.0);
    let feedback = feedback.clamp(0.0, 0.95); // < 1 or the comb self-oscillates
    let base = (base_ms * 0.001 * sr).max(1.0);
    let depth = (depth_ms * 0.001 * sr).max(0.0);
    // The line must hold the deepest tap plus a sample for interpolation.
    let len = (base + depth) as usize + 4;
    let step = TAU * rate_hz / sr;

    let mut bufs: Vec<Vec<f32>> = (0..ch).map(|_| vec![0.0f32; len]).collect();
    let mut write = 0usize;
    let src = data.samples();
    let mut out = src.to_vec();
    for f in 0..frames {
        for (c, buf) in bufs.iter_mut().enumerate() {
            let phase = step * f as f32 + STEREO_PHASE * c as f32;
            // LFO 0..1 → tap sweeps base .. base+depth.
            let delay = base + depth * (0.5 + 0.5 * phase.sin());
            let delayed = frac_read(buf, write, delay);
            let x = src[f * ch + c];
            buf[write] = x + feedback * delayed;
            out[f * ch + c] = ((1.0 - mix) * x + mix * delayed).clamp(-1.0, 1.0);
        }
        write = (write + 1) % len;
    }
    SampleData::from_interleaved(out, data.format())
}

/// Number of first-order all-pass sections. Four gives two moving notches — the
/// classic phaser. More is smoother but muddier; fewer barely swirls.
const PHASER_STAGES: usize = 4;
/// The all-pass sweep rides between these corner frequencies (Hz), scaled by `depth`.
const PHASER_FC_MIN: f32 = 200.0;
const PHASER_FC_MAX: f32 = 2_000.0;

/// **Phaser**: a cascade of modulated first-order all-passes mixed with the dry
/// signal. Where the cascade's phase hits 180° the sum notches; sweeping the corner
/// frequency at `rate_hz` slides the notches, the swirl. `depth` (0..1) scales how
/// far the corner sweeps; `mix` crossfades dry→wet.
pub(super) fn phaser(data: &SampleData, rate_hz: f32, depth: f32, mix: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mix = mix.clamp(0.0, 1.0);
    let depth = depth.clamp(0.0, 1.0);
    let step = TAU * rate_hz / sr;

    // Per-channel all-pass state: the input and output of each stage last frame.
    let mut xprev = vec![[0.0f32; PHASER_STAGES]; ch];
    let mut yprev = vec![[0.0f32; PHASER_STAGES]; ch];
    let src = data.samples();
    let mut out = src.to_vec();
    for f in 0..frames {
        for c in 0..ch {
            let phase = step * f as f32 + STEREO_PHASE * c as f32;
            // Corner sweeps log-symmetrically around the band, `depth` sets the span.
            let lfo = 0.5 + 0.5 * phase.sin(); // 0..1
            let fc = PHASER_FC_MIN * (PHASER_FC_MAX / PHASER_FC_MIN).powf(depth * lfo);
            // First-order all-pass coefficient for corner `fc` (bilinear).
            let t = (std::f32::consts::PI * fc / sr).tan();
            let a = (t - 1.0) / (t + 1.0);

            let x = src[f * ch + c];
            let mut sig = x;
            for s in 0..PHASER_STAGES {
                let y = a * sig + xprev[c][s] - a * yprev[c][s];
                xprev[c][s] = sig;
                yprev[c][s] = y;
                sig = y;
            }
            out[f * ch + c] = ((1.0 - mix) * x + mix * sig).clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// **Tremolo**: amplitude modulation. The gain rides `1 .. 1-depth` at `rate_hz`, so
/// `depth` 0 is a byte-identical no-op (`× 1.0`) and `depth` 1 dips to silence at each
/// trough. Both channels are modulated in phase — an out-of-phase version would be an
/// auto-pan, a different effect.
pub(super) fn tremolo(data: &SampleData, rate_hz: f32, depth: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let depth = depth.clamp(0.0, 1.0);
    let step = TAU * rate_hz / sr;
    let src = data.samples();
    let mut out = src.to_vec();
    for f in 0..frames {
        let lfo = (step * f as f32).sin(); // -1..1
        let gain = 1.0 - depth * (0.5 - 0.5 * lfo); // 1 .. 1-depth
        for c in 0..ch {
            out[f * ch + c] = src[f * ch + c] * gain;
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// **Ring modulator**: multiplies the signal by a sine **carrier** at `freq` — the
/// classic robot / Dalek / metallic voice. Where tremolo is amplitude modulation at a
/// *sub-audio* rate (a wobble you hear as loudness), the carrier here sits *in* the
/// audio band, so it splits every input partial into sum-and-difference sidebands: an
/// inharmonic, clangorous timbre. One carrier drives both channels (a per-channel
/// phase would smear a stereo source), and `mix` crossfades dry→wet, so `mix` 0 is a
/// pass-through — the neutral point the rack's bypass relies on.
///
/// Output stays bounded: it is a convex blend of `x` and `x·carrier`, both `≤ |x|`.
pub(super) fn ring_mod(data: &SampleData, freq: f32, mix: f32) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mix = mix.clamp(0.0, 1.0);
    let step = TAU * freq / sr;
    let src = data.samples();
    let mut out = src.to_vec();
    // Accumulate and wrap the phase instead of `sin(step * f)`: over a long clip the
    // bare argument grows large enough to lose carrier precision.
    let mut phase = 0.0f32;
    for f in 0..frames {
        let carrier = phase.sin();
        for c in 0..ch {
            let x = src[f * ch + c];
            out[f * ch + c] = (1.0 - mix) * x + mix * (x * carrier);
        }
        phase += step;
        if phase >= TAU {
            phase -= TAU;
        }
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
    fn mono(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::mono(48_000))
    }

    /// A 440 Hz stereo tone, 4 800 frames — every modulation effect leaves a mark.
    fn probe() -> SampleData {
        let tau = std::f32::consts::TAU;
        stereo(
            (0..4_800)
                .flat_map(|n| {
                    let t = n as f32 / 48_000.0;
                    let s = 0.5 * (tau * 440.0 * t).sin();
                    [s, s]
                })
                .collect(),
        )
    }

    /// Each effect must keep the frame count — they route through the
    /// length-preserving splice.
    #[test]
    fn modulation_preserves_length() {
        let d = probe();
        assert_eq!(
            modulated_delay(&d, 18.0, 5.0, 1.0, 0.0, 0.5).frame_count(),
            4_800
        );
        assert_eq!(phaser(&d, 0.5, 0.8, 0.5).frame_count(), 4_800);
        assert_eq!(tremolo(&d, 5.0, 0.8).frame_count(), 4_800);
    }

    /// THE delay-line invariant: with depth 0 the tap is a *fixed* delay, so fully
    /// wet with no feedback an impulse comes back out exactly `base_ms` later. Pins
    /// that the fractional read lands where it claims.
    #[test]
    fn a_fully_wet_delay_echoes_the_impulse_at_the_base_time() {
        // 1 kHz sample rate makes the arithmetic legible: 10 ms = 10 samples.
        let mut x = vec![0.0f32; 64];
        x[0] = 1.0;
        let d = SampleData::from_interleaved(x, AudioFormat::mono(1_000));
        let out = modulated_delay(&d, 10.0, 0.0, 0.0, 0.0, 1.0); // base 10 ms, no depth, wet
        assert!(
            out.samples()[10] > 0.9,
            "impulse should re-emerge at frame 10, got {:?}",
            &out.samples()[8..13]
        );
        assert!(
            out.samples()[0].abs() < 1e-6,
            "fully wet: no dry at frame 0"
        );
    }

    /// Feedback must recirculate: the same impulse comes back a second time, quieter.
    #[test]
    fn delay_feedback_recirculates() {
        let mut x = vec![0.0f32; 64];
        x[0] = 1.0;
        let d = SampleData::from_interleaved(x, AudioFormat::mono(1_000));
        let out = modulated_delay(&d, 10.0, 0.0, 0.0, 0.5, 1.0);
        assert!(out.samples()[10] > 0.9, "first echo");
        assert!(
            out.samples()[20] > 0.4 && out.samples()[20] < out.samples()[10],
            "second echo is present and decayed: {:?}",
            &out.samples()[18..23]
        );
    }

    /// Every effect changes the audio when engaged, and never exceeds full scale.
    #[test]
    fn engaged_modulation_changes_the_audio_and_stays_bounded() {
        let d = probe();
        for out in [
            modulated_delay(&d, 18.0, 5.0, 1.0, 0.3, 0.5),
            phaser(&d, 0.5, 0.8, 0.6),
            tremolo(&d, 5.0, 0.8),
        ] {
            assert_ne!(out.samples(), d.samples(), "effect did nothing");
            assert!(
                out.samples().iter().all(|s| s.abs() <= 1.0 + 1e-4),
                "clipped"
            );
        }
    }

    /// Tremolo actually modulates the amplitude at its rate: a steady tone comes out
    /// with an envelope that dips. Deeper depth dips further.
    #[test]
    fn tremolo_modulates_the_envelope() {
        // A DC-ish steady 0.8 so "envelope" is just |output|.
        let d = mono(vec![0.8f32; 48_000]);
        let out = tremolo(&d, 4.0, 1.0); // 4 Hz, full depth → dips to 0
        let s = out.samples();
        let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        let trough = s.iter().fold(1.0f32, |m, v| m.min(v.abs()));
        assert!(peak > 0.79, "peak should reach the input level, got {peak}");
        assert!(
            trough < 0.05,
            "full depth should dip to ~silence, got {trough}"
        );

        // Half depth dips only halfway.
        let half = tremolo(&d, 4.0, 0.5);
        let half_trough = half.samples().iter().fold(1.0f32, |m, v| m.min(v.abs()));
        assert!(
            half_trough > 0.35 && half_trough < 0.45,
            "half depth troughs near 0.4, got {half_trough}"
        );
    }

    /// Depth 0 tremolo is a byte-identical no-op — `× 1.0` at every frame.
    #[test]
    fn tremolo_at_zero_depth_is_identity() {
        let d = probe();
        assert_eq!(tremolo(&d, 5.0, 0.0).samples(), d.samples());
    }

    /// A deeper phaser sweep deviates further from the dry signal — `depth` is a real
    /// knob, not decoration.
    #[test]
    fn phaser_depth_deepens_the_effect() {
        let d = probe();
        let dev = |out: &SampleData| -> f32 {
            out.samples()
                .iter()
                .zip(d.samples())
                .map(|(a, b)| (a - b).abs())
                .sum()
        };
        let shallow = phaser(&d, 0.5, 0.2, 1.0);
        let deep = phaser(&d, 0.5, 1.0, 1.0);
        assert!(
            dev(&deep) > dev(&shallow),
            "deeper sweep must deviate more: {} vs {}",
            dev(&deep),
            dev(&shallow)
        );
    }

    /// The stereo LFO phase offset must actually split the channels — a stereo chorus
    /// whose sides move together is a mono chorus in a trenchcoat.
    #[test]
    fn stereo_channels_modulate_out_of_phase() {
        let d = probe();
        let out = modulated_delay(&d, 18.0, 8.0, 2.0, 0.0, 1.0);
        let s = out.samples();
        // The input is identical L/R, so any L≠R is the phase offset at work.
        let diff: f32 = (0..4_800).map(|f| (s[f * 2] - s[f * 2 + 1]).abs()).sum();
        assert!(diff > 1.0, "channels stayed in lockstep: diff {diff}");
    }

    /// Mix 0 is a byte-identical pass-through — the rack's neutral point.
    #[test]
    fn ring_mod_at_zero_mix_is_identity() {
        let d = probe();
        assert_eq!(ring_mod(&d, 500.0, 0.0).samples(), d.samples());
    }

    /// The defining behaviour, pinned exactly: fully wet, a DC input becomes the
    /// carrier itself (`x·sin` with x constant), so a 0.5 DC through a 100 Hz carrier
    /// comes out a 100 Hz sine of amplitude 0.5 — swinging symmetrically through zero.
    #[test]
    fn ring_mod_turns_dc_into_the_carrier() {
        let d = mono(vec![0.5f32; 48_000]);
        let out = ring_mod(&d, 100.0, 1.0);
        let s = out.samples();
        let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        let min = s.iter().fold(0.0f32, |m, v| m.min(*v));
        let mean = s.iter().sum::<f32>() / s.len() as f32;
        assert!(
            (peak - 0.5).abs() < 0.01,
            "carrier peak should be 0.5, got {peak}"
        );
        assert!(min < -0.4, "carrier must swing negative, got {min}");
        assert!(
            mean.abs() < 0.01,
            "a symmetric carrier averages to ~0, got {mean}"
        );
    }

    /// `mix` is a real knob: a deeper blend deviates further from the dry signal.
    #[test]
    fn ring_mod_mix_scales_the_effect() {
        let d = probe();
        let dev = |out: &SampleData| -> f32 {
            out.samples()
                .iter()
                .zip(d.samples())
                .map(|(a, b)| (a - b).abs())
                .sum()
        };
        let shallow = ring_mod(&d, 500.0, 0.3);
        let deep = ring_mod(&d, 500.0, 1.0);
        assert!(
            dev(&deep) > dev(&shallow),
            "deeper mix must deviate more: {} vs {}",
            dev(&deep),
            dev(&shallow)
        );
        assert!(
            deep.samples().iter().all(|s| s.abs() <= 1.0 + 1e-4),
            "ring mod clipped"
        );
    }
}
