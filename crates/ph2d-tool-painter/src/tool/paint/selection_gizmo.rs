//! **Isolated selection gizmos** (ADR-0103 Am.2 v2, Enio 2026-07-03). Selection editing has its OWN gizmo
//! system that NEVER touches the stroke shape editors (`self.paint.{ellipse,curve,polygon,line}`) or
//! `stroke_method` — so the Brush's Shape system can never be contaminated by selection editing (the bug).
//!
//! **All three selection gizmos are STANDARDISED to the Sprite transform gizmo** (Enio 2026-07-03): an
//! oriented bounding box with **8 scale squares** (4 corners + 4 edge mids), **rotate** engaged by grabbing
//! the ring just OUTSIDE a square (the square reads as a circle on that hover), a **centre-move square**, and
//! — only for the Polygon — the **sides diamond**. Ellipse/Polygon boxes ride the shape's orientation `u`;
//! Freehand uses the anchors' AABB and the transform is applied to every point. Each shape gets a DISTINCT
//! fluorescent accent. Pure, self-contained geometry (rotate is transcendental-free via dot·cross).

use super::PainterTool;
use super::selection_curve_gizmo::{self, CurveGrab, SelectionCurveEdit};
use super::selection_shapes::{SelectionShape, sel_polygon_vertices};
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

/// Smallest half-extent a scale drag allows (image px).
const MIN_AXIS_PX: f32 = 1.0;
/// The rotate ring reaches this many grab-radii beyond a scale square.
const ROTATE_BAND: f32 = 2.6;
/// Sides-handle offset (polygon): base + per-side step beyond the right edge, in grab-radii.
const SIDES_GAP: f32 = 3.0;
const SIDES_STEP: f32 = 1.5;
const MIN_SIDES: u32 = 3;
const MAX_SIDES: u32 = 12;

// Unified handle ids (every shape): 0..=3 corners, 4..=7 edges (R,T,L,B), 8 rotate, 9 centre-move, 10 sides.
const H_SCALE_END: u8 = 8; // scale handles are 0..8
const H_ROTATE: u8 = 8;
const H_MOVE: u8 = 9;
const H_SIDES: u8 = 10;

/// The active gizmo grab — carries the shape's PRISTINE state + pointer at grab, so every transform
/// (move/scale/rotate) is computed drift-free from the untouched geometry.
#[derive(Clone, Debug)]
pub(crate) struct SelectionGrab {
    pub shape: usize,
    pub handle: u8,
    pub start: [f32; 2],
    pub initial: SelectionShape,
    /// `Some` when the grab is a CONVERTED-curve point/handle (anchor / in / out) rather than a box handle —
    /// the drag edits the Bézier point instead of transforming the whole shape.
    pub curve: Option<CurveGrab>,
}

/// A drawable, Sprite-style selection gizmo for one shape (image-space px). `outline` is the actual shape
/// (thin); `box_corners` is the oriented bounding box; `scale_handles` are the 8 squares (corners + edges,
/// which read as circles when the cursor is in their rotate ring); `center` is the move square; `diamond` is
/// the polygon sides handle. `scale_tol`/`rotate_tol` drive the on-square-vs-rotate-ring cue. `accent` is the
/// distinct fluorescent index.
pub struct SelectionGizmoView {
    pub outline: Vec<[f32; 2]>,
    pub box_corners: [[f32; 2]; 4],
    pub scale_handles: [[f32; 2]; 8],
    pub center: [f32; 2],
    pub diamond: Option<[f32; 2]>,
    pub scale_tol: f32,
    pub rotate_tol: f32,
    pub accent: usize,
    /// `Some` for a CONVERTED curve (Freehand with Bézier handles): the shell draws its editable anchors +
    /// in/out handles INSTEAD of the transform box (a raw lasso Freehand keeps `None` = the box).
    pub edit_curve: Option<SelectionCurveEdit>,
}

/// The oriented editing frame of a shape: `center`, unit axis `u` (local +x), and half-extents `hx`/`hy`.
struct Frame {
    center: [f32; 2],
    u: [f32; 2],
    hx: f32,
    hy: f32,
}

impl Frame {
    fn v(&self) -> [f32; 2] {
        [-self.u[1], self.u[0]]
    }
    /// `[TL, TR, BR, BL, R, T, L, B]` corners + edge mids in image space.
    fn handles(&self) -> [[f32; 2]; 8] {
        let (c, u, v) = (self.center, self.u, self.v());
        let hxu = [u[0] * self.hx, u[1] * self.hx];
        let hyv = [v[0] * self.hy, v[1] * self.hy];
        let mk = |sx: f32, sy: f32| {
            [
                c[0] + sx * hxu[0] + sy * hyv[0],
                c[1] + sx * hxu[1] + sy * hyv[1],
            ]
        };
        [
            mk(-1.0, 1.0),  // 0 TL
            mk(1.0, 1.0),   // 1 TR
            mk(1.0, -1.0),  // 2 BR
            mk(-1.0, -1.0), // 3 BL
            mk(1.0, 0.0),   // 4 R
            mk(0.0, 1.0),   // 5 T
            mk(-1.0, 0.0),  // 6 L
            mk(0.0, -1.0),  // 7 B
        ]
    }
}

/// The editing frame of a selection shape (`None` for `Raster`, which has no gizmo).
fn shape_frame(shape: &SelectionShape) -> Option<Frame> {
    match shape {
        SelectionShape::Ellipse { center, u, rx, ry } => Some(Frame {
            center: *center,
            u: *u,
            hx: *rx,
            hy: *ry,
        }),
        SelectionShape::Polygon {
            center, u, rx, ry, ..
        } => Some(Frame {
            center: *center,
            u: *u,
            hx: *rx,
            hy: *ry,
        }),
        SelectionShape::Freehand { points, u, .. } => {
            // ORIENTED bounding box in the stored `u`/`v` frame, so the box rotates WITH the selection.
            let uu = unit_or(*u, [1.0, 0.0]);
            let v = [-uu[1], uu[0]];
            let (mut umin, mut umax, mut vmin, mut vmax) = (
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
            );
            for p in points {
                let du = dot(*p, uu);
                let dv = dot(*p, v);
                umin = umin.min(du);
                umax = umax.max(du);
                vmin = vmin.min(dv);
                vmax = vmax.max(dv);
            }
            if !umin.is_finite() {
                return Some(Frame {
                    center: [0.0, 0.0],
                    u: uu,
                    hx: 0.5,
                    hy: 0.5,
                });
            }
            let (uc, vc) = ((umin + umax) * 0.5, (vmin + vmax) * 0.5);
            Some(Frame {
                center: [uc * uu[0] + vc * v[0], uc * uu[1] + vc * v[1]],
                u: uu,
                hx: ((umax - umin) * 0.5).max(0.5),
                hy: ((vmax - vmin) * 0.5).max(0.5),
            })
        }
        SelectionShape::Raster { .. } => None,
    }
}

impl PainterTool {
    /// `true` while the selection gizmos are shown (the panel's **Show Selection Gizmos** checkbox).
    #[must_use]
    pub fn selection_gizmos_visible(&self) -> bool {
        self.paint.selection_edit_mode
    }

    /// Build one Sprite-style gizmo view per editable selection shape (for the shell overlay), each with a
    /// distinct fluorescent `accent`. Empty when the gizmos are hidden or nothing is selected.
    #[must_use]
    pub fn selection_gizmos(&self) -> Vec<SelectionGizmoView> {
        if !self.paint.selection_edit_mode {
            return Vec::new();
        }
        let tol = self.paint.shape_grab_tol_px;
        let mut out = Vec::new();
        for (accent, e) in self.paint.selection_shapes.iter().enumerate() {
            let Some(f) = shape_frame(&e.shape) else {
                continue;
            };
            let h = f.handles();
            out.push(SelectionGizmoView {
                outline: self.shape_outline(&e.shape),
                box_corners: [h[0], h[1], h[2], h[3]],
                scale_handles: h,
                center: f.center,
                diamond: polygon_sides_handle(&e.shape, tol),
                scale_tol: tol,
                rotate_tol: tol * ROTATE_BAND,
                accent,
                // A converted curve edits per-anchor (points + Bézier handles), not via the transform box.
                edit_curve: selection_curve_gizmo::curve_edit_view(&e.shape),
            });
        }
        out
    }

    /// The thin shape boundary drawn INSIDE the transform box (ellipse perimeter / polygon perimeter /
    /// freehand spine), so the artist still sees the actual selection shape.
    fn shape_outline(&self, shape: &SelectionShape) -> Vec<[f32; 2]> {
        match shape {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                let mut p = Vec::new();
                ph2d_painter_brush::ellipse_perimeter(*center, *u, *rx, *ry, &mut p);
                p
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => sel_polygon_vertices(*center, *u, *rx, *ry, *sides),
            SelectionShape::Freehand {
                points, handles, ..
            } => self.freehand_spine(points, handles),
            SelectionShape::Raster { .. } => Vec::new(),
        }
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
            let shape = &self.paint.selection_shapes[i].shape;
            // A CONVERTED curve edits its anchors/handles (points), never the transform box.
            if selection_curve_gizmo::is_converted_curve(shape) {
                if let Some(cg) = selection_curve_gizmo::hit_curve(shape, pos, tol) {
                    grab = Some(SelectionGrab {
                        shape: i,
                        handle: H_MOVE,
                        start: pos,
                        initial: shape.clone(),
                        curve: Some(cg),
                    });
                    break;
                }
                continue; // no box fallback for a converted curve
            }
            if let Some(h) = hit_shape(shape, pos, tol) {
                grab = Some(SelectionGrab {
                    shape: i,
                    handle: h,
                    start: pos,
                    initial: shape.clone(),
                    curve: None,
                });
                break;
            }
        }
        let has = grab.is_some();
        self.paint.selection_grab = grab;
        if has {
            self.paint.stroke_undo = Some(self.snapshot_model());
        }
        true
    }

    fn selection_gizmo_move(&mut self, pos: [f32; 2]) -> bool {
        let Some(grab) = self.paint.selection_grab.clone() else {
            return false;
        };
        if grab.shape >= self.paint.selection_shapes.len() {
            return false;
        }
        let tol = self.paint.shape_grab_tol_px;
        self.paint.selection_shapes[grab.shape].shape = if let Some(cg) = grab.curve {
            selection_curve_gizmo::apply_curve_drag(&grab.initial, cg, grab.start, pos)
        } else {
            apply_gizmo_drag(&grab.initial, grab.handle, grab.start, pos, tol)
        };
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

/// The polygon's sides diamond handle (beyond the right edge along `u`), or `None` for non-polygons.
fn polygon_sides_handle(shape: &SelectionShape, tol: f32) -> Option<[f32; 2]> {
    if let SelectionShape::Polygon {
        center,
        u,
        rx,
        sides,
        ..
    } = shape
    {
        let off =
            rx + tol * SIDES_GAP + (sides.saturating_sub(MIN_SIDES)) as f32 * tol * SIDES_STEP;
        Some([center[0] + u[0] * off, center[1] + u[1] * off])
    } else {
        None
    }
}

/// Hit-test one shape's unified gizmo; returns the grabbed handle id within tolerance.
fn hit_shape(shape: &SelectionShape, pos: [f32; 2], tol: f32) -> Option<u8> {
    let f = shape_frame(shape)?;
    let handles = f.handles();
    let tol2 = tol * tol;
    // 1) ON a scale square → scale.
    let mut best = None;
    let mut bestd = tol2;
    for (i, h) in handles.iter().enumerate() {
        let d = dist2(*h, pos);
        if d <= bestd {
            bestd = d;
            best = Some(i as u8);
        }
    }
    if let Some(i) = best {
        return Some(i);
    }
    // 2) Sides diamond (polygon).
    if let Some(sh) = polygon_sides_handle(shape, tol)
        && dist2(sh, pos) <= tol2
    {
        return Some(H_SIDES);
    }
    // 3) Centre-move square.
    if dist2(f.center, pos) <= tol2 {
        return Some(H_MOVE);
    }
    // 4) Rotate ring — the band just OUTSIDE a scale square (farther from the centre than it).
    let rot2 = (tol * ROTATE_BAND) * (tol * ROTATE_BAND);
    for h in &handles {
        let d = dist2(*h, pos);
        if d > tol2 && d <= rot2 && dist2(f.center, pos) > dist2(*h, f.center) {
            return Some(H_ROTATE);
        }
    }
    None
}

/// Apply a gizmo drag to the PRISTINE `initial` shape and return the transformed shape (drift-free).
fn apply_gizmo_drag(
    initial: &SelectionShape,
    handle: u8,
    start: [f32; 2],
    pos: [f32; 2],
    tol: f32,
) -> SelectionShape {
    let Some(f) = shape_frame(initial) else {
        return initial.clone();
    };
    if handle == H_SIDES {
        return drag_sides(initial, &f, pos, tol);
    }
    if handle == H_MOVE {
        let d = [pos[0] - start[0], pos[1] - start[1]];
        return transform_shape(initial, |p| [p[0] + d[0], p[1] + d[1]], f.u, f.u);
    }
    if handle == H_ROTATE {
        let v0 = unit_or([start[0] - f.center[0], start[1] - f.center[1]], [1.0, 0.0]);
        let v1 = unit_or([pos[0] - f.center[0], pos[1] - f.center[1]], [1.0, 0.0]);
        let cos = dot(v0, v1);
        let sin = v0[0] * v1[1] - v0[1] * v1[0];
        let c = f.center;
        let nu = [f.u[0] * cos - f.u[1] * sin, f.u[0] * sin + f.u[1] * cos];
        return transform_shape(
            initial,
            move |p| {
                let r = [p[0] - c[0], p[1] - c[1]];
                [
                    c[0] + r[0] * cos - r[1] * sin,
                    c[1] + r[0] * sin + r[1] * cos,
                ]
            },
            f.u,
            nu,
        );
    }
    // Scale (handles 0..8): map the pointer onto the frame axes → new half-extents.
    if handle < H_SCALE_END {
        let v = f.v();
        let rel = [pos[0] - f.center[0], pos[1] - f.center[1]];
        let du = dot(rel, f.u).abs().max(MIN_AXIS_PX);
        let dv = dot(rel, v).abs().max(MIN_AXIS_PX);
        // Corners (0..3) scale both axes; edge R/L (4/6) scale hx; edge T/B (5/7) scale hy.
        let (nhx, nhy) = match handle {
            0..=3 => (du, dv),
            4 | 6 => (du, f.hy),
            _ => (f.hx, dv),
        };
        return scale_shape(initial, &f, nhx, nhy);
    }
    initial.clone()
}

/// Rewrite a shape under a positional transform `xf` (applied to every geometric point), plus the new axis
/// `new_u` for the parametric shapes (ellipse/polygon) whose orientation is stored explicitly.
fn transform_shape(
    shape: &SelectionShape,
    xf: impl Fn([f32; 2]) -> [f32; 2],
    _old_u: [f32; 2],
    new_u: [f32; 2],
) -> SelectionShape {
    match shape {
        SelectionShape::Ellipse { rx, ry, center, .. } => SelectionShape::Ellipse {
            center: xf(*center),
            u: new_u,
            rx: *rx,
            ry: *ry,
        },
        SelectionShape::Polygon {
            rx,
            ry,
            sides,
            center,
            ..
        } => SelectionShape::Polygon {
            center: xf(*center),
            u: new_u,
            rx: *rx,
            ry: *ry,
            sides: *sides,
        },
        SelectionShape::Freehand {
            points, handles, ..
        } => SelectionShape::Freehand {
            points: points.iter().map(|&p| xf(p)).collect(),
            handles: handles.iter().map(|h| [xf(h[0]), xf(h[1])]).collect(),
            // The box orientation follows the shape (identity for move; rotated for rotate).
            u: new_u,
        },
        SelectionShape::Raster { .. } => shape.clone(),
    }
}

/// Scale a shape about its frame centre to new half-extents `nhx`/`nhy` (parametric → set rx/ry; freehand →
/// scale every point/handle in the frame's u/v axes).
fn scale_shape(shape: &SelectionShape, f: &Frame, nhx: f32, nhy: f32) -> SelectionShape {
    match shape {
        SelectionShape::Ellipse { center, u, .. } => SelectionShape::Ellipse {
            center: *center,
            u: *u,
            rx: nhx,
            ry: nhy,
        },
        SelectionShape::Polygon {
            center, u, sides, ..
        } => SelectionShape::Polygon {
            center: *center,
            u: *u,
            rx: nhx,
            ry: nhy,
            sides: *sides,
        },
        SelectionShape::Freehand {
            points, handles, ..
        } => {
            let (c, u, v) = (f.center, f.u, f.v());
            let sx = nhx / f.hx.max(0.001);
            let sy = nhy / f.hy.max(0.001);
            let scale = |p: [f32; 2]| {
                let d = [p[0] - c[0], p[1] - c[1]];
                let du = dot(d, u) * sx;
                let dv = dot(d, v) * sy;
                [c[0] + du * u[0] + dv * v[0], c[1] + du * u[1] + dv * v[1]]
            };
            SelectionShape::Freehand {
                points: points.iter().map(|&p| scale(p)).collect(),
                handles: handles.iter().map(|h| [scale(h[0]), scale(h[1])]).collect(),
                u: f.u, // scaling keeps the box orientation
            }
        }
        SelectionShape::Raster { .. } => shape.clone(),
    }
}

/// Drag the polygon **sides** diamond → new side count from the pointer's projection along `u`.
fn drag_sides(shape: &SelectionShape, f: &Frame, pos: [f32; 2], tol: f32) -> SelectionShape {
    let SelectionShape::Polygon {
        center, u, rx, ry, ..
    } = shape
    else {
        return shape.clone();
    };
    let rel = [pos[0] - f.center[0], pos[1] - f.center[1]];
    let proj = dot(rel, *u);
    let base = *rx + tol * SIDES_GAP;
    let step = (tol * SIDES_STEP).max(1.0);
    let raw = MIN_SIDES as f32 + ((proj - base) / step).round();
    let sides = (raw as i32).clamp(MIN_SIDES as i32, MAX_SIDES as i32) as u32;
    SelectionShape::Polygon {
        center: *center,
        u: *u,
        rx: *rx,
        ry: *ry,
        sides,
    }
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
