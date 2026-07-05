//! Pure geometry helpers for the Curve / Free Hand editor — flattening (incl. the closed-loop seam),
//! point hit-test / nearest / insert-index, and the small distance primitives. Transcendental-free
//! (projection + `sqrt` only, HR-5). Split from [`super::curve`] for the workspace LOC cap; these are
//! free fns with no `CurveEditor` access, so the editor calls them as `curve_geom::*`. The perpendicular
//! **offset** lives in the sibling [`super::curve_offset`] (it reuses [`cubic_at`] + [`split_cubic`] here).

use ph2d_painter_brush::{flatten_bezier, flatten_catmull_rom};

/// A curve's control geometry: anchor `points` + their parallel `[in, out]` Bézier `handles`. Aliased to
/// keep [`simplify_curve`]'s signature under clippy's type-complexity lint.
pub(super) type ControlGeometry = (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>);

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

/// Simplify a curve to a clean minimal control polygon via the **Free Hand fit** (Schneider): flatten it to
/// a dense spine, then re-fit cubic Béziers within `max_error` px. Apply & Keep uses this to collapse the
/// offset reconstruction's dense anchors back to a faithful, editable handful (the same algorithm a Free
/// Hand stroke is simplified with). `None` — caller keeps the current geometry — when the curve is too
/// short, the fit is degenerate, or it would exceed `max_points`. A CLOSED loop has its duplicated seam
/// anchor merged (drop the end, graft its incoming handle onto the start) so no doubled dot remains.
pub(super) fn simplify_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    closed: bool,
    max_error: f32,
    max_points: usize,
) -> Option<ControlGeometry> {
    if points.len() < 3 {
        return None;
    }
    let mut spine = Vec::new();
    flatten_spine(points, handles, closed, &mut spine);
    if spine.len() < 3 {
        return None;
    }
    let fit = ph2d_painter_brush::fit_curve(&spine, max_error);
    let (mut anchors, mut h) = (fit.anchors, fit.handles);
    if anchors.len() < 2 || anchors.len() > max_points || h.len() != anchors.len() {
        return None;
    }
    if closed && anchors.len() >= 3 {
        let n = anchors.len();
        if dist2(anchors[0], anchors[n - 1]) < 1.0 {
            let in_h = h[n - 1][0]; // the seam's incoming tangent
            anchors.pop();
            h.pop();
            h[0][0] = in_h; // graft it onto the start anchor so the seam keeps its curve
        }
    }
    Some((anchors, h))
}

/// A corner keeps sharp (zero-length) handles when the turn between its two edges is at least this steep:
/// `cos(angle) ≤ 0.5` ⇒ ≥ 60°. Below it the anchor is smooth (Catmull-Rom tangent). Transcendental-free —
/// the test is a dot product against this cosine.
const SIMPLIFY_CORNER_COS: f32 = 0.5;

/// **Closed-curve Simplify** (Enio 2026-07-05): reduce a CLOSED editable curve to precise but few anchors,
/// smooth where the outline is smooth and SHARP where it corners — the Schneider [`simplify_curve`] cannot do
/// this (`fit_curve` degenerates on a loop whose start == end). Flatten to a dense spine, Douglas–Peucker the
/// closed ring to `tol` px (accurate + capped at `max_points`), then assign each surviving anchor a Catmull-
/// Rom tangent, collapsing to a hard corner when the local turn is ≥ 60° (a rectangle stays a rectangle; an
/// organic lasso reads as smooth as a Free-Hand fit). `None` (caller keeps the curve) when it is too short.
pub(super) fn simplify_closed_smooth(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    tol: f32,
    max_points: usize,
) -> Option<ControlGeometry> {
    if points.len() < 3 {
        return None;
    }
    let mut spine = Vec::new();
    flatten_spine(points, handles, true, &mut spine);
    if spine.len() >= 2 && dist2(spine[0], spine[spine.len() - 1]) < 1e-6 {
        spine.pop(); // drop the closing seam duplicate so DP sees a clean ring
    }
    if spine.len() < 4 {
        return None;
    }
    let anchors = super::selection_trace::simplify_closed(&spine, tol, max_points);
    let n = anchors.len();
    if n < 3 {
        return None;
    }
    let mut h = Vec::with_capacity(n);
    for i in 0..n {
        let prev = anchors[(i + n - 1) % n];
        let cur = anchors[i];
        let next = anchors[(i + 1) % n];
        let din = norm(sub(cur, prev)); // incoming edge direction
        let dout = norm(sub(next, cur)); // outgoing edge direction
        // Sharp turn ⇒ keep a hard corner (zero-length handles); smooth ⇒ Catmull-Rom tangent (1/3 of the
        // adjacent chord each side), giving a clean editable Bézier that flattens back onto the outline.
        if dot(din, dout) <= SIMPLIFY_CORNER_COS {
            h.push([cur, cur]);
        } else {
            let t = norm(sub(next, prev));
            let lin = dist2(cur, prev).sqrt() / 3.0;
            let lout = dist2(cur, next).sqrt() / 3.0;
            h.push([
                [cur[0] - t[0] * lin, cur[1] - t[1] * lin],
                [cur[0] + t[0] * lout, cur[1] + t[1] * lout],
            ]);
        }
    }
    Some((anchors, h))
}

/// `a − b`.
fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
/// Dot product.
fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
/// Unit vector, or zero when `a` is ~zero-length (a collapsed edge contributes no tangent).
fn norm(a: [f32; 2]) -> [f32; 2] {
    let m = (a[0] * a[0] + a[1] * a[1]).sqrt();
    if m > 1e-6 { [a[0] / m, a[1] / m] } else { [0.0, 0.0] }
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
/// EVERY segment — including the closing seam when `closed` (the converted Ellipse / Polygon bug: the old
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

/// Evaluate the cubic Bézier `b` at `t` (Bernstein form; transcendental-free). Shared with the offset.
pub(super) fn cubic_at(b: &[[f32; 2]; 4], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    [
        b[0][0] * w[0] + b[1][0] * w[1] + b[2][0] * w[2] + b[3][0] * w[3],
        b[0][1] * w[0] + b[1][1] * w[1] + b[2][1] * w[2] + b[3][1] * w[3],
    ]
}

/// **de Casteljau** split of cubic `b` at `t` → `[prev_out, new_in, anchor, new_out, next_in]`: the two
/// sub-cubics `[P0, q0, r0, s]` + `[s, r1, q2, P3]` reproduce `b` exactly, so subdividing never deforms it.
/// Shared with the offset reconstruction.
pub(super) fn split_cubic(b: &[[f32; 2]; 4], t: f32) -> [[f32; 2]; 5] {
    let lerp = |a: [f32; 2], c: [f32; 2]| [a[0] + (c[0] - a[0]) * t, a[1] + (c[1] - a[1]) * t];
    let q0 = lerp(b[0], b[1]);
    let q1 = lerp(b[1], b[2]);
    let q2 = lerp(b[2], b[3]);
    let r0 = lerp(q0, q1);
    let r1 = lerp(q1, q2);
    let s = lerp(r0, r1);
    [q0, r0, s, r1, q2]
}

/// Subdivide a CLOSED Bézier curve's segments (repeated [`split_cubic`] — the shape is reproduced
/// EXACTLY, only more anchors appear) until every anchor span is ≤ `target_px` of estimated arc length,
/// capped at `max_points` anchors total. Convert-to-Curve wants extreme precision + MANY manipulation
/// points (Enio 2026-07-05) — the opposite direction of Simplify, which re-fits sparse. Handles stay
/// faithful: each split re-derives the exact sub-segment tangents, so straight (degenerate-handle) edges
/// densify into collinear anchors and arcs stay arcs.
pub(super) fn densify_closed_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    target_px: f32,
    max_points: usize,
) -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    let n = points.len();
    if n < 2 || handles.len() != n || target_px <= 0.0 {
        return (points.to_vec(), handles.to_vec());
    }
    // Per-segment split count from the standard length estimate `(chord + control polygon) / 2`.
    let seg = |i: usize| -> [[f32; 2]; 4] {
        let j = (i + 1) % n;
        [points[i], handles[i][1], handles[j][0], points[j]]
    };
    let dist = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    let mut counts: Vec<usize> = (0..n)
        .map(|i| {
            let b = seg(i);
            let chord = dist(b[0], b[3]);
            let poly = dist(b[0], b[1]) + dist(b[1], b[2]) + dist(b[2], b[3]);
            let len = (chord + poly) * 0.5;
            ((len / target_px).ceil() as usize).max(1)
        })
        .collect();
    // Cap the total anchor count: scale the EXTRA anchors (Σ(k−1)) down proportionally.
    let extra: usize = counts.iter().map(|k| k - 1).sum();
    let budget = max_points.saturating_sub(n);
    if extra > budget && extra > 0 {
        for k in counts.iter_mut() {
            *k = 1 + (*k - 1) * budget / extra;
        }
    }
    // Split every segment into its `k` pieces (each split takes an equal-parameter slice off the front),
    // collecting the adjusted endpoint tangents + the interior anchors.
    struct SegOut {
        p0_out: [f32; 2],
        interior: Vec<([f32; 2], [f32; 2], [f32; 2])>, // (in, anchor, out)
        p3_in: [f32; 2],
    }
    let segs: Vec<SegOut> = (0..n)
        .map(|i| {
            let mut b = seg(i);
            let k = counts[i];
            let mut out = SegOut {
                p0_out: b[1],
                interior: Vec::with_capacity(k.saturating_sub(1)),
                p3_in: b[2],
            };
            for s in 0..k.saturating_sub(1) {
                let t = 1.0 / ((k - s) as f32);
                let [q0, r0, m, r1, q2] = split_cubic(&b, t);
                // The split shortens the CURRENT left endpoint's out-handle (exactly).
                if s == 0 {
                    out.p0_out = q0;
                } else if let Some(last) = out.interior.last_mut() {
                    last.2 = q0;
                }
                out.interior.push((r0, m, r1));
                b = [m, r1, q2, b[3]];
            }
            out.p3_in = b[2];
            out
        })
        .collect();
    let total = n + segs.iter().map(|s| s.interior.len()).sum::<usize>();
    let mut new_pts = Vec::with_capacity(total);
    let mut new_handles = Vec::with_capacity(total);
    for i in 0..n {
        let prev = (i + n - 1) % n;
        new_pts.push(points[i]);
        new_handles.push([segs[prev].p3_in, segs[i].p0_out]);
        for &(hin, m, hout) in &segs[i].interior {
            new_pts.push(m);
            new_handles.push([hin, hout]);
        }
    }
    (new_pts, new_handles)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let d = std::f32::consts::FRAC_1_SQRT_2;
        let mid = [r * d, r * d]; // ~45° point on the unit circle × r
        let ins = curve_insert(&pts, &handles, false, mid);
        assert_eq!(ins.index, 1);
        // The split anchor is within ~1px of the true arc midpoint (radius preserved).
        let radius = (ins.anchor[0] * ins.anchor[0] + ins.anchor[1] * ins.anchor[1]).sqrt();
        assert!(
            (radius - r).abs() < 0.5,
            "split anchor stays on the arc: r={radius}"
        );
    }

    #[test]
    fn densify_closed_curve_adds_anchors_without_deforming() {
        // An exact 4-arc Bézier circle densified to ~16 px spacing: many more anchors, EVERY one still
        // on the circle (de Casteljau splits reproduce the curve exactly), and the cap is honoured.
        const K: f32 = 0.552_285;
        let r = 40.0f32;
        let pts = vec![[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
        let handles = vec![
            [[r, -K * r], [r, K * r]],
            [[K * r, r], [-K * r, r]],
            [[-r, K * r], [-r, -K * r]],
            [[-K * r, -r], [K * r, -r]],
        ];
        let (p, h) = densify_closed_curve(&pts, &handles, 16.0, 512);
        assert!(p.len() >= 12, "densified to many anchors: {}", p.len());
        assert_eq!(p.len(), h.len(), "handles stay parallel");
        for a in &p {
            let d = (a[0] * a[0] + a[1] * a[1]).sqrt();
            assert!((d - r).abs() < 0.05, "anchor stays ON the circle: {d}");
        }
        // The cap clamps the anchor count.
        let (pc, _) = densify_closed_curve(&pts, &handles, 0.5, 24);
        assert!(pc.len() <= 24, "cap honoured: {}", pc.len());
    }
}
