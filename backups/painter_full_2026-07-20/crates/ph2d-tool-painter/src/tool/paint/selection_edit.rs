//! Selection **gizmo-mode toggle + list operations** (ADR-0103 Am.2 v2). The per-shape editing itself lives
//! in the isolated [`super::selection_gizmo`] module (which never touches the stroke editors). This file
//! owns the **Show Selection Gizmos** toggle and the two whole-list ops:
//! - **Convert to Curve** — flatten every shape into ONE editable closed Bézier curve (a single Ellipse
//!   yields its faithful 4-arc curve; a single Polygon/Freehand keeps its outline; several shapes trace the
//!   composed mask).
//! - **Simplify Curve** — run the Free-Hand fit on the (single) converted curve to reduce its point count.

use super::PainterTool;
use super::curve::{
    CONVERT_ANCHOR_SPACING_PX, CONVERT_FIT_ERROR, FREEHAND_FIT_ERROR, MAX_CONVERT_POINTS,
    MAX_CURVE_POINTS, MERGE_SIMPLIFY_TOL_PX, SIMPLIFY_KEEP_FRACTION, SIMPLIFY_MIN_POINTS,
};
use super::curve_handle::HandleKind;
use super::curve_model::CurveModel;
use super::selection_shapes::{SelectionEntry, SelectionShape, sel_polygon_vertices};

impl PainterTool {
    /// Enter **Show Selection Gizmos** mode — reveal every editable shape's gizmo at once. No editor install,
    /// no `stroke_method` change (isolated from the stroke system). No-op without a live selection.
    pub(super) fn enter_selection_edit(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        self.paint.selection_edit_mode = true;
        // If NOTHING in the selection is editable — only baked `Raster` coverage (after Offset Apply /
        // Apply & Keep, or an Automatic flood) — turn it into editable curve(s) so Edit Gizmos always opens a
        // boundary to edit (Enio 2026-07-03/04). An active OFFSET materialises EVERY ring boundary into its
        // own editable curve (band-parity fill preserves the intercalated selection, and the curves persist
        // until Clear); a plain baked/Automatic mask becomes a single traced curve.
        let has_editable = self
            .paint
            .selection_shapes
            .iter()
            .any(|e| e.shape.is_editable());
        if !has_editable {
            if self.paint.selection_offset_active {
                self.materialise_offset_rings_to_curves();
            } else {
                self.reset_selection_offset();
                self.selection_convert_to_curve();
            }
        }
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
        // Convert EVERY shape to its OWN editable Bézier curve, preserving the multi-shape list + each
        // entry's boolean op — so multiple SEPARATE selections all survive (Enio 2026-07-04: the former
        // single-composed-contour path lost every region but the first). An Ellipse keeps its exact 4-arc,
        // a Polygon its sharp vertices, a Freehand fits its outline; a Raster traces the composed mask (its
        // flood has no parametric outline). Non-editable entries that don't yield a curve are dropped.
        let entries: Vec<(usize, u8)> = self
            .paint
            .selection_shapes
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.op))
            .collect();
        let mut converted: Vec<SelectionEntry> = Vec::new();
        for (i, op) in entries {
            let shape = self.paint.selection_shapes[i].shape.clone();
            if let Some(model) = self.shape_to_closed_curve(&shape) {
                converted.push(SelectionEntry {
                    shape: SelectionShape::Freehand {
                        model,
                        u: [1.0, 0.0],
                    },
                    op,
                });
            }
        }
        if converted.is_empty() {
            return; // nothing convertible (all < 3 points)
        }
        // The first surviving curve becomes the base op (New) so the recomposed mask isn't a no-op when the
        // original base was consumed; the rest keep Add/Remove.
        converted[0].op = 0;
        let before = self.snapshot_model();
        self.paint.selection_shapes = converted;
        self.paint.selection_ring_stack = false; // boolean curves, not a ring stack
        self.paint.selection_edit_mode = true; // keep the gizmos shown on the new curves
        self.recompose_selection_mask();
        self.commit_structural_edit(before);
    }

    /// Convert one selection shape to a closed editable [`CurveModel`], or `None` when it has too few points
    /// to fit. Shared by [`Self::selection_convert_to_curve`] for the per-shape conversion.
    fn shape_to_closed_curve(&self, shape: &SelectionShape) -> Option<CurveModel> {
        // Convert-to-Curve is the DENSE direction (Enio 2026-07-05): parametric shapes densify their
        // EXACT curve to many manipulation anchors; outlines fit tight. Simplify Curve goes back down.
        let densify = |p: Vec<[f32; 2]>, h: Vec<[[f32; 2]; 2]>| {
            super::curve_geom::densify_closed_curve(
                &p,
                &h,
                CONVERT_ANCHOR_SPACING_PX,
                MAX_CONVERT_POINTS,
            )
        };
        match shape {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                let (p, h) = ellipse_to_closed_curve(*center, *u, *rx, *ry);
                let (p, h) = densify(p, h);
                Some(CurveModel::from_curve(p, h, true, HandleKind::Aligned))
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => {
                let verts = sel_polygon_vertices(*center, *u, *rx, *ry, *sides);
                let handles = verts.iter().map(|p| [*p, *p]).collect(); // sharp corners
                let (p, h) = densify(verts, handles);
                Some(CurveModel::from_curve(p, h, true, HandleKind::Free))
            }
            SelectionShape::Freehand { model, .. } => {
                to_closed_curve_precise(&model.points, &model.handles)
            }
            SelectionShape::Raster { .. } => {
                to_closed_curve_precise(&self.traced_curve_points(), &[])
            }
        }
    }

    /// **Merge Curves** — collapse the WHOLE selection (every shape / curve, folded by their boolean ops)
    /// into a single high-precision multipoint closed curve by tracing the composed mask (Enio 2026-07-05).
    /// Overlapping shapes become ONE curve; genuinely disjoint regions each keep their own dense curve (a
    /// single closed curve can't span two blobs). Precise + many points — **Simplify Curve** reduces them
    /// later. Leaves the gizmos shown. One undo entry. No-op without an active selection.
    pub fn selection_merge_curves(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        // Trace EVERY connected component of the composed boolean mask (the union/subtract result the artist
        // sees), then fit each to a DENSE precise closed curve. `trace_all_contours` closes each loop with a
        // duplicated seam point — drop it so the fit sees no zero-length segment.
        let contours = super::selection_trace::trace_all_contours(&self.paint.selection_mask, w, h);
        let mut merged: Vec<SelectionEntry> = Vec::new();
        for mut c in contours {
            if c.len() >= 2 && c[0] == c[c.len() - 1] {
                c.pop();
            }
            if let Some(model) = to_closed_curve_smooth(&c) {
                merged.push(SelectionEntry {
                    // First surviving contour is the base (New/replace); the rest union onto it (Add).
                    op: u8::from(!merged.is_empty()),
                    shape: SelectionShape::Freehand {
                        model,
                        u: [1.0, 0.0],
                    },
                });
            }
        }
        if merged.is_empty() {
            return; // nothing traced (empty / sub-3-point mask)
        }
        let before = self.snapshot_model();
        self.paint.selection_shapes = merged;
        self.paint.selection_ring_stack = false; // boolean curves, not a ring stack
        self.paint.selection_edit_mode = true; // keep the gizmos shown on the merged curve
        self.recompose_selection_mask();
        self.commit_structural_edit(before);
    }

    /// **Simplify Curve** — re-fit EVERY converted Free-Hand curve in the selection (before OR after a Merge)
    /// to a SPARSE Bézier via the same high-quality Schneider fit the Free Hand stroke uses: precise, but far
    /// fewer anchors (Enio 2026-07-05). Non-Freehand entries pass through untouched. No-op if nothing was
    /// simplifiable. One undo entry.
    pub fn selection_simplify_curve(&mut self) {
        if self.paint.selection_shapes.is_empty() {
            return;
        }
        let before = self.snapshot_model();
        let mut changed = false;
        for e in &mut self.paint.selection_shapes {
            if let SelectionShape::Freehand { model, u } = &e.shape {
                let u = *u;
                // PROGRESSIVE refit (Enio 2026-07-05): corner-split + piecewise Schneider least-squares fit,
                // escalating the tolerance until ~20% of the anchors shed — Aligned smooth joins, Free corners.
                if let Some(r) = super::curve_refit::refit_progressive(
                    &model.points,
                    &model.handles,
                    SIMPLIFY_KEEP_FRACTION,
                    SIMPLIFY_MIN_POINTS,
                ) {
                    e.shape = SelectionShape::Freehand {
                        model: CurveModel {
                            points: r.points,
                            handles: r.handles,
                            kinds: r.kinds,
                            selected: None,
                            closed: true,
                        },
                        u,
                    };
                    changed = true;
                }
            }
        }
        if !changed {
            return; // no Freehand curve reduced (all too short)
        }
        self.paint.selection_ring_stack = false; // boolean curves, not a ring stack
        self.recompose_selection_mask();
        // COALESCED: a run of progressive Simplify presses = ONE undo step back to the pre-run curves.
        self.commit_structural_edit_coalesced(crate::undo::CoalesceKind::Simplify, before);
    }

    /// The composed mask traced into a closed anchor polygon (for Convert with 0 / multiple parametric shapes).
    fn traced_curve_points(&self) -> Vec<[f32; 2]> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        super::selection_trace::trace_selection_contour(&self.paint.selection_mask, w, h)
    }
}

/// Fit a closed outline (`points` + optional parallel `handles`) to a SPARSE editable [`CurveModel`] via the
/// Schneider fit the stroke Free Hand / Simplify uses, seeded to Aligned. A polygon-like outline (no handles)
/// is closed via degenerate `[p, p]` handles so the flatten runs the closing seam before fitting. Falls back
/// to the exact (dense) outline as an editable curve if the fit is degenerate; `None` only when < 3 points.
pub(super) fn to_closed_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
) -> Option<CurveModel> {
    to_closed_curve_with(points, handles, FREEHAND_FIT_ERROR, MAX_CURVE_POINTS, false)
}

/// The CONVERT variant of [`to_closed_curve`]: tight fit + densified to the convert anchor spacing —
/// extreme precision, many manipulation points (Enio 2026-07-05). [`to_closed_curve`] (the sparse
/// Free-Hand fit) stays the **Simplify** direction.
pub(super) fn to_closed_curve_precise(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
) -> Option<CurveModel> {
    to_closed_curve_with(points, handles, CONVERT_FIT_ERROR, MAX_CONVERT_POINTS, true)
}

/// A traced closed CONTOUR → a CLEAN editable curve via the REFIT funnel (`curve_refit::refit_closed_spine`
/// — corner-split + piecewise Schneider least-squares fit within [`MERGE_SIMPLIFY_TOL_PX`] px): few anchors
/// that hug the outline, Aligned smooth joins, Free corners, no dense-fit spikes at concave waists. The Merge
/// paths use this instead of [`to_closed_curve_precise`] so a merged outline reads perfect without a manual
/// Simplify (Enio 2026-07-05). `None` when the contour is degenerate.
pub(super) fn to_closed_curve_smooth(points: &[[f32; 2]]) -> Option<CurveModel> {
    let r = super::curve_refit::refit_closed_spine(points, MERGE_SIMPLIFY_TOL_PX)?;
    Some(CurveModel {
        points: r.points,
        handles: r.handles,
        kinds: r.kinds,
        selected: None,
        closed: true,
    })
}

fn to_closed_curve_with(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    max_error: f32,
    max_points: usize,
    densify: bool,
) -> Option<CurveModel> {
    if points.len() < 3 {
        return None;
    }
    let deg: Vec<[[f32; 2]; 2]> = points.iter().map(|p| [*p, *p]).collect();
    let src: &[[[f32; 2]; 2]] = if handles.len() == points.len() && !handles.is_empty() {
        handles
    } else {
        &deg
    };
    let mut model =
        CurveModel::from_fit(points, src, true, max_error, max_points).unwrap_or_else(|| {
            CurveModel::from_curve(points.to_vec(), src.to_vec(), true, HandleKind::Aligned)
        });
    if densify {
        let (p, h) = super::curve_geom::densify_closed_curve(
            &model.points,
            &model.handles,
            CONVERT_ANCHOR_SPACING_PX,
            max_points,
        );
        model = CurveModel::from_curve(p, h, true, HandleKind::Aligned);
    }
    Some(model)
}

/// The four cardinal anchors + `k = 0.5523` Bézier handles that reproduce the ellipse exactly (mirrors
/// `ellipse::ellipse_to_curve`, but from stored params — no editor need be open). Shared with the stroke
/// per-shape Convert (a parked Ellipse state → its editable curve; Enio 2026-07-05).
pub(super) fn ellipse_to_closed_curve(
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
