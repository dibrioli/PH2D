//! Exact integer predicates. Coordinates are Q4 ≤ 2^20, so every product fits `i64`.

pub(crate) type P = [i32; 2];

pub(crate) fn sub(a: P, b: P) -> [i64; 2] {
    [
        i64::from(a[0]) - i64::from(b[0]),
        i64::from(a[1]) - i64::from(b[1]),
    ]
}

/// `(a - o) × (b - o)`; positive = `b` on the solid (left) side of `o → a`.
pub(crate) fn cross(o: P, a: P, b: P) -> i64 {
    let u = sub(a, o);
    let v = sub(b, o);
    u[0] * v[1] - u[1] * v[0]
}

pub(crate) fn dist2(a: P, b: P) -> i64 {
    let d = sub(a, b);
    d[0] * d[0] + d[1] * d[1]
}

pub(crate) fn area2(r: &[P]) -> i64 {
    let n = r.len();
    (0..n)
        .map(|i| {
            let a = r[i];
            let b = r[(i + 1) % n];
            i64::from(a[0]) * i64::from(b[1]) - i64::from(b[0]) * i64::from(a[1])
        })
        .sum()
}

/// Odd number of crossings of the ray `+x` from `p` with ring `r` (half-open in y).
pub(crate) fn ring_crosses(r: &[P], p: P) -> bool {
    let n = r.len();
    (0..n)
        .filter(|i| edge_crosses(r[*i], r[(i + 1) % n], p))
        .count()
        % 2
        == 1
}

/// The ray `+x` from `p` crosses the edge `a → b` (half-open in y: `min y ≤ p.y < max y`).
pub(crate) fn edge_crosses(a: P, b: P, p: P) -> bool {
    if (a[1] > p[1]) == (b[1] > p[1]) {
        return false;
    }
    // p.x < a.x + (p.y - a.y)(b.x - a.x)/(b.y - a.y), sign-corrected.
    let dy = i64::from(b[1]) - i64::from(a[1]);
    let lhs = (i64::from(p[0]) - i64::from(a[0])) * dy;
    let rhs = (i64::from(p[1]) - i64::from(a[1])) * (i64::from(b[0]) - i64::from(a[0]));
    (dy > 0 && lhs < rhs) || (dy < 0 && lhs > rhs)
}

pub(crate) fn rotate_to_smallest(r: &mut [P]) {
    if let Some((i, _)) = r.iter().enumerate().min_by_key(|(_, p)| **p) {
        r.rotate_left(i);
    }
}

fn on_seg(a: P, b: P, p: P) -> bool {
    p[0] >= a[0].min(b[0])
        && p[0] <= a[0].max(b[0])
        && p[1] >= a[1].min(b[1])
        && p[1] <= a[1].max(b[1])
}

/// Closed segment intersection (touching counts).
pub(crate) fn segs_meet(a: P, b: P, c: P, d: P) -> bool {
    let o1 = cross(a, b, c).signum();
    let o2 = cross(a, b, d).signum();
    let o3 = cross(c, d, a).signum();
    let o4 = cross(c, d, b).signum();
    if o1 * o2 < 0 && o3 * o4 < 0 {
        return true;
    }
    (o1 == 0 && on_seg(a, b, c))
        || (o2 == 0 && on_seg(a, b, d))
        || (o3 == 0 && on_seg(c, d, a))
        || (o4 == 0 && on_seg(c, d, b))
}

/// Two segments are in conflict: they meet anywhere except at a shared endpoint, or they share an
/// endpoint and fold back over each other.
pub(crate) fn segs_conflict(a: P, b: P, c: P, d: P) -> bool {
    let shared = [(a, c, b, d), (a, d, b, c), (b, c, a, d), (b, d, a, c)]
        .into_iter()
        .find(|(x, y, _, _)| x == y);
    match shared {
        None => segs_meet(a, b, c, d),
        Some((s, _, u, v)) => {
            if u == v {
                return true;
            }
            let du = sub(u, s);
            let dv = sub(v, s);
            cross(s, u, v) == 0 && du[0] * dv[0] + du[1] * dv[1] > 0
        }
    }
}

/// O'Rourke's `InCone`: `b` seen from `a` lies strictly inside the solid angle `a0 → a → a1`.
pub(crate) fn in_cone(a0: P, a: P, a1: P, b: P) -> bool {
    if cross(a0, a, a1) >= 0 {
        cross(a, b, a0) > 0 && cross(b, a, a1) > 0
    } else {
        !(cross(a, b, a1) >= 0 && cross(b, a, a0) >= 0)
    }
}
