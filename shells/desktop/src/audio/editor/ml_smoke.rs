//! Dev smoke for **AI Denoise** (W7, ADR-0123): `PH2D_AUDIO_ML_SMOKE=1` stages a clip that is
//! obviously noisy — a voiced tone buried under broadband hiss at roughly equal energy (0 dB SNR,
//! the level the acceptance experiment used) — so the DeepFilterNet button has something to
//! visibly, audibly clean. The clip opens with **0.75 s of hiss alone** so the W5 `Denoise` gets a
//! fair shot too (see `LEAD_IN_S`).
//!
//! The one thing a human has to check that no gate can: **it sounds better**. Play the staged
//! clip (hiss under a tone), click **AI Denoise (Voice)**, play again — the hiss should fall away
//! and the tone stay. The parity gate already proved the +12 dB against the CLI on real speech;
//! this is the hand on the door.
//!
//! **The honest A/B against the W5.** Select the opening hiss → **Learn Noise** → **Denoise**. The
//! W5 will pull the level down without pulling the tone out of the hiss: at 0 dB SNR broadband
//! there is no bin where the signal dominates, so a Wiener gain lands low *everywhere* and the
//! whole waveform shrinks (the +1.9 dB the acceptance experiment measured, seen as a picture).
//! That is the tool working, not failing — and it is the reason the AI one bought its keep.
//! **Neither replaces the other:** the AI model is speech-trained and removes whatever is not a
//! voice (0% of a game SFX survives it), so the W5 remains the content-agnostic denoise for SFX,
//! ambience and music.
//!
//! Staged unconditionally (it is just audio), but the **AI Denoise (Voice)** button only exists in
//! a build made with `--features audio-ml` — so the smoke is run as
//! `cargo run --features audio-ml` with the env var set. Without the feature the clip still
//! stages, and the W5 (spectral) Denoise is there to compare against.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};

use crate::audio::AudioSystem;
use crate::audio::editor::EditorTransport;

const SR: u32 = 48_000; // DeepFilterNet's rate, so the smoke exercises the no-resample path.

/// A cheap deterministic white-noise source (splitmix64) — a smoke must not be flaky, so no `rand`.
fn hiss(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / 8_388_608.0 - 1.0 // in [-1, 1)
}

/// The clip opens with **hiss alone**, then a voiced tone (150 Hz + harmonics under a slow
/// tremolo) joins it at roughly equal RMS — the 0 dB-SNR case DeepFilterNet is built for.
///
/// **The lead-in is the fixture's whole fairness argument.** The W5 `Denoise` subtracts a profile
/// the user *teaches* it, so it needs a stretch of pure noise to Learn from. Without the lead-in
/// the tone is present in every window a human would select, Learn captures the tone as "noise",
/// and Denoise dutifully removes the tone — the W5 looks broken when it is in fact obeying. A
/// smoke that rigs the A/B teaches the wrong lesson about the tool it is comparing against.
/// (The tremolo dips to zero, but only for an instant — an instant is not a selectable gap.)
///
/// The AI Denoise ignores the lead-in: it learns the noise itself and needs no Learn.
const LEAD_IN_S: f32 = 0.75;

fn noisy_clip() -> SampleData {
    let tau = std::f32::consts::TAU;
    let frames = SR as usize * 4; // 0.75 s of hiss alone, then 3.25 s of tone under hiss
    let mut rng = 0x5EEDu64;
    // Pre-roll the noise so the value at frame 0 is already mixed (splitmix has no warmup, but this
    // keeps the generator's state independent of the interleaving order below).
    SampleData::from_fn(frames, AudioFormat::new(SR, ChannelLayout::Mono), |i| {
        let t = i as f32 / SR as f32;
        let voice: f32 = if t < LEAD_IN_S {
            0.0 // hiss only: the gap the W5's Learn needs
        } else {
            let tv = t - LEAD_IN_S;
            let env = 0.5 + 0.5 * (tau * 2.0 * tv).sin();
            (1..=6)
                .map(|k| (tau * 150.0 * k as f32 * tv).sin() / k as f32)
                .sum::<f32>()
                * env
                * 0.20
        };
        (voice + hiss(&mut rng) * 0.20).clamp(-1.0, 1.0)
    })
}

impl AudioSystem {
    /// Stage the noisy clip for the AI Denoise smoke.
    pub(crate) fn editor_ml_smoke(&mut self) {
        let _ = self.engine.stop_preview();
        self.editor.name = "ai-denoise-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(noisy_clip()));
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);
        println!(
            "audio: AI Denoise smoke staged (PH2D_AUDIO_ML_SMOKE)\n  \
             clip: 0.75 s of hiss alone, then 3.25 s of a voiced tone under that hiss (~0 dB SNR).\n  \
             do:   open the Audio Editor (top bar) -> expand the Spectral section -> Play -> click\n  \
                   AI Denoise (Voice) -> Play again. The hiss should fall away, the tone stay.\n  \
             a/b:  for the W5 Denoise, select the OPENING HISS -> Learn Noise -> Denoise. It pulls\n  \
                   the whole level down rather than lifting the tone out: at 0 dB SNR broadband no\n  \
                   bin favours the signal. That is the +1.9 dB (vs the AI's +12.8) as a picture.\n  \
             note: the two do NOT overlap. The AI model is speech-trained -- it deletes whatever is\n  \
                   not a voice (0% of a game SFX survives), so the W5 stays as the denoise for SFX,\n  \
                   ambience and music. The AI button exists ONLY with --features audio-ml."
        );
    }
}
