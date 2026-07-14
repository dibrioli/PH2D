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
    ///
    /// **This copies.** `Arc::from(Vec)` cannot reuse the `Vec`'s allocation: an `Arc` stores its
    /// refcount inline, immediately before the data, and a `Vec`'s buffer has no room for one. So
    /// it allocates a second buffer and memcpy's the whole thing — two blocks, twice the peak.
    ///
    /// Prefer [`SampleData::from_fn`] when the samples can be produced by index; it writes each
    /// one exactly once. This constructor stays for callers that genuinely hold a `Vec` already
    /// (decoders hand us one).
    pub fn from_interleaved(samples: Vec<Sample>, format: AudioFormat) -> Self {
        Self::new(samples, format)
    }

    /// Build a buffer by writing each sample **exactly once** — no intermediate `Vec`, no memcpy
    /// (ADR-0117 D2).
    ///
    /// `Arc<[T]>: FromIterator<T>` specializes on `TrustedLen`, and `Map<Range<usize>, F>` is
    /// `TrustedLen`: the `ArcInner` is allocated once, at the right size, and `f` writes straight
    /// into it. One block instead of two, half the peak.
    ///
    /// That specialization is an implementation detail of the standard library, so it is
    /// **measured, not assumed** — `ph2d-audio-edit/tests/measure_arc_build.rs` fails if a future
    /// std stops allocating once, rather than letting the copy creep back in silently.
    pub fn from_fn(len: usize, format: AudioFormat, f: impl FnMut(usize) -> Sample) -> Self {
        Self {
            samples: (0..len).map(f).collect(),
            format,
        }
    }

    /// Allocate `len` samples of silence and let `f` fill them — **one allocation** (ADR-0117 D2).
    ///
    /// The third of the three shapes the rack builds output in, and the one for effects whose
    /// output is a **different length** than their input: a reverb's ring-out, a pitch shift, a
    /// time-stretch. `vec![0.0; n]` followed by [`SampleData::from_interleaved`] allocates the
    /// buffer twice and memcpy's it once for nothing.
    pub fn build(len: usize, format: AudioFormat, f: impl FnOnce(&mut [Sample])) -> Self {
        // `RepeatN` is `TrustedLen` — one allocation, zeroed, no `Vec`.
        let mut samples: Arc<[Sample]> = std::iter::repeat_n(0.0, len).collect();
        // Always `Some`: nothing else can hold an `Arc` created on the line above.
        if let Some(slice) = Arc::get_mut(&mut samples) {
            f(slice);
        }
        Self { samples, format }
    }

    /// The samples, **mutably** — but only if nobody else holds this buffer.
    ///
    /// `SampleData` is an immutable `Arc<[f32]>` on purpose: it is handed to the RT thread, and a
    /// buffer that could change under the mixer would tear. This is the one escape hatch, and it
    /// is safe precisely because it **refuses** when it is not the only owner: `Arc::get_mut`
    /// returns `None` the moment a clone exists (in the mixer, in a snapshot, anywhere).
    ///
    /// What it buys: the editor's audition buffer is re-rendered on every frame of a knob drag,
    /// and a re-render is a whole-clip copy (11.3 ms on a 3-minute clip) of audio that did not
    /// change — the DSP itself touches only the selection. A caller that already holds a buffer
    /// whose out-of-range audio is correct can rewrite just the range, and pay only for it
    /// (ADR-0117 §5, now ADR-0120).
    ///
    /// Callers must have a **full re-render fallback** for the `None` case: the mixer may not
    /// have handed the buffer back yet, and a frame that cannot mutate must still be correct.
    pub fn get_mut(&mut self) -> Option<&mut [Sample]> {
        std::sync::Arc::get_mut(&mut self.samples)
    }

    /// Copy `src` and let `f` rewrite it in place — **one allocation** (ADR-0117 D2).
    ///
    /// The shape most of the rack's effects have: start from the input, walk it with a filter's
    /// state, write over it. Done with a `Vec` that is then handed to [`SampleData::from_interleaved`]
    /// it costs **two** buffers and an extra full memcpy; done here it costs one.
    ///
    /// `f` sees exactly the buffer the `Vec` version saw, in the same order, so an effect ported to
    /// this computes bit-for-bit what it computed before. The container changed; the arithmetic did
    /// not.
    pub fn map_in_place(src: &SampleData, f: impl FnOnce(&mut [Sample])) -> Self {
        // `Copied<slice::Iter>` is `TrustedLen`, so this allocates the `Arc` once, at the right
        // size, and copies straight into it — no `Vec` in between.
        let mut samples: Arc<[Sample]> = src.samples().iter().copied().collect();
        // Always `Some`: the `Arc` was created on the line above and nothing else can hold it yet.
        if let Some(slice) = Arc::get_mut(&mut samples) {
            f(slice);
        }
        Self {
            samples,
            format: src.format(),
        }
    }

    /// Mutable access to the samples **iff nothing else holds them**.
    ///
    /// `SampleData` is an `Arc<[Sample]>` precisely so that handing a clip to a voice on the audio
    /// thread is a refcount bump. The flip side is that the editor cannot scribble on a buffer a
    /// voice might be reading. So: `Some` when this is the sole owner (the ordinary editing case —
    /// nothing is playing it), `None` when it is shared, and the caller copies.
    ///
    /// That is copy-on-write, and it is not a fallback — it is the correct semantics. The RT
    /// thread never sees a half-written buffer, and the editor never pays for a copy it did not
    /// need.
    pub fn samples_mut(&mut self) -> Option<&mut [Sample]> {
        Arc::get_mut(&mut self.samples)
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
