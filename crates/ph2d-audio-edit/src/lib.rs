#![forbid(unsafe_code)]
//! `ph2d-audio-edit` — offline, destructive editing of [`ph2d_audio::SampleData`].
//!
//! `SampleData` is an immutable `Arc<[f32]>`, so every edit produces a **fresh**
//! buffer. This crate is the editor's document layer. It runs on the **control
//! thread** (never the RT audio thread), so HR-3 (no-alloc) and HR-5 (no
//! transcendentals) do not constrain it — fades, normalise, filters, FFT and
//! pitch work can allocate and use `exp`/`sin`/FFT freely.
//!
//! ## Layout
//!
//! - [`EditClip`] — the document: clip + [`PeakCache`] (waveform) + selection +
//!   an undo timeline. Every `apply_*` acts on the **target range** (the
//!   selection, or the whole clip when there is none) and commits one undo step.
//! - `ops` — sample operations (gain, normalise peak/LUFS, reverse, DC, trim,
//!   cut, silence, fade, zero-crossing snap), plus the two splice primitives that
//!   make everything selection-aware: [`in_range`] (length-preserving) and
//!   [`in_range_tail`] (tail-extending).
//! - `fx` — the effects rack: [`Effect`] (filters/dynamics/character, splices via
//!   [`in_range`]) and [`TailEffect`] (reverb/delay, splices via
//!   [`in_range_tail`] because their ring-out outlives the input).
//!
//! Plan: `docs/Audio/02_plano_implementacao_completo.md`.

mod clipboard;
mod fx;
mod history;
mod loops;
mod ops;
mod peaks;
mod pieces;
mod structure;
mod truepeak;
mod variation;

pub use fx::{Effect, TailEffect};
pub use loops::crossfaded_loop;
pub use ops::{
    FadeDir, FadeShape, force_mono, in_range, in_range_tail, in_range_warm, normalize_lufs, peak,
    snap_to_zero_crossing,
};
pub use peaks::{ColumnPeaks, DEFAULT_BIN_SIZE, PeakCache, column_peaks};
pub use pieces::{boundaries, ranges};
pub use truepeak::{OVERSAMPLE, true_peak};
pub use variation::{
    Jitter, PickStrategy, Variation, VariationPicker, VariationSet, WEIGHT_RANGE, natural_cmp,
    parse as parse_variation_set, serialize as serialize_variation_set,
};

use std::ops::Range;

use ph2d_audio::SampleData;

use history::History;
use structure::{FrameMap, Structure};

pub use clipboard::{conform, insert, split_at};

/// A named cue point on the clip's timeline (W6 asset-prep) — a transition / sync /
/// sustain marker a game runtime can react to. Written to the WAV's `cue`+`LIST/adtl`
/// chunks. Metadata, like the loop region: NOT part of the undo timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// Position in **frames**.
    pub frame: usize,
    /// Display / export label (e.g. `M1`).
    pub name: String,
}

/// The editor's in-memory document: a clip, its waveform peak cache, and the
/// current sample selection. Rebuilding the cache is the only cost of replacing
/// the clip, so edits (W2) will call [`EditClip::set_data`].
#[derive(Debug, Clone)]
pub struct EditClip {
    data: SampleData,
    peaks: PeakCache,
    /// Selection in **frames**, `start..end`. `None` = nothing selected. Where the *user* is
    /// pointing — not a property of the audio, and so deliberately not part of the undo step.
    selection: Option<Range<usize>>,
    /// Everything positional that is glued to the audio: **cuts, markers, the loop region**.
    ///
    /// One value, and part of the undo step, because the edits that reorder and stretch pieces
    /// move all of it at once — an undo that restored the samples but not the boundaries would
    /// draw the cuts across the wrong audio. See [`structure`].
    structure: Structure,
    /// Undo timeline — **deltas, capped by bytes** (ADR-0117). Each step holds only the samples
    /// its edit displaced, so fixing one second of a three-minute clip costs a second. It used to
    /// hold 64 whole clips: on a 3-minute stereo clip that peaked at **4351 MB**, against a
    /// 3500 MB budget for the entire application.
    history: History,
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
            history: History::new(),
            data,
            peaks,
            selection: None,
            structure: Structure::default(),
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

    /// Install `data` as the current clip: rebuild the peak cache + clamp the selection and the
    /// structure. Does NOT touch the undo timeline (callers manage that).
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
        self.structure.clamp(frames);
    }

    /// Replace the clip and **reset** the undo timeline (the load path — a new clip starts a
    /// fresh history and a fresh structure: no cuts, no loop, no markers).
    pub fn set_data(&mut self, data: SampleData) {
        self.history.clear();
        self.structure = Structure::default();
        self.install(data);
    }

    /// Commit an edited buffer whose frames did not **move** — every in-place op (gain, reverse,
    /// an effect), and tail growth, which only appends. Positions keep their meaning, so the
    /// structure rides along untouched.
    ///
    /// The step records **only what changed** (`history::History` diffs the two buffers), and the
    /// timeline is capped by **bytes**, not by a count of edits — see ADR-0117. An edit that
    /// changed nothing records no step, so a no-op never lights the Undo button.
    fn commit(&mut self, data: SampleData) {
        let structure = self.structure.clone();
        self.commit_with(data, structure);
    }

    /// Commit an edit that **moved audio** (a delete, a paste, a trim, a reorder, a stretch).
    ///
    /// Markers and the loop are carried across `map`; the cuts are the ones the caller derived
    /// from the new layout — never arithmetic on the old ones (see [`Structure::remap`]).
    fn commit_moved(&mut self, data: SampleData, cuts: Vec<usize>, map: &FrameMap) {
        let mut structure = self.structure.clone();
        structure.remap(map);
        structure.cuts = cuts;
        self.commit_with(data, structure);
    }

    /// Commit a change to the **structure alone** — a split, a merge. Not one sample moves, so the
    /// step carries no audio at all; it is still a step, because a cut you cannot take back is a
    /// cut you cannot experiment with. `false` when it changed nothing.
    fn commit_structure(&mut self, cuts: Vec<usize>) -> bool {
        let mut structure = self.structure.clone();
        structure.cuts = cuts;
        structure.clamp(self.frame_count());
        if structure == self.structure {
            return false;
        }
        let data = self.data.clone();
        self.commit_with(data, structure);
        true
    }

    /// The one place a new (buffer, structure) pair becomes the document.
    fn commit_with(&mut self, data: SampleData, mut structure: Structure) {
        // Clamp BEFORE recording, so the timeline holds what the document will actually hold —
        // otherwise `install`'s clamp would silently diverge from the step, and a redo would
        // resurrect a boundary past the end of the clip.
        structure.clamp(data.frame_count());
        self.history
            .push(&self.data, &self.structure, &data, &structure);
        self.structure = structure;
        self.install(data);
    }

    /// Step back one edit. Returns `false` at the start of the timeline.
    pub fn undo(&mut self) -> bool {
        let Some((data, structure)) = self.history.undo(&self.data, &self.structure) else {
            return false;
        };
        self.structure = structure;
        self.install(data);
        true
    }

    /// Step forward one edit. Returns `false` at the tip of the timeline.
    pub fn redo(&mut self) -> bool {
        let Some((data, structure)) = self.history.redo(&self.data, &self.structure) else {
            return false;
        };
        self.structure = structure;
        self.install(data);
        true
    }

    /// Whether an [`EditClip::undo`] would do anything.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether an [`EditClip::redo`] would do anything.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
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

    /// Downmix the WHOLE clip to mono (for 3D positional audio). No-op if already
    /// mono. Whole-clip only — a mono selection inside a stereo clip is meaningless —
    /// so it ignores the selection; the frame count is preserved, so the selection and
    /// loop survive.
    pub fn apply_force_mono(&mut self) {
        if self.data.format().channel_count() <= 1 {
            return;
        }
        self.commit(ops::force_mono(&self.data));
    }

    /// Render an offline [`Effect`] over the target range **without committing** —
    /// the buffer the editor auditions live. Length-preserving, so it splices back
    /// in place. Stateful effects (the IIR filters) are pre-rolled with the audio
    /// before the range via [`in_range_warm`], so a mid-clip selection doesn't
    /// click at its leading edge; memoryless ones get `warmup = 0` and behave
    /// exactly like [`in_range`]. Pair with [`EditClip::commit_rendered`] so what
    /// the user heard is bit-for-bit what lands in the undo timeline.
    pub fn render_effect(&self, fx: Effect) -> SampleData {
        let warmup = fx.warmup_frames(self.data.format().sample_rate);
        ops::in_range_warm(&self.data, self.target(), warmup, |d| fx.apply(d))
    }

    /// Render a **tail-extending** [`TailEffect`] over the target range without
    /// committing. The ring-out bleeds onto the audio after the range, and the
    /// buffer **grows** when the range reaches the end (see [`in_range_tail`]).
    pub fn render_tail_effect(&self, fx: &TailEffect) -> SampleData {
        let tail = fx.tail_frames(self.data.format().sample_rate);
        ops::in_range_tail(&self.data, self.target(), tail, |region, tail| {
            fx.render(region, tail)
        })
    }

    /// Commit a buffer produced by [`EditClip::render_effect`] /
    /// [`EditClip::render_tail_effect`] as one undo step.
    pub fn commit_rendered(&mut self, data: SampleData) {
        self.commit(data);
    }

    /// Render + commit an [`Effect`] in one step (one undo step).
    pub fn apply_effect(&mut self, fx: Effect) {
        let out = self.render_effect(fx);
        self.commit(out);
    }

    /// Render + commit a [`TailEffect`] in one step (one undo step). The clip
    /// **grows** when the target range reaches the end; the selection survives.
    pub fn apply_tail_effect(&mut self, fx: &TailEffect) {
        let out = self.render_tail_effect(fx);
        self.commit(out);
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
            let map = FrameMap::keep(sel.clone());
            let cuts = map.cuts(&self.structure.cuts);
            self.commit_moved(ops::trim(&self.data, sel), cuts, &map);
            self.selection = None;
        }
    }

    /// Delete the selection (ripple), closing the gap. Clears the selection.
    ///
    /// The audio after the gap slides left, and so does everything standing on it. Before the
    /// structure was one value this was a real bug: the samples shifted and the markers did not,
    /// so every cue past the deleted range quietly ended up on different audio.
    pub fn apply_delete(&mut self) {
        if let Some(sel) = self.selection.clone() {
            let map = FrameMap::splice(self.frame_count(), sel.start, sel.len(), 0);
            let cuts = map.cuts(&self.structure.cuts);
            self.commit_moved(ops::delete(&self.data, sel), cuts, &map);
            self.selection = None;
        }
    }

    /// The selection, as a clip of its own — the **copy** half of the clipboard (W2).
    ///
    /// Non-destructive, so it costs no undo step: copying is not an edit.
    pub fn copy_selection(&self) -> Option<SampleData> {
        let sel = self.selection.clone()?;
        (sel.start < sel.end).then(|| ops::trim(&self.data, sel))
    }

    /// **Cut**: take the selection *with you*, and ripple the rest closed. One undo step.
    ///
    /// Returns what was taken, for the caller's clipboard. The button that says "Cut" used to do
    /// only the second half — it deleted the selection rather than cutting it, which is the one
    /// thing the word rules out.
    pub fn apply_cut(&mut self) -> Option<SampleData> {
        let taken = self.copy_selection()?;
        self.apply_delete();
        Some(taken)
    }

    /// **Paste** `clip`: over the selection when there is one (replacing it), otherwise **at
    /// `at`** — the playhead. One undo step.
    ///
    /// `at` is a parameter and not derived from the selection because **a selection cannot express
    /// a caret**: an empty range is stored as `None` (see [`EditClip::install`]), so "insert here,
    /// selecting nothing" has nowhere to live. The insertion point of a paste is the *playhead*, as
    /// it is in every DAW, and only the caller knows where that is.
    ///
    /// `clip` is **conformed** to this clip's rate and channel layout first. The clipboard outlives
    /// the document — copy from a 44.1 kHz stereo take, `Load` a 48 kHz mono one, paste — and raw
    /// samples would go in 8 % sharp and in the wrong channels. Nobody hears that as a bug; they
    /// hear it as "the paste sounds off", which is worse.
    pub fn apply_paste(&mut self, clip: &SampleData, at: usize) {
        if clip.is_empty() {
            return;
        }
        let fitted = clipboard::conform(clip, self.data.format());
        // A selection is a replacement target, as it is in every DAW: the pasted audio takes its
        // place rather than shoving it aside.
        let (base, removed, source) = match self.selection.clone() {
            Some(sel) if sel.start < sel.end => {
                (sel.start, sel.len(), ops::delete(&self.data, sel))
            }
            _ => (at.min(self.frame_count()), 0, self.data.clone()),
        };
        // Net effect on the ORIGINAL clip: `removed` frames at `base` replaced by the pasted ones.
        let map = FrameMap::splice(self.frame_count(), base, removed, fitted.frame_count());
        let cuts = map.cuts(&self.structure.cuts);
        let out = clipboard::insert(&source, base, &fitted);
        self.commit_moved(out, cuts, &map);
        self.selection = Some(base..base + fitted.frame_count());
    }

    /// Current selection (frames), if any.
    pub fn selection(&self) -> Option<Range<usize>> {
        self.selection.clone()
    }

    /// Set the selection (frames); an empty or inverted range clears it.
    pub fn set_selection(&mut self, range: Option<Range<usize>>) {
        self.selection = range.and_then(|r| self.clamp_frames(r));
    }

    /// Clamp a frame range to the clip, returning `None` if it collapses.
    fn clamp_frames(&self, r: Range<usize>) -> Option<Range<usize>> {
        let frames = self.frame_count();
        let start = r.start.min(frames);
        let end = r.end.min(frames);
        (start < end).then_some(start..end)
    }

    /// The current loop region (frames), if any. Half-open `start..end`.
    pub fn loop_region(&self) -> Option<Range<usize>> {
        self.structure.loop_region.clone()
    }

    /// Whether a loop region is set.
    pub fn has_loop(&self) -> bool {
        self.structure.loop_region.is_some()
    }

    /// Set the loop region (frames); an empty or inverted range clears it.
    pub fn set_loop_region(&mut self, range: Option<Range<usize>>) {
        self.structure.loop_region = range.and_then(|r| self.clamp_frames(r));
    }

    /// Adopt the current selection as the loop region (no-op with no selection).
    pub fn set_loop_from_selection(&mut self) {
        if let Some(sel) = self.selection.clone() {
            self.structure.loop_region = self.clamp_frames(sel);
        }
    }

    /// Clear the loop region.
    pub fn clear_loop(&mut self) {
        self.structure.loop_region = None;
    }

    /// The clip's cue markers, sorted by frame.
    pub fn markers(&self) -> &[Marker] {
        &self.structure.markers
    }

    /// Add a named cue marker at `frame` (clamped to the clip), keeping the list sorted
    /// by frame. Returns `false` (no-op) if a marker already sits on that exact frame.
    pub fn add_marker(&mut self, frame: usize, name: impl Into<String>) -> bool {
        let frame = frame.min(self.frame_count().saturating_sub(1));
        let pos = self.structure.markers.partition_point(|m| m.frame < frame);
        if self
            .structure
            .markers
            .get(pos)
            .is_some_and(|m| m.frame == frame)
        {
            return false;
        }
        self.structure.markers.insert(
            pos,
            Marker {
                frame,
                name: name.into(),
            },
        );
        true
    }

    /// Remove the marker nearest to `frame` within `window` frames. Returns the removed
    /// marker, or `None` if none is close enough.
    pub fn remove_marker_near(&mut self, frame: usize, window: usize) -> Option<Marker> {
        let (idx, _) = self
            .structure
            .markers
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.frame.abs_diff(frame)))
            .filter(|&(_, d)| d <= window)
            .min_by_key(|&(_, d)| d)?;
        Some(self.structure.markers.remove(idx))
    }

    /// Remove every marker.
    pub fn clear_markers(&mut self) {
        self.structure.markers.clear();
    }

    /// Snap both loop endpoints to the nearest zero crossing within `window` frames
    /// (per [`snap_to_zero_crossing`]). No-op without a loop, or if the snap would
    /// collapse the region.
    pub fn snap_loop_to_zero_crossing(&mut self, window: usize) {
        if let Some(lp) = self.structure.loop_region.clone() {
            let start = ops::snap_to_zero_crossing(&self.data, lp.start, window);
            let end = ops::snap_to_zero_crossing(&self.data, lp.end, window);
            if start < end {
                self.structure.loop_region = Some(start..end);
            }
        }
    }

    /// Build the click-free looping buffer for the current loop region with an
    /// `xfade`-frame pre-loop crossfade — the buffer the shell auditions on repeat.
    /// `None` when no loop is set.
    pub fn loop_audition_buffer(&self, xfade: usize) -> Option<SampleData> {
        let lp = self.structure.loop_region.clone()?;
        loops::crossfaded_loop(&self.data, lp, xfade)
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
mod ops_oracle;
#[cfg(test)]
mod tests;
