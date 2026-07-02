//! The Curve editor's **commit** verbs — Apply (bake + close) and Apply & Keep (bake + re-baseline, keep
//! editable) plus the keep-mode aggregator. Split from [`super::curve`] for the workspace LOC cap; the panel
//! drives them via `route_brush_dab_event` (the Apply / Apply & Keep buttons). The Simplify button is a
//! separate verb ([`super::curve`]'s `curve_simplify`).

use super::*;

impl PainterTool {
    /// Commit the curve (Enter): keep the painted dabs + push one undo entry. `true` when a session was open.
    pub fn curve_commit(&mut self) -> bool {
        if !self.paint.curve.as_ref().is_some_and(|ed| ed.editing) {
            return false; // none open / mid initial-drag — nothing to bake yet
        }
        self.flush_shape_txn(); // close any coalesced Offset drag as its own entry first
        let before = self.capture_shape_model(); // the open curve (for undo of the Apply)
        self.paint.curve = None;
        self.commit_drag_preview(); // drop the restore record → the painted curve stays baked
        self.paint.shape_offset_base_px = 0.0; // the offset baked into the painted dabs → reset the Offset
        self.paint.shape_offset_norm = 0.5;
        let after = self.capture_shape_model(); // shape gone, pixels baked
        self.undo.record_structural(before, after); // Apply (bake + close) is one undo entry
        true
    }

    /// Commit the curve but KEEP the editor open (Apply & Keep): bake the painted preview as ONE undo entry
    /// whose `after` keeps the editor open over the baked pixels — interleaving with the surrounding shape
    /// edits. The curve geometry is UNCHANGED: the live offset is folded into the accumulator and the slider
    /// re-centred, so the displayed curve doesn't move but the user can keep offsetting in the same direction
    /// (Enio 2026-06-28). `true` when open. (Simplify is its own button now.)
    pub fn curve_commit_keep(&mut self) -> bool {
        if !self.paint.curve.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.flush_shape_txn();
        let before = self.capture_shape_model(); // curve + live preview, pre-bake
        self.accumulate_offset(); // fold the slider into the accumulator + re-centre it (geometry untouched)
        self.commit_drag_preview(); // the painted curve becomes permanent (no live preview left)
        let after = self.capture_shape_model(); // curve kept open (same geometry), pixels baked, no preview
        self.undo.record_structural(before, after);
        self.paint.drag_preview = None;
        true
    }

    /// Commit whichever on-canvas shape editor is open but KEEP it editable (the **Apply & Keep** button)
    /// — the keep-mode aggregator paired with [`PainterTool::commit_open_shape`]. At most one is open.
    pub fn commit_open_shape_keep(&mut self) -> bool {
        self.curve_commit_keep() || self.ellipse_commit_keep() || self.polygon_commit_keep()
    }

    /// **Simplify** the editable curve (the Simplify button): re-fit it to a clean minimal control polygon
    /// via the Free Hand fit, then re-fill. One undo step. `true` when a curve was open and the fit applied.
    /// (Split from [`super::curve`] for the workspace LOC cap; the consts/helpers live there.)
    pub fn curve_simplify(&mut self) -> bool {
        use super::curve::{FREEHAND_FIT_ERROR, MAX_CURVE_POINTS};
        use super::curve_handle::{self, HandleKind};
        self.begin_shape_txn(); // re-fitting the curve is one undo step (dropped below if it no-ops)
        let Some(ed) = self.paint.curve.as_ref().filter(|e| e.editing) else {
            self.paint.stroke_undo = None; // nothing open → discard the speculative txn
            return false;
        };
        let Some((p, h)) = super::curve_geom::simplify_curve(
            &ed.points,
            &ed.handles,
            ed.closed,
            FREEHAND_FIT_ERROR,
            MAX_CURVE_POINTS,
        ) else {
            self.paint.stroke_undo = None; // fit declined → discard the speculative txn
            return false;
        };
        let ed = self.paint.curve.as_mut().expect("curve present");
        ed.kinds = vec![HandleKind::Aligned; p.len()];
        (ed.points, ed.handles, ed.selected) = (p, h, None);
        curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed);
        self.curve_refill();
        self.commit_shape_txn();
        true
    }
}
