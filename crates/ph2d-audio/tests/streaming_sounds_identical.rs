//! **A2 of ADR-0118 — the heart of it.** A streamed voice must render the *same buffer* a resident
//! one renders. Not "close". The same.
//!
//! A streaming path that sounds *nearly* the same is the worst possible outcome: it passes every
//! listening test, ships, and then someone spends a week on "the music sounds a bit different in
//! the build". So the gate is bit-equality, and it covers the three places the two paths could
//! diverge:
//!
//! - **fractional advance** (a 44.1 kHz asset in a 48 kHz mixer): the interpolation straddles two
//!   source frames, and the streamed window has to hold exactly the pair the resident index would.
//! - **the loop point**: the resident path wraps its cursor and interpolates against frame 0; the
//!   streamed one never wraps at all — the producer just keeps feeding, splicing the file's start
//!   onto its end. Those must land on the same samples.
//! - **the end**: a resident voice holds its last frame and stops when the cursor passes the
//!   length. A stream has to discover both from `Ended`.

use ph2d_audio::{
    AudioEngine, AudioFormat, ChannelLayout, PlayParams, SampleData, StreamFeeder, VoiceId, stream,
};

const OUT_RATE: u32 = 48_000;

/// A signal with structure — a ramp would hide an off-by-one that this exposes.
fn clip(frames: usize, rate: u32, channels: ChannelLayout) -> SampleData {
    let ch = if channels == ChannelLayout::Stereo {
        2
    } else {
        1
    };
    SampleData::from_fn(
        frames * ch,
        AudioFormat {
            sample_rate: rate,
            channels,
        },
        |i| {
            let x = (i % 313) as f32 / 313.0;
            let y = ((i / 313) % 7) as f32 / 7.0;
            x * 0.6 - 0.3 + y * 0.1
        },
    )
}

/// A stand-in for the real producer (`ph2d-audio-stream`, which will pull this from a decoder).
///
/// It **remembers where it is**, which is the whole job: a producer that re-sent the file from the
/// top every time it was topped up would be feeding an endless stream, and the voice would dutifully
/// play it forever. (It is also the first bug this test had.)
///
/// `loops` repetitions, then `finish()`. A looping stream is simply one that keeps coming: the
/// producer splices the start of the file onto its end, so the voice never has to know.
struct Producer {
    pending: Vec<[f32; 2]>,
    at: usize,
}

impl Producer {
    fn new(data: &SampleData, loops: usize) -> Self {
        let mut pending = Vec::with_capacity(data.frame_count() * loops);
        for _ in 0..loops {
            for f in 0..data.frame_count() {
                // Up-mixed exactly as `SampleData::frame_stereo` does — mono into both channels.
                pending.push(data.frame_stereo(f));
            }
        }
        Self { pending, at: 0 }
    }

    /// Fill whatever chunks the audio thread has handed back.
    fn top_up(&mut self, feeder: &StreamFeeder) {
        while self.at < self.pending.len() {
            let Some(mut chunk) = feeder.take_empty() else {
                return; // ring is full — the producer is ahead, which is where it belongs
            };
            let n = (self.pending.len() - self.at).min(ph2d_audio::STREAM_CHUNK_FRAMES);
            {
                let buf = chunk.buffer_mut();
                for k in 0..n {
                    buf[k * 2] = self.pending[self.at + k][0];
                    buf[k * 2 + 1] = self.pending[self.at + k][1];
                }
            }
            chunk.set_frames(n);
            if feeder.submit(chunk).is_err() {
                return;
            }
            self.at += n;
        }
        feeder.finish();
    }
}

/// Render `blocks` blocks of a voice into one long buffer.
fn render(
    engine: &mut AudioEngine,
    renderer: &mut ph2d_audio::AudioRenderer,
    blocks: usize,
) -> Vec<f32> {
    const FRAMES: usize = 256;
    let mut out = Vec::new();
    for _ in 0..blocks {
        let mut block = vec![0.0f32; FRAMES * 2];
        renderer.render(&mut block, FRAMES);
        out.extend_from_slice(&block);
        engine.collect_returns();
    }
    out
}

/// Play `data` resident, and render.
fn resident_output(data: &SampleData, params: PlayParams, blocks: usize) -> Vec<f32> {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(OUT_RATE));
    engine.play(data.clone(), params).expect("play");
    render(&mut engine, &mut renderer, blocks)
}

/// Play `data` streamed (kept fed, so no underruns), and render.
fn streamed_output(data: &SampleData, params: PlayParams, blocks: usize, loops: usize) -> Vec<f32> {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(OUT_RATE));
    let (feeder, handle) = stream(data.format().sample_rate);
    if params.looping {
        // What the real producer does when it reaches the end of the first pass.
        feeder.set_loop_frames(data.frame_count());
    }
    let mut producer = Producer::new(data, loops);
    producer.top_up(&feeder);
    engine.play_stream(handle, params).expect("play_stream");

    const FRAMES: usize = 256;
    let mut out = Vec::new();
    for _ in 0..blocks {
        // Top the ring up between blocks — the producer thread's job, done inline here so the
        // comparison isolates the *reading*, not the scheduling.
        producer.top_up(&feeder);
        let mut block = vec![0.0f32; FRAMES * 2];
        renderer.render(&mut block, FRAMES);
        out.extend_from_slice(&block);
        engine.collect_returns();
    }
    out
}

fn assert_identical(a: &[f32], b: &[f32], what: &str) {
    assert_eq!(a.len(), b.len(), "{what}: different block counts");
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        assert_eq!(
            x.to_bits(),
            y.to_bits(),
            "{what}: sample {i} differs — resident {x}, streamed {y}"
        );
    }
}

/// The plain case: same rate, no loop, plays out and ends.
#[test]
fn a_streamed_voice_renders_what_a_resident_one_renders() {
    let data = clip(3_000, OUT_RATE, ChannelLayout::Stereo);
    let p = PlayParams::default();
    let a = resident_output(&data, p.clone(), 20);
    let b = streamed_output(&data, p, 20, 1);
    assert_identical(&a, &b, "plain");
    assert!(a.iter().any(|s| *s != 0.0), "the fixture rendered silence");
}

/// **Fractional advance.** A 44.1 kHz asset in a 48 kHz mixer: every output frame lands between two
/// source frames, so the interpolation is doing real work and the streamed window has to hold
/// exactly the pair the resident index would.
#[test]
fn a_stream_at_another_rate_interpolates_identically() {
    let data = clip(3_000, 44_100, ChannelLayout::Stereo);
    let p = PlayParams::default();
    let a = resident_output(&data, p.clone(), 20);
    let b = streamed_output(&data, p, 20, 1);
    assert_identical(&a, &b, "44.1k -> 48k");
}

/// **Mono up-mix.** The resident path up-mixes with `frame_stereo`; the producer up-mixes into the
/// chunk. Up-mixing is linear, so interpolating up-mixed frames and up-mixing interpolated frames
/// are the same arithmetic — this is the gate that says so out loud.
#[test]
fn a_mono_stream_up_mixes_identically() {
    let data = clip(3_000, OUT_RATE, ChannelLayout::Mono);
    let p = PlayParams::default();
    let a = resident_output(&data, p.clone(), 20);
    let b = streamed_output(&data, p, 20, 1);
    assert_identical(&a, &b, "mono up-mix");
}

/// **The loop point** — the one that is easy to get *nearly* right.
///
/// The resident voice wraps its cursor and interpolates the last frame against **frame 0**. The
/// streamed voice never wraps: the producer splices the file's start onto its end, so the "next
/// frame" after the last one simply *is* frame 0. Same samples, arrived at from opposite
/// directions. If they disagree, a looping stream ticks at the seam.
#[test]
fn a_looping_stream_crosses_the_seam_identically() {
    let data = clip(700, 44_100, ChannelLayout::Stereo);
    let p = PlayParams {
        looping: true,
        ..PlayParams::default()
    };
    // Long enough to cross the loop point several times.
    let a = resident_output(&data, p.clone(), 30);
    let b = streamed_output(&data, p, 30, 12);
    assert_identical(&a, &b, "loop seam");
}

/// **A5: the end.** A streamed voice must stop exactly where a resident one stops — not a frame
/// early (a clipped tail) and not a frame late (a repeated sample).
#[test]
fn a_stream_ends_where_a_resident_clip_ends() {
    let data = clip(500, OUT_RATE, ChannelLayout::Stereo);
    let p = PlayParams::default();
    let a = resident_output(&data, p.clone(), 8);
    let b = streamed_output(&data, p, 8, 1);
    assert_identical(&a, &b, "end");

    // And it really did end: the tail of the render is silence, and the voice is gone.
    let tail = &b[b.len() - 512..];
    assert!(
        tail.iter().all(|s| *s == 0.0),
        "the streamed voice kept playing past the end of its source"
    );
}

/// A voice that never got fed plays **silence** and **survives** (A4) — it does not panic, it does
/// not end, and it does not steal a slot forever by mistake.
#[test]
fn an_underrun_is_silence_that_the_voice_survives() {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(OUT_RATE));
    let (feeder, handle) = stream(OUT_RATE);
    let id = engine
        .play_stream(handle, PlayParams::default())
        .expect("play");
    assert_ne!(id, VoiceId::NONE);

    let out = render(&mut engine, &mut renderer, 4);
    assert!(
        out.iter().all(|s| *s == 0.0),
        "a starved stream must render silence, not garbage"
    );

    // Now feed it. The voice is still alive, and it plays what finally arrived — nothing was lost.
    let data = clip(1_000, OUT_RATE, ChannelLayout::Stereo);
    Producer::new(&data, 1).top_up(&feeder);
    let out = render(&mut engine, &mut renderer, 4);
    assert!(
        out.iter().any(|s| *s != 0.0),
        "the voice did not resume after the producer caught up — an underrun killed it"
    );
}
