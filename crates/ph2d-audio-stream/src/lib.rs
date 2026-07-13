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
/// `looping` splices the file's start onto its end, endlessly. That is all a whole-buffer looping
/// stream is: the voice never learns the audio ended, so its interpolation across the loop point
/// lands on frame 0 by itself — exactly as a resident clip's does.
///
/// `region` (ADR-0119) makes it an **intro→loop** instead: the producer emits `[0..end)` once and
/// then `[start..end)` for ever. **This is where a stream's region is given** — a streamed voice
/// reads its region from the producer, never from `PlayParams`, because the producer is the only
/// side that knows what the file really contains and can clamp an over-long region to the audio that
/// is actually there.
pub fn play_file(
    path: impl AsRef<Path>,
    looping: bool,
    region: Option<(u64, u64)>,
) -> Result<(StreamHandle, StreamPlayer), StreamError> {
    let path: PathBuf = path.as_ref().to_path_buf();
    let mut src = open(&path)?;
    let rate = src.sample_rate();

    let (feeder, handle) = stream(rate);

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&stop);

    let worker = std::thread::Builder::new()
        .name("ph2d-audio-stream".into())
        .spawn(move || pump(&mut *src, &feeder, looping, region, &flag))
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
    region: Option<(u64, u64)>,
    stop: &std::sync::atomic::AtomicBool,
) {
    use std::sync::atomic::Ordering;

    let ch = src.channels().max(1);
    // Frames decoded but not yet handed over — a packet rarely lands on a chunk boundary.
    let mut carry: Vec<[f32; 2]> = Vec::new();
    let mut ended = false;
    // Source-frame index of the next frame to come out of the decoder. Reset to 0 on every rewind —
    // it is a position in the FILE, not in the stream we are emitting.
    let mut pos = 0u64;
    // Have we been round once? After that, the intro (`[0..start)`) is skipped: an intro is heard
    // exactly once, which is the whole point of a region (ADR-0119 A2).
    let mut looped = false;
    // The effective region, published once we know it. We are always ahead of the audio thread, so
    // it lands before the voice's first wrap needs it — the `Release`/`Acquire` pair in
    // `set_loop_region` makes that ordering real rather than merely likely.
    let mut published = false;
    let region = region.filter(|&(s, e)| s < e);

    while !stop.load(Ordering::Acquire) {
        let Some(mut chunk) = feeder.take_empty() else {
            // The ring is full: the producer is ahead, which is exactly where it belongs. Sleeping
            // here is not laziness, it is the difference between a worker thread and a spin.
            std::thread::sleep(std::time::Duration::from_millis(4));
            continue;
        };

        // Fill the chunk, decoding as much as it takes.
        'fill: while carry.len() < STREAM_CHUNK_FRAMES && !ended {
            match src.next_packet() {
                Some(pcm) => {
                    // Up-mix to stereo exactly as `SampleData::frame_stereo` does — mono into both
                    // channels, >2 channels down to the first two. Up-mixing is linear, which is
                    // what keeps a streamed voice bit-identical to a resident one.
                    for f in pcm.chunks_exact(ch) {
                        // Reached the loop end: back to the top of the file. We rewind and DISCARD
                        // our way to `start` rather than seeking — seeking is per-format and coarse,
                        // and a loop that lands a few frames off is a loop that clicks. Discarding is
                        // exact for every format, and it costs one re-decode of the intro per lap on
                        // a thread that is already far ahead of playback.
                        if looping
                            && let Some((s, e)) = region
                            && pos >= e
                        {
                            if !published {
                                feeder.set_loop_region(s as usize, e as usize);
                                published = true;
                            }
                            if !src.rewind() {
                                ended = true;
                                break 'fill;
                            }
                            pos = 0;
                            looped = true;
                            // The rest of this packet is past the loop end — none of it is wanted.
                            continue 'fill;
                        }
                        // On every lap after the first, the intro is behind us.
                        let intro = looped && region.is_some_and(|(s, _)| pos < s);
                        if !intro {
                            carry.push(match ch {
                                1 => [f[0], f[0]],
                                _ => [f[0], f[1]],
                            });
                        }
                        pos += 1;
                    }
                }
                None => {
                    if !published {
                        // The file ended before the region did (or there is no region): the end of
                        // the first pass IS the turn-around, whatever anyone asked for. A region
                        // whose end runs past the audio is clamped to the audio — a voice that
                        // wrapped on frames that are not there would be worse than one that ignored
                        // the ask (A8).
                        let end = pos;
                        let start = region.map_or(0, |(s, _)| s.min(end.saturating_sub(1)));
                        feeder.set_loop_region(start as usize, end as usize);
                        published = true;
                    }
                    if looping && src.rewind() {
                        pos = 0;
                        looped = true;
                        continue; // splice the loop's start onto its end
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
