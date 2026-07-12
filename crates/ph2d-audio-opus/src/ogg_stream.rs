//! The container: Opus packets → a real `.opus` file (RFC 7845, "Ogg Encapsulation for Opus").
//!
//! An encoder produces packets. A *file* is more than that, and the difference is where a
//! naive Opus writer produces bytes nothing will play:
//!
//! - **`OpusHead`** — the identification header, alone on the first page. It carries the
//!   channel count, the original sample rate (informational — the audio is 48 kHz regardless),
//!   and the **pre-skip**.
//! - **`OpusTags`** — the comment header, alone on the second page. Even with nothing to say,
//!   it is mandatory: a decoder that does not find it will refuse the stream.
//! - **Granule positions** — a per-page count, in 48 kHz samples, of how much audio has been
//!   decoded by the end of that page. This is where the file's *duration* actually lives; get
//!   it wrong and players report the wrong length, seek to the wrong place, or stop early.
//!
//! ## Pre-skip, and why it matters more than it looks
//!
//! The encoder consumes some audio before it emits anything (its algorithmic delay). Those
//! leading samples come back out of the decoder as silence. `pre_skip` tells the decoder how
//! many to discard — and the granule positions are offset by it, so the final one still equals
//! the true sample count. Forget it, and every file you export begins with a small gap that
//! nobody put there and everybody hears.

use ogg::PacketWriter;
use ogg::writing::PacketWriteEndInfo;

use crate::encoder::Encoder;
use crate::{EncodeError, FRAME};

/// Ogg logical-stream serial. A single-stream file may use any value; a fixed one keeps the
/// output **deterministic**, which is what lets a test compare two encodes and what stops a
/// rebuilt asset from looking changed to git when nothing about it changed.
const SERIAL: u32 = 0x5048_3244; // "PH2D"

/// Encode `pcm` (interleaved i16 at 48 kHz) into a complete Ogg Opus file.
pub fn write_ogg_opus(
    pcm: &[i16],
    channels: usize,
    original_rate: u32,
    bitrate: i32,
) -> Result<Vec<u8>, EncodeError> {
    let mut enc = Encoder::new(channels, bitrate)?;
    let pre_skip = enc.lookahead()?;

    let mut out = Vec::new();
    {
        let mut w = PacketWriter::new(&mut out);

        // Header page 1: OpusHead, alone on its page (RFC 7845 §5.1).
        w.write_packet(
            opus_head(channels, pre_skip, original_rate),
            SERIAL,
            PacketWriteEndInfo::EndPage,
            0,
        )?;
        // Header page 2: OpusTags, alone on its page. Mandatory even when empty.
        w.write_packet(opus_tags(), SERIAL, PacketWriteEndInfo::EndPage, 0)?;

        // The audio. One packet per 20 ms frame; the final frame is zero-padded, and the last
        // granule position tells the decoder where the real audio actually ended, so the
        // padding is never heard.
        let frames = pcm.len() / channels.max(1);
        let total_frames = frames.div_ceil(FRAME);
        for i in 0..total_frames {
            let start = i * FRAME * channels;
            let end = ((i + 1) * FRAME * channels).min(pcm.len());
            let mut block = vec![0i16; FRAME * channels];
            block[..end - start].copy_from_slice(&pcm[start..end]);

            let packet = enc.encode(&block, FRAME)?;
            let last = i + 1 == total_frames;
            // The granule position counts DECODED samples, so it includes the pre-skip. On the
            // last page it is the true length + pre_skip: everything past that is padding, and
            // the decoder drops it.
            let decoded = if last {
                frames as u64
            } else {
                ((i + 1) * FRAME) as u64
            };
            let absgp = decoded + u64::from(pre_skip);
            let info = if last {
                PacketWriteEndInfo::EndStream
            } else {
                PacketWriteEndInfo::NormalPacket
            };
            w.write_packet(packet, SERIAL, info, absgp)?;
        }
        // A clip with no audio at all still needs its stream ended, or the file is truncated.
        if total_frames == 0 {
            w.write_packet(
                Vec::new(),
                SERIAL,
                PacketWriteEndInfo::EndStream,
                u64::from(pre_skip),
            )?;
        }
    }
    Ok(out)
}

/// The `OpusHead` identification header (RFC 7845 §5.1).
fn opus_head(channels: usize, pre_skip: u16, original_rate: u32) -> Vec<u8> {
    let mut h = Vec::with_capacity(19);
    h.extend_from_slice(b"OpusHead");
    h.push(1); // version
    h.push(channels as u8);
    h.extend_from_slice(&pre_skip.to_le_bytes());
    // The ORIGINAL rate, which is informational only: the audio in the file is 48 kHz whatever
    // this says. It is recorded so a tool can tell where the clip came from — and lying here
    // would not change a single sample, which is exactly why it is easy to get wrong.
    h.extend_from_slice(&original_rate.to_le_bytes());
    h.extend_from_slice(&0i16.to_le_bytes()); // output gain: none
    h.push(0); // mapping family 0: mono/stereo, the channels are what they say they are
    h
}

/// The `OpusTags` comment header (RFC 7845 §5.2): a vendor string and no comments.
fn opus_tags() -> Vec<u8> {
    const VENDOR: &[u8] = b"PH2D";
    let mut t = Vec::with_capacity(8 + 4 + VENDOR.len() + 4);
    t.extend_from_slice(b"OpusTags");
    t.extend_from_slice(&(VENDOR.len() as u32).to_le_bytes());
    t.extend_from_slice(VENDOR);
    t.extend_from_slice(&0u32.to_le_bytes()); // zero user comments
    t
}
