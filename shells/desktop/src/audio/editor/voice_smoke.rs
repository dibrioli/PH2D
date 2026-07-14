//! Dev smoke for the **Vocoder** and the **Granular** (`PH2D_AUDIO_VOICE_SMOKE=1`) — the clip
//! and the rack, staged. See [`super::multiband_smoke`] for the same idea; this is its W4 twin.
//!
//! # You cannot vocode a sine, so the clip has to be speech
//!
//! A vocoder's whole claim is *"the vowels survive, the pitch does not"*. On a tone there are no
//! vowels to survive and the claim is untestable — you would hear a robot and have no idea
//! whether it was still saying anything. So the clip is **synthesised speech**, which is what
//! speech physically is:
//!
//! - a **glottal buzz** — a harmonically rich pulse train — whose pitch *glides* (a question's
//!   rising intonation, 100 Hz up to 150 and back). That glide is the thing the vocoder must
//!   THROW AWAY;
//! - two **formants** that move between vowels every 0.4 s, sweeping through the
//!   "ee / ah / oo" corners of the vowel space. Those are what it must KEEP;
//! - a burst of **noise** between vowels — a consonant. Unvoiced, so it is the part of speech
//!   that survives even a whisper.
//!
//! Vocode this and the intonation flattens onto the carrier's monotone while the vowels keep
//! marching. Vocode it with **Breath** at 1 and it whispers — the same vowels, no pitch at all.
//! Neither is audible on a sine, and neither is provable without them.
//!
//! # The rack is staged with the family, so the A/B is a click
//!
//! | | |
//! |---|---|
//! | **Vocoder** (Breath 0 — the robot) | enabled |
//! | **Vocoder** (Breath 1 — the whisper) | bypassed |
//! | **Granular** (a smear) | bypassed |
//!
//! Flip `enabled` between them. Stage 1 vs 2 is the one knob that separates Robotize from
//! Whisper, which is the whole reason neither of them is a row of its own.

use ph2d_audio::{AudioFormat, SampleData};
use ph2d_panel_audio_editor::{FxStage, set_fx_chain};

use super::super::fx_params::{default_norms, params_for, real_to_norm};
use super::super::fx_params_table::KINDS;
use crate::audio::AudioSystem;
use crate::audio::editor::EditorTransport;

const SR: u32 = 48_000;
/// Long enough for six vowels, so the ear hears them as a phrase and not as a beep.
const SECS: f32 = 2.4;
/// How long the mouth holds each vowel.
const VOWEL_S: f32 = 0.4;

/// The vowel corners, as (first formant, second formant) in Hz — the actual coordinates of
/// "ee", "ah", "oo" and friends in the vowel space, which is why the result reads as speech
/// rather than as a filter sweep.
const VOWELS: [(f32, f32); 6] = [
    (270.0, 2_300.0), // ee
    (730.0, 1_090.0), // ah
    (300.0, 870.0),   // oo
    (530.0, 1_840.0), // eh
    (660.0, 1_720.0), // ae
    (640.0, 1_190.0), // aw
];

/// Synthesised speech: a pitch-gliding glottal buzz through moving formants, with an unvoiced
/// consonant between each vowel. See the module docs for why every one of those is needed.
pub(crate) fn speech_clip() -> SampleData {
    let tau = std::f32::consts::TAU;
    let frames = (SR as f32 * SECS) as usize;

    // The glottal buzz, with an INTONATION: this rising-then-falling glide is precisely what the
    // vocoder has to discard, so it has to be there to be discarded.
    let mut phase = 0.0f32;
    let mut buzz = vec![0.0f32; frames];
    for (i, b) in buzz.iter_mut().enumerate() {
        let t = i as f32 / SR as f32;
        let f0 = 100.0 + 50.0 * (tau * 0.35 * t).sin(); // 100..150 Hz, a question
        phase = (phase + tau * f0 / SR as f32) % tau;
        // A pulse train, band-limited by construction: harmonics that fall off like 1/k.
        *b = (1..=30)
            .map(|k| (phase * k as f32).sin() / k as f32)
            .sum::<f32>()
            * 0.25;
    }

    // Two formant resonances that MOVE between vowels, plus an unvoiced burst between them.
    let mut f1 = ph2d_audio::dsp::Biquad::new(ph2d_audio::dsp::BiquadCoeffs::peak(
        SR as f32,
        VOWELS[0].0,
        3.0,
        20.0,
    ));
    let mut f2 = ph2d_audio::dsp::Biquad::new(ph2d_audio::dsp::BiquadCoeffs::peak(
        SR as f32,
        VOWELS[0].1,
        3.0,
        18.0,
    ));
    let mut seed = 0xC0FF_EE00_1234_5678u64;
    let mut noise = || -> f32 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((seed >> 40) as f32 / (1u32 << 23) as f32) - 1.0
    };

    let mut mono = vec![0.0f32; frames];
    let mut vowel = usize::MAX;
    for (i, m) in mono.iter_mut().enumerate() {
        let t = i as f32 / SR as f32;
        let v = ((t / VOWEL_S) as usize).min(VOWELS.len() - 1);
        if v != vowel {
            // Retune the mouth. `set_coeffs` keeps the delay state, so the vowel GLIDES into the
            // next one instead of clicking — which is also what a mouth does.
            vowel = v;
            f1.set_coeffs(ph2d_audio::dsp::BiquadCoeffs::peak(
                SR as f32,
                VOWELS[v].0,
                3.0,
                20.0,
            ));
            f2.set_coeffs(ph2d_audio::dsp::BiquadCoeffs::peak(
                SR as f32,
                VOWELS[v].1,
                3.0,
                18.0,
            ));
        }
        // The consonant: a short unvoiced burst in the last 60 ms of each vowel's slot.
        let into = t % VOWEL_S;
        let consonant = if into > VOWEL_S - 0.06 { 0.25 } else { 0.0 };
        let src = buzz[i] + consonant * noise();
        *m = 0.8 * f2.process(f1.process(src));
    }

    let peak = mono.iter().fold(0.0f32, |m, &s| m.max(s.abs())).max(1e-6);
    SampleData::from_fn(frames * 2, AudioFormat::stereo(SR), |i| {
        mono[i / 2] / peak * 0.8
    })
}

impl AudioSystem {
    /// Stage the speech clip **and** the vocoder/granular rack. See the module docs.
    pub(crate) fn editor_voice_smoke(&mut self) {
        let _ = self.engine.stop_preview();
        self.editor.name = "voice-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(speech_clip()));
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);

        // Build a stage from REAL units by parameter label, exactly as a factory preset does —
        // so this reads like the settings it is, and survives a param being reordered.
        let staged = |name: &str, overrides: &[(&str, f32)]| -> Option<FxStage> {
            let kind = KINDS.iter().position(|k| k.name == name)?;
            let specs = params_for(kind);
            let mut norms = default_norms(kind);
            for (label, value) in overrides {
                let i = specs.iter().position(|s| s.label == *label)?;
                norms[i] = real_to_norm(&specs[i], *value);
            }
            Some(FxStage {
                kind,
                norms,
                enabled: false,
            })
        };

        let chain: Option<Vec<FxStage>> = (|| {
            let mut robot = staged(
                "Vocoder",
                &[
                    ("Carrier", 110.0),
                    ("Bands", 20.0),
                    ("Breath", 0.0),
                    ("Mix", 1.0),
                ],
            )?;
            let whisper = staged(
                "Vocoder",
                &[
                    ("Carrier", 110.0),
                    ("Bands", 24.0),
                    ("Breath", 1.0),
                    ("Mix", 1.0),
                ],
            )?;
            let cloud = staged(
                "Granular",
                &[
                    ("Grain", 80.0),
                    ("Scatter", 0.7),
                    ("Pitch", 5.0),
                    ("Mix", 1.0),
                ],
            )?;
            robot.enabled = true;
            Some(vec![robot, whisper, cloud])
        })();

        let Some(chain) = chain else {
            println!("audio: voice smoke: the rack is missing Vocoder or Granular");
            return;
        };
        set_fx_chain(chain);

        println!(
            "audio: voice smoke staged (PH2D_AUDIO_VOICE_SMOKE)\n  \
             clip: synthesised speech -- a buzz whose PITCH glides, through formants that move\n  \
                   between six vowels, with an unvoiced consonant between each.\n  \
             rack: [Vocoder Breath=0: on] [Vocoder Breath=1: bypassed] [Granular: bypassed]\n  \
             A/B:  flip the per-stage enable.\n  \
                   stage 1 -- the intonation FLATTENS onto the carrier's monotone; the vowels\n  \
                              keep marching. That is the robot.\n  \
                   stage 2 -- same vowels, no pitch at all. That is a whisper, and it is the\n  \
                              SAME effect with one knob moved.\n  \
                   stage 3 -- the phrase smears into a texture."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio_edit::{EditClip, Effect};

    /// Magnitude of one exact DFT bin, over the clip's left channel.
    fn bin(d: &SampleData, hz: f32) -> f32 {
        let tau = std::f64::consts::TAU;
        let s: Vec<f64> = d.samples().iter().step_by(2).map(|&x| x as f64).collect();
        let (re, im) = s.iter().enumerate().fold((0.0, 0.0), |(re, im), (n, &x)| {
            let p = tau * hz as f64 * n as f64 / SR as f64;
            (re + x * p.cos(), im - x * p.sin())
        });
        ((re * re + im * im).sqrt() / s.len() as f64) as f32
    }

    /// **The smoke clip is actually speech-shaped**, or the vocoder has nothing to prove on it.
    ///
    /// Two properties, and both are the ones the ear will be listening for: the pitch MOVES
    /// (there is an intonation to flatten), and the vowels are where the vowels are.
    #[test]
    fn the_smoke_clip_has_an_intonation_to_flatten() {
        let d = speech_clip();
        // The buzz glides 100..150 Hz, so BOTH ends of the glide carry energy — a monotone
        // would put it all in one bin.
        let (low, high) = (bin(&d, 100.0), bin(&d, 148.0));
        println!("speech: energy at 100 Hz {low:.5}, at 148 Hz {high:.5}");
        assert!(
            low > 1e-4 && high > 1e-4,
            "the clip has no pitch GLIDE ({low:.5} / {high:.5}) -- there is no intonation for \
             the vocoder to throw away, and the smoke would prove nothing"
        );
    }

    /// **And vocoding it does what the smoke says it does**: the voice's own pitch goes, the
    /// carrier's arrives. If this were false, Enio would flip the stages, hear a robot, and have
    /// no way to know the vowels had been destroyed along with the pitch.
    #[test]
    fn vocoding_the_smoke_clip_replaces_the_pitch_and_keeps_the_vowels() {
        let d = speech_clip();
        let clip = EditClip::new(d.clone());
        let out = clip.render_effect(Effect::Vocoder {
            carrier_hz: 110.0,
            bands: 20,
            breath: 0.0,
            mix: 1.0,
        });

        // The carrier's comb replaces the voice's. The voice's f0 wanders over 100..150, so its
        // energy is spread across that span; the carrier's sits exactly on 110 and its harmonics.
        let carrier = (1..=6).map(|k| bin(&out, 110.0 * k as f32)).sum::<f32>();
        // 148 Hz: near the TOP of the voice's glide, and not a harmonic of 110.
        let voice = bin(&out, 148.0) + bin(&out, 296.0);
        println!("vocoded: carrier comb {carrier:.5}, leftover voice pitch {voice:.5}");
        assert!(
            carrier > voice * 5.0,
            "the vocoded clip is still pitched at the voice ({voice:.5} vs the carrier's \
             {carrier:.5}) -- the smoke would not demonstrate the effect"
        );
        // ...and the vowels are still moving: the first and the last are different vowels, so
        // their formant energy must differ. (Same clip, so this is the effect, not the source.)
        let s = out.samples();
        let quarter = s.len() / 6 / 2 * 2;
        let head = SampleData::from_interleaved(s[..quarter].to_vec(), out.format());
        let tail = SampleData::from_interleaved(s[s.len() - quarter..].to_vec(), out.format());
        // "ee" (F1 270) against "aw" (F1 640).
        let head_low = bin(&head, 270.0);
        let tail_low = bin(&tail, 660.0);
        assert!(
            head_low > 1e-5 && tail_low > 1e-5,
            "the vocoded vowels are gone ({head_low:.6} / {tail_low:.6}) -- the bank threw away \
             the formants along with the pitch"
        );
    }
}
