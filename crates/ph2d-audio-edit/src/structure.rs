//! The clip's **structure** — everything positional that is glued to the audio but is not the
//! audio — and the map that carries it across an edit that moves audio around.
//!
//! ## Why this is one thing
//!
//! Cuts, markers and the loop region are all *frame positions*. They look like independent
//! little features, and they were written as such: markers were "metadata, survives undo", the
//! loop clamped itself, and cuts did not exist. That worked only because no edit had ever
//! **moved** audio before — every op either rewrote samples in place or chopped off an end.
//!
//! Reordering pieces moves audio. The moment it does, the three of them stop being independent:
//! a marker on a footstep has to follow the footstep, a cut has to stay on the seam it names,
//! and an undo that restored the samples but not the boundaries would draw the cuts across the
//! wrong audio. So they travel together, in the undo step, as one value.
//!
//! Folding them together also fixes a bug that predates pieces entirely: a ripple delete shifted
//! the samples after it but left the markers where they were, so every marker past the cut
//! silently slid onto different audio. [`FrameMap`] is the one place that can no longer happen.

use std::ops::Range;

use crate::Marker;

/// Everything positional the clip carries besides the samples themselves.
///
/// The selection is deliberately NOT here: it is where the *user* is pointing, not a property of
/// the audio, and no DAW undoes a selection.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Structure {
    /// Cut positions in frames, sorted and strictly inside the clip (`0 < c < frames`). `n` cuts
    /// divide the clip into `n + 1` pieces. A cut at 0 or at the end would name an empty piece.
    pub cuts: Vec<usize>,
    /// Named cue points, sorted by frame.
    pub markers: Vec<Marker>,
    /// The loop region, if set.
    pub loop_region: Option<Range<usize>>,
}

impl Structure {
    /// Roughly what this costs to keep in an undo step. Small next to the audio, but the byte
    /// cap is only honest if it counts everything it is holding.
    pub(crate) fn bytes(&self) -> usize {
        self.cuts.len() * std::mem::size_of::<usize>()
            + self
                .markers
                .iter()
                .map(|m| std::mem::size_of::<Marker>() + m.name.len())
                .sum::<usize>()
            + std::mem::size_of::<Option<Range<usize>>>()
    }

    /// Drop anything an edit pushed outside the (possibly shorter) clip, and restore the
    /// invariants: cuts sorted, unique, strictly inside; markers sorted.
    ///
    /// The safety net, not the mechanism — [`FrameMap`] is what *moves* positions. This only
    /// catches what a length change left dangling.
    pub(crate) fn clamp(&mut self, frames: usize) {
        self.cuts.retain(|&c| c > 0 && c < frames);
        self.cuts.sort_unstable();
        self.cuts.dedup();
        self.markers.retain(|m| m.frame < frames);
        self.markers.sort_by_key(|m| m.frame);
        self.loop_region = self.loop_region.take().and_then(|r| {
            let (s, e) = (r.start.min(frames), r.end.min(frames));
            (s < e).then_some(s..e)
        });
    }

    /// Carry the markers and the loop across a structural edit, dropping what the edit destroyed.
    ///
    /// **Cuts are not remapped here — the caller supplies them.** For a splice they *could* be
    /// (`map.cuts` does it), but for a **reorder** they cannot: a cut is the start of the piece
    /// after it, so remapping it would send piece 0's start to wherever piece 0 was dropped —
    /// producing a cut in the middle of nothing and losing the seam that actually exists. After a
    /// permutation the cuts are re-derived from the new layout, where they are exact by
    /// construction. Making that the rule for every structural edit means there is no arithmetic
    /// on old cut positions anywhere, so there is nothing for a drift bug to live in.
    pub(crate) fn remap(&mut self, map: &FrameMap) {
        self.markers.retain_mut(|m| match map.frame(m.frame) {
            Some(f) => {
                m.frame = f;
                true
            }
            None => false,
        });
        self.loop_region = self.loop_region.take().and_then(|r| map.range(r));
    }
}

/// Where the audio went: a piecewise-linear map from **old** frame to **new** frame.
///
/// Every structural edit is the same shape — some runs of the old clip survive, in some order,
/// possibly stretched, and everything else is gone. A delete keeps two runs; a paste keeps two
/// and pushes the second along; a stretch keeps three and scales the middle; a reorder keeps one
/// run per piece and permutes them. One type expresses all four, so there is one place where a
/// position can be carried wrong instead of four.
#[derive(Debug, Clone, Default)]
pub(crate) struct FrameMap {
    /// `(old_start, old_len, new_start, new_len)`, in old-timeline order. Old frames not covered
    /// by any run were destroyed by the edit.
    runs: Vec<(usize, usize, usize, usize)>,
}

impl FrameMap {
    /// `old[at..at+removed]` replaced by `inserted` brand-new frames. Positions inside the
    /// replaced span are **gone** — the audio they named is not there any more.
    ///
    /// This is a delete (`inserted = 0`), a paste (`removed = 0`, or the selection it replaced),
    /// and an overwrite, all at once.
    pub(crate) fn splice(len: usize, at: usize, removed: usize, inserted: usize) -> Self {
        let at = at.min(len);
        let end = (at + removed).min(len);
        let mut runs = Vec::with_capacity(2);
        if at > 0 {
            runs.push((0, at, 0, at));
        }
        if end < len {
            runs.push((end, len - end, at + inserted, len - end));
        }
        Self { runs }
    }

    /// Only `range` survives, moved to the front — a trim. Everything outside it is gone.
    pub(crate) fn keep(range: Range<usize>) -> Self {
        Self {
            runs: vec![(range.start, range.len(), 0, range.len())],
        }
    }

    /// `old[at..at+old_len]` time-stretched to `new_len` frames. Positions inside **scale**: a
    /// marker halfway through a piece is still halfway through it after the piece is stretched,
    /// which is the whole difference between stretching audio and replacing it.
    pub(crate) fn stretch(len: usize, at: usize, old_len: usize, new_len: usize) -> Self {
        let at = at.min(len);
        let end = (at + old_len).min(len);
        let mut runs = Vec::with_capacity(3);
        if at > 0 {
            runs.push((0, at, 0, at));
        }
        runs.push((at, end - at, at, new_len));
        if end < len {
            runs.push((end, len - end, at + new_len, len - end));
        }
        Self { runs }
    }

    /// The pieces of the old clip, re-laid in `order`. Each surviving run is a whole piece, so a
    /// position follows the piece it was standing on.
    pub(crate) fn permute(pieces: &[Range<usize>], order: &[usize]) -> Self {
        // Where each piece lands, walking the new layout once.
        let mut new_start = vec![0usize; pieces.len()];
        let mut at = 0usize;
        for &p in order {
            new_start[p] = at;
            at += pieces[p].len();
        }
        let runs = pieces
            .iter()
            .enumerate()
            .map(|(i, r)| (r.start, r.len(), new_start[i], r.len()))
            .collect();
        Self { runs }
    }

    /// Where old frame `f` landed, or `None` if the edit destroyed it.
    pub(crate) fn frame(&self, f: usize) -> Option<usize> {
        let &(os, ol, ns, nl) = self.runs.iter().find(|&&(os, ol, _, _)| {
            // A zero-length run covers nothing; `f < os + ol` handles that on its own.
            f >= os && f < os + ol
        })?;
        if ol == nl {
            return Some(ns + (f - os));
        }
        // A stretched run: keep the fraction, not the offset. Computed in the wider type so a
        // long piece cannot overflow the product on the way.
        let frac = (f - os) as u64 * nl as u64 / (ol.max(1)) as u64;
        Some(ns + frac as usize)
    }

    /// Carry a cut list across a **splice** (never across a reorder — see [`Structure::remap`]).
    /// A cut inside the removed span is dropped: the seam it named is not there any more.
    pub(crate) fn cuts(&self, cuts: &[usize]) -> Vec<usize> {
        cuts.iter().filter_map(|&c| self.frame(c)).collect()
    }

    /// Does this edit keep the audio **in order**? True for a splice, a trim, a stretch — audio
    /// slides, but never leapfrogs. False for a reorder, which is the entire point of a reorder.
    ///
    /// The distinction decides what happens to a range the edit cut through, and it is the
    /// difference between "the loop shrank" and "the loop is meaningless now".
    fn is_monotone(&self) -> bool {
        self.runs.windows(2).all(|w| w[0].2 <= w[1].2)
    }

    /// Where old range `r` landed.
    ///
    /// Inside one run, it simply moves. Cut through by the edit, the answer depends on whether the
    /// edit preserved order:
    ///
    /// - **In order** (delete, trim, stretch): what survives of the range is still one contiguous
    ///   span, so the image is that intersection — delete a second out of the middle of a loop and
    ///   the loop gets a second shorter, which is what every DAW does and what the user meant.
    /// - **Reordered**: the range's audio flew to two different places. There is no honest single
    ///   range to return, so it is dropped — a loop that straddled two pieces is cleared rather
    ///   than silently re-pointed at whatever audio slid into those frame numbers.
    pub(crate) fn range(&self, r: Range<usize>) -> Option<Range<usize>> {
        if r.start >= r.end {
            return None;
        }
        // Every run the range actually touches.
        let hits: Vec<&(usize, usize, usize, usize)> = self
            .runs
            .iter()
            .filter(|&&(os, ol, _, _)| os < r.end && os + ol > r.start)
            .collect();
        let (&(fos, fol, ..), &(los, lol, ..)) = (*hits.first()?, *hits.last()?);
        if hits.len() > 1 && !self.is_monotone() {
            return None;
        }
        // Clamp to the surviving audio on each side. Both land inside a run they intersect, so
        // `frame` cannot come back empty.
        let first_old = fos.max(r.start).min(fos + fol - 1);
        let last_old = (los + lol).min(r.end) - 1;
        let (s, e) = (self.frame(first_old)?, self.frame(last_old)? + 1);
        (s < e).then_some(s..e)
    }
}
