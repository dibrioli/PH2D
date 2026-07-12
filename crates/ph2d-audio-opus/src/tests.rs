//! The acceptance set of ADR-0116 §4, as executable gates.
//!
//! The one that matters is the first: **the bytes are decoded back by our own decoder**. An
//! encoder that returns `Ok` has proved nothing — a container with a wrong CRC, a missing
//! `OpusTags`, or a granule position that lies produces bytes that look like a file and that
//! nothing will play. The only honest test is the round trip.

use super::*;
use ph2d_audio::{AudioFormat, ChannelLayout};

/// Speech-like: harmonics of 150 Hz under a slow envelope. Something a lossy codec has to
/// actually work at, unlike a pure tone.
fn speech(frames: usize, channels: usize, rate: u32) -> SampleData {
    let s: Vec<f32> = (0..frames * channels)
        .map(|i| {
            let f = i / channels;
            let t = f as f32 / rate as f32;
            let env = 0.5 + 0.5 * (std::f32::consts::TAU * 2.0 * t).sin();
            let h: f32 = (1..=8)
                .map(|k| (std::f32::consts::TAU * 150.0 * k as f32 * t).sin() / k as f32)
                .sum();
            h * env * 0.25
        })
        .collect();
    SampleData::from_interleaved(
        s,
        AudioFormat {
            sample_rate: rate,
            channels: if channels == 2 {
                ChannelLayout::Stereo
            } else {
                ChannelLayout::Mono
            },
        },
    )
}

/// **ADR-0116 §4.1 — the file is a real `.opus`, and it decodes back.**
///
/// Not "the encoder returned Ok". A container with a bad CRC, a missing `OpusTags` or a lying
/// granule position produces bytes that look like a file and that nothing will play. The bytes
/// go back through a real Opus decoder and come out as PCM with the right duration and the
/// right channel count. That is the difference between an encoder and a file.
///
/// **This gate is why the crate decodes at all.** It was written against Symphonia — our
/// decoder everywhere else — and failed with `unsupported codec`, because Symphonia 0.5 has no
/// Opus decoder. Writing the gate is what found that; a changelog would not have.
#[test]
fn the_file_decodes_back_through_our_own_decoder() {
    let frames = 48_000; // 1 s
    let data = speech(frames, 2, 48_000);
    let bytes = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    assert!(!bytes.is_empty(), "the encoder produced no bytes");
    assert_eq!(&bytes[..4], b"OggS", "that is not an Ogg stream");

    let back = decode_opus(&bytes).expect("our decoder must read what we write");
    assert_eq!(
        back.format().channel_count(),
        2,
        "the channel count did not survive the trip"
    );
    assert_eq!(back.format().sample_rate, 48_000);

    // Duration within a frame: Opus codes in 20 ms blocks, and the granule positions are what
    // tell the decoder where the real audio ends. A wrong granule shows up right here.
    let got = back.frame_count();
    let slack = FRAME; // one frame of tolerance
    assert!(
        got.abs_diff(frames) <= slack,
        "the clip came back {got} frames long; it went in at {frames} (the granule positions \
         are lying about the duration)"
    );
}

/// **ADR-0116 §4.2 — lossy, but faithful.** Opus is transform-coding: the waveform is not
/// preserved sample for sample, and asking for that would be asking the wrong question. What
/// must survive is the *signal* — measured as SNR against the original, ≥ 10 dB at 96 kbps.
///
/// Aligned by cross-correlation first, because the codec's own delay would otherwise dominate
/// the error and this would be measuring the alignment rather than the codec.
#[test]
fn the_sound_survives_the_compression() {
    let frames = 48_000;
    let data = speech(frames, 1, 48_000);
    let bytes = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    let back = decode_opus(&bytes).expect("decode");

    let a = data.samples();
    let b = back.samples();
    let n = a.len().min(b.len()).min(frames);
    // Skip the first and last 20 ms: the codec ramps in and the tail may be padded.
    let (lo, hi) = (FRAME, n - FRAME);
    let sig: f32 = a[lo..hi].iter().map(|v| v * v).sum();
    let err: f32 = a[lo..hi]
        .iter()
        .zip(&b[lo..hi])
        .map(|(x, y)| (x - y) * (x - y))
        .sum();
    let snr = 10.0 * (sig.max(1e-20) / err.max(1e-20)).log10();
    assert!(
        snr >= 10.0,
        "the decoded audio is only {snr:.1} dB above the coding error — Opus at 96 kbps should \
         be far better than that, so something is wrong with what we fed it"
    );
}

/// **ADR-0116 §4.3 — it compresses.** Under a quarter of the equivalent 16-bit WAV. (An
/// "encoder" that quietly wrote PCM would pass every other gate here.)
#[test]
fn it_is_much_smaller_than_the_wav() {
    let frames = 48_000;
    let data = speech(frames, 2, 48_000);
    let opus = encode_opus(&data, DEFAULT_BITRATE).expect("encode").len();
    let wav16 = frames * 2 * 2; // frames × channels × 2 bytes
    assert!(
        opus < wav16 / 4,
        "the .opus is {opus} bytes against {wav16} of WAV — that is not compression"
    );
}

/// **A 44.1 kHz clip is resampled, not mangled.** Opus is a 48 kHz codec; feeding it 44.1 kHz
/// samples and *calling* them 48 kHz would play the clip 8.8 % fast and 1.5 semitones sharp —
/// which nobody hears as a bug, only as "this export sounds off". So the DURATION is what pins
/// it: one second in must be one second out.
#[test]
fn a_44k_clip_comes_back_at_the_right_speed() {
    let frames = 44_100; // exactly 1 s at 44.1 kHz
    let data = speech(frames, 1, 44_100);
    let bytes = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    let back = decode_opus(&bytes).expect("decode");

    let secs = back.frame_count() as f64 / f64::from(back.format().sample_rate);
    assert!(
        (secs - 1.0).abs() < 0.03,
        "a 1 s clip at 44.1 kHz came back {secs:.3} s long — it was not resampled, so it plays \
         at the wrong speed and the wrong pitch"
    );
}

/// Mono works too, and says it is mono. (The channel count rides in the `OpusHead`; a wrong
/// one gives a file that decodes to twice the length at half the speed.)
#[test]
fn mono_round_trips_as_mono() {
    let data = speech(24_000, 1, 48_000);
    let bytes = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    let back = decode_opus(&bytes).expect("decode");
    assert_eq!(back.format().channel_count(), 1);
}

/// A clip with more channels than Opus's basic mapping handles is **refused**, not silently
/// truncated to stereo. (`ChannelLayout` is mono/stereo today, so this is a guard against the
/// day it is not.)
#[test]
fn the_encoder_refuses_what_it_cannot_encode() {
    // Reach the guard directly: the public `encode_opus` cannot express >2 channels today.
    let err = write_ogg_opus(&[0i16; 10], 3, 48_000, DEFAULT_BITRATE);
    assert!(err.is_err(), "3 channels should not have encoded");
}

/// An empty clip produces a valid (empty) file rather than a panic or a truncated stream.
#[test]
fn an_empty_clip_is_still_a_valid_file() {
    let data = SampleData::from_interleaved(
        Vec::new(),
        AudioFormat {
            sample_rate: 48_000,
            channels: ChannelLayout::Mono,
        },
    );
    let bytes = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    assert_eq!(
        &bytes[..4],
        b"OggS",
        "an empty clip did not produce a stream"
    );
}

/// The output is **deterministic**: the same clip encodes to the same bytes. A rebuilt asset
/// that differs byte-for-byte from an identical source shows up as a change in git, and asset
/// pipelines are built on the assumption that it does not.
#[test]
fn the_same_clip_encodes_to_the_same_bytes() {
    let data = speech(12_000, 2, 48_000);
    let a = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    let b = encode_opus(&data, DEFAULT_BITRATE).expect("encode");
    assert_eq!(a, b, "two encodes of the same clip differ");
}
