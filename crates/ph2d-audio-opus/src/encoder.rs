//! **The one place `unsafe` lives.** A safe wrapper over the transpiled libopus C ABI.
//!
//! `unsafe-libopus` is libopus 1.3.1 run through `c2rust`: the algorithm is the reference
//! encoder, but the interface is the raw C one — `*mut OpusEncoder`, out-parameters, an
//! integer error channel, and a `ctl` that takes varargs. Calling that is `unsafe` by
//! construction, and no amount of wishing makes it otherwise.
//!
//! So it is contained. [`Encoder`] owns the pointer, frees it on `Drop`, and every method it
//! exposes takes slices and returns `Result`. **No raw pointer crosses this module's
//! boundary**, and nothing above it needs an `unsafe` block — which is what lets
//! `ph2d-audio-encode` keep its `#![forbid(unsafe_code)]` while the rest of the module gains
//! an Opus encoder (ADR-0116).
//!
//! Every `unsafe` block below is annotated with the invariant that makes it sound.

use unsafe_libopus::{
    OpusEncoder, opus_encode, opus_encoder_create, opus_encoder_ctl, opus_encoder_destroy,
};

use crate::EncodeError;

/// `OPUS_APPLICATION_AUDIO` — tuned for music and general audio rather than for speech
/// intelligibility at low bitrates (`OPUS_APPLICATION_VOIP`). A game's SFX are audio.
const APPLICATION_AUDIO: i32 = 2049;
/// `OPUS_SET_BITRATE_REQUEST`.
const SET_BITRATE: i32 = 4002;
/// `OPUS_GET_LOOKAHEAD_REQUEST` — how many samples the encoder consumes before it produces
/// anything. It becomes the Ogg header's `pre_skip`, and a decoder that is not told about it
/// plays that much silence at the head of the file.
const GET_LOOKAHEAD: i32 = 4027;

/// A libopus encoder. Owns the allocation; frees it on drop.
pub(crate) struct Encoder {
    st: *mut OpusEncoder,
    channels: usize,
}

impl Encoder {
    /// Create an encoder at 48 kHz (the only rate this crate feeds it — see `lib.rs`).
    pub(crate) fn new(channels: usize, bitrate: i32) -> Result<Self, EncodeError> {
        let mut err: i32 = 0;
        // SAFETY: `channels` is 1 or 2 (checked by the caller, which builds it from
        // `ChannelLayout`), 48 000 is a rate libopus accepts, and `err` is a live `i32` we own
        // for the duration of the call. The returned pointer is checked below before any use.
        let st =
            unsafe { opus_encoder_create(48_000, channels as i32, APPLICATION_AUDIO, &mut err) };
        if st.is_null() || err < 0 {
            return Err(EncodeError::Encoder(err));
        }
        let enc = Self { st, channels };

        // SAFETY: `enc.st` is non-null (checked above) and points at an encoder libopus itself
        // just built. `SET_BITRATE` takes one `i32` by value, which is what is passed.
        let rc = unsafe { opus_encoder_ctl!(enc.st, SET_BITRATE, bitrate) };
        if rc < 0 {
            return Err(EncodeError::Encoder(rc));
        }
        Ok(enc)
    }

    /// The encoder's algorithmic delay, in samples at 48 kHz. This is the file's `pre_skip`:
    /// the decoder must throw this many samples away, or the clip starts with a gap of silence
    /// that nobody put there.
    pub(crate) fn lookahead(&self) -> Result<u16, EncodeError> {
        let mut n: i32 = 0;
        // SAFETY: `self.st` is a live encoder (the type cannot be constructed with a null one),
        // and `GET_LOOKAHEAD` writes a single `i32` through the pointer it is given — which is
        // a live local we own.
        let rc = unsafe { opus_encoder_ctl!(self.st, GET_LOOKAHEAD, &mut n) };
        if rc < 0 {
            return Err(EncodeError::Encoder(rc));
        }
        Ok(n.clamp(0, i32::from(u16::MAX)) as u16)
    }

    /// Encode exactly one frame: `frame_size` samples **per channel**, interleaved.
    ///
    /// Returns the packet. Opus produces one packet per frame, and it may legitimately be a
    /// single byte (a frame of pure silence) — which is not an error, and callers must not
    /// treat it as one.
    pub(crate) fn encode(
        &mut self,
        pcm: &[i16],
        frame_size: usize,
    ) -> Result<Vec<u8>, EncodeError> {
        if pcm.len() != frame_size * self.channels {
            return Err(EncodeError::Frame {
                got: pcm.len(),
                want: frame_size * self.channels,
            });
        }
        // 4000 bytes is libopus's own recommended maximum for a single packet; anything it can
        // produce at any sane bitrate fits, and it will never write past what we say it may.
        let mut out = vec![0u8; 4_000];
        // SAFETY: `self.st` is a live encoder. `pcm` is a slice we hold for the whole call and
        // whose length is exactly `frame_size * channels` (checked immediately above), which is
        // what libopus reads. `out` is a buffer we own, and we hand libopus its true length, so
        // it cannot write past the end. The return value is the number of bytes it wrote.
        let n = unsafe {
            opus_encode(
                self.st,
                pcm.as_ptr(),
                frame_size as i32,
                out.as_mut_ptr(),
                out.len() as i32,
            )
        };
        if n < 0 {
            return Err(EncodeError::Encoder(n));
        }
        out.truncate(n as usize);
        Ok(out)
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // SAFETY: `self.st` came from `opus_encoder_create` and has not been freed — this type
        // is the sole owner, is not `Copy`, and frees exactly once, here.
        unsafe { opus_encoder_destroy(self.st) };
    }
}
