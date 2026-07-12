//! Encoder unit tests (round-trips through `ph2d-audio-decode`). Split out of
//! `lib.rs` to keep it under the crate LOC cap (HR-18).

use super::*;
use ph2d_audio::{AudioFormat, ChannelLayout};

#[test]
fn pcm16_round_trips_through_the_decoder() {
    // A stereo ramp so order + interleave survive.
    let frames = 200;
    let mut interleaved = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let t = i as f32 / frames as f32; // 0..1
        interleaved.push(t * 2.0 - 1.0); // L: -1..1 ramp
        interleaved.push(0.5 - t); // R: 0.5..-0.5 ramp
    }
    let src = SampleData::from_interleaved(interleaved, AudioFormat::stereo(48_000));

    let bytes = encode_wav(&src, BitDepth::Pcm16).expect("encode");
    let back = ph2d_audio_decode::decode(&bytes).expect("decode");

    assert_eq!(back.format().sample_rate, 48_000);
    assert_eq!(back.format().channels, ChannelLayout::Stereo);
    assert_eq!(back.frame_count(), frames);
    // PCM16 round-trip tolerance. The error is quantisation (½ LSB) plus the
    // standard encode-by-32767 / decode-by-32768 scale asymmetry — together
    // ~1.5 LSB near full scale (≈4.5e-5). 1e-4 covers it with margin.
    for (a, b) in src.samples().iter().zip(back.samples()) {
        assert!((a - b).abs() < 1e-4, "sample drift {a} vs {b}");
    }
}

#[test]
fn float32_round_trips_bit_close() {
    let src = SampleData::from_interleaved(
        vec![0.123_456, -0.987_654, 0.0, 1.0, -1.0, 0.5],
        AudioFormat::stereo(44_100),
    );
    let bytes = encode_wav(&src, BitDepth::Float32).expect("encode");
    let back = ph2d_audio_decode::decode(&bytes).expect("decode");
    assert_eq!(back.frame_count(), 3);
    for (a, b) in src.samples().iter().zip(back.samples()) {
        assert!((a - b).abs() < 1e-6, "float drift {a} vs {b}");
    }
}

#[test]
fn clamps_out_of_range_on_quantise() {
    // Values beyond [-1,1] must clamp, not wrap.
    assert_eq!(sample_to_i16(2.0), 32_767);
    assert_eq!(sample_to_i16(-2.0), -32_767);
    let [b0, b1, b2, _] = sample_to_i24_le(2.0);
    assert_eq!(i32::from_le_bytes([b0, b1, b2, 0]), 8_388_607);
}

#[test]
fn smpl_loop_round_trips_and_audio_still_decodes() {
    let frames = 500;
    let mut interleaved = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let t = i as f32 / frames as f32;
        interleaved.push(t * 2.0 - 1.0);
        interleaved.push(0.5 - t);
    }
    let src = SampleData::from_interleaved(interleaved, AudioFormat::stereo(48_000));
    let meta = WavMeta {
        loops: vec![LoopRegion {
            start: 100,
            end: 400,
        }],
        ..Default::default()
    };

    let bytes = encode_wav_with_meta(&src, BitDepth::Pcm16, &meta).expect("encode");

    // The loop region survives the write → read (half-open end preserved).
    let back_loops = read_loop_regions(&bytes);
    assert_eq!(back_loops, meta.loops, "smpl loop must round-trip exactly");

    // Symphonia ignores the unknown `smpl` chunk and still decodes the audio.
    let audio = ph2d_audio_decode::decode(&bytes).expect("decode with smpl present");
    assert_eq!(
        audio.frame_count(),
        frames,
        "smpl must not corrupt the audio"
    );
    assert_eq!(audio.format().channels, ChannelLayout::Stereo);
}

#[test]
fn cue_markers_round_trip_and_audio_still_decodes() {
    let src = SampleData::from_interleaved(vec![0.0; 2_000], AudioFormat::stereo(48_000));
    let meta = WavMeta {
        loops: Vec::new(),
        markers: vec![
            Marker {
                frame: 100,
                name: "intro".to_string(),
            },
            Marker {
                frame: 500,
                name: "hit".to_string(),
            },
        ],
    };
    let bytes = encode_wav_with_meta(&src, BitDepth::Pcm16, &meta).expect("encode");

    // Markers (position + label) round-trip, sorted by frame.
    assert_eq!(
        read_markers(&bytes),
        meta.markers,
        "cue+adtl must round-trip"
    );
    // Symphonia ignores the cue + LIST chunks and still decodes the audio.
    let audio = ph2d_audio_decode::decode(&bytes).expect("decode with cue/LIST present");
    assert_eq!(
        audio.frame_count(),
        1_000,
        "markers must not corrupt the audio"
    );
}

#[test]
fn loops_and_markers_coexist_in_one_file() {
    let src = SampleData::from_interleaved(vec![0.1; 40], AudioFormat::mono(48_000));
    let meta = WavMeta {
        loops: vec![LoopRegion { start: 4, end: 30 }],
        markers: vec![Marker {
            frame: 12,
            name: "M".to_string(),
        }],
    };
    let bytes = encode_wav_with_meta(&src, BitDepth::Float32, &meta).unwrap();
    assert_eq!(
        read_loop_regions(&bytes),
        meta.loops,
        "smpl survives alongside cue"
    );
    assert_eq!(
        read_markers(&bytes),
        meta.markers,
        "cue survives alongside smpl"
    );
    assert!(
        ph2d_audio_decode::decode(&bytes).is_ok(),
        "audio still decodes"
    );
}

#[test]
fn no_loop_is_byte_identical_to_bare_encode() {
    // The metadata path must not change the bytes when there are no loops — the
    // regression guard for existing callers / fixtures.
    let src = SampleData::from_interleaved(vec![0.1, -0.2, 0.3, -0.4], AudioFormat::mono(44_100));
    let bare = encode_wav(&src, BitDepth::Pcm16).unwrap();
    let empty = encode_wav_with_meta(&src, BitDepth::Pcm16, &WavMeta::default()).unwrap();
    assert_eq!(
        bare, empty,
        "empty meta must be byte-for-byte the bare file"
    );
    assert!(read_loop_regions(&bare).is_empty(), "no smpl chunk to find");
}

#[test]
fn reads_smpl_placed_before_data() {
    // The `smpl` chunk sits between `fmt ` and `data`; the walker must skip `fmt `.
    let src = SampleData::from_interleaved(vec![0.0; 20], AudioFormat::mono(48_000));
    let meta = WavMeta {
        loops: vec![
            LoopRegion { start: 2, end: 8 },
            LoopRegion { start: 9, end: 15 },
        ],
        ..Default::default()
    };
    let bytes = encode_wav_with_meta(&src, BitDepth::Float32, &meta).unwrap();
    assert_eq!(
        &bytes[36..40],
        b"smpl",
        "smpl comes right after the fmt chunk"
    );
    assert_eq!(
        read_loop_regions(&bytes),
        meta.loops,
        "both loops round-trip"
    );
}

#[test]
fn header_is_well_formed() {
    let src = SampleData::from_interleaved(vec![0.0; 10], AudioFormat::mono(22_050));
    let bytes = encode_wav(&src, BitDepth::Pcm16).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");
    assert_eq!(&bytes[36..40], b"data");
    // data_len = 10 samples * 2 bytes.
    assert_eq!(
        u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]),
        20
    );
}

#[test]
fn ogg_vorbis_round_trips_through_the_decoder() {
    // A 440 Hz stereo sine, 0.5 s. Vorbis is lossy, so we assert structure +
    // duration + energy survive, not sample-exact bytes (unlike the WAV tests).
    let sr = 48_000u32;
    let frames = (sr / 2) as usize;
    let mut v = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        let s = (n as f32 * std::f32::consts::TAU * 440.0 / sr as f32).sin() * 0.5;
        v.push(s);
        v.push(s);
    }
    let src = SampleData::from_interleaved(v, AudioFormat::stereo(sr));

    let bytes = encode_ogg(&src, OGG_DEFAULT_QUALITY).expect("encode ogg");
    assert_eq!(&bytes[0..4], b"OggS", "not an Ogg stream");
    assert!(
        bytes.len() < src.samples().len() * 4,
        "compression didn't shrink vs raw f32 ({} bytes)",
        bytes.len()
    );

    // Re-decode through the real Symphonia path (Vorbis is a supported decoder).
    let back = ph2d_audio_decode::decode(&bytes).expect("decode ogg");
    assert_eq!(back.format().channels, ChannelLayout::Stereo);
    assert_eq!(back.format().sample_rate, sr);
    // Vorbis pads with lookahead/lookback; duration is within ~50 ms.
    let drift = (back.frame_count() as i64 - frames as i64).abs();
    assert!(
        drift < (sr / 20) as i64,
        "duration drifted: {} vs {frames}",
        back.frame_count()
    );
    // Lossy but not silent: a 0.5-amplitude sine has RMS ≈ 0.354; allow wide margin.
    let rms = (back.samples().iter().map(|s| s * s).sum::<f32>()
        / back.samples().len().max(1) as f32)
        .sqrt();
    assert!((0.2..0.5).contains(&rms), "decoded RMS off: {rms}");
}

#[test]
fn ogg_encodes_mono() {
    // Mono must de-interleave to a single planar channel (i % 1 == 0).
    let sr = 44_100u32;
    let frames = 4_000usize;
    let v: Vec<f32> = (0..frames)
        .map(|n| (n as f32 * std::f32::consts::TAU * 220.0 / sr as f32).sin() * 0.4)
        .collect();
    let src = SampleData::from_interleaved(v, AudioFormat::mono(sr));
    let bytes = encode_ogg(&src, 0.3).expect("encode mono ogg");
    let back = ph2d_audio_decode::decode(&bytes).expect("decode mono ogg");
    assert_eq!(back.format().channels, ChannelLayout::Mono);
}

/// **Chunked encoding must not lose or repeat a block.** `encode_ogg` now feeds
/// libvorbis in `CHUNK_FRAMES` slices — handing it the whole take at once made the cost
/// blow up superlinearly (measured: 48 ms for a 10 s clip but **27.5 seconds** for a
/// 5-minute one; chunked, the same 5 minutes take 0.95 s). The hazard the rewrite
/// introduces is off-by-a-block at the seams, which a duration check alone would miss:
/// a dropped chunk in the middle still decodes to *about* the right length.
///
/// So the fixture carries an amplitude RAMP, and the test reads the envelope back at
/// three points. Drop or duplicate a chunk and the ramp shifts, and these move.
#[test]
fn chunked_ogg_keeps_the_audio_in_the_right_place() {
    let sr = 48_000u32;
    // Deliberately NOT a multiple of the chunk size — the last block is a short one.
    let frames = 4_096 * 3 + 777;
    let mut v = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        let ramp = n as f32 / frames as f32; // 0 -> 1 across the clip
        let s = (n as f32 * std::f32::consts::TAU * 440.0 / sr as f32).sin() * 0.9 * ramp;
        v.push(s);
        v.push(s);
    }
    let src = SampleData::from_interleaved(v, AudioFormat::stereo(sr));
    let bytes = encode_ogg(&src, OGG_DEFAULT_QUALITY).expect("encode ogg");
    let back = ph2d_audio_decode::decode(&bytes).expect("decode ogg");

    // Peak of the ramp at a quarter, a half and three quarters through: the envelope is
    // a straight line, so each is its own position.
    let peak_at = |d: &SampleData, frac: f32| -> f32 {
        let n = d.frame_count();
        let c = (n as f32 * frac) as usize;
        let lo = c.saturating_sub(400);
        let hi = (c + 400).min(n);
        (lo..hi)
            .map(|f| d.samples()[f * 2].abs())
            .fold(0.0f32, f32::max)
    };
    for frac in [0.25f32, 0.5, 0.75] {
        let want = peak_at(&src, frac);
        let got = peak_at(&back, frac);
        assert!(
            (got - want).abs() < 0.12,
            "the ramp moved at {frac}: expected ~{want:.2}, decoded {got:.2} \
             (a chunk was dropped or repeated)"
        );
    }
}
