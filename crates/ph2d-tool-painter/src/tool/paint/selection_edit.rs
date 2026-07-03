//! Selection **Edit mode** (ADR-0103 Amendment 2) — the boundary becomes editable via each shape's OWN
//! native gizmo (Ellipse editor / closed Curve), reusing the Stroke-Method shape editors. Entering Edit
//! installs the native editor for the last editable list entry; every pointer edit rewrites that entry and
//! recomposites the mask; the **Offset** slider grows/shrinks it; **Convert to Curve** flattens the whole
//! list into one editable Bézier curve. Falls back to tracing the raw mask when no parametric shape exists
//! (e.g. after Invert / Automatic). Split from `selection` for the LOC cap.

use super::selection_shapes::{SelectionEntry, SelectionShape};
use super::PainterTool;
use crate::undo::{CurveState, EllipseState, ShapeEditState};
use ph2d_editor_core::tool::CanvasPointer;
use std::sync::Arc;

impl PainterTool {
    /// Enter Selection **Edit** mode: install the native gizmo for the last editable shape in the list
    /// (Ellipse → the ellipse gizmo; Rect/Freehand → a closed Curve). With no parametric shape, trace the
    /// mask into a closed Curve (fallback). No-op without a live selection.
    pub(super) fn enter_selection_edit(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        self.paint.selection_edit_saved_method = Some(self.paint.brush.stroke_method);
        let idx = self
            .paint
            .selection_shapes
            .iter()
            .rposition(|e| e.shape.is_editable());
        let installed = if let Some(i) = idx {
            let shape = self.paint.selection_shapes[i].shape.clone();
            self.install_native_selection_editor(&shape);
            self.paint.selection_edit_idx = Some(i);
            self.paint.ellipse.is_some() || self.paint.curve.is_some()
        } else {
            self.paint.selection_edit_idx = None;
            self.install_traced_selection_curve()
        };
        if !installed {
            self.paint.selection_edit_saved_method = None;
            return;
        }
        self.paint.selection_edit_mode = true;
        self.invalidate_composite();
    }

    /// Leave Edit mode: bake the live gizmo (incl. Offset) back into its list entry, drop the overlay, restore
    /// the pre-edit stroke method, and recomposite from the (now static) list.
    pub(super) fn exit_selection_edit(&mut self) {
        if !self.paint.selection_edit_mode {
            return;
        }
        self.flush_shape_txn();
        self.bake_active_selection_editor();
        self.paint.selection_edit_mode = false;
        self.discard_open_shape();
        if let Some(m) = self.paint.selection_edit_saved_method.take() {
            self.paint.brush.stroke_method = m;
        }
        let had_idx = self.paint.selection_edit_idx.take();
        if had_idx.is_some() {
            self.recompose_selection_mask();
        }
        self.invalidate_composite();
    }

    /// Toggle Edit mode (the panel's **Edit Selection** button).
    pub fn toggle_selection_edit(&mut self) {
        if self.paint.selection_edit_mode {
            self.exit_selection_edit();
        } else {
            self.enter_selection_edit();
        }
    }

    /// Route a canvas pointer to the active native editor while in Edit mode, then re-derive the mask. The
    /// editor's own shape-txn snapshots the model (incl. the mask), so boundary edits ride the single queue.
    pub(super) fn selection_edit_pointer(&mut self, ev: CanvasPointer) -> bool {
        let consumed = if self.paint.ellipse.is_some() {
            self.ellipse_pointer(ev)
        } else {
            self.curve_pointer(ev) // freehand / rect-as-curve / traced fallback
        };
        if self.paint.selection_edit_idx.is_some() {
            self.recompose_selection_mask();
        } else {
            self.selection_refill_from_curve(); // fallback: mask = the traced/edited curve
        }
        consumed
    }

    /// Install the native editor matching `shape`: Ellipse → the ellipse gizmo; Rect → a closed sharp Curve
    /// (4 corners); Freehand → a closed Bézier Curve (fitting raw lasso points on first edit, else reusing the
    /// stored handles). `restore_shape_overlay` clears the other slots + routes the method.
    fn install_native_selection_editor(&mut self, shape: &SelectionShape) {
        let seed = self.next_shape_seed();
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        match shape {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                let st = EllipseState {
                    center: *center,
                    u: *u,
                    rx: *rx,
                    ry: *ry,
                    editing: true,
                    seed,
                };
                self.restore_shape_overlay(Some(Box::new(ShapeEditState::Ellipse(st))), None);
            }
            SelectionShape::Rect { a, b } => {
                let corners = super::selection_shapes::rect_corners(*a, *b);
                let handles: Vec<[[f32; 2]; 2]> = corners.iter().map(|p| [*p, *p]).collect();
                let kinds = vec![2u8; corners.len()]; // Vector → sharp straight sides
                let st = closed_curve_state(corners, handles, kinds, seed, false);
                self.restore_shape_overlay(Some(Box::new(ShapeEditState::Curve(st))), None);
            }
            SelectionShape::Freehand {
                points,
                handles,
                kinds,
            } => {
                let (pts, hs, ks) = if handles.len() == points.len() && !points.is_empty() {
                    (points.clone(), handles.clone(), kinds.clone())
                } else {
                    self.fit_closed_freehand(points)
                };
                let st = closed_curve_state(pts, hs, ks, seed, true);
                self.restore_shape_overlay(Some(Box::new(ShapeEditState::Curve(st))), None);
            }
            SelectionShape::Raster { .. } => {}
        }
    }

    /// Fit a raw lasso path into a CLOSED Bézier curve using the SAME Schneider fit + point cap the Free Hand
    /// stroke uses (ADR-0103 Am.2 §2), returning `(anchors, handles, kinds)`; degenerates to the raw polygon.
    fn fit_closed_freehand(&self, points: &[[f32; 2]]) -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>, Vec<u8>) {
        if points.len() >= 3 {
            let fit = ph2d_painter_brush::fit_curve(points, super::curve::FREEHAND_FIT_ERROR);
            if !fit.anchors.is_empty() && fit.anchors.len() <= super::curve::MAX_CURVE_POINTS {
                let kinds = vec![1u8; fit.anchors.len()]; // Aligned — the artist's smooth handles
                return (fit.anchors, fit.handles, kinds);
            }
            let pts = super::curve_geom::cap_curve_points(fit.anchors, super::curve::MAX_CURVE_POINTS);
            let kinds = vec![2u8; pts.len()];
            return (pts, Vec::new(), kinds);
        }
        let kinds = vec![2u8; points.len()];
        (points.to_vec(), Vec::new(), kinds)
    }

    /// Trace the raw mask contour into a closed editable Curve (fallback when no parametric shape exists).
    /// `true` when a traceable boundary was installed.
    fn install_traced_selection_curve(&mut self) -> bool {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let pts = super::selection_trace::trace_selection_contour(&self.paint.selection_mask, w, h);
        if pts.len() < 3 {
            return false;
        }
        let handles: Vec<[[f32; 2]; 2]> = pts.iter().map(|p| [*p, *p]).collect();
        let kinds = vec![2u8; pts.len()]; // Vector → straight segments between the traced anchors
        let seed = self.next_shape_seed();
        let st = closed_curve_state(pts, handles, kinds, seed, true);
        self.restore_shape_overlay(Some(Box::new(ShapeEditState::Curve(st))), None);
        true
    }

    /// Rasterize the LIVE active editor (Offset applied) into a coverage buffer — the source for the active
    /// entry during editing, so the mask tracks the gizmo before it's baked back into the list.
    pub(super) fn rasterize_active_editor(&self) -> Option<Vec<u8>> {
        if let Some(ov) = self.ellipse_overlay() {
            if ov.perimeter.len() >= 3 {
                return Some(self.raster_lasso(&ov.perimeter));
            }
        }
        if let Some(ed) = self.paint.curve.as_ref() {
            let (pts, handles, _) = super::curve_offset::offset_curve_refined(
                &ed.points,
                &ed.handles,
                self.shape_offset_px(),
                ed.closed,
            );
            let mut spine = Vec::new();
            super::curve_geom::flatten_spine(&pts, &handles, ed.closed, &mut spine);
            if spine.len() >= 3 {
                return Some(self.raster_lasso(&spine));
            }
        }
        None
    }

    /// Re-derive the mask from the live Curve editor (fallback traced-curve path — no list entry to rewrite).
    pub(super) fn selection_refill_from_curve(&mut self) {
        let Some(cov) = self.rasterize_active_editor() else {
            return;
        };
        self.set_selection_from_crisp(cov);
    }

    /// Bake the active editor (+ live Offset) back into its list entry (Ellipse stays Ellipse; a Curve becomes
    /// a Freehand of its offset-applied anchors). The fallback traced curve freezes into one `Raster` entry.
    fn bake_active_selection_editor(&mut self) {
        let off = self.shape_offset_px();
        let Some(i) = self.paint.selection_edit_idx else {
            let crisp = Arc::clone(&self.paint.selection_crisp);
            self.paint.selection_shapes = vec![SelectionEntry {
                shape: SelectionShape::Raster { crisp },
                op: 0,
            }];
            return;
        };
        let Some(cap) = self.capture_shape() else {
            return;
        };
        let shape = match *cap {
            ShapeEditState::Ellipse(s) => SelectionShape::Ellipse {
                center: s.center,
                u: s.u,
                rx: (s.rx + off).max(0.5),
                ry: (s.ry + off).max(0.5),
            },
            ShapeEditState::Curve(s) => {
                if off != 0.0 {
                    let (pts, handles, _) =
                        super::curve_offset::offset_curve_refined(&s.points, &s.handles, off, s.closed);
                    let kinds = vec![1u8; pts.len()];
                    SelectionShape::Freehand {
                        points: pts,
                        handles,
                        kinds,
                    }
                } else {
                    SelectionShape::Freehand {
                        points: s.points,
                        handles: s.handles,
                        kinds: s.kinds,
                    }
                }
            }
            _ => return,
        };
        if i < self.paint.selection_shapes.len() {
            self.paint.selection_shapes[i].shape = shape;
        }
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
    }

    /// Set the selection **Offset** (grow/shrink) slider (`0..1`, `0.5` = no change) while editing — the
    /// coalesced Offset drag is one undo entry; the mask re-derives live from the offset gizmo.
    pub fn set_selection_offset(&mut self, norm: f32) {
        if !self.paint.selection_edit_mode {
            return;
        }
        self.begin_offset_txn();
        self.paint.shape_offset_norm = norm.clamp(0.0, 1.0);
        if self.paint.selection_edit_idx.is_some() {
            self.recompose_selection_mask();
        } else {
            self.selection_refill_from_curve();
        }
    }

    /// The selection Offset slider position (`0..1`).
    #[must_use]
    pub fn selection_offset(&self) -> f32 {
        self.paint.shape_offset_norm
    }

    /// **Convert to Curve** (ADR-0103 Am.2 §3): flatten the whole shape list into ONE editable closed Bézier
    /// curve. A single Ellipse yields its faithful 4-arc curve; a single Rect/Freehand keeps its anchors;
    /// several shapes trace the composed mask. Leaves Edit mode ACTIVE on the new curve. One undo entry.
    pub fn selection_convert_to_curve(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        if self.paint.selection_edit_mode {
            self.flush_shape_txn();
            self.bake_active_selection_editor();
        }
        let editable: Vec<SelectionShape> = self
            .paint
            .selection_shapes
            .iter()
            .filter(|e| e.shape.is_editable())
            .map(|e| e.shape.clone())
            .collect();
        let (points, handles) = if editable.len() == 1 {
            match &editable[0] {
                SelectionShape::Ellipse { center, u, rx, ry } => ellipse_to_closed_curve(*center, *u, *rx, *ry),
                SelectionShape::Rect { a, b } => (super::selection_shapes::rect_corners(*a, *b), Vec::new()),
                SelectionShape::Freehand { points, handles, .. } => (points.clone(), handles.clone()),
                SelectionShape::Raster { .. } => (self.traced_curve_points(), Vec::new()),
            }
        } else {
            (self.traced_curve_points(), Vec::new())
        };
        if points.len() < 3 {
            return;
        }
        let kinds = if handles.len() == points.len() {
            vec![1u8; points.len()] // explicit handles → Aligned
        } else {
            vec![2u8; points.len()] // no handles → Vector (straight)
        };
        let handles = if handles.len() == points.len() {
            handles
        } else {
            points.iter().map(|p| [*p, *p]).collect()
        };
        let seed = self.next_shape_seed();
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        let st = closed_curve_state(points.clone(), handles.clone(), kinds.clone(), seed, true);
        self.restore_shape_overlay(Some(Box::new(ShapeEditState::Curve(st))), None);
        self.paint.selection_shapes = vec![SelectionEntry {
            shape: SelectionShape::Freehand {
                points,
                handles,
                kinds,
            },
            op: 0,
        }];
        self.paint.selection_edit_idx = Some(0);
        self.paint.selection_edit_mode = true;
        if self.paint.selection_edit_saved_method.is_none() {
            self.paint.selection_edit_saved_method = Some(self.paint.brush.stroke_method);
        }
        self.recompose_selection_mask();
        self.commit_structural_edit(before);
    }

    /// The composed mask traced into a closed anchor polygon (for Convert with 0 / multiple parametric shapes).
    fn traced_curve_points(&self) -> Vec<[f32; 2]> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        super::selection_trace::trace_selection_contour(&self.paint.selection_mask, w, h)
    }

    /// Allocate a fresh per-shape jitter seed.
    fn next_shape_seed(&mut self) -> u64 {
        let seed = self.paint.seed;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        seed
    }
}

/// Build a `CurveState` for a closed selection boundary (the editor closes last→first itself).
fn closed_curve_state(
    points: Vec<[f32; 2]>,
    handles: Vec<[[f32; 2]; 2]>,
    kinds: Vec<u8>,
    seed: u64,
    freehand: bool,
) -> CurveState {
    let anchor = points.first().copied().unwrap_or([0.0, 0.0]);
    CurveState {
        points,
        handles,
        kinds,
        selected: None,
        added_point: false,
        closed: true,
        editing: true,
        freehand,
        seed,
        anchor,
        stabilized: anchor,
    }
}

/// The four cardinal anchors + `k = 0.5523` Bézier handles that reproduce the ellipse exactly (mirrors
/// `ellipse::ellipse_to_curve`, but from stored params — the ellipse editor need not be open).
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
