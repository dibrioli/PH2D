//! **Streaming voices** — playing audio without holding all of it (ADR-0118).
//!
//! ## The hole this fills
//!
//! A [`Voice`](crate::voice) used to hold nothing but a [`SampleData`]: the whole clip, decoded to
//! `f32`. One three-minute stereo song is then **65.9 MB resident**, against HR-13's **30 MB** for
//! *all* audio on iPad — 2.2× the entire budget, before a single sound effect
//! (`tests/the_mixer_fits_its_budget.rs`).
//!
//! It also made the codec a lie. Opus is 6.4 % of WAV16 **on disk** and 100 % of it **in RAM**,
//! because the first thing the loader does is expand it back. A codec only means something to
//! memory once the audio does not have to be resident to play — which is this module.
//!
//! ## The contract with the audio thread (HR-3)
//!
//! **The audio thread does not decode, does not allocate, does not free, does not block.** It pops
//! a [`Chunk`] of already-decoded frames from a bounded ring, reads it, and **hands the spent chunk
//! back through a second ring**:
//!
//! ```text
//!   producer  --[ ring: FULL chunks  ]-->  audio thread
//!   producer  <--[ ring: EMPTY chunks ]--  audio thread     (recycling: no free() on the RT side)
//! ```
//!
//! The recycling ring is not an optimisation, it is the whole point. *Dropping* a chunk on the
//! audio thread would call `free()`, and a `free()` is an allocation running backwards. The crate
//! already does exactly this for the `SampleData` of a voice that ends (shipped back to the control
//! thread to be dropped there); streaming is the same move, continuously.
//!
//! Both rings are sized to the chunk count, so **returning a chunk can never fail** — there is
//! always a slot for a chunk that just came out of the other ring. That invariant is what lets the
//! audio thread recycle unconditionally instead of choosing between a leak and a `free()`.
//!
//! ## No codec reaches the mixer
//!
//! This module has **no dependency on any decoder** — the reason `ph2d-audio-decode` has always
//! been a separate crate. `ph2d-audio` owns the rings and the reading; whoever fills the chunks
//! (`ph2d-audio-stream`, on a worker thread) owns the codecs.
//!
//! ## Chunks carry STEREO
//!
//! The producer up-mixes to stereo, exactly as [`SampleData::frame_stereo`] does (mono → both
//! channels). That is not a shortcut: up-mixing is **linear**, so interpolating up-mixed frames and
//! up-mixing interpolated frames are the same arithmetic, in the same order. It is what lets a
//! streamed voice be **bit-identical** to a resident one (ADR-0118 A2).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::command::{Consumer, Producer, ring};
use crate::format::Sample;

/// Frames in one chunk. ~43 ms at 48 kHz: long enough that the ring's atomics cost nothing next to
/// the work, short enough that the producer can be late once without the ring running dry.
pub const STREAM_CHUNK_FRAMES: usize = 2_048;

/// Chunks in flight. Four of them is ~170 ms of lookahead — enough to ride out a scheduler hiccup
/// or a slow read, and 32 KB per voice, not 65.9 MB.
pub const STREAM_DEPTH: usize = 4;

/// A block of decoded, **stereo-interleaved** frames travelling between the producer and the audio
/// thread — and back again, empty, to be refilled.
///
/// The buffer is allocated once, at full capacity, and **never resized**. The audio thread only
/// ever reads it and passes it on.
pub struct Chunk {
    samples: Vec<Sample>,
    frames: usize,
}

impl Chunk {
    fn new() -> Self {
        Self {
            samples: vec![0.0; STREAM_CHUNK_FRAMES * 2],
            frames: 0,
        }
    }

    /// The writable buffer: `STREAM_CHUNK_FRAMES * 2` samples, stereo-interleaved. Producer side.
    pub fn buffer_mut(&mut self) -> &mut [Sample] {
        &mut self.samples
    }

    /// How many frames of [`Chunk::buffer_mut`] the producer actually filled.
    pub fn set_frames(&mut self, frames: usize) {
        self.frames = frames.min(STREAM_CHUNK_FRAMES);
    }

    /// Frames of real audio in this chunk.
    pub fn frames(&self) -> usize {
        self.frames
    }
}

/// What the audio thread got when it asked for the next frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Pull {
    /// A frame of audio.
    Frame([Sample; 2]),
    /// The producer has not caught up. **Not** an error and **not** the end: the voice plays
    /// silence for this frame and carries on. Counted, because an underrun nobody counts is a bug
    /// that only ever happens on the player's machine.
    Underrun,
    /// The source is exhausted and the ring is drained. The voice ends here — exactly where a
    /// resident one would have.
    Ended,
}

/// Shared state both halves can see without locking.
struct Shared {
    /// Set once by the producer when the source has no more audio.
    eof: AtomicBool,
    /// Frames the audio thread wanted and did not get.
    underruns: AtomicU64,
    /// The **effective loop region** the producer is actually emitting, published once it knows
    /// (ADR-0119). `loop_end == 0` = not yet known.
    ///
    /// The producer publishes it, not the caller, because the producer is the only side that knows
    /// what the file really contains — a region whose end runs past the audio is clamped to the
    /// audio, and the voice must wrap where the frames actually stop, not where someone hoped they
    /// would.
    loop_start: AtomicU64,
    loop_end: AtomicU64,
}

/// The **audio-thread** half of a stream: pop frames, recycle chunks. Never allocates, never
/// blocks, never frees.
pub struct StreamHandle {
    full: Consumer<Chunk>,
    empty: Producer<Chunk>,
    shared: Arc<Shared>,
    /// The chunk being read right now, and how far into it we are.
    cur: Option<Chunk>,
    at: usize,
    /// Sample rate of the source — the voice needs it to derive `advance`, exactly as it does from
    /// a resident clip's format.
    rate: u32,
}

impl StreamHandle {
    /// The source's sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.rate
    }

    /// The effective loop region `(start, end)` in source frames, once the producer knows it — what
    /// lets the voice wrap its cursor exactly as a resident one does. A whole-buffer loop publishes
    /// `(0, length)`, which is what looping always meant.
    ///
    /// `end` is the release-acquire flag: reading a non-zero `end` guarantees `start` is visible.
    pub(crate) fn loop_region(&self) -> Option<(usize, usize)> {
        match self.shared.loop_end.load(Ordering::Acquire) {
            0 => None,
            end => Some((
                self.shared.loop_start.load(Ordering::Relaxed) as usize,
                end as usize,
            )),
        }
    }

    /// Frames the audio thread asked for and did not get. Zero is the only good number.
    pub fn underruns(&self) -> u64 {
        self.shared.underruns.load(Ordering::Relaxed)
    }

    /// The next frame of the stream. **The audio thread's only entry point.**
    pub(crate) fn pull(&mut self) -> Pull {
        loop {
            if let Some(c) = &self.cur {
                if self.at < c.frames {
                    let i = self.at * 2;
                    self.at += 1;
                    return Pull::Frame([c.samples[i], c.samples[i + 1]]);
                }
                // Spent. Hand it back to be refilled — never dropped here (that would be a free()
                // on the audio thread). The empty ring is sized to the chunk count, so this cannot
                // fail; if it somehow did, holding the chunk is still better than freeing it.
                if let Some(spent) = self.cur.take() {
                    let _ = self.empty.push(spent);
                }
                self.at = 0;
            }

            match self.full.pop() {
                Some(c) => self.cur = Some(c),
                None => {
                    // Nothing queued. Either the source really is finished, or the producer is
                    // simply late — and those are very different things.
                    return if self.shared.eof.load(Ordering::Acquire) {
                        Pull::Ended
                    } else {
                        self.shared.underruns.fetch_add(1, Ordering::Relaxed);
                        Pull::Underrun
                    };
                }
            }
        }
    }
}

/// The **producer** half: take an empty chunk, fill it with decoded audio, submit it.
///
/// Lives on a worker thread, and is where the codecs are (`ph2d-audio-stream`). Nothing here is
/// real-time constrained — it may allocate, block on a file, and take its time. That is the entire
/// division of labour.
pub struct StreamFeeder {
    full: Producer<Chunk>,
    empty: Consumer<Chunk>,
    shared: Arc<Shared>,
}

impl StreamFeeder {
    /// An empty chunk to fill, if the audio thread has returned one. `None` means the ring is
    /// already as full as it gets — the producer is ahead, which is where it should be.
    pub fn take_empty(&self) -> Option<Chunk> {
        self.empty.pop()
    }

    /// Hand a filled chunk to the audio thread. `Err` gives it back when the ring is full.
    pub fn submit(&self, chunk: Chunk) -> Result<(), Chunk> {
        self.full.push(chunk)
    }

    /// The source has no more audio. The voice ends when the ring drains — not when this is called,
    /// because everything already queued must still play.
    pub fn finish(&self) {
        self.shared.eof.store(true, Ordering::Release);
    }

    /// Tell the voice **where** the loop it is emitting turns around (ADR-0119), so a looping stream
    /// wraps its cursor exactly as a resident clip does (ADR-0118 A2).
    ///
    /// Why it can be published *late* rather than known up front: the producer is always **ahead**
    /// of the audio thread. It reaches the turn-around — and therefore learns it — before the voice
    /// has finished *playing* its way there, and the voice needs the number only at its first wrap,
    /// which comes after. The `Release`/`Acquire` pair makes that ordering real rather than merely
    /// likely.
    ///
    /// That is what lets this be **exact for every format**, instead of only for the containers that
    /// happen to declare a frame count in their header.
    pub fn set_loop_region(&self, start: usize, end: usize) {
        // `start` first, `end` last with Release: a reader that sees a non-zero `end` is guaranteed
        // to see the `start` that goes with it.
        self.shared
            .loop_start
            .store(start as u64, Ordering::Relaxed);
        self.shared.loop_end.store(end as u64, Ordering::Release);
    }

    /// The whole source is the loop — what `looping` has always meant. Publishes `(0, frames)`.
    pub fn set_loop_frames(&self, frames: usize) {
        self.set_loop_region(0, frames);
    }

    /// Whether [`StreamFeeder::finish`] has been called.
    pub fn is_finished(&self) -> bool {
        self.shared.eof.load(Ordering::Acquire)
    }
}

/// Build a stream: the producer half and the audio-thread half.
///
/// A looping stream should also call [`StreamFeeder::set_loop_frames`] once the producer knows the
/// source's length — that is what lets the voice wrap its cursor exactly as a resident one does, and
/// stay bit-identical to it.
///
/// Every chunk is allocated **here**, once, on the calling thread. Neither ring ever creates or
/// destroys one afterwards — they only pass the same [`STREAM_DEPTH`] chunks back and forth.
pub fn stream(rate: u32) -> (StreamFeeder, StreamHandle) {
    let (full_tx, full_rx) = ring::<Chunk>(STREAM_DEPTH);
    let (empty_tx, empty_rx) = ring::<Chunk>(STREAM_DEPTH);
    for _ in 0..STREAM_DEPTH {
        // Cannot fail: the ring's capacity IS the chunk count.
        let _ = empty_tx.push(Chunk::new());
    }
    let shared = Arc::new(Shared {
        eof: AtomicBool::new(false),
        underruns: AtomicU64::new(0),
        loop_start: AtomicU64::new(0),
        loop_end: AtomicU64::new(0),
    });
    (
        StreamFeeder {
            full: full_tx,
            empty: empty_rx,
            shared: Arc::clone(&shared),
        },
        StreamHandle {
            full: full_rx,
            empty: empty_tx,
            shared,
            cur: None,
            at: 0,
            rate: rate.max(1),
        },
    )
}
