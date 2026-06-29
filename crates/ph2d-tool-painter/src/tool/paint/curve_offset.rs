//! Perpendicular **offset** (parallel curve) of the Curve / Free Hand control geometry — the Offset slider.
//! [`offset_curve`] moves the control points (exact for lines + circles, Tiller–Hanson); [`offset_curve_refined`]
//! first *reconstructs* the curve with extra anchors wherever that control-point offset would stray from the
//! true parallel curve (adaptive subdivision to sub-pixel tolerance — CAD-grade, cf. Levien 2022), staying an
//! ordinary editable anchor/handle curve. Transcendental-free (rotation = complex multiply from chord unit
//! vectors; normals `sqrt`-normalised — HR-5). Free fns, called as `curve_offset::*`.

use super::curve_geom::{cubic_at, dist2, split_cubic};
use super::curve_handle::HandleKind;

/// Adaptive-subdivision tolerance: max px the densified offset may stray before a segment is split (sub-pixel).
const OFFSET_TOL_PX: f32 = 0.3;
/// Depth cap on the offset subdivision (≤ 2^6 leaves per input segment) — bounds the work near a cusp.
const MAX_OFFSET_SUBDIV: u32 = 6;
/// Miter limit: cap a sharp corner's offset displacement to this multiple of `d` so an acute spike truncates
/// rather than shooting to infinity (cf. SVG `stroke-miterlimit` = 4, Clipper `MiterLimit`).
const MITER_LIMIT: f32 = 4.0;

/// Offset `(points, handles)` + per output anchor its ORIGINAL index (`None` = inserted). Aliased for clippy.
type RefinedOffset = (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>, Vec<Option<usize>>);
/// [`offset_curve_refined_kinds`] output: dense `(points, handles)` + carried kinds + remapped selection.
type RefinedKinds = (
    Vec<[f32; 2]>,
    Vec<[[f32; 2]; 2]>,
    Vec<HandleKind>,
    Option<usize>,
);

/// [`offset_curve`] preceded by [`densify_for_offset`]: the offset hugs the true parallel curve even through
/// tight, varying-curvature bends (Tiller–Hanson alone is exact only for lines + circles; subdivision is the
/// fix), staying an editable anchor/handle curve. The `Vec<Option<usize>>` maps each output anchor to its
/// ORIGINAL (`None` = inserted), so a bake carries kinds + selection. `d == 0` ⇒ unchanged clones + identity.
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

/// [`offset_curve_refined`] that also carries the handle KINDS + the SELECTION across the reconstruction:
/// each output anchor takes its ORIGINAL anchor's kind (inserted anchors → `Free`), and `selected` remaps to
/// its dense index. For materialising the densified offset into the editable curve (the bake).
pub(super) fn offset_curve_refined_kinds(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    kinds: &[HandleKind],
    selected: Option<usize>,
    d: f32,
    closed: bool,
) -> RefinedKinds {
    let (p, h, origin) = offset_curve_refined(points, handles, d, closed);
    let k = origin
        .iter()
        .map(|o| {
            o.and_then(|i| kinds.get(i).copied())
                .unwrap_or(HandleKind::Free)
        })
        .collect();
    let sel = selected.and_then(|s| origin.iter().position(|o| *o == Some(s)));
    (p, h, k, sel)
}

/// Offset the curve's **control geometry** perpendicular by `d` px: each anchor shifts along its averaged
/// right-normal, then each handle is rotated + scaled to follow its moved segment (a circle stays concentric;
/// a collapsed handle stays collapsed → sharp corner). Spine/dots/tangents/paint all derive from this, moving
/// TOGETHER. `closed` wraps the endpoints; `d == 0` (or a length mismatch) ⇒ unchanged clones.
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
    // 1. Offset the anchors along their tangent-based vertex normals (perpendicular to the CURVE → even).
    let op: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let nrm = vertex_normal(points, handles, i, closed);
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

/// Recursively split leaf `b` (segment `seg`) at its midpoint until [`cubic_offset_error`] ≤ tol / depth cap.
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

/// Px error of offsetting one cubic by `d` with the control-polygon rule (endpoint normals + [`SegXform`], as
/// [`offset_curve`] does per leaf): max deviation from the TRUE offset `C(t) + d·N(t)`. ≈ 0 for a line / arc
/// (exact there); grows with `|d|·curvature` on a varying-curvature bend.
fn cubic_offset_error(b: &[[f32; 2]; 4], d: f32) -> f32 {
    let (Some(n0), Some(n3)) = (endpoint_normal(b, true), endpoint_normal(b, false)) else {
        return 0.0; // a zero-length segment (P0 == P1 == P2 == P3) — nothing to offset
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

/// Unit **right-normal** `(dy, −dx)/|v|` of direction `v` (matches [`vertex_normal`]); `None` if ≈ zero-length.
fn unit_right_normal(v: [f32; 2]) -> Option<[f32; 2]> {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (len > 1e-6).then(|| [v[1] / len, -v[0] / len])
}

/// Right-normal at a cubic ENDPOINT (`start` = t≈0 else t≈1), robust to a **collapsed handle**: a converted
/// **Polygon** `Free` corner zeros the analytic tangent when curved, so fall back to the first non-coincident
/// control point (the real cusp direction) — else the segment skips densification + self-crosses (Enio 2026-06-28).
fn endpoint_normal(b: &[[f32; 2]; 4], start: bool) -> Option<[f32; 2]> {
    let cands = if start {
        [[b[1], b[0]], [b[2], b[0]], [b[3], b[0]]]
    } else {
        [[b[3], b[2]], [b[3], b[1]], [b[3], b[0]]]
    };
    cands
        .into_iter()
        .find_map(|[p, q]| unit_right_normal([p[0] - q[0], p[1] - q[1]]))
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

/// The rotate-and-scale mapping a base segment's chord to its offset chord — applied to a tangent handle so it
/// follows the offset (a circle's handles scale by the radius ratio). Identity for a degenerate base segment.
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

    /// Re-place absolute `handle` at its offset anchor: rotate off the old anchor by the segment turn + scale.
    fn apply(&self, anchor: [f32; 2], handle: [f32; 2], offset_anchor: [f32; 2]) -> [f32; 2] {
        let v = [handle[0] - anchor[0], handle[1] - anchor[1]];
        let rx = (v[0] * self.cos - v[1] * self.sin) * self.scale;
        let ry = (v[0] * self.sin + v[1] * self.cos) * self.scale;
        [offset_anchor[0] + rx, offset_anchor[1] + ry]
    }
}

/// **Miter** offset displacement at anchor `i` (per unit `d`): the two side-normals (arriving + leaving edge,
/// each ⊥ the Bézier **tangent**, chord fallback for a collapsed handle) combined as the INTERSECTION of the
/// two offset edges — so a CORNER lands on the true parallel-curve miter `d / cos δ` out, not the averaged-
/// normal undershoot `d` (the "quinas mais curtas que as curvas" bug — Enio 2026-06-28). A SMOOTH vertex
/// (n_in ≈ n_out) yields a unit vector ⇒ circles / Auto / Vector stay byte-identical; an open endpoint has one
/// side ⇒ its plain unit normal. A positive offset outsets a CCW loop; zero for a degenerate vertex.
fn vertex_normal(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    i: usize,
    closed: bool,
) -> [f32; 2] {
    let n = points.len();
    let bez = handles.len() == n;
    let sub = |a: [f32; 2], b: [f32; 2]| [a[0] - b[0], a[1] - b[1]];
    let prev = (i > 0).then(|| i - 1).or_else(|| closed.then(|| n - 1));
    let next = (i + 1 < n).then(|| i + 1).or_else(|| closed.then_some(0));
    let tan_in = bez.then(|| sub(points[i], handles[i][0])); // travel arriving at i
    let tan_out = bez.then(|| sub(handles[i][1], points[i])); // travel leaving i
    // The (up to two) unit side-normals: one per incident edge, ⊥ its tangent (chord fallback).
    let mut sides = [[0.0f32; 2]; 2];
    let mut count = 0usize;
    let mut push = |tan: Option<[f32; 2]>, chord: [f32; 2]| {
        let t = tan
            .filter(|v| v[0] * v[0] + v[1] * v[1] > 1e-6)
            .unwrap_or(chord);
        if let Some(nm) = unit_right_normal(t) {
            sides[count] = nm;
            count += 1;
        }
    };
    if let Some(p) = prev {
        push(tan_in, sub(points[i], points[p]));
    }
    if let Some(x) = next {
        push(tan_out, sub(points[x], points[i]));
    }
    match count {
        0 => [0.0, 0.0],
        1 => sides[0],
        _ => miter(sides[0], sides[1]),
    }
}

/// Combine two unit side-normals into the offset displacement: `(n1+n2)/(1+n1·n2)`, the intersection of the
/// two offset edges = the bisector at distance `1/cos δ` (the true miter). Smooth (n1 ≈ n2) ⇒ unit ⇒ no
/// change; clamped to [`MITER_LIMIT`] so an acute spike (near-antiparallel normals → miter → ∞) truncates
/// along the bisector instead of shooting out. Transcendental-free.
fn miter(n1: [f32; 2], n2: [f32; 2]) -> [f32; 2] {
    let s = [n1[0] + n2[0], n1[1] + n2[1]];
    let sl = (s[0] * s[0] + s[1] * s[1]).sqrt();
    if sl < 1e-6 {
        return [0.0, 0.0]; // antiparallel cusp: no well-defined offset direction
    }
    let denom = 1.0 + n1[0] * n2[0] + n1[1] * n2[1];
    let mag = (sl / denom.max(1e-6)).min(MITER_LIMIT); // |miter| = 1/cos δ; cap the spike (denom→0 ⇒ ∞)
    [s[0] / sl * mag, s[1] / sl * mag]
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
    fn vertex_normal_follows_the_handle_tangent_not_the_chord() {
        // An anchor whose handles define a HORIZONTAL tangent must offset VERTICALLY (⊥ the curve), even when
        // its chords to the neighbours point elsewhere — the even-offset property a converted Polygon lacked.
        let pts = vec![[0.0, 0.0], [10.0, 10.0], [40.0, 30.0]];
        let handles = vec![
            [[0.0, 0.0], [3.0, 0.0]],
            [[5.0, 10.0], [15.0, 10.0]], // middle anchor's tangent is horizontal (in→out along +x)
            [[40.0, 30.0], [40.0, 30.0]],
        ];
        let nrm = vertex_normal(&pts, &handles, 1, false);
        assert!(
            nrm[0].abs() < 0.05,
            "normal is vertical (⊥ the horizontal tangent): {nrm:?}"
        );
    }

    #[test]
    fn a_collapsed_endpoint_handle_curve_still_densifies() {
        // A converted-Polygon corner stays `Free` with a COLLAPSED handle; curving the adjacent segment zeros
        // the cubic's t=0 tangent. It must STILL densify — it was skipped as degenerate, so the curved segment
        // fell back to the old control-point offset and self-crossed (Enio 2026-06-28).
        let b = [[0.0, 0.0], [0.0, 0.0], [10.0, 80.0], [120.0, 0.0]]; // collapsed start handle, real curvature
        assert!(
            cubic_offset_error(&b, 12.0) > OFFSET_TOL_PX,
            "real offset error despite the zero analytic start tangent"
        );
        let pts = vec![[0.0, 0.0], [120.0, 0.0]];
        let handles = vec![[[0.0, 0.0], [0.0, 0.0]], [[10.0, 80.0], [120.0, 0.0]]];
        let (dp, _, _) = densify_for_offset(&pts, &handles, false, 12.0);
        assert!(
            dp.len() > 2,
            "the curved degenerate-endpoint segment densified: {}",
            dp.len()
        );
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
    fn a_corner_offsets_to_the_true_miter_distance_not_the_undershot_average() {
        // A right-angle corner (collapsed handles → sharp): the offset anchor must reach the true parallel-
        // curve miter at d/cos45° = d·√2 along the bisector, NOT the averaged-normal undershoot d. This is the
        // fix for "quinas mais curtas que as curvas" — corners are no longer pulled in (Enio 2026-06-28).
        let pts = vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0]];
        let handles = vec![[[0.0, 0.0]; 2], [[20.0, 0.0]; 2], [[20.0, 20.0]; 2]];
        let m = vertex_normal(&pts, &handles, 1, false);
        assert!(
            (m[0] - 1.0).abs() < 1e-3 && (m[1] + 1.0).abs() < 1e-3,
            "miter vector (1,-1): {m:?}"
        );
        let len = (m[0] * m[0] + m[1] * m[1]).sqrt();
        assert!(
            (len - std::f32::consts::SQRT_2).abs() < 1e-3,
            "miter length = 1/cos45° = √2: {len}"
        );
    }

    #[test]
    fn a_smooth_vertex_miter_stays_unit_so_circles_do_not_change() {
        // n_in ≈ n_out (a smooth, continuous-tangent vertex) ⇒ the miter collapses to the plain unit normal,
        // guaranteeing circles / Auto / Vector offsets stay byte-identical to before the corner fix.
        let n = [0.6f32, 0.8];
        let m = miter(n, n);
        assert!(
            (m[0] - n[0]).abs() < 1e-4 && (m[1] - n[1]).abs() < 1e-4,
            "smooth miter is the unit normal: {m:?}"
        );
    }

    #[test]
    fn an_acute_spike_is_clamped_to_the_miter_limit() {
        // Near-antiparallel side-normals (a very sharp corner) would miter toward infinity; the limit truncates
        // the displacement along the bisector so the offset never shoots out into a spike.
        let n1 = [1.0f32, 0.0];
        let h = (1.0f32 - 0.99 * 0.99).sqrt();
        let n2 = [-0.99f32, h]; // ~172° from n1, unit length
        let m = miter(n1, n2);
        let len = (m[0] * m[0] + m[1] * m[1]).sqrt();
        assert!(len <= MITER_LIMIT + 1e-3, "clamped to the miter limit: {len}");
    }
}
