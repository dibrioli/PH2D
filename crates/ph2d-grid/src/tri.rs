//! Triangular grid — equilateral triangles tiling the plane.
//!
//! # Indexing scheme
//!
//! Each triangle is identified by `(k, r)` where:
//! - `r` is the **row strip**: the tri's tightest enclosing strip
//!   `y ∈ [r * h, (r + 1) * h]`, with `h = edge_length * √3 / 2`.
//! - `k` is the **apex column**: the integer column of the tri's
//!   single non-shared (apex) vertex along the lattice line at
//!   `y = r * h` or `y = (r + 1) * h`, in units of `edge_length / 2`.
//!
//! The orientation falls out of parity:
//! - **Up triangle** (apex at top, base on bottom): `(k + r)` is odd.
//!   Vertices `((k - 1) * e/2, r * h)`, `((k + 1) * e/2, r * h)`,
//!   `(k * e/2, (r + 1) * h)`.
//! - **Down triangle** (apex at bottom, base on top): `(k + r)` is
//!   even. Vertices `((k - 1) * e/2, (r + 1) * h)`,
//!   `((k + 1) * e/2, (r + 1) * h)`, `(k * e/2, r * h)`.
//!
//! # Edge-3 neighbors
//!
//! Derived algebraically from the apex-column scheme above.
//! - Up `(k, r)`: down `(k, r - 1)` (bottom), down `(k - 1, r)`
//!   (left), down `(k + 1, r)` (right).
//! - Down `(k, r)`: up `(k, r + 1)` (top), up `(k - 1, r)` (left),
//!   up `(k + 1, r)` (right).
//!
//! # Distance / line / range
//!
//! Triangular grids have no clean cube-distance closed form
//! comparable to hex; the canonical fast formula requires a
//! 3-axis "tri-cube" with `a + b + c ∈ {0, 1}` that complicates
//! the storage representation. For v1 we ship correct-but-slower
//! **BFS** distance / range, capped at radius 64 to keep the
//! buffers bounded. Adequate for the inspect-panel use case
//! (two probe points, debug-info only) and gameplay code that
//! pre-computes paths via [`crate::astar`] in stage 6.

use crate::{GridMath, Vec2};
use std::collections::{BTreeMap, VecDeque};

const SQRT_3: f32 = 1.732_050_8;

/// Tri neighborhood policy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TriNeighborhood {
    /// 3 edge-sharing neighbors (canonical).
    Edge3,
    /// 12 — 3 edge + 9 vertex-only (every other tri sharing at
    /// least one vertex with the cell).
    Vertex12,
}

/// Triangular grid.
#[derive(Copy, Clone, Debug)]
pub struct TriGrid {
    /// Side length of each triangle in world meters.
    pub edge_length: f32,
    /// Neighborhood selection.
    pub neighborhood: TriNeighborhood,
}

/// Tri cell — apex column + row strip. Orientation derived from
/// parity: `(k + r)` odd → up, even → down.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TriCell {
    pub k: i32,
    pub r: i32,
}

impl TriCell {
    pub const fn new(k: i32, r: i32) -> Self {
        Self { k, r }
    }

    /// True when this cell is an up-pointing triangle (apex at the
    /// top of its row strip).
    pub fn is_up(self) -> bool {
        (self.k + self.r).rem_euclid(2) == 1
    }
}

impl TriGrid {
    pub fn new(edge_length: f32, neighborhood: TriNeighborhood) -> Self {
        debug_assert!(edge_length > 0.0);
        Self {
            edge_length,
            neighborhood,
        }
    }

    fn strip_height(&self) -> f32 {
        self.edge_length * SQRT_3 * 0.5
    }
}

/// Edge-3 neighbors of `cell`. Independent of [`TriGrid`] config —
/// pure index-space math. Clears + pushes 3 cells.
pub fn tri_edge3_neighbors(cell: TriCell, out: &mut Vec<TriCell>) {
    out.clear();
    let TriCell { k, r } = cell;
    if cell.is_up() {
        out.push(TriCell::new(k, r - 1)); // bottom (shared base)
        out.push(TriCell::new(k - 1, r)); // left
        out.push(TriCell::new(k + 1, r)); // right
    } else {
        out.push(TriCell::new(k, r + 1)); // top (shared base)
        out.push(TriCell::new(k - 1, r)); // left
        out.push(TriCell::new(k + 1, r)); // right
    }
}

/// Vertex-12 neighbors: 3 edge + 9 vertex-only (every other tri
/// sharing at least one vertex with `cell`).
///
/// Derived geometrically — at each of `cell`'s 3 vertices, 6 tris
/// meet (incl. `cell` + 2 edge-neighbors); the remaining 3 per
/// vertex are vertex-only neighbors. 3 vertices × 3 vertex-only =
/// 9, plus 3 edge = 12 total. No duplicates across vertices since
/// each vertex-only tri shares exactly one vertex with `cell`.
pub fn tri_vertex12_neighbors(cell: TriCell, out: &mut Vec<TriCell>) {
    out.clear();
    let TriCell { k, r } = cell;
    if cell.is_up() {
        // Edge-3.
        out.push(TriCell::new(k, r - 1));
        out.push(TriCell::new(k - 1, r));
        out.push(TriCell::new(k + 1, r));
        // Vertex-only from bottom-left vertex (k-1, r).
        out.push(TriCell::new(k - 2, r));
        out.push(TriCell::new(k - 2, r - 1));
        out.push(TriCell::new(k - 1, r - 1));
        // Vertex-only from bottom-right vertex (k+1, r).
        out.push(TriCell::new(k + 2, r));
        out.push(TriCell::new(k + 2, r - 1));
        out.push(TriCell::new(k + 1, r - 1));
        // Vertex-only from apex vertex (k, r+1).
        out.push(TriCell::new(k, r + 1));
        out.push(TriCell::new(k - 1, r + 1));
        out.push(TriCell::new(k + 1, r + 1));
    } else {
        // Down(k, r) — mirror of up about the horizontal midline.
        // Edge-3.
        out.push(TriCell::new(k, r + 1));
        out.push(TriCell::new(k - 1, r));
        out.push(TriCell::new(k + 1, r));
        // Vertex-only from top-left vertex (k-1, r+1).
        out.push(TriCell::new(k - 2, r + 1));
        out.push(TriCell::new(k - 2, r));
        out.push(TriCell::new(k - 1, r + 1));
        // Vertex-only from top-right vertex (k+1, r+1).
        out.push(TriCell::new(k + 2, r + 1));
        out.push(TriCell::new(k + 2, r));
        out.push(TriCell::new(k + 1, r + 1));
        // Vertex-only from apex vertex (k, r).
        out.push(TriCell::new(k, r - 1));
        out.push(TriCell::new(k - 1, r - 1));
        out.push(TriCell::new(k + 1, r - 1));
    }
}

impl GridMath for TriGrid {
    type Cell = TriCell;

    fn world_to_cell(&self, world: Vec2) -> TriCell {
        // Coarse estimate, then exact search over a 3×3 neighborhood
        // of candidate cells by centroid distance. O(9) — bounded.
        let h = self.strip_height();
        let half_e = self.edge_length * 0.5;
        let r_est = (world[1] / h).floor() as i32;
        let k_est = (world[0] / half_e).round() as i32;

        let mut best = TriCell::new(k_est, r_est);
        let mut best_d2 = f32::INFINITY;
        for dr in -1..=1 {
            for dk in -1..=1 {
                let cand = TriCell::new(k_est + dk, r_est + dr);
                let c = self.cell_to_world_center(cand);
                let d2 = (c[0] - world[0]).powi(2) + (c[1] - world[1]).powi(2);
                if d2 < best_d2 {
                    best_d2 = d2;
                    best = cand;
                }
            }
        }
        best
    }

    fn cell_to_world_center(&self, cell: TriCell) -> Vec2 {
        let h = self.strip_height();
        let half_e = self.edge_length * 0.5;
        let x = cell.k as f32 * half_e;
        // Up centroid: 1/3 of strip from bottom. Down centroid: 2/3.
        let y_frac = if cell.is_up() { 1.0 / 3.0 } else { 2.0 / 3.0 };
        let y = (cell.r as f32 + y_frac) * h;
        [x, y]
    }

    fn cell_to_world_vertices(&self, cell: TriCell, out: &mut Vec<Vec2>) {
        out.clear();
        let h = self.strip_height();
        let half_e = self.edge_length * 0.5;
        let k = cell.k as f32;
        let r = cell.r as f32;
        if cell.is_up() {
            // CCW from base-left: (k-1, r), (k+1, r), apex (k, r+1).
            out.push([(k - 1.0) * half_e, r * h]);
            out.push([(k + 1.0) * half_e, r * h]);
            out.push([k * half_e, (r + 1.0) * h]);
        } else {
            // CCW from apex: apex (k, r), base-right (k+1, r+1),
            // base-left (k-1, r+1).
            out.push([k * half_e, r * h]);
            out.push([(k + 1.0) * half_e, (r + 1.0) * h]);
            out.push([(k - 1.0) * half_e, (r + 1.0) * h]);
        }
    }

    fn neighbors(&self, cell: TriCell, out: &mut Vec<TriCell>) {
        match self.neighborhood {
            TriNeighborhood::Edge3 => tri_edge3_neighbors(cell, out),
            TriNeighborhood::Vertex12 => tri_vertex12_neighbors(cell, out),
        }
    }

    fn distance(&self, a: TriCell, b: TriCell) -> u32 {
        // BFS over edge3 connectivity, capped at 64 to bound runtime.
        // For typical inspect-panel queries (distance < ~20) this is
        // O(N²); fast enough for one-shot debug readouts.
        if a == b {
            return 0;
        }
        let mut seen: BTreeMap<TriCell, u32> = BTreeMap::new();
        seen.insert(a, 0);
        let mut queue: VecDeque<TriCell> = VecDeque::new();
        queue.push_back(a);
        let mut nbuf = Vec::with_capacity(3);
        while let Some(cur) = queue.pop_front() {
            let d = seen[&cur];
            if d >= 64 {
                continue;
            }
            tri_edge3_neighbors(cur, &mut nbuf);
            for n in &nbuf {
                if *n == b {
                    return d + 1;
                }
                if !seen.contains_key(n) {
                    seen.insert(*n, d + 1);
                    queue.push_back(*n);
                }
            }
        }
        u32::MAX
    }

    fn line(&self, a: TriCell, b: TriCell, out: &mut Vec<TriCell>) {
        // Greedy: at each step, pick the edge-3 neighbor whose
        // centroid is closest to the line from `cur` to `b`. Caps at
        // 256 steps to avoid pathological inputs.
        out.clear();
        out.push(a);
        if a == b {
            return;
        }
        let mut cur = a;
        let mut nbuf = Vec::with_capacity(3);
        for _ in 0..256 {
            tri_edge3_neighbors(cur, &mut nbuf);
            let target = self.cell_to_world_center(b);
            let mut best = nbuf[0];
            let mut best_d2 = f32::INFINITY;
            for n in &nbuf {
                let c = self.cell_to_world_center(*n);
                let d2 = (c[0] - target[0]).powi(2) + (c[1] - target[1]).powi(2);
                if d2 < best_d2 {
                    best_d2 = d2;
                    best = *n;
                }
            }
            out.push(best);
            if best == b {
                return;
            }
            cur = best;
        }
    }

    fn range(&self, center: TriCell, radius: u32, out: &mut Vec<TriCell>) {
        // BFS up to radius (capped at 64 same as distance).
        out.clear();
        let cap = radius.min(64);
        let mut seen: BTreeMap<TriCell, u32> = BTreeMap::new();
        seen.insert(center, 0);
        out.push(center);
        let mut queue: VecDeque<TriCell> = VecDeque::new();
        queue.push_back(center);
        let mut nbuf = Vec::with_capacity(3);
        while let Some(cur) = queue.pop_front() {
            let d = seen[&cur];
            if d >= cap {
                continue;
            }
            tri_edge3_neighbors(cur, &mut nbuf);
            for n in &nbuf {
                if !seen.contains_key(n) {
                    seen.insert(*n, d + 1);
                    out.push(*n);
                    queue.push_back(*n);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parity_distinguishes_up_and_down() {
        assert!(TriCell::new(1, 0).is_up()); // 1+0 = 1, odd → up
        assert!(!TriCell::new(2, 0).is_up()); // 2+0 = 2, even → down
        assert!(TriCell::new(0, 1).is_up()); // 0+1 = 1, odd → up
    }

    #[test]
    fn edge3_neighbors_count_three_and_swap_orientation() {
        let mut out = Vec::new();
        let up = TriCell::new(1, 0); // up
        tri_edge3_neighbors(up, &mut out);
        assert_eq!(out.len(), 3);
        for n in &out {
            assert!(!n.is_up(), "neighbor of up must be down: {n:?}");
        }

        let down = TriCell::new(2, 0);
        tri_edge3_neighbors(down, &mut out);
        assert_eq!(out.len(), 3);
        for n in &out {
            assert!(n.is_up(), "neighbor of down must be up: {n:?}");
        }
    }

    #[test]
    fn vertex12_returns_twelve() {
        let mut out = Vec::new();
        tri_vertex12_neighbors(TriCell::new(1, 0), &mut out);
        assert_eq!(out.len(), 12, "got {out:?}");
        // No duplicates.
        for i in 0..out.len() {
            for j in (i + 1)..out.len() {
                assert_ne!(out[i], out[j]);
            }
        }
    }

    #[test]
    fn world_round_trip_centroids() {
        let g = TriGrid::new(2.0, TriNeighborhood::Edge3);
        for k in -3..=3 {
            for r in -3..=3 {
                let c = TriCell::new(k, r);
                let center = g.cell_to_world_center(c);
                assert_eq!(g.world_to_cell(center), c, "round-trip {c:?}");
            }
        }
    }

    #[test]
    fn vertices_three_for_every_cell() {
        let g = TriGrid::new(1.0, TriNeighborhood::Edge3);
        let mut v = Vec::new();
        for c in [
            TriCell::new(1, 0),
            TriCell::new(2, 0),
            TriCell::new(0, 3),
            TriCell::new(-2, -1),
        ] {
            g.cell_to_world_vertices(c, &mut v);
            assert_eq!(v.len(), 3, "vertex count for {c:?}");
        }
    }

    #[test]
    fn distance_self_is_zero() {
        let g = TriGrid::new(1.0, TriNeighborhood::Edge3);
        assert_eq!(g.distance(TriCell::new(3, 2), TriCell::new(3, 2)), 0);
    }

    #[test]
    fn distance_one_for_edge3_neighbors() {
        let g = TriGrid::new(1.0, TriNeighborhood::Edge3);
        let center = TriCell::new(1, 0);
        let mut nb = Vec::new();
        tri_edge3_neighbors(center, &mut nb);
        for n in nb {
            assert_eq!(g.distance(center, n), 1, "neighbor {n:?}");
        }
    }

    #[test]
    fn line_endpoints_match() {
        let g = TriGrid::new(1.0, TriNeighborhood::Edge3);
        let mut out = Vec::new();
        g.line(TriCell::new(0, 0), TriCell::new(4, 2), &mut out);
        assert_eq!(out.first(), Some(&TriCell::new(0, 0)));
        assert_eq!(out.last(), Some(&TriCell::new(4, 2)));
    }

    #[test]
    fn range_radius_zero_is_center() {
        let g = TriGrid::new(1.0, TriNeighborhood::Edge3);
        let mut out = Vec::new();
        g.range(TriCell::new(2, 1), 0, &mut out);
        assert_eq!(out, vec![TriCell::new(2, 1)]);
    }

    #[test]
    fn range_radius_one_has_four_cells() {
        let g = TriGrid::new(1.0, TriNeighborhood::Edge3);
        let mut out = Vec::new();
        g.range(TriCell::new(1, 0), 1, &mut out);
        assert_eq!(out.len(), 4, "center + 3 edge neighbors");
    }
}
