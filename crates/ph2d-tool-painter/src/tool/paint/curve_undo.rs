//! Per-session **undo / redo of curve edits** — point moves, inserts, deletes, handle-kind changes — woven
//! into the painter's Ctrl+Z / Ctrl+Y so editing a curve undoes step-by-step like the rest of the paint
//! flow (instead of the old "first undo commits the shape"). A gesture snapshots the editable state at its
//! start ([`CurveEditor::begin_edit`]) and keeps it iff the curve actually changed ([`CurveEditor::commit_edit`]);
//! navigating restores a snapshot + re-fills the painted preview. Split from `curve` for the workspace LOC cap.

use super::curve::CurveEditor;
use super::curve_handle::HandleKind;
use super::*;

/// A restorable snapshot of the editable curve state (everything the overlay + paint derive from).
#[derive(Clone)]
pub(super) struct EditSnapshot {
    points: Vec<[f32; 2]>,
    handles: Vec<[[f32; 2]; 2]>,
    kinds: Vec<HandleKind>,
    selected: Option<usize>,
    added_point: bool,
}

impl CurveEditor {
    /// Clone the current editable state into a snapshot.
    fn snapshot(&self) -> EditSnapshot {
        EditSnapshot {
            points: self.points.clone(),
            handles: self.handles.clone(),
            kinds: self.kinds.clone(),
            selected: self.selected,
            added_point: self.added_point,
        }
    }

    /// Reinstate a snapshot (the caller re-fills the painted preview afterwards).
    fn restore(&mut self, s: EditSnapshot) {
        self.points = s.points;
        self.handles = s.handles;
        self.kinds = s.kinds;
        self.selected = s.selected;
        self.added_point = s.added_point;
    }

    /// Tentatively record the pre-edit state at a gesture's start (pointer-down). [`Self::commit_edit`]
    /// discards it if the gesture turned out to be a no-op (a pure selection click).
    pub(super) fn begin_edit(&mut self) {
        let s = self.snapshot();
        self.edit_undo.push(s);
    }

    /// Close a gesture (pointer-up): keep the tentative snapshot only if the curve changed, else drop it.
    /// A real change also clears the redo stack (a new edit invalidates the redo branch).
    pub(super) fn commit_edit(&mut self) {
        let changed = self.edit_undo.last().is_some_and(|s| {
            s.points != self.points || s.handles != self.handles || s.kinds != self.kinds
        });
        if changed {
            self.edit_redo.clear();
        } else {
            self.edit_undo.pop();
        }
    }

    /// Record one discrete edit (delete / handle-kind change) — snapshot the pre-edit state + clear redo.
    pub(super) fn push_edit(&mut self) {
        let s = self.snapshot();
        self.edit_undo.push(s);
        self.edit_redo.clear();
    }

    /// Undo one edit: restore the previous snapshot, pushing the current state onto redo. `false` if empty.
    fn undo_edit(&mut self) -> bool {
        let Some(prev) = self.edit_undo.pop() else {
            return false;
        };
        let cur = self.snapshot();
        self.edit_redo.push(cur);
        self.restore(prev);
        true
    }

    /// Redo one edit: restore the next snapshot, pushing the current state back onto undo. `false` if empty.
    fn redo_edit(&mut self) -> bool {
        let Some(next) = self.edit_redo.pop() else {
            return false;
        };
        let cur = self.snapshot();
        self.edit_undo.push(cur);
        self.restore(next);
        true
    }
}

impl PainterTool {
    /// Undo one in-session curve edit if a curve is being edited and has edit history. Re-fills the preview.
    /// `true` when an edit was undone (so the shell's undo stops here, not touching the layer history).
    pub(crate) fn curve_undo_edit(&mut self) -> bool {
        let undone = self
            .paint
            .curve
            .as_mut()
            .is_some_and(|ed| ed.editing && ed.undo_edit());
        if undone {
            self.curve_refill();
        }
        undone
    }

    /// Redo one in-session curve edit. Mirror of [`Self::curve_undo_edit`].
    pub(crate) fn curve_redo_edit(&mut self) -> bool {
        let redone = self
            .paint
            .curve
            .as_mut()
            .is_some_and(|ed| ed.editing && ed.redo_edit());
        if redone {
            self.curve_refill();
        }
        redone
    }

    /// `true` if an in-session curve edit is available to undo (the shell ORs this into its undo-enabled).
    pub(crate) fn can_curve_undo(&self) -> bool {
        self.paint
            .curve
            .as_ref()
            .is_some_and(|ed| ed.editing && !ed.edit_undo.is_empty())
    }

    /// `true` if an undone in-session curve edit is available to redo.
    pub(crate) fn can_curve_redo(&self) -> bool {
        self.paint
            .curve
            .as_ref()
            .is_some_and(|ed| ed.editing && !ed.edit_redo.is_empty())
    }
}
