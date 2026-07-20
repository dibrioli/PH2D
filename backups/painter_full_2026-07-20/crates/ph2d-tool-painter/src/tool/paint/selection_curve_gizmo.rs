//! **Converted-selection-curve point editor** (ADR-0103 Am.2 §3, Enio 2026-07-03). After **Convert to
//! Curve** / **Simplify Curve** the selection is a single `Freehand` whose [`CurveModel`] carries explicit
//! Bézier handles, and it must edit *identically* to the stroke Shape system's Curve — per-anchor points
//! with in/out Bézier handles, the same 5 handle kinds, the same right-click menu, insert/delete/select —
//! NOT the whole-shape transform box a RAW lasso Freehand (empty handles) shows.
//!
//! This module is a **thin adapter** over the SHARED [`super::curve_model::CurveModel`]: the editing ops
//! (hit / insert / drag / delete / set-kind / select) live there and are the exact code paths the stroke
//! editor runs, so the two can never drift. Isolation (ADR-0103 Am.2 v2) is preserved — the selection curve
//! keeps its own `CurveModel` inside the `selection_shapes` list and never reads/writes the stroke slot
//! `self.paint.curve`; only the pure model + free fns are shared.

use super::PainterTool;
use super::curve::MAX_CONVERT_POINTS;
use super::curve_model::{CurveGrab, CurveModel};
use super::curve_tangent::{self, TangentHandles};
use super::selection_shapes::SelectionShape;

/// The drawable point-editor view of a converted selection curve (image px) — mirrors the stroke curve
/// overlay: the anchors, the selected index, and the SELECTED anchor's Bézier tangent handles (only the
/// selected one shows handles, like the stroke). The spine itself is drawn from the gizmo `outline`.
pub struct SelectionCurveEdit {
    /// Control anchors (image px) — the selected one draws highlighted, the rest as squares.
    pub anchors: Vec<[f32; 2]>,
    /// The selected anchor index (drawn highlighted), if any.
    pub selected: Option<usize>,
    /// The selected anchor's tangent handles (stems + grabbable dots), or `None` when none/collapsed.
    pub tangents: Option<TangentHandles>,
    /// The selected anchor's **handle-kind** wire `u8` (`0=Free 1=Aligned 2=Vector 3=Auto 4=Symmetric`), or
    /// `None` — lets the shell mark the active right-click menu item (as the stroke curve overlay does).
    pub selected_kind: Option<u8>,
}

/// `true` when `shape` is a CONVERTED curve (a `Freehand` whose model carries Bézier handles) — the Convert /
/// Simplify output. A raw lasso `Freehand` has an empty-handle model ⇒ `false` (it shows the transform box).
/// This is the switch between the two Freehand editing modes.
#[must_use]
pub(super) fn is_converted_curve(shape: &SelectionShape) -> bool {
    matches!(shape, SelectionShape::Freehand { model, .. } if model.is_curve())
}

/// Build the point-editor overlay of a converted curve (`None` for any other shape). `grabbed` is the tangent
/// handle being dragged this gesture `(anchor, is_out)`, for the accent colour; `tol` is the grab tolerance.
#[must_use]
pub(super) fn curve_edit_view(
    shape: &SelectionShape,
    grabbed: Option<(usize, bool)>,
    tol: f32,
) -> Option<SelectionCurveEdit> {
    let SelectionShape::Freehand { model, .. } = shape else {
        return None;
    };
    if !model.is_curve() {
        return None;
    }
    let tangents = model.selected.and_then(|s| {
        curve_tangent::build_tangents(&model.points, &model.handles, s, grabbed, tol, model.closed)
    });
    Some(SelectionCurveEdit {
        anchors: model.points.clone(),
        selected: model.selected,
        tangents,
        selected_kind: model.selected_kind_wire(),
    })
}

/// The point-editor's fall-through after a click misses every control point / tangent AND the transform box:
/// insert a new anchor on the curve (and grab it), else select the nearest at the point cap. The caller
/// (`selection_gizmo_down`) runs `model.hit_point` and the box hit-test FIRST, so this only fires on a real
/// miss — mirroring the stroke `curve_down` order (point → gizmo → insert → nearest).
#[must_use]
pub(super) fn selection_curve_insert_or_nearest(
    model: &mut CurveModel,
    pos: [f32; 2],
) -> Option<CurveGrab> {
    if model.points.len() < MAX_CONVERT_POINTS {
        let i = model.insert(pos); // de Casteljau split — the SHAPE (and mask) is unchanged
        model.selected = Some(i);
        return Some(CurveGrab::Anchor(i));
    }
    model.select_nearest(pos);
    None
}

impl PainterTool {
    /// The topmost converted-curve shape that currently has a SELECTED anchor — the target for the
    /// right-click handle-kind menu + the Delete key.
    fn selection_curve_active_index(&self) -> Option<usize> {
        self.paint.selection_shapes.iter().rposition(|e| {
            matches!(&e.shape, SelectionShape::Freehand { model, .. }
                if model.is_curve() && model.selected.is_some())
        })
    }

    /// Select the control point under `pos` (within `tol`) on the topmost converted curve, clearing any
    /// selection on the others so exactly one anchor is active — the shell's secondary-click before opening
    /// the handle-kind menu. `true` iff a point was hit. Mirrors the stroke `curve_select_point_at`.
    pub fn selection_curve_select_point_at(&mut self, pos: [f32; 2], tol: f32) -> bool {
        let mut hit = false;
        for e in self.paint.selection_shapes.iter_mut().rev() {
            if let SelectionShape::Freehand { model, .. } = &mut e.shape
                && model.is_curve()
            {
                if !hit && model.select_point_at(pos, tol) {
                    hit = true; // topmost-first (rev): the first hit wins
                } else {
                    model.selected = None;
                }
            }
        }
        hit
    }

    /// Set the selected converted-curve anchor's **handle kind** from the menu's wire `u8` (`0=Free 1=Aligned
    /// 2=Vector 3=Auto 4=Symmetric`), recomposite, one undo entry. Mirrors the stroke `set_curve_handle_kind`.
    pub fn set_selection_curve_handle_kind(&mut self, wire: u8) -> bool {
        let Some(i) = self.selection_curve_active_index() else {
            return false;
        };
        let before = self.snapshot_model();
        let changed = match &mut self.paint.selection_shapes[i].shape {
            SelectionShape::Freehand { model, .. } => model.set_kind(wire),
            _ => false,
        };
        if changed {
            self.recompose_selection_mask();
            self.commit_structural_edit(before);
        }
        changed
    }

    /// Delete the selected anchor of the active converted curve (Delete key). Keeps `≥ 2` points, recomposite,
    /// one undo entry. `true` when one was removed. Mirrors the stroke `curve_delete_selected`.
    pub fn selection_curve_delete_selected_point(&mut self) -> bool {
        let Some(i) = self.selection_curve_active_index() else {
            return false;
        };
        let before = self.snapshot_model();
        let removed = match &mut self.paint.selection_shapes[i].shape {
            SelectionShape::Freehand { model, .. } => model.delete_selected(),
            _ => false,
        };
        if removed {
            self.recompose_selection_mask();
            self.commit_structural_edit(before);
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::super::curve_handle::HandleKind;
    use super::*;

    fn converted() -> SelectionShape {
        SelectionShape::Freehand {
            model: CurveModel::from_curve(
                vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]],
                vec![
                    [[-2.0, 0.0], [2.0, 0.0]],
                    [[8.0, 0.0], [12.0, 0.0]],
                    [[10.0, 8.0], [10.0, 12.0]],
                ],
                true,
                HandleKind::Aligned,
            ),
            u: [1.0, 0.0],
        }
    }

    #[test]
    fn only_a_curve_with_handles_is_point_editable() {
        assert!(is_converted_curve(&converted()), "handles present ⇒ curve");
        let raw = SelectionShape::Freehand {
            model: CurveModel::raw_lasso(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], true),
            u: [1.0, 0.0],
        };
        assert!(
            !is_converted_curve(&raw),
            "empty handles ⇒ box, not a curve"
        );
    }

    #[test]
    fn press_grabs_the_out_handle_then_a_drag_moves_only_it() {
        let SelectionShape::Freehand { mut model, .. } = converted() else {
            unreachable!()
        };
        model.selected = Some(0); // tangent_hit only tests the SELECTED anchor's handles
        // The out-handle of anchor 0 is at (2,0); a hit there grabs it (not the anchor).
        let g = model.hit_point([2.0, 0.0], 1.0).expect("grabbed a handle");
        assert_eq!(g, CurveGrab::Tangent(0, true));
        model.drag(g, [5.0, 3.0]);
        assert_eq!(model.points[0], [0.0, 0.0], "the anchor did not move");
        assert_eq!(
            model.handles[0][1],
            [5.0, 3.0],
            "the out handle moved to the pointer"
        );
    }

    #[test]
    fn insert_or_nearest_adds_an_anchor_on_a_miss() {
        let SelectionShape::Freehand { mut model, .. } = converted() else {
            unreachable!()
        };
        let n = model.points.len();
        // A click away from any point/handle inserts a new anchor on the curve and grabs it.
        let g =
            selection_curve_insert_or_nearest(&mut model, [5.0, 0.0]).expect("inserted + grabbed");
        assert!(matches!(g, CurveGrab::Anchor(_)));
        assert_eq!(model.points.len(), n + 1, "an anchor was inserted");
    }
}
