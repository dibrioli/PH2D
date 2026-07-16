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
//! a build made with `--features audio-ml`. Without the feature the clip still stages, and the W5
//! (spectral) Denoise is there to compare against.
//!
//! **Run it with `--release`, and that is not a preference.** Measured on this machine, the model
//! runs at 0.03× real-time in release (a 4 s clip in 0.16 s — instant) and **16× slower in debug**
//! (the same clip takes 2.7 s). A debug build makes the button feel broken and invites a bug report
//! about a wait that the product does not have:
//!
//! ```text
//! PH2D_AUDIO_ML_SMOKE=1 cargo run --release -p ph2d-host-desktop --features audio-ml
//! ```
//!
//! ## Seeing the progress bar
//!
//! The default 4 s clip denoises in ~0.16 s, so **the bar flashes past** — which is the product
//! working, and is also why it cannot be checked here. `PH2D_AUDIO_ML_SMOKE_SECS` stages a clip of
//! any length, and the interesting one is the case the bar was built for: a 3-minute take, ~5 s of
//! inference.
//!
//! ```text
//! PH2D_AUDIO_ML_SMOKE=1 PH2D_AUDIO_ML_SMOKE_SECS=180 cargo run --release -p ph2d-host-desktop --features audio-ml
//! ```
//!
//! While it runs: the window still redraws, the bar climbs, the percentage moves, the Spectral
//! section is dimmed with a reason on its status line — and when it lands the hiss is gone. An
//! indicator nobody can observe has not been verified, so this knob is part of the feature, not a
//! convenience.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};

use crate::audio::AudioSystem;
use crate::audio::editor::EditorTransport;

const SR: u32 = 48_000; // DeepFilterNet's rate, so the smoke exercises the no-resample path.

/// Clip length when `PH2D_AUDIO_ML_SMOKE_SECS` says nothing. Long enough to hear the model work,
/// short enough that the button feels instant — which is the honest default, since that IS how it
/// feels on the material this feature was built for.
const DEFAULT_SECS: f32 = 4.0;

/// How long a clip to stage, in seconds (`PH2D_AUDIO_ML_SMOKE_SECS`).
///
/// Clamped to something a smoke can survive: 0 s is not a clip, and past 10 minutes the staging
/// itself (a few million transcendentals) starts to be the thing you are waiting for.
fn clip_secs() -> f32 {
    std::env::var("PH2D_AUDIO_ML_SMOKE_SECS")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.5, 600.0))
        .unwrap_or(DEFAULT_SECS)
}

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

fn noisy_clip(secs: f32) -> SampleData {
    let tau = std::f32::consts::TAU;
    // 0.75 s of hiss alone, then tone under hiss for the rest.
    let frames = (SR as f32 * secs) as usize;
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
        let secs = clip_secs();
        self.editor.name = "ai-denoise-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(noisy_clip(secs)));
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);
        // 0.03x real-time, measured in release (ADR-0123 / the W7 closure handoff).
        let est = secs * 0.03; // LITERAL-PX-OK: measured release-build speed ratio, not a dimension
        println!(
            "audio: AI Denoise smoke staged (PH2D_AUDIO_ML_SMOKE)\n  \
             clip: {secs:.2} s -- 0.75 s of hiss alone, then a voiced tone under that hiss (~0 dB SNR).\n  \
             do:   open the Audio Editor (top bar) -> expand the Spectral section -> Play -> click\n  \
                   AI Denoise (Voice) -> Play again. The hiss should fall away, the tone stay.\n  \
             a/b:  for the W5 Denoise, select the OPENING HISS -> Learn Noise -> Denoise. It pulls\n  \
                   the whole level down rather than lifting the tone out: at 0 dB SNR broadband no\n  \
                   bin favours the signal. That is the +1.9 dB (vs the AI's +12.8) as a picture.\n  \
             note: the two do NOT overlap. The AI model is speech-trained -- it deletes whatever is\n  \
                   not a voice (0% of a game SFX survives), so the W5 stays as the denoise for SFX,\n  \
                   ambience and music. The AI button exists ONLY with --features audio-ml.\n  \
             bar:  the denoise runs OFF the UI thread with a progress bar at the top-centre. This\n  \
                   clip should take ~{est:.2} s in release, so the bar {bar}.\n  \
                   For the case it was built for, stage a real take: PH2D_AUDIO_ML_SMOKE_SECS=180\n  \
                   (~5 s of work) -- the window keeps redrawing, the bar climbs, and the Spectral\n  \
                   section is dimmed until it lands.\n  \
             speed: RELEASE builds run the model at 0.03x real-time. A DEBUG build is ~16x slower\n  \
                   and makes the button feel broken -- use --release.",
            bar = if est < 1.0 {
                "will flash past (that is the product being fast, not the bar being broken)"
            } else {
                "should be plainly visible"
            }
        );
    }
}
