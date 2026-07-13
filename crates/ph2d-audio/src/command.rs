//! The control↔audio message protocol and the lock-free rings that carry it.
//!
//! Two [`ArrayQueue`]-backed rings cross the thread boundary, both bounded and
//! zero-alloc on push/pop (HR-3):
//! - **command ring** — control thread → audio thread ("play this", "stop that").
//! - **return ring** — audio thread → control thread, carrying finished
//!   [`SampleData`] so its `Arc` is *dropped off* the RT thread (freeing on the
//!   audio thread could block the allocator).

use std::sync::Arc;

use crossbeam_queue::ArrayQueue;

use crate::buffer::SampleData;
use crate::bus::BusId;
use crate::dsp::{AdsrParams, BiquadCoeffs};
use crate::stream::StreamHandle;
use crate::voice::Source;
use crate::voice::VoiceId;

/// A **loop region** in source frames, half-open `start..end` (ADR-0119).
///
/// This is the structure of essentially every piece of game music: an intro that plays once, then a
/// body that repeats. There is no separate "intro" feature — the intro is simply whatever lies
/// before `start`:
///
/// ```text
///   play  [0 .. end)      once
///   then  [start .. end)  forever
/// ```
///
/// The runtime **jumps** at `end`; it does not crossfade (that would need a second read head, and on
/// a stream it would need audio the producer has already discarded). The asset is authored to loop
/// cleanly instead — which is what every loop-point format, `smpl` included, assumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRegion {
    /// First frame of the repeating body.
    pub start: u64,
    /// One past the last repeating frame.
    pub end: u64,
}

impl LoopRegion {
    /// The region, or `None` if it could not name a loop in a source of `frames` frames.
    ///
    /// A degenerate region is **refused, not obeyed** (ADR-0119 A8): an empty or inverted one, or
    /// one whose end runs past the source, is not a loop — and a voice that tried to honour it would
    /// wrap on audio that is not there. Falling back to *no region* means the voice loops the whole
    /// buffer, which is wrong but bounded; hanging on a zero-length region is neither.
    pub fn clamped(self, frames: u64) -> Option<Self> {
        let end = self.end.min(frames);
        (self.start < end).then_some(Self {
            start: self.start,
            end,
        })
    }
}

/// How a voice should play, passed to [`crate::AudioEngine::play`].
#[derive(Clone, Debug)]
pub struct PlayParams {
    /// Linear gain applied to the voice (`1.0` = unity).
    pub gain: f32,
    /// Stereo position, `-1.0` (left) … `1.0` (right).
    pub pan: f32,
    /// Playback-rate multiplier (`1.0` = original pitch; `2.0` = one octave up).
    pub pitch: f32,
    /// Loop the sample instead of stopping at its end.
    pub looping: bool,
    /// **Where** to loop, when `looping` (ADR-0119). `None` = the whole buffer, which is what
    /// `looping` has always meant and what it must go on meaning **byte-identically** (A1).
    ///
    /// Ignored when `looping` is false: a loop region on a one-shot names nothing.
    pub loop_region: Option<LoopRegion>,
    /// Optional amplitude envelope; `None` plays at flat gain until the sample ends.
    pub envelope: Option<AdsrParams>,
    /// Which mixer bus this voice sums into (default [`BusId::Master`]).
    pub bus: BusId,
}

impl Default for PlayParams {
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            pitch: 1.0,
            looping: false,
            loop_region: None,
            envelope: None,
            bus: BusId::Master,
        }
    }
}

/// A message from the control thread to the audio thread.
pub(crate) enum AudioCommand {
    Play {
        voice: VoiceId,
        data: SampleData,
        params: PlayParams,
    },
    /// Play from a **stream** instead of a resident clip (ADR-0118). Boxed: the command ring is a
    /// fixed-size array of `AudioCommand`, and every variant pays for the largest — a fat handle
    /// inline would make every `Stop` carry its weight.
    PlayStream {
        voice: VoiceId,
        handle: Box<StreamHandle>,
        params: PlayParams,
    },
    Stop {
        voice: VoiceId,
    },
    Release {
        voice: VoiceId,
    },
    SetVoiceGain {
        voice: VoiceId,
        gain: f32,
    },
    SetVoicePan {
        voice: VoiceId,
        pan: f32,
    },
    SetMasterGain {
        gain: f32,
    },
    /// Master low-pass filter coefficients (computed control-side so no
    /// transcendentals run on the audio thread).
    SetMasterFilter {
        coeffs: BiquadCoeffs,
    },
    /// Set a sub-bus's fader gain (smoothed). Mute is folded in control-side by
    /// sending gain `0.0`, mirroring the master strip.
    SetBusGain {
        bus: BusId,
        gain: f32,
    },
    /// Set a bus's stereo balance, `-1.0` (left) … `1.0` (right); `0.0` = center.
    /// [`BusId::Master`] targets the master balance.
    SetBusPan {
        bus: BusId,
        pan: f32,
    },
    /// Engage/disengage the master soft-clip limiter (tames peaks below the
    /// clip ceiling instead of hard-clipping).
    SetMasterLimiter {
        on: bool,
    },
    /// Set a sub-bus's low-pass filter coefficients (computed control-side so no
    /// transcendentals run on the audio thread). Identity = open (bypass).
    SetBusFilter {
        bus: BusId,
        coeffs: BiquadCoeffs,
    },
    /// Master high-pass (low-cut) filter coefficients (computed control-side).
    /// Identity = off (bypass); in series ahead of the master low-pass.
    SetMasterHighpass {
        coeffs: BiquadCoeffs,
    },
    /// A sub-bus's high-pass (low-cut) filter coefficients (computed
    /// control-side). Identity = off (bypass); in series ahead of that bus's
    /// low-pass.
    SetBusHighpass {
        bus: BusId,
        coeffs: BiquadCoeffs,
    },
    /// Master reverb: enable + return `mix` level (0..1, how much of the wet
    /// return is folded back into the master) + `room_size` (0..1 decay). The
    /// reverb is a parallel return fed by the per-bus [`AudioCommand::SetBusSend`]
    /// aux sends, not a master insert.
    SetReverb {
        on: bool,
        mix: f32,
        room_size: f32,
    },
    /// Set a sub-bus's reverb aux-send `amount` (0..1) — how much of that bus's
    /// post-fader signal is routed into the reverb return.
    SetBusSend {
        bus: BusId,
        amount: f32,
    },
    /// Master 3-band EQ coefficients (low shelf / mid peak / high shelf), computed
    /// control-side. Identity per band = flat (transparent).
    SetMasterEq {
        low: BiquadCoeffs,
        mid: BiquadCoeffs,
        high: BiquadCoeffs,
    },
    /// Master delay/echo return: enable + `time` (s) + `feedback` (0..1) + return
    /// `mix` level (0..1). A parallel return fed by the per-bus delay sends.
    SetDelay {
        on: bool,
        time: f32,
        feedback: f32,
        mix: f32,
    },
    /// Set a sub-bus's delay aux-send `amount` (0..1) — how much of that bus's
    /// post-fader signal feeds the delay return.
    SetBusDelaySend {
        bus: BusId,
        amount: f32,
    },
    /// Set a sub-bus's compressor: `on`, `threshold` (linear 0..1), `ratio`
    /// (>=1), and pre-computed per-sample `attack` / `release` coefficients.
    SetBusCompressor {
        bus: BusId,
        on: bool,
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
    },
    /// Start (or replace) the editor **preview** — a single dedicated voice on
    /// the renderer, outside the game voice pool, whose playback position is
    /// published for the transport playhead. Any previous preview is freed.
    PlayPreview {
        data: SampleData,
        params: PlayParams,
    },
    /// Jump the preview's playback cursor to `frame` (scrub / seek).
    SeekPreview {
        frame: u64,
    },
    /// Hot-swap the preview's sample (an edit applied without stopping playback),
    /// keeping the read cursor. No-op if no preview is sounding.
    SetPreviewData {
        data: SampleData,
    },
    /// Enable/disable preview looping live (the editor's Loop toggle mid-play).
    /// Set (or clear) the preview's loop **region** — the editor moving the loop on a sounding
    /// voice (ADR-0119). Takes effect on the next lap; it does not re-trigger the clip.
    SetPreviewLoopRegion {
        region: Option<LoopRegion>,
    },
    SetPreviewLooping {
        looping: bool,
    },
    /// Pause (`true`) or resume (`false`) the preview without losing its position.
    PausePreview {
        paused: bool,
    },
    /// Stop the preview and free its sample (dropped off the RT thread).
    StopPreview,
}

/// A message from the audio thread back to the control thread.
pub(crate) enum AudioReturn {
    /// A finished/stolen voice's source, to be dropped on the control thread.
    ///
    /// It carries a whole [`Source`] and not just a `SampleData` because a **stream** must come
    /// home too: dropping its chunks on the audio thread would be a `free()` there, which is the
    /// one thing HR-3 forbids (ADR-0118).
    FinishedSource(Source),
}

/// Producer half of a bounded lock-free ring.
pub(crate) struct Producer<T>(Arc<ArrayQueue<T>>);

/// Consumer half of a bounded lock-free ring.
pub(crate) struct Consumer<T>(Arc<ArrayQueue<T>>);

/// Build a ring of the given capacity, returning both ends.
pub(crate) fn ring<T>(capacity: usize) -> (Producer<T>, Consumer<T>) {
    let q = Arc::new(ArrayQueue::new(capacity.max(1)));
    (Producer(Arc::clone(&q)), Consumer(q))
}

impl<T> Producer<T> {
    /// Push a value; returns `Err(value)` if the ring is full (caller decides
    /// whether to drop or retry). Never allocates, never blocks.
    pub(crate) fn push(&self, value: T) -> Result<(), T> {
        self.0.push(value)
    }

    pub(crate) fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<T> Consumer<T> {
    /// Pop the oldest value, or `None` if empty. Never allocates, never blocks.
    pub(crate) fn pop(&self) -> Option<T> {
        self.0.pop()
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_is_fifo_and_bounded() {
        let (tx, rx) = ring::<u32>(2);
        assert_eq!(tx.capacity(), 2);
        assert!(tx.push(1).is_ok());
        assert!(tx.push(2).is_ok());
        // Full → rejected without allocating or blocking.
        assert_eq!(tx.push(3), Err(3));
        assert_eq!(rx.pop(), Some(1));
        assert_eq!(rx.pop(), Some(2));
        assert_eq!(rx.pop(), None);
        assert_eq!(rx.len(), 0);
    }

    #[test]
    fn default_play_params_are_unity() {
        let p = PlayParams::default();
        assert_eq!((p.gain, p.pan, p.pitch), (1.0, 0.0, 1.0));
        assert!(!p.looping && p.envelope.is_none());
    }
}
