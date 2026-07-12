//! Reading an Opus file back — the other half of the `unsafe` containment, and a feature we
//! did not set out to build.
//!
//! ## Why this exists
//!
//! The plan was to encode Opus and let our existing decoder (Symphonia) read it back — both as
//! the acceptance gate (ADR-0116 §4.1) and so that an exported asset could be re-imported.
//! **Symphonia 0.5 has no Opus decoder.** Its `all-codecs` feature is aac, adpcm, alac, flac,
//! mp1/2/3, pcm and vorbis; Opus is simply not among them. That was discovered by writing the
//! gate and watching it fail with `unsupported codec` — not by reading a changelog, which is
//! the only reason it was discovered before shipping.
//!
//! Which left the feature in an indefensible shape: the editor would export a format it could
//! not open. Export a variation set as `.opus`, and the tool that made it can no longer read
//! it.
//!
//! The fix costs **nothing new**: the two crates already added for encoding do decoding too —
//! `unsafe-libopus` has `opus_decode`, and `ogg` has a `PacketReader`. So the same containment
//! holds (the `unsafe` stays here, behind a safe API), and the module gains a real round trip.
//!
//! ## What has to be right
//!
//! - **Pre-skip.** The header says how many leading samples the encoder's own delay produced.
//!   They are decoded and then **thrown away** — keep them and every file opens with a gap of
//!   silence that was never in the source.
//! - **The final granule position.** It is where the audio actually ends; the last packet is
//!   zero-padded out to a whole frame, and the padding must not survive the trip.

use ogg::PacketReader;
use unsafe_libopus::{OpusDecoder, opus_decode, opus_decoder_create, opus_decoder_destroy};

use crate::{DecodeError, OPUS_RATE};
use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};

/// The most samples per channel one Opus packet can decode to (120 ms at 48 kHz — the largest
/// frame the format allows).
const MAX_FRAME: usize = 5_760;

/// A libopus decoder. Owns the allocation; frees it on drop. Mirror of `encoder::Encoder`, and
/// the second (and last) place `unsafe` appears in this crate.
struct Decoder {
    st: *mut OpusDecoder,
    channels: usize,
}

impl Decoder {
    fn new(channels: usize) -> Result<Self, DecodeError> {
        let mut err: i32 = 0;
        // SAFETY: `channels` is 1 or 2 (checked by the caller against the file's header), 48 000
        // is a rate libopus accepts, and `err` is a live local we own for the call. The returned
        // pointer is checked before any use.
        let st = unsafe { opus_decoder_create(OPUS_RATE as i32, channels as i32, &mut err) };
        if st.is_null() || err < 0 {
            return Err(DecodeError::Decoder(err));
        }
        Ok(Self { st, channels })
    }

    /// Decode one packet. Returns interleaved i16, `samples_per_channel × channels` long.
    fn decode(&mut self, packet: &[u8]) -> Result<Vec<i16>, DecodeError> {
        let mut out = vec![0i16; MAX_FRAME * self.channels];
        // SAFETY: `self.st` is a live decoder (the type cannot hold a null one). `packet` is a
        // slice we hold for the whole call, and its true length is passed. `out` is ours, and we
        // tell libopus its real capacity **in samples per channel**, so it cannot write past the
        // end. The return value is how many samples per channel it actually wrote.
        let n = unsafe {
            opus_decode(
                self.st,
                packet.as_ptr(),
                packet.len() as i32,
                out.as_mut_ptr(),
                MAX_FRAME as i32,
                0, // no forward error correction: this is a file, not a network stream
            )
        };
        if n < 0 {
            return Err(DecodeError::Decoder(n));
        }
        out.truncate(n as usize * self.channels);
        Ok(out)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: `self.st` came from `opus_decoder_create` and has not been freed — this type is
        // its sole owner and frees exactly once, here.
        unsafe { opus_decoder_destroy(self.st) };
    }
}

/// Decode a complete Ogg Opus file back to samples.
pub fn decode_opus(bytes: &[u8]) -> Result<SampleData, DecodeError> {
    let mut reader = PacketReader::new(std::io::Cursor::new(bytes));

    // Packet 1 is OpusHead. Everything the decode needs is in it.
    let head = reader
        .read_packet()?
        .ok_or(DecodeError::Malformed("no packets"))?;
    let (channels, pre_skip) = parse_head(&head.data)?;
    // Packet 2 is OpusTags — mandatory, and of no interest here beyond being present.
    let tags = reader
        .read_packet()?
        .ok_or(DecodeError::Malformed("no comment header"))?;
    if !tags.data.starts_with(b"OpusTags") {
        return Err(DecodeError::Malformed("second packet is not OpusTags"));
    }

    let mut dec = Decoder::new(channels)?;
    let mut pcm: Vec<i16> = Vec::new();
    let mut last_granule = 0u64;
    while let Some(p) = reader.read_packet()? {
        last_granule = p.absgp_page();
        pcm.extend_from_slice(&dec.decode(&p.data)?);
    }

    // The header's pre-skip is the encoder's own delay, decoded as silence. Drop it, or every
    // file opens with a gap nobody put there.
    let skip = usize::from(pre_skip) * channels;
    if skip < pcm.len() {
        pcm.drain(..skip);
    } else {
        pcm.clear();
    }
    // The last packet was padded out to a whole frame. The final granule position says where the
    // audio really ended — trust it over the buffer's length.
    let real_frames = (last_granule.saturating_sub(u64::from(pre_skip))) as usize;
    let want = real_frames * channels;
    if want < pcm.len() {
        pcm.truncate(want);
    }

    let samples: Vec<f32> = pcm.iter().map(|v| f32::from(*v) / 32_768.0).collect();
    Ok(SampleData::from_interleaved(
        samples,
        AudioFormat {
            sample_rate: OPUS_RATE,
            channels: if channels == 2 {
                ChannelLayout::Stereo
            } else {
                ChannelLayout::Mono
            },
        },
    ))
}

/// Read the `OpusHead` identification header: channel count and pre-skip (RFC 7845 §5.1).
fn parse_head(data: &[u8]) -> Result<(usize, u16), DecodeError> {
    if data.len() < 19 || !data.starts_with(b"OpusHead") {
        return Err(DecodeError::Malformed("not an OpusHead"));
    }
    let channels = usize::from(data[9]);
    if channels == 0 || channels > 2 {
        return Err(DecodeError::Malformed(
            "channel mapping is not mono or stereo",
        ));
    }
    let pre_skip = u16::from_le_bytes([data[10], data[11]]);
    Ok((channels, pre_skip))
}

/// Whether these bytes look like an Ogg Opus file — cheap enough to try on every import, and it
/// is what lets the app route a `.opus` here instead of to Symphonia, which cannot read it.
pub fn is_opus(bytes: &[u8]) -> bool {
    // "OggS" then, within the first page, the OpusHead magic.
    bytes.starts_with(b"OggS") && bytes.len() > 40 && find_head(&bytes[..bytes.len().min(128)])
}

fn find_head(window: &[u8]) -> bool {
    window.windows(8).any(|w| w == b"OpusHead")
}
