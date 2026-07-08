#![forbid(unsafe_code)]
//! `ph2d-audio-edit` — offline, destructive editing of [`ph2d_audio::SampleData`].
//!
//! `SampleData` is an immutable `Arc<[f32]>`, so every edit produces a **fresh**
//! buffer. This crate is the editor's document layer. It runs on the **control
//! thread** (never the RT audio thread), so HR-3 (no-alloc) and HR-5 (no
//! transcendentals) do not constrain it — fades, normalise, filters, FFT and
//! pitch work can allocate and use `exp`/`sin`/FFT freely.
//!
//! ## W1 scope
//!
//! [`PeakCache`] / [`column_peaks`] for waveform rendering and the [`EditClip`]
//! document (clip + peak cache + selection). The editing operations
//! (trim/split/fade/normalise/…) land in W2 —
//! `docs/Audio/02_plano_implementacao_completo.md` §5.

mod peaks;

pub use peaks::{ColumnPeaks, DEFAULT_BIN_SIZE, PeakCache, column_peaks};

use std::ops::Range;

use ph2d_audio::SampleData;

/// The editor's in-memory document: a clip, its waveform peak cache, and the
/// current sample selection. Rebuilding the cache is the only cost of replacing
/// the clip, so edits (W2) will call [`EditClip::set_data`].
#[derive(Debug, Clone)]
pub struct EditClip {
    data: SampleData,
    peaks: PeakCache,
    /// Selection in **frames**, `start..end`. `None` = nothing selected.
    selection: Option<Range<usize>>,
}

impl EditClip {
    /// Wrap a clip, building its peak cache at the default bin size.
    pub fn new(data: SampleData) -> Self {
        Self::with_bin_size(data, DEFAULT_BIN_SIZE)
    }

    /// Wrap a clip with an explicit peak-cache bin size.
    pub fn with_bin_size(data: SampleData, bin_size: usize) -> Self {
        let peaks = PeakCache::build(&data, bin_size);
        Self {
            data,
            peaks,
            selection: None,
        }
    }

    /// The underlying clip.
    pub fn data(&self) -> &SampleData {
        &self.data
    }

    /// The waveform peak cache.
    pub fn peaks(&self) -> &PeakCache {
        &self.peaks
    }

    /// Frame count of the clip.
    pub fn frame_count(&self) -> usize {
        self.data.frame_count()
    }

    /// Length in seconds.
    pub fn duration_secs(&self) -> f64 {
        self.data
            .format()
            .frames_to_secs(self.data.frame_count() as u64)
    }

    /// Replace the clip and rebuild the peak cache (keeps the bin size).
    /// Clamps any existing selection to the new length.
    pub fn set_data(&mut self, data: SampleData) {
        let bin = self.peaks.bin_size();
        self.peaks = PeakCache::build(&data, bin);
        let frames = data.frame_count();
        self.data = data;
        if let Some(sel) = &self.selection {
            let start = sel.start.min(frames);
            let end = sel.end.min(frames);
            self.selection = (start < end).then_some(start..end);
        }
    }

    /// Current selection (frames), if any.
    pub fn selection(&self) -> Option<Range<usize>> {
        self.selection.clone()
    }

    /// Set the selection (frames); an empty or inverted range clears it.
    pub fn set_selection(&mut self, range: Option<Range<usize>>) {
        self.selection = range.and_then(|r| {
            let frames = self.frame_count();
            let start = r.start.min(frames);
            let end = r.end.min(frames);
            (start < end).then_some(start..end)
        });
    }

    /// Reduce a visible window to `columns` per-channel min/max pairs.
    pub fn column_peaks(
        &self,
        frame_start: usize,
        frame_end: usize,
        columns: usize,
    ) -> ColumnPeaks {
        column_peaks(&self.data, &self.peaks, frame_start, frame_end, columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    #[test]
    fn duration_and_selection_clamp() {
        let data = SampleData::from_interleaved(vec![0.0; 96_000], AudioFormat::stereo(48_000));
        let mut clip = EditClip::new(data);
        assert_eq!(clip.frame_count(), 48_000);
        assert!((clip.duration_secs() - 1.0).abs() < 1e-9);

        clip.set_selection(Some(10..60_000)); // end past the clip
        assert_eq!(clip.selection(), Some(10..48_000));

        clip.set_selection(Some(500..500)); // empty → cleared
        assert_eq!(clip.selection(), None);
    }

    #[test]
    fn set_data_rebuilds_and_clamps_selection() {
        let big = SampleData::from_interleaved(vec![0.1; 20_000], AudioFormat::mono(48_000));
        let mut clip = EditClip::new(big);
        clip.set_selection(Some(100..19_000));
        // Shrink the clip; selection must clamp.
        let small = SampleData::from_interleaved(vec![0.2; 1_000], AudioFormat::mono(48_000));
        clip.set_data(small);
        assert_eq!(clip.frame_count(), 1_000);
        assert_eq!(clip.selection(), Some(100..1_000));
    }
}
