//! [`AudioMeter`] — block-peak levels published by the renderer for metering.
//!
//! The audio thread stores the current block's peak magnitude; the control
//! thread reads it (e.g. once per UI frame). Lock-free and monitoring-only, so
//! `Relaxed` ordering is enough — a level read one block stale is invisible.
//! Peak-hold / decay ballistics live on the UI side (the meter widget), not here.

use std::sync::atomic::{AtomicU32, Ordering};

/// Shared, lock-free stereo peak levels. Shared via `Arc` between the renderer
/// (writer) and the control-side [`crate::AudioEngine`] (reader).
#[derive(Debug, Default)]
pub struct AudioMeter {
    peak_l: AtomicU32,
    peak_r: AtomicU32,
}

impl AudioMeter {
    /// Store this block's peak magnitude (called by the renderer each block).
    pub(crate) fn store(&self, left: f32, right: f32) {
        self.peak_l.store(left.to_bits(), Ordering::Relaxed);
        self.peak_r.store(right.to_bits(), Ordering::Relaxed);
    }

    /// The most recent block's peak magnitude as `[left, right]`. `0.0` is
    /// silence; values above `1.0` mean the mix is clipping.
    pub fn peaks(&self) -> [f32; 2] {
        [
            f32::from_bits(self.peak_l.load(Ordering::Relaxed)),
            f32::from_bits(self.peak_r.load(Ordering::Relaxed)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_peaks() {
        let m = AudioMeter::default();
        assert_eq!(m.peaks(), [0.0, 0.0]);
        m.store(0.5, 1.25);
        assert_eq!(m.peaks(), [0.5, 1.25]);
    }
}
