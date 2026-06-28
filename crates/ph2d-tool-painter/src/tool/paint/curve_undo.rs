//! Per-session **undo / redo of curve edits** — point moves, inserts, deletes, handle-kind changes, AND
//! the Offset slider — woven into the painter's Ctrl+Z / Ctrl+Y so editing a curve undoes step-by-step.
//! While a shape is being authored, undo is captured here: it walks the curve's edit history and NEVER
//! Applies the shape nor touches the layer history (Enio 2026-06-27). A gesture snapshots the editable
//! state at its start ([`CurveEditor::begin_edit`]) and keeps it iff the curve changed
//! ([`CurveEditor::commit_edit`]); an Offset drag coalesces into one step. Split from `curve` for the LOC cap.

use super::curve::CurveEditor;
use super::curve_handle::HandleKind;
use super::*;

/// A restorable snapshot of the editable curve state + the Offset slider (everything paint derives from).
#[derive(Clone)]
pub(super) struct EditSnapshot {
    points: Vec<[f32; 2]>,
    handles: Vec<[[f32; 2]; 2]>,
    kinds: Vec<HandleKind>,
    selected: Option<usize>,
    added_point: bool,
    offset: f32,
}

impl CurveEditor {
    /// Clone the current editable state + `offset` into a snapshot.
    fn snapshot(&self, offset: f32) -> EditSnapshot {
        EditSnapshot {
            points: self.points.clone(),
            handles: self.handles.clone(),
            kinds: self.kinds.clone(),
            selected: self.selected,
            added_point: self.added_point,
            offset,
        }
    }

    /// Reinstate a snapshot's geometry; the Offset is returned for the caller (it lives in `PaintState`).
    fn restore(&mut self, s: EditSnapshot) -> f32 {
        self.points = s.points;
        self.handles = s.handles;
        self.kinds = s.kinds;
        self.selected = s.selected;
        self.added_point = s.added_point;
        s.offset
    }

    /// Tentatively record the pre-edit state at a gesture's start (pointer-down); a point gesture also ends
    /// any Offset-drag coalescing. [`Self::commit_edit`] drops it if the gesture was a no-op.
    pub(super) fn begin_edit(&mut self, offset: f32) {
        let s = self.snapshot(offset);
        self.edit_undo.push(s);
        self.offset_dragging = false;
    }

    /// Close a gesture (pointer-up): keep the tentative snapshot only if the curve / Offset changed, else
    /// drop it. A real change clears the redo branch.
    pub(super) fn commit_edit(&mut self, offset: f32) {
        let changed = self.edit_undo.last().is_some_and(|s| {
            s.points != self.points
                || s.handles != self.handles
                || s.kinds != self.kinds
                || s.offset != offset
        });
        if changed {
            self.edit_redo.clear();
        } else {
            self.edit_undo.pop();
        }
        self.offset_dragging = false;
    }

    /// Record one discrete edit (delete / handle-kind) — snapshot the pre-edit state + clear redo.
    pub(super) fn push_edit(&mut self, offset: f32) {
        let s = self.snapshot(offset);
        self.edit_undo.push(s);
        self.edit_redo.clear();
        self.offset_dragging = false;
    }

    /// Snapshot the start of an Offset-slider drag as ONE undo step — coalesced (a contiguous run of slider
    /// values from the same baseline collapses to a single entry until another action breaks the drag).
    pub(super) fn begin_offset_edit(&mut self, offset: f32) {
        if self.offset_dragging {
            return;
        }
        let s = self.snapshot(offset);
        self.edit_undo.push(s);
        self.edit_redo.clear();
        self.offset_dragging = true;
    }

    /// Undo one edit: restore the previous snapshot, pushing the current state (geometry + `cur_offset`) onto
    /// redo. Returns the Offset to reinstate, or `None` when there's nothing to undo.
    fn undo_edit(&mut self, cur_offset: f32) -> Option<f32> {
        let prev = self.edit_undo.pop()?;
        let cur = self.snapshot(cur_offset);
        self.edit_redo.push(cur);
        Some(self.restore(prev))
    }

    /// Redo one edit: mirror of [`Self::undo_edit`].
    fn redo_edit(&mut self, cur_offset: f32) -> Option<f32> {
        let next = self.edit_redo.pop()?;
        let cur = self.snapshot(cur_offset);
        self.edit_undo.push(cur);
        Some(self.restore(next))
    }
}

impl PainterTool {
    /// `true` while any on-canvas shape (Curve / Free Hand / Circle / Polygon) is being authored — undo /
    /// redo are then captured by the shape session instead of Applying it or walking the layer history.
    pub(crate) fn is_editing_shape(&self) -> bool {
        self.paint.curve.as_ref().is_some_and(|ed| ed.editing)
            || self.paint.circle.is_some()
            || self.paint.polygon.is_some()
    }

    /// Undo one in-session curve edit (geometry or Offset), re-filling the preview. When the edit history is
    /// exhausted, the next undo cancels the **creation** itself (reverts the painted curve + resets the
    /// Offset), so a curve folds into the same undo sequence as the paint flow (Enio 2026-06-28). `true` when
    /// something was undone. A no-op for a Circle / Polygon (no editable history).
    pub(crate) fn curve_undo_edit(&mut self) -> bool {
        let cur = self.paint.shape_offset_norm;
        let restored = self
            .paint
            .curve
            .as_mut()
            .filter(|ed| ed.editing)
            .and_then(|ed| ed.undo_edit(cur));
        if let Some(off) = restored {
            self.paint.shape_offset_norm = off;
            self.curve_refill();
            return true;
        }
        // No edits left → undo the creation: cancel the session (reverts the canvas + resets the Offset).
        self.paint.curve.as_ref().is_some_and(|ed| ed.editing) && self.curve_cancel()
    }

    /// Redo one in-session curve edit. Mirror of [`Self::curve_undo_edit`].
    pub(crate) fn curve_redo_edit(&mut self) -> bool {
        let cur = self.paint.shape_offset_norm;
        let restored = self
            .paint
            .curve
            .as_mut()
            .filter(|ed| ed.editing)
            .and_then(|ed| ed.redo_edit(cur));
        if let Some(off) = restored {
            self.paint.shape_offset_norm = off;
            self.curve_refill();
            true
        } else {
            false
        }
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
