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

mod fx;
mod loops;
mod ops;
mod peaks;
mod truepeak;

pub use fx::{Effect, TailEffect};
pub use loops::crossfaded_loop;
pub use ops::{
    FadeDir, FadeShape, in_range, in_range_tail, in_range_warm, peak, snap_to_zero_crossing,
};
pub use peaks::{ColumnPeaks, DEFAULT_BIN_SIZE, PeakCache, column_peaks};
pub use truepeak::{OVERSAMPLE, true_peak};

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
    /// Loop region in **frames**, `start..end` (half-open). Metadata, NOT part of
    /// the undo timeline — it is not sample data, so it survives undo/redo and only
    /// clamps when an edit shrinks the clip (mirror of `selection`). Drives the
    /// click-free audition ([`crossfaded_loop`]) and the exported `smpl` chunk.
    loop_region: Option<Range<usize>>,
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
            loop_region: None,
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
        if let Some(lp) = &self.loop_region {
            let start = lp.start.min(frames);
            let end = lp.end.min(frames);
            self.loop_region = (start < end).then_some(start..end);
        }
    }

    /// Replace the clip and **reset** the undo timeline (the load path — a new
    /// clip starts a fresh history + a fresh loop).
    pub fn set_data(&mut self, data: SampleData) {
        self.history = vec![data.clone()];
        self.cursor = 0;
        self.loop_region = None;
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
    pub fn render_tail_effect(&self, fx: TailEffect) -> SampleData {
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
    pub fn apply_tail_effect(&mut self, fx: TailEffect) {
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
        self.loop_region.clone()
    }

    /// Whether a loop region is set.
    pub fn has_loop(&self) -> bool {
        self.loop_region.is_some()
    }

    /// Set the loop region (frames); an empty or inverted range clears it.
    pub fn set_loop_region(&mut self, range: Option<Range<usize>>) {
        self.loop_region = range.and_then(|r| self.clamp_frames(r));
    }

    /// Adopt the current selection as the loop region (no-op with no selection).
    pub fn set_loop_from_selection(&mut self) {
        if let Some(sel) = self.selection.clone() {
            self.loop_region = self.clamp_frames(sel);
        }
    }

    /// Clear the loop region.
    pub fn clear_loop(&mut self) {
        self.loop_region = None;
    }

    /// Snap both loop endpoints to the nearest zero crossing within `window` frames
    /// (per [`snap_to_zero_crossing`]). No-op without a loop, or if the snap would
    /// collapse the region.
    pub fn snap_loop_to_zero_crossing(&mut self, window: usize) {
        if let Some(lp) = self.loop_region.clone() {
            let start = ops::snap_to_zero_crossing(&self.data, lp.start, window);
            let end = ops::snap_to_zero_crossing(&self.data, lp.end, window);
            if start < end {
                self.loop_region = Some(start..end);
            }
        }
    }

    /// Build the click-free looping buffer for the current loop region with an
    /// `xfade`-frame pre-loop crossfade — the buffer the shell auditions on repeat.
    /// `None` when no loop is set.
    pub fn loop_audition_buffer(&self, xfade: usize) -> Option<SampleData> {
        let lp = self.loop_region.clone()?;
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
    fn tail_effect_grows_the_clip_and_undo_restores_it() {
        let d = SampleData::from_interleaved(vec![0.5; 2_000], AudioFormat::stereo(48_000));
        let mut clip = EditClip::new(d);
        assert_eq!(clip.frame_count(), 1_000);

        let echo = TailEffect::Delay {
            time_secs: 0.002,
            feedback: 0.3,
            mix: 0.5,
            tail_secs: 0.01, // 480 frames
        };
        clip.apply_tail_effect(echo);
        assert_eq!(
            clip.frame_count(),
            1_480,
            "no selection → whole clip + tail"
        );
        assert!(clip.can_undo());
        assert!(clip.undo());
        assert_eq!(
            clip.frame_count(),
            1_000,
            "undo restores the original length"
        );

        // A selection mid-clip: the tail bleeds in, the clip does NOT grow.
        clip.set_selection(Some(0..100));
        clip.apply_tail_effect(echo);
        assert_eq!(clip.frame_count(), 1_000, "tail fits inside → no growth");
    }

    /// The live-audition contract: the buffer the user HEARS (`render_*`) is
    /// bit-for-bit the one `commit_rendered` puts in the undo timeline, and it is
    /// identical to what the one-shot `apply_*` would have produced. A drift here
    /// means Apply silently sounds different from the preview.
    #[test]
    fn rendered_audition_is_what_gets_committed() {
        let d = SampleData::from_interleaved(vec![0.6; 2_000], AudioFormat::stereo(48_000));
        let fx = Effect::Saturate { drive: 4.0 };

        let mut auditioned = EditClip::new(d.clone());
        let heard = auditioned.render_effect(fx);
        assert_eq!(auditioned.frame_count(), 1_000, "render must NOT mutate");
        assert!(!auditioned.can_undo(), "render must NOT touch the timeline");
        auditioned.commit_rendered(heard.clone());

        let mut applied = EditClip::new(d);
        applied.apply_effect(fx);
        assert_eq!(auditioned.data().samples(), applied.data().samples());
        assert_eq!(heard.samples(), applied.data().samples());
        assert!(auditioned.can_undo() && auditioned.undo());
    }

    /// Same contract for the tail family, where the audition also changes length.
    #[test]
    fn rendered_tail_audition_matches_apply() {
        let d = SampleData::from_interleaved(vec![0.5; 2_000], AudioFormat::stereo(48_000));
        let fx = TailEffect::Delay {
            time_secs: 0.002,
            feedback: 0.3,
            mix: 0.5,
            tail_secs: 0.01,
        };
        let clip = EditClip::new(d.clone());
        let heard = clip.render_tail_effect(fx);
        assert_eq!(clip.frame_count(), 1_000, "render must NOT mutate");

        let mut applied = EditClip::new(d);
        applied.apply_tail_effect(fx);
        assert_eq!(heard.samples(), applied.data().samples());
        assert_eq!(heard.frame_count(), 1_480);
    }

    /// A filter applied to a MID-CLIP selection must not click at its leading edge.
    /// Found by the 2026-07-09 audit: `in_range` hands the op an isolated region, so
    /// the biquad starts with cleared memory — as if silence preceded the selection —
    /// and its first output collapses toward zero while the untouched neighbour is
    /// still at full level. `render_effect` now pre-rolls the real preceding audio.
    #[test]
    fn filtering_a_mid_clip_selection_does_not_click_at_the_leading_edge() {
        // 4000 frames of steady DC: any level jump at the splice IS the artifact.
        let d = SampleData::from_interleaved(vec![0.7; 8_000], AudioFormat::stereo(48_000));
        let fx = Effect::LowPass {
            cutoff: 1_000.0,
            q: 0.707,
        };
        let sel = 2_000..3_000;
        let step = |x: &SampleData| (x.samples()[2_000 * 2] - x.samples()[1_999 * 2]).abs();

        // The cold splice (what plain `in_range` does) really does click.
        let cold = in_range(&d, sel.clone(), |x| fx.apply(x));
        assert!(
            step(&cold) > 0.5,
            "expected the cold-start click to reproduce, got {}",
            step(&cold)
        );

        // `render_effect` pre-rolls the filter → the edge is continuous.
        let mut clip = EditClip::new(d.clone());
        clip.set_selection(Some(sel));
        let warm = clip.render_effect(fx);
        assert!(
            step(&warm) < 0.01,
            "warm-up must remove the edge click, got {}",
            step(&warm)
        );

        // No selection: nothing precedes the range, so nothing to warm up on — and
        // the result stays byte-identical to applying the op directly.
        let whole = EditClip::new(d.clone());
        assert_eq!(whole.render_effect(fx).samples(), fx.apply(&d).samples());
    }

    /// Loop metadata: adopted from the selection, survives undo/redo (it is not
    /// sample data), clamps when an edit shrinks the clip, and clears on a new load.
    #[test]
    fn loop_region_is_metadata_that_survives_undo_and_clamps() {
        let d = SampleData::from_interleaved(vec![0.5; 20_000], AudioFormat::stereo(48_000));
        let mut clip = EditClip::new(d); // 10_000 frames
        assert!(!clip.has_loop());

        clip.set_selection(Some(2_000..8_000));
        clip.set_loop_from_selection();
        assert_eq!(clip.loop_region(), Some(2_000..8_000));

        // An edit (gain) does NOT disturb the loop, and undo leaves it alone.
        clip.apply_gain(0.5);
        assert_eq!(
            clip.loop_region(),
            Some(2_000..8_000),
            "edit keeps the loop"
        );
        assert!(clip.undo());
        assert_eq!(
            clip.loop_region(),
            Some(2_000..8_000),
            "undo keeps the loop"
        );

        // Trimming to 0..5_000 shrinks the clip → the loop clamps to the new length.
        clip.set_selection(Some(0..5_000));
        clip.apply_trim();
        assert_eq!(clip.frame_count(), 5_000);
        assert_eq!(
            clip.loop_region(),
            Some(2_000..5_000),
            "loop clamps to the clip"
        );

        // A new clip clears the loop.
        clip.set_data(SampleData::from_interleaved(
            vec![0.0; 1_000],
            AudioFormat::stereo(48_000),
        ));
        assert!(!clip.has_loop(), "load clears the loop");
    }

    /// Snap moves both endpoints onto zero crossings; the audition buffer is the
    /// region length and loops without a click.
    #[test]
    fn loop_snap_and_audition_buffer() {
        // A mono sine: zero crossings are dense, so a small window always finds one.
        let step = std::f32::consts::TAU * 200.0 / 48_000.0;
        let v: Vec<f32> = (0..4_800).map(|i| (i as f32 * step).sin()).collect();
        let mut clip = EditClip::new(SampleData::from_interleaved(v, AudioFormat::mono(48_000)));
        clip.set_selection(Some(1_001..3_099));
        clip.set_loop_from_selection();
        clip.snap_loop_to_zero_crossing(64);
        let lp = clip.loop_region().unwrap();
        let sample = |f: usize| clip.data().samples()[f];
        let crosses = |f: usize| f > 0 && (sample(f - 1) <= 0.0) != (sample(f) <= 0.0);
        assert!(crosses(lp.start), "loop start on a zero crossing");
        assert!(crosses(lp.end), "loop end on a zero crossing");

        let buf = clip.loop_audition_buffer(256).expect("loop is set");
        assert_eq!(buf.frame_count(), lp.len());
        // The crossfade drives the seam down to the source's OWN continuity at the
        // loop point — `|data[start] − data[start-1]|`, one adjacent-sample step — not
        // to zero. That is the click-free floor: the wrap becomes the source's natural
        // `start-1 → start` transition.
        let natural = (sample(lp.start) - sample(lp.start - 1)).abs();
        assert!(
            loops::seam_step(&buf) <= natural + 1e-3,
            "audition seam {} must reach the natural step {natural}",
            loops::seam_step(&buf)
        );
        // No loop → no audition buffer.
        clip.clear_loop();
        assert!(clip.loop_audition_buffer(256).is_none());
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
