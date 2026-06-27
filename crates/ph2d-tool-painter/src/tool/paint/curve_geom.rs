//! Pure geometry helpers for the Curve / Free Hand editor — flattening (incl. the closed-loop seam),
//! point hit-test / nearest / insert-index, and the small distance primitives. Transcendental-free
//! (projection + `sqrt` only, HR-5). Split from [`super::curve`] for the workspace LOC cap; these are
//! free fns with no `CurveEditor` access, so the editor calls them as `curve_geom::*`.

use ph2d_painter_brush::{flatten_bezier, flatten_catmull_rom};

/// Flatten the curve to a dense spine: Bézier (explicit handles) else the Catmull-Rom auto-smooth. When
/// `closed`, append the closing segment (last anchor → first, via `handles[last].out` + `handles[0].in`),
/// reusing `flatten_bezier` on that 2-anchor sub-curve. The fill and the overlay both call this, so the
/// painted dabs match the drawn guide exactly.
pub(super) fn flatten_spine(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    closed: bool,
    out: &mut Vec<[f32; 2]>,
) {
    if handles.len() == points.len() && points.len() >= 2 {
        flatten_bezier(points, handles, out);
        if closed {
            let last = points.len() - 1;
            let mut tail = Vec::new();
            flatten_bezier(
                &[points[last], points[0]],
                &[[points[last], handles[last][1]], [handles[0][0], points[0]]],
                &mut tail,
            );
            out.extend_from_slice(&tail[1..]); // skip the duplicate start point
        }
    } else {
        flatten_catmull_rom(points, out);
    }
}

/// Squared distance between two points (the Free Hand capture's min-spacing gate).
pub(super) fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Clamp a fitted control polygon to `max`: a clean fit is already tiny, but a very busy scribble can
/// exceed the editor's per-frame cap, so uniformly decimate the interior, keeping the true endpoints.
/// A no-op (returns as-is) when already within the cap.
pub(super) fn cap_curve_points(out: Vec<[f32; 2]>, max: usize) -> Vec<[f32; 2]> {
    if out.len() <= max {
        return out;
    }
    let last = out.len() - 1;
    let step = out.len() as f32 / max as f32;
    let mut capped: Vec<[f32; 2]> = (0..max)
        .map(|i| out[((i as f32 * step) as usize).min(last)])
        .collect();
    capped[max - 1] = out[last];
    capped
}

/// Index of the control point within `tol` px of `pos` (closest wins), or `None` on a miss.
pub(super) fn curve_hit(pts: &[[f32; 2]], pos: [f32; 2], tol: f32) -> Option<usize> {
    let mut best = None;
    let mut bestd = tol * tol;
    for (i, p) in pts.iter().enumerate() {
        let d = dist2(*p, pos);
        if d <= bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}

/// Index of the nearest control point (no tolerance) — the select fallback at the point cap.
pub(super) fn curve_nearest(pts: &[[f32; 2]], pos: [f32; 2]) -> Option<usize> {
    let mut best = None;
    let mut bestd = f32::INFINITY;
    for (i, p) in pts.iter().enumerate() {
        let d = dist2(*p, pos);
        if d < bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}

/// Insert position for a new point at `pos`: just after the segment whose body it lies closest to, so a
/// click on/near the curve subdivides it there (a click in open space drops into the nearest run).
pub(super) fn curve_insert_index(pts: &[[f32; 2]], pos: [f32; 2]) -> usize {
    if pts.len() < 2 {
        return pts.len();
    }
    let mut best = 0usize;
    let mut bestd = f32::INFINITY;
    for i in 0..pts.len() - 1 {
        let d = dist2_point_seg(pos, pts[i], pts[i + 1]);
        if d < bestd {
            bestd = d;
            best = i;
        }
    }
    best + 1
}

/// Offset the curve's **control geometry** perpendicular by `d` px (the Offset slider): translate each
/// anchor AND its two handles by that anchor's averaged right-normal, so the flattened spine + the control
/// dots + the tangents all move TOGETHER (the curve moves with the paint). `closed` wraps the endpoint
/// normals (a converted Circle / Polygon). `d == 0` (or a length mismatch) returns unchanged clones.
pub(super) fn offset_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    d: f32,
    closed: bool,
) -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    if d == 0.0 || points.len() < 2 || handles.len() != points.len() {
        return (points.to_vec(), handles.to_vec());
    }
    let mut op = Vec::with_capacity(points.len());
    let mut oh = Vec::with_capacity(points.len());
    for i in 0..points.len() {
        let nrm = vertex_normal(points, i, closed);
        let dv = [nrm[0] * d, nrm[1] * d];
        op.push([points[i][0] + dv[0], points[i][1] + dv[1]]);
        oh.push([
            [handles[i][0][0] + dv[0], handles[i][0][1] + dv[1]],
            [handles[i][1][0] + dv[0], handles[i][1][1] + dv[1]],
        ]);
    }
    (op, oh)
}

/// The unit **right-normal** at anchor `i` (averaged over its incident segments), so a positive offset
/// outsets a CCW closed loop. Zero vector for a degenerate vertex (the point doesn't move).
fn vertex_normal(points: &[[f32; 2]], i: usize, closed: bool) -> [f32; 2] {
    let n = points.len();
    let prev = if i > 0 {
        Some((points[i - 1], points[i]))
    } else if closed {
        Some((points[n - 1], points[i]))
    } else {
        None
    };
    let next = if i + 1 < n {
        Some((points[i], points[i + 1]))
    } else if closed {
        Some((points[i], points[0]))
    } else {
        None
    };
    let mut acc = [0.0f32, 0.0];
    for (a, b) in [prev, next].into_iter().flatten() {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1e-6 {
            acc[0] += dy / len; // right-normal of direction (dx,dy) is (dy, -dx)
            acc[1] += -dx / len;
        }
    }
    let al = (acc[0] * acc[0] + acc[1] * acc[1]).sqrt();
    if al > 1e-6 {
        [acc[0] / al, acc[1] / al]
    } else {
        [0.0, 0.0]
    }
}

/// Squared distance from `p` to the segment `a→b` — transcendental-free (projection + clamp).
fn dist2_point_seg(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let abx = b[0] - a[0];
    let aby = b[1] - a[1];
    let apx = p[0] - a[0];
    let apy = p[1] - a[1];
    let denom = abx * abx + aby * aby;
    let t = if denom > 0.0 {
        ((apx * abx + apy * aby) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let cx = a[0] + abx * t;
    let cy = a[1] + aby * t;
    dist2(p, [cx, cy])
}
