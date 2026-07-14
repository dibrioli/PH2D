//! **Channel vocoder** — the voice's spectral envelope, played by a synthetic carrier.
//!
//! A bank of band-pass filters splits the voice (the *modulator*) into N bands and follows
//! the level of each. The same bank splits a carrier the effect synthesises, and each carrier
//! band is scaled by the voice's level in that band. What comes out has the voice's
//! **formants** — its vowels, its consonants, its intelligibility — riding on the carrier's
//! **excitation**. Say something into a sawtooth and you get Kraftwerk.
//!
//! # One input, so the carrier is internal — and that is what collapses the family
//!
//! A studio vocoder takes two signals. An effect in this rack takes one, so the carrier is
//! synthesised here, at a pitch the user sets. Which means **"Robotize" is not a second
//! effect** — a vocoder with a fixed-pitch carrier *is* the robot, and it is this one with
//! `breath` at 0. And a vocoder with a **noise** carrier is not a third effect either: it is
//! a **whisper**, because unvoiced excitation through a vocal tract is what whispering
//! physically *is*. One engine, one knob ([`breath`](Effect::Vocoder)) between them, and the
//! two sounds are presets rather than rows.
//!
//! # The carrier has to be band-limited, or the robot is made of aliasing
//!
//! A naive sawtooth folds every harmonic above Nyquist back down into the audible band, and
//! the fold-down lands *inharmonically* — it does not sound like a bright saw, it sounds
//! broken. So the saw is built by **additive synthesis into a wavetable of exactly one
//! period**: harmonics up to Nyquist and not one more. The period is an integer number of
//! samples, so the table loops with no interpolation and no drift at all — at the cost of
//! quantising the carrier pitch to the nearest whole period (a few cents at most, which for
//! a robot voice is not a pitch anyone is checking).
//!
//! Control thread only, so `sin`/`exp` are free (HR-5 does not apply).

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs};

use super::dynamics::time_coeff;
use crate::ops::channels;

/// Bottom of the analysed band. Below this is the voice's fundamental, which carries pitch,
/// not identity — and pitch is precisely what the carrier is here to replace.
pub(super) const VOC_LO_HZ: f32 = 120.0;
/// Top of the analysed band: sibilance lives under this, and above it a band-pass bank is
/// mostly following hiss.
const VOC_HI_HZ: f32 = 8_000.0;
/// Fewest and most bands. Under ~4 the vowels stop being distinguishable; over ~32 the bands
/// are narrower than a formant and the vocoder just resynthesises the input.
pub(super) const VOC_MIN_BANDS: u32 = 4;
pub(super) const VOC_MAX_BANDS: u32 = 32;
/// The envelope follower: fast enough to catch a consonant, slow enough not to track the
/// pitch period itself (a 120 Hz fundamental has an 8 ms period).
const ENV_ATTACK_S: f32 = 0.003;
const ENV_RELEASE_S: f32 = 0.020;
/// Floor under the carrier's own band level, so **whitening** it (dividing each carrier band
/// by its own envelope, which is what stops a saw's 1/f rolloff from making every robot dull)
/// cannot divide by zero in a band the carrier has no energy in — a 400 Hz saw genuinely has
/// nothing below 400 Hz, and the honest answer there is silence, not gain.
const CARRIER_FLOOR: f32 = 1e-3;
/// A vocoder is neutral when it is fully dry.
pub(super) const VOC_BYPASS_MIX: f32 = 0.0;

/// Band centres, log-spaced — the ear's spacing, and the spacing formants actually keep.
fn centres(bands: usize) -> Vec<f32> {
    let ratio = (VOC_HI_HZ / VOC_LO_HZ).powf(1.0 / (bands - 1).max(1) as f32);
    (0..bands)
        .map(|i| VOC_LO_HZ * ratio.powi(i as i32))
        .collect()
}

/// The Q that makes the bank **tile** the spectrum instead of leaving holes or piling up.
///
/// Derived from the spacing, not chosen: with centres a factor `r` apart, a band's edges sit
/// at `f/√r` and `f·√r`, so its width is `f(√r − 1/√r)` and `Q = f / width` is the reciprocal
/// of that — the same number for every band, which is what log spacing buys.
pub(super) fn bank_q(bands: usize) -> f32 {
    let ratio = (VOC_HI_HZ / VOC_LO_HZ).powf(1.0 / (bands - 1).max(1) as f32);
    let root = ratio.sqrt();
    1.0 / (root - 1.0 / root).max(1e-3)
}

/// One period of a band-limited sawtooth, by additive synthesis: every harmonic up to
/// Nyquist, and not one more. Returned at unit peak.
///
/// The period is `round(sr / hz)` **samples**, so the table loops exactly — no interpolation,
/// no phase drift over a long clip. The price is that the carrier's pitch quantises to the
/// nearest whole period; at 48 kHz that is a couple of cents, and a robot has no melody to
/// put out of tune.
fn saw_table(sr: f32, hz: f32) -> Vec<f32> {
    let len = (sr / hz.max(1.0)).round().max(4.0) as usize;
    let harmonics = len / 2; // Nyquist, exactly: harmonic k has k cycles per period.
    let mut t: Vec<f32> = (0..len)
        .map(|n| {
            let phase = std::f32::consts::TAU * n as f32 / len as f32;
            (1..=harmonics)
                .map(|k| (phase * k as f32).sin() / k as f32)
                .sum()
        })
        .collect();
    let peak = t.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    if peak > f32::EPSILON {
        for s in &mut t {
            *s /= peak;
        }
    }
    t
}

/// Deterministic white noise (splitmix64) — the *unvoiced* carrier, and therefore the whisper.
/// Deterministic because a rendered effect that differs run to run cannot be finger-printed.
fn noise(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // Top 24 bits to a float in [-1, 1).
    ((z >> 40) as f32 / (1u32 << 23) as f32) - 1.0
}

/// See [`Effect::Vocoder`](super::Effect::Vocoder).
pub(super) fn vocoder(
    data: &SampleData,
    carrier_hz: f32,
    bands: u32,
    breath: f32,
    mix: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let n = (bands.clamp(VOC_MIN_BANDS, VOC_MAX_BANDS)) as usize;
    let breath = breath.clamp(0.0, 1.0);
    let mix = mix.clamp(0.0, 1.0);

    let q = bank_q(n);
    let coeffs: Vec<BiquadCoeffs> = centres(n)
        .into_iter()
        .map(|f| BiquadCoeffs::bandpass(sr, f, q))
        .collect();

    // The carrier is MONO and shared by both channels: a per-channel carrier would decorrelate
    // the robot and smear the image the voice arrived with.
    let table = saw_table(sr, carrier_hz);
    let mut seed = 0x5EED_5EED_5EED_5EEDu64;
    let carrier: Vec<f32> = (0..frames)
        .map(|f| {
            let saw = table[f % table.len()];
            let hiss = noise(&mut seed);
            (1.0 - breath) * saw + breath * hiss
        })
        .collect();

    let atk = time_coeff(ENV_ATTACK_S, sr);
    let rel = time_coeff(ENV_RELEASE_S, sr);
    let env_coeff = time_coeff(ENV_RELEASE_S, sr);

    let mut car_filt: Vec<Biquad> = coeffs.iter().map(|&c| Biquad::new(c)).collect();
    let mut car_env = vec![0.0f32; n];
    let mut mod_filt: Vec<Vec<Biquad>> = (0..ch)
        .map(|_| coeffs.iter().map(|&c| Biquad::new(c)).collect())
        .collect();
    let mut mod_env = vec![vec![0.0f32; n]; ch];
    let mut whitened = vec![0.0f32; n];

    let src = data.samples();
    let peak_in = crate::peak(data);
    SampleData::build(src.len(), data.format(), |out| {
        for f in 0..frames {
            // The carrier's bands advance ONCE per frame — they are shared, so running them
            // inside the channel loop would step their filter state twice per sample.
            for b in 0..n {
                let cb = car_filt[b].process(carrier[f]);
                car_env[b] += env_coeff * (cb.abs() - car_env[b]);
                // Whitening: each carrier band at unit level, so the voice's envelope is the
                // ONLY thing shaping the output. Without it a saw's 1/f tilt is baked into
                // every vowel and the top of the vocoder goes dead.
                whitened[b] = cb / (car_env[b] + CARRIER_FLOOR);
            }
            for c in 0..ch {
                let x = src[f * ch + c];
                let mut wet = 0.0;
                for b in 0..n {
                    let mb = mod_filt[c][b].process(x).abs();
                    let e = &mut mod_env[c][b];
                    // Fast up, slow down: a consonant is an attack, and releasing as fast as
                    // it attacks would follow the pitch period and buzz.
                    let k = if mb > *e { atk } else { rel };
                    *e += k * (mb - *e);
                    wet += whitened[b] * *e;
                }
                out[f * ch + c] = wet;
            }
        }

        // The wet signal's level is the product of two unrelated things (the bank's overlap and
        // the carrier's spectrum), so it means nothing on its own. Put it back at the voice's
        // own peak before blending — the same peak-preserving contract the compressor keeps.
        let peak_wet = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        if peak_wet > f32::EPSILON && peak_in > f32::EPSILON {
            let g = peak_in / peak_wet;
            for s in out.iter_mut() {
                *s *= g;
            }
        }
        for (o, d) in out.iter_mut().zip(src) {
            *o = ((1.0 - mix) * d + mix * *o).clamp(-1.0, 1.0);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;
    /// The voice's own pitch. Nothing here is a harmonic of the carrier below.
    const VOICE_F0: f32 = 90.0;
    /// The robot's pitch. 90 and 160 share no harmonic under 1440 Hz, so the two combs are
    /// separable — which is the only reason "whose pitch is this?" is answerable at all.
    const CARRIER_HZ: f32 = 160.0;

    /// Magnitude of one **exact DFT bin** — a Goertzel, in effect.
    ///
    /// The first version of these gates probed with a `Q`=4 band-pass, and every one of them
    /// was measuring the probe: a band that wide is ~180 Hz across, so the voice's 180 Hz
    /// harmonic and the carrier's 160 Hz one land in the SAME band, and broadband noise (the
    /// whisper carrier) reads as though it were pitched. Over a one-second buffer a DFT bin is
    /// 1 Hz wide and the question has an answer.
    fn bin(d: &SampleData, hz: f32) -> f32 {
        let tau = std::f64::consts::TAU;
        let s: Vec<f64> = d.samples().iter().step_by(2).map(|&x| x as f64).collect();
        let (re, im) = s.iter().enumerate().fold((0.0, 0.0), |(re, im), (n, &x)| {
            let p = tau * hz as f64 * n as f64 / SR as f64;
            (re + x * p.cos(), im - x * p.sin())
        });
        ((re * re + im * im).sqrt() / s.len() as f64) as f32
    }

    /// How strongly a signal is pitched at `f0`: the energy standing in its harmonic comb.
    fn comb(d: &SampleData, f0: f32) -> f32 {
        (1..=6).map(|k| bin(d, f0 * k as f32)).sum()
    }

    /// A **source-filter** voice: a glottal buzz (rich in harmonics, pitched at [`VOICE_F0`])
    /// through ONE resonance. That is what a vowel is, and it is what the vocoder's bank is
    /// supposed to be able to read off.
    ///
    /// The first fixture added sine tones at both formants and only swapped their amplitudes,
    /// so both "vowels" contained both formants and the contrast in the source was 1.4x. A
    /// fixture that barely holds the property cannot prove the code holds it.
    fn voice(formant_hz: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        let n = SR as usize;
        let mut f = Biquad::new(BiquadCoeffs::peak(SR as f32, formant_hz, 2.0, 24.0));
        let raw: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / SR as f32;
                let buzz: f32 = (1..=60)
                    .map(|k| (tau * VOICE_F0 * k as f32 * t).sin() / k as f32)
                    .sum();
                f.process(0.25 * buzz)
            })
            .collect();
        SampleData::from_interleaved(
            raw.iter().flat_map(|&s| [s, s]).collect(),
            AudioFormat::stereo(SR),
        )
    }

    /// **The vocoder takes its pitch from the CARRIER.** The voice's own excitation is thrown
    /// away and only its envelope is kept, so the 90 Hz comb the voice arrived with must not
    /// survive.
    #[test]
    fn the_pitch_comes_from_the_carrier_not_the_voice() {
        let d = voice(700.0);
        let out = vocoder(&d, CARRIER_HZ, 16, 0.0, 1.0);
        let (voice_pitch, carrier_pitch) = (comb(&out, VOICE_F0), comb(&out, CARRIER_HZ));
        println!("pitch: voice comb {voice_pitch:.5}, carrier comb {carrier_pitch:.5}");
        assert!(
            carrier_pitch > voice_pitch * 3.0,
            "the output is still pitched at the voice ({voice_pitch:.5}) rather than the \
             carrier ({carrier_pitch:.5}) -- the excitation is leaking through"
        );
    }

    /// ...and **the formants from the VOICE.** Without this, a vocoder wired to the wrong
    /// envelopes is just a synth playing a note -- and it would sail through the gate above.
    #[test]
    fn the_formants_come_from_the_voice_not_the_carrier() {
        let low = vocoder(&voice(700.0), CARRIER_HZ, 20, 0.0, 1.0);
        let high = vocoder(&voice(2_400.0), CARRIER_HZ, 20, 0.0, 1.0);
        // Same carrier, different vowel: the output's energy has to move with the vowel.
        let a = bin(&low, 640.0) + bin(&low, 800.0); // carrier harmonics 4 and 5, around 700
        let b = bin(&high, 640.0) + bin(&high, 800.0);
        println!("formants: near 700 -- vowel@700 {a:.5}, vowel@2400 {b:.5}");
        // The bar sits BETWEEN the two measurements, not next to one of them: a working bank
        // measures 3.5x (0.111 vs 0.031) and a bank that lost the voice's envelope would sit at
        // ~1x, since the same carrier would come out of both. 2.0 has margin on both sides; 3.0
        // was 17% from the truth, which is a flaky CI waiting for a transcendental to differ.
        assert!(
            a > b * 2.0,
            "moving the voice's formant did not move the output's ({a:.5} vs {b:.5}) -- the \
             bands are not carrying the voice's envelope"
        );
    }

    /// **Breath is the whisper knob**, and that is why Whisper is not a third effect: a noise
    /// carrier has no comb at all, so the same voice comes out unpitched -- which is what
    /// whispering IS (unvoiced excitation through a vocal tract).
    #[test]
    fn a_noise_carrier_has_no_pitch_left() {
        let d = voice(700.0);
        let robot = vocoder(&d, CARRIER_HZ, 16, 0.0, 1.0);
        let whisper = vocoder(&d, CARRIER_HZ, 16, 1.0, 1.0);
        let (r, w) = (comb(&robot, CARRIER_HZ), comb(&whisper, CARRIER_HZ));
        println!("breath: saw comb {r:.5}, noise comb {w:.5}");
        assert!(
            w < r * 0.25,
            "the noise carrier is still pitched ({w:.5} vs the saw's {r:.5}) -- Breath is not \
             reaching the carrier"
        );
        // ...but the vowel survives, or it is hiss, not a whisper.
        assert!(
            bin(&whisper, 700.0) > bin(&whisper, 4_000.0) * 3.0,
            "the whisper lost the voice's formant -- that is hiss, not speech"
        );
    }

    /// **The carrier is band-limited, and that is load-bearing.**
    ///
    /// The table is additive, so harmonic `k` must come out at exactly `1/k`. Summing past
    /// Nyquist would be the aliasing bug -- and it does not add inharmonic hiss (the period is
    /// a whole number of samples, so a folded harmonic lands back ON a harmonic): it lands
    /// there with a **sign flip**, and the harmonics nearest Nyquist very nearly cancel. So
    /// "1/k, all the way up" is the property, and it is exactly the one aliasing destroys.
    #[test]
    fn the_carrier_is_band_limited_all_the_way_to_nyquist() {
        let table = saw_table(SR as f32, CARRIER_HZ);
        let len = table.len();
        let tau = std::f64::consts::TAU;
        // Harmonic k's amplitude in a table of exactly one period.
        let amp = |k: usize| -> f64 {
            let (re, im) = table
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(re, im), (n, &x)| {
                    let p = tau * k as f64 * n as f64 / len as f64;
                    (re + x as f64 * p.cos(), im - x as f64 * p.sin())
                });
            2.0 * (re * re + im * im).sqrt() / len as f64
        };
        // Normalised against the fundamental, every harmonic must be 1/k -- including the last
        // one under Nyquist, which is where the fold-down would have cancelled it.
        let fund = amp(1);
        for k in [2usize, 7, 32, len / 2 - 1] {
            let got = amp(k) / fund;
            let want = 1.0 / k as f64;
            assert!(
                (got - want).abs() < want * 0.05,
                "harmonic {k} is at {got:.5} of the fundamental, not 1/{k} = {want:.5} -- the \
                 table is not band-limited"
            );
        }
    }

    /// The bank must TILE the spectrum: Q is the reciprocal of the (log) spacing, so asking
    /// for more bands has to make each one narrower.
    #[test]
    fn more_bands_means_narrower_ones() {
        assert!(
            bank_q(32) > bank_q(8) * 2.0,
            "Q did not follow the band count ({} vs {}) -- the bank either overlaps into mud \
             or leaves holes",
            bank_q(32),
            bank_q(8)
        );
    }
}
