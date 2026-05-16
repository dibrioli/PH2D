//! Square grids — Von Neumann (4) and Moore (8) neighborhoods.
//!
//! Cell origin convention: cell `(0, 0)` occupies world rect
//! `[0, cell_size) × [0, cell_size)`. World `(0, 0)` is the corner
//! shared by cells `(0,0)`, `(-1,0)`, `(0,-1)`, `(-1,-1)`. Cell
//! center for `(i, j)` is `((i + 0.5) * s, (j + 0.5) * s)`.
//!
//! World Y-up convention (matches `ph2d-editor`); rows increase as
//! cell index `j` grows.

use crate::{GridMath, Vec2};

/// Neighborhood policy for [`SquareGrid`]. Determines both
/// `neighbors()` output and which graph distance applies.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SquareNeighborhood {
    /// 4-neighbor ortho (von Neumann). Graph distance = Manhattan.
    Von4,
    /// 8-neighbor ortho + diagonal (Moore). Graph distance =
    /// Chebyshev. Diagonal travel cost in pathfinding is √2 — but
    /// graph distance counts edge steps, so a diagonal move is 1
    /// step; A* layers (`crate::astar`) take a cost callback that
    /// can return `SQRT_2` for diagonals if so desired.
    Moore8,
}

/// Square / orthogonal grid. Uniform cell size; integer cell coords.
#[derive(Copy, Clone, Debug)]
pub struct SquareGrid {
    /// Side length of each cell in world meters. Must be > 0.
    pub cell_size: f32,
    /// Active neighborhood (Von4 / Moore8).
    pub neighborhood: SquareNeighborhood,
}

/// Cell identifier `(col, row)`. `col` grows with world +X; `row`
/// grows with world +Y.
pub type SquareCell = (i32, i32);

impl SquareGrid {
    /// New grid with the given cell size + neighborhood. Caller
    /// guarantees `cell_size > 0`; a non-positive value will panic in
    /// debug and silently produce nonsense in release (no recovery —
    /// HR-3 forbids defensive branches in hot math).
    pub fn new(cell_size: f32, neighborhood: SquareNeighborhood) -> Self {
        debug_assert!(cell_size > 0.0, "cell_size must be positive");
        Self {
            cell_size,
            neighborhood,
        }
    }
}

impl GridMath for SquareGrid {
    type Cell = SquareCell;

    fn world_to_cell(&self, world: Vec2) -> SquareCell {
        // floor() rather than `as i32` cast — the latter truncates
        // toward zero, which would map world (-0.5, -0.5) to cell
        // (0, 0) instead of (-1, -1).
        let i = (world[0] / self.cell_size).floor() as i32;
        let j = (world[1] / self.cell_size).floor() as i32;
        (i, j)
    }

    fn cell_to_world_center(&self, cell: SquareCell) -> Vec2 {
        let s = self.cell_size;
        [(cell.0 as f32 + 0.5) * s, (cell.1 as f32 + 0.5) * s]
    }

    fn cell_to_world_vertices(&self, cell: SquareCell, out: &mut Vec<Vec2>) {
        out.clear();
        let s = self.cell_size;
        let x0 = cell.0 as f32 * s;
        let y0 = cell.1 as f32 * s;
        out.push([x0, y0]);
        out.push([x0 + s, y0]);
        out.push([x0 + s, y0 + s]);
        out.push([x0, y0 + s]);
    }

    fn neighbors(&self, cell: SquareCell, out: &mut Vec<SquareCell>) {
        out.clear();
        let (i, j) = cell;
        match self.neighborhood {
            SquareNeighborhood::Von4 => {
                // E, N, W, S (deterministic ordering).
                out.push((i + 1, j));
                out.push((i, j + 1));
                out.push((i - 1, j));
                out.push((i, j - 1));
            }
            SquareNeighborhood::Moore8 => {
                // E, NE, N, NW, W, SW, S, SE (deterministic CCW).
                out.push((i + 1, j));
                out.push((i + 1, j + 1));
                out.push((i, j + 1));
                out.push((i - 1, j + 1));
                out.push((i - 1, j));
                out.push((i - 1, j - 1));
                out.push((i, j - 1));
                out.push((i + 1, j - 1));
            }
        }
    }

    fn distance(&self, a: SquareCell, b: SquareCell) -> u32 {
        match self.neighborhood {
            SquareNeighborhood::Von4 => manhattan(a, b),
            SquareNeighborhood::Moore8 => chebyshev(a, b),
        }
    }

    fn line(&self, a: SquareCell, b: SquareCell, out: &mut Vec<SquareCell>) {
        bresenham_line(a, b, out);
    }

    fn range(&self, center: SquareCell, radius: u32, out: &mut Vec<SquareCell>) {
        out.clear();
        let r = radius as i32;
        let (ci, cj) = center;
        match self.neighborhood {
            SquareNeighborhood::Von4 => {
                // Manhattan disk: |di| + |dj| ≤ r.
                for di in -r..=r {
                    let span = r - di.abs();
                    for dj in -span..=span {
                        out.push((ci + di, cj + dj));
                    }
                }
            }
            SquareNeighborhood::Moore8 => {
                // Chebyshev disk: max(|di|, |dj|) ≤ r → axis-aligned
                // square of side (2r+1).
                for di in -r..=r {
                    for dj in -r..=r {
                        out.push((ci + di, cj + dj));
                    }
                }
            }
        }
    }
}

/// Manhattan distance between two square cells.
///
/// `|a.col - b.col| + |a.row - b.row|`. Returns `u32` via i64
/// intermediate so callers near i32 extremes can't overflow.
pub fn manhattan(a: SquareCell, b: SquareCell) -> u32 {
    let dx = (a.0 as i64 - b.0 as i64).unsigned_abs();
    let dy = (a.1 as i64 - b.1 as i64).unsigned_abs();
    (dx + dy).min(u32::MAX as u64) as u32
}

/// Chebyshev distance between two square cells (Moore neighborhood).
///
/// `max(|a.col - b.col|, |a.row - b.row|)`.
pub fn chebyshev(a: SquareCell, b: SquareCell) -> u32 {
    let dx = (a.0 as i64 - b.0 as i64).unsigned_abs();
    let dy = (a.1 as i64 - b.1 as i64).unsigned_abs();
    dx.max(dy).min(u32::MAX as u64) as u32
}

/// Squared Euclidean distance between cell centers (units of cell
/// edges, not world meters). Cheaper than `euclidean` when callers
/// only need to compare distances.
pub fn euclidean_squared(a: SquareCell, b: SquareCell) -> f64 {
    let dx = a.0 as f64 - b.0 as f64;
    let dy = a.1 as f64 - b.1 as f64;
    dx * dx + dy * dy
}

/// Euclidean distance between cell centers (in cell-edge units).
pub fn euclidean(a: SquareCell, b: SquareCell) -> f64 {
    euclidean_squared(a, b).sqrt()
}

/// Bresenham's line algorithm from `a` to `b`, inclusive. Pushes
/// each cell along the line into `out` (cleared first).
///
/// Standard integer-only formulation (no float ops, no division).
/// Output order is `a` first, `b` last; produces a 4-connected
/// path (one step per cell, no diagonal jumps).
pub fn bresenham_line(a: SquareCell, b: SquareCell, out: &mut Vec<SquareCell>) {
    out.clear();
    let (mut x0, mut y0) = a;
    let (x1, y1) = b;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        out.push((x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_cell_floor_at_origin() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        assert_eq!(g.world_to_cell([0.0, 0.0]), (0, 0));
        assert_eq!(g.world_to_cell([0.99, 0.99]), (0, 0));
        assert_eq!(g.world_to_cell([1.0, 0.0]), (1, 0));
        // Negative side: -0.5 → cell (-1, -1), not (0, 0).
        assert_eq!(g.world_to_cell([-0.5, -0.5]), (-1, -1));
    }

    #[test]
    fn cell_center_round_trip() {
        let g = SquareGrid::new(2.5, SquareNeighborhood::Moore8);
        for i in -5..=5 {
            for j in -5..=5 {
                let c = (i, j);
                let center = g.cell_to_world_center(c);
                assert_eq!(g.world_to_cell(center), c, "round-trip at {c:?}");
            }
        }
    }

    #[test]
    fn vertices_form_unit_square() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut v = Vec::new();
        g.cell_to_world_vertices((3, 4), &mut v);
        assert_eq!(v.len(), 4);
        assert_eq!(v[0], [6.0, 8.0]);
        assert_eq!(v[1], [8.0, 8.0]);
        assert_eq!(v[2], [8.0, 10.0]);
        assert_eq!(v[3], [6.0, 10.0]);
    }

    #[test]
    fn neighbors_von4_returns_four() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut out = Vec::new();
        g.neighbors((0, 0), &mut out);
        assert_eq!(out.len(), 4);
        assert!(out.contains(&(1, 0)));
        assert!(out.contains(&(-1, 0)));
        assert!(out.contains(&(0, 1)));
        assert!(out.contains(&(0, -1)));
    }

    #[test]
    fn neighbors_moore8_returns_eight() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Moore8);
        let mut out = Vec::new();
        g.neighbors((0, 0), &mut out);
        assert_eq!(out.len(), 8);
        // Diagonals present.
        assert!(out.contains(&(1, 1)));
        assert!(out.contains(&(-1, 1)));
        assert!(out.contains(&(-1, -1)));
        assert!(out.contains(&(1, -1)));
    }

    #[test]
    fn neighbors_buffer_reuse_is_alloc_free() {
        // Capacity preserved across calls — the buffer-reuse contract
        // we promise in lib.rs docs. (We can't assert "zero alloc"
        // here without dhat, but capacity-stays-same is the
        // observable invariant.)
        let g = SquareGrid::new(1.0, SquareNeighborhood::Moore8);
        let mut out = Vec::with_capacity(8);
        let cap = out.capacity();
        for cell in [(0, 0), (5, -3), (-100, 100)] {
            g.neighbors(cell, &mut out);
            assert_eq!(out.capacity(), cap, "capacity changed");
            assert_eq!(out.len(), 8);
        }
    }

    #[test]
    fn distance_manhattan_under_von4() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        assert_eq!(g.distance((0, 0), (3, 4)), 7);
        assert_eq!(g.distance((0, 0), (-3, -4)), 7);
        assert_eq!(g.distance((0, 0), (0, 0)), 0);
    }

    #[test]
    fn distance_chebyshev_under_moore8() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Moore8);
        assert_eq!(g.distance((0, 0), (3, 4)), 4);
        assert_eq!(g.distance((0, 0), (-7, 2)), 7);
    }

    #[test]
    fn distance_helpers_handle_i32_extremes() {
        // |i32::MIN - i32::MAX| would overflow if done in i32; the
        // i64 intermediate keeps `manhattan` well-defined.
        let d = manhattan((i32::MIN, 0), (i32::MAX, 0));
        // (i64) i32::MAX - i32::MIN = 2^32 - 1, capped at u32::MAX.
        assert_eq!(d, u32::MAX);
    }

    #[test]
    fn line_horizontal() {
        let mut out = Vec::new();
        bresenham_line((0, 0), (5, 0), &mut out);
        assert_eq!(out, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]);
    }

    #[test]
    fn line_vertical() {
        let mut out = Vec::new();
        bresenham_line((2, -1), (2, 2), &mut out);
        assert_eq!(out, vec![(2, -1), (2, 0), (2, 1), (2, 2)]);
    }

    #[test]
    fn line_diagonal_45() {
        let mut out = Vec::new();
        bresenham_line((0, 0), (3, 3), &mut out);
        assert_eq!(out, vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn line_endpoints_included() {
        let mut out = Vec::new();
        bresenham_line((1, 2), (4, 6), &mut out);
        assert_eq!(out.first(), Some(&(1, 2)));
        assert_eq!(out.last(), Some(&(4, 6)));
    }

    #[test]
    fn line_single_cell() {
        // Degenerate: same start and end.
        let mut out = Vec::new();
        bresenham_line((7, 7), (7, 7), &mut out);
        assert_eq!(out, vec![(7, 7)]);
    }

    #[test]
    fn range_von4_is_manhattan_disk() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut out = Vec::new();
        g.range((0, 0), 1, &mut out);
        // Center + 4 axis neighbors = 5 cells.
        assert_eq!(out.len(), 5);
        // Every output cell satisfies Manhattan ≤ 1.
        for cell in &out {
            assert!(manhattan((0, 0), *cell) <= 1);
        }
    }

    #[test]
    fn range_moore8_is_chebyshev_disk() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Moore8);
        let mut out = Vec::new();
        g.range((0, 0), 2, &mut out);
        // (2r+1)^2 = 25 cells.
        assert_eq!(out.len(), 25);
        for cell in &out {
            assert!(chebyshev((0, 0), *cell) <= 2);
        }
    }

    #[test]
    fn range_radius_zero_is_singleton() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut out = Vec::new();
        g.range((3, -2), 0, &mut out);
        assert_eq!(out, vec![(3, -2)]);
    }
}
