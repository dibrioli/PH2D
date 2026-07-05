//! Contour tracing for the Selection **Edit** mode (ADR-0103 Amendment 1). Turns the selection mask into
//! an editable boundary polyline: Moore-neighbour trace of the 50%-coverage outer contour, then
//! Douglas–Peucker simplification to a handful of anchors that seed a `CurveState` (handles/gizmos +
//! Offset reuse the Shape editors). Pure geometry — no `self`, transcendental-free (HR-5).

/// A texel is INSIDE the selection when its coverage is ≥ the half contour.
#[inline]
fn inside(mask: &[u8], w: usize, h: usize, x: i32, y: i32) -> bool {
    x >= 0
        && y >= 0
        && (x as usize) < w
        && (y as usize) < h
        && mask[(y as usize) * w + x as usize] >= 128
}

/// Moore-neighbour offsets in CLOCKWISE order starting at West (index 0): W · NW · N · NE · E · SE · S · SW.
const OFF: [(i32, i32); 8] = [
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
];

/// Trace the outer contour of EVERY connected component of `mask` (multi-blob) — used by the stroke boolean
/// composite, where separate Add/Remove regions must each become their own stroked contour (Enio 2026-07-04).
/// Each component is flood-filled out of a working copy, then its outer boundary is traced + simplified.
pub(super) fn trace_all_contours(mask: &[u8], w: usize, h: usize) -> Vec<Vec<[f32; 2]>> {
    if w == 0 || h == 0 || mask.len() != w * h {
        return Vec::new();
    }
    let mut work: Vec<u8> = mask.to_vec();
    let mut out = Vec::new();
    let guard = w * h + 1; // at most one component per pixel
    for _ in 0..guard {
        // First inside texel in row-major order (also the trace start), or done.
        let Some(start) = (0..w * h).find(|&i| work[i] >= 128) else {
            break;
        };
        // Flood-fill this component into `comp` + clear it from `work` so the next pass finds the next blob.
        let mut comp = vec![0u8; w * h];
        let mut stack = vec![start];
        work[start] = 0;
        while let Some(i) = stack.pop() {
            comp[i] = 255;
            let (x, y) = (i % w, i / w);
            let push = |xx: usize, yy: usize, st: &mut Vec<usize>, wk: &mut [u8]| {
                let j = yy * w + xx;
                if wk[j] >= 128 {
                    wk[j] = 0;
                    st.push(j);
                }
            };
            if x + 1 < w {
                push(x + 1, y, &mut stack, &mut work);
            }
            if x > 0 {
                push(x - 1, y, &mut stack, &mut work);
            }
            if y + 1 < h {
                push(x, y + 1, &mut stack, &mut work);
            }
            if y > 0 {
                push(x, y - 1, &mut stack, &mut work);
            }
        }
        let raw = trace_contour_raw(&comp, w, h); // RAW dense boundary (not the DP-simplified polygon)
        if raw.len() >= 3 {
            // Smooth the ±0.5px pixel staircase so the stroked contour reads as regular as a direct ellipse
            // outline (Enio 2026-07-04: Add/Remove was "discretamente irregular"), then CLOSE the loop so the
            // stroke has no gap at the seam.
            let mut c = smooth_closed(&raw, 2);
            c.push(c[0]);
            out.push(c);
        }
    }
    out
}

/// A gentle closed-polyline smoother (3-point moving average, wrap-around, `passes` times) — removes the
/// 1px raster staircase of a traced contour while barely moving it, so a brush stroked along it is smooth.
fn smooth_closed(pts: &[[f32; 2]], passes: usize) -> Vec<[f32; 2]> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let mut cur = pts.to_vec();
    for _ in 0..passes {
        let mut next = cur.clone();
        for i in 0..n {
            let a = cur[(i + n - 1) % n];
            let b = cur[i];
            let d = cur[(i + 1) % n];
            next[i] = [(a[0] + b[0] + d[0]) / 3.0, (a[1] + b[1] + d[1]) / 3.0];
        }
        cur = next;
    }
    cur
}

/// Trace the outer contour of the selection mask (Moore-neighbour, clockwise), then simplify it. Returns a
/// closed polyline of image-px points (pixel centres), or empty if nothing is selected. Only the outer
/// boundary of the first (row-major) component is traced — good enough to seed an editable boundary.
pub(super) fn trace_selection_contour(mask: &[u8], w: usize, h: usize) -> Vec<[f32; 2]> {
    // Simplify: a 1.5px tolerance turns the pixel staircase into clean anchors, capped so a huge blob can't
    // seed thousands of curve points. (The stroke boolean uses the RAW dense contour instead — no polygonal
    // straight segments — via `trace_contour_raw`.)
    simplify_closed(&trace_contour_raw(mask, w, h), 1.5, 256)
}

/// The RAW pixel-centre outer boundary of the first (row-major) mask component (Moore-neighbour, clockwise),
/// NOT simplified — dense, so a brush stroked along it reads SMOOTH (no visible straight segments). Empty
/// when nothing is inside.
fn trace_contour_raw(mask: &[u8], w: usize, h: usize) -> Vec<[f32; 2]> {
    if w == 0 || h == 0 || mask.len() != w * h {
        return Vec::new();
    }
    // Start = the first inside texel in row-major order; the texel to its left is therefore outside, so the
    // initial backtrack is due West.
    let mut start: Option<(i32, i32)> = None;
    'scan: for y in 0..h {
        for x in 0..w {
            if mask[y * w + x] >= 128 {
                start = Some((x as i32, y as i32));
                break 'scan;
            }
        }
    }
    let Some(start) = start else {
        return Vec::new();
    };
    let mut boundary: Vec<(i32, i32)> = vec![start];
    let mut current = start;
    let mut backtrack = (start.0 - 1, start.1); // West of start (outside)
    let cap = w * h + 8; // safety bound (a contour can't be longer than the pixel count)
    loop {
        let b_off = (backtrack.0 - current.0, backtrack.1 - current.1);
        let b_idx = OFF.iter().position(|&o| o == b_off).unwrap_or(0);
        let mut found: Option<usize> = None;
        for k in 1..=8 {
            let idx = (b_idx + k) % 8;
            let px = (current.0 + OFF[idx].0, current.1 + OFF[idx].1);
            if inside(mask, w, h, px.0, px.1) {
                found = Some(idx);
                break;
            }
        }
        let Some(idx) = found else {
            break; // isolated texel — no contour
        };
        let prev = (idx + 7) % 8;
        backtrack = (current.0 + OFF[prev].0, current.1 + OFF[prev].1);
        current = (current.0 + OFF[idx].0, current.1 + OFF[idx].1);
        if current == start {
            break;
        }
        boundary.push(current);
        if boundary.len() > cap {
            break;
        }
    }
    boundary
        .into_iter()
        .map(|(x, y)| [x as f32 + 0.5, y as f32 + 0.5])
        .collect()
}

/// Douglas–Peucker simplification of a CLOSED polyline: split at the two mutually-farthest points, simplify
/// each half around the ring, and clamp to `max_pts` by raising the tolerance until it fits. `tol` is a
/// perpendicular distance (px); comparisons are done on SQUARED distances (no `sqrt`, HR-5-safe). Shared with
/// the selection-curve **Simplify** (a closed-loop-correct reducer the Schneider fit can't be — Enio 2026-07-05).
pub(super) fn simplify_closed(pts: &[[f32; 2]], tol: f32, max_pts: usize) -> Vec<[f32; 2]> {
    if pts.len() < 4 {
        return pts.to_vec();
    }
    // The two DP anchors that split the loop: pts[0] and the point farthest from it.
    let a = 0;
    let b = farthest_from(pts, a);
    let run = |t: f32| {
        let mut out = Vec::new();
        dp(pts, a, b, t * t, &mut out); // a → b, pushes a … (not b)
        dp(pts, b, a, t * t, &mut out); // b → a (wrap), pushes b … (not a) → closed loop
        out
    };
    let mut t = tol;
    let mut result = run(t);
    while result.len() > max_pts && result.len() >= 4 {
        t *= 1.6;
        result = run(t);
    }
    result
}

/// Index of the point farthest (squared distance) from `pts[i]`.
fn farthest_from(pts: &[[f32; 2]], i: usize) -> usize {
    let p = pts[i];
    let mut best = i;
    let mut best_d = -1.0f32;
    for (j, q) in pts.iter().enumerate() {
        let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2);
        if d > best_d {
            best_d = d;
            best = j;
        }
    }
    best
}

/// Recursive Douglas–Peucker over the ring segment from index `a` to index `b` (walking FORWARD with
/// wrap-around). `tol2` is the squared perpendicular tolerance. Pushes `pts[a]` + retained interior points;
/// never pushes `pts[b]` (the caller's next segment does).
fn dp(pts: &[[f32; 2]], a: usize, b: usize, tol2: f32, out: &mut Vec<[f32; 2]>) {
    let n = pts.len();
    let mut idx = (a + 1) % n;
    let mut far: Option<usize> = None;
    let mut far_d = tol2;
    while idx != b {
        let d = point_seg_dist2(pts[idx], pts[a], pts[b]);
        if d > far_d {
            far_d = d;
            far = Some(idx);
        }
        idx = (idx + 1) % n;
    }
    match far {
        Some(m) => {
            dp(pts, a, m, tol2, out);
            dp(pts, m, b, tol2, out);
        }
        None => out.push(pts[a]),
    }
}

/// SQUARED perpendicular distance from `p` to the segment `a`-`b` (degenerates to squared point distance
/// when `a == b`).
fn point_seg_dist2(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    if len2 <= f32::EPSILON {
        return (p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2);
    }
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0);
    let (px, py) = (a[0] + t * dx, a[1] + t * dy);
    (p[0] - px).powi(2) + (p[1] - py).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w*h` mask with a filled rectangle `[x0,x1) × [y0,y1)` set to 255.
    fn rect_mask(w: usize, h: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<u8> {
        let mut m = vec![0u8; w * h];
        for y in y0..y1 {
            for x in x0..x1 {
                m[y * w + x] = 255;
            }
        }
        m
    }

    #[test]
    fn empty_mask_traces_nothing() {
        assert!(trace_selection_contour(&[0u8; 64], 8, 8).is_empty());
    }

    #[test]
    fn rectangle_traces_to_roughly_four_corners() {
        // A 10×10 filled square inside a 32×32 canvas → the simplified contour is a small ring near the
        // 4 corners, tightly bounded to the square.
        let m = rect_mask(32, 32, 8, 8, 18, 18);
        let poly = trace_selection_contour(&m, 32, 32);
        assert!(
            (4..=12).contains(&poly.len()),
            "a rectangle simplifies to a handful of anchors, got {}",
            poly.len()
        );
        for [x, y] in &poly {
            assert!(
                (7.0..=18.5).contains(x) && (7.0..=18.5).contains(y),
                "contour point ({x},{y}) hugs the square boundary"
            );
        }
        // The bounding box of the contour spans the square (each side is represented).
        let (mut minx, mut maxx, mut miny, mut maxy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for [x, y] in &poly {
            minx = minx.min(*x);
            maxx = maxx.max(*x);
            miny = miny.min(*y);
            maxy = maxy.max(*y);
        }
        assert!(
            minx < 9.0 && maxx > 16.0 && miny < 9.0 && maxy > 16.0,
            "contour spans the square"
        );
    }
}
