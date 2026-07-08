//! [`LoudnessMeter`] — momentary loudness (LUFS) per **ITU-R BS.1770**:
//! K-weighting (a pre-filter high shelf + an RLB high-pass) followed by a 400 ms
//! sliding mean-square window.
//!
//! Approximation, honestly scoped: the two K-weighting biquads are built from the
//! **RBJ cookbook** ([`BiquadCoeffs`]) at the runtime sample rate using the
//! published BS.1770 corner/Q/gain parameters — not the exact BS.1770 48 kHz
//! coefficient tables — so the readout tracks loudness faithfully and works at any
//! rate, but it is a monitoring meter, not a certified loudness measurement.
//!
//! HR-3: the window ring is allocated once in [`LoudnessMeter::new`] and never
//! resized. HR-5: [`LoudnessMeter::process`] (the audio-thread hot path) is
//! mul/add/square only — the `log10` that turns mean-square into LUFS runs on the
//! control thread in [`lufs_from_mean_square`], never on the RT thread.

use crate::dsp::{Biquad, BiquadCoeffs};

/// K-weighting stage 1 — pre-filter high shelf (BS.1770 parameters).
const PREFILTER_HZ: f32 = 1_681.97;
const PREFILTER_Q: f32 = 0.707_175_2;
const PREFILTER_GAIN_DB: f32 = 3.999_84;
/// K-weighting stage 2 — RLB high-pass (BS.1770 parameters).
const RLB_HZ: f32 = 38.135_47;
const RLB_Q: f32 = 0.500_327;
/// BS.1770 loudness calibration offset (dB).
const ABS_OFFSET: f32 = -0.691;
/// Momentary-loudness integration window (BS.1770): 400 ms.
const WINDOW_SECS: f32 = 0.4;
/// Below this mean-square the window reads as silence.
const SILENCE_MS: f32 = 1e-12;
/// Loudness reported for a silent window (dB LUFS floor, instead of -inf).
pub const SILENCE_LUFS: f32 = -120.0;

/// Convert a K-weighted mean-square to LUFS. Control-thread only (uses `log10`);
/// the RT thread stores the linear mean-square and this runs on read.
pub fn lufs_from_mean_square(mean_square: f32) -> f32 {
    if mean_square <= SILENCE_MS {
        SILENCE_LUFS
    } else {
        ABS_OFFSET + 10.0 * mean_square.log10()
    }
}

/// Full-clip **integrated** loudness (LUFS) of an interleaved buffer — the offline
/// counterpart to the momentary [`LoudnessMeter`]. Applies the same K-weighting
/// (so it agrees with the meter's calibration) then averages the mean-square over
/// the WHOLE clip (an ungated BS.1770 approximation — good enough for a normalize
/// target). Control-thread / editing only; never on the RT path (it iterates the
/// whole clip and uses `log10`).
pub fn integrated_lufs(samples: &[f32], channels: usize, sample_rate: u32) -> f32 {
    let sr = sample_rate as f32;
    let pre = BiquadCoeffs::highshelf(sr, PREFILTER_HZ, PREFILTER_Q, PREFILTER_GAIN_DB);
    let rlb = BiquadCoeffs::highpass(sr, RLB_HZ, RLB_Q);
    let mut pre_l = Biquad::new(pre);
    let mut pre_r = Biquad::new(pre);
    let mut rlb_l = Biquad::new(rlb);
    let mut rlb_r = Biquad::new(rlb);
    let ch = channels.max(1);
    let frames = samples.len() / ch;
    if frames == 0 {
        return SILENCE_LUFS;
    }
    let mut sum = 0.0f64;
    for f in 0..frames {
        let l = samples[f * ch];
        let r = if ch >= 2 { samples[f * ch + 1] } else { l };
        let kl = rlb_l.process(pre_l.process(l));
        let kr = rlb_r.process(pre_r.process(r));
        sum += (kl * kl + kr * kr) as f64;
    }
    lufs_from_mean_square((sum / frames as f64) as f32)
}

/// Momentary-loudness meter over the final master output. Feed every stereo
/// sample with [`LoudnessMeter::process`]; read the window mean-square with
/// [`LoudnessMeter::mean_square`] and convert with [`lufs_from_mean_square`].
pub struct LoudnessMeter {
    // K-weighting: pre-filter high shelf then RLB high-pass, per channel.
    pre_l: Biquad,
    pre_r: Biquad,
    rlb_l: Biquad,
    rlb_r: Biquad,
    /// Sliding window of per-sample K-weighted energy (`ks_l² + ks_r²`).
    ring: Vec<f32>,
    idx: usize,
    /// Running sum of `ring` (f64 to resist drift over long runs).
    sum: f64,
    /// Valid samples so far (until the ring first fills).
    filled: usize,
}

impl LoudnessMeter {
    /// Build the K-weighting filters + the 400 ms window for `sample_rate`.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate as f32;
        let pre = BiquadCoeffs::highshelf(sr, PREFILTER_HZ, PREFILTER_Q, PREFILTER_GAIN_DB);
        let rlb = BiquadCoeffs::highpass(sr, RLB_HZ, RLB_Q);
        let window_len = ((WINDOW_SECS * sr) as usize).max(1);
        Self {
            pre_l: Biquad::new(pre),
            pre_r: Biquad::new(pre),
            rlb_l: Biquad::new(rlb),
            rlb_r: Biquad::new(rlb),
            ring: vec![0.0; window_len],
            idx: 0,
            sum: 0.0,
            filled: 0,
        }
    }

    /// Feed one stereo sample (the final master output) into the window.
    #[inline]
    pub fn process(&mut self, l: f32, r: f32) {
        let kl = self.rlb_l.process(self.pre_l.process(l));
        let kr = self.rlb_r.process(self.pre_r.process(r));
        let energy = kl * kl + kr * kr;
        // Evict the oldest sample, insert the newest, carry the running sum.
        let old = self.ring[self.idx];
        self.sum += (energy - old) as f64;
        self.ring[self.idx] = energy;
        self.idx = (self.idx + 1) % self.ring.len();
        if self.filled < self.ring.len() {
            self.filled += 1;
        }
    }

    /// The K-weighted mean-square over the window (linear; convert with
    /// [`lufs_from_mean_square`]). `0.0` until the first sample lands.
    pub fn mean_square(&self) -> f32 {
        let n = self.filled.max(1);
        (self.sum.max(0.0) / n as f64) as f32
    }

    /// Window capacity (fixed at construction) — for the HR-3 no-alloc gate.
    pub fn window_capacity(&self) -> usize {
        self.ring.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_reads_the_floor() {
        let m = LoudnessMeter::new(48_000);
        assert_eq!(m.mean_square(), 0.0);
        assert_eq!(lufs_from_mean_square(m.mean_square()), SILENCE_LUFS);
    }

    #[test]
    fn louder_input_reads_higher_lufs() {
        // Two 1 kHz sines, one 12 dB louder; the louder must read a higher LUFS.
        let lufs_of = |amp: f32| {
            let mut m = LoudnessMeter::new(48_000);
            let step = std::f32::consts::TAU * 1_000.0 / 48_000.0;
            // Fill well past the 400 ms window so it is fully populated + settled.
            for i in 0..48_000 {
                let s = (i as f32 * step).sin() * amp;
                m.process(s, s);
            }
            lufs_from_mean_square(m.mean_square())
        };
        let quiet = lufs_of(0.1);
        let loud = lufs_of(0.4); // +12 dB
        assert!(
            quiet > SILENCE_LUFS && loud > quiet + 6.0,
            "louder input must read markedly higher LUFS (quiet {quiet}, loud {loud})"
        );
        // A full-scale-ish 1 kHz tone lands in a sane LUFS ballpark (near 0, not
        // wildly off) — a loose sanity bound on the K-weighting calibration.
        assert!(
            (-12.0..6.0).contains(&loud),
            "a loud 1 kHz tone should read a plausible LUFS, got {loud}"
        );
    }
}
