//! Dev smoke for **AI Denoise** (W7, ADR-0123): `PH2D_AUDIO_ML_SMOKE=1` stages a clip that is
//! obviously noisy — a voiced tone buried under broadband hiss at roughly equal energy (0 dB SNR,
//! the level the acceptance experiment used) — so the DeepFilterNet button has something to
//! visibly, audibly clean.
//!
//! The one thing a human has to check that no gate can: **it sounds better**. Play the staged
//! clip (hiss under a tone), click **AI Denoise**, play again — the hiss should fall away and the
//! tone stay. The parity gate already proved the +12 dB against the CLI on real speech; this is
//! the hand on the door.
//!
//! Staged unconditionally (it is just audio), but the **AI Denoise** button only exists in a build
//! made with `--features audio-ml` — so the smoke is run as
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

/// 3 s of a voiced tone (150 Hz + harmonics under a slow tremolo) plus broadband hiss at roughly
/// equal RMS — the 0 dB-SNR case DeepFilterNet is built for and the W5 struggles with.
fn noisy_clip() -> SampleData {
    let tau = std::f32::consts::TAU;
    let frames = SR as usize * 3;
    let mut rng = 0x5EEDu64;
    // Pre-roll the noise so the value at frame 0 is already mixed (splitmix has no warmup, but this
    // keeps the generator's state independent of the interleaving order below).
    SampleData::from_fn(frames, AudioFormat::new(SR, ChannelLayout::Mono), |i| {
        let t = i as f32 / SR as f32;
        let env = 0.5 + 0.5 * (tau * 2.0 * t).sin();
        let voice: f32 = (1..=6)
            .map(|k| (tau * 150.0 * k as f32 * t).sin() / k as f32)
            .sum::<f32>()
            * env
            * 0.20;
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
             clip: 3 s of a voiced tone buried under broadband hiss at ~0 dB SNR.\n  \
             do:   open the Audio Editor (top bar) -> expand the Spectral section -> Play (hiss\n  \
                   under a tone) -> click AI Denoise -> Play again. The hiss should fall away and\n  \
                   the tone stay. (Compare with the W5 Denoise: Learn a silent-ish gap first, then\n  \
                   Denoise -- it removes far less.)\n  \
             note: the AI Denoise button exists ONLY in a build made with --features audio-ml."
        );
    }
}
