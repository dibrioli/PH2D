#![forbid(unsafe_code)]
//! `ph2d-audio-spectral` — frequency-domain audio: spectrogram, repair, denoise (W5).
//!
//! ## Why this is its own crate
//!
//! It is the **only** place an FFT dependency lives (ADR-0115). That is the same
//! containment that keeps Symphonia inside `ph2d-audio-decode` and libvorbis inside
//! `ph2d-audio-encode`: no heavy DSP dependency is allowed to reach the RT mixer
//! (`ph2d-audio`), which must stay allocation-free and predictable. This crate runs on the
//! **control thread**, offline, so HR-3 (no-alloc) and HR-5 (no transcendentals) do not
//! constrain it — it may allocate and call `cos`/`log10`/FFT freely.
//!
//! ## Why frequency domain at all
//!
//! The rest of the audio module deliberately avoided the FFT: pitch shift is WSOLA, formant
//! shift is LPC, de-click is LPC + LSAR — all time-domain, all better suited to their job.
//! What lives here is the work that **cannot** be faked in the time domain, because it is
//! work on one frequency at a time while the rest of the sound is left alone:
//!
//! - [`Spectrogram`] — see the sound. A beep, a cough, a hum and a hiss look like nothing
//!   in a waveform and are unmistakable here.
//! - [`repair`] — erase a region of time-AND-frequency and rebuild it from what surrounds
//!   it. This is the one that has no time-domain equivalent: a beep sitting on top of
//!   speech shares the same *samples* as the speech, and only the bins can tell them apart.
//! - [`denoise`] — learn what the noise looks like from a patch of it, then subtract that
//!   shape from every column.
//!
//! ## Layout
//!
//! - `stft` — the transform. Everything else is a callback into it.
//! - `spectrogram` — the cached, display-resolution picture, plus its RGBA rendering.
//! - `repair` — spectral inpaint.
//! - `denoise` — noise profile + decision-directed Wiener suppression.
//! - `convolve` — fast convolution, which is what a **convolution reverb** is made of. The
//!   second thing the FFT bought, and it cost no new dependency: an impulse response *is* a
//!   room, and multiplying spectra is the only way to afford using one.

pub mod convolve;
pub mod denoise;
pub mod repair;
pub mod spectrogram;
pub mod stft;

pub use convolve::{convolve, normalize_ir};
pub use denoise::{NoiseProfile, denoise};
pub use repair::repair;
pub use spectrogram::{Band, Spectrogram};
pub use stft::{HOP, Stft, WINDOW, bin_hz, magnitude};
