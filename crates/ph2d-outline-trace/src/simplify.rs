//! Outward Douglas–Peucker (owner 09/11: «Detail = the most the outline may stand OUT of the art»).
//! A chord `i → j` over the staircase stands when the strip holding every vertex it skips is at most
//! `detail` wide; its edge is the chord's line pushed to the EMPTY side until no skipped vertex is left
//! out. Each kept vertex becomes the meeting point of its two pushed lines (exact `i128`, rounded to
//! Q4); a meeting point farther than `detail` from its staircase vertex (sharp turn, parallel lines)
//! gets both chords split. Edges that meet any other edge (except at a shared endpoint) get their split
//! vertex back, until none do (the staircase itself never conflicts).

use crate::geom::{P, cross, dist2, segs_conflict};

/// A kept chord: `to` may be `n`, meaning `0`; `push` is how far its line moves out, in cross units
/// (`push / |b - a|` Q4).
#[derive(Clone, Copy)]
struct Chord {
    from: usize,
    to: usize,
    push: i64,
}

/// `Ok(push)` when the chord `i → j` stands at `eps`, else `Err(k)`: the skipped vertex farthest from it.
fn judge(r: &[P], i: usize, j: usize, eps: i64) -> Result<i64, usize> {
    if j <= i + 1 {
        return Ok(0);
    }
    let n = r.len();
    let (a, b) = (r[i], r[j % n]);
    let len2 = dist2(a, b);
    if len2 == 0 {
        return Err(farthest(r, i, j));
    }
    let (mut lo, mut hi) = (0i64, 0i64);
    for &pk in &r[i + 1..j] {
        let c = cross(a, b, pk);
        lo = lo.min(c);
        hi = hi.max(c);
    }
    let width = i128::from(hi - lo);
    if width * width > i128::from(eps).pow(2) * i128::from(len2) {
        return Err(farthest(r, i, j));
    }
    Ok(-lo)
}

/// The skipped vertex farthest from the chord `i → j` (from `r[i]` when the chord is a point); ties to
/// the lowest index.
fn farthest(r: &[P], i: usize, j: usize) -> usize {
    let n = r.len();
    let (a, b) = (r[i], r[j % n]);
    let len2 = dist2(a, b);
    (i + 1..j)
        .max_by_key(|k| {
            let d = if len2 == 0 {
                dist2(a, r[*k])
            } else {
                cross(a, b, r[*k]).abs()
            };
            (d, usize::MAX - *k)
        })
        .unwrap_or(i + 1)
}

fn dp(r: &[P], eps: i64) -> Vec<bool> {
    let n = r.len();
    let mut keep = vec![false; n];
    keep[0] = true;
    if n < 3 {
        keep.iter_mut().for_each(|k| *k = true);
        return keep;
    }
    let far = (1..n)
        .max_by_key(|k| (dist2(r[0], r[*k]), usize::MAX - *k))
        .unwrap_or(1);
    keep[far] = true;
    let mut stack = vec![(0, far), (far, n)];
    while let Some((i, j)) = stack.pop() {
        if let Err(k) = judge(r, i, j, eps) {
            keep[k] = true;
            stack.push((i, k));
            stack.push((k, j));
        }
    }
    keep
}

/// The pushed line of `c` as `α x + β y = γ`.
fn line(r: &[P], c: Chord) -> [i128; 3] {
    let (a, b) = (r[c.from], r[c.to % r.len()]);
    let (dx, dy) = (
        i128::from(b[0]) - i128::from(a[0]),
        i128::from(b[1]) - i128::from(a[1]),
    );
    // cross(a, b, x) = dx (y - ay) - dy (x - ax) = -push.
    let g = dx * i128::from(a[1]) - dy * i128::from(a[0]) - i128::from(c.push);
    [-dy, dx, g]
}

/// Round-to-nearest of `p / q` (`q > 0`), ties up: integer, the same on every machine.
fn div_round(p: i128, q: i128) -> i128 {
    (2 * p + q).div_euclid(2 * q)
}

/// The vertex that the chords `prev` (ending at `v`) and `next` (starting at `v`) put at `r[v]`, or
/// `None` when it would stand farther than `eps` from `r[v]`.
fn corner(r: &[P], v: usize, prev: Chord, next: Chord, eps: i64) -> Option<P> {
    if prev.push == 0 && next.push == 0 {
        return Some(r[v]);
    }
    let [a1, b1, g1] = line(r, prev);
    let [a2, b2, g2] = line(r, next);
    let mut det = a1 * b2 - a2 * b1;
    let (mut px, mut py) = (g1 * b2 - g2 * b1, a1 * g2 - a2 * g1);
    if det == 0 {
        return None;
    }
    if det < 0 {
        (det, px, py) = (-det, -px, -py);
    }
    let p = [
        i32::try_from(div_round(px, det)).ok()?,
        i32::try_from(div_round(py, det)).ok()?,
    ];
    (dist2(p, r[v]) <= eps * eps).then_some(p)
}

/// The chords (and their neighbours) whose run leaves a pixel centre of the art outside the polygons:
/// each staircase edge is checked at every pixel centre beside it on the solid side. Where two pushed
/// lines meet behind a sharp tip, a run can end up behind its own edge; this puts the vertex back.
fn uncovered(rings: &[Vec<P>], chords: &[(usize, Chord)], polys: &[Vec<P>]) -> Vec<(usize, Chord)> {
    let half = crate::Q4 / 2;
    let inside = |p: P| {
        polys
            .iter()
            .filter(|r| crate::geom::ring_crosses(r, p))
            .count()
            % 2
            == 1
    };
    let mut out = Vec::new();
    let mut base = 0;
    for (ri, r) in rings.iter().enumerate() {
        let own = &chords[base..];
        let m = own.iter().take_while(|c| c.0 == ri).count();
        base += m;
        for (t, &(_, c)) in own[..m].iter().enumerate() {
            let cut = (c.from..c.to).any(|e| {
                let (a, b) = (r[e], r[(e + 1) % r.len()]);
                let (dx, dy) = ((b[0] - a[0]).signum(), (b[1] - a[1]).signum());
                let steps = (b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / crate::Q4;
                // The solid side of `a → b` (`cross > 0`): normal (-dy, dx).
                (0..steps).any(|s| {
                    let along = s * crate::Q4 + half;
                    !inside([a[0] + dx * along - dy * half, a[1] + dy * along + dx * half])
                })
            });
            if cut {
                out.push((ri, own[(t + m - 1) % m].1));
                out.push((ri, c));
                out.push((ri, own[(t + 1) % m].1));
            }
        }
    }
    out
}

pub(crate) fn simplify(rings: &[Vec<P>], eps: i64) -> Vec<Vec<P>> {
    let mut keeps: Vec<Vec<bool>> = rings.iter().map(|r| dp(r, eps)).collect();
    loop {
        // Per ring: its chords and the polygon they make.
        let mut chords: Vec<(usize, Chord)> = Vec::new();
        let mut polys: Vec<Vec<P>> = Vec::with_capacity(rings.len());
        let mut force: Vec<(usize, Chord)> = Vec::new();
        for (ri, k) in keeps.iter().enumerate() {
            let r = &rings[ri];
            let idx: Vec<usize> = (0..k.len()).filter(|i| k[*i]).collect();
            let own: Vec<Chord> = idx
                .iter()
                .enumerate()
                .map(|(t, &a)| {
                    let b = idx.get(t + 1).copied().unwrap_or(k.len());
                    // `dp` and every forced split leave only chords that stand.
                    let push = judge(r, a, b, eps).unwrap_or(0);
                    Chord {
                        from: a,
                        to: b,
                        push,
                    }
                })
                .collect();
            let m = own.len();
            let mut poly = Vec::with_capacity(m);
            for t in 0..m {
                let (prev, next) = (own[(t + m - 1) % m], own[t]);
                match corner(r, next.from, prev, next, eps) {
                    Some(p) => poly.push(p),
                    None => {
                        force.push((ri, prev));
                        force.push((ri, next));
                        poly.push(r[next.from]);
                    }
                }
            }
            chords.extend(own.into_iter().map(|c| (ri, c)));
            polys.push(poly);
        }
        if force.is_empty() {
            // Edge `t` of ring `ri` runs from `polys[ri][t]` to the next vertex: one per chord, in order.
            let mut edges: Vec<(P, P)> = Vec::with_capacity(chords.len());
            for p in &polys {
                for t in 0..p.len() {
                    edges.push((p[t], p[(t + 1) % p.len()]));
                }
            }
            let mut bad = vec![false; edges.len()];
            for x in 0..edges.len() {
                for y in x + 1..edges.len() {
                    let ((a, b), (c, d)) = (edges[x], edges[y]);
                    if segs_conflict(a, b, c, d) {
                        bad[x] = true;
                        bad[y] = true;
                    }
                }
            }
            force.extend(
                chords
                    .iter()
                    .zip(&bad)
                    .filter(|(_, b)| **b)
                    .map(|(c, _)| *c),
            );
        }
        if force.is_empty() {
            force = uncovered(rings, &chords, &polys);
            if force.is_empty() {
                return polys;
            }
        }
        let mut changed = false;
        for (ri, c) in force {
            if c.to > c.from + 1 {
                let k = farthest(&rings[ri], c.from, c.to);
                changed |= !keeps[ri][k];
                keeps[ri][k] = true;
            }
        }
        if !changed {
            // Only staircase chords conflict now, which cannot happen; never loop.
            return polys;
        }
    }
}
