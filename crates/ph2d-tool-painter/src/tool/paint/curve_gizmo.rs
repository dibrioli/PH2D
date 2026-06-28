//! Whole-curve **transform gizmo** for the Curve / Free Hand editor — move / scale / rotate the ENTIRE
//! curve as one, via a bounding-box gizmo: the 4 corners scale both axes, the 4 edge mids scale one, the
//! centre moves, and the ring just outside a corner rotates (the Stencil-gizmo precedent). Pure,
//! transcendental-free affine maths — the rotation is a complex multiply of the grab/cursor unit vectors
//! (dot = cos, cross = sin), like `curve_geom::offset_curve` (HR-5). Split from `curve` for the workspace
//! LOC cap. The grab snapshots the control geometry at pointer-down, so the transform maps the SAME source
//! every move (drift-free), and manual handles ride along while derived (Auto / Vector) ones are rebuilt.

use super::curve_geom::dist2;

/// Margin (× the grab tolerance) the gizmo box is inflated beyond the control-point bounds, so its handles
/// sit clear of the anchors / tangents (an extreme anchor under a corner is still grabbable — anchors win).
const GIZMO_MARGIN: f32 = 1.8;
/// The rotate ring reaches from a corner out to this × the grab tolerance (matches the Stencil gizmo band).
const GIZMO_ROTATE_BAND: f32 = 2.6;

/// The transform-gizmo overlay snapshot for the shell. Handle indices: `0..=3` corners (TL, TR, BR, BL),
/// `4..=7` edge mids (top, right, bottom, left), `8` centre. The shell draws the box outline + handle dots;
/// the grabbed handle highlights, and a rotation re-styles the corners (rings, not squares).
pub struct TransformGizmo {
    /// The inflated box `[min, max]` in image-space px — the outline + the corner / edge handle frame.
    pub bbox: [[f32; 2]; 2],
    /// The 9 handle positions (image px) in the order documented on the type.
    pub handles: [[f32; 2]; 9],
    /// The grabbed handle index (`0..=8`) for highlighting, or `None`.
    pub grabbed: Option<u8>,
    /// The active grab is a rotation (the shell draws the corners as rotate rings, not scale squares).
    pub rotating: bool,
}

/// What a live gizmo drag does + the snapshot it maps. Held in the `CurveEditor` only between pointer-down
/// and up. Every move re-maps `snap_*` (never the previous frame), so the transform can't accumulate drift.
pub(super) struct GizmoGrab {
    pub(super) drag: GizmoDrag,
    /// The pointer position at grab-down — the Move delta origin.
    grab_pos: [f32; 2],
    /// Visual handle index for the overlay highlight.
    pub(super) handle: u8,
    /// Control points at grab-down — every transform maps THESE.
    snap_points: Vec<[f32; 2]>,
    /// Handles at grab-down, parallel to `snap_points`.
    snap_handles: Vec<[[f32; 2]; 2]>,
}

/// The kind of whole-curve transform a [`GizmoGrab`] applies.
pub(super) enum GizmoDrag {
    /// Translate the whole curve by the cursor delta.
    Move,
    /// Scale about `pivot` (box centre); `ref_corner` is the grabbed handle at grab; `axes` gates x / y
    /// (a corner scales both, an edge mid one).
    Scale {
        pivot: [f32; 2],
        ref_corner: [f32; 2],
        axes: [bool; 2],
    },
    /// Rotate about `pivot` (box centre); `start` is the grab cursor relative to the pivot.
    Rotate { pivot: [f32; 2], start: [f32; 2] },
}

/// Axis-aligned bounding box `[min, max]` of `points`, inflated by `margin` px on every side. `None` for an
/// empty slice (the editor always holds ≥ 2 points while editing).
fn inflated_bbox(points: &[[f32; 2]], margin: f32) -> Option<[[f32; 2]; 2]> {
    let first = *points.first()?;
    let (mut min, mut max) = (first, first);
    for p in points {
        min = [min[0].min(p[0]), min[1].min(p[1])];
        max = [max[0].max(p[0]), max[1].max(p[1])];
    }
    Some([
        [min[0] - margin, min[1] - margin],
        [max[0] + margin, max[1] + margin],
    ])
}

/// The 9 handle positions from box `[min, max]`: 4 corners (TL, TR, BR, BL), 4 edge mids (T, R, B, L), centre.
fn gizmo_handles(bbox: [[f32; 2]; 2]) -> [[f32; 2]; 9] {
    let [mn, mx] = bbox;
    let (cx, cy) = ((mn[0] + mx[0]) * 0.5, (mn[1] + mx[1]) * 0.5);
    [
        [mn[0], mn[1]],
        [mx[0], mn[1]],
        [mx[0], mx[1]],
        [mn[0], mx[1]], // corners
        [cx, mn[1]],
        [mx[0], cy],
        [cx, mx[1]],
        [mn[0], cy], // edge mids T, R, B, L
        [cx, cy],    // centre
    ]
}

/// Build the gizmo overlay for the (already offset) control `points` — `None` when there are none. `grabbed`
/// / `rotating` come from the live grab so the shell can highlight + re-style.
pub(super) fn overlay(
    points: &[[f32; 2]],
    tol: f32,
    grabbed: Option<u8>,
    rotating: bool,
) -> Option<TransformGizmo> {
    let bbox = inflated_bbox(points, tol * GIZMO_MARGIN)?;
    Some(TransformGizmo {
        bbox,
        handles: gizmo_handles(bbox),
        grabbed,
        rotating,
    })
}

/// Hit-test the gizmo at `pos`; on a hit, snapshot `points` / `handles` into a [`GizmoGrab`]. Priority:
/// corner (scale both) → edge mid (scale one axis) → the ring just outside a corner (rotate) → centre (move).
pub(super) fn grab(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    pos: [f32; 2],
    tol: f32,
) -> Option<GizmoGrab> {
    let bbox = inflated_bbox(points, tol * GIZMO_MARGIN)?;
    let hs = gizmo_handles(bbox);
    let pivot = hs[8];
    let tol2 = tol * tol;
    let mk = |drag, handle| {
        Some(GizmoGrab {
            drag,
            grab_pos: pos,
            handle,
            snap_points: points.to_vec(),
            snap_handles: handles.to_vec(),
        })
    };
    for (i, &h) in hs.iter().enumerate().take(4) {
        if dist2(h, pos) <= tol2 {
            return mk(
                GizmoDrag::Scale {
                    pivot,
                    ref_corner: h,
                    axes: [true, true],
                },
                i as u8,
            );
        }
    }
    for (i, &h) in hs.iter().enumerate().take(8).skip(4) {
        if dist2(h, pos) <= tol2 {
            let axes = if i == 4 || i == 6 {
                [false, true] // top / bottom → vertical
            } else {
                [true, false] // right / left → horizontal
            };
            return mk(
                GizmoDrag::Scale {
                    pivot,
                    ref_corner: h,
                    axes,
                },
                i as u8,
            );
        }
    }
    // Ring just OUTSIDE a corner (farther from the pivot than the corner) = rotate about the centre.
    let band2 = (tol * GIZMO_ROTATE_BAND).powi(2);
    for (i, &h) in hs.iter().enumerate().take(4) {
        let d2 = dist2(h, pos);
        if d2 > tol2 && d2 <= band2 && dist2(pivot, pos) > dist2(pivot, h) {
            return mk(
                GizmoDrag::Rotate {
                    pivot,
                    start: [pos[0] - pivot[0], pos[1] - pivot[1]],
                },
                i as u8,
            );
        }
    }
    if dist2(hs[8], pos) <= tol2 {
        return mk(GizmoDrag::Move, 8);
    }
    None
}

/// Apply the live drag to the grab's snapshot, returning the transformed `(points, handles)`. Affine only:
/// Move translates, Scale multiplies about the pivot per axis, Rotate spins about the pivot via the
/// grab→cursor rotation (complex multiply of the unit vectors — no `atan2`, HR-5).
pub(super) fn apply(g: &GizmoGrab, pos: [f32; 2]) -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    let f: Box<dyn Fn([f32; 2]) -> [f32; 2]> = match &g.drag {
        GizmoDrag::Move => {
            let d = [pos[0] - g.grab_pos[0], pos[1] - g.grab_pos[1]];
            Box::new(move |p| [p[0] + d[0], p[1] + d[1]])
        }
        GizmoDrag::Scale {
            pivot,
            ref_corner,
            axes,
        } => {
            let sx = if axes[0] {
                safe_ratio(pos[0] - pivot[0], ref_corner[0] - pivot[0])
            } else {
                1.0
            };
            let sy = if axes[1] {
                safe_ratio(pos[1] - pivot[1], ref_corner[1] - pivot[1])
            } else {
                1.0
            };
            let pivot = *pivot;
            Box::new(move |p| {
                [
                    pivot[0] + (p[0] - pivot[0]) * sx,
                    pivot[1] + (p[1] - pivot[1]) * sy,
                ]
            })
        }
        GizmoDrag::Rotate { pivot, start } => {
            let (c, s) = rot_cos_sin(*start, [pos[0] - pivot[0], pos[1] - pivot[1]]);
            let pivot = *pivot;
            Box::new(move |p| {
                let v = [p[0] - pivot[0], p[1] - pivot[1]];
                [
                    pivot[0] + v[0] * c - v[1] * s,
                    pivot[1] + v[0] * s + v[1] * c,
                ]
            })
        }
    };
    let points = g.snap_points.iter().map(|&p| f(p)).collect();
    let handles = g.snap_handles.iter().map(|h| [f(h[0]), f(h[1])]).collect();
    (points, handles)
}

/// Whether this drag is a rotation (drives the shell's corner ring cue).
pub(super) fn is_rotate(g: &GizmoGrab) -> bool {
    matches!(g.drag, GizmoDrag::Rotate { .. })
}

/// Scale factor `num / den`, guarded to 1.0 for a (near) zero denominator (a degenerate box axis).
fn safe_ratio(num: f32, den: f32) -> f32 {
    if den.abs() <= 1e-4 { 1.0 } else { num / den }
}

/// `(cos, sin)` of the rotation taking `unit(from) → unit(to)` — dot + cross of the unit vectors. Identity
/// for a degenerate input (no spin until the cursor leaves the pivot).
fn rot_cos_sin(from: [f32; 2], to: [f32; 2]) -> (f32, f32) {
    let fl = (from[0] * from[0] + from[1] * from[1]).sqrt();
    let tl = (to[0] * to[0] + to[1] * to[1]).sqrt();
    if fl <= 1e-6 || tl <= 1e-6 {
        return (1.0, 0.0);
    }
    let (fx, fy) = (from[0] / fl, from[1] / fl);
    let (tx, ty) = (to[0] / tl, to[1] / tl);
    (fx * tx + fy * ty, fx * ty - fy * tx)
}

#[cfg(test)]
mod tests;
