//! Perpendicular **offset** (parallel curve) of the Curve / Free Hand control geometry — the Offset slider.
//! [`offset_curve_refined`] *reconstructs* the curve with extra anchors wherever a control-point offset would
//! stray from the true parallel curve (adaptive subdivision to sub-pixel tolerance — CAD-grade, cf. Levien
//! 2022), then [`super::curve_join::offset_curve`] offsets + joins the corners, staying an ordinary editable
//! anchor/handle curve. Transcendental-free (rotation = complex multiply from chord unit vectors; normals
//! `sqrt`-normalised — HR-5). Free fns, called as `curve_offset::*`.

use super::curve_geom::{cubic_at, dist2, split_cubic};
use super::curve_handle::HandleKind;

/// Adaptive-subdivision tolerance: max px the densified offset may stray before a segment is split (sub-pixel).
const OFFSET_TOL_PX: f32 = 0.3;
/// Depth cap on the offset subdivision (≤ 2^6 leaves per input segment) — bounds the work near a cusp.
const MAX_OFFSET_SUBDIV: u32 = 6;

/// Offset `(points, handles)` + per output anchor its ORIGINAL index (`None` = inserted). Aliased for clippy.
type RefinedOffset = (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>, Vec<Option<usize>>);
/// [`offset_curve_refined_kinds`] output: dense `(points, handles)` + carried kinds + remapped selection.
type RefinedKinds = (
    Vec<[f32; 2]>,
    Vec<[[f32; 2]; 2]>,
    Vec<HandleKind>,
    Option<usize>,
);

/// [`super::curve_join::offset_curve`] preceded by [`densify_for_offset`]: the offset hugs the true parallel curve even through
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
    // The corner JOIN may SPLIT a concave corner into two anchors (so the spine crosses + Trim cuts) — compose
    // its per-output `remap` (output → dense index) with the dense `origin` to keep each output's source.
    let (op, oh, remap) = super::curve_join::offset_curve(&dp, &dh, d, closed);
    let origin = remap
        .iter()
        .map(|&i| origin.get(i).copied().flatten())
        .collect();
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

/// Reconstruct the curve with extra anchors wherever the control-polygon offset would stray from the true
/// parallel curve by more than [`OFFSET_TOL_PX`] (adaptive de Casteljau split at the midpoint). The SHAPE is
/// unchanged — the sub-cubics sum to the original arcs — only the resolution rises where curvature needs it,
/// so the subsequent offset (in [`super::curve_join`]) follows the true offset. Exact cases (straight legs, the cubic circle
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
/// the corner join does per leaf): max deviation from the TRUE offset `C(t) + d·N(t)`. ≈ 0 for a line / arc
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

/// Unit **right-normal** `(dy, −dx)/|v|` of direction `v`; `None` if ≈ zero-length. Shared with the corner
/// join ([`super::curve_join`]).
pub(super) fn unit_right_normal(v: [f32; 2]) -> Option<[f32; 2]> {
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
/// Shared with the corner join ([`super::curve_join`]).
pub(super) struct SegXform {
    cos: f32,
    sin: f32,
    scale: f32,
}

impl SegXform {
    pub(super) fn new(a: [f32; 2], b: [f32; 2], oa: [f32; 2], ob: [f32; 2]) -> Self {
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
    pub(super) fn apply(
        &self,
        anchor: [f32; 2],
        handle: [f32; 2],
        offset_anchor: [f32; 2],
    ) -> [f32; 2] {
        let v = [handle[0] - anchor[0], handle[1] - anchor[1]];
        let rx = (v[0] * self.cos - v[1] * self.sin) * self.scale;
        let ry = (v[0] * self.sin + v[1] * self.cos) * self.scale;
        [offset_anchor[0] + rx, offset_anchor[1] + ry]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
