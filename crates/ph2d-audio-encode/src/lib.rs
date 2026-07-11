#![forbid(unsafe_code)]
//! `ph2d-audio-encode` — write a [`ph2d_audio::SampleData`] out to a file.
//!
//! **Save-time only** (never on the RT audio thread): editing produces a fresh
//! interleaved-`f32` buffer, and this crate serialises it to disk so an edit is
//! persistable. Kept OUT of the lean `ph2d-audio` core so no writer dependency
//! reaches the mixer — mirroring how `ph2d-audio-decode` isolates Symphonia.
//!
//! ## Scope
//!
//! Canonical RIFF/WAVE: PCM 16-bit, PCM 24-bit, and IEEE-float 32-bit. These
//! round-trip cleanly through `ph2d-audio-decode` (Symphonia `wav`+`pcm`). The
//! game asset-prep wave (`docs/Audio/02_plano_implementacao_completo.md` §6) adds
//! side-car chunks via [`WavMeta`]: loop regions (`smpl`, [`read_loop_regions`]) and
//! named cue markers (`cue `+`LIST/adtl`, [`read_markers`]), both written by
//! [`encode_wav_with_meta`]. Ogg Vorbis / Opus are still to come.

use std::io::Write;
use std::path::Path;

use ph2d_audio::{Sample, SampleData};

/// A loop region for the `smpl` chunk, in **frames**, half-open `start..end`
/// (matching `ph2d_audio_edit::EditClip::loop_region`). Stored in the WAV's `smpl`
/// chunk so the loop survives re-decode and a game runtime can loop sample-exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopRegion {
    /// First frame of the loop.
    pub start: u32,
    /// One past the last looped frame (exclusive). The `smpl` chunk stores the
    /// INCLUSIVE last frame (`end - 1`); the conversion is handled at the boundary.
    pub end: u32,
}

/// A named cue point for the `cue `+`LIST/adtl` chunks, at frame `frame`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    /// Sample-frame position of the cue point.
    pub frame: u32,
    /// Label, written as an `adtl`/`labl` sub-chunk.
    pub name: String,
}

/// Side-car metadata written alongside the audio (loop regions + cue markers). Empty
/// by default, so [`encode_wav`] stays byte-for-byte the bare `fmt`+`data` file.
#[derive(Debug, Clone, Default)]
pub struct WavMeta {
    /// Loop regions for the `smpl` chunk (usually one). Empty ⇒ no `smpl` chunk.
    pub loops: Vec<LoopRegion>,
    /// Cue markers for the `cue `+`LIST/adtl` chunks. Empty ⇒ neither chunk.
    pub markers: Vec<Marker>,
}

/// Errors from encoding / writing audio.
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    /// Writing the encoded bytes to disk failed.
    #[error("audio write failed: {0}")]
    Io(#[from] std::io::Error),
    /// The clip's total byte size overflows a 32-bit RIFF size field
    /// (WAV caps a chunk at 4 GiB; larger needs RF64/W64, not yet supported).
    #[error("clip too large for a 32-bit WAV (needs RF64/W64)")]
    TooLarge,
}

/// Sample encoding for a WAV file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    /// 16-bit signed PCM — the ubiquitous, smallest lossless integer form.
    Pcm16,
    /// 24-bit signed PCM — headroom for further processing without float.
    Pcm24,
    /// 32-bit IEEE float — bit-exact for `[-1.0, 1.0]` `f32` sample data.
    Float32,
}

impl BitDepth {
    /// Bytes per single (one-channel) sample.
    fn bytes_per_sample(self) -> usize {
        match self {
            BitDepth::Pcm16 => 2,
            BitDepth::Pcm24 => 3,
            BitDepth::Float32 => 4,
        }
    }

    /// The WAVE `wFormatTag`: 1 = integer PCM, 3 = IEEE float.
    fn format_tag(self) -> u16 {
        match self {
            BitDepth::Pcm16 | BitDepth::Pcm24 => 1,
            BitDepth::Float32 => 3,
        }
    }

    fn bits(self) -> u16 {
        (self.bytes_per_sample() * 8) as u16
    }
}

/// Encode `data` to an in-memory canonical RIFF/WAVE byte buffer (no side-car
/// metadata). Byte-for-byte a bare `fmt`+`data` file — see [`encode_wav_with_meta`]
/// to attach loop points.
///
/// Interleaving and channel count come straight from the clip's format; the
/// float samples are quantised (with clamp to `[-1.0, 1.0]`) for the integer
/// depths and written verbatim for [`BitDepth::Float32`].
pub fn encode_wav(data: &SampleData, depth: BitDepth) -> Result<Vec<u8>, EncodeError> {
    encode_wav_with_meta(data, depth, &WavMeta::default())
}

/// Fixed part of a `smpl` chunk (9 × u32), before the per-loop records.
const SMPL_HEADER_LEN: usize = 36;
/// Bytes per loop record in a `smpl` chunk (6 × u32).
const SMPL_LOOP_LEN: usize = 24;

/// Encode `data` with side-car [`WavMeta`] — the audio plus a `smpl` chunk when
/// `meta.loops` is non-empty. The `smpl` chunk is written BEFORE `data` so any
/// reader that stops walking chunks at `data` still parses it; unknown-chunk
/// skippers (Symphonia) ignore it and decode the audio unchanged.
pub fn encode_wav_with_meta(
    data: &SampleData,
    depth: BitDepth,
    meta: &WavMeta,
) -> Result<Vec<u8>, EncodeError> {
    let format = data.format();
    let channels = format.channel_count();
    let bytes_per = depth.bytes_per_sample();
    let samples = data.samples();

    let data_len_usize = samples.len() * bytes_per;
    let data_len = u32::try_from(data_len_usize).map_err(|_| EncodeError::TooLarge)?;

    // Optional smpl chunk body (header + one record per loop); `None` ⇒ omit it.
    let smpl = (!meta.loops.is_empty()).then(|| smpl_chunk(&meta.loops, format.sample_rate));
    let smpl_total = smpl.as_ref().map_or(0, |b| 8 + b.len()); // 8 = "smpl" + size
    // Optional cue + LIST/adtl chunks (marker positions + their labels).
    let cue = (!meta.markers.is_empty()).then(|| cue_chunk(&meta.markers));
    let adtl = (!meta.markers.is_empty()).then(|| adtl_chunk(&meta.markers));
    let cue_total = cue.as_ref().map_or(0, |b| 8 + b.len());
    let adtl_total = adtl.as_ref().map_or(0, |b| 8 + b.len());

    // RIFF size covers everything after the 8-byte RIFF header: "WAVE" (4) + fmt
    // (8+16) + optional smpl / cue / LIST + data (8+data_len).
    let riff_len = u32::try_from(4 + 24 + smpl_total + cue_total + adtl_total + 8 + data_len_usize)
        .map_err(|_| EncodeError::TooLarge)?;

    let mut v = Vec::with_capacity(44 + smpl_total + cue_total + adtl_total + data_len_usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&riff_len.to_le_bytes());
    v.extend_from_slice(b"WAVE");

    // fmt  chunk (16-byte PCM/float form).
    let block_align = (channels * bytes_per) as u16;
    let byte_rate = format.sample_rate * block_align as u32;
    v.extend_from_slice(b"fmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&depth.format_tag().to_le_bytes());
    v.extend_from_slice(&(channels as u16).to_le_bytes());
    v.extend_from_slice(&format.sample_rate.to_le_bytes());
    v.extend_from_slice(&byte_rate.to_le_bytes());
    v.extend_from_slice(&block_align.to_le_bytes());
    v.extend_from_slice(&depth.bits().to_le_bytes());

    // smpl chunk (loop points), if any.
    if let Some(body) = smpl {
        v.extend_from_slice(b"smpl");
        v.extend_from_slice(&(body.len() as u32).to_le_bytes());
        v.extend_from_slice(&body);
    }
    // cue + LIST/adtl chunks (marker positions + labels), if any.
    if let Some(body) = cue {
        v.extend_from_slice(b"cue ");
        v.extend_from_slice(&(body.len() as u32).to_le_bytes());
        v.extend_from_slice(&body);
    }
    if let Some(body) = adtl {
        v.extend_from_slice(b"LIST");
        v.extend_from_slice(&(body.len() as u32).to_le_bytes());
        v.extend_from_slice(&body);
    }

    // data chunk.
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    match depth {
        BitDepth::Pcm16 => {
            for &s in samples {
                v.extend_from_slice(&sample_to_i16(s).to_le_bytes());
            }
        }
        BitDepth::Pcm24 => {
            for &s in samples {
                let [b0, b1, b2, _] = sample_to_i24_le(s);
                v.push(b0);
                v.push(b1);
                v.push(b2);
            }
        }
        BitDepth::Float32 => {
            for &s in samples {
                v.extend_from_slice(&s.to_le_bytes());
            }
        }
    }
    Ok(v)
}

/// Encode `data` and write it to `path` (creating / truncating the file).
pub fn write_wav(path: &Path, data: &SampleData, depth: BitDepth) -> Result<(), EncodeError> {
    write_wav_with_meta(path, data, depth, &WavMeta::default())
}

/// Encode `data` with [`WavMeta`] (loop points) and write it to `path`.
pub fn write_wav_with_meta(
    path: &Path,
    data: &SampleData,
    depth: BitDepth,
    meta: &WavMeta,
) -> Result<(), EncodeError> {
    let bytes = encode_wav_with_meta(data, depth, meta)?;
    let mut f = std::fs::File::create(path)?;
    f.write_all(&bytes)?;
    Ok(())
}

/// Build the body of a `smpl` chunk (everything after the 8-byte chunk header) for
/// `loops` at `sample_rate`. Per the sampler chunk spec: a fixed 9-word header then
/// one 6-word record per loop. Loop `end` is written INCLUSIVE (`region.end - 1`).
fn smpl_chunk(loops: &[LoopRegion], sample_rate: u32) -> Vec<u8> {
    // Nanoseconds per frame — the sample period the spec asks for.
    let sample_period = (1_000_000_000f64 / sample_rate.max(1) as f64).round() as u32;
    let mut b = Vec::with_capacity(SMPL_HEADER_LEN + loops.len() * SMPL_LOOP_LEN);
    let mut w = |x: u32| b.extend_from_slice(&x.to_le_bytes());
    w(0); // manufacturer
    w(0); // product
    w(sample_period);
    w(60); // MIDI unity note (middle C) — a neutral default
    w(0); // MIDI pitch fraction
    w(0); // SMPTE format
    w(0); // SMPTE offset
    w(loops.len() as u32); // number of sample loops
    w(0); // sampler-specific data byte count
    for (i, lp) in loops.iter().enumerate() {
        w(i as u32); // cue-point identifier
        w(0); // type: 0 = forward loop
        w(lp.start); // loop start (frame)
        w(lp.end.saturating_sub(1)); // loop end (INCLUSIVE last frame)
        w(0); // fraction
        w(0); // play count: 0 = loop forever
    }
    b
}

/// Read loop regions back out of an encoded WAV's `smpl` chunk. Walks the RIFF
/// chunk list directly (independent of the audio decoder, which ignores `smpl`) and
/// converts the INCLUSIVE `smpl` loop end back to our half-open `end`. Returns empty
/// for a WAV with no `smpl` chunk or malformed bytes.
pub fn read_loop_regions(bytes: &[u8]) -> Vec<LoopRegion> {
    // Header: "RIFF" <u32 size> "WAVE" then a flat list of `<id:4><size:u32><body>`
    // chunks (each padded to an even length).
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Vec::new();
    }
    let u32_at =
        |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(pos + 4) as usize;
        let body = pos + 8;
        if body + size > bytes.len() {
            break;
        }
        if id == b"smpl" {
            return parse_smpl(&bytes[body..body + size]);
        }
        // Chunks are word-aligned: an odd size carries a pad byte.
        pos = body + size + (size & 1);
    }
    Vec::new()
}

/// Parse the body of a `smpl` chunk into loop regions (INCLUSIVE end → half-open).
fn parse_smpl(body: &[u8]) -> Vec<LoopRegion> {
    if body.len() < SMPL_HEADER_LEN {
        return Vec::new();
    }
    let u32_at = |o: usize| u32::from_le_bytes([body[o], body[o + 1], body[o + 2], body[o + 3]]);
    let n = u32_at(28) as usize; // cSampleLoops
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let base = SMPL_HEADER_LEN + i * SMPL_LOOP_LEN;
        if base + SMPL_LOOP_LEN > body.len() {
            break;
        }
        let start = u32_at(base + 8);
        let end_inclusive = u32_at(base + 12);
        out.push(LoopRegion {
            start,
            end: end_inclusive.saturating_add(1),
        });
    }
    out
}

/// Bytes per cue point in a `cue ` chunk (6 × u32).
const CUE_POINT_LEN: usize = 24;

/// Build the body of a `cue ` chunk: a count then one 24-byte record per marker. Each
/// cue point's identifier is its index (matched to a label in the `adtl` chunk) and
/// its `dwSampleOffset` (+ `dwPosition`) is the marker frame.
fn cue_chunk(markers: &[Marker]) -> Vec<u8> {
    let mut b = Vec::with_capacity(4 + markers.len() * CUE_POINT_LEN);
    b.extend_from_slice(&(markers.len() as u32).to_le_bytes()); // dwCuePoints
    for (i, m) in markers.iter().enumerate() {
        b.extend_from_slice(&(i as u32).to_le_bytes()); // dwName (identifier)
        b.extend_from_slice(&m.frame.to_le_bytes()); // dwPosition (play order)
        b.extend_from_slice(b"data"); // fccChunk — the cue is into the data chunk
        b.extend_from_slice(&0u32.to_le_bytes()); // dwChunkStart
        b.extend_from_slice(&0u32.to_le_bytes()); // dwBlockStart
        b.extend_from_slice(&m.frame.to_le_bytes()); // dwSampleOffset (the frame)
    }
    b
}

/// Build the body of a `LIST` chunk of type `adtl` (associated data): one `labl`
/// sub-chunk per marker carrying its cue-point id + null-terminated name.
fn adtl_chunk(markers: &[Marker]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(b"adtl");
    for (i, m) in markers.iter().enumerate() {
        let name = m.name.as_bytes();
        let labl_size = 4 + name.len() + 1; // dwCuePointID + text + null
        b.extend_from_slice(b"labl");
        b.extend_from_slice(&(labl_size as u32).to_le_bytes());
        b.extend_from_slice(&(i as u32).to_le_bytes()); // dwCuePointID
        b.extend_from_slice(name);
        b.push(0); // null terminator
        if labl_size % 2 == 1 {
            b.push(0); // pad each sub-chunk to an even length
        }
    }
    b
}

/// Read cue markers back from an encoded WAV — joins the `cue ` positions with their
/// `adtl`/`labl` names by cue-point id. Walks the RIFF chunk list directly (the audio
/// decoder ignores both). Returns markers sorted by frame; empty if there are none.
pub fn read_markers(bytes: &[u8]) -> Vec<Marker> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Vec::new();
    }
    let u32_at =
        |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let mut positions: Vec<(u32, u32)> = Vec::new(); // (id, frame)
    let mut labels: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(pos + 4) as usize;
        let body = pos + 8;
        if body + size > bytes.len() {
            break;
        }
        if id == b"cue " {
            positions = parse_cue(&bytes[body..body + size]);
        } else if id == b"LIST" && size >= 4 && &bytes[body..body + 4] == b"adtl" {
            labels = parse_adtl(&bytes[body..body + size]);
        }
        pos = body + size + (size & 1);
    }
    let mut out: Vec<Marker> = positions
        .into_iter()
        .map(|(id, frame)| Marker {
            frame,
            name: labels.get(&id).cloned().unwrap_or_default(),
        })
        .collect();
    out.sort_by_key(|m| m.frame);
    out
}

/// Parse a `cue ` chunk body into `(id, sample_offset)` pairs.
fn parse_cue(body: &[u8]) -> Vec<(u32, u32)> {
    if body.len() < 4 {
        return Vec::new();
    }
    let u32_at = |o: usize| u32::from_le_bytes([body[o], body[o + 1], body[o + 2], body[o + 3]]);
    let n = u32_at(0) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let base = 4 + i * CUE_POINT_LEN;
        if base + CUE_POINT_LEN > body.len() {
            break;
        }
        out.push((u32_at(base), u32_at(base + 20))); // id, dwSampleOffset
    }
    out
}

/// Parse a `LIST/adtl` chunk body into `id → label`.
fn parse_adtl(body: &[u8]) -> std::collections::HashMap<u32, String> {
    let mut map = std::collections::HashMap::new();
    let u32_at = |o: usize| u32::from_le_bytes([body[o], body[o + 1], body[o + 2], body[o + 3]]);
    let mut pos = 4; // skip the "adtl" tag
    while pos + 8 <= body.len() {
        let sub_id = &body[pos..pos + 4];
        let sub_size = u32_at(pos + 4) as usize;
        let sub_body = pos + 8;
        if sub_body + sub_size > body.len() {
            break;
        }
        if sub_id == b"labl" && sub_size >= 4 {
            let id = u32_at(sub_body);
            let text = &body[sub_body + 4..sub_body + sub_size];
            let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
            map.insert(id, String::from_utf8_lossy(&text[..end]).into_owned());
        }
        pos = sub_body + sub_size + (sub_size & 1);
    }
    map
}

/// Quantise a `[-1.0, 1.0]` sample to signed 16-bit (round-to-nearest, clamped).
#[inline]
fn sample_to_i16(s: Sample) -> i16 {
    // 32767 (not 32768) so +1.0 maps to the max representable value and the
    // scale is symmetric — the standard audio-domain convention.
    let scaled = (s.clamp(-1.0, 1.0) * 32_767.0).round();
    scaled as i16
}

/// Quantise a `[-1.0, 1.0]` sample to signed 24-bit, returned as 4 LE bytes
/// (the 4th is the sign-extension byte the caller drops).
#[inline]
fn sample_to_i24_le(s: Sample) -> [u8; 4] {
    let scaled = (s.clamp(-1.0, 1.0) * 8_388_607.0).round();
    (scaled as i32).to_le_bytes()
}

#[cfg(test)]
mod tests {
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
        let src =
            SampleData::from_interleaved(vec![0.1, -0.2, 0.3, -0.4], AudioFormat::mono(44_100));
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
}
