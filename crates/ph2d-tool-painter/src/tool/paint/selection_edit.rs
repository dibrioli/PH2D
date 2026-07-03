//! Selection **gizmo-mode toggle + list operations** (ADR-0103 Am.2 v2). The per-shape editing itself lives
//! in the isolated [`super::selection_gizmo`] module (which never touches the stroke editors). This file
//! owns the **Show Selection Gizmos** toggle and the two whole-list ops:
//! - **Convert to Curve** — flatten every shape into ONE editable closed Bézier curve (a single Ellipse
//!   yields its faithful 4-arc curve; a single Polygon/Freehand keeps its outline; several shapes trace the
//!   composed mask).
//! - **Simplify Curve** — run the Free-Hand fit on the (single) converted curve to reduce its point count.

use super::PainterTool;
use super::selection_shapes::{SelectionEntry, SelectionShape, sel_polygon_vertices};

impl PainterTool {
    /// Enter **Show Selection Gizmos** mode — reveal every editable shape's gizmo at once. No editor install,
    /// no `stroke_method` change (isolated from the stroke system). No-op without a live selection.
    pub(super) fn enter_selection_edit(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        self.paint.selection_edit_mode = true;
        self.invalidate_composite();
    }

    /// Leave gizmo mode — just hide the gizmos; the `selection_shapes` list already IS the truth (nothing to
    /// bake, nothing to restore).
    pub(super) fn exit_selection_edit(&mut self) {
        self.paint.selection_edit_mode = false;
        self.paint.selection_grab = None;
        self.invalidate_composite();
    }

    /// Toggle **Show Selection Gizmos** (the panel checkbox).
    pub fn toggle_selection_edit(&mut self) {
        if self.paint.selection_edit_mode {
            self.exit_selection_edit();
        } else {
            self.enter_selection_edit();
        }
    }

    /// **Convert to Curve** (ADR-0103 Am.2 §3): flatten the whole shape list into ONE editable closed Bézier
    /// curve, stored as a single `Freehand` entry (NOT installed into the stroke curve editor). A single
    /// Ellipse → its faithful 4-arc curve; a single Polygon/Freehand → its outline; several shapes → the
    /// traced composed mask. Leaves the gizmos shown so the new curve's points are editable. One undo entry.
    pub fn selection_convert_to_curve(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        let editable: Vec<SelectionShape> = self
            .paint
            .selection_shapes
            .iter()
            .filter(|e| e.shape.is_editable())
            .map(|e| e.shape.clone())
            .collect();
        let (points, handles) = if editable.len() == 1 {
            match &editable[0] {
                SelectionShape::Ellipse { center, u, rx, ry } => {
                    ellipse_to_closed_curve(*center, *u, *rx, *ry)
                }
                SelectionShape::Polygon {
                    center,
                    u,
                    rx,
                    ry,
                    sides,
                } => (
                    sel_polygon_vertices(*center, *u, *rx, *ry, *sides),
                    Vec::new(),
                ),
                SelectionShape::Freehand {
                    points, handles, ..
                } => (points.clone(), handles.clone()),
                SelectionShape::Raster { .. } => (self.traced_curve_points(), Vec::new()),
            }
        } else {
            (self.traced_curve_points(), Vec::new())
        };
        if points.len() < 3 {
            return;
        }
        let handles = if handles.len() == points.len() {
            handles
        } else {
            points.iter().map(|p| [*p, *p]).collect()
        };
        self.paint.selection_shapes = vec![SelectionEntry {
            shape: SelectionShape::Freehand { points, handles },
            op: 0,
        }];
        self.paint.selection_edit_mode = true; // keep the gizmos shown on the new curve
        self.recompose_selection_mask();
        self.commit_structural_edit(before);
    }

    /// **Simplify Curve** — reduce the point count of the (single) converted Free-Hand curve with the same
    /// Schneider fit the Free Hand stroke uses (some precision is traded for fewer anchors). No-op unless the
    /// selection is exactly one Freehand shape. One undo entry.
    pub fn selection_simplify_curve(&mut self) {
        let one_freehand = self.paint.selection_shapes.len() == 1
            && matches!(
                self.paint.selection_shapes[0].shape,
                SelectionShape::Freehand { .. }
            );
        if !one_freehand {
            return;
        }
        // Flatten the current curve to a dense polyline, then re-fit it to a sparse Bézier.
        let SelectionShape::Freehand {
            points, handles, ..
        } = &self.paint.selection_shapes[0].shape
        else {
            return;
        };
        let spine = self.freehand_spine(points, handles);
        if spine.len() < 3 {
            return;
        }
        let fit = ph2d_painter_brush::fit_curve(&spine, super::curve::FREEHAND_FIT_ERROR);
        if fit.anchors.len() < 3 || fit.anchors.len() > super::curve::MAX_CURVE_POINTS {
            return;
        }
        let before = self.snapshot_model();
        self.paint.selection_shapes[0].shape = SelectionShape::Freehand {
            points: fit.anchors,
            handles: fit.handles,
        };
        self.recompose_selection_mask();
        self.commit_structural_edit(before);
    }

    /// The composed mask traced into a closed anchor polygon (for Convert with 0 / multiple parametric shapes).
    fn traced_curve_points(&self) -> Vec<[f32; 2]> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        super::selection_trace::trace_selection_contour(&self.paint.selection_mask, w, h)
    }
}

/// The four cardinal anchors + `k = 0.5523` Bézier handles that reproduce the ellipse exactly (mirrors
/// `ellipse::ellipse_to_curve`, but from stored params — no editor need be open).
fn ellipse_to_closed_curve(
    c: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
) -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    const K: f32 = 0.552_285;
    let v = [-u[1], u[0]];
    let pt = |su: f32, sv: f32| {
        [
            c[0] + su * rx * u[0] + sv * ry * v[0],
            c[1] + su * rx * u[1] + sv * ry * v[1],
        ]
    };
    let (right, top, left, bottom) = (pt(1.0, 0.0), pt(0.0, 1.0), pt(-1.0, 0.0), pt(0.0, -1.0));
    let hv = [v[0] * K * ry, v[1] * K * ry];
    let hu = [u[0] * K * rx, u[1] * K * rx];
    let sub = |a: [f32; 2], b: [f32; 2]| [a[0] - b[0], a[1] - b[1]];
    let add = |a: [f32; 2], b: [f32; 2]| [a[0] + b[0], a[1] + b[1]];
    let points = vec![right, top, left, bottom];
    let handles = vec![
        [sub(right, hv), add(right, hv)],
        [add(top, hu), sub(top, hu)],
        [add(left, hv), sub(left, hv)],
        [sub(bottom, hu), add(bottom, hu)],
    ];
    (points, handles)
}
