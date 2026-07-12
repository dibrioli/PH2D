//! The **tail-extending** effect family (reverb, delay, ping-pong). Split out of
//! `fx.rs` to keep that file under the workspace LOC cap; the length-preserving
//! [`super::Effect`] enum and its neutral points stay there.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Delay, Reverb};

use super::space::{pingpong, render_wet};

/// A **tail-extending** offline effect: its output rings on after the input stops,
/// so it renders `region + tail` frames and splices via [`crate::in_range_tail`]
/// (never [`crate::in_range`], which would truncate the tail).
///
/// `mix` crossfades dry→wet inside the region (`0` = dry, `1` = fully wet); the
/// tail is pure wet, since the dry signal has ended there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TailEffect {
    /// Stereo Freeverb room reverb.
    Reverb {
        /// Decay length (0..1).
        room_size: f32,
        /// High-frequency absorption (0..1).
        damp: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
        /// Ring-out rendered past the region, in seconds.
        tail_secs: f32,
    },
    /// Feedback delay (echo). `time_secs` is clamped to the kit's 1 s line.
    Delay {
        /// Echo tap time (seconds, < 1.0).
        time_secs: f32,
        /// Repeat feedback (0..1, clamped below unity so echoes decay).
        feedback: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
        /// Ring-out rendered past the region, in seconds.
        tail_secs: f32,
    },
    /// **Ping-pong** delay: like [`TailEffect::Delay`], but each repeat bounces to the
    /// opposite channel — a stereo echo walking L→R→L. Neutral at `mix` 0.
    PingPong {
        /// Bounce time (seconds, < 1.0).
        time_secs: f32,
        /// Repeat feedback (0..1, clamped below unity so bounces decay).
        feedback: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
        /// Ring-out rendered past the region, in seconds.
        tail_secs: f32,
    },
}

impl TailEffect {
    /// Dry→wet crossfade of any variant.
    fn mix(&self) -> f32 {
        match *self {
            TailEffect::Reverb { mix, .. }
            | TailEffect::Delay { mix, .. }
            | TailEffect::PingPong { mix, .. } => mix,
        }
    }

    /// Whether this effect is at its neutral point (fully dry). Then it must NOT
    /// even ring out: appending a silent tail would lengthen the clip for nothing.
    pub fn is_bypass(&self) -> bool {
        self.mix() <= 0.0
    }

    /// How many frames of ring-out this effect needs at `sample_rate`. **Zero when
    /// bypassed**, which is what keeps a fully-dry reverb from growing the clip.
    pub fn tail_frames(&self, sample_rate: u32) -> usize {
        if self.is_bypass() {
            return 0;
        }
        let secs = match *self {
            TailEffect::Reverb { tail_secs, .. }
            | TailEffect::Delay { tail_secs, .. }
            | TailEffect::PingPong { tail_secs, .. } => tail_secs,
        };
        (secs.max(0.0) * sample_rate as f32) as usize
    }

    /// Render `data` followed by `tail_frames` of ring-out. Always returns
    /// `data.frame_count() + tail_frames` frames.
    pub fn render(&self, data: &SampleData, tail_frames: usize) -> SampleData {
        if self.is_bypass() {
            return data.clone(); // fully dry: `tail_frames` is 0, so lengths match
        }
        let sr = data.format().sample_rate;
        match *self {
            TailEffect::Reverb {
                room_size,
                damp,
                mix,
                ..
            } => {
                let mut rv = Reverb::new(sr);
                rv.set_params(room_size, damp);
                render_wet(data, tail_frames, mix, move |l, r| rv.process(l, r))
            }
            TailEffect::Delay {
                time_secs,
                feedback,
                mix,
                ..
            } => {
                let mut dl = Delay::new(sr);
                dl.set_params(time_secs, feedback);
                render_wet(data, tail_frames, mix, move |l, r| dl.process(l, r))
            }
            TailEffect::PingPong {
                time_secs,
                feedback,
                mix,
                ..
            } => pingpong(data, tail_frames, time_secs, feedback, mix),
        }
    }
}
