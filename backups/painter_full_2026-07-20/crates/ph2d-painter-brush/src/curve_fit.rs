//! **Piecewise cubic Bézier curve fitting** (Schneider, *Graphics Gems* 1990) — clean-room.
//!
//! Fits the FEWEST cubic Béziers to a captured freehand point stream within an error tolerance, then
//! returns the anchors + their tangent **handles** ([`CurveFit`]). The Free Hand stroke stores these and
//! flattens the Béziers directly ([`flatten_bezier`]): far fewer, better-placed points than a
//! Douglas–Peucker subset (which keeps a vertex at every wiggle) AND faithful to the stroke (the handles
//! ARE the fitted tangents, so there's no Catmull-Rom re-smoothing deformation). The fit places an anchor
//! only where the curvature genuinely turns, and least-squares-fits the tangents in between.
//!
//! Deterministic (HR-5): only `+ − × ÷` and `sqrt` (vector normalize + chord length); no `sin/cos/exp/
//! pow` — the Bernstein basis and the Newton reparameterisation are plain polynomials.

type P = [f32; 2];

#[inline]
fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
#[inline]
fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1]]
}
#[inline]
fn scale(a: P, s: f32) -> P {
    [a[0] * s, a[1] * s]
}
#[inline]
fn dot(a: P, b: P) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
#[inline]
fn dist2(a: P, b: P) -> f32 {
    let d = sub(a, b);
    dot(d, d)
}
#[inline]
fn norm(a: P) -> P {
    let l = dot(a, a).sqrt();
    if l > 1e-6 {
        [a[0] / l, a[1] / l]
    } else {
        [0.0, 0.0]
    }
}

/// The cubic Bernstein basis at `t` (B0..B3).
#[inline]
fn bernstein(t: f32) -> [f32; 4] {
    let mt = 1.0 - t;
    [mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t]
}

/// Evaluate a cubic Bézier (control points `b`) at `t`.
#[inline]
fn bezier_at(b: &[P; 4], t: f32) -> P {
    let w = bernstein(t);
    add(
        add(scale(b[0], w[0]), scale(b[1], w[1])),
        add(scale(b[2], w[2]), scale(b[3], w[3])),
    )
}

/// First derivative `Q'(t)` of the cubic Bézier (a quadratic Bézier on the hodograph).
#[inline]
fn bezier_d1(b: &[P; 4], t: f32) -> P {
    let mt = 1.0 - t;
    let a = scale(sub(b[1], b[0]), 3.0 * mt * mt);
    let c = scale(sub(b[2], b[1]), 6.0 * mt * t);
    let d = scale(sub(b[3], b[2]), 3.0 * t * t);
    add(add(a, c), d)
}

/// Second derivative `Q''(t)` of the cubic Bézier.
#[inline]
fn bezier_d2(b: &[P; 4], t: f32) -> P {
    let mt = 1.0 - t;
    let a = scale(add(sub(b[2], scale(b[1], 2.0)), b[0]), 6.0 * mt);
    let c = scale(add(sub(b[3], scale(b[2], 2.0)), b[1]), 6.0 * t);
    add(a, c)
}

/// A fitted curve: the anchor points + per-anchor Bézier **handles** (absolute positions). `handles[i]`
/// is `[in, out]` — the incoming and outgoing tangent control points around `anchors[i]`; the segment
/// `anchors[i] → anchors[i+1]` is the cubic Bézier `[anchors[i], handles[i].out, handles[i+1].in,
/// anchors[i+1]]`. The first anchor's `in` and the last anchor's `out` equal their anchor (no handle).
/// Unlike a Catmull-Rom polygon (whose tangents are invented from neighbours), these handles ARE the
/// fitted tangents — flattening them reproduces the captured stroke faithfully (no deformation).
pub struct CurveFit {
    /// The anchor points, start→end.
    pub anchors: Vec<P>,
    /// `[in, out]` absolute handle positions per anchor (same length as `anchors`).
    pub handles: Vec<[P; 2]>,
}

/// Fit `points` to cubic Béziers within `max_error` (px) deviation and return the anchors + their Bézier
/// handles ([`CurveFit`]). A larger `max_error` yields fewer points. Endpoints are always preserved.
/// Fewer than 3 input points pass through unchanged (after collapsing exact duplicates), with zero-length
/// handles (a straight segment).
#[must_use]
pub fn fit_curve(points: &[P], max_error: f32) -> CurveFit {
    // Collapse consecutive exact-duplicate samples (a parked cursor) — they give degenerate tangents.
    let mut pts: Vec<P> = Vec::with_capacity(points.len());
    for &p in points {
        if pts.last().is_none_or(|&q| sub(p, q) != [0.0, 0.0]) {
            pts.push(p);
        }
    }
    if pts.len() <= 2 {
        let handles = pts.iter().map(|&a| [a, a]).collect();
        return CurveFit {
            anchors: pts,
            handles,
        };
    }
    let left_t = norm(sub(pts[1], pts[0]));
    let n = pts.len();
    let right_t = norm(sub(pts[n - 2], pts[n - 1]));
    let err = max_error.max(0.1);
    let mut segs: Vec<[P; 4]> = Vec::new();
    fit_cubic(&pts, left_t, right_t, err * err, &mut segs);
    segments_to_fit(&segs)
}

/// Assemble the recursion's ordered cubic-Bézier segments (`[P0, C0, C1, P1]`) into anchors + per-anchor
/// `[in, out]` handles: anchor 0 is the first `P0` (in = itself), each join takes its `in` from the
/// previous segment's `C1` and its `out` from the next segment's `C0`, the last anchor's `out` = itself.
fn segments_to_fit(segs: &[[P; 4]]) -> CurveFit {
    if segs.is_empty() {
        return CurveFit {
            anchors: Vec::new(),
            handles: Vec::new(),
        };
    }
    let mut anchors = vec![segs[0][0]];
    let mut handles = vec![[segs[0][0], segs[0][1]]]; // first: in = anchor, out = C0
    for (i, s) in segs.iter().enumerate() {
        anchors.push(s[3]); // P1
        let out = segs.get(i + 1).map_or(s[3], |nx| nx[1]); // next C0, else the anchor (last)
        handles.push([s[2], out]); // in = this C1, out = next C0
    }
    CurveFit { anchors, handles }
}

/// Fit one run `pts` (with end tangents) to Béziers within squared error `err2`, appending each emitted
/// cubic segment `[P0, C0, C1, P1]` to `out`. Recurses, splitting at the worst-fit point (Schneider).
fn fit_cubic(pts: &[P], left_t: P, right_t: P, err2: f32, out: &mut Vec<[P; 4]>) {
    let n = pts.len();
    if n == 2 {
        // A 2-point run → a straight cubic (handles at the 1/3 chord, so flattening is the segment).
        let third = scale(sub(pts[1], pts[0]), 1.0 / 3.0);
        out.push([pts[0], add(pts[0], third), sub(pts[1], third), pts[1]]);
        return;
    }
    let mut u = chord_length_param(pts);
    let mut bez = generate_bezier(pts, &u, left_t, right_t);
    let (mut max_d2, mut split) = max_error_point(pts, &bez, &u);
    if max_d2 < err2 {
        out.push(bez);
        return;
    }
    // Close enough to try Newton reparameterisation a few times before giving up and splitting
    // (Schneider's `maxError < error·4` gate, here in squared space: `(4·error)² = 16·err2`).
    if max_d2 < 16.0 * err2 {
        for _ in 0..4 {
            u = reparameterize(pts, &u, &bez);
            bez = generate_bezier(pts, &u, left_t, right_t);
            let (d2, s) = max_error_point(pts, &bez, &u);
            max_d2 = d2;
            split = s;
            if max_d2 < err2 {
                out.push(bez);
                return;
            }
        }
    }
    let split = split.clamp(1, n - 2);
    let center_t = norm(sub(pts[split - 1], pts[split + 1]));
    fit_cubic(&pts[..=split], left_t, center_t, err2, out);
    fit_cubic(&pts[split..], scale(center_t, -1.0), right_t, err2, out);
}

/// Least-squares fit of the two interior control points, given the endpoints (`pts` first/last) and the
/// end tangents — the Bézier through `pts` at chord parameters `u`. Falls back to the Wu–Barsky 1/3-chord
/// heuristic when the normal equations are degenerate or yield a non-positive tangent length.
fn generate_bezier(pts: &[P], u: &[f32], left_t: P, right_t: P) -> [P; 4] {
    let n = pts.len();
    let first = pts[0];
    let last = pts[n - 1];
    let (mut c00, mut c01, mut c11) = (0.0f32, 0.0f32, 0.0f32);
    let (mut x0, mut x1) = (0.0f32, 0.0f32);
    for i in 0..n {
        let w = bernstein(u[i]);
        let a0 = scale(left_t, w[1]); // tangent · B1
        let a1 = scale(right_t, w[2]); // tangent · B2
        c00 += dot(a0, a0);
        c01 += dot(a0, a1);
        c11 += dot(a1, a1);
        // residual = point − (first·(B0+B1) + last·(B2+B3))
        let base = add(scale(first, w[0] + w[1]), scale(last, w[2] + w[3]));
        let tmp = sub(pts[i], base);
        x0 += dot(a0, tmp);
        x1 += dot(a1, tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let chord = dist2(first, last).sqrt();
    let (mut al, mut ar) = if det.abs() > 1e-12 {
        ((x0 * c11 - c01 * x1) / det, (c00 * x1 - c01 * x0) / det)
    } else {
        (0.0, 0.0)
    };
    let eps = 1e-6 * chord;
    if al < eps || ar < eps {
        let third = chord / 3.0;
        al = third;
        ar = third;
    }
    [
        first,
        add(first, scale(left_t, al)),
        add(last, scale(right_t, ar)),
        last,
    ]
}

/// Worst squared deviation of `pts` from `bez` at parameters `u`, and the index where it occurs (the
/// split point). The endpoints are exact by construction, so the search runs the interior.
fn max_error_point(pts: &[P], bez: &[P; 4], u: &[f32]) -> (f32, usize) {
    let n = pts.len();
    let (mut max_d2, mut split) = (0.0f32, n / 2);
    for i in 1..n - 1 {
        let d2 = dist2(bezier_at(bez, u[i]), pts[i]);
        if d2 >= max_d2 {
            max_d2 = d2;
            split = i;
        }
    }
    (max_d2, split)
}

/// One Newton–Raphson step per point toward the nearest `t` on `bez` (improves the chord-length guess so
/// the next least-squares fit is tighter — Schneider's reparameterisation).
fn reparameterize(pts: &[P], u: &[f32], bez: &[P; 4]) -> Vec<f32> {
    pts.iter()
        .zip(u)
        .map(|(&p, &ui)| {
            let q = bezier_at(bez, ui);
            let q1 = bezier_d1(bez, ui);
            let q2 = bezier_d2(bez, ui);
            let diff = sub(q, p);
            let den = dot(q1, q1) + dot(diff, q2);
            if den.abs() < 1e-12 {
                ui
            } else {
                (ui - dot(diff, q1) / den).clamp(0.0, 1.0)
            }
        })
        .collect()
}

/// Cumulative chord-length parameterisation of `pts`, normalised to `[0, 1]`.
fn chord_length_param(pts: &[P]) -> Vec<f32> {
    let mut u = vec![0.0f32; pts.len()];
    for i in 1..pts.len() {
        u[i] = u[i - 1] + dist2(pts[i], pts[i - 1]).sqrt();
    }
    let total = u[pts.len() - 1];
    if total > 0.0 {
        for x in &mut u {
            *x /= total;
        }
    }
    u
}

/// **Auto-smooth handles** for an AUTHORED control polygon (the Curve method): per-anchor `[in, out]`
/// Bézier handles for a smooth interpolating spline THROUGH `points`, with handle lengths proportional to
/// the adjacent segment lengths (a **chordal** Catmull-Rom). This is the no-overshoot upgrade over uniform
/// Catmull-Rom (`(next−prev)/2`), which loops/cusps on unevenly-spaced points; here a short segment gets a
/// short handle, so the curve stays inside its control polygon. For evenly-spaced points it reduces to the
/// uniform spline (handle = `(next−prev)/6`), so a straight run stays straight. Recompute on every edit
/// (the spline re-smooths as the artist drags / adds / deletes points). Deterministic (HR-5: `+−×÷`,
/// `sqrt`). `< 2` points ⇒ zero-length handles.
#[must_use]
pub fn auto_handles(points: &[P]) -> Vec<[P; 2]> {
    let n = points.len();
    if n < 2 {
        return points.iter().map(|&a| [a, a]).collect();
    }
    // Tension so even spacing matches the uniform Catmull-Rom (handle = chord/6): `fa = fb = T/2 = 1/6`.
    const T: f32 = 1.0 / 3.0;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let cur = points[i];
        let prev = points[i.saturating_sub(1)];
        let next = points[(i + 1).min(n - 1)];
        let d_in = dist2(cur, prev).sqrt();
        let d_out = dist2(cur, next).sqrt();
        let sum = d_in + d_out;
        let chord = sub(next, prev); // the neighbour chord = the tangent direction
        let (fa, fb) = if sum > 1e-6 {
            (T * d_in / sum, T * d_out / sum)
        } else {
            (0.0, 0.0)
        };
        out.push([sub(cur, scale(chord, fa)), add(cur, scale(chord, fb))]);
    }
    out
}

/// Flatten the cubic Béziers defined by `anchors` + per-anchor `[in, out]` `handles` into a dense
/// polyline (cleared first; starts at `anchors[0]`). Each segment `anchors[i] → anchors[i+1]` is the
/// cubic `[anchors[i], handles[i].out, handles[i+1].in, anchors[i+1]]`, subdivided finer than the dab
/// spacing so the curve never facets — the SAME density rule as the Catmull-Rom flattener, so the painted
/// dabs match the on-screen guide. A length mismatch (`anchors != handles`) yields just the start point.
pub fn flatten_bezier(anchors: &[P], handles: &[[P; 2]], out: &mut Vec<P>) {
    out.clear();
    if anchors.is_empty() {
        return;
    }
    out.push(anchors[0]);
    if anchors.len() != handles.len() {
        return;
    }
    for i in 0..anchors.len() - 1 {
        let seg = [anchors[i], handles[i][1], handles[i + 1][0], anchors[i + 1]];
        let n = ((dist2(anchors[i], anchors[i + 1]).sqrt() / 3.0).ceil() as usize).clamp(1, 96);
        for k in 1..=n {
            out.push(bezier_at(&seg, k as f32 / n as f32));
        }
    }
}

#[cfg(test)]
mod tests;
