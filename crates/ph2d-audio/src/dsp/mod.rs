//! DSP primitives — small, `#[inline]`, no-alloc, unit-tested building blocks.
//!
//! Fundamentals ship them wired where they belong (gain + pan + envelope drive
//! every voice); [`Biquad`] is provided and tested as a ready primitive for the
//! per-voice / per-bus filtering that lands in the features phase.

mod biquad;
mod envelope;
mod gain;
mod pan;

pub use biquad::{Biquad, BiquadCoeffs};
pub use envelope::{Adsr, AdsrParams, AdsrStage};
pub use gain::SmoothGain;
pub use pan::equal_power_pan;
