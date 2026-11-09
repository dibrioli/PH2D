//! Outline → convex pieces: holes bridged to visible vertices, ear clipping with exact predicates,
//! Hertel–Mehlhorn, then a merge of neighbours whose hull stays within `detail` and covers no hole.

use crate::geom::{P, area2, cross, dist2, in_cone, ring_crosses, segs_conflict, segs_meet};
use crate::{Outline, Ring};

/// A simple-by-index polygon: outer ring with each hole spliced in by a two-way bridge.
fn bridge(outer: &Ring, holes: &[&Ring]) -> Vec<P> {
    let mut poly = outer.clone();
    let mut order: Vec<&Ring> = holes.to_vec();
    order.sort_by_key(|h| std::cmp::Reverse(h.iter().map(|p| (p[0], -p[1])).max()));
    for (hi, hole) in order.iter().enumerate() {
        let n = hole.len();
        let mut best: Option<(i64, usize, usize)> = None;
        for (m, &pm) in hole.iter().enumerate() {
            let (hp, hn) = (hole[(m + n - 1) % n], hole[(m + 1) % n]);
            for (k, &pk) in poly.iter().enumerate() {
                let d = dist2(pm, pk);
                if best.is_some_and(|(b, _, _)| d >= b) {
                    continue;
                }
                let pn = poly.len();
                let (kp, kn) = (poly[(k + pn - 1) % pn], poly[(k + 1) % pn]);
                if !in_cone(hp, pm, hn, pk) || !in_cone(kp, pk, kn, pm) {
                    continue;
                }
                let clear = |r: &[P]| {
                    (0..r.len()).all(|e| !segs_conflict(pm, pk, r[e], r[(e + 1) % r.len()]))
                };
                if clear(&poly) && order[hi..].iter().all(|h| clear(h)) {
                    best = Some((d, m, k));
                }
            }
        }
        let Some((_, m, k)) = best else { continue };
        let mut spliced: Vec<P> = poly[..=k].to_vec();
        spliced.extend((0..=n).map(|t| hole[(m + t) % n]));
        spliced.extend_from_slice(&poly[k..]);
        poly = spliced;
    }
    poly
}

fn is_ear(poly: &[P], live: &[usize], t: usize) -> bool {
    let n = live.len();
    let (a, b, c) = (
        poly[live[(t + n - 1) % n]],
        poly[live[t]],
        poly[live[(t + 1) % n]],
    );
    if cross(a, b, c) < 0 {
        return false;
    }
    let (ap, cn) = (poly[live[(t + n - 2) % n]], poly[live[(t + 2) % n]]);
    if n > 3 && (!in_cone(ap, a, b, c) || !in_cone(b, c, cn, a)) {
        return false;
    }
    (0..n).all(|e| {
        let (p, q) = (poly[live[e]], poly[live[(e + 1) % n]]);
        [p, q].iter().any(|x| *x == a || *x == c) || !segs_meet(a, c, p, q)
    })
}

/// Triangles (indices into the returned vertices) of each outer ring with its holes, zero-area dropped.
#[must_use]
pub fn triangulate(o: &Outline) -> Vec<(Vec<P>, Vec<[u32; 3]>)> {
    groups(o)
        .into_iter()
        .map(|(outer, holes)| {
            let poly = bridge(outer, &holes);
            let mut live: Vec<usize> = (0..poly.len()).collect();
            let mut tris = Vec::new();
            while live.len() > 3 {
                let Some(t) = (0..live.len()).find(|t| is_ear(&poly, &live, *t)) else {
                    break;
                };
                let n = live.len();
                let tri = [live[(t + n - 1) % n], live[t], live[(t + 1) % n]];
                if cross(poly[tri[0]], poly[tri[1]], poly[tri[2]]) > 0 {
                    tris.push(tri.map(|i| u32::try_from(i).unwrap_or(u32::MAX)));
                }
                live.remove(t);
            }
            if live.len() == 3 && cross(poly[live[0]], poly[live[1]], poly[live[2]]) > 0 {
                tris.push(
                    [live[0], live[1], live[2]].map(|i| u32::try_from(i).unwrap_or(u32::MAX)),
                );
            }
            (poly, tris)
        })
        .collect()
}

/// Each outer ring with the holes directly inside it.
fn groups(o: &Outline) -> Vec<(&Ring, Vec<&Ring>)> {
    let outers: Vec<&Ring> = o.rings.iter().filter(|r| area2(r) > 0).collect();
    let mut out: Vec<(&Ring, Vec<&Ring>)> = outers.iter().map(|r| (*r, Vec::new())).collect();
    for h in o.rings.iter().filter(|r| area2(r) < 0) {
        // A point just inside the solid next to the hole's first edge.
        let (a, b) = (h[0], h[1 % h.len()]);
        let probe = [
            (a[0] + b[0]) / 2 - (b[1] - a[1]).signum(),
            (a[1] + b[1]) / 2 + (b[0] - a[0]).signum(),
        ];
        if let Some(i) = (0..outers.len())
            .filter(|i| ring_crosses(outers[*i], probe))
            .min_by_key(|i| area2(outers[*i]))
        {
            out[i].1.push(h);
        }
    }
    out
}

/// Who owns each directed index edge. A directed edge lives in at most one polygon (indices are unique
/// even where the hole bridges repeat a point), so «the first later polygon holding the twin» is «the
/// owner, if it comes later» — the same answer as a scan, without the scan (W0: the scan was cubic).
#[derive(Default)]
struct EdgeOwner(std::collections::BTreeMap<(u32, u32), usize>);

impl EdgeOwner {
    fn claim(&mut self, poly: &[u32], owner: usize) {
        for e in 0..poly.len() {
            self.0.insert((poly[e], poly[(e + 1) % poly.len()]), owner);
        }
    }
}

/// Hertel–Mehlhorn over index triangles: drop a shared diagonal whenever both junctions stay convex.
#[must_use]
pub fn hertel_mehlhorn(v: &[P], tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    let mut polys: Vec<Vec<u32>> = tris.iter().map(|t| t.to_vec()).collect();
    let mut owner = EdgeOwner::default();
    for (k, p) in polys.iter().enumerate() {
        owner.claim(p, k);
    }
    let pt = |i: u32| v[i as usize];
    let mut i = 0;
    while i < polys.len() {
        let mut e = 0;
        while e < polys[i].len() {
            let n1 = polys[i].len();
            let (a, b) = (polys[i][e], polys[i][(e + 1) % n1]);
            let found = owner
                .0
                .get(&(b, a))
                .copied()
                .filter(|j| *j > i)
                .and_then(|j| {
                    let pj = &polys[j];
                    (0..pj.len())
                        .find(|f| pj[*f] == b && pj[(f + 1) % pj.len()] == a)
                        .map(|f| (j, f))
                });
            let Some((j, f)) = found else {
                e += 1;
                continue;
            };
            let (p1, p2) = (&polys[i], &polys[j]);
            let n2 = p2.len();
            // Junction at a: prev in p1, a, next after a in p2. Junction at b: prev of b in p2, b, next in p1.
            let ok_a = cross(pt(p1[(e + n1 - 1) % n1]), pt(a), pt(p2[(f + 2) % n2])) >= 0;
            let ok_b = cross(pt(p2[(f + n2 - 1) % n2]), pt(b), pt(p1[(e + 2) % n1])) >= 0;
            if !(ok_a && ok_b) {
                e += 1;
                continue;
            }
            let mut merged: Vec<u32> = (0..n1).map(|t| p1[(e + 1 + t) % n1]).collect(); // b .. a
            merged.extend((2..n2).map(|t| p2[(f + t) % n2])); // after a .. before b
            owner.0.remove(&(a, b));
            owner.0.remove(&(b, a));
            owner.claim(&merged, i);
            polys[i] = merged;
            polys.swap_remove(j);
            if j < polys.len() {
                owner.claim(&polys[j], j);
            }
            e = 0;
        }
        i += 1;
    }
    polys
}

fn hull(mut pts: Vec<P>) -> Vec<P> {
    pts.sort_unstable();
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let mut h: Vec<P> = Vec::new();
    for pass in 0..2 {
        let start = h.len();
        let it: Box<dyn Iterator<Item = &P>> = if pass == 0 {
            Box::new(pts.iter())
        } else {
            Box::new(pts.iter().rev())
        };
        for &p in it {
            while h.len() >= start + 2 && cross(h[h.len() - 2], h[h.len() - 1], p) <= 0 {
                h.pop();
            }
            h.push(p);
        }
        h.pop();
    }
    h
}

fn shares_edge(a: &[P], b: &[P]) -> bool {
    (0..a.len()).any(|i| {
        let (p, q) = (a[i], a[(i + 1) % a.len()]);
        (0..b.len()).any(|k| b[k] == q && b[(k + 1) % b.len()] == p)
    })
}

/// Every vertex of `u` is within `eps` of the hull boundary, and no hole vertex is strictly inside.
fn merge_ok(h: &[P], u: &[P], holes: &[&Ring], eps: i64) -> bool {
    let n = h.len();
    let near = |p: P| {
        (0..n).any(|e| {
            let c = i128::from(cross(h[e], h[(e + 1) % n], p));
            c * c <= i128::from(eps).pow(2) * i128::from(dist2(h[e], h[(e + 1) % n]))
        })
    };
    let strictly_in = |p: P| (0..n).all(|e| cross(h[e], h[(e + 1) % n], p) > 0);
    u.iter().all(|p| near(*p)) && holes.iter().all(|r| r.iter().all(|p| !strictly_in(*p)))
}

/// The merged hull of `i` and `j`, when they share an edge and the merge is allowed.
fn try_merge(pieces: &[Vec<P>], i: usize, j: usize, holes: &[&Ring], eps: i64) -> Option<Vec<P>> {
    if !shares_edge(&pieces[i], &pieces[j]) {
        return None;
    }
    let both: Vec<P> = pieces[i].iter().chain(&pieces[j]).copied().collect();
    let h = hull(both.clone());
    merge_ok(&h, &both, holes, eps).then_some(h)
}

/// Merge the lexicographically FIRST allowed pair `(i, j)` until none is left — the same sequence as
/// rescanning from `(0, 1)` after every merge, without the rescan: after a merge only piece `i` is new,
/// so the rows before it can only gain the pair with column `i`, and the scan resumes at row `i`.
fn merge_neighbours(pieces: &mut Vec<Vec<P>>, holes: &[&Ring], eps: i64) {
    let mut changed = 0;
    loop {
        let column = (0..changed)
            .find_map(|r| try_merge(pieces, r, changed, holes, eps).map(|h| (r, changed, h)));
        let hit = column.or_else(|| {
            (changed..pieces.len()).find_map(|i| {
                (i + 1..pieces.len())
                    .find_map(|j| try_merge(pieces, i, j, holes, eps).map(|h| (i, j, h)))
            })
        });
        let Some((i, j, h)) = hit else { break };
        pieces[i] = h;
        pieces.remove(j);
        changed = i;
    }
}

/// Convex pieces in Q4, outer orientation, deterministic order.
#[must_use]
pub fn convex_pieces(o: &Outline, detail_q4: u16) -> Vec<Vec<[i32; 2]>> {
    let eps = i64::from(detail_q4);
    let holes: Vec<&Ring> = o.rings.iter().filter(|r| area2(r) < 0).collect();
    let mut out = Vec::new();
    for (v, tris) in triangulate(o) {
        let mut pieces: Vec<Vec<P>> = hertel_mehlhorn(&v, &tris)
            .into_iter()
            .map(|p| p.into_iter().map(|i| v[i as usize]).collect())
            .collect();
        merge_neighbours(&mut pieces, &holes, eps);
        out.extend(pieces);
    }
    for p in &mut out {
        crate::geom::rotate_to_smallest(p);
    }
    out.sort();
    out
}
