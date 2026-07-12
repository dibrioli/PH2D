#![forbid(unsafe_code)]
//! `ph2d-audio-stream` — **the producer side of a streaming voice** (ADR-0118).
//!
//! A worker thread decodes a file ahead, in chunks, and feeds the lock-free ring the audio thread
//! reads from. This is where the codecs live, and the split is the whole point:
//!
//! - **`ph2d-audio`** is the real-time mixer. It pops already-decoded chunks and hands the spent
//!   ones back. It does not decode, allocate, free, or block — and it has **no codec dependency**,
//!   enforced by `ph2d-audio/tests/no_codec_reaches_the_mixer.rs`.
//! - **here**, on an ordinary thread, we may do all four.
//!
//! ## What it buys
//!
//! A three-minute stereo song costs **65.9 MB** decoded and resident — 2.2× HR-13's *entire* iPad
//! audio budget, before a single sound effect. Streamed, it costs the ring: **0.06 MB**, whatever
//! its length. A forty-minute ambient bed costs the same 0.06 MB.
//!
//! ## Reading from the FILE, not from its bytes
//!
//! The reader opens the file and decodes packets off disk. Holding the file's *bytes* in memory to
//! avoid holding its *samples* would be no saving at all for a WAV, which is barely smaller encoded
//! than decoded — and a hollow one for the rest. Streaming means the audio is never all in memory,
//! not that it is in memory in a different shape.

use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use ph2d_audio::{STREAM_CHUNK_FRAMES, StreamFeeder, StreamHandle, stream};

/// What can go wrong opening a stream.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    /// Neither decoder could open the file.
    #[error("could not open {path} for streaming: {reason}")]
    Open { path: String, reason: String },
}

/// A source of decoded audio, read a packet at a time.
///
/// Two implementations, because Symphonia does not read Opus (which is why `ph2d-audio-opus`
/// exists — see that crate's `decoder.rs`). Both are opened by **content**, never by extension: a
/// renamed file is still what it is.
trait Packets: Send {
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> usize;
    /// The next packet of interleaved `f32`, or `None` at the end of the file.
    fn next_packet(&mut self) -> Option<&[f32]>;
    /// Start over — how a looping stream loops.
    fn rewind(&mut self) -> bool;
}

struct Symphonia(ph2d_audio_decode::Reader);

impl Packets for Symphonia {
    fn sample_rate(&self) -> u32 {
        self.0.sample_rate()
    }
    fn channels(&self) -> usize {
        self.0.channels()
    }
    fn next_packet(&mut self) -> Option<&[f32]> {
        // A decode error mid-file ends the stream rather than killing the process: the voice stops
        // where the file stopped making sense, which is the same thing a corrupt file does to any
        // other player.
        self.0.next_packet().ok().flatten()
    }
    fn rewind(&mut self) -> bool {
        self.0.rewind().is_ok()
    }
}

struct Opus(ph2d_audio_opus::Reader);

impl Packets for Opus {
    fn sample_rate(&self) -> u32 {
        self.0.sample_rate()
    }
    fn channels(&self) -> usize {
        self.0.channels()
    }
    fn next_packet(&mut self) -> Option<&[f32]> {
        self.0.next_packet().ok().flatten()
    }
    fn rewind(&mut self) -> bool {
        self.0.rewind().is_ok()
    }
}

/// Open a file for streaming, routing by **content**.
fn open(path: &Path) -> Result<Box<dyn Packets>, StreamError> {
    let fail = |reason: String| StreamError::Open {
        path: path.display().to_string(),
        reason,
    };
    if ph2d_audio_opus::is_opus_file(path) {
        let r = ph2d_audio_opus::Reader::open(path).map_err(|e| fail(e.to_string()))?;
        return Ok(Box::new(Opus(r)));
    }
    let r = ph2d_audio_decode::Reader::open(path).map_err(|e| fail(e.to_string()))?;
    Ok(Box::new(Symphonia(r)))
}

/// A running stream: the producer thread, and the handle to stop it.
///
/// Dropping this asks the thread to stop and waits for it. The [`StreamHandle`] it produced is
/// handed to `AudioEngine::play_stream` and lives on the audio thread — the two halves never touch
/// each other except through the rings.
pub struct StreamPlayer {
    worker: Option<JoinHandle<()>>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl StreamPlayer {
    /// Ask the producer to stop and wait for it.
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

impl Drop for StreamPlayer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Open `path` and start decoding it into a stream.
///
/// Returns the [`StreamHandle`] to hand to `AudioEngine::play_stream`, and the [`StreamPlayer`]
/// that owns the producer thread. **Keep the player alive for as long as the voice plays** — drop
/// it and the audio stops being fed.
///
/// `looping` splices the file's start onto its end, endlessly. That is all a looping stream is: the
/// voice never learns the audio ended, so its interpolation across the loop point lands on frame 0
/// by itself — exactly as a resident clip's does.
pub fn play_file(
    path: impl AsRef<Path>,
    looping: bool,
) -> Result<(StreamHandle, StreamPlayer), StreamError> {
    let path: PathBuf = path.as_ref().to_path_buf();
    let mut src = open(&path)?;
    let rate = src.sample_rate();

    let (feeder, handle) = stream(rate);

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&stop);

    let worker = std::thread::Builder::new()
        .name("ph2d-audio-stream".into())
        .spawn(move || pump(&mut *src, &feeder, looping, &flag))
        .map_err(|e| StreamError::Open {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

    Ok((
        handle,
        StreamPlayer {
            worker: Some(worker),
            stop,
        },
    ))
}

/// The producer loop: keep the ring full, and go back to sleep.
///
/// Every chunk it fills came *out of the ring* — the audio thread returned it after reading it.
/// Nothing here allocates a chunk, and nothing on the audio thread frees one. They go round.
fn pump(
    src: &mut dyn Packets,
    feeder: &StreamFeeder,
    looping: bool,
    stop: &std::sync::atomic::AtomicBool,
) {
    use std::sync::atomic::Ordering;

    let ch = src.channels().max(1);
    // Frames decoded but not yet handed over — a packet rarely lands on a chunk boundary.
    let mut carry: Vec<[f32; 2]> = Vec::new();
    let mut ended = false;
    // Frames emitted in the FIRST pass. On reaching the end we publish it, and a looping voice can
    // then wrap its cursor exactly as a resident one does (ADR-0118 A2) — no header field needed,
    // and exact for every format. We are always ahead of the audio thread, so the number lands
    // before the voice's first wrap needs it.
    let mut first_pass = 0usize;
    let mut length_known = false;

    while !stop.load(Ordering::Acquire) {
        let Some(mut chunk) = feeder.take_empty() else {
            // The ring is full: the producer is ahead, which is exactly where it belongs. Sleeping
            // here is not laziness, it is the difference between a worker thread and a spin.
            std::thread::sleep(std::time::Duration::from_millis(4));
            continue;
        };

        // Fill the chunk, decoding as much as it takes.
        while carry.len() < STREAM_CHUNK_FRAMES && !ended {
            match src.next_packet() {
                Some(pcm) => {
                    // Up-mix to stereo exactly as `SampleData::frame_stereo` does — mono into both
                    // channels, >2 channels down to the first two. Up-mixing is linear, which is
                    // what keeps a streamed voice bit-identical to a resident one.
                    for f in pcm.chunks_exact(ch) {
                        carry.push(match ch {
                            1 => [f[0], f[0]],
                            _ => [f[0], f[1]],
                        });
                        if !length_known {
                            first_pass += 1;
                        }
                    }
                }
                None => {
                    if !length_known {
                        // The end of the first pass IS the source's length.
                        feeder.set_loop_frames(first_pass);
                        length_known = true;
                    }
                    if looping && src.rewind() {
                        continue; // splice the file's start onto its end
                    }
                    ended = true;
                }
            }
        }

        let n = carry.len().min(STREAM_CHUNK_FRAMES);
        {
            let buf = chunk.buffer_mut();
            for (k, fr) in carry.iter().take(n).enumerate() {
                buf[k * 2] = fr[0];
                buf[k * 2 + 1] = fr[1];
            }
        }
        chunk.set_frames(n);
        carry.drain(..n);

        if feeder.submit(chunk).is_err() {
            // Ring filled between the take and the submit — impossible in a single-producer setup,
            // but if it ever happened, the chunk must not be dropped on the floor.
            break;
        }
        if ended && carry.is_empty() {
            feeder.finish();
            return;
        }
    }
    // Stopped early (the player was dropped): tell the voice, or it waits forever for audio that is
    // never coming.
    feeder.finish();
}
