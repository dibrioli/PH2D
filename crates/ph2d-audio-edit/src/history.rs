//! The undo timeline: **deltas, capped by bytes** (ADR-0117 D1).
//!
//! ## What was wrong
//!
//! The timeline used to hold 64 whole clips. `SampleData` is an `Arc<[f32]>`, so pushing one was a
//! refcount bump and looked free — but every edit *produces a new buffer*, so the 64 entries were
//! 64 distinct buffers. `MAX_HISTORY = 64` was not a cap on bookkeeping; it was a **64× multiplier
//! on the clip**. Measured: 64 edits on a 3-minute stereo clip peaked at **4351 MB** — the whole
//! app's platform budget is 3500 MB.
//!
//! ## The shape of the fix
//!
//! An edit changes a **range**. A step keeps the samples of that range and nothing else, so fixing
//! one second of a three-minute clip costs a second, not three minutes.
//!
//! Two things make it small:
//!
//! 1. **The step holds only the side that is NOT in the document.** Before the step is undone that
//!    is the old audio; after it is undone it is the new audio. So undo and redo are *the same
//!    operation* — a **swap** — and one delta serves both directions instead of two.
//! 2. **The range is found by diffing**, not by trusting each op to declare what it touched. The
//!    common prefix and suffix are skipped and the rest is the step. That covers effects, trims,
//!    deletes, fades, silence and tail-growth without instrumenting a single one of them — and it
//!    is the same "register by diff at one choke point" that the editor's global undo already uses
//!    (`shells/desktop/src/undo.rs`).
//!
//! A pleasant consequence: **an edit that changes nothing costs no step**, everywhere and by
//! construction. That bug was found by the W5 audit and patched at one call site (a `repair` whose
//! band collapsed still lit the Undo button); now it cannot happen at any of them.
//!
//! ## Why a byte cap and not a count
//!
//! A count ignores what it is counting, which is exactly the bug. Whole-clip edits are *irreducibly*
//! one clip per step — you cannot undo a change to every sample without keeping every sample — so
//! the only honest bound is a **memory** one. Undo depth therefore becomes adaptive: a short SFX
//! gets hundreds of steps, a 66 MB clip gets as many as fit. The user loses old undo instead of
//! losing the application, which is the trade every DAW makes.

use std::collections::VecDeque;

use ph2d_audio::{AudioFormat, SampleData};

/// How much audio the timeline may hold, in bytes (ADR-0117). Sized so that the worst realistic
/// case — repeated **whole-clip** edits on a 3-minute stereo clip, the one that used to reach
/// 4351 MB — stays inside the A2 bar of 512 MB peak once the live buffer and the render transient
/// are counted too.
const HISTORY_BUDGET_BYTES: usize = 256 * 1024 * 1024;

/// One undo step: the samples a single edit displaced.
#[derive(Debug, Clone)]
struct Step {
    /// Interleaved sample index where the old and new buffers stop agreeing.
    at: usize,
    /// The samples for this range that are **not currently in the document**. Undo swaps them in
    /// and takes the document's out; redo swaps them back. One delta, both directions.
    other: Box<[f32]>,
    /// How many interleaved samples this range currently occupies **in** the document. Differs
    /// from `other.len()` whenever the clip grew or shrank — a reverb tail, a trim, a delete.
    in_doc: usize,
    /// The format that goes with `other`. `apply_force_mono` changes the channel count, and then
    /// the two buffers are not comparable sample-for-sample at all; such a step degenerates to a
    /// whole-buffer snapshot, which is honest — nothing *was* preserved.
    other_format: AudioFormat,
}

impl Step {
    /// What this step costs. `Box<[f32]>` is one allocation of exactly this size.
    fn bytes(&self) -> usize {
        self.other.len() * std::mem::size_of::<f32>()
    }

    /// Swap this step's samples with the document's, returning the new document.
    ///
    /// This is undo. It is also redo. After the swap the step holds whatever it displaced, so
    /// running it again is the inverse — which is the whole reason one delta suffices.
    fn swap(&mut self, doc: &SampleData) -> SampleData {
        let src = doc.samples();
        let at = self.at.min(src.len());
        let end = (at + self.in_doc).min(src.len());
        let mid = self.other.len();

        // The new document: head, then this step's samples, then the tail. One pass, one
        // allocation (ADR-0117 D2) — an undo must not cost more memory than the edit did.
        let out = SampleData::from_fn(at + mid + (src.len() - end), self.other_format, |i| {
            if i < at {
                src[i]
            } else if i < at + mid {
                self.other[i - at]
            } else {
                src[end + (i - at - mid)]
            }
        });

        // Now hold what we displaced, so the next swap undoes this one.
        self.other = src[at..end].into();
        self.in_doc = mid;
        self.other_format = doc.format();
        out
    }
}

/// The undo timeline. `applied` steps are behind us; the rest is the redo tail.
#[derive(Debug, Clone)]
pub(crate) struct History {
    steps: VecDeque<Step>,
    /// How many steps are currently applied. `steps[..applied]` can be undone; `steps[applied..]`
    /// is the redo tail.
    applied: usize,
    bytes: usize,
}

impl History {
    pub(crate) fn new() -> Self {
        Self {
            steps: VecDeque::new(),
            applied: 0,
            bytes: 0,
        }
    }

    /// Forget everything (a fresh clip starts a fresh timeline).
    pub(crate) fn clear(&mut self) {
        self.steps.clear();
        self.applied = 0;
        self.bytes = 0;
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.applied > 0
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.applied < self.steps.len()
    }

    /// Bytes of audio the timeline is holding — the number the ADR-0117 gates measure.
    #[cfg(test)]
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// Record the transition `old` → `new` as one step.
    ///
    /// Returns `false` when the two buffers are identical: **a no-op costs no undo step**. That is
    /// a guarantee of the diff, not a check each caller has to remember.
    pub(crate) fn push(&mut self, old: &SampleData, new: &SampleData) -> bool {
        // The redo tail is unreachable the moment a new edit lands.
        while self.steps.len() > self.applied {
            if let Some(s) = self.steps.pop_back() {
                self.bytes -= s.bytes();
            }
        }

        let Some(step) = diff(old, new) else {
            return false;
        };
        self.bytes += step.bytes();
        self.steps.push_back(step);
        self.applied += 1;
        self.enforce_budget();
        true
    }

    /// Step back. `None` at the start of the timeline.
    pub(crate) fn undo(&mut self, doc: &SampleData) -> Option<SampleData> {
        if !self.can_undo() {
            return None;
        }
        self.applied -= 1;
        Some(self.swap_at(self.applied, doc))
    }

    /// Step forward. `None` at the tip.
    pub(crate) fn redo(&mut self, doc: &SampleData) -> Option<SampleData> {
        if !self.can_redo() {
            return None;
        }
        let out = self.swap_at(self.applied, doc);
        self.applied += 1;
        Some(out)
    }

    /// Swap step `i` with the document, keeping the byte count honest.
    ///
    /// A swap **changes the step's size**: it comes out holding the side it displaced, and the two
    /// sides are different lengths whenever the edit changed the clip's length — a reverb tail, a
    /// trim, a delete. Assuming the size is stable across a swap is how the running total goes
    /// stale, and it is what an existing test (`tail_effect_grows_the_clip_and_undo_restores_it`)
    /// caught: the count underflowed on the next commit.
    fn swap_at(&mut self, i: usize, doc: &SampleData) -> SampleData {
        let step = &mut self.steps[i];
        let was = step.bytes();
        let out = step.swap(doc);
        let now = step.bytes();
        self.bytes = self.bytes + now - was;
        out
    }

    /// Drop the oldest steps until the timeline fits its budget.
    ///
    /// The oldest is the only end we *can* drop: every later step's samples are relative to the
    /// state its predecessors produced, so dropping from the middle would break the chain.
    ///
    /// **The most recent step is never dropped**, even if it alone exceeds the budget. A single
    /// edit on a very large clip would otherwise silently arrive with Undo already disabled, and
    /// an edit you cannot take back is a worse failure than an over-budget one.
    ///
    /// Load-bearing: this is only ever reached from `push`, which has just truncated the redo tail
    /// — so every step in the queue is an *applied* one, and taking from the front can only shorten
    /// undo, never destroy a redo. Call it from anywhere else and that stops being true.
    fn enforce_budget(&mut self) {
        while self.bytes > HISTORY_BUDGET_BYTES && self.steps.len() > 1 {
            let Some(s) = self.steps.pop_front() else {
                break;
            };
            self.bytes -= s.bytes();
            self.applied = self.applied.saturating_sub(1);
        }
    }
}

/// Find what an edit changed: skip the common prefix and the common suffix, and the rest is the
/// step. `None` when the buffers are identical.
///
/// Samples are compared **by bits**, not by `==`. `f32`'s `==` says `-0.0 == 0.0` and
/// `NaN != NaN`; an undo timeline needs *identity*, and a flipped sign bit is a real difference
/// that must be restorable. This is also exactly the standard the rack holds itself to — its
/// invariant is *byte*-identical, not *approximately* identical.
fn diff(old: &SampleData, new: &SampleData) -> Option<Step> {
    let (a, b) = (old.samples(), new.samples());

    // A channel-layout change (force-mono) makes the buffers incomparable sample-for-sample: index
    // i means a different thing on each side. Snapshot the whole thing rather than diff nonsense.
    if old.format() != new.format() {
        return Some(Step {
            at: 0,
            other: a.into(),
            in_doc: b.len(),
            other_format: old.format(),
        });
    }

    let same = |x: f32, y: f32| x.to_bits() == y.to_bits();
    let head = a.iter().zip(b).take_while(|(x, y)| same(**x, **y)).count();
    if head == a.len() && a.len() == b.len() {
        return None; // nothing changed — no step, no lit Undo button
    }
    // The common suffix, stopping before it can overlap the prefix on either side.
    let max_tail = (a.len() - head).min(b.len() - head);
    let tail = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take(max_tail)
        .take_while(|(x, y)| same(**x, **y))
        .count();

    Some(Step {
        at: head,
        other: a[head..a.len() - tail].into(),
        in_doc: b.len() - head - tail,
        other_format: old.format(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn buf(samples: Vec<f32>, ch: usize) -> SampleData {
        let format = if ch == 2 {
            AudioFormat::stereo(48_000)
        } else {
            AudioFormat::mono(48_000)
        };
        SampleData::from_interleaved(samples, format)
    }

    fn noise(n: usize, ch: usize, seed: u32) -> SampleData {
        let mut s = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
        buf(
            (0..n * ch)
                .map(|_| {
                    s ^= s << 13;
                    s ^= s >> 17;
                    s ^= s << 5;
                    (s as f32 / u32::MAX as f32) * 2.0 - 1.0
                })
                .collect(),
            ch,
        )
    }

    /// **A7 of ADR-0117.** The delta timeline must land on exactly the buffer a snapshot timeline
    /// would have — for any sequence of edits and any walk of undo/redo through it.
    ///
    /// The oracle is the old design itself: keep every whole buffer, and index into them. If the
    /// deltas ever disagree with that by one bit, an undo has silently corrupted the user's audio,
    /// which is the worst thing an editor can do and the thing a "looks fine" smoke would never
    /// catch.
    ///
    /// The edits deliberately include the ones that change the clip's LENGTH (grow and shrink) and
    /// the one that changes its CHANNEL COUNT — the three cases where a naive delta gets the
    /// splice bounds wrong, and where a step's two sides are different sizes.
    #[test]
    fn undo_and_redo_land_byte_for_byte_where_snapshots_would() {
        for &ch in &[1usize, 2] {
            for seed in 0..12u32 {
                let start = noise(40, ch, seed);

                // The oracle: every state, whole.
                let mut snapshots = vec![start.clone()];
                let mut h = History::new();
                let mut doc = start.clone();

                // A varied edit sequence, driven off the seed so the runs differ.
                for k in 0..8u32 {
                    let n = doc.frame_count();
                    let next = match (seed + k) % 5 {
                        // Whole-clip: every sample changes (the irreducible case).
                        0 => buf(doc.samples().iter().map(|s| s * 0.5).collect(), ch),
                        // A local edit: only a middle range moves.
                        1 => {
                            let mut v = doc.samples().to_vec();
                            for s in v.iter_mut().skip(10 * ch).take(6 * ch) {
                                *s = -*s;
                            }
                            buf(v, ch)
                        }
                        // GROW: append a tail (a reverb ring-out).
                        2 => {
                            let mut v = doc.samples().to_vec();
                            v.extend(std::iter::repeat_n(0.25, 5 * ch));
                            buf(v, ch)
                        }
                        // SHRINK: cut the front (a trim).
                        3 if n > 6 => buf(doc.samples()[4 * ch..].to_vec(), ch),
                        // A no-op — it must NOT produce a step.
                        _ => doc.clone(),
                    };

                    let changed = h.push(&doc, &next);
                    assert_eq!(
                        changed,
                        next.samples() != doc.samples(),
                        "a no-op must cost no undo step, and a real edit must cost one"
                    );
                    if changed {
                        snapshots.push(next.clone());
                    }
                    doc = next;
                }

                // Walk all the way back, checking every state against the snapshot.
                let mut i = snapshots.len() - 1;
                while let Some(prev) = h.undo(&doc) {
                    doc = prev;
                    i -= 1;
                    assert_eq!(
                        doc.samples(),
                        snapshots[i].samples(),
                        "undo diverged at step {i} (ch={ch}, seed={seed})"
                    );
                    assert_eq!(doc.format(), snapshots[i].format());
                }
                assert_eq!(i, 0, "undo stopped short of the original");

                // And all the way forward again.
                while let Some(next) = h.redo(&doc) {
                    doc = next;
                    i += 1;
                    assert_eq!(
                        doc.samples(),
                        snapshots[i].samples(),
                        "redo diverged at step {i} (ch={ch}, seed={seed})"
                    );
                }
                assert_eq!(i, snapshots.len() - 1, "redo stopped short of the tip");
            }
        }
    }

    /// A channel-layout change (force-mono) makes the two buffers incomparable sample-for-sample.
    /// The step degenerates to a whole snapshot — and undo must still restore the *format*, not
    /// just the samples.
    #[test]
    fn a_channel_layout_change_survives_undo() {
        let stereo = noise(20, 2, 5);
        let mono = buf(
            stereo
                .samples()
                .chunks(2)
                .map(|f| (f[0] + f[1]) * 0.5)
                .collect(),
            1,
        );
        let mut h = History::new();
        assert!(h.push(&stereo, &mono));
        let back = h.undo(&mono).expect("undo");
        assert_eq!(back.samples(), stereo.samples());
        assert_eq!(
            back.format(),
            stereo.format(),
            "undo lost the channel layout"
        );
        let fwd = h.redo(&back).expect("redo");
        assert_eq!(fwd.format(), mono.format());
        assert_eq!(fwd.samples(), mono.samples());
    }

    /// **A2's mechanism.** The timeline is capped by BYTES. Whole-clip edits are irreducibly one
    /// clip per step, so a count-based cap (the old `MAX_HISTORY = 64`) was a multiplier on the
    /// clip rather than a bound on anything.
    #[test]
    fn the_budget_bounds_the_bytes_not_the_step_count() {
        // Steps big enough that a handful blows the budget.
        let big = HISTORY_BUDGET_BYTES / (4 * 8); // frames per clip -> ~1/8 budget each
        let mut h = History::new();
        let mut doc = noise(big, 1, 1);
        for k in 0..40u32 {
            // Whole-clip edit: nothing is shared, so each step must hold a whole clip.
            let next = buf(
                doc.samples()
                    .iter()
                    .map(|s| s * (0.9 + k as f32 * 0.001))
                    .collect(),
                1,
            );
            h.push(&doc, &next);
            doc = next;
            assert!(
                h.bytes() <= HISTORY_BUDGET_BYTES,
                "the timeline blew its budget: {} > {}",
                h.bytes(),
                HISTORY_BUDGET_BYTES
            );
        }
        // It kept SOME undo — a bound that reduces the feature to nothing is not a bound, it is a
        // removal.
        assert!(h.can_undo(), "the budget ate the entire timeline");
    }

    /// Local edits are cheap — that is the whole point of the delta. Sixty-four one-frame edits in
    /// a big clip must not cost sixty-four clips.
    #[test]
    fn a_local_edit_costs_the_locality_not_the_clip() {
        let frames = 100_000;
        let clip_bytes = frames * 4;
        let mut h = History::new();
        let mut doc = noise(frames, 1, 9);
        for k in 0..64usize {
            let mut v = doc.samples().to_vec();
            v[k * 10] = 0.5;
            let next = buf(v, 1);
            h.push(&doc, &next);
            doc = next;
        }
        // 64 single-sample edits: the old design held 64 whole clips.
        assert!(
            h.bytes() < clip_bytes,
            "64 one-sample edits cost {} bytes; a single clip is {clip_bytes}",
            h.bytes()
        );
    }
}
