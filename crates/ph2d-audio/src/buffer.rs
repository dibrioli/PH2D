//! In-memory PCM ([`SampleData`]) and the per-block mix scratch ([`MixScratch`]).

use std::sync::Arc;

use crate::format::{AudioFormat, ChannelLayout, Sample};

/// Immutable decoded PCM, interleaved, cheaply shareable across threads.
///
/// The `Arc` makes handing a sample to a voice (on the audio thread) a refcount
/// bump — no allocation. The last drop frees, so a finished voice's `SampleData`
/// is shipped back to the control thread to be dropped off the RT thread (HR-3).
#[derive(Clone)]
pub struct SampleData {
    samples: Arc<[Sample]>,
    format: AudioFormat,
}

impl SampleData {
    /// Wrap interleaved PCM.
    pub fn new(samples: impl Into<Arc<[Sample]>>, format: AudioFormat) -> Self {
        Self {
            samples: samples.into(),
            format,
        }
    }

    /// Wrap an owned interleaved `Vec`.
    pub fn from_interleaved(samples: Vec<Sample>, format: AudioFormat) -> Self {
        Self::new(samples, format)
    }

    /// Raw interleaved samples.
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// This buffer's format.
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Number of frames (interleaved samples / channels).
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.format.channel_count()
    }

    /// True when there are no frames.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The frame at index `frame`, up-mixed to stereo (mono → both channels).
    /// Out-of-range indices read as silence, so a caller need not bounds-check.
    #[inline]
    pub fn frame_stereo(&self, frame: usize) -> [Sample; 2] {
        match self.format.channels {
            ChannelLayout::Mono => {
                let s = self.samples.get(frame).copied().unwrap_or(0.0);
                [s, s]
            }
            ChannelLayout::Stereo => {
                let base = frame * 2;
                let l = self.samples.get(base).copied().unwrap_or(0.0);
                let r = self.samples.get(base + 1).copied().unwrap_or(0.0);
                [l, r]
            }
        }
    }
}

impl std::fmt::Debug for SampleData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleData")
            .field("frames", &self.frame_count())
            .field("format", &self.format)
            .finish()
    }
}

/// Pre-allocated per-block accumulation buffers, interleaved stereo.
///
/// Four reused `Vec`s: the `master` mix, one `bus` scratch the mixer reuses for
/// **each** sub-bus in turn (render → apply the bus fader → fold into master),
/// and two parallel effect-return input buses — `send` (reverb) and `delay_send`
/// — that each accumulate their per-bus aux-send across the block. N sub-buses
/// cost three extra buffers, not 3N.
///
/// HR-3: the mixer `reset`s (clear + zero-fill) and refills the same `Vec`s
/// every block. Once warm (block size stable), `reset` reuses capacity and
/// never reallocates — asserted by the `no_alloc_render` capacity gate.
pub(crate) struct MixScratch {
    master: Vec<Sample>,
    bus: Vec<Sample>,
    send: Vec<Sample>,
    delay_send: Vec<Sample>,
}

impl MixScratch {
    pub(crate) fn new() -> Self {
        Self {
            master: Vec::new(),
            bus: Vec::new(),
            send: Vec::new(),
            delay_send: Vec::new(),
        }
    }

    /// Zero all buffers to hold `stereo_len` interleaved samples. Reallocs only
    /// when the block grows past the warm capacity.
    pub(crate) fn reset(&mut self, stereo_len: usize) {
        self.master.clear();
        self.master.resize(stereo_len, 0.0);
        self.bus.clear();
        self.bus.resize(stereo_len, 0.0);
        self.send.clear();
        self.send.resize(stereo_len, 0.0);
        self.delay_send.clear();
        self.delay_send.resize(stereo_len, 0.0);
    }

    /// The master mix, the per-bus scratch, the reverb-send bus, and the
    /// delay-send bus — all mutable at once so the mixer folds each bus into
    /// master while accumulating both effect sends.
    pub(crate) fn split_mut(
        &mut self,
    ) -> (&mut [Sample], &mut [Sample], &mut [Sample], &mut [Sample]) {
        (
            &mut self.master,
            &mut self.bus,
            &mut self.send,
            &mut self.delay_send,
        )
    }

    /// Allocated capacity of the master buffer (for the HR-3 capacity gate).
    pub(crate) fn capacity(&self) -> usize {
        self.master.capacity()
    }

    /// Allocated capacity of the per-bus scratch (HR-3 capacity gate).
    pub(crate) fn bus_capacity(&self) -> usize {
        self.bus.capacity()
    }

    /// Allocated capacity of the reverb-send bus (HR-3 capacity gate).
    pub(crate) fn send_capacity(&self) -> usize {
        self.send.capacity()
    }

    /// Allocated capacity of the delay-send bus (HR-3 capacity gate).
    pub(crate) fn delay_send_capacity(&self) -> usize {
        self.delay_send.capacity()
    }
}

impl Default for MixScratch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_upmixes_to_both_channels() {
        let d = SampleData::from_interleaved(vec![0.25, -0.5], AudioFormat::mono(48_000));
        assert_eq!(d.frame_count(), 2);
        assert_eq!(d.frame_stereo(0), [0.25, 0.25]);
        assert_eq!(d.frame_stereo(1), [-0.5, -0.5]);
        // out of range → silence
        assert_eq!(d.frame_stereo(99), [0.0, 0.0]);
    }

    #[test]
    fn stereo_reads_interleaved() {
        let d = SampleData::from_interleaved(vec![0.1, 0.2, 0.3, 0.4], AudioFormat::stereo(48_000));
        assert_eq!(d.frame_count(), 2);
        assert_eq!(d.frame_stereo(0), [0.1, 0.2]);
        assert_eq!(d.frame_stereo(1), [0.3, 0.4]);
    }

    #[test]
    fn scratch_reuses_capacity_when_warm() {
        let mut s = MixScratch::new();
        s.reset(1024);
        let cap = s.capacity();
        assert!(cap >= 1024);
        for _ in 0..50 {
            s.reset(1024);
        }
        assert_eq!(s.capacity(), cap, "warm reset must not reallocate");
        let (master, bus, send, delay_send) = s.split_mut();
        assert!(master.iter().all(|&x| x.abs() < f32::EPSILON));
        assert!(bus.iter().all(|&x| x.abs() < f32::EPSILON));
        assert!(send.iter().all(|&x| x.abs() < f32::EPSILON));
        assert!(delay_send.iter().all(|&x| x.abs() < f32::EPSILON));
    }
}
