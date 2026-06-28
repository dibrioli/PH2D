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

/// A point insertion on the curve: where to splice the new anchor + the **de Casteljau split** of the
/// chosen cubic, so the new point lands exactly on the curve and the SHAPE is unchanged (the split sums
/// to the original arc). Applied by [`super::PainterTool::curve_down`].
pub(super) struct CurveInsert {
    /// Index to insert the new anchor at (`prev_idx + 1`; the closing-seam insert appends at the end).
    pub index: usize,
    /// The new anchor (the split point — exactly on the curve).
    pub anchor: [f32; 2],
    /// The new anchor's `[in, out]` handles (collinear — a smooth split point).
    pub handles: [[f32; 2]; 2],
    /// The segment's start anchor index; its OUT handle becomes [`Self::prev_out`].
    pub prev_idx: usize,
    /// The segment's end anchor index (pre-insert); its IN handle becomes [`Self::next_in`].
    pub next_idx: usize,
    /// Replacement OUT handle for `prev_idx` (de Casteljau Q0) — a sharp/collapsed side stays collapsed.
    pub prev_out: [f32; 2],
    /// Replacement IN handle for `next_idx` (de Casteljau Q2).
    pub next_in: [f32; 2],
}

/// Find where to insert a new anchor for a click at `pos`: the NEAREST point on the actual curve over
/// EVERY segment — including the closing seam when `closed` (the converted Circle / Polygon bug: the old
/// straight-control-polygon scan ignored the seam + landed off the bulge) — then split that cubic there.
pub(super) fn curve_insert(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    closed: bool,
    pos: [f32; 2],
) -> CurveInsert {
    let n = points.len();
    let bezier = handles.len() == n && n >= 2;
    let seg_count = if closed { n } else { n - 1 };
    let mut best_d = f32::INFINITY;
    let mut best: Option<(usize, usize, f32, [[f32; 2]; 4])> = None; // (i, j, t, cubic)
    for i in 0..seg_count {
        let j = (i + 1) % n;
        // The cubic for segment i→j: a collapsed handle ⇒ that control point sits on its anchor (a line).
        let cubic = if bezier {
            [points[i], handles[i][1], handles[j][0], points[j]]
        } else {
            [points[i], points[i], points[j], points[j]]
        };
        let (t, d) = nearest_t_on_cubic(&cubic, pos);
        if d < best_d {
            best_d = d;
            best = Some((i, j, t, cubic));
        }
    }
    let Some((i, j, t, cubic)) = best else {
        return CurveInsert {
            index: n,
            anchor: pos,
            handles: [pos, pos],
            prev_idx: 0,
            next_idx: 0,
            prev_out: pos,
            next_in: pos,
        };
    };
    let [q0, r0, s, r1, q2] = split_cubic(&cubic, t);
    CurveInsert {
        index: i + 1,
        anchor: s,
        handles: [r0, r1],
        prev_idx: i,
        next_idx: j,
        prev_out: q0,
        next_in: q2,
    }
}

/// Nearest parameter `t ∈ [0, 1]` on the cubic `b` to `pos` + its squared distance: a coarse scan then a
/// few ternary-search refinements (transcendental-free; the curve is unimodal enough locally).
fn nearest_t_on_cubic(b: &[[f32; 2]; 4], pos: [f32; 2]) -> (f32, f32) {
    const STEPS: usize = 24;
    let mut best_t = 0.0;
    let mut best_d = f32::INFINITY;
    for k in 0..=STEPS {
        let t = k as f32 / STEPS as f32;
        let d = dist2(cubic_at(b, t), pos);
        if d < best_d {
            best_d = d;
            best_t = t;
        }
    }
    let span = 1.0 / STEPS as f32;
    let (mut lo, mut hi) = ((best_t - span).max(0.0), (best_t + span).min(1.0));
    for _ in 0..24 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        if dist2(cubic_at(b, m1), pos) < dist2(cubic_at(b, m2), pos) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    let t = (lo + hi) * 0.5;
    (t, dist2(cubic_at(b, t), pos))
}

/// Evaluate the cubic Bézier `b` at `t` (Bernstein form; transcendental-free).
fn cubic_at(b: &[[f32; 2]; 4], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    [
        b[0][0] * w[0] + b[1][0] * w[1] + b[2][0] * w[2] + b[3][0] * w[3],
        b[0][1] * w[0] + b[1][1] * w[1] + b[2][1] * w[2] + b[3][1] * w[3],
    ]
}

/// **de Casteljau** split of cubic `b` at `t` → `[prev_out, new_in, anchor, new_out, next_in]`: the two
/// sub-cubics `[P0, q0, r0, s]` + `[s, r1, q2, P3]` reproduce `b` exactly, so subdividing never deforms it.
fn split_cubic(b: &[[f32; 2]; 4], t: f32) -> [[f32; 2]; 5] {
    let lerp = |a: [f32; 2], c: [f32; 2]| [a[0] + (c[0] - a[0]) * t, a[1] + (c[1] - a[1]) * t];
    let q0 = lerp(b[0], b[1]);
    let q1 = lerp(b[1], b[2]);
    let q2 = lerp(b[2], b[3]);
    let r0 = lerp(q0, q1);
    let r1 = lerp(q1, q2);
    let s = lerp(r0, r1);
    [q0, r0, s, r1, q2]
}

/// Offset the curve's **control geometry** perpendicular by `d` px (the Offset slider). Each anchor shifts
/// along its averaged right-normal, then each tangent handle is RECALCULATED — rotated + scaled to follow
/// its (now moved) segment — rather than rigidly translated, so the offset curve stays faithful: a circle
/// offsets to a clean concentric circle (handles scale by the radius ratio), sharp corners stay sharp
/// (a collapsed handle has a zero vector → stays collapsed), and there's no control-level deformation
/// (the rigid-translate version warped tight bends — Enio 2026-06-28). The spine, dots, tangents + paint
/// all derive from this, so they move TOGETHER. `closed` wraps the endpoints. Transcendental-free (the
/// segment rotation is a complex multiply from the chord unit vectors' dot/cross). `d == 0` (or a length
/// mismatch) ⇒ unchanged clones.
pub(super) fn offset_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    d: f32,
    closed: bool,
) -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    let n = points.len();
    if d == 0.0 || n < 2 || handles.len() != n {
        return (points.to_vec(), handles.to_vec());
    }
    // 1. Offset the anchors along their averaged vertex normals.
    let op: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let nrm = vertex_normal(points, i, closed);
            [points[i][0] + nrm[0] * d, points[i][1] + nrm[1] * d]
        })
        .collect();
    // 2. Recalculate handles per segment: `out[a]` + `in[b]` share segment a→b's rotate+scale, so they
    //    track the offset chord. Default to a translate (fallback for the unused open endpoints / degenerate
    //    segments), then overwrite each segment's two handles.
    let mut oh: Vec<[[f32; 2]; 2]> = (0..n)
        .map(|i| {
            let dv = [op[i][0] - points[i][0], op[i][1] - points[i][1]];
            [
                [handles[i][0][0] + dv[0], handles[i][0][1] + dv[1]],
                [handles[i][1][0] + dv[0], handles[i][1][1] + dv[1]],
            ]
        })
        .collect();
    let seg_count = if closed { n } else { n - 1 };
    for s in 0..seg_count {
        let (a, b) = (s, (s + 1) % n);
        let xf = SegXform::new(points[a], points[b], op[a], op[b]);
        oh[a][1] = xf.apply(points[a], handles[a][1], op[a]); // out handle of a
        oh[b][0] = xf.apply(points[b], handles[b][0], op[b]); // in handle of b
    }
    (op, oh)
}

/// The rotate-and-scale that maps a base segment's chord to its offset chord — applied to a tangent handle
/// so it follows the offset (a circle's handles scale by the radius ratio, no spurious bend). Identity for
/// a degenerate base segment.
struct SegXform {
    cos: f32,
    sin: f32,
    scale: f32,
}

impl SegXform {
    fn new(a: [f32; 2], b: [f32; 2], oa: [f32; 2], ob: [f32; 2]) -> Self {
        let old = [b[0] - a[0], b[1] - a[1]];
        let new = [ob[0] - oa[0], ob[1] - oa[1]];
        let old_len = (old[0] * old[0] + old[1] * old[1]).sqrt();
        let new_len = (new[0] * new[0] + new[1] * new[1]).sqrt();
        if old_len <= 1e-6 || new_len <= 1e-6 {
            return Self {
                cos: 1.0,
                sin: 0.0,
                scale: 1.0,
            };
        }
        let (ox, oy) = (old[0] / old_len, old[1] / old_len);
        let (nx, ny) = (new[0] / new_len, new[1] / new_len);
        Self {
            cos: ox * nx + oy * ny, // dot → cos of the turn from old→new chord direction
            sin: ox * ny - oy * nx, // cross → sin of the turn
            scale: new_len / old_len,
        }
    }

    /// Re-place `handle` (an absolute control point) relative to its offset anchor: rotate its vector off the
    /// old anchor by the segment turn, scale by the chord ratio, re-anchor at the offset anchor.
    fn apply(&self, anchor: [f32; 2], handle: [f32; 2], offset_anchor: [f32; 2]) -> [f32; 2] {
        let v = [handle[0] - anchor[0], handle[1] - anchor[1]];
        let rx = (v[0] * self.cos - v[1] * self.sin) * self.scale;
        let ry = (v[0] * self.sin + v[1] * self.cos) * self.scale;
        [offset_anchor[0] + rx, offset_anchor[1] + ry]
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsetting_a_circle_stays_concentric_and_scales_the_handles() {
        // 4 cardinal anchors, radius 20, with circle tangent handles (k ≈ 0.5523·r). Offsetting the closed
        // loop outward must keep every anchor equidistant from the centre (a clean concentric circle) AND
        // scale the handles by the radius ratio — proof the recompute follows the curve without deforming it
        // (the old rigid translate warped it — Enio 2026-06-28).
        let r = 20.0f32;
        let k = 0.55228f32 * r;
        let pts = vec![[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
        let handles = vec![
            [[r, -k], [r, k]],   // (r,0): tangent vertical
            [[k, r], [-k, r]],   // (0,r): tangent horizontal
            [[-r, k], [-r, -k]], // (-r,0)
            [[-k, -r], [k, -r]], // (0,-r)
        ];
        let (op, oh) = offset_curve(&pts, &handles, 10.0, true);
        let radius = |p: [f32; 2]| (p[0] * p[0] + p[1] * p[1]).sqrt();
        let r0 = radius(op[0]);
        for p in &op {
            assert!(
                (radius(*p) - r0).abs() < 1e-3,
                "all anchors equidistant: {p:?}"
            );
        }
        assert!(
            (r0 - 30.0).abs() < 1e-3,
            "offset outward to radius 30: {r0}"
        );
        let hlen =
            |a: [f32; 2], h: [f32; 2]| ((h[0] - a[0]).powi(2) + (h[1] - a[1]).powi(2)).sqrt();
        let want = k * r0 / r; // scaled by the radius ratio
        assert!(
            (hlen(op[0], oh[0][1]) - want).abs() < 0.05,
            "handle scaled by the radius ratio: {} vs {want}",
            hlen(op[0], oh[0][1])
        );
    }

    #[test]
    fn a_sharp_corner_stays_sharp_under_offset() {
        // A collapsed (sharp) handle has a zero vector, so the recompute leaves it collapsed on the offset
        // anchor — a Polygon corner offsets to a corner, not a rounded blob.
        let pts = vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0]];
        let handles = vec![[[0.0, 0.0]; 2], [[20.0, 0.0]; 2], [[20.0, 20.0]; 2]];
        let (op, oh) = offset_curve(&pts, &handles, 5.0, false);
        for i in 0..op.len() {
            assert_eq!(oh[i][0], op[i], "in handle stays collapsed at {i}");
            assert_eq!(oh[i][1], op[i], "out handle stays collapsed at {i}");
        }
    }

    #[test]
    fn insert_on_a_closed_loop_picks_the_seam_segment_for_a_click_near_it() {
        // Regression (Enio 2026-06-28): inserting on a CLOSED curve must consider the closing seam
        // (last → first). A square loop `(0,0)→(20,0)→(20,20)→(0,20)→back`; the seam (segment 3→0) is the
        // LEFT edge (x=0). A click just outside it must splice AFTER index 3 (`index == 4`, an append) and
        // land the new anchor ON that edge — the old straight-control scan ignored the seam entirely.
        let pts = vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]];
        let handles: Vec<[[f32; 2]; 2]> = pts.iter().map(|&p| [p, p]).collect(); // sharp corners
        let ins = curve_insert(&pts, &handles, true, [-1.0, 10.0]); // near the seam (left edge, x=0)
        assert_eq!(ins.index, 4, "the seam segment 3→0 → append at the end");
        assert_eq!(ins.prev_idx, 3);
        assert_eq!(ins.next_idx, 0);
        assert!(
            (ins.anchor[0]).abs() < 1e-3 && (ins.anchor[1] - 10.0).abs() < 1.0,
            "the new anchor sits on the seam edge: {:?}",
            ins.anchor
        );
    }

    #[test]
    fn insert_splits_an_arc_without_deforming_it() {
        // A de Casteljau split lands the anchor ON the cubic and the two sub-arcs reproduce it — so the
        // new anchor lies on the curve at the nearest parameter (a quarter-circle arc; click near its mid).
        let r = 20.0;
        let k = 0.55228 * r;
        // Quarter arc from (r,0) to (0,r): out of P0 = (r, k), in of P1 = (k, r).
        let pts = vec![[r, 0.0], [0.0, r]];
        let handles = vec![[[r, 0.0], [r, k]], [[k, r], [0.0, r]]];
        let mid = [r * 0.7071, r * 0.7071]; // ~45° point on the unit circle × r
        let ins = curve_insert(&pts, &handles, false, mid);
        assert_eq!(ins.index, 1);
        // The split anchor is within ~1px of the true arc midpoint (radius preserved).
        let radius = (ins.anchor[0] * ins.anchor[0] + ins.anchor[1] * ins.anchor[1]).sqrt();
        assert!(
            (radius - r).abs() < 0.5,
            "split anchor stays on the arc: r={radius}"
        );
    }
}
