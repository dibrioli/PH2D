//! [`TimelineHistory`] — bounded undo/redo over [`TimelineDoc`] snapshots.
//!
//! Mirrors `ph2d-vec-edit`'s `History`: `begin` snapshots the doc at a gesture's
//! start, `commit_if_changed` at its end turns it into an undo step **only if
//! the doc actually changed** (content equality, so a click that moved nothing
//! never pollutes the stack). Atomic operations use [`TimelineHistory::push`]
//! directly. Panel-only state (selection, pan/zoom, flags) is **not** snapshotted
//! — it is never undoable (per plan B2/T4).

use crate::doc::TimelineDoc;

/// Maximum number of undo steps kept (older steps drop off the bottom).
pub const HISTORY_CAP: usize = 128;

/// Bounded undo/redo stack of document snapshots.
#[derive(Debug, Default, Clone)]
pub struct TimelineHistory {
    undo: Vec<TimelineDoc>,
    redo: Vec<TimelineDoc>,
    pending: Option<TimelineDoc>,
}

impl TimelineHistory {
    /// An empty history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the current doc before a gesture that may mutate it.
    pub fn begin(&mut self, doc: &TimelineDoc) {
        self.pending = Some(doc.clone());
    }

    /// Close a gesture: if `doc` differs from the [`begin`](Self::begin)
    /// snapshot, push an undo step (and clear redo); otherwise discard it.
    pub fn commit_if_changed(&mut self, doc: &TimelineDoc) {
        if let Some(pre) = self.pending.take()
            && &pre != doc
        {
            self.push(pre);
        }
    }

    /// Discard the pending snapshot without making a step (cancelled gesture).
    pub fn cancel(&mut self) {
        self.pending = None;
    }

    /// Whether a gesture is currently bracketed (a `begin` awaits its commit).
    /// Atomic edits check this so an edit *inside* a gesture does not close it —
    /// a handle drag emits one edit per frame and must undo as a single step.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }

    /// Push a known-changed pre-state directly onto undo (atomic op).
    pub fn push(&mut self, pre: TimelineDoc) {
        if self.undo.len() >= HISTORY_CAP {
            self.undo.remove(0);
        }
        self.undo.push(pre);
        self.redo.clear();
    }

    /// Whether an undo step is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether a redo step is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Undo: return the previous doc; push `current` onto redo.
    pub fn undo(&mut self, current: &TimelineDoc) -> Option<TimelineDoc> {
        let prev = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(prev)
    }

    /// Redo: return the next doc; push `current` back onto undo.
    pub fn redo(&mut self, current: &TimelineDoc) -> Option<TimelineDoc> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prop::PropKind;
    use ph2d_anim::{AnimValue, Interp, RationalTime};

    fn secs(t: f64) -> RationalTime {
        RationalTime::from_seconds(t)
    }

    #[test]
    fn commit_only_pushes_on_real_change() {
        let mut doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        // A no-op gesture leaves the stack empty.
        h.begin(&doc);
        h.commit_if_changed(&doc);
        assert!(!h.can_undo());
        // A real edit becomes one step.
        h.begin(&doc);
        doc.insert_key(
            1,
            PropKind::TranslationX,
            secs(0.0),
            AnimValue::Float(5.0),
            Interp::Linear,
        );
        h.commit_if_changed(&doc);
        assert!(h.can_undo());
    }

    #[test]
    fn undo_redo_restores_snapshots() {
        let mut doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        let empty = doc.clone();
        h.begin(&doc);
        doc.insert_key(
            1,
            PropKind::Rotation,
            secs(0.0),
            AnimValue::Float(1.0),
            Interp::Hold,
        );
        h.commit_if_changed(&doc);
        let edited = doc.clone();

        let back = h.undo(&doc).expect("undo");
        assert_eq!(back, empty);
        assert!(h.can_redo());
        let fwd = h.redo(&back).expect("redo");
        assert_eq!(fwd, edited);
    }
}
