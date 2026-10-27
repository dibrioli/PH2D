//! ⭐ **Flakes** (Óleo Real W3 K5, `docs/Painter/Pinceis/W3_K5.md`): one ANGULAR, elongated flake per lattice cell —
//! a convex quadrilateral `2–3 : 1` at a random angle, its corners jittered off the rectangle. `0` inside a flake,
//! `1` outside — or, with **Invert**, the flakes are the PAINT and the angular gaps between them the paper (the Rebelle
//! Dabber 3: paint chips with white gaps), under the Subtract grain law.
//! Voronoi / Stucci / Musgrave were measured and refused (`W3_K3b_cerda.md`): a grid of square holes, sponge, rare
//! dots. Transcendental-free (HR-5): the angle is a normalised hashed vector, not `sin`/`cos`.

use super::math::{hash2, ifloor};

/// The flake's half-length is `MIN + RANGE · Size` cells; the search reaches `REACH` cells, so `MIN + RANGE` stays
/// under it (a flake never pokes past the neighbourhood that is searched).
const HALF_MIN: f32 = 0.2;
const HALF_RANGE: f32 = 0.75;
const REACH: i32 = 1;
/// The edge ramp, in cells — the analytic antialias of the polygon (a hard step aliases at small Sizes).
const EDGE: f32 = 0.025;

/// The `s`-th independent hash of cell `(ix, iy)`.
fn h(ix: i32, iy: i32, s: i32) -> f32 {
    hash2(
        ix.wrapping_add(s.wrapping_mul(1013)),
        iy.wrapping_sub(s.wrapping_mul(7919)),
    )
}

/// `k` = `[Size, Elongation, Irregularity, Invert]` (the slots after Contrast / Brightness).
pub(super) fn flakes(u: f32, v: f32, k: &[f32]) -> f32 {
    let f = Forma::de(k);
    let (bx, by) = (ifloor(u), ifloor(v));
    let mut cover = 0.0f32;
    for dy in -REACH..=REACH {
        for dx in -REACH..=REACH {
            cover = cover.max(f.cobre(bx + dx, by + dy, u, v));
            if cover >= 1.0 {
                break;
            }
        }
    }
    if k.get(3).is_some_and(|&i| i > 0.5) {
        cover
    } else {
        1.0 - cover
    }
}

/// The knobs, decoded.
struct Forma {
    size: f32,
    elong: f32,
    irr: f32,
}

impl Forma {
    fn de(k: &[f32]) -> Self {
        let g = |i: usize| k.get(i).copied().unwrap_or(0.5).clamp(0.0, 1.0);
        Self {
            size: g(0),
            elong: 1.0 + 3.0 * g(1),
            irr: 0.3 * g(2),
        }
    }

    /// How much the flake of cell `(ix, iy)` covers `(u, v)`, `0..1` (the edge ramp is [`EDGE`]).
    fn cobre(&self, ix: i32, iy: i32, u: f32, v: f32) -> f32 {
        let c = [ix as f32 + h(ix, iy, 1), iy as f32 + h(ix, iy, 2)];
        // per-flake length in `[0.7, 1] ·` the knob's, so the flakes are not all one size
        let a = (HALF_MIN + HALF_RANGE * self.size) * (0.7 + 0.3 * h(ix, iy, 3));
        let b = a / self.elong;
        let p = [u - c[0], v - c[1]];
        if p[0].abs() > a * 1.4 || p[1].abs() > a * 1.4 {
            return 0.0;
        }
        let mut d = [2.0 * h(ix, iy, 4) - 1.0, 2.0 * h(ix, iy, 5) - 1.0];
        let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
        d = if n > 0.05 {
            [d[0] / n, d[1] / n]
        } else {
            [1.0, 0.0]
        };
        let q = [p[0] * d[0] + p[1] * d[1], -p[0] * d[1] + p[1] * d[0]];
        // the rectangle's corners (CCW), each pushed by up to `irr` of its half-extents — still convex
        let mut cs = [[a, b], [-a, b], [-a, -b], [a, -b]];
        for (i, cc) in cs.iter_mut().enumerate() {
            let s = 6 + 2 * i as i32;
            cc[0] += self.irr * a * (2.0 * h(ix, iy, s) - 1.0);
            cc[1] += self.irr * b * (2.0 * h(ix, iy, s + 1) - 1.0);
        }
        let mut inside = f32::INFINITY;
        for i in 0..4 {
            let (e0, e1) = (cs[i], cs[(i + 1) % 4]);
            let e = [e1[0] - e0[0], e1[1] - e0[1]];
            let len = (e[0] * e[0] + e[1] * e[1]).sqrt().max(1e-6);
            // CCW ⇒ positive on the inner side
            inside = inside.min((e[0] * (q[1] - e0[1]) - e[1] * (q[0] - e0[0])) / len);
        }
        (inside / EDGE + 0.5).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Measured on a 400² patch at the defaults: the gaps are ELONGATED (the second-moment aspect of each flake ≥ 2)
    /// and cover a real but minor share of the plane.
    #[test]
    fn as_lascas_sao_alongadas_e_esparsas() {
        let k = [0.5, 0.5, 0.5];
        let n = 400;
        let mut buraco = 0usize;
        for j in 0..n {
            for i in 0..n {
                if flakes(i as f32 * 0.02, j as f32 * 0.02, &k) < 0.5 {
                    buraco += 1;
                }
            }
        }
        let frac = buraco as f32 / (n * n) as f32;
        assert!((0.1..0.6).contains(&frac), "fracção de lasca {frac}");
        // the aspect of the flake of cell (3, 3), from its own pixels
        let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut m) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0f64);
        for j in 0..300 {
            for i in 0..300 {
                let (u, v) = (2.0 + i as f32 * 0.01, 2.0 + j as f32 * 0.01);
                if Forma::de(&k).cobre(3, 3, u, v) > 0.5 {
                    let (x, y) = (f64::from(u), f64::from(v));
                    sx += x;
                    sy += y;
                    sxx += x * x;
                    syy += y * y;
                    sxy += x * y;
                    m += 1.0;
                }
            }
        }
        let (cx, cy) = (sx / m, sy / m);
        let (vx, vy, cxy) = (sxx / m - cx * cx, syy / m - cy * cy, sxy / m - cx * cy);
        let t = (vx + vy) / 2.0;
        let r = (((vx - vy) / 2.0).powi(2) + cxy * cxy).sqrt();
        let aspecto = ((t + r) / (t - r).max(1e-9)).sqrt();
        assert!(aspecto > 1.8, "aspecto da lasca {aspecto:.2}");
    }
}
