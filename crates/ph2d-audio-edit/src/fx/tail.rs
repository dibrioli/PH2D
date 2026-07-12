//! The **tail-extending** effect family (reverb, delay, ping-pong). Split out of
//! `fx.rs` to keep that file under the workspace LOC cap; the length-preserving
//! [`super::Effect`] enum and its neutral points stay there.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Delay, Reverb};

use super::conv;
use super::space::{pingpong, render_wet};

/// A **tail-extending** offline effect: its output rings on after the input stops,
/// so it renders `region + tail` frames and splices via [`crate::in_range_tail`]
/// (never [`crate::in_range`], which would truncate the tail).
///
/// `mix` crossfades dry→wet inside the region (`0` = dry, `1` = fully wet); the
/// tail is pure wet, since the dry signal has ended there.
/// **Not `Copy`, and that is the convolution reverb's doing.** Every other variant here is a
/// handful of floats; a convolution carries a *room* — an impulse response, which is a buffer.
/// The alternative was to hide the IR in ambient state and let `apply` reach for it, which
/// would have broken the rack's central invariant: that an effect is entirely determined by
/// its own value (and therefore that its neutral point is byte-identical, and testable). The
/// `Arc` is the honest cost of modelling what the thing actually is.
#[derive(Debug, Clone, PartialEq)]
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
    /// **Convolution reverb**: put the sound in a room that was *measured*, not modelled.
    ///
    /// The impulse response is what a real space did to a starter pistol. Convolve with it and
    /// the sound is in that space. Neutral at `mix` 0 — and with no IR at all, whatever the mix.
    Convolution {
        /// The room, interleaved. Empty = no room = bypass.
        ir: std::sync::Arc<[f32]>,
        /// Channels in `ir`. Mono = one room for both sides; stereo = the room's own width.
        ir_channels: u8,
        /// The rate `ir` was captured at. A room recorded at 44.1 kHz and dropped straight into
        /// a 48 kHz clip is a room 8 % wrong — resampled at render, so it is the room it was.
        ir_rate: u32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
}

impl TailEffect {
    /// Dry→wet crossfade of any variant.
    fn mix(&self) -> f32 {
        match *self {
            TailEffect::Reverb { mix, .. }
            | TailEffect::Delay { mix, .. }
            | TailEffect::PingPong { mix, .. }
            | TailEffect::Convolution { mix, .. } => mix,
        }
    }

    /// Whether this effect is at its neutral point (fully dry). Then it must NOT
    /// even ring out: appending a silent tail would lengthen the clip for nothing.
    ///
    /// A convolution with **no impulse response** is also bypassed, whatever the Mix says:
    /// there is no room to put the sound in, and convolving with an empty buffer would
    /// silence the clip rather than reverberate it.
    pub fn is_bypass(&self) -> bool {
        if let TailEffect::Convolution { ir, .. } = self
            && ir.is_empty()
        {
            return true;
        }
        self.mix() <= 0.0
    }

    /// How many frames of ring-out this effect needs at `sample_rate`. **Zero when
    /// bypassed**, which is what keeps a fully-dry reverb from growing the clip.
    ///
    /// For a convolution the answer does not come from a knob: **the impulse response IS the
    /// tail**. Its length is how long that room takes to go quiet, and cutting it short would
    /// stop the cathedral rather than let it end.
    pub fn tail_frames(&self, sample_rate: u32) -> usize {
        if self.is_bypass() {
            return 0;
        }
        let secs = match self {
            TailEffect::Reverb { tail_secs, .. }
            | TailEffect::Delay { tail_secs, .. }
            | TailEffect::PingPong { tail_secs, .. } => *tail_secs,
            TailEffect::Convolution {
                ir,
                ir_channels,
                ir_rate,
                ..
            } => {
                // The room's own length, in THIS clip's frames — the IR is resampled into the
                // clip's rate, so a 1 s room is a 1 s tail whatever rate it was captured at.
                let frames = conv::ir_frames(ir, usize::from(*ir_channels));
                let scaled = if *ir_rate == 0 || *ir_rate == sample_rate {
                    frames
                } else {
                    ((frames as f64) * f64::from(sample_rate) / f64::from(*ir_rate)).round()
                        as usize
                };
                return scaled.saturating_sub(1);
            }
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
        if let TailEffect::Convolution {
            ir,
            ir_channels,
            ir_rate,
            mix,
        } = self
        {
            return conv::render_convolution(
                data,
                ir,
                usize::from(*ir_channels),
                *ir_rate,
                *mix,
                tail_frames,
            );
        }
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
            // Handled above (it carries an `Arc`, so it cannot be matched by value).
            TailEffect::Convolution { .. } => data.clone(),
        }
    }
}
