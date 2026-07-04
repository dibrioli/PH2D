//! **Stroke boolean composite** (Enio 2026-07-04): Add/Remove stroke shapes combine like the Selection ops
//! — overlapping regions union (Add) / subtract (Remove); SEPARATE regions stay individual (each traced as
//! its own contour); Overlay shapes never combine. Reuses the Selection rasteriser + `combine_into` +
//! multi-component contour trace, so a group of fillable Add/Remove shapes becomes the STROKED outline of
//! the combined region (the inner arcs where they overlap vanish, as in a real boolean).

use super::selection_shapes::{SelectionShape, combine_into};
use super::stroke_multi::StrokeOp;
use super::*;
use crate::undo::ShapeEditState;

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
            ShapeEditState::Polygon(p) => Some(SelectionShape::Polygon {
                center: p.center,
                u: p.u,
                rx: (p.rx + off).max(0.5),
                ry: (p.ry + off).max(0.5),
                sides: p.sides,
            }),
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

    /// Boolean-composite the Add/Remove fillable `shapes` into contour polylines (image px): rasterize each,
    /// union (Add) / subtract (Remove) in DRAW ORDER, then trace EVERY connected component — so separate
    /// regions come back as separate contours (individual) while overlapping ones merge into one.
    pub(super) fn stroke_boolean_contours(
        &self,
        shapes: &[(ShapeEditState, StrokeOp)],
        off: f32,
    ) -> Vec<Vec<[f32; 2]>> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 {
            return Vec::new();
        }
        let mut crisp = vec![0u8; w * h];
        let mut any = false;
        for (st, op) in shapes {
            let Some(sel) = self.stroke_state_to_fill_shape(st, off) else {
                continue;
            };
            let region = self.rasterize_selection_shape(&sel);
            if region.len() == w * h {
                combine_into(&mut crisp, &region, op.to_wire());
                any = true;
            }
        }
        if !any {
            return Vec::new();
        }
        super::selection_trace::trace_all_contours(&crisp, w, h)
    }
}
