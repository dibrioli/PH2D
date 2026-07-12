//! `ph2d-audio-opus` — write a [`ph2d_audio::SampleData`] out as an **Opus** file (ADR-0116).
//!
//! ## Why this is its own crate
//!
//! `ph2d-audio-encode` declares `#![forbid(unsafe_code)]`, and the only Rust Opus encoder that
//! does not drag a system `libopus` into the CI of three operating systems is
//! `unsafe-libopus` — libopus transpiled by `c2rust` — which exposes the raw C ABI, and is
//! therefore `unsafe` by construction.
//!
//! That looked like a dilemma (give up the `forbid`, or take the system dependency) and it is
//! a false one. The `unsafe` lives **here**, in exactly one module (`encoder`), behind a safe
//! API, with every block annotated. No raw pointer crosses this crate's boundary; nothing
//! above it needs an `unsafe` block; and `ph2d-audio-encode` keeps its guarantee. This is the
//! idiomatic division of the Rust ecosystem, and it is the whole reason the crate exists.
//!
//! ## What an Opus *file* actually is
//!
//! Not just packets. A `.opus` is Opus **encapsulated in Ogg**: pages with CRCs, a granule
//! position per page, an `OpusHead` identification header and an `OpusTags` comment header.
//! `opus_encode` hands back one packet per frame and nothing else — so the container is half
//! the work, and it is done with the `ogg` crate rather than by hand (a mis-computed CRC is a
//! corrupt file, and there is a correct implementation to port instead of reinvent).
//!
//! ## The two things that go silently wrong, and are handled
//!
//! - **Opus is a 48 kHz codec.** It accepts 48/24/16/12/8 kHz and nothing else. A 44.1 kHz clip
//!   is **resampled** to 48 kHz on the way in — said out loud here, because a silent resample
//!   is precisely the class of bug the W5 audit caught twice.
//! - **The encoder has a lookahead**, and the file has to declare it as `pre_skip`. A decoder
//!   that is not told plays that much silence at the head of the clip, every time.

#![forbid(unsafe_op_in_unsafe_fn)]

mod decoder;
mod encoder;
mod ogg_stream;
mod reader;

use ph2d_audio::SampleData;

pub use decoder::{decode_opus, is_opus};
pub use ogg_stream::write_ogg_opus;
pub use reader::{Reader, is_opus_file};

/// Opus's native rate. It accepts a few others; this crate feeds it only this one, and
/// resamples anything else into it.
pub const OPUS_RATE: u32 = 48_000;

/// Samples per channel in one encoded frame — 20 ms at 48 kHz, the standard Opus frame and the
/// one every decoder handles without thinking.
pub const FRAME: usize = 960;

/// The default bitrate: 96 kbit/s, which is transparent-ish for game SFX and dialogue and
/// still roughly a sixth of 16-bit WAV.
pub const DEFAULT_BITRATE: i32 = 96_000;

/// What can go wrong READING an Opus file.
///
/// This crate decodes as well as encodes, which was not the plan. Symphonia — our decoder
/// everywhere else — **has no Opus decoder** (its `all-codecs` is aac/adpcm/alac/flac/mp1-3/
/// pcm/vorbis), which was found by writing the round-trip gate and watching it fail. Without
/// this, the editor would export a format it could not re-open. It costs no new dependency:
/// the two crates already here decode too.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// libopus refused. The `i32` is its own error code (negative).
    #[error("opus decoder failed (code {0})")]
    Decoder(i32),
    /// The Ogg stream could not be read.
    #[error("ogg container: {0}")]
    Container(#[from] ogg::OggReadError),
    /// The file is not a well-formed Ogg Opus stream.
    #[error("malformed opus file: {0}")]
    Malformed(&'static str),
    /// The file could not be read (streaming reads from disk, not from memory — ADR-0118).
    #[error("reading the opus file: {0}")]
    Io(#[from] std::io::Error),
}

/// What can go wrong writing an Opus file.
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    /// libopus refused. The `i32` is its own error code (negative).
    #[error("opus encoder failed (code {0})")]
    Encoder(i32),
    /// A frame of the wrong length reached the encoder — a bug here, not bad input.
    #[error("internal: frame of {got} samples, expected {want}")]
    Frame { got: usize, want: usize },
    /// The Ogg container could not be written.
    #[error("ogg container: {0}")]
    Container(#[from] std::io::Error),
    /// More channels than Opus's basic (non-ambisonic) mapping handles.
    #[error("opus supports mono and stereo; this clip has {0} channels")]
    Channels(usize),
}

/// Encode `data` as a complete `.opus` file (Ogg-encapsulated Opus).
///
/// `bitrate` is in bits per second; [`DEFAULT_BITRATE`] is a sensible one. The clip is
/// resampled to 48 kHz if it is not there already, because Opus does not do anything else.
pub fn encode_opus(data: &SampleData, bitrate: i32) -> Result<Vec<u8>, EncodeError> {
    let channels = data.format().channel_count().max(1);
    if channels > 2 {
        return Err(EncodeError::Channels(channels));
    }
    let rate = data.format().sample_rate;
    let pcm = to_opus_pcm(data.samples(), channels, rate);
    write_ogg_opus(&pcm, channels, rate, bitrate)
}

/// Convert a clip to what the encoder eats: interleaved **i16** at **48 kHz**.
///
/// Two conversions, and both are honest about what they cost:
///
/// - **Rate.** Opus is a 48 kHz codec. A 44.1 kHz clip is resampled — linearly, which is enough
///   here: whatever a linear resampler smears sits far below what a lossy codec is about to
///   throw away anyway. (The pitch shifter needs better; this does not.)
/// - **Depth.** libopus's `opus_encode` takes `i16`. The float is scaled and **clamped**, so a
///   clip that was over full scale comes out at full scale rather than wrapping — a wrap would
///   turn a hot sample into a loud click of the opposite sign.
fn to_opus_pcm(samples: &[f32], channels: usize, rate: u32) -> Vec<i16> {
    let frames = samples.len() / channels;
    let quantise = |v: f32| (v.clamp(-1.0, 1.0) * 32_767.0).round() as i16;

    if rate == OPUS_RATE || rate == 0 || frames == 0 {
        return samples.iter().map(|v| quantise(*v)).collect();
    }
    let ratio = f64::from(OPUS_RATE) / f64::from(rate);
    let out_frames = ((frames as f64) * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_frames * channels);
    for f in 0..out_frames {
        let src = f as f64 / ratio;
        let i = src.floor() as usize;
        let t = (src - i as f64) as f32;
        for c in 0..channels {
            let a = samples.get(i * channels + c).copied().unwrap_or(0.0);
            let b = samples.get((i + 1) * channels + c).copied().unwrap_or(a);
            out.push(quantise(a + (b - a) * t));
        }
    }
    out
}

#[cfg(test)]
mod tests;
