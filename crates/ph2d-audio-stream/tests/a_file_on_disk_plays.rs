//! **End to end.** A real file, on disk, decoded by a real producer thread, played by a real voice.
//!
//! The engine half of ADR-0118 was gated in `ph2d-audio` — rings, windows, bit-identity. All of it
//! could be green while the feature was still dead, because nothing decoded a file into a stream.
//! That is the exact failure this project has a name for: unit-green, integration-dead.
//!
//! So: write a WAV, write an Opus, stream both, and check that audio actually comes out — through
//! `play_file`, the entry point a game would use.

use std::io::Write;

use ph2d_audio::{AudioEngine, AudioFormat, ChannelLayout, PlayParams, SampleData};

const RATE: u32 = 48_000;

fn tone(frames: usize) -> SampleData {
    SampleData::from_fn(
        frames * 2,
        AudioFormat {
            sample_rate: RATE,
            channels: ChannelLayout::Stereo,
        },
        |i| {
            let t = (i / 2) as f32 / RATE as f32;
            (std::f32::consts::TAU * 220.0 * t).sin() * 0.4
        },
    )
}

/// Write `bytes` to a uniquely-named temp file and hand back the path.
fn temp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("ph2d-stream-{}-{name}", std::process::id()));
    let mut f = std::fs::File::create(&p).expect("create temp file");
    f.write_all(bytes).expect("write temp file");
    p
}

/// Play `path` streamed and return what the mixer rendered.
fn stream_out(path: &std::path::Path, looping: bool, blocks: usize) -> Vec<f32> {
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(RATE));
    let (handle, player) =
        ph2d_audio_stream::play_file(path, looping, None).expect("open the stream");
    engine
        .play_stream(handle, PlayParams::default())
        .expect("play_stream");

    const FRAMES: usize = 512;
    let mut out = Vec::new();
    for _ in 0..blocks {
        // Give the producer thread a moment to get ahead — in a game the audio callback is a
        // millisecond of work every ~10 ms, and the producer has all that time. Here we are looping
        // as fast as the CPU allows, so without this the test measures the scheduler, not the code.
        std::thread::sleep(std::time::Duration::from_millis(3));
        let mut block = vec![0.0f32; FRAMES * 2];
        renderer.render(&mut block, FRAMES);
        out.extend_from_slice(&block);
        engine.collect_returns();
    }
    player.stop();
    out
}

fn rms(x: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    (x.iter().map(|s| s * s).sum::<f32>() / x.len() as f32).sqrt()
}

/// **A WAV on disk plays.** The whole chain: `play_file` → Symphonia `Reader` → producer thread →
/// ring → streaming voice → mixer.
#[test]
fn a_wav_file_streams_and_makes_sound() {
    let clip = tone(48_000); // one second
    let wav =
        ph2d_audio_encode::encode_wav(&clip, ph2d_audio_encode::BitDepth::Pcm16).expect("wav");
    let path = temp("tone.wav", &wav);

    let out = stream_out(&path, false, 40);
    let _ = std::fs::remove_file(&path);

    assert!(
        rms(&out) > 0.05,
        "the WAV streamed silence (rms {:.4}) — the producer, the ring, or the voice is not \
         connected",
        rms(&out)
    );
}

/// **An Opus file on disk plays** — the codec Symphonia cannot read, streamed through the crate that
/// can. This is the combination the whole ADR is about: small on disk *and* small in RAM.
#[test]
fn an_opus_file_streams_and_makes_sound() {
    let clip = tone(48_000);
    let opus = ph2d_audio_encode::encode_opus(&clip, 0.5).expect("opus");
    let path = temp("tone.opus", &opus);

    let out = stream_out(&path, false, 40);
    let _ = std::fs::remove_file(&path);

    assert!(
        rms(&out) > 0.05,
        "the Opus file streamed silence (rms {:.4})",
        rms(&out)
    );
}

/// **A looping stream keeps going** past the end of its file — and the producer publishes the
/// length so the voice can wrap its cursor exactly as a resident one does.
#[test]
fn a_looping_stream_outlives_its_file() {
    // Deliberately short: 0.2 s, so 40 blocks of 512 frames (0.43 s) must cross the loop point.
    let clip = tone(9_600);
    let wav =
        ph2d_audio_encode::encode_wav(&clip, ph2d_audio_encode::BitDepth::Pcm16).expect("wav");
    let path = temp("loop.wav", &wav);

    let out = stream_out(&path, true, 40);
    let _ = std::fs::remove_file(&path);

    // The tail must still be sounding: a non-looping stream would have ended long ago.
    let tail = &out[out.len() - 4_096..];
    assert!(
        rms(tail) > 0.05,
        "the looping stream went quiet after its file ran out (tail rms {:.4}) — it did not loop",
        rms(tail)
    );
}

/// **Intro→loop, end to end** (ADR-0119 A2/A4): a real file, a real region, the real producer.
///
/// The producer is where the sequence `[0..end)` then `[start..end)` is actually *built*, and it is
/// the one piece of this that lives nowhere else. Removing the "only skip the intro after the first
/// lap" condition left every gate in `ph2d-audio` green — the mixer half is fine either way, because
/// it just plays what it is fed. This is the gate that watches what it is fed.
///
/// The file is built to say which part is playing: **loud intro, silent body, loud outro**. So both
/// failures are the same question about energy, and the answer is a number:
///
/// - re-send the intro on every lap → the tail is loud;
/// - forget to turn around at the loop end → playback runs on into the **outro**, and the tail is
///   loud.
///
/// The outro is why it is there. Without it the region ends where the file does, and "never turn
/// around at the loop end" is indistinguishable from "turn around at EOF" — the gate would be blind
/// to it, which is exactly what mutation showed it was.
#[test]
fn a_streamed_region_plays_its_intro_exactly_once_and_never_reaches_the_outro() {
    const INTRO: usize = 24_000; // 0.5 s of tone
    const BODY: usize = 24_000; // 0.5 s of quiet — the loop
    const OUTRO: usize = 24_000; // 0.5 s of tone the loop must never reach
    let clip = SampleData::from_fn(
        (INTRO + BODY + OUTRO) * 2,
        AudioFormat {
            sample_rate: RATE,
            channels: ChannelLayout::Stereo,
        },
        |i| {
            let f = i / 2;
            if !(INTRO..INTRO + BODY).contains(&f) {
                let t = f as f32 / RATE as f32;
                (std::f32::consts::TAU * 220.0 * t).sin() * 0.5
            } else {
                0.0
            }
        },
    );
    let wav =
        ph2d_audio_encode::encode_wav(&clip, ph2d_audio_encode::BitDepth::Pcm16).expect("wav");
    let path = temp("introloop.wav", &wav);

    // Loop the QUIET body: [24_000 .. 48_000).
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(RATE));
    let (handle, player) =
        ph2d_audio_stream::play_file(&path, true, Some((INTRO as u64, (INTRO + BODY) as u64)))
            .expect("open the stream");
    engine
        .play_stream(handle, PlayParams::default())
        .expect("play_stream");

    const FRAMES: usize = 512;
    let mut out = Vec::new();
    for _ in 0..160 {
        // ~1.7 s: the 0.5 s intro, then three laps of the 0.5 s body.
        std::thread::sleep(std::time::Duration::from_millis(3));
        let mut block = vec![0.0f32; FRAMES * 2];
        renderer.render(&mut block, FRAMES);
        out.extend_from_slice(&block);
        engine.collect_returns();
    }
    player.stop();
    let _ = std::fs::remove_file(&path);

    // The first half-second is the intro: loud.
    let intro_out = &out[..INTRO * 2];
    assert!(
        rms(intro_out) > 0.1,
        "the intro never played (rms {:.4}) — a region must be heard from frame 0, not from `start`",
        rms(intro_out)
    );

    // Everything after it is the body, on repeat: quiet, for ever. Two different bugs would break
    // that, and this one number sees both — the intro coming back around, and playback running on
    // into the outro instead of turning around.
    let body_out = &out[(INTRO + 2_000) * 2..];
    assert!(
        rms(body_out) < 0.02,
        "loud audio (rms {:.4}) where the looping body should be silent — either the intro came \
         back around (it is heard exactly ONCE) or playback ran on into the outro (a loop turns \
         around at its end; it does not fall out of it)",
        rms(body_out)
    );
}
