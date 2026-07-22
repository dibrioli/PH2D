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

    /// Commit the whole MULTI-SHAPE set (Enter / Apply): bake every shape's painted preview at once (the
    /// active shape and every parked shape) as ONE undo entry, then drop them all — "as curvas desaparecem
    /// todas de uma vez" (Enio 2026-07-04). No parked shapes ⇒ byte-identical to the former single-shape Apply.
    /// Returns `true` when something committed (an editable active shape OR any parked shape).
    pub fn commit_open_shape(&mut self) -> bool {
        if !self.is_editing_shape() && !self.has_parked_shapes() {
            return false; // nothing open / mid initial-drag — nothing to bake yet
        }
        self.flush_shape_txn(); // close any coalesced Offset drag as its own entry first
        let before = self.capture_shape_model(); // active + parked + live preview (for undo of the Apply)
        // Drop every editor + the parked set; the painted pixels (all shapes) stay via `commit_drag_preview`.
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        self.clear_parked_shapes();
        self.commit_drag_preview(); // drop the restore record → all shapes stay baked
        self.paint.shape_offset_base_px = 0.0; // the offset baked into the pixels → reset the Offset
        self.paint.shape_offset_norm = 0.5;
        let after = self.capture_shape_model(); // nothing open, pixels baked
        self.undo.record_structural(before, after); // Apply = one undo entry for the whole set
        true
    }

    /// Cancel the whole MULTI-SHAPE set (Esc / leaving the method): revert every shape's preview to pristine
    /// and drop them all. Returns `true` when a set was open.
    pub fn cancel_open_shape(&mut self) -> bool {
        // The pixels are peeled back to pristine — so the relief has to go with them. A cancelled shape
        // whose envelope survived would be a ridge the light shades over paint that is no longer there:
        // the same ghost the Eraser gate refuses, arriving through the Esc key.
        self.reset_stroke_height();
        self.drop_live_relief();
        // The SCULPT obeys the same sentence, and could not obey it for free: it stages nothing in an
        // envelope, it rewrites the layer's plane LIVE. So a cancelled Curve used to leave its carving
        // behind — shape gone, smoothing permanent, and NO undo entry, because the shape was never
        // committed. This puts the relief back from the frozen `pre`, then drops the session.
        self.cancel_sculpt_session();
        let cancelled = self.curve_cancel()
            || self.ellipse_cancel()
            || self.polygon_cancel()
            || self.line_cancel();
        if self.has_parked_shapes() {
            // Peel any preview the parked shapes are still stamped into, then drop the whole set.
            self.peel_drag_preview(); // doc 21: cancel door — owned peel, stash cleared
            self.clear_parked_shapes();
            self.paint.stroke_undo = None;
            self.paint.wet_shape_active = false; // #3: end the shape wash session (ground rebuilds next)
            return true;
        }
        self.paint.wet_shape_active = false; // #3: end the shape wash session on cancel
        cancelled
    }

    /// Drop any open shape editor + the whole parked set without touching pixels — for teardown where the
    /// canvas is replaced or cleared (fresh source / deactivate / non-paintable layer).
    pub(crate) fn discard_open_shape(&mut self) {
        // Teardown (fresh source / deactivate): the canvas under the shape is going away, so its
        // un-committed relief must not outlive it either.
        self.reset_stroke_height();
        self.drop_live_relief();
        self.curve_discard();
        self.ellipse_discard();
        self.polygon_discard();
        self.line_discard();
        self.clear_parked_shapes();
        self.paint.wet_shape_active = false; // #3: end any shape wash session on teardown
    }

    /// Route a Stroke-panel **shape button** Click (Apply / Apply & Keep / Delete / Edit / Operation mode /
    /// Simplify) to its verb. `true` when `id` matched one — the caller returns early. Split from the panel
    /// dispatcher in [`super::stencil`] for that file's LOC cap; the shape verbs already live in this module.
    pub(crate) fn route_stroke_shape_button(&mut self, id: &ph2d_a11y::NodeId) -> bool {
        use ph2d_editor_core::ids as core_ids;
        if *id == core_ids::PAINTER_BRUSH_STROKE_APPLY {
            self.commit_open_shape(); // Enter / Apply — bake the whole multi-shape set, one undo entry
        } else if *id == core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP {
            self.commit_open_shape_keep(); // bake but keep every shape editable
        } else if *id == core_ids::PAINTER_BRUSH_STROKE_DELETE {
            self.cancel_open_shape(); // drop the open set without baking
        } else if *id == core_ids::PAINTER_BRUSH_STROKE_EDIT {
            self.convert_open_shape_to_curve(); // Ellipse/Polygon → editable Bézier curve
        } else if let Some(i) = core_ids::PAINTER_STROKE_OP_IDS.iter().position(|x| x == id) {
            self.set_stroke_op_mode(i as u8); // multi-shape Operation for the NEXT shape
        } else if *id == core_ids::PAINTER_BRUSH_STROKE_SIMPLIFY {
            self.curve_simplify(); // re-fit the editable curve to a clean control polygon
        } else if *id == core_ids::PAINTER_BRUSH_STROKE_MERGE {
            self.merge_open_shapes_to_curves(); // fold all fillable shapes into one/few dense curves
        } else {
            return false;
        }
        true
    }

    /// Commit whichever on-canvas shape editor is open but KEEP it editable (the **Apply & Keep** button)
    /// — the keep-mode aggregator paired with [`PainterTool::commit_open_shape`]. At most one is open.
    pub fn commit_open_shape_keep(&mut self) -> bool {
        self.curve_commit_keep()
            || self.ellipse_commit_keep()
            || self.polygon_commit_keep()
            || self.line_commit_keep()
    }

    /// **Simplify** the editable curve (the Simplify button): re-fit it to as FEW anchors as the shape allows,
    /// then re-fill. One undo step. `true` when a curve was open and the fit applied. A **closed** curve uses
    /// the [`super::curve_refit`] quality funnel (corner-split + piecewise Schneider least-squares fit — the
    /// Inkscape/paper.js simplify pipeline; Enio 2026-07-05 "curvas simplificadas perfeitas"): smooth joins are
    /// **Aligned**, corners **Free**, and each press escalates the fit tolerance until ~20% more anchors shed
    /// (progressive). An **open** curve keeps the Free Hand Schneider fit (its capture fits fine open).
    pub fn curve_simplify(&mut self) -> bool {
        use super::curve::{
            FREEHAND_FIT_ERROR, MAX_CURVE_POINTS, SIMPLIFY_KEEP_FRACTION, SIMPLIFY_MIN_POINTS,
        };
        use super::curve_handle::{self, HandleKind};
        self.begin_shape_txn(); // re-fitting the curve is one undo step (dropped below if it no-ops)
        let Some(ed) = self.paint.curve.as_ref().filter(|e| e.editing) else {
            self.paint.stroke_undo = None; // nothing open → discard the speculative txn
            return false;
        };
        let closed = ed.model.closed;
        if closed {
            let Some(r) = super::curve_refit::refit_progressive(
                &ed.model.points,
                &ed.model.handles,
                SIMPLIFY_KEEP_FRACTION,
                SIMPLIFY_MIN_POINTS,
            ) else {
                self.paint.stroke_undo = None; // nothing left to shed (e.g. already at the corner anchors)
                return false;
            };
            let ed = self.paint.curve.as_mut().expect("curve present");
            ed.model = super::curve_model::CurveModel {
                points: r.points,
                handles: r.handles,
                kinds: r.kinds,
                selected: None,
                closed: true,
            };
            self.curve_refill();
            // COALESCED: a run of progressive Simplify presses = ONE undo step back to the pre-run curve.
            self.commit_shape_txn_coalesced(crate::undo::CoalesceKind::Simplify);
            return true;
        }
        let Some((p, h)) = super::curve_geom::simplify_curve(
            &ed.model.points,
            &ed.model.handles,
            false,
            FREEHAND_FIT_ERROR,
            MAX_CURVE_POINTS,
        ) else {
            self.paint.stroke_undo = None; // fit declined → discard the speculative txn
            return false;
        };
        let ed = self.paint.curve.as_mut().expect("curve present");
        ed.model.kinds = vec![HandleKind::Aligned; p.len()];
        (ed.model.points, ed.model.handles, ed.model.selected) = (p, h, None);
        curve_handle::rebuild(
            &ed.model.points,
            &ed.model.kinds,
            &mut ed.model.handles,
            ed.model.closed,
        );
        self.curve_refill();
        // COALESCED: repeated open-curve Simplify presses also collapse to one undo step.
        self.commit_shape_txn_coalesced(crate::undo::CoalesceKind::Simplify);
        true
    }
}
