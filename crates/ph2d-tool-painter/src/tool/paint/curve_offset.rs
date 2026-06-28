//! Perpendicular **offset** (parallel curve) of the Curve / Free Hand control geometry — the Offset slider.
//! Two layers: [`offset_curve`] moves the control points (exact for lines + circles, the Tiller–Hanson
//! family), and [`offset_curve_refined`] first *reconstructs* the curve with extra anchors wherever that
//! control-point offset would stray from the true parallel curve (adaptive subdivision to a sub-pixel
//! tolerance — the CAD-grade approach, cf. Levien 2022), so a tight, varying-curvature bend offsets
//! faithfully while the result stays an ordinary editable anchor/handle curve. Transcendental-free (the
//! segment rotation is a complex multiply from chord unit vectors; normals are `sqrt`-normalised — HR-5).
//! Split from [`super::curve_geom`] for the workspace LOC cap; free fns, called as `curve_offset::*`.

use super::curve_geom::{cubic_at, dist2, split_cubic};

/// Adaptive-subdivision tolerance for the offset reconstruction: the max px the densified offset may stray
/// from the true parallel curve before a segment is split. Sub-pixel ⇒ a visually perfect offset.
const OFFSET_TOL_PX: f32 = 0.3;
/// Depth cap on the offset subdivision (≤ 2^6 leaves per input segment) — bounds the work near a cusp
/// (where `|d|·curvature → 1` and no single cubic can follow the offset; we densify, we don't trim).
const MAX_OFFSET_SUBDIV: u32 = 6;

/// The reconstruction's output: offset `(points, handles)` plus, per output anchor, the ORIGINAL anchor
/// index it came from (`None` = inserted by the subdivision). Aliased for clippy's type-complexity lint.
type RefinedOffset = (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>, Vec<Option<usize>>);

/// [`offset_curve`] preceded by [`densify_for_offset`]: the offset hugs the true parallel curve even through
/// tight, varying-curvature bends — the CAD-grade result — while staying an ordinary editable anchor/handle
/// curve (Tiller–Hanson control-point offsetting alone is exact only for lines + circles; subdivision to a
/// tolerance is the fix). The returned `Vec<Option<usize>>` maps each output anchor to the ORIGINAL anchor
/// it came from (`None` = inserted), so a bake carries the handle kinds + selection across the
/// reconstruction. `d == 0` ⇒ unchanged clones + identity map.
pub(super) fn offset_curve_refined(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    d: f32,
    closed: bool,
) -> RefinedOffset {
    let (dp, dh, origin) = densify_for_offset(points, handles, closed, d);
    let (op, oh) = offset_curve(&dp, &dh, d, closed);
    (op, oh, origin)
}

/// **Trim** a flattened offset spine's self-intersections (the Offset card's Trim checkbox): where the path
/// crosses itself — an offset whose distance exceeds the local radius of curvature folds the concave side
/// into a loop — insert ONE point at the crossing and drop the looped excess between the two crossing
/// segments. Repeats until no crossing remains (multiple loops). A strict-interior test means shared
/// endpoints / adjacent segments never trigger. Transcendental-free (cross products + one divide). `poly`
/// is the dab/guide polyline (open, or closed-as-polyline); a no-op when nothing crosses.
pub(super) fn trim_self_intersections(poly: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut pts = poly.to_vec();
    loop {
        let mut hit = None;
        'scan: for i in 0..pts.len().saturating_sub(1) {
            for j in (i + 2)..pts.len().saturating_sub(1) {
                if let Some(x) = seg_cross(pts[i], pts[i + 1], pts[j], pts[j + 1]) {
                    hit = Some((i, j, x));
                    break 'scan;
                }
            }
        }
        let Some((i, j, x)) = hit else { break };
        // Keep `0..=i`, drop the loop `i+1..=j`, splice the crossing point in, keep `j+1..`.
        let mut next = pts[..=i].to_vec();
        next.push(x);
        next.extend_from_slice(&pts[j + 1..]);
        pts = next;
    }
    pts
}

/// Intersection point of segments `a0→a1` and `b0→b1` when they cross in BOTH interiors (strict, so a
/// shared endpoint isn't a crossing), else `None`. Parametric: `a0 + t·(a1−a0)`, `t,u ∈ (ε, 1−ε)`.
fn seg_cross(a0: [f32; 2], a1: [f32; 2], b0: [f32; 2], b1: [f32; 2]) -> Option<[f32; 2]> {
    let r = [a1[0] - a0[0], a1[1] - a0[1]];
    let s = [b1[0] - b0[0], b1[1] - b0[1]];
    let denom = r[0] * s[1] - r[1] * s[0];
    if denom.abs() < 1e-6 {
        return None; // parallel / degenerate
    }
    let d = [b0[0] - a0[0], b0[1] - a0[1]];
    let t = (d[0] * s[1] - d[1] * s[0]) / denom;
    let u = (d[0] * r[1] - d[1] * r[0]) / denom;
    const E: f32 = 1e-4;
    (t > E && t < 1.0 - E && u > E && u < 1.0 - E).then(|| [a0[0] + r[0] * t, a0[1] + r[1] * t])
}

/// Offset the curve's **control geometry** perpendicular by `d` px. Each anchor shifts along its averaged
/// right-normal, then each tangent handle is RECALCULATED — rotated + scaled to follow its (now moved)
/// segment — rather than rigidly translated, so the offset stays faithful: a circle offsets to a clean
/// concentric circle (handles scale by the radius ratio), sharp corners stay sharp (a collapsed handle has a
/// zero vector → stays collapsed), and there's no control-level deformation. The spine, dots, tangents +
/// paint all derive from this, so they move TOGETHER. `closed` wraps the endpoints. `d == 0` (or a length
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

/// Reconstruct the curve with extra anchors wherever the control-polygon offset would stray from the true
/// parallel curve by more than [`OFFSET_TOL_PX`] (adaptive de Casteljau split at the midpoint). The SHAPE is
/// unchanged — the sub-cubics sum to the original arcs — only the resolution rises where curvature needs it,
/// so the subsequent [`offset_curve`] follows the true offset. Exact cases (straight legs, the cubic circle
/// approximation) split ZERO ⇒ byte-identical to the un-densified offset. Returns dense points + handles +
/// the per-anchor origin map (`None` = inserted).
fn densify_for_offset(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    closed: bool,
    d: f32,
) -> RefinedOffset {
    let n = points.len();
    let bezier = handles.len() == n && n >= 2;
    if !bezier || d == 0.0 {
        return (
            points.to_vec(),
            handles.to_vec(),
            (0..n).map(Some).collect(),
        );
    }
    let seg_count = if closed { n } else { n - 1 };
    let mut leaves: Vec<([[f32; 2]; 4], usize)> = Vec::new();
    for i in 0..seg_count {
        let j = (i + 1) % n;
        let cubic = [points[i], handles[i][1], handles[j][0], points[j]];
        subdivide_for_offset(cubic, i, 0, d, &mut leaves);
    }
    // Re-thread the leaf cubics into anchors: each anchor's OUT handle is its leaf's P1, its IN handle the
    // previous leaf's P2; an anchor is ORIGINAL when it sits on a boundary between two different input
    // segments (else it's an inserted split point).
    let m = leaves.len();
    let orig_of = |k: usize| -> Option<usize> {
        if k == 0 {
            Some(0)
        } else if leaves[k].1 != leaves[k - 1].1 {
            Some(leaves[k].1)
        } else {
            None
        }
    };
    let mut op = Vec::with_capacity(m + 1);
    let mut oh = Vec::with_capacity(m + 1);
    let mut origin = Vec::with_capacity(m + 1);
    for k in 0..m {
        let cub = leaves[k].0;
        let in_h = if k == 0 {
            if closed {
                leaves[m - 1].0[2]
            } else {
                handles[0][0]
            }
        } else {
            leaves[k - 1].0[2]
        };
        op.push(cub[0]);
        oh.push([in_h, cub[1]]);
        origin.push(orig_of(k));
    }
    if !closed {
        let last = leaves[m - 1].0;
        op.push(last[3]);
        oh.push([last[2], handles[n - 1][1]]);
        origin.push(Some(n - 1));
    }
    (op, oh, origin)
}

/// Recursively split `b` (a leaf of the original segment `seg`) at its midpoint until the control-polygon
/// offset error ([`cubic_offset_error`]) is within tolerance or the depth cap is hit, collecting the leaves.
fn subdivide_for_offset(
    b: [[f32; 2]; 4],
    seg: usize,
    depth: u32,
    d: f32,
    out: &mut Vec<([[f32; 2]; 4], usize)>,
) {
    if depth >= MAX_OFFSET_SUBDIV || cubic_offset_error(&b, d) <= OFFSET_TOL_PX {
        out.push((b, seg));
        return;
    }
    let [q0, r0, s, r1, q2] = split_cubic(&b, 0.5);
    subdivide_for_offset([b[0], q0, r0, s], seg, depth + 1, d, out);
    subdivide_for_offset([s, r1, q2, b[3]], seg, depth + 1, d, out);
}

/// Px error of offsetting one cubic by `d` with the control-polygon rule (endpoint normals + the segment
/// rotate/scale of [`SegXform`], exactly as [`offset_curve`] does at the leaf level): build that offset
/// cubic and measure its max deviation from the TRUE offset `C(t) + d·N(t)` over the span. ≈ 0 for a line
/// or a circular arc (the rule is exact there); grows with `|d|·curvature` on a varying-curvature bend.
fn cubic_offset_error(b: &[[f32; 2]; 4], d: f32) -> f32 {
    let (Some(n0), Some(n3)) = (
        unit_right_normal(cubic_tangent(b, 0.0)),
        unit_right_normal(cubic_tangent(b, 1.0)),
    ) else {
        return 0.0; // degenerate endpoint tangent — nothing meaningful to offset
    };
    let oa = [b[0][0] + n0[0] * d, b[0][1] + n0[1] * d];
    let ob = [b[3][0] + n3[0] * d, b[3][1] + n3[1] * d];
    let xf = SegXform::new(b[0], b[3], oa, ob);
    let cand = [oa, xf.apply(b[0], b[1], oa), xf.apply(b[3], b[2], ob), ob];
    let mut max_err = 0.0f32;
    for k in 1..8 {
        let t = k as f32 / 8.0;
        let Some(nt) = unit_right_normal(cubic_tangent(b, t)) else {
            continue;
        };
        let c = cubic_at(b, t);
        let truth = [c[0] + nt[0] * d, c[1] + nt[1] * d];
        max_err = max_err.max(dist2(truth, cubic_at(&cand, t)).sqrt());
    }
    max_err
}

/// Unit **right-normal** `(dy, −dx)/|v|` of a direction `v` (matches [`vertex_normal`]'s convention), or
/// `None` for a degenerate (≈ zero-length) direction.
fn unit_right_normal(v: [f32; 2]) -> Option<[f32; 2]> {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (len > 1e-6).then(|| [v[1] / len, -v[0] / len])
}

/// Tangent (first derivative) of cubic `b` at `t`: `B'(t) = 3[u²(P1−P0) + 2ut(P2−P1) + t²(P3−P2)]`.
fn cubic_tangent(b: &[[f32; 2]; 4], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    let (c0, c1, c2) = (3.0 * u * u, 6.0 * u * t, 3.0 * t * t);
    [
        c0 * (b[1][0] - b[0][0]) + c1 * (b[2][0] - b[1][0]) + c2 * (b[3][0] - b[2][0]),
        c0 * (b[1][1] - b[0][1]) + c1 * (b[2][1] - b[1][1]) + c2 * (b[3][1] - b[2][1]),
    ]
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
        // scale the handles by the radius ratio — proof the recompute follows the curve without deforming it.
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
    fn an_exact_arc_offsets_within_tolerance_without_subdivision() {
        // The cubic circle approximation offsets exactly (the control-point rule is exact for arcs), so a
        // quarter arc's offset error is sub-pixel ⇒ ZERO splits: an exact shape is never fragmented.
        let r = 20.0;
        let k = 0.55228 * r;
        let arc = [[r, 0.0], [r, k], [k, r], [0.0, r]]; // bézier quarter circle
        assert!(
            cubic_offset_error(&arc, 10.0) <= OFFSET_TOL_PX,
            "the arc offset is exact ⇒ no split: {}",
            cubic_offset_error(&arc, 10.0)
        );
    }

    #[test]
    fn a_straight_run_is_never_densified() {
        // Zero curvature ⇒ the offset is an exact translation ⇒ the reconstruction adds nothing.
        let pts = vec![[0.0, 0.0], [100.0, 0.0]];
        let handles = vec![[[0.0, 0.0], [33.0, 0.0]], [[66.0, 0.0], [100.0, 0.0]]];
        let (dp, _, origin) = densify_for_offset(&pts, &handles, false, 15.0);
        assert_eq!(dp.len(), 2, "a straight segment is not split");
        assert!(origin.iter().all(Option::is_some));
    }

    #[test]
    fn a_curvy_segment_is_reconstructed_under_tolerance() {
        // One cubic with strong, varying curvature: offsetting the WHOLE segment by the control-point rule
        // strays far from the true parallel curve, but the reconstruction subdivides until EVERY piece is
        // within tolerance — a provably faithful offset (the user's "place points where needed" intuition).
        let p0 = [0.0, 0.0];
        let p1 = [10.0, 80.0];
        let p2 = [110.0, -80.0];
        let p3 = [120.0, 0.0];
        let d = 12.0;
        assert!(
            cubic_offset_error(&[p0, p1, p2, p3], d) > 5.0,
            "the raw single-segment offset is far off: {}",
            cubic_offset_error(&[p0, p1, p2, p3], d)
        );
        let pts = vec![p0, p3];
        let handles = vec![[p0, p1], [p2, p3]]; // [in, out] per anchor
        let (dp, dh, origin) = densify_for_offset(&pts, &handles, false, d);
        assert!(
            dp.len() > 2,
            "reconstructed with inserted anchors: {}",
            dp.len()
        );
        assert!(
            origin.iter().any(Option::is_none),
            "some anchors are inserted split points"
        );
        for i in 0..dp.len() - 1 {
            let leaf = [dp[i], dh[i][1], dh[i + 1][0], dp[i + 1]];
            assert!(
                cubic_offset_error(&leaf, d) <= OFFSET_TOL_PX + 1e-3,
                "leaf {i} now within tolerance: {}",
                cubic_offset_error(&leaf, d)
            );
        }
    }

    #[test]
    fn simplify_collapses_a_reconstructed_offset_back_to_a_clean_curve() {
        // Apply & Keep: the offset reconstruction densifies a curvy segment to many anchors, then the Free
        // Hand fit must collapse it back to a faithful handful — fewer anchors, spine essentially unchanged.
        use super::super::curve_geom::{flatten_spine, simplify_curve};
        let pts = vec![[0.0, 0.0], [120.0, 0.0]];
        let handles = vec![[[0.0, 0.0], [10.0, 80.0]], [[110.0, -80.0], [120.0, 0.0]]];
        let (dp, dh, _) = offset_curve_refined(&pts, &handles, 12.0, false);
        assert!(dp.len() > 6, "the reconstruction densified: {}", dp.len());
        let (sp, sh) = simplify_curve(&dp, &dh, false, 4.0, 64).expect("simplifies");
        assert!(
            sp.len() < dp.len(),
            "fewer anchors: {} < {}",
            sp.len(),
            dp.len()
        );
        let (mut a, mut b) = (Vec::new(), Vec::new());
        flatten_spine(&dp, &dh, false, &mut a);
        flatten_spine(&sp, &sh, false, &mut b);
        for q in &b {
            let best = a
                .iter()
                .map(|w| dist2(*q, *w))
                .fold(f32::INFINITY, f32::min);
            assert!(
                best.sqrt() < 6.0,
                "simplified spine stays on the shape: {}",
                best.sqrt()
            );
        }
    }

    #[test]
    fn trim_cuts_a_self_intersecting_loop() {
        // A polyline that crosses itself: the trailing segment dives back across the leading one. Trim splices
        // the crossing point in and drops the looped excess between the two crossing segments.
        let poly = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [5.0, 10.0],
            [5.0, -5.0],
        ];
        let out = trim_self_intersections(&poly);
        assert_eq!(out.len(), 3, "loop removed: {out:?}");
        assert_eq!(out[0], [0.0, 0.0]);
        assert!(
            (out[1][0] - 5.0).abs() < 1e-3 && out[1][1].abs() < 1e-3,
            "crossing point (5,0) spliced: {:?}",
            out[1]
        );
        assert_eq!(out[2], [5.0, -5.0]);
    }

    #[test]
    fn trim_leaves_a_non_crossing_polyline_untouched() {
        let poly = vec![[0.0, 0.0], [10.0, 0.0], [20.0, 5.0], [30.0, 0.0]];
        assert_eq!(trim_self_intersections(&poly), poly);
    }
}
