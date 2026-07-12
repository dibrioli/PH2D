//! **Incremental Opus decode** — a packet at a time, from a file (ADR-0118).
//!
//! The sibling of `ph2d-audio-decode`'s `Reader`, for the one codec Symphonia cannot read (which is
//! why this crate exists at all — see `decoder.rs`).
//!
//! Ogg is *made* for this: a `PacketReader` hands over one packet at a time and never needs the
//! whole file. The streaming producer decodes each into the ring and moves on, so a forty-minute
//! ambient bed costs the ring, not forty minutes of `f32`.

use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use ogg::PacketReader;

use crate::DecodeError;
use crate::decoder::{Decoder, parse_head};

/// Everything `build` hands back to re-arm a reader: the Ogg packet reader, the libopus decoder,
/// the channel count, and the pre-skip still to be discarded.
type Armed = (PacketReader<BufReader<File>>, Decoder, usize, usize);

/// An Opus file being decoded a packet at a time.
pub struct Reader {
    path: PathBuf,
    packets: PacketReader<BufReader<File>>,
    stream: Decoder,
    channels: usize,
    /// Frames of the encoder's own delay still to be thrown away. Streaming or not, the pre-skip is
    /// not optional: keep it and every loop of the file opens with a gap nobody put there.
    skip: usize,
    /// Reused across packets — allocated once.
    buf: Vec<f32>,
}

impl Reader {
    /// Open an Ogg Opus file for incremental decoding.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DecodeError> {
        let path = path.as_ref().to_path_buf();
        let (packets, stream, channels, skip) = Self::build(&path)?;
        Ok(Self {
            path,
            packets,
            stream,
            channels,
            skip,
            buf: Vec::new(),
        })
    }

    fn build(path: &Path) -> Result<Armed, DecodeError> {
        let file = File::open(path)?;
        let mut packets = PacketReader::new(BufReader::new(file));

        let head = packets
            .read_packet()?
            .ok_or(DecodeError::Malformed("no packets"))?;
        let (channels, pre_skip) = parse_head(&head.data)?;
        let tags = packets
            .read_packet()?
            .ok_or(DecodeError::Malformed("no comment header"))?;
        if !tags.data.starts_with(b"OpusTags") {
            return Err(DecodeError::Malformed("second packet is not OpusTags"));
        }
        let stream = Decoder::new(channels)?;
        Ok((packets, stream, channels, usize::from(pre_skip)))
    }

    /// Opus is a 48 kHz codec. Always.
    pub fn sample_rate(&self) -> u32 {
        crate::OPUS_RATE
    }

    /// Channels the file declares (1 or 2 — `parse_head` rejects anything else).
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Decode the next packet. `Ok(None)` at the end of the file.
    ///
    /// Valid until the next call — the reader's own buffer, so a long file allocates once.
    pub fn next_packet(&mut self) -> Result<Option<&[f32]>, DecodeError> {
        loop {
            let Some(p) = self.packets.read_packet()? else {
                return Ok(None);
            };
            let pcm = self.stream.decode(&p.data)?;

            // Drop the encoder's delay off the front, however many packets it spans.
            let mut at = 0usize;
            if self.skip > 0 {
                let drop_samples = (self.skip * self.channels).min(pcm.len());
                self.skip -= drop_samples / self.channels;
                at = drop_samples;
            }
            if at >= pcm.len() {
                continue; // this packet was entirely pre-skip; ask for the next one
            }

            self.buf.clear();
            self.buf
                .extend(pcm[at..].iter().map(|v| f32::from(*v) / 32_768.0));
            return Ok(Some(&self.buf));
        }
    }

    /// Start the file over — how a looping stream loops. Re-opens (exact for every format) rather
    /// than seeking, and the pre-skip is re-armed with it.
    pub fn rewind(&mut self) -> Result<(), DecodeError> {
        let (packets, stream, channels, skip) = Self::build(&self.path)?;
        self.packets = packets;
        self.stream = stream;
        self.channels = channels;
        self.skip = skip;
        Ok(())
    }
}

/// Whether `path` looks like an Ogg Opus file — read the first bytes, not the extension.
pub fn is_opus_file(path: impl AsRef<Path>) -> bool {
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let mut head = [0u8; 128];
    use std::io::Read;
    let n = f.read(&mut head).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(0));
    crate::is_opus(&head[..n])
}
