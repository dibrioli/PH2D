//! **Cuts and pieces** — the clip as a sequence of parts you can select, reorder and stretch.
//!
//! ## Why cuts, and not a multi-clip track
//!
//! The obvious way to let a user rearrange a recording is the DAW arrangement view: a timeline of
//! independent clips with positions, gaps and overlaps. That is a second document model, and it
//! would drag playback, export, the effects rack, delivery pricing and the peak cache along with
//! it — every one of which today knows how to handle exactly one thing: *a buffer*.
//!
//! But the ask does not need it. The pieces are dropped **at the other cut points** — the parts
//! land where other parts already start. That is not free positioning; it is a **permutation**.
//! And a permutation of a buffer is still a buffer.
//!
//! So the document stays one buffer and grows a list of **cuts**. `n` cuts make `n + 1` pieces;
//! reordering rewrites the buffer and recomputes the cuts; nothing downstream notices. The whole
//! feature is a list of `usize` and two edits.
//!
//! ## The two edits
//!
//! - **Reorder** — a permutation. Every sample survives, exactly once: the output is the input's
//!   multiset, which is a property strong enough to test directly (`reorder_is_a_permutation`).
//! - **Stretch** — WSOLA time-scaling of one piece (same pitch, new length), rippling what
//!   follows. The "reduce the time without cutting the track" tool.
//!
//! Both move audio, so both carry the clip's [`Structure`](crate::structure::Structure) across a
//! [`FrameMap`] rather than leaving markers and cuts sitting on frame numbers whose audio walked
//! away underneath them.

use std::ops::Range;

use ph2d_audio::SampleData;

use crate::EditClip;
use crate::structure::FrameMap;

/// How far a single Scale gesture may take a piece. A drag is already bounded by the waveform
/// view (you cannot drop past the right edge), so this is the backstop, not the ergonomics:
/// without it a repeated 10× stretch is an unbounded allocation, and the editor's memory budget
/// is measured now (ADR-0117), not merely declared.
const MAX_STRETCH: f64 = 8.0;

/// The piece boundaries: `0`, every cut, and the end. `n` cuts give `n + 2` boundaries and
/// `n + 1` pieces.
pub fn boundaries(cuts: &[usize], frames: usize) -> Vec<usize> {
    let mut b = Vec::with_capacity(cuts.len() + 2);
    b.push(0);
    b.extend(cuts.iter().copied().filter(|&c| c > 0 && c < frames));
    b.push(frames);
    b
}

/// The pieces `[start..end)` the cuts divide the clip into. Always at least one (the whole clip).
pub fn ranges(cuts: &[usize], frames: usize) -> Vec<Range<usize>> {
    let b = boundaries(cuts, frames);
    b.windows(2).map(|w| w[0]..w[1]).collect()
}

impl EditClip {
    /// The cut positions (frames), sorted and strictly inside the clip.
    pub fn cuts(&self) -> &[usize] {
        &self.structure.cuts
    }

    /// The pieces the cuts divide the clip into — one range each, in order. Never empty: with no
    /// cuts, the whole clip is one piece, which is what makes Move and Scale mean something
    /// before you have split anything.
    pub fn pieces(&self) -> Vec<Range<usize>> {
        ranges(&self.structure.cuts, self.frame_count())
    }

    /// Which piece `frame` falls in.
    pub fn piece_at(&self, frame: usize) -> usize {
        self.structure
            .cuts
            .iter()
            .take_while(|&&c| c <= frame)
            .count()
    }

    /// The boundary index (`0..=piece_count`) nearest to `frame` — where a dragged piece would
    /// drop. Boundary `i` is the seam *before* piece `i`; the last is the end of the clip.
    pub fn nearest_boundary(&self, frame: usize) -> usize {
        let b = boundaries(&self.structure.cuts, self.frame_count());
        b.iter()
            .enumerate()
            .min_by_key(|&(_, pos)| pos.abs_diff(frame))
            .map_or(0, |(i, _)| i)
    }

    /// Cut the clip at `frame` — the Split button, at the playhead.
    ///
    /// Structure only: not one sample moves. It is still an undo step, because it changes what the
    /// user sees and what the next Move will act on, and a change you cannot take back is not a
    /// change you can experiment with.
    pub fn split_at(&mut self, frame: usize) -> bool {
        if frame == 0 || frame >= self.frame_count() || self.structure.cuts.contains(&frame) {
            return false;
        }
        let mut cuts = self.structure.cuts.clone();
        cuts.push(frame);
        cuts.sort_unstable();
        self.commit_structure(cuts)
    }

    /// Cut the clip at every marker — one recording of N takes becomes N pieces.
    ///
    /// This **splits the clip**, and that is all it does. It used to encode the pieces to disk and
    /// adopt them as a variation set; emitting files is a *delivery* verb and now lives under one
    /// ([`EditClip::piece_clips`] feeds it), while the word "split" gets to mean what it says.
    pub fn split_at_markers(&mut self) -> bool {
        let frames = self.frame_count();
        let mut cuts = self.structure.cuts.clone();
        cuts.extend(
            self.structure
                .markers
                .iter()
                .map(|m| m.frame)
                .filter(|&f| f > 0 && f < frames),
        );
        cuts.sort_unstable();
        cuts.dedup();
        self.commit_structure(cuts)
    }

    /// Remove every cut — the clip is one piece again. The audio is untouched (the pieces were
    /// never separate buffers), so this un-splits without un-doing any reorder you made.
    pub fn clear_cuts(&mut self) -> bool {
        self.commit_structure(Vec::new())
    }

    /// Drop the cut nearest `frame` within `window` frames (clicking a seam to heal it).
    pub fn remove_cut_near(&mut self, frame: usize, window: usize) -> bool {
        let Some((i, _)) = self
            .structure
            .cuts
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, c.abs_diff(frame)))
            .filter(|&(_, d)| d <= window)
            .min_by_key(|&(_, d)| d)
        else {
            return false;
        };
        let mut cuts = self.structure.cuts.clone();
        cuts.remove(i);
        self.commit_structure(cuts)
    }

    /// The pieces as clips of their own — what **Export Pieces** writes out, and what the
    /// variation importer reads back as one group. Non-destructive.
    pub fn piece_clips(&self) -> Vec<SampleData> {
        crate::clipboard::split_at(&self.data, &self.structure.cuts)
    }

    /// Move piece `from` to boundary `to` — the Move tool's drop.
    ///
    /// `to` is a boundary index in the **current** layout (`0..=piece_count`), which is what the
    /// user is pointing at: the seam they are about to drop onto. Dropping a piece on its own two
    /// seams is a no-op, and says so (`false`), so it never lands an empty undo step.
    ///
    /// One undo step, and the selection follows the piece — the thing you just dragged stays the
    /// thing that is selected.
    pub fn move_piece(&mut self, from: usize, to: usize) -> bool {
        let pieces = self.pieces();
        let n = pieces.len();
        if from >= n || to > n {
            return false;
        }
        // Dropping onto the seam before or after itself leaves the sequence alone. The insertion
        // index is into the list with `from` already lifted out, so a boundary past it shifts down
        // by one — the classic off-by-one of every reorder, worth naming rather than rediscovering.
        let ins = if to <= from { to } else { to - 1 };
        if ins == from {
            return false;
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.remove(from);
        order.insert(ins, from);

        let ch = self.data.format().channel_count().max(1);
        let src = self.data.samples();
        // One buffer, written once (ADR-0117 D2): each piece is memcpy'd straight into its new
        // home. A permutation cannot change the length, so the output is exactly as long as the
        // input — and holds exactly the same samples.
        let out = SampleData::build(src.len(), self.data.format(), |dst| {
            let mut at = 0usize;
            for &p in &order {
                let r = &pieces[p];
                let (s, e) = (r.start * ch, r.end * ch);
                dst[at..at + (e - s)].copy_from_slice(&src[s..e]);
                at += e - s;
            }
        });

        // The cuts fall out of the new lengths: no arithmetic on the old ones, so nothing can drift.
        let mut cuts = Vec::with_capacity(n.saturating_sub(1));
        let mut at = 0usize;
        for &p in order.iter().take(n - 1) {
            at += pieces[p].len();
            cuts.push(at);
        }

        let map = FrameMap::permute(&pieces, &order);
        self.commit_moved(out, cuts, &map);
        // Re-select the piece where it landed, so it is still under the cursor that dropped it.
        let landed = order.iter().position(|&p| p == from).unwrap_or(0);
        let new_pieces = self.pieces();
        self.set_selection(new_pieces.get(landed).cloned());
        true
    }

    /// Time-stretch piece `idx` to `new_len` frames — the Scale tool.
    ///
    /// Same pitch, new length: the take gets shorter (or longer) **without being cut**. What
    /// follows ripples along, so the clip grows or shrinks by the difference.
    pub fn stretch_piece(&mut self, idx: usize, new_len: usize) -> bool {
        let pieces = self.pieces();
        let Some(r) = pieces.get(idx).cloned() else {
            return false;
        };
        let old_len = r.len();
        if old_len == 0 {
            return false;
        }
        // Bounded per gesture (see `MAX_STRETCH`), and never to nothing.
        let lo = ((old_len as f64 / MAX_STRETCH).ceil() as usize).max(1);
        let hi = (old_len as f64 * MAX_STRETCH) as usize;
        let new_len = new_len.clamp(lo, hi);
        if new_len == old_len {
            return false;
        }

        let ch = self.data.format().channel_count().max(1);
        let src = self.data.samples();
        let piece = crate::ops::trim(&self.data, r.clone());
        let stretched = crate::fx::wsola::stretch(&piece, new_len);
        let s = stretched.samples();

        let head = r.start * ch;
        let tail_from = r.end * ch;
        let len = head + s.len() + (src.len() - tail_from);
        // Head, stretched piece, tail — written once (ADR-0117 D2).
        let out = SampleData::build(len, self.data.format(), |dst| {
            dst[..head].copy_from_slice(&src[..head]);
            dst[head..head + s.len()].copy_from_slice(s);
            dst[head + s.len()..].copy_from_slice(&src[tail_from..]);
        });

        // The cuts before the piece do not move, the ones after slide by the difference, and the
        // two that bound the piece land on its new edges — all of which the map already knows.
        let map = FrameMap::stretch(self.frame_count(), r.start, old_len, new_len);
        let cuts = map.cuts(&self.structure.cuts);
        self.commit_moved(out, cuts, &map);
        let new_pieces = self.pieces();
        self.set_selection(new_pieces.get(idx).cloned());
        true
    }
}
