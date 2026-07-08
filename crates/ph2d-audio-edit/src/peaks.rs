//! Waveform peak cache: min/max envelope per fixed-size bin, per channel.
//!
//! The overlay draws a waveform by asking for `columns` (min, max) pairs over a
//! visible frame window. Scanning every sample per repaint is O(samples); the
//! cache pre-reduces to one (min, max) per `bin_size` frames so an overview
//! repaint is O(bins). When the view is zoomed in past bin resolution
//! ([`ColumnPeaks`] with fewer frames-per-column than `bin_size`), the query
//! falls back to raw samples so close-ups stay exact.

use ph2d_audio::SampleData;

/// Default frames per bin — ~5 ms at 48 kHz, ~1 MB of cache for a 3-min stereo
/// clip. Small enough that the raw-sample fallback only kicks in when genuinely
/// zoomed to the sample level.
pub const DEFAULT_BIN_SIZE: usize = 256;

/// Pre-reduced min/max envelope of a clip. Flattened `[bin * channels + ch]`.
#[derive(Debug, Clone)]
pub struct PeakCache {
    bin_size: usize,
    channels: usize,
    bins: usize,
    min: Vec<f32>,
    max: Vec<f32>,
}

impl PeakCache {
    /// Build the cache from a clip. `bin_size` is clamped to at least 1 frame.
    pub fn build(data: &SampleData, bin_size: usize) -> Self {
        let bin_size = bin_size.max(1);
        let channels = data.format().channel_count().max(1);
        let frames = data.frame_count();
        let bins = frames.div_ceil(bin_size);
        let samples = data.samples();

        let mut min = vec![0.0f32; bins * channels];
        let mut max = vec![0.0f32; bins * channels];
        for b in 0..bins {
            let f0 = b * bin_size;
            let f1 = (f0 + bin_size).min(frames);
            for ch in 0..channels {
                let mut lo = f32::INFINITY;
                let mut hi = f32::NEG_INFINITY;
                for f in f0..f1 {
                    let s = samples[f * channels + ch];
                    lo = lo.min(s);
                    hi = hi.max(s);
                }
                // Empty bin (shouldn't happen given div_ceil) → flat zero.
                if !lo.is_finite() {
                    lo = 0.0;
                    hi = 0.0;
                }
                min[b * channels + ch] = lo;
                max[b * channels + ch] = hi;
            }
        }
        Self {
            bin_size,
            channels,
            bins,
            min,
            max,
        }
    }

    /// Frames per bin.
    pub fn bin_size(&self) -> usize {
        self.bin_size
    }

    /// Channel count.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Number of bins.
    pub fn bins(&self) -> usize {
        self.bins
    }

    /// Aggregate min/max of the bins covering `[f0, f1)` for `ch`.
    fn bin_range(&self, f0: usize, f1: usize, ch: usize) -> (f32, f32) {
        let b0 = f0 / self.bin_size;
        let b1 = f1.div_ceil(self.bin_size).min(self.bins);
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for b in b0..b1 {
            lo = lo.min(self.min[b * self.channels + ch]);
            hi = hi.max(self.max[b * self.channels + ch]);
        }
        if lo.is_finite() { (lo, hi) } else { (0.0, 0.0) }
    }
}

/// Per-channel column min/max for one visible frame window, ready to draw.
/// Flattened `[col * channels + ch]`.
#[derive(Debug, Clone)]
pub struct ColumnPeaks {
    /// Channel count (1 mono, 2 stereo).
    pub channels: usize,
    /// Number of columns produced (== requested, clamped to ≥1).
    pub columns: usize,
    /// Column minima, `[col * channels + ch]`.
    pub min: Vec<f32>,
    /// Column maxima, `[col * channels + ch]`.
    pub max: Vec<f32>,
}

impl ColumnPeaks {
    /// The `(min, max)` for a column/channel (clamped indices).
    pub fn get(&self, col: usize, ch: usize) -> (f32, f32) {
        let i = col.min(self.columns.saturating_sub(1)) * self.channels + ch.min(self.channels - 1);
        (self.min[i], self.max[i])
    }
}

/// Reduce a clip's `[frame_start, frame_end)` window to `columns` min/max pairs
/// per channel. Uses the bin cache when a column spans ≥ `bin_size` frames,
/// otherwise reads raw samples (exact close-ups). Ranges are clamped to the clip.
pub fn column_peaks(
    data: &SampleData,
    cache: &PeakCache,
    frame_start: usize,
    frame_end: usize,
    columns: usize,
) -> ColumnPeaks {
    let channels = cache.channels();
    let columns = columns.max(1);
    let frames = data.frame_count();
    let start = frame_start.min(frames);
    let end = frame_end.clamp(start, frames);
    let span = end - start;
    let samples = data.samples();

    let mut min = vec![0.0f32; columns * channels];
    let mut max = vec![0.0f32; columns * channels];
    if span == 0 {
        return ColumnPeaks {
            channels,
            columns,
            min,
            max,
        };
    }

    for c in 0..columns {
        // Integer-proportional column bounds (no float drift across columns).
        let c0 = start + c * span / columns;
        let c1 = (start + (c + 1) * span / columns).max(c0 + 1).min(end);
        let per_col = c1 - c0;
        for ch in 0..channels {
            let (lo, hi) = if per_col >= cache.bin_size() {
                cache.bin_range(c0, c1, ch)
            } else {
                let mut lo = f32::INFINITY;
                let mut hi = f32::NEG_INFINITY;
                for f in c0..c1 {
                    let s = samples[f * channels + ch];
                    lo = lo.min(s);
                    hi = hi.max(s);
                }
                if lo.is_finite() { (lo, hi) } else { (0.0, 0.0) }
            };
            min[c * channels + ch] = lo;
            max[c * channels + ch] = hi;
        }
    }
    ColumnPeaks {
        channels,
        columns,
        min,
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn ramp_stereo(frames: usize) -> SampleData {
        let mut v = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            let t = i as f32 / frames as f32;
            v.push(t); // L climbs 0..1
            v.push(-t); // R descends 0..-1
        }
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    #[test]
    fn cache_shape_and_bounds() {
        let data = ramp_stereo(1000);
        let cache = PeakCache::build(&data, 256);
        assert_eq!(cache.channels(), 2);
        assert_eq!(cache.bins(), 1000usize.div_ceil(256)); // 4
        // First bin (frames 0..256): L max ~ 255/1000, min 0.
        let (lo, hi) = cache.bin_range(0, 256, 0);
        assert!(lo.abs() < 1e-6);
        assert!((hi - 255.0 / 1000.0).abs() < 1e-6);
    }

    #[test]
    fn constant_signal_returns_constant() {
        let data = SampleData::from_interleaved(vec![0.7; 2000], AudioFormat::mono(48_000));
        let cache = PeakCache::build(&data, 256);
        let cols = column_peaks(&data, &cache, 0, 2000, 50);
        for c in 0..cols.columns {
            let (lo, hi) = cols.get(c, 0);
            assert!((lo - 0.7).abs() < 1e-6 && (hi - 0.7).abs() < 1e-6);
        }
    }

    #[test]
    fn zoomed_in_uses_raw_samples() {
        let data = ramp_stereo(1000);
        let cache = PeakCache::build(&data, 256);
        // 8 frames across 8 columns → 1 frame/col, below bin_size → raw path.
        let cols = column_peaks(&data, &cache, 0, 8, 8);
        assert_eq!(cols.columns, 8);
        // Column 0 = frame 0 → L=0; column 7 = frame 7 → L=7/1000.
        assert!(cols.get(0, 0).1.abs() < 1e-6);
        assert!((cols.get(7, 0).1 - 7.0 / 1000.0).abs() < 1e-6);
    }

    #[test]
    fn empty_window_is_silent() {
        let data = ramp_stereo(100);
        let cache = PeakCache::build(&data, 256);
        let cols = column_peaks(&data, &cache, 50, 50, 10);
        assert!(cols.min.iter().all(|&x| x == 0.0));
        assert!(cols.max.iter().all(|&x| x == 0.0));
    }
}
