//! Isometric grid — square cells projected in 2:1 dimetric (26.57°).
//!
//! Index space is identical to [`crate::square`]; only the world
//! projection differs:
//!
//! ```text
//! screen_x = (gx - gy) * tile_w / 2
//! screen_y = (gx + gy) * tile_h / 2
//! ```
//!
//! Default `tile_w = 2.0`, `tile_h = 1.0` gives the classic 2:1
//! dimetric ratio (Diablo, SimCity 2000, Stardew Valley UI).
//!
//! Neighbor / distance / line math is delegated to the same logic
//! as Square — diagonals in *index* space become axis-aligned in
//! *world* space because the rotation+scale is part of the
//! projection.

use crate::square::SquareNeighborhood;
use crate::{GridMath, Vec2};

/// Isometric grid. Cells are diamonds (rhombi) in world space; the
/// underlying index grid is square.
#[derive(Copy, Clone, Debug)]
pub struct IsoGrid {
    /// Width of one tile in world meters at the widest horizontal
    /// span (left corner → right corner of the diamond). Must be > 0.
    pub tile_w: f32,
    /// Height of one tile in world meters at the tallest vertical
    /// span (top corner → bottom corner). For 2:1 dimetric,
    /// `tile_h == tile_w / 2`. Must be > 0.
    pub tile_h: f32,
    /// 4 or 8 neighborhood (same semantics as Square).
    pub neighborhood: SquareNeighborhood,
}

/// Cell identifier `(grid_x, grid_y)` — same shape as `SquareCell`.
pub type IsoCell = (i32, i32);

impl IsoGrid {
    /// Classic 2:1 dimetric: `tile_w = 2 * tile_h`.
    pub fn dimetric_2_1(tile_h: f32, neighborhood: SquareNeighborhood) -> Self {
        debug_assert!(tile_h > 0.0);
        Self {
            tile_w: tile_h * 2.0,
            tile_h,
            neighborhood,
        }
    }
}

/// Grid `(gx, gy)` → world. Inverse of [`world_to_iso`].
pub fn iso_to_world(grid: &IsoGrid, cell: IsoCell) -> Vec2 {
    let gx = cell.0 as f32;
    let gy = cell.1 as f32;
    let x = (gx - gy) * grid.tile_w * 0.5;
    let y = (gx + gy) * grid.tile_h * 0.5;
    [x, y]
}

/// World → grid `(gx, gy)`. Splits the inverse projection then
/// floors to the containing diamond cell.
pub fn world_to_iso(grid: &IsoGrid, world: Vec2) -> IsoCell {
    // Inverse projection (in continuous coords).
    let u = world[0] / (grid.tile_w * 0.5); // = gx - gy
    let v = world[1] / (grid.tile_h * 0.5); // = gx + gy
    let fx = (u + v) * 0.5;
    let fy = (v - u) * 0.5;
    (fx.floor() as i32, fy.floor() as i32)
}

impl GridMath for IsoGrid {
    type Cell = IsoCell;

    fn world_to_cell(&self, world: Vec2) -> IsoCell {
        world_to_iso(self, world)
    }

    fn cell_to_world_center(&self, cell: IsoCell) -> Vec2 {
        // Diamond center of cell (gx, gy) — average of the four
        // corners simplifies to ((gx - gy) * tw / 2, (gx + gy + 1) * th / 2).
        let gx = cell.0 as f32;
        let gy = cell.1 as f32;
        let cx = (gx - gy) * self.tile_w * 0.5;
        let cy = (gx + gy + 1.0) * self.tile_h * 0.5;
        [cx, cy]
    }

    fn cell_to_world_vertices(&self, cell: IsoCell, out: &mut Vec<Vec2>) {
        out.clear();
        // The diamond covering grid cell (gx, gy) has 4 corners.
        // Use iso_to_world on the four index corners (gx, gy),
        // (gx+1, gy), (gx+1, gy+1), (gx, gy+1) which become the
        // top, right, bottom, left corners of the diamond.
        let top = iso_to_world(self, (cell.0, cell.1));
        let right = iso_to_world(self, (cell.0 + 1, cell.1));
        let bottom = iso_to_world(self, (cell.0 + 1, cell.1 + 1));
        let left = iso_to_world(self, (cell.0, cell.1 + 1));
        // CCW order in world space (Y-up): start at right corner.
        out.push(right);
        out.push(bottom);
        out.push(left);
        out.push(top);
    }

    fn neighbors(&self, cell: IsoCell, out: &mut Vec<IsoCell>) {
        out.clear();
        let (i, j) = cell;
        match self.neighborhood {
            SquareNeighborhood::Von4 => {
                out.push((i + 1, j));
                out.push((i, j + 1));
                out.push((i - 1, j));
                out.push((i, j - 1));
            }
            SquareNeighborhood::Moore8 => {
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

    fn distance(&self, a: IsoCell, b: IsoCell) -> u32 {
        match self.neighborhood {
            SquareNeighborhood::Von4 => crate::square::manhattan(a, b),
            SquareNeighborhood::Moore8 => crate::square::chebyshev(a, b),
        }
    }

    fn line(&self, a: IsoCell, b: IsoCell, out: &mut Vec<IsoCell>) {
        crate::square::bresenham_line(a, b, out);
    }

    fn range(&self, center: IsoCell, radius: u32, out: &mut Vec<IsoCell>) {
        // Identical to Square — index space is the same.
        out.clear();
        let r = radius as i32;
        let (ci, cj) = center;
        match self.neighborhood {
            SquareNeighborhood::Von4 => {
                for di in -r..=r {
                    let span = r - di.abs();
                    for dj in -span..=span {
                        out.push((ci + di, cj + dj));
                    }
                }
            }
            SquareNeighborhood::Moore8 => {
                for di in -r..=r {
                    for dj in -r..=r {
                        out.push((ci + di, cj + dj));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_origin_maps_to_origin() {
        let g = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Von4);
        let p = iso_to_world(&g, (0, 0));
        assert!(p[0].abs() < 1e-6 && p[1].abs() < 1e-6);
    }

    #[test]
    fn projection_2_to_1_ratio() {
        // tile_w = 2.0, tile_h = 1.0: stepping +1 in gx moves +1.0 in
        // world x and +0.5 in world y (half a tile each axis).
        let g = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Von4);
        let p = iso_to_world(&g, (1, 0));
        assert!((p[0] - 1.0).abs() < 1e-6, "x: {}", p[0]);
        assert!((p[1] - 0.5).abs() < 1e-6, "y: {}", p[1]);
    }

    #[test]
    fn world_round_trip_inside_cell() {
        let g = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Von4);
        for i in -3..=3 {
            for j in -3..=3 {
                // Nudge from the cell's diamond center (which is at
                // grid-corner (i+0.5, j+0.5)). center is in world.
                let center = g.cell_to_world_center((i, j));
                let nudged = [center[0] + 0.01, center[1] - 0.01];
                assert_eq!(world_to_iso(&g, nudged), (i, j), "at ({i},{j})");
            }
        }
    }

    #[test]
    fn vertices_are_four_corners() {
        let g = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Von4);
        let mut v = Vec::new();
        g.cell_to_world_vertices((0, 0), &mut v);
        assert_eq!(v.len(), 4);
        // Cell (0,0) diamond corners (CCW from right):
        //   right  = iso_to_world((1, 0)) = ( 1.0,  0.5)
        //   bottom = iso_to_world((1, 1)) = ( 0.0,  1.0)
        //   left   = iso_to_world((0, 1)) = (-1.0,  0.5)
        //   top    = iso_to_world((0, 0)) = ( 0.0,  0.0)
        let eps = 1e-5;
        let close = |a: Vec2, b: Vec2| (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps;
        assert!(close(v[0], [1.0, 0.5]));
        assert!(close(v[1], [0.0, 1.0]));
        assert!(close(v[2], [-1.0, 0.5]));
        assert!(close(v[3], [0.0, 0.0]));
    }

    #[test]
    fn neighbors_match_square_in_index_space() {
        let g = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Moore8);
        let mut out = Vec::new();
        g.neighbors((5, 5), &mut out);
        assert_eq!(out.len(), 8);
        assert!(out.contains(&(6, 6)));
        assert!(out.contains(&(4, 4)));
    }

    #[test]
    fn distance_matches_square_in_index_space() {
        let g = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Von4);
        assert_eq!(g.distance((0, 0), (3, 4)), 7);
        let g8 = IsoGrid::dimetric_2_1(1.0, SquareNeighborhood::Moore8);
        assert_eq!(g8.distance((0, 0), (3, 4)), 4);
    }
}
