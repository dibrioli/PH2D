//! Marching-squares iso-contour (the `sdf == 0` zero level set) of an
//! [`SdfGrid`] — ADR-0065 Phase 3's draft *visual*. The SDF is a grid, not a
//! [`VectorNetwork`]; this extracts the silhouette boundary as world-space line
//! segments that the bridge strokes as the real-time boolean preview (the exact
//! Linesweeper reconcile owns the final geometry).
//!
//! ## Determinism (ADR-0065 §2.4)
//!
//! Fixed grid + a fixed `y`-major / `x`-minor scan + a fixed case table + the
//! saddle cases (5, 10) resolved one consistent way (isolate the *inside*
//! corners) → bit-stable. No parallel reductions.

use crate::SdfGrid;
use glam::Vec2;

/// World-space point where the segment `a → b` (two adjacent sample centers)
/// crosses `sdf == 0`, given the signed values `va`, `vb` at its ends. Callers
/// only consult this for edges whose ends straddle zero; the `denom == 0` guard
/// keeps it total (and deterministic) regardless.
fn cross(a: Vec2, b: Vec2, va: f32, vb: f32) -> Vec2 {
    let denom = va - vb;
    // value(t) = va + t·(vb − va) = 0  ⟹  t = va / (va − vb).
    let t = if denom.abs() > f32::EPSILON {
        (va / denom).clamp(0.0, 1.0)
    } else {
        0.5
    };
    a + (b - a) * t
}

/// Extract the `sdf == 0` iso-contour of `grid` as world-space line segments
/// `[p0, p1]`. `network_sdf`'s convention is **negative inside**, so the contour
/// is the filled silhouette's boundary. Marching-squares over the `(res-1)²`
/// blocks of adjacent sample centers; the two saddle cases split into two
/// non-crossing segments. Empty for `res < 2` or a sign-uniform field.
#[must_use]
pub fn marching_contour(grid: &SdfGrid) -> Vec<[Vec2; 2]> {
    let res = grid.res;
    if res < 2 {
        return Vec::new();
    }
    let mut segs = Vec::new();
    for y in 0..res - 1 {
        for x in 0..res - 1 {
            // Corner SDF values + world positions (BL, BR, TR, TL).
            let d00 = grid.at(x, y);
            let d10 = grid.at(x + 1, y);
            let d11 = grid.at(x + 1, y + 1);
            let d01 = grid.at(x, y + 1);
            let c00 = grid.cell_center(x, y);
            let c10 = grid.cell_center(x + 1, y);
            let c11 = grid.cell_center(x + 1, y + 1);
            let c01 = grid.cell_center(x, y + 1);

            let case = u8::from(d00 < 0.0)
                | (u8::from(d10 < 0.0) << 1)
                | (u8::from(d11 < 0.0) << 2)
                | (u8::from(d01 < 0.0) << 3);

            // Zero-crossing per edge (only consulted where the case uses it).
            let bottom = cross(c00, c10, d00, d10); // BL → BR
            let right = cross(c10, c11, d10, d11); // BR → TR
            let top = cross(c11, c01, d11, d01); // TR → TL
            let left = cross(c01, c00, d01, d00); // TL → BL

            // Standard marching-squares table; complementary cases (N and 15−N)
            // share the same contour edges (only inside/outside swaps).
            match case {
                0 | 15 => {}
                1 | 14 => segs.push([left, bottom]),
                2 | 13 => segs.push([bottom, right]),
                3 | 12 => segs.push([left, right]),
                4 | 11 => segs.push([right, top]),
                6 | 9 => segs.push([bottom, top]),
                7 | 8 => segs.push([top, left]),
                // Saddles: isolate each inside corner with its own segment.
                5 => {
                    segs.push([left, bottom]); // isolates BL
                    segs.push([top, right]); // isolates TR
                }
                10 => {
                    segs.push([bottom, right]); // isolates BR
                    segs.push([left, top]); // isolates TL
                }
                _ => unreachable!("case is a 4-bit value"),
            }
        }
    }
    segs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounds, SdfGrid};

    /// Exact analytic SDF of an axis-aligned box (Inigo Quilez) — negative
    /// inside, matching `network_sdf`'s convention. Decouples the marching test
    /// from network cooking.
    fn box_sdf(p: Vec2, center: Vec2, half: Vec2) -> f32 {
        let q = (p - center).abs() - half;
        q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
    }

    fn analytic_box_grid(res: u32, bounds: Bounds, center: Vec2, half: Vec2) -> SdfGrid {
        let span = bounds.max - bounds.min;
        let mut data = vec![0.0f32; (res * res) as usize];
        for y in 0..res {
            for x in 0..res {
                let u = (x as f32 + 0.5) / res as f32;
                let v = (y as f32 + 0.5) / res as f32;
                let p = bounds.min + Vec2::new(span.x * u, span.y * v);
                data[(y * res + x) as usize] = box_sdf(p, center, half);
            }
        }
        SdfGrid { res, bounds, data }
    }

    #[test]
    fn square_contour_hugs_the_boundary() {
        let center = Vec2::new(50.0, 50.0);
        let half = Vec2::splat(10.0); // 20×20 box, boundary at {40,60}²
        let bounds = Bounds {
            min: Vec2::new(30.0, 30.0),
            max: Vec2::new(70.0, 70.0),
        };
        let res = 64;
        let grid = analytic_box_grid(res, bounds, center, half);
        let segs = marching_contour(&grid);

        // The perimeter is 80 world units; at ~0.625 world/cell that is well
        // over 20 marching segments around the loop.
        assert!(
            segs.len() > 20,
            "a 20×20 box should yield a dense contour, got {}",
            segs.len()
        );

        // Every contour endpoint sits on the box boundary (|sdf| ≈ 0), within
        // about one cell of linear-interpolation error.
        let cell = (bounds.max.x - bounds.min.x) / res as f32; // ≈ 0.625
        for s in &segs {
            for &p in s {
                assert!(
                    box_sdf(p, center, half).abs() < cell * 2.0,
                    "contour point {p:?} should lie on the boundary (sdf {})",
                    box_sdf(p, center, half)
                );
            }
        }
    }

    #[test]
    fn uniform_field_has_no_contour() {
        // No sign change anywhere → no zero crossing → no segments.
        let grid = SdfGrid {
            res: 8,
            bounds: Bounds {
                min: Vec2::ZERO,
                max: Vec2::ONE,
            },
            data: vec![1.0; 64],
        };
        assert!(marching_contour(&grid).is_empty());
    }

    #[test]
    fn degenerate_resolution_is_empty() {
        let grid = SdfGrid {
            res: 1,
            bounds: Bounds {
                min: Vec2::ZERO,
                max: Vec2::ONE,
            },
            data: vec![-1.0],
        };
        assert!(marching_contour(&grid).is_empty());
    }
}
