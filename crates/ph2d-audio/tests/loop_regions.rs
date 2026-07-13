//! **ADR-0119** — the mixer honours a loop region, so intro→loop is a thing a game can play.
//!
//! What each gate here is defending:
//!
//! - **A1** — a voice that did not ask for a region renders **byte-identically** to before. This is
//!   the one that lets the change be safe: everything already in the engine keeps its output.
//! - **A2** — `[0..end)` once, then `[start..end)` for ever. The intro is heard **exactly once**,
//!   and the frame after `end - 1` is `start` — not a held frame, not a notch of silence.
//! - **A3** — the **streamed** path is bit-identical to the resident one across the wrap (ADR-0118's
//!   standard; the 1-ulp lesson is why this is a gate and not a listen).
//! - **A8** — a degenerate region is **refused, not obeyed**: never a hang, never a stutter.

use ph2d_audio::{
    AudioEngine, AudioFormat, ChannelLayout, LoopRegion, PlayParams, SampleData, StreamFeeder,
    stream,
};

const OUT_RATE: u32 = 48_000;

/// Frame `i` carries the value `(i + 1) / 64` — a **stamp**, so the output says exactly which source
/// frame it came from.
///
/// Starting at 1 (not 0) so the constant gain × pan factor can be read off frame 0 and divided out;
/// scaled down because **the master clips at unity**, and a raw stamp of 2.0 comes out of the mixer
/// as 1.0 like every other one. (That is how this gate first failed: every stamp read back as √2.)
fn stamped(frames: usize, rate: u32) -> SampleData {
    SampleData::from_fn(frames, AudioFormat::mono(rate), |i| (i + 1) as f32 / 64.0)
}

/// A signal with structure — a ramp would hide an off-by-one that this exposes.
fn textured(frames: usize, rate: u32, channels: ChannelLayout) -> SampleData {
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

fn params(looping: bool, region: Option<LoopRegion>) -> PlayParams {
    PlayParams {
        looping,
        loop_region: region,
        ..PlayParams::default()
    }
}

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

fn resident_output(data: &SampleData, p: PlayParams, blocks: usize) -> Vec<f32> {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(OUT_RATE));
    engine.play(data.clone(), p).expect("play");
    render(&mut engine, &mut renderer, blocks)
}

/// The LEFT channel, in source-frame units: the voice applies a constant gain × pan, and frame 0's
/// stamp is 1/64, so frame 0's output IS that constant divided by 64.
fn left_in_stamps(out: &[f32]) -> Vec<f32> {
    let k = out[0];
    assert!(k > 0.0, "the constant gain factor must be readable");
    out.chunks(2).map(|f| f[0] / k).collect()
}

/// Compare recovered stamps. The comparison is approximate on purpose: the question a stamp answers
/// is **which source frame is this**, and neighbouring frames are a whole 1.0 apart — so a tolerance
/// of a thousandth cannot let a wrong frame through, and it does not make the gate hostage to the
/// last bit of a gain constant that was divided back out.
fn assert_stamps(actual: &[f32], expected: &[f32], msg: &str) {
    assert!(actual.len() >= expected.len(), "not enough output");
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - e).abs() < 1e-3,
            "{msg}\n  output frame {i}: read source frame {a:.3}, expected {e:.3}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// A1 — nothing changes for anyone who did not ask.
// ---------------------------------------------------------------------------------------------

/// A whole-buffer loop (`loop_region: None`) renders **byte-identically** to a voice that never
/// heard of regions. Everything already in the engine keeps its output, exactly.
#[test]
fn a_loop_without_a_region_is_byte_identical_to_the_old_whole_buffer_loop() {
    let data = textured(1_000, 44_100, ChannelLayout::Stereo);
    let looped = resident_output(&data, params(true, None), 12);

    // The same thing said the other way: a region that spans the whole buffer.
    let whole = resident_output(
        &data,
        params(
            true,
            Some(LoopRegion {
                start: 0,
                end: 1_000,
            }),
        ),
        12,
    );
    assert_eq!(
        looped, whole,
        "a region covering the whole buffer must be the plain whole-buffer loop, bit for bit"
    );
}

// ---------------------------------------------------------------------------------------------
// A2 — intro → loop.
// ---------------------------------------------------------------------------------------------

/// The shape of the whole feature: play `[0..8)`, then `[4..8)` for ever. **The intro is heard
/// exactly once** — which is what makes this intro→loop and not just "a loop".
#[test]
fn the_intro_plays_once_and_then_the_body_repeats() {
    let data = stamped(12, OUT_RATE); // 1:1 rate → advance = 1.0, so every output IS a source frame
    let out = resident_output(
        &data,
        params(true, Some(LoopRegion { start: 4, end: 8 })),
        1,
    );
    let stamps = left_in_stamps(&out);

    // Source frame `i` carries `i + 1`, so the expected stamps are the frame indices plus one.
    let expected: Vec<f32> = (0..8)
        .chain((4..8).cycle().take(24))
        .map(|f| (f + 1) as f32)
        .collect();
    assert_stamps(
        &stamps,
        &expected,
        "expected the intro [0..8) once, then [4..8) round and round",
    );

    // And the tail past the loop end is never reached: a looping voice does not fall out of its
    // loop into the outro.
    assert!(
        !stamps[..256].iter().any(|&s| s > 8.0),
        "audio past the loop end leaked into playback"
    );
}

/// **The seam is sample-accurate.** Halfway between the last frame of the lap and the next one, the
/// voice must be interpolating towards `start` — not holding the last frame, and not reading on into
/// the outro. At a half-rate advance the cursor lands exactly on that midpoint, so the value is a
/// single number and there is nowhere for a fudge to hide.
#[test]
fn the_frame_after_the_loop_end_is_the_loop_start() {
    // Source at half the output rate → advance = 0.5: every other output frame sits *between* two
    // source frames.
    let data = stamped(12, OUT_RATE / 2);
    let out = resident_output(
        &data,
        params(true, Some(LoopRegion { start: 4, end: 8 })),
        1,
    );
    let stamps = left_in_stamps(&out);

    // Output frame 15 → cursor 7.5: straddling source frames 7 and (the wrap) 4.
    // Stamps are frame+1, so: 8 + (5 - 8) * 0.5 = 6.5.
    //   held last frame  would give 8.0
    //   reading on to frame 8 would give 8.5
    assert!(
        (stamps[15] - 6.5).abs() < 1e-5,
        "the seam interpolated to {} — it must land halfway between the lap's last frame and its \
         FIRST (6.5); 8.0 means it held, 8.5 means it read past the loop into the outro",
        stamps[15]
    );
}

// ---------------------------------------------------------------------------------------------
// A3 — streamed == resident, bit for bit, across the wrap.
// ---------------------------------------------------------------------------------------------

/// The producer's job for a region: emit `[0..end)`, then `[start..end)` for ever. This is the
/// stand-in for `ph2d-audio-stream`'s `pump`, doing exactly that.
struct Producer {
    pending: Vec<[f32; 2]>,
    at: usize,
}

impl Producer {
    fn for_region(data: &SampleData, r: LoopRegion, laps: usize) -> Self {
        let mut pending = Vec::new();
        for f in 0..r.end as usize {
            pending.push(data.frame_stereo(f));
        }
        for _ in 0..laps {
            for f in r.start as usize..r.end as usize {
                pending.push(data.frame_stereo(f));
            }
        }
        Self { pending, at: 0 }
    }

    fn top_up(&mut self, feeder: &StreamFeeder) {
        while self.at < self.pending.len() {
            let Some(mut chunk) = feeder.take_empty() else {
                return;
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
    }
}

fn streamed_output(data: &SampleData, r: LoopRegion, blocks: usize, laps: usize) -> Vec<f32> {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(OUT_RATE));
    let (feeder, handle) = stream(data.format().sample_rate);
    // What the real producer publishes once it reaches the turn-around.
    feeder.set_loop_region(r.start as usize, r.end as usize);
    let mut producer = Producer::for_region(data, r, laps);
    producer.top_up(&feeder);
    engine
        .play_stream(handle, params(true, None))
        .expect("play_stream");

    const FRAMES: usize = 256;
    let mut out = Vec::new();
    for _ in 0..blocks {
        producer.top_up(&feeder);
        let mut block = vec![0.0f32; FRAMES * 2];
        renderer.render(&mut block, FRAMES);
        out.extend_from_slice(&block);
        engine.collect_returns();
    }
    out
}

/// **The same buffer. Not "close".** A fractional rate (44.1 kHz asset, 48 kHz mixer) so the
/// interpolation straddles source frames, and enough blocks to go round the loop several times —
/// which is where the resident path's cursor wrap and the stream's `base` bookkeeping have to agree
/// to the last bit. They parted company by exactly one ulp the last time this was not gated.
#[test]
fn a_streamed_region_is_bit_identical_to_a_resident_one() {
    let data = textured(1_000, 44_100, ChannelLayout::Stereo);
    let r = LoopRegion {
        start: 200,
        end: 800,
    };
    let blocks = 20; // ~5_120 output frames: several laps of a 600-frame body

    let resident = resident_output(&data, params(true, Some(r)), blocks);
    let streamed = streamed_output(&data, r, blocks, 20);

    assert_eq!(resident.len(), streamed.len());
    for (i, (a, b)) in resident.iter().zip(&streamed).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "sample {i}: resident {a} vs streamed {b} — a streaming path that only sounds NEARLY \
             the same is the worst possible outcome"
        );
    }
}

/// Same, for a mono source up-mixed into a stereo mixer — the other place the two paths could drift.
#[test]
fn a_streamed_mono_region_is_bit_identical_too() {
    let data = textured(700, 32_000, ChannelLayout::Mono);
    let r = LoopRegion {
        start: 100,
        end: 500,
    };
    let resident = resident_output(&data, params(true, Some(r)), 16);
    let streamed = streamed_output(&data, r, 16, 30);
    assert_eq!(
        resident.iter().map(|f| f.to_bits()).collect::<Vec<_>>(),
        streamed.iter().map(|f| f.to_bits()).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------------------------
// A8 — a degenerate region is refused, not obeyed.
// ---------------------------------------------------------------------------------------------

/// A region that names nothing must fall back to the whole-buffer loop — **never** hang, stutter, or
/// silence the voice. Each of these is a value a `smpl` chunk in the wild can actually contain.
#[test]
fn a_region_that_names_nothing_falls_back_to_the_whole_buffer() {
    let data = textured(500, 48_000, ChannelLayout::Mono);
    let plain = resident_output(&data, params(true, None), 8);

    for (r, what) in [
        (
            LoopRegion {
                start: 300,
                end: 300,
            },
            "empty (start == end)",
        ),
        (
            LoopRegion {
                start: 400,
                end: 100,
            },
            "inverted (end < start)",
        ),
        (
            LoopRegion {
                start: 600,
                end: 900,
            },
            "entirely past the end of the audio",
        ),
    ] {
        let out = resident_output(&data, params(true, Some(r)), 8);
        assert_eq!(
            out, plain,
            "a {what} region must be ignored, not obeyed — it fell through to something else"
        );
    }
}

/// A region whose END runs past the audio is **clamped to the audio**, not obeyed: the voice wraps
/// where the frames actually stop.
#[test]
fn a_region_running_past_the_end_is_clamped_to_the_audio() {
    let data = stamped(12, OUT_RATE);
    let out = resident_output(
        &data,
        // Asked for [4..99) on a 12-frame clip.
        params(true, Some(LoopRegion { start: 4, end: 99 })),
        1,
    );
    let stamps = left_in_stamps(&out);
    let expected: Vec<f32> = (0..12)
        .chain((4..12).cycle().take(24))
        .map(|f| (f + 1) as f32)
        .collect();
    assert_stamps(
        &stamps,
        &expected,
        "the loop must turn around at the last real frame, not at the frame someone hoped for",
    );
}

/// A one-shot ignores a region entirely: a loop region on a sound that does not loop names nothing.
#[test]
fn a_one_shot_ignores_the_region() {
    let data = stamped(12, OUT_RATE);
    let with = resident_output(
        &data,
        params(false, Some(LoopRegion { start: 4, end: 8 })),
        1,
    );
    let without = resident_output(&data, params(false, None), 1);
    assert_eq!(
        with, without,
        "a one-shot must not loop just because a region was set"
    );
    // And it really did stop: everything past the clip is silence.
    assert!(
        with[40..].iter().all(|&s| s == 0.0),
        "the one-shot kept going"
    );
}
