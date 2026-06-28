//! Perpendicular **offset** (parallel curve) of the Curve / Free Hand control geometry — the Offset slider.
//! [`offset_polyline`] offsets the FLATTENED curve as a polyline by an EVEN distance (miter joins) — the
//! painted spine, immune to handle conditioning. [`offset_curve`] moves the control points (used for the
//! editable dots and the bake). Transcendental-free (`sqrt` normals; miter = cross-product line
//! intersection — HR-5). Free fns, called as `curve_offset::*`.

use super::curve_geom::dist2;

/// Offset an open/closed **POLYLINE** by `d` px (right-normal side) with **miter joins** — every output edge
/// sits exactly `d` from its input edge, so the offset distance is EVEN everywhere, IMMUNE to how the source
/// curve's handles are conditioned. (The control-point [`offset_curve`] bulges on edited/long handles and
/// undershoots sharp corners by `d·cos(δ)` — densifying control points can't fix that robustly; offsetting
/// the *flattened* curve does, because it never touches the control points.) A convex turn extends the two
/// offset edges to their intersection (clamped to a miter limit → bevel for spikes); a collinear/smooth run
/// stays a single offset point; concave folds are left for [`super::curve_trim`]. Transcendental-free (`sqrt`
/// normals + a cross-product line intersection). `d == 0` / `< 2` pts ⇒ clone.
pub(super) fn offset_polyline(poly: &[[f32; 2]], d: f32, closed: bool) -> Vec<[f32; 2]> {
    let n = poly.len();
    if d == 0.0 || n < 2 {
        return poly.to_vec();
    }
    const MITER_LIMIT: f32 = 4.0; // beyond this × |d|, a sharp spike is beveled rather than mitered to infinity
    let seg = if closed { n } else { n - 1 };
    let edge_normal = |i: usize| {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        unit_right_normal([b[0] - a[0], b[1] - a[1]])
    };
    let mut out = Vec::with_capacity(n + 4);
    for i in 0..n {
        let prev = (i > 0).then(|| i - 1).or_else(|| closed.then(|| n - 1));
        let next = (i < seg).then_some(i);
        let p = poly[i];
        match (prev.and_then(edge_normal), next.and_then(edge_normal)) {
            (Some(np), Some(nn)) => {
                // Both edges meet at p: the offset corner is the intersection of the two offset edge lines.
                let pp = poly[if i > 0 { i - 1 } else { n - 1 }];
                let pn = poly[(i + 1) % n];
                let l1 = [p[0] + np[0] * d, p[1] + np[1] * d];
                let l2 = [p[0] + nn[0] * d, p[1] + nn[1] * d];
                match line_intersect(
                    l1,
                    [p[0] - pp[0], p[1] - pp[1]],
                    l2,
                    [pn[0] - p[0], pn[1] - p[1]],
                ) {
                    Some(q) if dist2(q, p).sqrt() <= MITER_LIMIT * d.abs() => out.push(q),
                    Some(_) => {
                        out.push(l1); // over the miter limit → bevel (the two offset endpoints)
                        out.push(l2);
                    }
                    None => out.push(l1), // collinear (smooth) → one offset point
                }
            }
            (Some(nm), None) | (None, Some(nm)) => out.push([p[0] + nm[0] * d, p[1] + nm[1] * d]),
            (None, None) => out.push(p),
        }
    }
    out
}

/// Intersection of the two INFINITE lines `p1 + t·d1` and `p2 + s·d2`; `None` when parallel. Cross-product
/// determinant (no trig) — the miter join for [`offset_polyline`].
fn line_intersect(p1: [f32; 2], d1: [f32; 2], p2: [f32; 2], d2: [f32; 2]) -> Option<[f32; 2]> {
    let denom = d1[0] * d2[1] - d1[1] * d2[0];
    if denom.abs() < 1e-6 {
        return None;
    }
    let dp = [p2[0] - p1[0], p2[1] - p1[1]];
    let t = (dp[0] * d2[1] - dp[1] * d2[0]) / denom;
    Some([p1[0] + d1[0] * t, p1[1] + d1[1] * t])
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

/// Unit **right-normal** `(dy, −dx)/|v|` of direction `v` (matches [`vertex_normal`]); `None` if ≈ zero-length.
fn unit_right_normal(v: [f32; 2]) -> Option<[f32; 2]> {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (len > 1e-6).then(|| [v[1] / len, -v[0] / len])
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

/// Unit right-normal at anchor `i` for offsetting, from the curve's actual Bézier **tangents** so the anchor
/// moves perpendicular to the CURVE — the chord gives an UNEVEN distance wherever it diverges from the tangent
/// (a converted-Polygon corner you then curved). Each side uses its handle tangent, falling back to the chord
/// when the handle is collapsed (a straight `Free` edge — there the chord IS the tangent). A positive offset
/// outsets a CCW loop; zero for a degenerate vertex.
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
    let mut acc = [0.0f32, 0.0];
    let mut add = |tan: Option<[f32; 2]>, chord: [f32; 2]| {
        let t = tan
            .filter(|v| v[0] * v[0] + v[1] * v[1] > 1e-6)
            .unwrap_or(chord);
        if let Some(nm) = unit_right_normal(t) {
            acc = [acc[0] + nm[0], acc[1] + nm[1]];
        }
    };
    if let Some(p) = prev {
        add(tan_in, sub(points[i], points[p]));
    }
    if let Some(x) = next {
        add(tan_out, sub(points[x], points[i]));
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
    fn offset_polyline_miters_a_corner_to_an_even_distance() {
        // A right-angle corner: the offset corner is the MITER (intersection of the two offset edges) → it sits
        // exactly `d` from BOTH edges, not `d·cos45` (the bisector undershoot of the control-point offset).
        let poly = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
        let off = offset_polyline(&poly, 2.0, false);
        // Right-normal side: edge1 offsets to y=-2, edge2 to x=12, so the miter corner lands at (12, -2).
        let corner = off
            .iter()
            .find(|p| p[0] > 11.0)
            .expect("a miter corner is present");
        assert!(
            (corner[0] - 12.0).abs() < 0.05 && (corner[1] + 2.0).abs() < 0.05,
            "miter corner at (12,-2) — d from both edges: {corner:?}"
        );
    }

    #[test]
    fn offset_polyline_keeps_every_edge_exactly_d_away() {
        // The even-distance property: each input edge's offset stays exactly `d` perpendicular, whether the
        // edge is part of a straight run or a curved (many-segment) run — what the control-point offset lacked.
        let poly = vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [80.0, 20.0]];
        let off = offset_polyline(&poly, 3.0, false);
        for w in poly.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = dist2(a, b).sqrt();
            let nrm = [(b[1] - a[1]) / len, -(b[0] - a[0]) / len]; // unit right-normal
            let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
            // Distance from the edge midpoint to the offset POLYLINE (nearest segment), signed by the normal.
            let want = [mid[0] + nrm[0] * 3.0, mid[1] + nrm[1] * 3.0];
            let best = off
                .windows(2)
                .map(|s| point_seg_dist(want, s[0], s[1]))
                .fold(f32::INFINITY, f32::min);
            assert!(
                best < 0.5,
                "edge midpoint's d-offset lands on the offset polyline: {best}"
            );
        }
    }

    /// Distance from `p` to segment `a→b` (for the even-offset test).
    fn point_seg_dist(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
        let ab = [b[0] - a[0], b[1] - a[1]];
        let l2 = ab[0] * ab[0] + ab[1] * ab[1];
        let t = if l2 > 1e-9 {
            (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        dist2(p, [a[0] + ab[0] * t, a[1] + ab[1] * t]).sqrt()
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
}
