//! Corner **joins** for the perpendicular offset: where one segment's offset meets the next, a SMOOTH vertex
//! merges to one anchor, a CONVEX corner extends to the miter apex, and a CONCAVE corner **splits** into its
//! two per-edge offset points — so the two offset edges overshoot to opposite sides and the flattened spine
//! self-CROSSES, which the Trim pass then cuts (the CAD "offset then trim" the user wants: a single merged
//! miter vertex can never self-cross — Enio 2026-06-29). Transcendental-free (cross products + the chord
//! complex-multiply rotation in [`super::curve_offset::SegXform`]). Free fns, called as `curve_join::*`.

use super::curve_offset::{SegXform, unit_right_normal};

/// Miter limit for a CONVEX corner (offset on the OUTSIDE of the turn — a gap): cap the apex displacement to
/// this multiple of `d` so an acute corner doesn't shoot into a spike (cf. SVG `stroke-miterlimit` = 4). A
/// convex spike can't self-cross, so Trim couldn't clean it — hence the bound. Concave corners aren't capped
/// here: they split (cross) instead.
const MITER_LIMIT: f32 = 4.0;
/// A vertex whose two side-normals agree to within this dot is SMOOTH (one merged anchor) — only a real
/// tangent discontinuity (a corner) drops below it and may split / miter, so a smooth bend never fragments.
const SMOOTH_DOT: f32 = 0.999;

/// [`offset_curve`] output: offset `(points, handles)` + a `remap` of each output anchor to its INPUT index
/// (a concave split maps two outputs to one input). Aliased for clippy's type-complexity lint.
type JoinedOffset = (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>, Vec<usize>);

/// Offset the curve's **control geometry** perpendicular by `d` px and JOIN the corners: each anchor offsets
/// along its side-normals; a smooth vertex stays one anchor, a CONVEX corner extends to the miter apex, a
/// CONCAVE corner **splits** into its two per-edge points so the offset self-crosses (Trim cuts the ear).
/// Returns the offset `(points, handles)` + a `remap` of each output anchor to its INPUT index (a split maps
/// two outputs to one input) so the caller carries the origin / kinds / selection. `d == 0` ⇒ unchanged clones.
pub(super) fn offset_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    d: f32,
    closed: bool,
) -> JoinedOffset {
    let n = points.len();
    if d == 0.0 || n < 2 || handles.len() != n {
        return (points.to_vec(), handles.to_vec(), (0..n).collect());
    }
    // Per input vertex → its output slot(s): `in_slot` ends the incoming segment, `out_slot` starts the
    // outgoing one. Equal for a smooth / convex vertex (1 anchor); a concave corner splits into two.
    let mut op: Vec<[f32; 2]> = Vec::with_capacity(n + 4);
    let mut remap: Vec<usize> = Vec::with_capacity(n + 4);
    let mut in_slot = vec![0usize; n];
    let mut out_slot = vec![0usize; n];
    for i in 0..n {
        let (sides, count) = side_normals(points, handles, i, closed);
        let at = |nrm: [f32; 2]| [points[i][0] + nrm[0] * d, points[i][1] + nrm[1] * d];
        if count == 2 && is_concave(sides[0], sides[1], d) {
            // SPLIT: P_in (incoming edge) then P_out (outgoing edge). On the concave side they sit on opposite
            // sides of the vertex, so the two offset segments overshoot + the spine crosses; the straight
            // connector between them is the ear Trim removes.
            in_slot[i] = op.len();
            op.push(at(sides[0]));
            remap.push(i);
            out_slot[i] = op.len();
            op.push(at(sides[1]));
            remap.push(i);
        } else {
            // ONE anchor: a lone side (open endpoint) keeps its unit normal; otherwise the miter (unit when
            // smooth, the clamped apex when convex).
            let disp = if count < 2 {
                sides[0]
            } else {
                miter(sides[0], sides[1])
            };
            in_slot[i] = op.len();
            out_slot[i] = op.len();
            op.push(at(disp));
            remap.push(i);
        }
    }
    // Handles: collapsed at each anchor by default (a straight connector across a split); then every segment
    // rotate+scales its two tangent handles onto its own offset chord.
    let mut oh: Vec<[[f32; 2]; 2]> = op.iter().map(|&p| [p, p]).collect();
    let seg_count = if closed { n } else { n - 1 };
    for s in 0..seg_count {
        let (a, b) = (s, (s + 1) % n);
        let (oa, ob) = (op[out_slot[a]], op[in_slot[b]]);
        let xf = SegXform::new(points[a], points[b], oa, ob);
        oh[out_slot[a]][1] = xf.apply(points[a], handles[a][1], oa); // out handle of a's outgoing anchor
        oh[in_slot[b]][0] = xf.apply(points[b], handles[b][0], ob); // in handle of b's incoming anchor
    }
    (op, oh, remap)
}

/// The (up to two) unit side-normals at anchor `i` — one per incident edge, ⊥ the Bézier **tangent** (chord
/// fallback for a collapsed handle, so a sharp `Free` corner still resolves). `sides[0]` is the arriving edge,
/// `sides[1]` the leaving; `count` is how many exist (1 at an open endpoint). The corner join combines them.
fn side_normals(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    i: usize,
    closed: bool,
) -> ([[f32; 2]; 2], usize) {
    let n = points.len();
    let bez = handles.len() == n;
    let sub = |a: [f32; 2], b: [f32; 2]| [a[0] - b[0], a[1] - b[1]];
    let prev = (i > 0).then(|| i - 1).or_else(|| closed.then(|| n - 1));
    let next = (i + 1 < n).then(|| i + 1).or_else(|| closed.then_some(0));
    let tan_in = bez.then(|| sub(points[i], handles[i][0])); // travel arriving at i
    let tan_out = bez.then(|| sub(handles[i][1], points[i])); // travel leaving i
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
    (sides, count)
}

/// A corner is CONCAVE on the offset side (the two edges overlap → split + cross) when its turn `n1×n2`
/// opposes `d`'s sign AND it's a real corner (the normals disagree past [`SMOOTH_DOT`], so a smooth bend never
/// splits). Convex / smooth ⇒ `false` (one merged anchor).
fn is_concave(n1: [f32; 2], n2: [f32; 2], d: f32) -> bool {
    let dot = n1[0] * n2[0] + n1[1] * n2[1];
    let turn = n1[0] * n2[1] - n1[1] * n2[0];
    dot < SMOOTH_DOT && turn * d < 0.0
}

/// Combine two unit side-normals into a CONVEX corner's apex displacement: `(n1+n2)/(1+n1·n2)`, the bisector
/// at distance `1/cos δ` (the true miter), clamped to [`MITER_LIMIT`]. Smooth (n1 ≈ n2) ⇒ unit. Concave
/// corners never reach here (they split). Transcendental-free.
fn miter(n1: [f32; 2], n2: [f32; 2]) -> [f32; 2] {
    let s = [n1[0] + n2[0], n1[1] + n2[1]];
    let sl = (s[0] * s[0] + s[1] * s[1]).sqrt();
    if sl < 1e-6 {
        return [0.0, 0.0]; // antiparallel cusp: no well-defined offset direction
    }
    let denom = (1.0 + n1[0] * n2[0] + n1[1] * n2[1]).max(1e-6);
    let mag = (sl / denom).min(MITER_LIMIT); // |miter| = 1/cos δ; bound so an acute convex corner can't spike
    [s[0] / sl * mag, s[1] / sl * mag]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsetting_a_circle_stays_concentric_and_scales_the_handles() {
        // 4 cardinal anchors, radius 20, with circle tangent handles (k ≈ 0.5523·r). Offsetting the closed
        // loop outward keeps every anchor equidistant (a clean concentric circle), scales the handles by the
        // radius ratio, and — being smooth — never splits (one anchor per vertex).
        let r = 20.0f32;
        let k = 0.55228f32 * r;
        let pts = vec![[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
        let handles = vec![
            [[r, -k], [r, k]],   // (r,0): tangent vertical
            [[k, r], [-k, r]],   // (0,r): tangent horizontal
            [[-r, k], [-r, -k]], // (-r,0)
            [[-k, -r], [k, -r]], // (0,-r)
        ];
        let (op, oh, remap) = offset_curve(&pts, &handles, 10.0, true);
        assert_eq!(op.len(), 4, "a smooth loop never splits");
        assert_eq!(remap, vec![0, 1, 2, 3]);
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
    fn a_convex_corner_stays_one_sharp_miter_anchor() {
        // A right-angle Polygon corner (collapsed handles) offset OUTWARD is convex: one anchor at the miter,
        // handles still collapsed (a corner, not a rounded blob), no split.
        let pts = vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0]];
        let handles = vec![[[0.0, 0.0]; 2], [[20.0, 0.0]; 2], [[20.0, 20.0]; 2]];
        let (op, oh, _) = offset_curve(&pts, &handles, 5.0, false);
        assert_eq!(op.len(), 3, "convex corner is not split");
        for i in 0..op.len() {
            assert_eq!(oh[i][0], op[i], "in handle stays collapsed at {i}");
            assert_eq!(oh[i][1], op[i], "out handle stays collapsed at {i}");
        }
    }

    #[test]
    fn a_concave_corner_splits_into_two_overshooting_anchors() {
        // The SAME sharp corner offset toward its INSIDE (sign of d flipped) is concave: it must split into
        // two anchors that land on OPPOSITE sides of the vertex, so the two offset edges overshoot and the
        // spine self-crosses — Trim then cuts the ear (the cross the user asked for).
        let pts = vec![[0.0, 10.0], [10.0, 0.0], [20.0, 10.0]]; // a V; its inside is concave for d < 0
        let handles = vec![[[0.0, 10.0]; 2], [[10.0, 0.0]; 2], [[20.0, 10.0]; 2]];
        let (op, _oh, remap) = offset_curve(&pts, &handles, -5.0, false);
        assert_eq!(
            op.len(),
            4,
            "concave corner split into two anchors: {}",
            op.len()
        );
        assert_eq!(
            remap,
            vec![0, 1, 1, 2],
            "both split anchors map to corner 1: {remap:?}"
        );
        let (p_in, p_out) = (op[1], op[2]);
        assert!(
            p_in[0] > 10.0 && p_out[0] < 10.0,
            "the two offset edges overshoot to opposite sides of the corner: {p_in:?} {p_out:?}"
        );
        // Convex (d > 0) on the same corner does NOT split.
        let (op2, _, _) = offset_curve(&pts, &handles, 5.0, false);
        assert_eq!(
            op2.len(),
            3,
            "the convex side stays one miter anchor: {}",
            op2.len()
        );
    }

    #[test]
    fn side_normals_follow_the_handle_tangent_not_the_chord() {
        // An anchor whose handles define a HORIZONTAL tangent has VERTICAL side-normals (⊥ the curve), even
        // when its chords to the neighbours point elsewhere — the even-offset property a converted Polygon lacked.
        let pts = vec![[0.0, 0.0], [10.0, 10.0], [40.0, 30.0]];
        let handles = vec![
            [[0.0, 0.0], [3.0, 0.0]],
            [[5.0, 10.0], [15.0, 10.0]], // middle anchor's tangent is horizontal (in→out along +x)
            [[40.0, 30.0], [40.0, 30.0]],
        ];
        let (sides, count) = side_normals(&pts, &handles, 1, false);
        assert_eq!(count, 2);
        assert!(
            sides[0][0].abs() < 0.05 && sides[1][0].abs() < 0.05,
            "both normals vertical: {sides:?}"
        );
    }

    #[test]
    fn the_convex_miter_reaches_the_true_distance_then_clamps() {
        // A 90° corner miters to d/cos45° = √2 (the true parallel-curve corner, no undershoot); a near-
        // antiparallel (very acute) corner clamps to the miter limit instead of shooting to infinity.
        let m = miter([0.0, -1.0], [1.0, 0.0]);
        assert!(
            (m[0] - 1.0).abs() < 1e-3 && (m[1] + 1.0).abs() < 1e-3,
            "miter vector (1,-1): {m:?}"
        );
        let len = (m[0] * m[0] + m[1] * m[1]).sqrt();
        assert!(
            (len - std::f32::consts::SQRT_2).abs() < 1e-3,
            "miter length √2: {len}"
        );
        let h = (1.0f32 - 0.99 * 0.99).sqrt();
        let acute = miter([1.0, 0.0], [-0.99, h]); // ~172° apart
        let alen = (acute[0] * acute[0] + acute[1] * acute[1]).sqrt();
        assert!(
            alen <= MITER_LIMIT + 1e-3,
            "acute convex corner clamped: {alen}"
        );
    }

    #[test]
    fn a_smooth_vertex_miter_stays_unit() {
        // n1 ≈ n2 ⇒ the miter collapses to the plain unit normal ⇒ circles / Auto / Vector are unchanged.
        let n = [0.6f32, 0.8];
        let m = miter(n, n);
        assert!(
            (m[0] - n[0]).abs() < 1e-4 && (m[1] - n[1]).abs() < 1e-4,
            "unit normal: {m:?}"
        );
    }
}
