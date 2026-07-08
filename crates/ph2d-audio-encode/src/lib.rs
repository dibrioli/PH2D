#![forbid(unsafe_code)]
//! `ph2d-audio-encode` — write a [`ph2d_audio::SampleData`] out to a file.
//!
//! **Save-time only** (never on the RT audio thread): editing produces a fresh
//! interleaved-`f32` buffer, and this crate serialises it to disk so an edit is
//! persistable. Kept OUT of the lean `ph2d-audio` core so no writer dependency
//! reaches the mixer — mirroring how `ph2d-audio-decode` isolates Symphonia.
//!
//! ## Scope (W1)
//!
//! Canonical RIFF/WAVE: PCM 16-bit, PCM 24-bit, and IEEE-float 32-bit. These
//! round-trip cleanly through `ph2d-audio-decode` (Symphonia `wav`+`pcm`). Loop
//! (`smpl`) and marker (`cue`) chunks, plus Ogg Vorbis / Opus, arrive with the
//! game asset-prep wave (`docs/Audio/02_plano_implementacao_completo.md` §6).

use std::io::Write;
use std::path::Path;

use ph2d_audio::{Sample, SampleData};

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

/// Encode `data` to an in-memory canonical RIFF/WAVE byte buffer.
///
/// Interleaving and channel count come straight from the clip's format; the
/// float samples are quantised (with clamp to `[-1.0, 1.0]`) for the integer
/// depths and written verbatim for [`BitDepth::Float32`].
pub fn encode_wav(data: &SampleData, depth: BitDepth) -> Result<Vec<u8>, EncodeError> {
    let format = data.format();
    let channels = format.channel_count();
    let bytes_per = depth.bytes_per_sample();
    let samples = data.samples();

    let data_len_usize = samples.len() * bytes_per;
    // RIFF size is `36 + data_len`; both must fit u32.
    let data_len = u32::try_from(data_len_usize).map_err(|_| EncodeError::TooLarge)?;
    let riff_len = 36u32.checked_add(data_len).ok_or(EncodeError::TooLarge)?;

    let mut v = Vec::with_capacity(44 + data_len_usize);
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
    let bytes = encode_wav(data, depth)?;
    let mut f = std::fs::File::create(path)?;
    f.write_all(&bytes)?;
    Ok(())
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
    fn header_is_well_formed() {
        let src = SampleData::from_interleaved(vec![0.0; 10], AudioFormat::mono(22_050));
        let bytes = encode_wav(&src, BitDepth::Pcm16).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");
        // data_len = 10 samples * 2 bytes.
        assert_eq!(u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]), 20);
    }
}
