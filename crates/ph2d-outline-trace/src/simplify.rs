//! One-sided Douglas–Peucker: a chord is kept only if every original vertex it skips lies on the
//! solid side (or on it) within `detail`. Chords that conflict with any other chord get their split
//! vertex back, until none do (the staircase itself never conflicts).

use crate::geom::{P, cross, dist2, segs_conflict};

/// Index of the vertex that must come back for the chord `i → j` (`j` may be `n`, meaning `0`), or
/// `None` when the chord is legal at `eps`.
fn split(r: &[P], i: usize, j: usize, eps: i64, force: bool) -> Option<usize> {
    if j <= i + 1 {
        return None;
    }
    let n = r.len();
    let (a, b) = (r[i], r[j % n]);
    let len2 = dist2(a, b);
    if len2 == 0 {
        return (i + 1..j).max_by_key(|k| (dist2(a, r[*k]), usize::MAX - *k));
    }
    let mut worst_in: Option<(i64, usize)> = None;
    let mut far: (i64, usize) = (-1, i + 1);
    for (k, &pk) in r.iter().enumerate().take(j).skip(i + 1) {
        let c = cross(a, b, pk);
        if c < 0 && worst_in.is_none_or(|(w, _)| c < w) {
            worst_in = Some((c, k));
        }
        if c > far.0 {
            far = (c, k);
        }
    }
    if let Some((_, k)) = worst_in {
        return Some(k);
    }
    let over = i128::from(far.0).pow(2) > i128::from(eps).pow(2) * i128::from(len2);
    (over || force).then_some(far.1)
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
        if let Some(k) = split(r, i, j, eps, false) {
            keep[k] = true;
            stack.push((i, k));
            stack.push((k, j));
        }
    }
    keep
}

pub(crate) fn simplify(rings: &[Vec<P>], eps: i64) -> Vec<Vec<P>> {
    let mut keeps: Vec<Vec<bool>> = rings.iter().map(|r| dp(r, eps)).collect();
    loop {
        // (ring, from, to) with `to` possibly `n`.
        let mut chords: Vec<(usize, usize, usize)> = Vec::new();
        for (ri, k) in keeps.iter().enumerate() {
            let idx: Vec<usize> = (0..k.len()).filter(|i| k[*i]).collect();
            for (t, &a) in idx.iter().enumerate() {
                let b = idx.get(t + 1).copied().unwrap_or(k.len());
                chords.push((ri, a, b));
            }
        }
        let pt = |&(ri, a, b): &(usize, usize, usize)| {
            let r = &rings[ri];
            (r[a], r[b % r.len()])
        };
        let mut bad = vec![false; chords.len()];
        for x in 0..chords.len() {
            for y in x + 1..chords.len() {
                let (a, b) = pt(&chords[x]);
                let (c, d) = pt(&chords[y]);
                if segs_conflict(a, b, c, d) {
                    bad[x] = true;
                    bad[y] = true;
                }
            }
        }
        let mut changed = false;
        for (c, _) in chords.iter().zip(&bad).filter(|(_, b)| **b) {
            if let Some(k) = split(&rings[c.0], c.1, c.2, eps, true) {
                keeps[c.0][k] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    rings
        .iter()
        .zip(&keeps)
        .map(|(r, k)| {
            r.iter()
                .zip(k)
                .filter(|(_, k)| **k)
                .map(|(p, _)| *p)
                .collect()
        })
        .collect()
}
