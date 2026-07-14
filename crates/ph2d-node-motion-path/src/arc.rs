//! Arc-length along a polyline — the leaf that makes "evenly spaced" mean **evenly spaced**.
//!
//! Sampling a curve at even *parameter* bunches the points on the tight bends; sampling at even
//! *arc-length* is what the eye reads as even. This is the same machinery `motion.distribute_curve`
//! keeps in its own `curve.rs`, cut down to a polyline (the shell hands the curve over already
//! flattened — the graph never sees a Bézier, and does not need to).
//!
//! Transcendental-free (HR-5): chord lengths are `sqrt`, everything else is arithmetic.

/// Cumulative length at each vertex: `lut[i]` = the distance from the start to vertex `i`.
/// `lut.last()` is the total. Fewer than two points → empty.
pub(crate) fn lut(pts: &[[f32; 2]]) -> Vec<f32> {
    if pts.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(pts.len());
    let mut acc = 0.0f32;
    out.push(0.0);
    for w in pts.windows(2) {
        let (dx, dy) = (w[1][0] - w[0][0], w[1][1] - w[0][1]);
        acc += (dx * dx + dy * dy).sqrt();
        out.push(acc);
    }
    out
}

/// The point (and the unit tangent) at arc-length fraction `s` of the polyline.
///
/// `s` is wrapped into `[0, 1)`, so an `offset` slides the whole set along the curve and around the
/// end — which is the gesture (a marquee flowing down a path) rather than a clamp at the tip.
pub(crate) fn at(pts: &[[f32; 2]], lut: &[f32], s: f32) -> ([f32; 2], [f32; 2]) {
    let total = *lut.last().unwrap_or(&0.0);
    if pts.len() < 2 || total <= 0.0 {
        return (*pts.first().unwrap_or(&[0.0, 0.0]), [1.0, 0.0]);
    }
    let target = (s - s.floor()) * total; // wrap into [0,1) — the curve is a loop to walk, not a line to fall off
    // The segment the target falls in. Linear scan: a path is tens of points, and a binary search
    // here would be a second way to be wrong about the same question.
    let mut i = 0;
    while i + 2 < lut.len() && lut[i + 1] < target {
        i += 1;
    }
    let (a, b) = (pts[i], pts[i + 1]);
    let seg = (lut[i + 1] - lut[i]).max(f32::MIN_POSITIVE);
    let t = ((target - lut[i]) / seg).clamp(0.0, 1.0); // CLAMP-OK: a fraction of one segment
    let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt().max(f32::MIN_POSITIVE);
    (p, [dx / len, dy / len])
}
