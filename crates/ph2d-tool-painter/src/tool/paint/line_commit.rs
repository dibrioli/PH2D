//! Line editor **commit / cancel / finish** paths — Enter/Apply bake, Apply & Keep, Esc cancel, the
//! right-click finish, plus the small paintable/enter-edit helpers. Split from [`super::line`] so that
//! module stays under the workspace file-LOC cap; a second `impl PainterTool` block over the same private
//! `PaintState` fields (via `super::*`).

use super::*;

impl PainterTool {
    /// End point-creation without baking (right-click / Esc-to-edit): enter the editing phase as its own
    /// undo step. No-op if idle, already editing, or too short (< 2 points) to be a line.
    pub fn line_finish_points(&mut self) -> bool {
        let can = self
            .paint
            .line
            .as_ref()
            .is_some_and(|ed| !ed.is_editing() && ed.point_count() >= 2);
        if !can {
            return false;
        }
        self.begin_shape_txn();
        if let Some(ed) = self.paint.line.as_mut() {
            ed.set_editing();
        }
        self.line_refill();
        self.commit_shape_txn();
        true
    }

    /// Commit the polyline (**Enter / Apply**): end creation, bake the painted line, close the session,
    /// one undo step. Works whether still drawing (≥2 points) or editing. No-op otherwise.
    pub fn line_commit(&mut self) -> bool {
        if !self.line_paintable() {
            return false;
        }
        self.line_enter_edit_phase();
        self.flush_shape_txn();
        let before = self.capture_shape_model();
        self.paint.line = None;
        self.commit_drag_preview();
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after);
        true
    }

    /// Commit the polyline (**Apply & Keep**) but keep the editor open for further reshape — one undo step
    /// whose `after` keeps the editor over the baked pixels (mirrors [`PainterTool::ellipse_commit_keep`]).
    pub fn line_commit_keep(&mut self) -> bool {
        if !self.line_paintable() {
            return false;
        }
        self.line_enter_edit_phase();
        self.flush_shape_txn();
        let before = self.capture_shape_model();
        self.commit_drag_preview();
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after);
        self.paint.drag_preview = None;
        true
    }

    /// Cancel the polyline (**Esc**): revert the painted preview + discard the session, no undo entry.
    pub fn line_cancel(&mut self) -> bool {
        if self.paint.line.is_none() {
            return false;
        }
        self.paint.line = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        true
    }

    /// Drop the Line session without touching pixels — teardown where the canvas is replaced/cleared
    /// (fresh source / tool deactivate), so a restore would read a stale buffer.
    pub(crate) fn line_discard(&mut self) {
        self.paint.line = None;
    }

    /// Whether the editor holds a paintable line (≥ 2 committed points), so Enter/Apply act.
    fn line_paintable(&self) -> bool {
        self.paint
            .line
            .as_ref()
            .is_some_and(|ed| ed.point_count() >= 2)
    }

    /// Move a still-drawing line into the editing phase (used by the commit paths, which then bake and
    /// close). No undo step of its own — the caller records the bake.
    fn line_enter_edit_phase(&mut self) {
        if let Some(ed) = self.paint.line.as_mut() {
            ed.set_editing();
        }
        self.line_refill();
    }
}
