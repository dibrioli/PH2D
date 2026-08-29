//! **Incremental decode** — one packet at a time, from a file on disk (ADR-0118).
//!
//! [`decode`](crate::decode) reads a whole file into a whole buffer. That is the right shape for
//! the editor, which *opens* a clip, and the wrong one for a game, which *plays* one: a
//! three-minute song costs 65.9 MB decoded, against HR-13's 30 MB for all audio on iPad.
//!
//! This is the other shape. It reads **from the file**, not from bytes already in memory — because
//! holding the file's bytes to avoid holding its samples would just be trading one 66 MB buffer for
//! another (a WAV is barely smaller decoded than encoded). The memory a streaming voice costs is
//! its ring, and the ring is a few tens of kilobytes.
//!
//! It lives here, and not in the streaming crate, because **this crate already owns the codec
//! list** (royalty-free / patent-expired only, HR-1). A second `symphonia` dependency configured
//! somewhere else would drift, and then the formats you can *stream* would quietly differ from the
//! formats you can *open* — a difference nobody would notice until an asset failed on one path and
//! worked on the other.

use std::fs::File;
use std::path::{Path, PathBuf};

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::DecodeError;

/// Everything `build` has to hand back to re-arm a reader: demuxer, decoder, track, rate, channels.
type Armed = (
    Box<dyn FormatReader>,
    Box<dyn AudioDecoder>,
    u32,
    u32,
    usize,
);

/// A file being decoded a packet at a time. Never holds more than one packet of audio.
pub struct Reader {
    path: PathBuf,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    rate: u32,
    channels: usize,
    /// Reused across packets — allocated on the first one and never again.
    ///
    /// ⚠️ Era um `SampleBuffer<f32>` até ao symphonia 0.6, que **removeu** esse tipo. O substituto
    /// é um `Vec` nosso que a `copy_to_vec_interleaved` reenche: ela faz `clear()` + `extend`, então
    /// a capacidade sobrevive entre pacotes e a promessa de "aloca uma vez" continua a valer.
    buf: Vec<f32>,
}

impl Reader {
    /// Open `path` for incremental decoding.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DecodeError> {
        let path = path.as_ref().to_path_buf();
        let (format, decoder, track_id, rate, channels) = Self::build(&path)?;
        Ok(Self {
            path,
            format,
            decoder,
            track_id,
            rate,
            channels,
            buf: Vec::new(),
        })
    }

    fn build(path: &Path) -> Result<Armed, DecodeError> {
        let file =
            File::open(path).map_err(|e| DecodeError::Symphonia(SymphoniaError::IoError(e)))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            // A hint, not a decision — Symphonia still probes the bytes. It just probes faster.
            hint.with_extension(ext);
        }
        // ⚠️ symphonia 0.6: `Probe::format` -> `Probe::probe`, e ele devolve o `FormatReader`
        // directamente em vez de um `ProbeResult` com campo `.format`.
        let format = symphonia::default::get_probe().probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;
        // ⚠️ 0.6: `Track::codec_params` passou a ser `Option<CodecParameters>`, e o `CodecParameters`
        // e' um ENUM (Audio/Video/Subtitle). "Faixa decodificavel" deixa de ser "o codec nao e' NULL"
        // e passa a ser "os parametros existem E sao de audio" — mais estrito, e de graca.
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.as_ref().is_some_and(|p| p.is_audio()))
            .ok_or(DecodeError::NoTrack)?;
        let track_id = track.id;
        let rate = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .and_then(|a| a.sample_rate)
            .ok_or(DecodeError::UnknownRate)?;
        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or(DecodeError::NoTrack)?;
        let channels = audio_params
            .channels
            .as_ref()
            .map(symphonia::core::audio::Channels::count)
            .unwrap_or(2)
            .max(1);
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(audio_params, &AudioDecoderOptions::default())?;
        Ok((format, decoder, track_id, rate, channels))
    }

    /// The source's sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.rate
    }

    /// The source's channel count (as the file has it — the caller decides what to do with >2).
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Decode the next packet. `Ok(None)` at the end of the file.
    ///
    /// The returned slice is interleaved `f32` at [`Reader::channels`] channels, and it is **valid
    /// until the next call** — it is the reader's own buffer, deliberately, so that decoding a
    /// four-minute song allocates once rather than once per packet.
    pub fn next_packet(&mut self) -> Result<Option<&[f32]>, DecodeError> {
        loop {
            let packet = match self.format.next_packet() {
                // ⚠️ symphonia 0.6: o fim do media passou a ser `Ok(None)` EXPLICITO. Antes ele
                // chegava disfarcado de `IoError(UnexpectedEof)`, e o ramo abaixo existia so' para
                // o traduzir. Ele FICA como rede — um MediaSource que corte a meio ainda produz um
                // EOF de I/O real, e ai' "acabou" continua a ser a resposta certa.
                Ok(None) => return Ok(None),
                Ok(Some(p)) => p,
                // Symphonia signals a clean end-of-stream as an unexpected EOF.
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(None);
                }
                Err(SymphoniaError::ResetRequired) => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    decoded.copy_to_vec_interleaved(&mut self.buf);
                    return Ok(Some(&self.buf));
                }
                // One corrupt packet must not end the song.
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(e.into()),
            }
        }
    }

    /// Start the file over — how a looping stream loops.
    ///
    /// Re-opens rather than seeks. A seek to zero is *usually* the same thing, but "usually" is
    /// doing real work in that sentence: seeking is per-format, some containers only seek to a
    /// coarse index, and a loop that starts a few milliseconds late is a click every time round.
    /// Re-opening is exact for every format, and it happens once per loop — not per packet.
    pub fn rewind(&mut self) -> Result<(), DecodeError> {
        let (format, decoder, track_id, rate, channels) = Self::build(&self.path)?;
        self.format = format;
        self.decoder = decoder;
        self.track_id = track_id;
        self.rate = rate;
        self.channels = channels;
        // The sample buffer is sized by the codec's packet capacity, which has not changed — but it
        // belonged to the old decoder, so let the first packet re-make it.
        self.buf.clear();
        Ok(())
    }
}
