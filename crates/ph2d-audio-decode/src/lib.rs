#![forbid(unsafe_code)]
//! `ph2d-audio-decode` — decode audio files into [`ph2d_audio::SampleData`] via
//! Symphonia. **Load-time only** (never on the RT audio thread); kept OUT of the
//! lean `ph2d-audio` core so Symphonia's dependency tree never reaches the mixer.
//!
//! Royalty-free / patent-expired codecs only (HR-1 FFI criterion #6): WAV
//! (PCM/ADPCM), AIFF, FLAC, Ogg Vorbis, MP3 (patents expired 2017). AAC / ALAC /
//! MP4 are deliberately NOT enabled.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Errors from decoding an audio buffer.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// The underlying Symphonia decode/demux failed.
    #[error("audio decode failed: {0}")]
    Symphonia(#[from] SymphoniaError),
    /// The container held no decodable audio track.
    #[error("no decodable audio track in the input")]
    NoTrack,
    /// The stream never revealed a sample rate.
    #[error("could not determine the sample rate")]
    UnknownRate,
    /// Decoding produced zero frames.
    #[error("decoded audio was empty")]
    Empty,
}

/// Decode an in-memory audio file into interleaved `f32` PCM. Sources with more
/// than two channels are down-mixed to stereo (first two channels), matching
/// [`SampleData`]'s mono/stereo layouts.
pub fn decode(bytes: &[u8]) -> Result<SampleData, DecodeError> {
    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    let probed = symphonia::default::get_probe().format(
        &Hint::new(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(DecodeError::NoTrack)?;
    let track_id = track.id;
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut rate: Option<u32> = track.codec_params.sample_rate;
    let mut channels: Option<usize> = track.codec_params.channels.map(|c| c.count());
    let mut interleaved: Vec<f32> = Vec::new();
    let mut buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            // Clean end-of-stream (Symphonia signals EOF as an IO error).
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(e.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                rate.get_or_insert(spec.rate);
                channels.get_or_insert(spec.channels.count());
                let sample_buf = buf.get_or_insert_with(|| {
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, spec)
                });
                sample_buf.copy_interleaved_ref(decoded);
                interleaved.extend_from_slice(sample_buf.samples());
            }
            // A single corrupt packet shouldn't abort the whole decode.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }

    let rate = rate.ok_or(DecodeError::UnknownRate)?;
    if interleaved.is_empty() {
        return Err(DecodeError::Empty);
    }
    let (samples, layout) = to_mono_or_stereo(interleaved, channels.unwrap_or(2).max(1));
    Ok(SampleData::from_interleaved(
        samples,
        AudioFormat::new(rate, layout),
    ))
}

/// Collapse N-channel interleaved PCM to mono (N==1) or stereo (N>=2 → first two
/// channels).
fn to_mono_or_stereo(interleaved: Vec<f32>, src_ch: usize) -> (Vec<f32>, ChannelLayout) {
    match src_ch {
        1 => (interleaved, ChannelLayout::Mono),
        2 => (interleaved, ChannelLayout::Stereo),
        n => {
            let frames = interleaved.len() / n;
            let mut out = Vec::with_capacity(frames * 2);
            for f in 0..frames {
                out.push(interleaved[f * n]);
                out.push(interleaved[f * n + 1]);
            }
            (out, ChannelLayout::Stereo)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal PCM16 mono WAV writer — a self-contained round-trip fixture
    /// (no external asset file needed).
    fn wav_mono16(rate: u32, samples: &[i16]) -> Vec<u8> {
        let data_len = (samples.len() * 2) as u32;
        let mut v = Vec::with_capacity(44 + data_len as usize);
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&(36 + data_len).to_le_bytes());
        v.extend_from_slice(b"WAVE");
        v.extend_from_slice(b"fmt ");
        v.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
        v.extend_from_slice(&1u16.to_le_bytes()); // format = PCM
        v.extend_from_slice(&1u16.to_le_bytes()); // channels = mono
        v.extend_from_slice(&rate.to_le_bytes());
        v.extend_from_slice(&(rate * 2).to_le_bytes()); // byte rate
        v.extend_from_slice(&2u16.to_le_bytes()); // block align
        v.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        v.extend_from_slice(b"data");
        v.extend_from_slice(&data_len.to_le_bytes());
        for &s in samples {
            v.extend_from_slice(&s.to_le_bytes());
        }
        v
    }

    #[test]
    fn decodes_pcm16_wav_round_trip() {
        // A ramp so we can confirm samples arrive in order.
        let src: Vec<i16> = (0..100).map(|i| (i * 300 - 15_000) as i16).collect();
        let wav = wav_mono16(24_000, &src);
        let data = decode(&wav).expect("decode wav");
        assert_eq!(data.format().sample_rate, 24_000);
        assert_eq!(data.format().channels, ChannelLayout::Mono);
        assert_eq!(data.frame_count(), 100);
        let s = data.samples();
        // i16 → f32 normalizes by 32768; endpoints must match within tolerance.
        assert!((s[0] - src[0] as f32 / 32_768.0).abs() < 1e-3);
        assert!((s[99] - src[99] as f32 / 32_768.0).abs() < 1e-3);
    }

    #[test]
    fn rejects_non_audio() {
        assert!(decode(b"this is definitely not an audio file").is_err());
    }
}
