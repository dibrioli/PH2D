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
use crate::dsp::AdsrParams;
use crate::voice::VoiceId;

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
    /// Optional amplitude envelope; `None` plays at flat gain until the sample ends.
    pub envelope: Option<AdsrParams>,
}

impl Default for PlayParams {
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            pitch: 1.0,
            looping: false,
            envelope: None,
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
}

/// A message from the audio thread back to the control thread.
pub(crate) enum AudioReturn {
    /// A finished/stolen voice's sample, to be dropped on the control thread.
    FinishedSample(SampleData),
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
