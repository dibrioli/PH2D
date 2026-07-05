#![forbid(unsafe_code)]
//! `ph2d-audio` — the real-time audio subsystem (SKILL §11.6).
//!
//! A **platform-agnostic** (HR-1) mixer: it turns playing samples into stereo
//! output blocks. The shell owns the device backend (`cpal`) and calls
//! [`AudioRenderer::render`] from its audio callback — `cpal` never enters this
//! crate.
//!
//! ## Two threads, two halves
//!
//! [`AudioEngine::new`] returns a `(control, audio)` pair joined by two
//! lock-free, bounded, zero-alloc rings:
//!
//! ```text
//!  control thread (game)                 audio thread (shell's cpal callback)
//!  ┌────────────────────┐   command ring  ┌──────────────────────────────────┐
//!  │ AudioEngine        │ ───────────────▶ │ AudioRenderer::render(out,frames) │
//!  │  play/stop/set_*   │                  │  drain cmds → mix voices → master │
//!  │  allocates here    │ ◀─────────────── │  no alloc, no free (HR-3)         │
//!  └────────────────────┘   return ring    └──────────────────────────────────┘
//! ```
//!
//! The control side allocates and hands back opaque [`VoiceId`]s (HR-8); the
//! audio side owns the [`MAX_VOICES`]-voice pool (oldest-quietest stealing) and
//! never allocates or frees — finished samples ride the return ring back to be
//! dropped off the RT thread. Audio is presentation (HR-5 exempt).
//!
//! ## Scope (fundamentals)
//!
//! Sample playback (pitch/resample, pan, gain, ADSR) → master bus. The Sound
//! **node domain** (authoring via `ph2d-eval-sound`, like `ph2d-eval-motion`),
//! asset decoding, the Luau/MCP API and the editor mixer UI are the features
//! phase; they build on this core.

mod buffer;
mod command;
mod engine;
mod format;
mod mixer;
mod pool;
mod voice;

pub mod dsp;

pub use buffer::SampleData;
pub use command::PlayParams;
pub use engine::{AudioEngine, AudioRenderer};
pub use format::{AudioFormat, ChannelLayout, Sample};
pub use voice::VoiceId;

use ph2d_core::MemoryBudget;

/// Maximum simultaneous voices (SKILL §11.6: 64-voice pool, oldest-quietest
/// stealing).
pub const MAX_VOICES: usize = 64;
/// Slots in the control→audio command ring.
pub const CMD_CAPACITY: usize = 1024;
/// Slots in the audio→control return ring (finished-sample drops).
pub const RETURN_CAPACITY: usize = 1024;

/// Errors surfaced by the control-side [`AudioEngine`] API.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AudioError {
    /// The command ring was full; the command (and its sample) was dropped
    /// rather than block the caller. Carries the ring capacity.
    #[error("audio command queue is full ({0} slots); command dropped")]
    QueueFull(usize),
}

/// This subsystem's memory budget (HR-13), consistent with SKILL §12.1's 30 MB
/// "Audio buffers" envelope. The fixed engine structures (voice pool + scratch +
/// rings) are well under 1 MB; the rest is headroom the decoded-sample cache
/// grows into in the features phase.
pub fn budget() -> MemoryBudget {
    MemoryBudget::new(0, 30, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_is_within_the_skill_envelope() {
        let b = budget();
        assert_eq!(b.vram_mb, 0);
        assert_eq!(b.heap_script_mb, 0);
        assert!(
            b.ram_mb <= 30,
            "audio RAM budget must stay within SKILL §12.1"
        );
    }
}
