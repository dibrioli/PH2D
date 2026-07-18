//! Authoring the clip stack: the document-level edits the panel drives.
//!
//! Lives beside `doc.rs` rather than in it — the document is already the biggest
//! file in the crate, and the stack is a self-contained thing to add, edit and
//! delete. Every mutation here keeps the two invariants the evaluator rests on:
//! strips are **sorted by start time** (a neighbour only means something in
//! order) and every strip's `clip` index **points at the clip it names**.

use crate::doc::TimelineDoc;
use crate::stack::{ClipLane, ClipStrip, StripId, StripSource};

/// How many lanes a stack may hold.
///
/// A real bound, for the same reason [`crate::MAX_CLIPS`] is one: each lane's
/// header carries hit ids (mute, mode, weight) and the chrome cannot mint a
/// `NodeId` at runtime, so the cap is however many ids the panel addresses. It
/// lives here, with the data, where [`TimelineDoc::add_lane`] can refuse rather
/// than let the panel paint a lane nothing can click.
pub const MAX_LANES: usize = 8;

impl TimelineDoc {
    /// Append an empty lane and return its index, or `None` past [`MAX_LANES`].
    pub fn add_lane(&mut self, name: String) -> Option<usize> {
        if self.stack().len() >= MAX_LANES {
            return None;
        }
        self.stack_mut().push(ClipLane::new(name));
        Some(self.stack().len() - 1)
    }

    /// Remove a lane and everything on it.
    pub fn remove_lane(&mut self, lane: usize) -> bool {
        if lane >= self.stack().len() {
            return false;
        }
        self.stack_mut().remove(lane);
        true
    }

    /// A name no lane is using yet ("Lane 1", "Lane 2", …).
    #[must_use]
    pub fn fresh_lane_name(&self) -> String {
        let mut n = self.stack().len() + 1;
        loop {
            let name = format!("Lane {n}");
            if !self.stack().iter().any(|l| l.name == name) {
                return name;
            }
            n += 1;
        }
    }

    /// Place `clip` on `lane` over `[t_start, t_end)`, playing the whole clip.
    /// Returns the new strip's identity, or `None` if the lane or clip is gone.
    pub fn add_strip(
        &mut self,
        lane: usize,
        clip: usize,
        t_start: f64,
        t_end: f64,
    ) -> Option<StripId> {
        if lane >= self.stack().len() || clip >= self.clips().len() {
            return None;
        }
        let src_len = self.clips()[clip].clip.duration().to_seconds().max(
            // A clip whose duration was never authored is as long as its last key
            // — the same rule the transport's "go to end" uses. A strip sized to a
            // duration of zero would be a strip nobody can see or grab.
            self.clip_end_seconds(clip),
        );
        let id = self.alloc_strip_id();
        let clip_ix = u16::try_from(clip).ok()?;
        let mut strip =
            ClipStrip::new(StripSource::Clip(clip_ix), t_start, t_end, src_len).with_id(id);
        // **`slice == span * speed` is the lane's invariant, and it has to be TRUE
        // at birth, not merely assumed later.** `stretch_strip` re-derives the rate
        // from the span (`speed = slice / span`), so a strip that started out with a
        // span its slice does not fill would jump to a wildly different rate on the
        // first hair of a stretch — a clip of 0.4 s dropped into a 1 s span went
        // from speed 1.0 to 0.4 the instant the drag began.
        let span = strip.span();
        if strip.slice() > 0.0 && span > 0.0 {
            strip.speed = strip.slice() / span;
        }
        self.stack_mut()[lane].insert(strip);
        Some(id)
    }

    /// Copy a strip and lay the copy down **immediately after** the original,
    /// same length, same slice, same rate.
    ///
    /// Adjacent, not overlapping — a duplicate that landed on top of its source
    /// would come up as a crossfade of a clip with itself, which is a null edit
    /// dressed as a blend. Butted end-to-start it reads as a repeat, and dragging
    /// it left by hand IS the crossfade (that is the whole gesture, ADR-0115 R1).
    pub fn duplicate_strip(&mut self, lane: usize, id: StripId) -> Option<StripId> {
        let mut copy = self.strip(lane, id)?.clone();
        let new_id = self.alloc_strip_id();
        let span = copy.span();
        copy.id = new_id;
        copy.t_start += span;
        copy.t_end += span;
        self.stack_mut().get_mut(lane)?.insert(copy);
        Some(new_id)
    }

    /// Drop a strip from a lane.
    pub fn remove_strip(&mut self, lane: usize, id: StripId) -> bool {
        let Some(l) = self.stack_mut().get_mut(lane) else {
            return false;
        };
        let Some(i) = l.index_of(id) else {
            return false;
        };
        l.strips.remove(i);
        true
    }

    /// A strip, by identity rather than position (see [`StripId`]).
    pub fn strip_mut(&mut self, lane: usize, id: StripId) -> Option<&mut ClipStrip> {
        let l = self.stack_mut().get_mut(lane)?;
        let i = l.index_of(id)?;
        l.strips.get_mut(i)
    }

    /// A strip, read-only.
    #[must_use]
    pub fn strip(&self, lane: usize, id: StripId) -> Option<&ClipStrip> {
        let l = self.stack().get(lane)?;
        l.strips.get(l.index_of(id)?)
    }

    /// **Every strip that named a clip must still name it after a clip is
    /// deleted.** Clips live in a `Vec` and a strip stores an *index* into it, so
    /// removing clip `gone` slides every later clip down one — and every strip
    /// pointing above the hole would silently start playing its neighbour.
    ///
    /// Strips of the deleted clip are removed outright: a strip whose clip does
    /// not exist has nothing to play, and leaving it would be a dead item that
    /// paints and cannot be evaluated.
    ///
    /// ⚠️ **Every stack, not just the document's** (ADR-0133): a container's interior is a
    /// stack too, and its strips index the SAME clip list. Repointing only the document's
    /// lanes would leave a container quietly playing its deleted clip's neighbour — the exact
    /// bug this function exists to prevent, hiding one level down where nobody is looking.
    /// Container strips are untouched: they index the *container* list, which this deletion
    /// does not move.
    pub(crate) fn repoint_strips_after_clip_removal(&mut self, gone: usize) {
        let Ok(gone_ix) = u16::try_from(gone) else {
            return;
        };
        let repoint = |lanes: &mut Vec<ClipLane>| {
            for lane in lanes.iter_mut() {
                lane.strips
                    .retain(|s| s.source != StripSource::Clip(gone_ix));
                for s in &mut lane.strips {
                    if let StripSource::Clip(c) = s.source
                        && c > gone_ix
                    {
                        s.source = StripSource::Clip(c - 1);
                    }
                }
            }
        };
        repoint(self.stack_mut());
        for i in 0..self.containers().len() {
            if let Some(lanes) = self.container_stack_mut(i) {
                repoint(lanes);
            }
        }
    }
}
