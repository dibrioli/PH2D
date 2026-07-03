//! **Isolated selection gizmos** (ADR-0103 Am.2 v2, Enio 2026-07-03). Selection editing has its OWN gizmo
//! system that NEVER touches the stroke shape editors (`self.paint.{ellipse,curve,polygon,line}`) or
//! `stroke_method` — so the Brush's Shape system can never be contaminated by selection editing (the bug).
//!
//! When "Show Selection Gizmos" is on, EVERY editable shape in `selection_shapes` shows its own gizmo at
//! once and each is independently manipulable: the **Ellipse** gizmo (resize/rotate), the **Polygon** gizmo
//! (resize/rotate/sides — the rect starts as a 4-gon), and the **Freehand** point curve (drag anchors).
//! Each edit mutates the shape's params IN PLACE and recomposites the mask. Pure, self-contained geometry.

use super::PainterTool;
use super::selection_shapes::{SelectionShape, sel_polygon_vertices};
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

/// Smallest half-axis a gizmo drag allows (image px).
const MIN_AXIS_PX: f32 = 1.0;
/// Rotate handle offset beyond the edge, in grab-radii.
const ROTATE_GAP: f32 = 3.0;
/// Sides handle base offset + per-side step beyond the right edge, in grab-radii.
const SIDES_GAP: f32 = 3.0;
const SIDES_STEP: f32 = 1.5;
const MIN_SIDES: u32 = 3;
const MAX_SIDES: u32 = 12;

// Handle ids for the ellipse/polygon gizmos (freehand uses the raw point index).
const H_RIGHT: u8 = 0;
const H_TOP: u8 = 1;
const H_LEFT: u8 = 2;
const H_BOTTOM: u8 = 3;
const H_ROTATE: u8 = 4;
const H_SIDES: u8 = 5; // polygon only
const H_CENTER_ELLIPSE: u8 = 5;
const H_CENTER_POLYGON: u8 = 6;

/// A drawable selection gizmo for one shape (image-space px). The shell strokes `outline` as the boundary,
/// draws `square_handles` as themed rounded squares (resize / anchor points), `circle_handles` as circles
/// (rotate cue) and `diamond_handles` as diamonds (the polygon **sides** handle — distinct from rotate).
pub struct SelectionGizmoView {
    pub outline: Vec<[f32; 2]>,
    pub closed: bool,
    pub square_handles: Vec<[f32; 2]>,
    pub circle_handles: Vec<[f32; 2]>,
    pub diamond_handles: Vec<[f32; 2]>,
}

impl PainterTool {
    /// `true` while the selection gizmos are shown (the panel's **Show Selection Gizmos** checkbox).
    #[must_use]
    pub fn selection_gizmos_visible(&self) -> bool {
        self.paint.selection_edit_mode
    }

    /// Build one gizmo view per editable selection shape (for the shell overlay). Empty when the gizmos are
    /// hidden or nothing is selected.
    #[must_use]
    pub fn selection_gizmos(&self) -> Vec<SelectionGizmoView> {
        if !self.paint.selection_edit_mode {
            return Vec::new();
        }
        let tol = self.paint.shape_grab_tol_px;
        let mut out = Vec::new();
        for e in &self.paint.selection_shapes {
            match &e.shape {
                SelectionShape::Ellipse { center, u, rx, ry } => {
                    let mut perim = Vec::new();
                    ph2d_painter_brush::ellipse_perimeter(*center, *u, *rx, *ry, &mut perim);
                    let h = ellipse_handles(*center, *u, *rx, *ry, tol * ROTATE_GAP);
                    out.push(SelectionGizmoView {
                        outline: perim,
                        closed: true,
                        square_handles: vec![h[0], h[1], h[2], h[3], h[5]],
                        circle_handles: vec![h[4]],
                        diamond_handles: Vec::new(),
                    });
                }
                SelectionShape::Polygon {
                    center,
                    u,
                    rx,
                    ry,
                    sides,
                } => {
                    let perim = sel_polygon_vertices(*center, *u, *rx, *ry, *sides);
                    let h = polygon_handles(*center, *u, *rx, *ry, *sides, tol);
                    out.push(SelectionGizmoView {
                        outline: perim,
                        closed: true,
                        square_handles: vec![h[0], h[1], h[2], h[3], h[6]],
                        circle_handles: vec![h[4]],  // rotate
                        diamond_handles: vec![h[5]], // sides (distinct from rotate)
                    });
                }
                SelectionShape::Freehand {
                    points, handles, ..
                } => {
                    let spine = self.freehand_spine(points, handles);
                    out.push(SelectionGizmoView {
                        outline: spine,
                        closed: true,
                        square_handles: points.clone(),
                        circle_handles: Vec::new(),
                        diamond_handles: Vec::new(),
                    });
                }
                SelectionShape::Raster { .. } => {}
            }
        }
        out
    }

    /// Route a canvas pointer to the isolated selection gizmos (called from `on_canvas_pointer` while the
    /// gizmos are shown). Hit-tests EVERY shape's handles, drags the grabbed one, and recomposites the mask.
    pub(super) fn selection_gizmo_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.selection_gizmo_down(ev.pos),
            PointerPhase::Move => self.selection_gizmo_move(ev.pos),
            PointerPhase::Up => self.selection_gizmo_up(),
            PointerPhase::Hover => false,
        }
    }

    fn selection_gizmo_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        // Topmost shape first (last drawn wins).
        let mut grab = None;
        for i in (0..self.paint.selection_shapes.len()).rev() {
            if let Some(h) = hit_shape(&self.paint.selection_shapes[i].shape, pos, tol) {
                grab = Some((i, h));
                break;
            }
        }
        self.paint.selection_grab = grab;
        if grab.is_some() {
            self.paint.stroke_undo = Some(self.snapshot_model());
        }
        true
    }

    fn selection_gizmo_move(&mut self, pos: [f32; 2]) -> bool {
        let Some((i, h)) = self.paint.selection_grab else {
            return false;
        };
        if i >= self.paint.selection_shapes.len() {
            return false;
        }
        let tol = self.paint.shape_grab_tol_px;
        drag_shape(&mut self.paint.selection_shapes[i].shape, h, pos, tol);
        self.recompose_selection_mask();
        true
    }

    fn selection_gizmo_up(&mut self) -> bool {
        let had = self.paint.selection_grab.take().is_some();
        if had && let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        had
    }
}

/// Hit-test one shape's handles; returns the grabbed handle id (or freehand point index) within `tol`.
fn hit_shape(shape: &SelectionShape, pos: [f32; 2], tol: f32) -> Option<u8> {
    match shape {
        SelectionShape::Ellipse { center, u, rx, ry } => {
            let h = ellipse_handles(*center, *u, *rx, *ry, tol * ROTATE_GAP);
            hit_handles(&h, pos, tol)
        }
        SelectionShape::Polygon {
            center,
            u,
            rx,
            ry,
            sides,
        } => {
            let h = polygon_handles(*center, *u, *rx, *ry, *sides, tol);
            hit_handles(&h, pos, tol)
        }
        SelectionShape::Freehand { points, .. } => {
            let tol2 = tol * tol;
            let mut best = None;
            let mut bestd = tol2;
            for (idx, p) in points.iter().enumerate().take(200) {
                let d = dist2(*p, pos);
                if d <= bestd {
                    bestd = d;
                    best = Some(idx as u8);
                }
            }
            best
        }
        SelectionShape::Raster { .. } => None,
    }
}

/// Apply a handle drag to one shape's params in place.
fn drag_shape(shape: &mut SelectionShape, handle: u8, pos: [f32; 2], tol: f32) {
    match shape {
        SelectionShape::Ellipse { center, u, rx, ry } => {
            drag_axis_frame(center, u, rx, ry, handle, pos, H_CENTER_ELLIPSE);
        }
        SelectionShape::Polygon {
            center,
            u,
            rx,
            ry,
            sides,
        } => {
            if handle == H_SIDES {
                let un = *u;
                let rel = [pos[0] - center[0], pos[1] - center[1]];
                let proj = dot(rel, un);
                let base = *rx + tol * SIDES_GAP;
                let step = (tol * SIDES_STEP).max(1.0);
                let raw = MIN_SIDES as f32 + ((proj - base) / step).round();
                *sides = (raw as i32).clamp(MIN_SIDES as i32, MAX_SIDES as i32) as u32;
            } else {
                drag_axis_frame(center, u, rx, ry, handle, pos, H_CENTER_POLYGON);
            }
        }
        SelectionShape::Freehand { points, handles } => {
            let idx = handle as usize;
            if idx < points.len() {
                // Carry the anchor's Bézier handles by the same delta so the local tangent shape is kept.
                let d = [pos[0] - points[idx][0], pos[1] - points[idx][1]];
                if idx < handles.len() {
                    for hp in &mut handles[idx] {
                        hp[0] += d[0];
                        hp[1] += d[1];
                    }
                }
                points[idx] = pos;
            }
        }
        SelectionShape::Raster { .. } => {}
    }
}

/// Shared axis-frame drag for the ellipse + polygon gizmos (right/left → rx, top/bottom → ry, rotate spins
/// the frame, `center_id` → translate). Mirrors the stroke editors' `apply_handle_drag`.
fn drag_axis_frame(
    center: &mut [f32; 2],
    u: &mut [f32; 2],
    rx: &mut f32,
    ry: &mut f32,
    handle: u8,
    pos: [f32; 2],
    center_id: u8,
) {
    let v = [-u[1], u[0]];
    let rel = [pos[0] - center[0], pos[1] - center[1]];
    if handle == H_RIGHT || handle == H_LEFT {
        *rx = dot(rel, *u).abs().max(MIN_AXIS_PX);
    } else if handle == H_TOP || handle == H_BOTTOM {
        *ry = dot(rel, v).abs().max(MIN_AXIS_PX);
    } else if handle == H_ROTATE {
        let nv = unit_or(rel, v);
        *u = [nv[1], -nv[0]];
    } else if handle == center_id {
        *center = pos;
    }
}

/// The 6 ellipse handles `[right, top, left, bottom, rotate, center]`.
fn ellipse_handles(c: [f32; 2], u: [f32; 2], rx: f32, ry: f32, gap: f32) -> [[f32; 2]; 6] {
    let v = [-u[1], u[0]];
    [
        [c[0] + u[0] * rx, c[1] + u[1] * rx],
        [c[0] + v[0] * ry, c[1] + v[1] * ry],
        [c[0] - u[0] * rx, c[1] - u[1] * rx],
        [c[0] - v[0] * ry, c[1] - v[1] * ry],
        [c[0] + v[0] * (ry + gap), c[1] + v[1] * (ry + gap)],
        c,
    ]
}

/// The 7 polygon handles `[right, top, left, bottom, rotate, sides, center]`.
fn polygon_handles(
    c: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    sides: u32,
    tol: f32,
) -> [[f32; 2]; 7] {
    let v = [-u[1], u[0]];
    let rot_gap = tol * ROTATE_GAP;
    let sides_off =
        rx + tol * SIDES_GAP + (sides.saturating_sub(MIN_SIDES)) as f32 * tol * SIDES_STEP;
    [
        [c[0] + u[0] * rx, c[1] + u[1] * rx],
        [c[0] + v[0] * ry, c[1] + v[1] * ry],
        [c[0] - u[0] * rx, c[1] - u[1] * rx],
        [c[0] - v[0] * ry, c[1] - v[1] * ry],
        [c[0] + v[0] * (ry + rot_gap), c[1] + v[1] * (ry + rot_gap)],
        [c[0] + u[0] * sides_off, c[1] + u[1] * sides_off],
        c,
    ]
}

/// Nearest axis/rotate/sides handle within `tol` (indices `0..len-1`), else the centre (last) as fallback.
fn hit_handles(handles: &[[f32; 2]], pos: [f32; 2], tol: f32) -> Option<u8> {
    let tol2 = tol * tol;
    let mut best = None;
    let mut bestd = tol2;
    let last = handles.len() - 1;
    for (i, h) in handles.iter().enumerate().take(last) {
        let d = dist2(*h, pos);
        if d <= bestd {
            bestd = d;
            best = Some(i as u8);
        }
    }
    if best.is_some() {
        return best;
    }
    if dist2(handles[last], pos) <= tol2 {
        return Some(last as u8);
    }
    None
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
/// Unit vector of `a`, or `fallback` when `a` is ~zero-length.
fn unit_or(a: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let m = (a[0] * a[0] + a[1] * a[1]).sqrt();
    if m > 1e-6 {
        [a[0] / m, a[1] / m]
    } else {
        fallback
    }
}
