//! The Curve editor's **commit** verbs — Apply (bake + close) and Apply & Keep (bake + re-baseline, keep
//! editable) plus the keep-mode aggregator. Split from [`super::curve`] for the workspace LOC cap; the panel
//! drives them via `route_brush_dab_event` (the Apply / Apply & Keep buttons). The Simplify button is a
//! separate verb ([`super::curve`]'s `curve_simplify`).

use super::*;

impl PainterTool {
    /// Commit the curve (Enter): keep the painted dabs + push one undo entry. `true` when a session was open.
    pub fn curve_commit(&mut self) -> bool {
        let Some(ed) = self.paint.curve.as_ref() else {
            return false;
        };
        if !ed.editing {
            return false; // mid initial-drag — nothing to bake yet
        }
        self.paint.curve = None;
        self.commit_drag_preview(); // drop the restore record → the painted curve stays
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        self.paint.shape_offset_norm = 0.5; // the offset baked into the painted dabs → reset the slider
        true
    }

    /// Commit the curve but KEEP the editor open (Apply & Keep): bake the painted preview + re-baseline so
    /// further edits restore onto the baked canvas. `true` when open. (Simplify is its own button now.)
    pub fn curve_commit_keep(&mut self) -> bool {
        if !self.paint.curve.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.bake_curve_offset(); // lock the offset into the kept geometry, reset the slider (offset now 0)
        self.commit_drag_preview(); // drop the restore record → the painted curve stays baked
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        // Re-baseline onto the baked canvas; KEEP the editor (the curve persists as a re-applicable shape).
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.paint.drag_preview = None;
        true
    }

    /// Commit whichever on-canvas shape editor is open but KEEP it editable (the **Apply & Keep** button)
    /// — the keep-mode aggregator paired with [`PainterTool::commit_open_shape`]. At most one is open.
    pub fn commit_open_shape_keep(&mut self) -> bool {
        self.curve_commit_keep() || self.circle_commit_keep() || self.polygon_commit_keep()
    }
}
