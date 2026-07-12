//! **WSOLA** — waveform-similarity overlap-add: the rack's time-scaler, and the
//! engine under [`Effect::PitchShift`](super::Effect::PitchShift) and the harmonizer.
//!
//! Reference: Verhelst & Roelands, "An overlap-add technique based on waveform
//! similarity (WSOLA) for high quality time-scale modification of speech" (ICASSP
//! 1993). Stretch a signal by overlap-adding grains of it — but before laying each
//! grain down, **search a small window of the input for the grain that best continues
//! the one already there**. The splice then lands on matching waveforms instead of on
//! an arbitrary phase, which is the whole ball game.
//!
//! ## Why not the delay-line granular shifter this replaced
//!
//! A two-tap delay line whose delay drifts and wraps every `GRAIN` samples re-splices
//! the waveform once per wrap — and since the grain length is fixed, the phase error
//! at that splice is the *same every time*: `frac(f·GRAIN/SR)` cycles. A constant
//! phase step per grain period is not a smear, it is a **frequency offset**:
//!
//! ```text
//!   Δf  =  frac(f_in · GRAIN / SR) · |rate| · SR / GRAIN
//! ```
//!
//! which detunes the output *flat*, worse the further you shift — measured at −23
//! cents for a third, −37 for a fifth and −54 for an octave, matching that formula on
//! all four data points. For a monster voice nobody notices. For a **harmonizer**,
//! which plays chords, a fifth 37 cents flat is simply out of tune, and no amount of
//! mixing hides it. WSOLA's splices are phase-aligned by construction, so there is no
//! per-grain phase step and no bias: the note lands where it was asked to.
//!
//! ## Pitch shift = resample, then stretch back
//!
//! Resampling by `r` moves the pitch **and the formants** by `r` and divides the
//! length by `r`; a WSOLA stretch by `r` puts the length back without touching either.
//! Net: pitch and formants up by `r`, same length — a tape speed change, which is
//! exactly what [`Effect::PitchShift`](super::Effect::PitchShift) promises (the
//! chipmunk / monster). Holding the formants still is a *different* effect and belongs
//! to [`super::formant`]; the two compose.
//!
//! Control thread only.

use std::f64::consts::TAU;

use ph2d_audio::SampleData;
use ph2d_audio::dsp::BiquadCoeffs;

use crate::ops::channels;

/// Grain length. ~21 ms at 48 kHz: long enough to hold a couple of periods of a low
/// voice, short enough that a transient is not smeared across it.
pub(super) const WINDOW: usize = 1_024;

/// Synthesis hop — half a grain, so a Hann window overlap-adds to unity.
const HOP: usize = WINDOW / 2;

/// How far the similarity search may slide the analysis position. It must cover a
/// whole period of the lowest pitch we care about, or the search cannot find the
/// phase-aligned splice it is looking for: ±400 samples spans 800, which is a 60 Hz
/// period at 48 kHz — below the bottom of a bass voice.
const SEARCH: usize = 400;

/// Samples compared when scoring a candidate splice. A quarter of a grain is plenty
/// to lock onto the waveform, and the search cost is linear in it.
const MATCH: usize = 256;

/// A shift this small is inaudible; below it the effect bypasses (and a
/// resample-then-stretch would still round every sample away from identity).
pub(super) const PITCH_BYPASS_ST: f32 = 1e-3;

/// Retune `data` by `semitones` (±), blended dry→wet by `mix`. Same length out.
///
/// Called only off the neutral point (`semitones` past [`PITCH_BYPASS_ST`] *and* `mix`
/// above 0); the caller returns the input untouched otherwise.
pub(super) fn pitch_shift(data: &SampleData, semitones: f32, mix: f32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    if frames == 0 || ch == 0 {
        return data.clone();
    }
    let mix = f64::from(mix.clamp(0.0, 1.0));
    let ratio = (f64::from(semitones) / 12.0).exp2();
    let sr = f64::from(data.format().sample_rate);

    let src = data.samples();
    // One allocation, not two (ADR-0117 D2). Safe in place: `dry` is taken from the PRISTINE
    // `src`, so the crossfade never reads a sample the loop has already written.
    SampleData::map_in_place(data, |out| {
        for c in 0..ch {
            let dry: Vec<f64> = (0..frames).map(|f| f64::from(src[f * ch + c])).collect();
            // Reading the signal FASTER than it was written folds everything above
            // `Nyquist/ratio` back down as alias. Filter it off first — the fold is
            // inharmonic, so it does not sound like the note, it sounds like grit.
            //
            // On a COPY: the anti-alias filter belongs to the shifted voice, not to the
            // dry one. Filtering in place would darken the dry side of the crossfade too,
            // so `mix` would fade between the wet voice and a muffled ghost of the input
            // rather than the input itself.
            let mut source = dry.clone();
            if ratio > 1.0 {
                lowpass_in_place(&mut source, sr, 0.45 * sr / ratio);
            }
            // Resample (pitch and formants move, length changes), then put the length back
            // without disturbing either.
            let wet = time_scale(&resample(&source, ratio), ratio, frames);
            for (f, &w) in wet.iter().enumerate().take(frames) {
                out[f * ch + c] = (((1.0 - mix) * dry[f] + mix * w) as f32).clamp(-1.0, 1.0);
            }
        }
    })
}

/// Read `x` at `ratio` times its written speed: the pitch (and the formants with it)
/// scale by `ratio`, and the signal comes out `1/ratio` as long.
fn resample(x: &[f64], ratio: f64) -> Vec<f64> {
    let out_len = ((x.len() as f64 / ratio).floor() as usize).max(1);
    (0..out_len)
        .map(|n| {
            let pos = n as f64 * ratio;
            let i = pos.floor() as usize;
            let frac = pos - i as f64;
            let a = x.get(i).copied().unwrap_or(0.0);
            let b = x.get(i + 1).copied().unwrap_or(0.0);
            a * (1.0 - frac) + b * frac
        })
        .collect()
}

/// One-pole-pair anti-alias filter, applied in place (direct form I).
fn lowpass_in_place(x: &mut [f64], sr: f64, cutoff_hz: f64) {
    let c = BiquadCoeffs::lowpass(sr as f32, cutoff_hz.max(1.0) as f32, 0.707);
    let (b0, b1, b2) = (f64::from(c.b0), f64::from(c.b1), f64::from(c.b2));
    let (a1, a2) = (f64::from(c.a1), f64::from(c.a2));
    let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
    for v in x.iter_mut() {
        let xn = *v;
        let yn = b0 * xn + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        x2 = x1;
        x1 = xn;
        y2 = y1;
        y1 = yn;
        *v = yn;
    }
}

/// Stretch `x` to `scale` times its length by waveform-similarity overlap-add, then
/// fit the result to exactly `want` samples.
///
/// Each grain is drawn from near the position the stretch factor calls for, but slid
/// by up to [`SEARCH`] samples to whichever offset best **continues the grain already
/// laid down**. That is the one line that separates WSOLA from plain overlap-add, and
/// the reason the output holds its pitch instead of drifting flat.
fn time_scale(x: &[f64], scale: f64, want: usize) -> Vec<f64> {
    let n_out = ((x.len() as f64 * scale).round() as usize).max(1);
    let hann: Vec<f64> = (0..WINDOW)
        .map(|i| 0.5 - 0.5 * (TAU * i as f64 / WINDOW as f64).cos())
        .collect();

    let mut acc = vec![0.0f64; n_out + WINDOW];
    let mut wsum = vec![0.0f64; n_out + WINDOW];
    // Where the audio that would naturally follow the last grain begins.
    let mut natural = 0usize;
    let mut m = 0usize;
    while m * HOP < n_out {
        let out_pos = m * HOP;
        let ideal = ((out_pos as f64) / scale).round() as usize;
        // The first grain has nothing to continue: take it where it lies.
        let start = if m == 0 {
            ideal.min(x.len().saturating_sub(1))
        } else {
            best_match(x, ideal, natural)
        };
        for i in 0..WINDOW {
            let Some(&v) = x.get(start + i) else { break };
            acc[out_pos + i] += v * hann[i];
            wsum[out_pos + i] += hann[i];
        }
        natural = start + HOP;
        m += 1;
    }

    let mut out = vec![0.0f64; want];
    for (f, slot) in out.iter_mut().enumerate() {
        // Past the stretched audio there is nothing to say; inside it, dividing by the
        // accumulated window makes every seam — including the first and last — land at
        // unity gain rather than fading.
        if f < n_out && wsum[f] > 1e-9 {
            *slot = acc[f] / wsum[f];
        }
    }
    out
}

/// Time-stretch `data` to exactly `new_frames` — same pitch, different length.
///
/// This is the **Scale** tool: shorten a take without trimming it, or stretch it to land on a
/// beat. Pitch shift is the other half of the same engine (resample, then stretch back); here
/// there is no resample, so nothing moves but the clock.
///
/// ## The channels are stretched TOGETHER
///
/// [`time_scale`] searches each signal it is given for the best splice. Run it once per channel
/// and the two channels of a stereo take choose **different** splices — same length out, but the
/// waveforms no longer line up, and the stereo image wanders and goes phasey. So the similarity
/// search runs once, on the channel sum, and every channel is overlap-added at the **same**
/// offsets: whatever the splice does to the image, it does to both channels identically, which
/// is the definition of leaving the image alone.
///
/// (Pitch shift keeps its own per-channel path, untouched — the rack's outputs are byte-identical
/// under a gate, and this is a new operation, not a change to that one.)
pub(crate) fn stretch(data: &SampleData, new_frames: usize) -> SampleData {
    let frames = data.frame_count();
    let want = new_frames.max(1);
    // Asking for the length it already has is not a stretch. Returning the input untouched keeps
    // a zero-move drag byte-identical, instead of running the audio through the grain mill for
    // nothing and rounding every sample.
    if want == frames || frames == 0 {
        return data.clone();
    }
    let ch = channels(data);
    let src = data.samples();

    let chans: Vec<Vec<f64>> = (0..ch)
        .map(|c| (0..frames).map(|f| f64::from(src[f * ch + c])).collect())
        .collect();
    // The key the search reads: the channel sum. It hears everything the channels hear together,
    // which is what a splice has to be good for.
    let key: Vec<f64> = (0..frames)
        .map(|f| (0..ch).map(|c| chans[c][f]).sum::<f64>() / ch as f64)
        .collect();

    let scale = want as f64 / frames as f64;
    let out = time_scale_linked(&chans, &key, scale, want);
    SampleData::from_fn(want * ch, data.format(), |i| {
        let (f, c) = (i / ch, i % ch);
        out[c][f].clamp(-1.0, 1.0) as f32
    })
}

/// Overlap-add every channel of `chans` at the splice offsets chosen by searching `key` — the
/// shared-offset core of [`stretch`]. Mirrors [`time_scale`] exactly, except that the offset is
/// decided once and spent on every channel.
fn time_scale_linked(chans: &[Vec<f64>], key: &[f64], scale: f64, want: usize) -> Vec<Vec<f64>> {
    let n_out = ((key.len() as f64 * scale).round() as usize).max(1);
    let hann: Vec<f64> = (0..WINDOW)
        .map(|i| 0.5 - 0.5 * (TAU * i as f64 / WINDOW as f64).cos())
        .collect();

    let mut acc = vec![vec![0.0f64; n_out + WINDOW]; chans.len()];
    // One window-sum for all channels: they are laid down at the same offsets, so they accumulate
    // the same window.
    let mut wsum = vec![0.0f64; n_out + WINDOW];
    let mut natural = 0usize;
    let mut m = 0usize;
    while m * HOP < n_out {
        let out_pos = m * HOP;
        let ideal = ((out_pos as f64) / scale).round() as usize;
        let start = if m == 0 {
            ideal.min(key.len().saturating_sub(1))
        } else {
            best_match(key, ideal, natural)
        };
        for i in 0..WINDOW {
            if start + i >= key.len() {
                break;
            }
            wsum[out_pos + i] += hann[i];
            for (c, chan) in chans.iter().enumerate() {
                acc[c][out_pos + i] += chan[start + i] * hann[i];
            }
        }
        natural = start + HOP;
        m += 1;
    }

    acc.iter()
        .map(|a| {
            (0..want)
                .map(|f| {
                    if f < n_out && wsum[f] > 1e-9 {
                        a[f] / wsum[f]
                    } else {
                        0.0
                    }
                })
                .collect()
        })
        .collect()
}

/// Slide the analysis position within ±[`SEARCH`] of `ideal` and return the offset
/// whose waveform best matches `natural` — the audio that would have followed the
/// grain already on the output.
///
/// Scored by normalised cross-correlation, so a loud candidate cannot win on volume
/// alone. The anchor's own energy is constant across candidates and drops out.
fn best_match(x: &[f64], ideal: usize, natural: usize) -> usize {
    let anchor: &[f64] = match x.get(natural..(natural + MATCH).min(x.len())) {
        Some(a) if !a.is_empty() => a,
        _ => return ideal.min(x.len().saturating_sub(1)),
    };
    let lo = ideal.saturating_sub(SEARCH);
    let hi = (ideal + SEARCH).min(x.len().saturating_sub(1));

    let mut best = (lo, f64::NEG_INFINITY);
    for cand in lo..=hi {
        let mut dot = 0.0f64;
        let mut energy = 0.0f64;
        for (i, &a) in anchor.iter().enumerate() {
            let Some(&v) = x.get(cand + i) else { break };
            dot += a * v;
            energy += v * v;
        }
        // An empty (silent) candidate scores zero rather than NaN.
        let score = if energy > 1e-20 {
            dot / energy.sqrt()
        } else {
            0.0
        };
        if score > best.1 {
            best = (cand, score);
        }
    }
    best.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;

    fn tone(hz: f32, n: usize) -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..n)
            .map(|i| 0.6 * (tau * hz * i as f32 / SR as f32).sin())
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(SR))
    }

    /// Energy at one frequency — a single DFT bin, computed directly (no FFT).
    fn energy_at(x: &[f32], hz: f64) -> f64 {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for (n, &v) in x.iter().enumerate() {
            let phase = TAU * hz * n as f64 / f64::from(SR);
            re += f64::from(v) * phase.cos();
            im -= f64::from(v) * phase.sin();
        }
        (re * re + im * im).sqrt() / x.len() as f64
    }

    /// Where the output's energy actually peaks, swept finely around `target`.
    fn peak_near(x: &[f32], target: f64) -> f64 {
        let mut best = (target, f64::NEG_INFINITY);
        let mut hz = target * 0.90;
        while hz < target * 1.10 {
            let e = energy_at(x, hz);
            if e > best.1 {
                best = (hz, e);
            }
            hz += 0.25;
        }
        best.0
    }

    /// Cents between two frequencies — the unit the ear actually hears.
    fn cents(got: f64, want: f64) -> f64 {
        1_200.0 * (got / want).log2()
    }

    /// THE bug this engine exists to kill, pinned as a red assertion.
    ///
    /// The delay-line granular shifter this replaced re-spliced the waveform at a
    /// fixed grain boundary, so every wrap injected the SAME phase error and the pitch
    /// came out systematically **flat** — measured at −23 cents for a third, −37 for a
    /// fifth, −54 for an octave. Nobody hears that on a monster voice; on the
    /// harmonizer's chords it is simply out of tune.
    ///
    /// WSOLA aligns each splice to the waveform, so the note lands where it was asked
    /// to. Ten cents is the threshold where a trained ear starts to notice a sustained
    /// interval; hold the whole slider inside it.
    #[test]
    fn the_shifted_note_lands_in_tune() {
        let root = 300.0f64;
        let d = tone(root as f32, 24_000);
        for st in [-12.0f32, -7.0, -5.0, 4.0, 7.0, 12.0] {
            let out = pitch_shift(&d, st, 1.0);
            let want = root * (f64::from(st) / 12.0).exp2();
            // Past the first grain, which has no history to align against.
            let got = peak_near(&out.samples()[WINDOW..], want);
            let off = cents(got, want);
            assert!(
                off.abs() < 10.0,
                "{st} st: wanted {want:.1} Hz, got {got:.1} Hz ({off:+.1} cents)"
            );
        }
    }

    /// ...and the note is actually THERE, not merely in tune on a whisper of energy.
    /// A stretch that dropped its grains would still pass the tuning check.
    #[test]
    fn the_shifted_note_keeps_its_level() {
        let d = tone(300.0, 24_000);
        let dry_level = energy_at(&d.samples()[WINDOW..], 300.0);
        for st in [-7.0f32, 7.0] {
            let out = pitch_shift(&d, st, 1.0);
            let want = 300.0 * (f64::from(st) / 12.0).exp2();
            let got = energy_at(
                &out.samples()[WINDOW..],
                peak_near(&out.samples()[WINDOW..], want),
            );
            assert!(
                got > dry_level * 0.5,
                "{st} st: the note came out at {got} against a dry {dry_level}"
            );
        }
    }

    #[test]
    fn pitch_shift_preserves_length() {
        assert_eq!(
            pitch_shift(&tone(300.0, 9_600), 7.0, 1.0).frame_count(),
            9_600
        );
    }

    /// Never exceeds full scale: a convex blend of bounded, unity-gain seams.
    #[test]
    fn pitch_shift_stays_bounded() {
        let d = tone(300.0, 12_000);
        for st in [-12.0f32, -5.0, 3.0, 12.0] {
            let out = pitch_shift(&d, st, 1.0);
            assert!(
                out.samples().iter().all(|s| s.abs() <= 1.0),
                "clipped at {st} st"
            );
        }
    }

    /// A stretch must not fade at its seams — the accumulated window is divided out,
    /// so the middle and the tail come back at the same level.
    #[test]
    fn the_stretch_holds_its_level_across_the_seams() {
        let d = tone(300.0, 24_000);
        let out = pitch_shift(&d, 5.0, 1.0);
        let rms = |s: &[f32]| (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt();
        let mid = rms(&out.samples()[10_000..14_000]);
        let tail = rms(&out.samples()[19_000..23_000]);
        assert!(
            tail > mid * 0.7 && mid > 0.1,
            "the seams faded: rms {mid} mid, {tail} tail"
        );
    }

    /// `mix` 0 returns the dry signal exactly — the blend, pinned. (The rack bypasses
    /// this case before it ever calls in, but the math must still be honest.)
    #[test]
    fn mix_zero_is_dry() {
        let d = tone(300.0, 4_800);
        assert_eq!(pitch_shift(&d, 7.0, 0.0).samples(), d.samples());
    }

    /// Stereo shifts per channel and keeps its length.
    #[test]
    fn stereo_is_shifted_per_channel() {
        let n = 9_600;
        let inter: Vec<f32> = (0..n)
            .flat_map(|i| {
                let t = i as f32 / SR as f32;
                let tau = std::f32::consts::TAU;
                [0.5 * (tau * 300.0 * t).sin(), 0.5 * (tau * 500.0 * t).sin()]
            })
            .collect();
        let d = SampleData::from_interleaved(inter, AudioFormat::stereo(SR));
        let out = pitch_shift(&d, 12.0, 1.0);
        assert_eq!(out.frame_count(), n);
        let left: Vec<f32> = out.samples().iter().step_by(2).copied().collect();
        let right: Vec<f32> = out.samples().iter().skip(1).step_by(2).copied().collect();
        // Each channel's own note doubled: 300 -> 600, 500 -> 1000.
        assert!(cents(peak_near(&left[WINDOW..], 600.0), 600.0).abs() < 20.0);
        assert!(cents(peak_near(&right[WINDOW..], 1_000.0), 1_000.0).abs() < 20.0);
    }
}
