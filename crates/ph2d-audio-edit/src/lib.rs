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

mod ops;
mod peaks;

pub use ops::{FadeDir, FadeShape, in_range, peak, snap_to_zero_crossing};
pub use peaks::{ColumnPeaks, DEFAULT_BIN_SIZE, PeakCache, column_peaks};

use std::ops::Range;

use ph2d_audio::SampleData;

/// Maximum retained undo snapshots (each is a cheap `Arc` clone of the clip).
const MAX_HISTORY: usize = 64;

/// The editor's in-memory document: a clip, its waveform peak cache, and the
/// current sample selection. Rebuilding the cache is the only cost of replacing
/// the clip, so edits (W2) will call [`EditClip::set_data`].
#[derive(Debug, Clone)]
pub struct EditClip {
    data: SampleData,
    peaks: PeakCache,
    /// Selection in **frames**, `start..end`. `None` = nothing selected.
    selection: Option<Range<usize>>,
    /// Undo timeline — snapshots of `data`; `cursor` is the current one. Cheap:
    /// each snapshot is an `Arc<[f32]>` refcount bump. A fresh clip / `set_data`
    /// resets it; every edit `commit`s a new snapshot (truncating the redo tail).
    history: Vec<SampleData>,
    cursor: usize,
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
            history: vec![data.clone()],
            cursor: 0,
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

    /// Install `data` as the current clip: rebuild the peak cache + clamp the
    /// selection. Does NOT touch the undo timeline (callers manage that).
    fn install(&mut self, data: SampleData) {
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

    /// Replace the clip and **reset** the undo timeline (the load path — a new
    /// clip starts a fresh history).
    pub fn set_data(&mut self, data: SampleData) {
        self.history = vec![data.clone()];
        self.cursor = 0;
        self.install(data);
    }

    /// Commit an edited buffer as a new undo snapshot (truncating any redo tail),
    /// capping the timeline at [`MAX_HISTORY`].
    fn commit(&mut self, data: SampleData) {
        self.history.truncate(self.cursor + 1);
        self.history.push(data.clone());
        if self.history.len() > MAX_HISTORY {
            let drop = self.history.len() - MAX_HISTORY;
            self.history.drain(0..drop);
        }
        self.cursor = self.history.len() - 1;
        self.install(data);
    }

    /// Step back one edit. Returns `false` at the start of the timeline.
    pub fn undo(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.install(self.history[self.cursor].clone());
        true
    }

    /// Step forward one edit. Returns `false` at the tip of the timeline.
    pub fn redo(&mut self) -> bool {
        if self.cursor + 1 >= self.history.len() {
            return false;
        }
        self.cursor += 1;
        self.install(self.history[self.cursor].clone());
        true
    }

    /// Whether an [`EditClip::undo`] would do anything.
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    /// Whether an [`EditClip::redo`] would do anything.
    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.history.len()
    }

    /// The range edits act on: the selection, or the whole clip when none.
    fn target(&self) -> Range<usize> {
        self.selection.clone().unwrap_or(0..self.frame_count())
    }

    /// Scale the target range (selection, or whole clip) by `linear` gain.
    pub fn apply_gain(&mut self, linear: f32) {
        let t = self.target();
        self.commit(ops::in_range(&self.data, t, |d| ops::gain(d, linear)));
    }

    /// Peak-normalize the target range to `target_peak` (linear).
    pub fn apply_normalize_peak(&mut self, target_peak: f32) {
        let t = self.target();
        self.commit(ops::in_range(&self.data, t, |d| {
            ops::normalize_peak(d, target_peak)
        }));
    }

    /// Loudness-normalize the target range to `target_lufs` (BS.1770).
    pub fn apply_normalize_lufs(&mut self, target_lufs: f32) {
        let t = self.target();
        self.commit(ops::in_range(&self.data, t, |d| {
            ops::normalize_lufs(d, target_lufs)
        }));
    }

    /// Reverse the target range (selection, or whole clip).
    pub fn apply_reverse(&mut self) {
        let t = self.target();
        self.commit(ops::in_range(&self.data, t, ops::reverse));
    }

    /// Invert polarity of the target range.
    pub fn apply_invert(&mut self) {
        let t = self.target();
        self.commit(ops::in_range(&self.data, t, ops::invert));
    }

    /// Remove DC offset from the target range.
    pub fn apply_remove_dc_offset(&mut self) {
        let t = self.target();
        self.commit(ops::in_range(&self.data, t, ops::remove_dc_offset));
    }

    /// Fade the target range (selection or whole clip).
    pub fn apply_fade(&mut self, shape: FadeShape, dir: FadeDir) {
        self.commit(ops::fade(&self.data, self.target(), shape, dir));
    }

    /// Silence the target range.
    pub fn apply_silence(&mut self) {
        self.commit(ops::silence(&self.data, self.target()));
    }

    /// Crop to the selection (no-op with no selection). Clears the selection.
    pub fn apply_trim(&mut self) {
        if let Some(sel) = self.selection.clone() {
            self.commit(ops::trim(&self.data, sel));
            self.selection = None;
        }
    }

    /// Delete the selection (ripple), closing the gap. Clears the selection.
    pub fn apply_delete(&mut self) {
        if let Some(sel) = self.selection.clone() {
            self.commit(ops::delete(&self.data, sel));
            self.selection = None;
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
    fn apply_undo_redo_timeline() {
        let d = SampleData::from_interleaved(vec![0.5; 8], AudioFormat::stereo(48_000));
        let mut clip = EditClip::new(d);
        assert!(!clip.can_undo() && !clip.can_redo());

        clip.apply_gain(0.5); // 0.5 → 0.25
        assert_eq!(clip.data().samples()[0], 0.25);
        assert!(clip.can_undo() && !clip.can_redo());

        clip.apply_invert(); // 0.25 → -0.25
        assert_eq!(clip.data().samples()[0], -0.25);

        assert!(clip.undo()); // back to 0.25
        assert_eq!(clip.data().samples()[0], 0.25);
        assert!(clip.undo()); // back to 0.5
        assert_eq!(clip.data().samples()[0], 0.5);
        assert!(!clip.undo(), "at the start of the timeline");

        assert!(clip.redo()); // 0.25 again
        assert_eq!(clip.data().samples()[0], 0.25);

        // A new edit truncates the redo tail.
        clip.apply_gain(2.0); // 0.25 → 0.5
        assert_eq!(clip.data().samples()[0], 0.5);
        assert!(!clip.can_redo(), "new edit dropped the redo branch");
    }

    #[test]
    fn trim_uses_selection_and_clears_it() {
        let d = SampleData::from_interleaved(
            vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0],
            AudioFormat::stereo(48_000),
        );
        let mut clip = EditClip::new(d);
        clip.set_selection(Some(1..2));
        clip.apply_trim();
        assert_eq!(clip.frame_count(), 1);
        assert_eq!(clip.data().samples(), &[2.0, 2.0]);
        assert_eq!(clip.selection(), None);
        assert!(clip.undo(), "trim is undoable");
        assert_eq!(clip.frame_count(), 3);
    }

    #[test]
    fn whole_clip_ops_respect_selection_and_undo() {
        // 4 stereo frames, all 0.5. Select the middle two and gain them to 0.
        let d = SampleData::from_interleaved(vec![0.5; 8], AudioFormat::stereo(48_000));
        let mut clip = EditClip::new(d);
        clip.set_selection(Some(1..3));
        clip.apply_gain(0.0);
        assert_eq!(
            clip.data().samples(),
            &[0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5],
            "gain must scope to the selection, not the whole clip"
        );
        // The selection survives a length-preserving op (chainable).
        assert_eq!(clip.selection(), Some(1..3));
        // Undo brings the selected samples back.
        assert!(clip.undo(), "selection-scoped gain is undoable");
        assert_eq!(clip.data().samples(), &[0.5; 8]);
        // With no selection the same op hits the whole clip.
        clip.set_selection(None);
        clip.apply_gain(0.0);
        assert_eq!(clip.data().samples(), &[0.0; 8]);
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
