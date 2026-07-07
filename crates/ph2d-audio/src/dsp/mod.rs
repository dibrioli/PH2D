//! DSP primitives — small, `#[inline]`, no-alloc, unit-tested building blocks.
//!
//! Fundamentals ship them wired where they belong (gain + pan + envelope drive
//! every voice); [`Biquad`] is provided and tested as a ready primitive for the
//! per-voice / per-bus filtering that lands in the features phase.

mod biquad;
mod envelope;
mod gain;
mod loudness;
mod pan;
mod reverb;

pub use biquad::{Biquad, BiquadCoeffs};
pub use envelope::{Adsr, AdsrParams, AdsrStage};
pub use gain::SmoothGain;
pub use loudness::{LoudnessMeter, SILENCE_LUFS, lufs_from_mean_square};
pub use pan::equal_power_pan;
pub use reverb::Reverb;
