//! Pixel-edge contours (integer marching squares). Solid is on the LEFT (positive cross) of travel;
//! at a diagonal pinch the walk turns toward the solid, so diagonal pixels are separate (4-connected).

use crate::Mask;
use crate::geom::P;

const DIRS: [[i32; 2]; 4] = [[1, 0], [0, 1], [-1, 0], [0, -1]];

pub(crate) fn march(m: &Mask) -> Vec<Vec<P>> {
    let (w, h) = (i64::from(m.w), i64::from(m.h));
    let vw = usize::try_from(w + 1).unwrap_or(0);
    let vh = usize::try_from(h + 1).unwrap_or(0);
    let mut out_edges = vec![0u8; vw * vh];
    let idx =
        |x: i64, y: i64| usize::try_from(y).unwrap_or(0) * vw + usize::try_from(x).unwrap_or(0);
    for y in 0..h {
        for x in 0..w {
            if !m.get(x, y) {
                continue;
            }
            if !m.get(x, y - 1) {
                out_edges[idx(x, y)] |= 1;
            }
            if !m.get(x + 1, y) {
                out_edges[idx(x + 1, y)] |= 2;
            }
            if !m.get(x, y + 1) {
                out_edges[idx(x + 1, y + 1)] |= 4;
            }
            if !m.get(x - 1, y) {
                out_edges[idx(x, y + 1)] |= 8;
            }
        }
    }
    let mut rings = Vec::new();
    for vy in 0..=h {
        for vx in 0..=w {
            for d0 in 0..4usize {
                if out_edges[idx(vx, vy)] & (1 << d0) == 0 {
                    continue;
                }
                rings.push(walk(&mut out_edges, idx, [vx, vy], d0));
            }
        }
    }
    rings
}

fn walk(edges: &mut [u8], idx: impl Fn(i64, i64) -> usize, start: [i64; 2], d0: usize) -> Vec<P> {
    let mut pts: Vec<[i64; 2]> = Vec::new();
    let mut cur = start;
    let mut d = d0;
    loop {
        edges[idx(cur[0], cur[1])] &= !(1 << d);
        let nxt = [
            cur[0] + i64::from(DIRS[d][0]),
            cur[1] + i64::from(DIRS[d][1]),
        ];
        let bits = edges[idx(nxt[0], nxt[1])];
        let closing = |c: usize| nxt == start && c == d0;
        let Some(nd) = [(d + 1) % 4, d, (d + 3) % 4]
            .into_iter()
            .find(|c| bits & (1 << c) != 0 || closing(*c))
        else {
            break;
        };
        if nd != d {
            pts.push(nxt);
        }
        if closing(nd) {
            break;
        }
        cur = nxt;
        d = nd;
    }
    pts.into_iter()
        .map(|[x, y]| {
            [
                i32::try_from(x).unwrap_or(i32::MAX),
                i32::try_from(y).unwrap_or(i32::MAX),
            ]
        })
        .collect()
}
