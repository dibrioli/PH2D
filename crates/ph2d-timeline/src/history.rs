//! [`TimelineHistory`] — bounded undo/redo over [`TimelineDoc`] snapshots.
//!
//! Mirrors `ph2d-vec-edit`'s `History`: `begin` snapshots the doc at a gesture's
//! start, `commit_if_changed` at its end turns it into an undo step **only if
//! the doc actually changed** (content equality, so a click that moved nothing
//! never pollutes the stack). Atomic operations use [`TimelineHistory::push`]
//! directly. Panel-only state (selection, pan/zoom, flags) is **not** snapshotted
//! — it is never undoable (per plan B2/T4).

use crate::doc::TimelineDoc;
use crate::state::SelectedKey;

/// Maximum number of undo steps kept (older steps drop off the bottom).
pub const HISTORY_CAP: usize = 128;

/// Bounded undo/redo stack of document snapshots.
#[derive(Debug, Default, Clone)]
pub struct TimelineHistory {
    undo: Vec<TimelineDoc>,
    redo: Vec<TimelineDoc>,
    pending: Option<TimelineDoc>,
    /// Keys moved inside the open gesture ([`Self::note_moved`]).
    moved: Vec<SelectedKey>,
    /// ⭐ As covas das purgas de donos mortos ([`ph2d_ecs::graveyard`]) — o undo global que devolve
    /// o objecto devolve os clipes no mesmo quadro (`ph2d-timeline-persist`).
    pub graveyard: ph2d_ecs::Graveyard<TimelineDoc>,
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

    /// The keys this gesture has moved so far: they absorb what lies under them only where the
    /// gesture closes (`intent_apply`), never at a frame they merely crossed.
    pub(crate) fn note_moved(&mut self, keys: &[SelectedKey]) {
        for k in keys {
            if !self.moved.contains(k) {
                self.moved.push(*k);
            }
        }
    }

    /// Hand over (and forget) the keys this gesture moved.
    pub(crate) fn take_moved(&mut self) -> Vec<SelectedKey> {
        std::mem::take(&mut self.moved)
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
        self.moved.clear();
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

    /// Toda fotografia guardada, mutável — para uma mudança de IDENTIDADE que não é um passo
    /// (a binding que muda de dono, [`crate::place_owner`]): ela tem de valer em todo passado a
    /// que o Ctrl+Z pode voltar.
    pub fn docs_mut(&mut self) -> impl Iterator<Item = &mut TimelineDoc> {
        self.undo
            .iter_mut()
            .chain(self.redo.iter_mut())
            .chain(self.pending.iter_mut())
    }

    /// ⭐ O passo do topo é uma purga enterrada ([`ph2d_ecs::Graveyard::holds`])? Então o Ctrl+Z é do
    /// undo global (ele devolve o objecto, e a cova devolve os clipes) — a fila não o consome.
    #[must_use]
    pub fn top_is_buried(&self) -> bool {
        self.undo.last().is_some_and(|t| self.graveyard.holds(t))
    }

    /// Tira o passo do topo se ele for `pre` — o cemitério a desfazer a purga sozinho, quando o undo
    /// global devolveu o dono. Não vai para o redo: o redo global volta a apagar, e a purga a enterrar.
    pub fn take_top_if(&mut self, pre: &TimelineDoc) -> Option<TimelineDoc> {
        (self.undo.last() == Some(pre)).then(|| self.undo.pop())?
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
