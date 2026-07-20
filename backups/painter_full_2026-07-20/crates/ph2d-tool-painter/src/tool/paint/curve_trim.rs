//! **Trim** the offset spine's self-intersections (the Offset card's Trim checkbox): where an over-offset
//! folds a concave side into a loop, cut it at the drawing level (the painted dab spine + guide) without
//! touching the control points. Open spines drop the looped excess; closed spines keep the main region and
//! drop the smaller "ears". Transcendental-free (segment cross products + the shoelace area). Split from
//! [`super::curve_offset`] for the workspace LOC cap; free fns, called as `curve_trim::*`.

/// **Trim** an OPEN offset spine's self-intersections: where the path crosses itself (an over-offset folds a
/// concave side into a loop), insert ONE point at the crossing and drop the looped excess. Repeats for
/// multiple loops. Strict-interior (shared/adjacent endpoints never trigger). `poly` is the dab/guide
/// polyline; CLOSED spines use [`trim_self_intersections_closed`] (area-ranked).
pub(super) fn trim_self_intersections(poly: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut pts = poly.to_vec();
    while let Some((i, j, x)) = first_crossing(&pts) {
        // Keep `0..=i`, drop the loop `i+1..=j`, splice the crossing point in, keep `j+1..`.
        let mut next = pts[..=i].to_vec();
        next.push(x);
        next.extend_from_slice(&pts[j + 1..]);
        pts = next;
    }
    pts
}

/// **Trim** a CLOSED offset spine: a self-crossing splits the loop into two sub-loops; the ear is the SMALLER
/// (by shoelace area), so keep the larger and drop the ear at every crossing — robust to which crossing is
/// found first + multiple ears (naive "drop `i+1..=j`" picked an arbitrary area; an orientation test fails as
/// a spike-ear can wind like the body — Enio 2026-06-28). No-op when nothing crosses.
/// Whether a closed loop's shoelace signed area is non-negative — the winding fingerprint the oriented
/// trim matches against (compute it on the PRISTINE control points, never on the offset spine: a deep
/// offset's ears can dominate the raw spine's total area and flip its sign).
pub(super) fn loop_sign_positive(points: &[[f32; 2]]) -> bool {
    signed_area(points) >= 0.0
}

pub(super) fn trim_self_intersections_closed(poly: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut pts = poly.to_vec();
    // An explicit closing vertex lets the scan cover the last→first edge (where an ear often crosses).
    if pts.len() >= 2 && pts.first() != pts.last() {
        pts.push(pts[0]);
    }
    while let Some((i, j, x)) = first_crossing(&pts) {
        // The crossing `x` joins two sub-loops: inner = `x → pts[i+1..=j]`, outer = `pts[..=i] → x → pts[j+1..]`.
        let inner: Vec<[f32; 2]> = std::iter::once(x)
            .chain(pts[i + 1..=j].iter().copied())
            .collect();
        let mut outer = pts[..=i].to_vec();
        outer.push(x);
        outer.extend_from_slice(&pts[j + 1..]);
        // Keep the larger region; the small loop is the unwanted crossed area (ear) → drop it.
        pts = if signed_area(&inner).abs() >= signed_area(&outer).abs() {
            inner
        } else {
            outer
        };
    }
    pts
}

/// The first self-crossing `(i, j, x)` of `poly` (segments `i`, `j`; point `x`), scanned in index order so the
/// trim is deterministic; `None` when simple. Strict-interior (shared / adjacent endpoints never count).
fn first_crossing(pts: &[[f32; 2]]) -> Option<(usize, usize, [f32; 2])> {
    for i in 0..pts.len().saturating_sub(1) {
        for j in (i + 2)..pts.len().saturating_sub(1) {
            if let Some(x) = seg_cross(pts[i], pts[i + 1], pts[j], pts[j + 1]) {
                return Some((i, j, x));
            }
        }
    }
    None
}

/// Twice the signed area of polygon `pts` (shoelace, closes implicitly). Only the MAGNITUDE is used (rank two
/// sub-loops at a crossing — the smaller is the ear). Transcendental-free.
fn signed_area(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    let mut a = 0.0f32;
    for i in 0..n {
        let p = pts[i];
        let q = pts[(i + 1) % n];
        a += p[0] * q[1] - q[0] * p[1];
    }
    a
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

#[cfg(test)]
mod tests {
    use super::super::curve_geom::dist2;
    use super::*;

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

    #[test]
    fn closed_trim_keeps_the_main_region_and_drops_the_reversed_ear() {
        // A big square with a small inward ear folded onto the top edge (the over-offset case): the path dives
        // in, crosses, and comes back, forming a tiny loop. The area-ranked closed trim must keep the big
        // square and excise the ear — NOT pick the small unwanted closed area.
        let poly = vec![
            [0.0, 0.0],
            [40.0, 0.0],
            [60.0, 20.0], // dive inward
            [40.0, 20.0], // cross back left → forms a small loop (ear) with the next leg
            [60.0, 0.0],
            [100.0, 0.0],
            [100.0, 100.0],
            [0.0, 100.0],
        ];
        let out = trim_self_intersections_closed(&poly);
        // The ear vertices (the inward dip) are gone; the four big corners survive.
        for corner in [[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]] {
            assert!(
                out.iter().any(|p| dist2(*p, corner) < 1e-3),
                "main corner {corner:?} kept: {out:?}"
            );
        }
        assert!(
            !out.iter().any(|p| dist2(*p, [60.0, 20.0]) < 1e-3),
            "the ear's tip is dropped: {out:?}"
        );
    }
}
