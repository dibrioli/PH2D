#![forbid(unsafe_code)]
//! `ph2d-audio-decode` — decode audio files into [`ph2d_audio::SampleData`] via
//! Symphonia. **Load-time only** (never on the RT audio thread); kept OUT of the
//! lean `ph2d-audio` core so Symphonia's dependency tree never reaches the mixer.
//!
//! Royalty-free / patent-expired codecs only (HR-1 FFI criterion #6): WAV
//! (PCM/ADPCM), AIFF, FLAC, Ogg Vorbis, MP3 (patents expired 2017). AAC / ALAC /
//! MP4 are deliberately NOT enabled.

mod reader;
pub use reader::Reader;

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

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
    // ⚠️ symphonia 0.6: `Probe::format` -> `Probe::probe`, devolvendo o `FormatReader` directo.
    let mut format = symphonia::default::get_probe().probe(
        &Hint::new(),
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;
    // ⚠️ 0.6: `codec_params` e' `Option<CodecParameters>` e o enum distingue audio de video/legenda.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.as_ref().is_some_and(|p| p.is_audio()))
        .ok_or(DecodeError::NoTrack)?;
    let track_id = track.id;
    let audio_params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or(DecodeError::NoTrack)?;
    let mut rate: Option<u32> = audio_params.sample_rate;
    let mut channels: Option<usize> = audio_params
        .channels
        .as_ref()
        .map(symphonia::core::audio::Channels::count);
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(audio_params, &AudioDecoderOptions::default())?;

    let mut interleaved: Vec<f32> = Vec::new();
    // ⚠️ O `SampleBuffer<f32>` sumiu na 0.6; o `copy_to_vec_interleaved` reenche este Vec
    // (clear + extend), então a capacidade sobrevive entre pacotes.
    let mut buf: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            // ⚠️ 0.6: fim do media e' `Ok(None)` explicito (era `IoError(UnexpectedEof)`).
            Ok(None) => break,
            Ok(Some(p)) => p,
            // Clean end-of-stream (Symphonia signals EOF as an IO error).
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(e.into()),
        };
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // ⚠️ 0.6: o `AudioSpec` deixou de ser `Copy` (ele carrega o mapa de canais).
                // Lê-se por referência — nada aqui precisa de posse.
                let spec = decoded.spec();
                rate.get_or_insert(spec.rate());
                channels.get_or_insert(spec.channels().count());
                decoded.copy_to_vec_interleaved(&mut buf);
                interleaved.extend_from_slice(&buf);
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
