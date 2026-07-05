//! Sample format primitives — the vocabulary the rest of the crate speaks in.
//!
//! Audio is **presentation** (HR-5 exempt, like render/motion), so plain `f32`
//! samples and `f64` frame math are fine; nothing here feeds simulation state.

/// One audio sample. Linear PCM, nominal range `[-1.0, 1.0]`.
pub type Sample = f32;

/// Channel layout of a buffer or device.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelLayout {
    /// Single channel.
    Mono,
    /// Interleaved left/right.
    Stereo,
}

impl ChannelLayout {
    /// Number of interleaved samples per frame.
    pub const fn count(self) -> usize {
        match self {
            ChannelLayout::Mono => 1,
            ChannelLayout::Stereo => 2,
        }
    }
}

/// Sample rate + channel layout of a stream.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AudioFormat {
    /// Frames per second (e.g. 48_000).
    pub sample_rate: u32,
    /// Channel layout.
    pub channels: ChannelLayout,
}

impl AudioFormat {
    /// Build from a rate + layout.
    pub const fn new(sample_rate: u32, channels: ChannelLayout) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    /// Stereo stream at `sample_rate`.
    pub const fn stereo(sample_rate: u32) -> Self {
        Self::new(sample_rate, ChannelLayout::Stereo)
    }

    /// Mono stream at `sample_rate`.
    pub const fn mono(sample_rate: u32) -> Self {
        Self::new(sample_rate, ChannelLayout::Mono)
    }

    /// Interleaved samples per frame.
    pub const fn channel_count(&self) -> usize {
        self.channels.count()
    }

    /// Seconds → frame count (rounded to nearest frame).
    pub fn secs_to_frames(&self, secs: f64) -> u64 {
        (secs * self.sample_rate as f64).round().max(0.0) as u64
    }

    /// Frame count → seconds.
    pub fn frames_to_secs(&self, frames: u64) -> f64 {
        frames as f64 / self.sample_rate as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_counts() {
        assert_eq!(ChannelLayout::Mono.count(), 1);
        assert_eq!(ChannelLayout::Stereo.count(), 2);
        assert_eq!(AudioFormat::stereo(48_000).channel_count(), 2);
    }

    #[test]
    fn frame_time_round_trips() {
        let fmt = AudioFormat::stereo(48_000);
        assert_eq!(fmt.secs_to_frames(1.0), 48_000);
        assert_eq!(fmt.secs_to_frames(0.5), 24_000);
        assert!((fmt.frames_to_secs(48_000) - 1.0).abs() < 1e-9);
    }
}
