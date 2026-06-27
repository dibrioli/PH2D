//! Perpendicular **path offset** for the shape-stroke fills — the "Offset" slider's geometry. Shifts a
//! flattened polyline sideways by `d` px along each vertex's averaged normal (so a closed Circle/Polygon
//! perimeter insets/outsets and an open Curve/Free Hand spine shifts to one side). Pure + transcendental-
//! free (normalize via `sqrt` only, HR-5). A child of [`super`] (`stroke`) shared by the three fills.

/// Return `points` shifted by `d` px along each vertex's (averaged) left-normal. `closed` wraps the
/// endpoint normals around the loop (Circle/Polygon perimeter); otherwise the endpoints use their single
/// incident segment (open Curve/Free Hand spine). Miter-less (averaged-then-normalized normal), which is
/// stable at any angle for a preview — sharp corners under-offset slightly rather than spiking.
pub(super) fn offset_polyline(points: &[[f32; 2]], d: f32, closed: bool) -> Vec<[f32; 2]> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    (0..n)
        .map(|i| {
            let nrm = vertex_normal(points, i, closed);
            [points[i][0] + nrm[0] * d, points[i][1] + nrm[1] * d]
        })
        .collect()
}

/// The unit left-normal at vertex `i`, the normalized average of the normals of its incident segments.
/// Returns a zero vector when the vertex has no usable segment (degenerate), so the point doesn't move.
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
            // Left-normal of the direction `(dx, dy)` is `(-dy, dx)`, unit-scaled.
            acc[0] += -dy / len;
            acc[1] += dx / len;
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
mod tests;
