//! **Stroke boolean composite** (Enio 2026-07-04): Add/Remove stroke shapes combine like the Selection ops
//! — overlapping regions union (Add) / subtract (Remove); SEPARATE regions stay individual (each traced as
//! its own contour); Overlay shapes never combine. Reuses the Selection rasteriser + `combine_into` +
//! multi-component contour trace, so a group of fillable Add/Remove shapes becomes the STROKED outline of
//! the combined region (the inner arcs where they overlap vanish, as in a real boolean).

use super::selection_shapes::{
    SelectionShape, combine_into, rasterize_ellipse, sel_polygon_vertices,
};
use super::stroke_multi::StrokeOp;
use super::*;
use crate::undo::ShapeEditState;

/// SUPERSAMPLE factor for the boolean mask: rasterise + trace at `SS×` the canvas resolution, then scale the
/// contour back down, so the stroked boolean outline reads as SMOOTH as a direct (analytic) ellipse outline —
/// no visible raster staircase (Enio 2026-07-04 wants a "perfect" Add/Remove appearance). Transcendental-free.
const SS: usize = 3;

/// Build an editing [`CurveState`](crate::undo::CurveState) from a fitted [`CurveModel`] — installs the
/// boolean-composite result as one editable Curve.
fn curve_state_from_model(model: curve_model::CurveModel, seed: u64) -> crate::undo::CurveState {
    let anchor = model.points.first().copied().unwrap_or([0.0, 0.0]);
    crate::undo::CurveState {
        kinds: model.kinds.iter().map(|k| k.wire()).collect(),
        points: model.points,
        handles: model.handles,
        selected: None,
        added_point: false,
        closed: model.closed,
        editing: true,
        freehand: true,
        seed,
        anchor,
        stabilized: anchor,
    }
}

/// Whether a stroke shape is a CLOSED fillable region that can take part in the boolean composite (Ellipse,
/// Polygon, closed Curve, closed Line). Open shapes (open Line / open Curve) always paint their own outline.
pub(super) fn is_boolean_fillable(st: &ShapeEditState) -> bool {
    matches!(st, ShapeEditState::Ellipse(_) | ShapeEditState::Polygon(_))
        || matches!(st, ShapeEditState::Curve(c) if c.closed)
        || matches!(st, ShapeEditState::Line(l) if l.closed)
}

impl PainterTool {
    /// Convert a boolean-fillable stroke shape to a [`SelectionShape`] (to reuse the selection rasteriser),
    /// with the live Offset `off` folded into its geometry so the composite tracks the Offset slider. `None`
    /// for shapes that don't fill (open Curve / Line).
    fn stroke_state_to_fill_shape(&self, st: &ShapeEditState, off: f32) -> Option<SelectionShape> {
        match st {
            ShapeEditState::Ellipse(e) => Some(SelectionShape::Ellipse {
                center: e.center,
                u: e.u,
                rx: (e.rx + off).max(0.5),
                ry: (e.ry + off).max(0.5),
            }),
            ShapeEditState::Polygon(p) => {
                // Use the SAME perimeter the gizmo + fill draw (`polygon_perimeter`, first vertex at the top)
                // as a Freehand polygon — NOT `SelectionShape::Polygon`, whose CORNER-seeded vertices (45°
                // phase) made the boolean outline diverge from the gizmo on rotation (Enio 2026-07-04).
                let mut perim = Vec::new();
                ph2d_painter_brush::polygon_perimeter(
                    p.center,
                    p.u,
                    (p.rx + off).max(0.5),
                    (p.ry + off).max(0.5),
                    p.sides,
                    &mut perim,
                );
                if perim.len() < 3 {
                    return None;
                }
                let handles = perim.iter().map(|q| [*q, *q]).collect(); // sharp corners
                let model = curve_model::CurveModel::from_curve(
                    perim,
                    handles,
                    true,
                    curve_handle::HandleKind::Free,
                );
                Some(SelectionShape::Freehand {
                    model,
                    u: [1.0, 0.0],
                })
            }
            ShapeEditState::Curve(c) if c.closed => {
                let (pts, handles, _) =
                    curve_offset::offset_curve_refined(&c.points, &c.handles, off, true);
                let model = curve_model::CurveModel::from_curve(
                    pts,
                    handles,
                    true,
                    curve_handle::HandleKind::Aligned,
                );
                Some(SelectionShape::Freehand {
                    model,
                    u: [1.0, 0.0],
                })
            }
            ShapeEditState::Line(l) if l.closed => {
                // Expand the corner Fillet/Chamfer + offset the parallel polyline, then fill it as a sharp
                // Freehand polygon (no Bézier handles → the rasteriser fills the raw polygon).
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                let expanded =
                    line_corner::expand(&l.points, true, &mods, self.paint.shape_grab_tol_px);
                let path = line_offset::offset_polyline(&expanded, true, off);
                if path.len() < 3 {
                    return None;
                }
                let handles = path.iter().map(|p| [*p, *p]).collect(); // sharp corners
                let model = curve_model::CurveModel::from_curve(
                    path,
                    handles,
                    true,
                    curve_handle::HandleKind::Free,
                );
                Some(SelectionShape::Freehand {
                    model,
                    u: [1.0, 0.0],
                })
            }
            _ => None,
        }
    }

    /// Boolean-composite the Add/Remove fillable `shapes` into contour polylines (image px): rasterize each
    /// at `SS×` resolution, union (Add) / subtract (Remove) in DRAW ORDER, trace EVERY connected component
    /// (separate regions → separate contours; overlapping → merged), then scale the contours back to canvas
    /// px. The supersample + the tracer's smoothing make the stroked outline as regular as a direct ellipse.
    pub(super) fn stroke_boolean_contours(
        &self,
        shapes: &[(ShapeEditState, StrokeOp)],
        off: f32,
    ) -> Vec<Vec<[f32; 2]>> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 {
            return Vec::new();
        }
        let (sw, sh) = (w * SS, h * SS);
        let mut crisp = vec![0u8; sw * sh];
        let mut any = false;
        for (st, op) in shapes {
            let Some(sel) = self.stroke_state_to_fill_shape(st, off) else {
                continue;
            };
            self.rasterize_fill_ss(&sel, sw, sh, &mut crisp, op.to_wire());
            any = true;
        }
        if !any {
            return Vec::new();
        }
        let s = SS as f32;
        super::selection_trace::trace_all_contours(&crisp, sw, sh)
            .into_iter()
            .map(|c| c.iter().map(|p| [p[0] / s, p[1] / s]).collect())
            .collect()
    }

    /// `true` when the multi-shape set has any Add/Remove CLOSED shape (active or parked) — a live boolean
    /// composite. Convert-to-Curve then folds the WHOLE boolean RESULT into one editable curve.
    pub(crate) fn has_boolean_shapes(&self) -> bool {
        let active = self
            .capture_shape()
            .is_some_and(|s| self.paint.active_op != StrokeOp::Overlay && is_boolean_fillable(&s));
        active
            || self
                .paint
                .parked_shapes
                .iter()
                .any(|s| s.op != StrokeOp::Overlay && is_boolean_fillable(&s.state))
    }

    /// Convert the Add/Remove BOOLEAN RESULT into editable curve(s): fit each combined-region contour to a
    /// sparse Bézier, DROP every boolean shape, and install the result as the live Curve (extra regions →
    /// parked Overlay curves). Overlay / open shapes are untouched. One undo step (Enio 2026-07-04).
    pub(crate) fn convert_boolean_result_to_curve(&mut self) -> bool {
        let off = self.shape_offset_px();
        let mut bool_shapes: Vec<(ShapeEditState, StrokeOp)> = self
            .paint
            .parked_shapes
            .iter()
            .filter(|s| s.op != StrokeOp::Overlay && is_boolean_fillable(&s.state))
            .map(|s| (s.state.clone(), s.op))
            .collect();
        if let Some(active) = self.capture_shape()
            && self.paint.active_op != StrokeOp::Overlay
            && is_boolean_fillable(&active)
        {
            bool_shapes.push((*active, self.paint.active_op));
        }
        if bool_shapes.is_empty() {
            return false;
        }
        let contours = self.stroke_boolean_contours(&bool_shapes, off);
        let mut states: Vec<ShapeEditState> = Vec::new();
        for c in &contours {
            if let Some(m) = super::selection_edit::to_closed_curve(c, &[]) {
                let seed = self.paint.seed;
                self.paint.seed = self.paint.seed.wrapping_add(1);
                states.push(ShapeEditState::Curve(curve_state_from_model(m, seed)));
            }
        }
        if states.is_empty() {
            return false;
        }
        let before = self.capture_shape_model();
        // Drop every boolean shape; keep Overlay / open ones parked. Clear the (boolean) active editor.
        self.paint
            .parked_shapes
            .retain(|s| s.op == StrokeOp::Overlay || !is_boolean_fillable(&s.state));
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        self.paint.active_op = StrokeOp::Overlay; // converted curve is plain overlay
        let first = states.remove(0);
        for s in states {
            self.paint
                .parked_shapes
                .push(super::stroke_multi::StrokeShape {
                    state: s,
                    op: StrokeOp::Overlay,
                });
        }
        self.install_shape_editor(first);
        self.curve_refill();
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after);
        true
    }

    /// Rasterise one fill shape scaled by `SS` into the supersampled `crisp` via `op` (transcendental-free:
    /// exact ellipse inside-test / baked-step polygon / flattened freehand → scanline fill).
    fn rasterize_fill_ss(
        &self,
        sel: &SelectionShape,
        sw: usize,
        sh: usize,
        crisp: &mut [u8],
        op: u8,
    ) {
        let s = SS as f32;
        let mut region = vec![0u8; sw * sh];
        match sel {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                rasterize_ellipse(
                    [center[0] * s, center[1] * s],
                    *u,
                    rx * s,
                    ry * s,
                    sw,
                    sh,
                    &mut region,
                );
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => {
                let verts: Vec<[f32; 2]> = sel_polygon_vertices(*center, *u, *rx, *ry, *sides)
                    .iter()
                    .map(|p| [p[0] * s, p[1] * s])
                    .collect();
                scanline_fill(&verts, sw, sh, &mut region);
            }
            SelectionShape::Freehand { model, .. } => {
                let spine = self.freehand_spine(&model.points, &model.handles);
                let verts: Vec<[f32; 2]> = spine.iter().map(|p| [p[0] * s, p[1] * s]).collect();
                scanline_fill(&verts, sw, sh, &mut region);
            }
            SelectionShape::Raster { .. } => {}
        }
        combine_into(crisp, &region, op);
    }
}

/// Even-odd scanline polygon fill of the closed polyline `pts` into `cov` (0/255) at `w`×`h`. Mirrors the
/// selection `raster_lasso` but writes into a caller buffer at an arbitrary (supersampled) resolution.
fn scanline_fill(pts: &[[f32; 2]], w: usize, h: usize, cov: &mut [u8]) {
    if pts.len() < 3 {
        return;
    }
    for yy in 0..h {
        let yc = yy as f32 + 0.5;
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..pts.len() {
            let p = pts[i];
            let q = pts[(i + 1) % pts.len()];
            if (p[1] <= yc && yc < q[1]) || (q[1] <= yc && yc < p[1]) {
                xs.push(p[0] + (yc - p[1]) / (q[1] - p[1]) * (q[0] - p[0]));
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i + 1 < xs.len() {
            let xa = xs[i].max(0.0).round() as usize;
            let xb = (xs[i + 1].min(w as f32).round() as usize).min(w);
            for xx in xa..xb {
                cov[yy * w + xx] = 255;
            }
            i += 2;
        }
    }
}
