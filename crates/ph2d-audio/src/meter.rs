//! [`AudioMeter`] — block peak **and** RMS levels published by the renderer.
//!
//! The audio thread stores, per block, the peak magnitude (fast, catches
//! transients) and the RMS (√mean-square, ≈ perceived loudness) for the master
//! and every sub-bus; the control thread reads them once per UI frame. Lock-free
//! and monitoring-only, so `Relaxed` ordering is enough — a level read one block
//! stale is invisible. Peak-hold / decay ballistics live on the UI side (the
//! meter widget / panel), not here.

use std::array::from_fn;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::bus::SUB_BUS_COUNT;

/// A stereo peak+RMS pair as stored/loaded (`[left, right]` each).
type Pair = [AtomicU32; 2];

fn load(p: &Pair) -> [f32; 2] {
    [
        f32::from_bits(p[0].load(Ordering::Relaxed)),
        f32::from_bits(p[1].load(Ordering::Relaxed)),
    ]
}

fn store_pair(p: &Pair, v: [f32; 2]) {
    p[0].store(v[0].to_bits(), Ordering::Relaxed);
    p[1].store(v[1].to_bits(), Ordering::Relaxed);
}

/// Shared, lock-free stereo peak + RMS levels. Shared via `Arc` between the
/// renderer (writer) and the control-side [`crate::AudioEngine`] (reader).
#[derive(Debug)]
pub struct AudioMeter {
    master_peak: Pair,
    master_rms: Pair,
    /// Momentary-loudness K-weighted **mean-square** (linear) of the master; the
    /// reader converts it to LUFS off the RT thread (no `log10` here).
    master_loudness_ms: AtomicU32,
    /// Post-fader `[L, R]` peak/RMS per sub-bus, indexed by `BusId::sub_index`.
    bus_peak: [Pair; SUB_BUS_COUNT],
    bus_rms: [Pair; SUB_BUS_COUNT],
}

impl Default for AudioMeter {
    fn default() -> Self {
        let zero_pair = || [AtomicU32::new(0), AtomicU32::new(0)];
        Self {
            master_peak: zero_pair(),
            master_rms: zero_pair(),
            master_loudness_ms: AtomicU32::new(0),
            bus_peak: from_fn(|_| zero_pair()),
            bus_rms: from_fn(|_| zero_pair()),
        }
    }
}

impl AudioMeter {
    /// Store this block's master peak + RMS (called by the renderer each block).
    pub(crate) fn store(&self, peak: [f32; 2], rms: [f32; 2]) {
        store_pair(&self.master_peak, peak);
        store_pair(&self.master_rms, rms);
    }

    /// Store the master's momentary K-weighted mean-square for this block.
    pub(crate) fn store_loudness(&self, mean_square: f32) {
        self.master_loudness_ms
            .store(mean_square.to_bits(), Ordering::Relaxed);
    }

    /// The master's momentary K-weighted mean-square (linear). Convert to LUFS
    /// with [`crate::dsp::lufs_from_mean_square`] on the reading thread.
    pub fn loudness_mean_square(&self) -> f32 {
        f32::from_bits(self.master_loudness_ms.load(Ordering::Relaxed))
    }

    /// Store sub-bus `i`'s post-fader peak + RMS for this block.
    pub(crate) fn store_bus(&self, i: usize, peak: [f32; 2], rms: [f32; 2]) {
        if let (Some(p), Some(r)) = (self.bus_peak.get(i), self.bus_rms.get(i)) {
            store_pair(p, peak);
            store_pair(r, rms);
        }
    }

    /// The most recent block's master peak magnitude `[left, right]`. `0.0` is
    /// silence; values above `1.0` mean the mix is clipping.
    pub fn peaks(&self) -> [f32; 2] {
        load(&self.master_peak)
    }

    /// The most recent block's master RMS `[left, right]` (≈ perceived loudness).
    pub fn rms(&self) -> [f32; 2] {
        load(&self.master_rms)
    }

    /// Every sub-bus's most recent `[L, R]` peak, in `BusId::SUB_BUSES` order.
    pub fn bus_peaks(&self) -> [[f32; 2]; SUB_BUS_COUNT] {
        from_fn(|i| load(&self.bus_peak[i]))
    }

    /// Every sub-bus's most recent `[L, R]` RMS, in `BusId::SUB_BUSES` order.
    pub fn bus_rms(&self) -> [[f32; 2]; SUB_BUS_COUNT] {
        from_fn(|i| load(&self.bus_rms[i]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_master_peak_and_rms() {
        let m = AudioMeter::default();
        assert_eq!(m.peaks(), [0.0, 0.0]);
        assert_eq!(m.rms(), [0.0, 0.0]);
        m.store([0.5, 1.25], [0.35, 0.8]);
        assert_eq!(m.peaks(), [0.5, 1.25]);
        assert_eq!(m.rms(), [0.35, 0.8]);
    }

    #[test]
    fn round_trips_master_loudness() {
        let m = AudioMeter::default();
        assert_eq!(m.loudness_mean_square(), 0.0);
        m.store_loudness(0.0125);
        assert_eq!(m.loudness_mean_square(), 0.0125);
    }

    #[test]
    fn round_trips_per_bus_peak_and_rms() {
        let m = AudioMeter::default();
        m.store_bus(0, [0.7, 0.3], [0.5, 0.2]);
        assert_eq!(m.bus_peaks()[0], [0.7, 0.3]);
        assert_eq!(m.bus_rms()[0], [0.5, 0.2]);
        assert_eq!(m.bus_peaks()[1], [0.0, 0.0], "other buses untouched");
        // Out-of-range index is a harmless no-op (never panics on the RT thread).
        m.store_bus(SUB_BUS_COUNT + 5, [1.0, 1.0], [1.0, 1.0]);
    }
}
