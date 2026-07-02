//! Perpendicular **Offset** for the Line polyline editor — the same Offset-slider affordance the other
//! shapes have (`shape_offset_px`), applied to the rendered path. A parallel polyline: every vertex shifts
//! along the miter of its two edge normals; open endpoints shift along their single edge normal. Sibling of
//! [`super::line`] (LOC cap). Transcendental-free (HR-5): unit normals + a miter via the bisector, no trig.

/// Miter limit — how far (× `|d|`) a sharp corner's join may extend before it's clamped, so an acute corner
/// doesn't shoot a spike to infinity (standard stroke-offset guard). ~11°+ corners are exact; sharper clamp.
const MITER_LIMIT: f32 = 4.0;

/// Offset the (already expanded) polyline `path` perpendicular by `d` px — positive = the right-hand side of
/// travel, negative = the left. Vertices join with clamped miters; open endpoints shift along their one edge
/// normal. `d == 0` or `< 2` points ⇒ an unchanged clone. A CLOSED path (its first point re-appended by
/// `expand`) is offset as a loop and re-closed.
pub(super) fn offset_polyline(path: &[[f32; 2]], closed: bool, d: f32) -> Vec<[f32; 2]> {
    let n = path.len();
    if d == 0.0 || n < 2 {
        return path.to_vec();
    }
    // A closed rendered path repeats its first point at the end (from `expand`); work on the unique ring.
    let closed_ring = closed && n >= 3 && path[0] == path[n - 1];
    let verts: &[[f32; 2]] = if closed_ring { &path[..n - 1] } else { path };
    let m = verts.len();
    if m < 2 {
        return path.to_vec();
    }
    let mut out = Vec::with_capacity(m + usize::from(closed_ring));
    for i in 0..m {
        let cur = verts[i];
        // Incoming edge normal (from the previous vertex) + outgoing edge normal (to the next), each present
        // unless `cur` is an open endpoint.
        let n_in = (closed_ring || i > 0).then(|| {
            let p = verts[if i == 0 { m - 1 } else { i - 1 }];
            right_normal([cur[0] - p[0], cur[1] - p[1]])
        });
        let n_out = (closed_ring || i + 1 < m).then(|| {
            let q = verts[if i + 1 == m { 0 } else { i + 1 }];
            right_normal([q[0] - cur[0], q[1] - cur[1]])
        });
        let disp = miter(n_in.flatten(), n_out.flatten());
        out.push([cur[0] + disp[0] * d, cur[1] + disp[1] * d]);
    }
    if closed_ring {
        out.push(out[0]); // re-close the loop for the polyline fill
    }
    out
}

/// The unit RIGHT-hand normal of edge vector `e` (rotate the unit direction by −90°: `(x,y) → (y,−x)`).
/// `None` for a degenerate (zero-length) edge.
fn right_normal(e: [f32; 2]) -> Option<[f32; 2]> {
    let len = (e[0] * e[0] + e[1] * e[1]).sqrt();
    if len < 1e-6 {
        return None;
    }
    Some([e[1] / len, -e[0] / len])
}

/// Combine the incoming + outgoing edge normals into the vertex displacement direction (unit-ish, scaled by
/// the miter factor). One normal (an endpoint) ⇒ that normal. Two ⇒ the bisector scaled by `1/cos(θ/2)` so
/// the offset segments meet, clamped to [`MITER_LIMIT`]. Degenerate ⇒ no move.
fn miter(n_in: Option<[f32; 2]>, n_out: Option<[f32; 2]>) -> [f32; 2] {
    match (n_in, n_out) {
        (Some(a), Some(b)) => {
            let sum = [a[0] + b[0], a[1] + b[1]];
            let len = (sum[0] * sum[0] + sum[1] * sum[1]).sqrt();
            if len < 1e-4 {
                return a; // ~antiparallel edges (a 180° reversal) — just use one normal
            }
            let bis = [sum[0] / len, sum[1] / len];
            // scale = 1 / (bis · a) = 1 / cos(half-angle); clamp the spike on sharp corners.
            let cos_half = bis[0] * a[0] + bis[1] * a[1];
            let scale = if cos_half.abs() < 1e-4 {
                MITER_LIMIT
            } else {
                (1.0 / cos_half).clamp(-MITER_LIMIT, MITER_LIMIT)
            };
            [bis[0] * scale, bis[1] * scale]
        }
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => [0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_shifts_a_straight_segment_perpendicular() {
        // A horizontal segment offset by +10 moves down by 10 (right-hand normal of +x travel is (0,-1)).
        let path = [[0.0, 0.0], [100.0, 0.0]];
        let off = offset_polyline(&path, false, 10.0);
        assert_eq!(off.len(), 2);
        assert!(
            (off[0][1] - -10.0).abs() < 1e-3,
            "start moved down: {off:?}"
        );
        assert!((off[1][1] - -10.0).abs() < 1e-3, "end moved down: {off:?}");
        assert!(
            (off[0][0]).abs() < 1e-3 && (off[1][0] - 100.0).abs() < 1e-3,
            "x unchanged"
        );
    }

    #[test]
    fn offset_miters_a_right_angle() {
        // An L (down then right) offset by d puts the corner at the miter apex, distance d·√2 from it.
        let path = [[0.0, 0.0], [0.0, 100.0], [100.0, 100.0]];
        let off = offset_polyline(&path, false, 10.0);
        assert_eq!(off.len(), 3);
        // Interior corner displaced by 10·√2 along the 45° bisector.
        let dx = off[1][0] - path[1][0];
        let dy = off[1][1] - path[1][1];
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            (dist - 10.0 * std::f32::consts::SQRT_2).abs() < 0.1,
            "corner mitered to d·√2, got {dist}"
        );
    }

    #[test]
    fn offset_zero_is_identity() {
        let path = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        assert_eq!(offset_polyline(&path, false, 0.0), path.to_vec());
    }

    #[test]
    fn closed_ring_stays_closed() {
        // A square (first point repeated at the end) offset outward stays a closed 5-point ring.
        let sq = [
            [0.0, 0.0],
            [100.0, 0.0],
            [100.0, 100.0],
            [0.0, 100.0],
            [0.0, 0.0],
        ];
        let off = offset_polyline(&sq, true, 10.0);
        assert_eq!(off.len(), 5, "4 unique verts + the re-closed first");
        assert_eq!(off[0], off[4], "loop re-closed");
    }
}
