//! Staggered grids — square cells with alternating-row offset, and
//! the array-friendly offset projection of hex cells.
//!
//! # Staggered square ([`StaggeredSquareGrid`])
//!
//! Square cells, every other row shifted by half a cell width.
//! Index space is the same `(col, row)` as a regular Square grid —
//! the offset is **purely a render-time projection**, so neighbor /
//! distance / line / range math is identical to Square. The shift
//! is what changes visually (and is exposed to gameplay code that
//! cares about the world-space pixel layout, e.g. tilesets with
//! brick-pattern art).
//!
//! # Staggered hex ([`StaggeredHexGrid`])
//!
//! Thin wrapper that exposes a [`crate::hex::HexGrid`] under an
//! offset-coord view. The grid's `Cell` is [`crate::hex::OffsetCell`]
//! `(col, row)`, ideal for array-of-arrays storage. All math is
//! delegated to axial via the active [`crate::hex::HexOffset`]
//! variant.

use crate::hex::{self, HexCell, HexGrid, HexOffset, OffsetCell, axial_to_offset, offset_to_axial};
use crate::square::SquareNeighborhood;
use crate::{GridMath, Vec2};

// =============================================================================
// Staggered square
// =============================================================================

/// Which rows get shifted right.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StaggerParity {
    /// Odd-indexed rows shifted by `cell_w / 2`.
    OddRows,
    /// Even-indexed rows shifted by `cell_w / 2`.
    EvenRows,
}

/// Square grid with alternating-row half-cell horizontal offset.
#[derive(Copy, Clone, Debug)]
pub struct StaggeredSquareGrid {
    pub cell_w: f32,
    pub cell_h: f32,
    pub parity: StaggerParity,
    pub neighborhood: SquareNeighborhood,
}

/// `(col, row)` — same index shape as `SquareCell`.
pub type StaggeredSquareCell = (i32, i32);

impl StaggeredSquareGrid {
    pub fn new(
        cell_w: f32,
        cell_h: f32,
        parity: StaggerParity,
        neighborhood: SquareNeighborhood,
    ) -> Self {
        debug_assert!(cell_w > 0.0 && cell_h > 0.0);
        Self {
            cell_w,
            cell_h,
            parity,
            neighborhood,
        }
    }

    fn row_shift(&self, row: i32) -> f32 {
        let parity_bit = row.rem_euclid(2);
        let shifted = match self.parity {
            StaggerParity::OddRows => parity_bit == 1,
            StaggerParity::EvenRows => parity_bit == 0,
        };
        if shifted { self.cell_w * 0.5 } else { 0.0 }
    }
}

impl GridMath for StaggeredSquareGrid {
    type Cell = StaggeredSquareCell;

    fn world_to_cell(&self, world: Vec2) -> StaggeredSquareCell {
        // Row is straightforward — divides world Y.
        let row = (world[1] / self.cell_h).floor() as i32;
        // Column accounts for the shift applied to this specific row.
        let shifted_x = world[0] - self.row_shift(row);
        let col = (shifted_x / self.cell_w).floor() as i32;
        (col, row)
    }

    fn cell_to_world_center(&self, cell: StaggeredSquareCell) -> Vec2 {
        let row = cell.1;
        let cx = (cell.0 as f32 + 0.5) * self.cell_w + self.row_shift(row);
        let cy = (row as f32 + 0.5) * self.cell_h;
        [cx, cy]
    }

    fn cell_to_world_vertices(&self, cell: StaggeredSquareCell, out: &mut Vec<Vec2>) {
        out.clear();
        let row = cell.1;
        let x0 = cell.0 as f32 * self.cell_w + self.row_shift(row);
        let y0 = row as f32 * self.cell_h;
        out.push([x0, y0]);
        out.push([x0 + self.cell_w, y0]);
        out.push([x0 + self.cell_w, y0 + self.cell_h]);
        out.push([x0, y0 + self.cell_h]);
    }

    fn neighbors(&self, cell: StaggeredSquareCell, out: &mut Vec<StaggeredSquareCell>) {
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

    fn distance(&self, a: StaggeredSquareCell, b: StaggeredSquareCell) -> u32 {
        match self.neighborhood {
            SquareNeighborhood::Von4 => crate::square::manhattan(a, b),
            SquareNeighborhood::Moore8 => crate::square::chebyshev(a, b),
        }
    }

    fn line(
        &self,
        a: StaggeredSquareCell,
        b: StaggeredSquareCell,
        out: &mut Vec<StaggeredSquareCell>,
    ) {
        crate::square::bresenham_line(a, b, out);
    }

    fn range(&self, center: StaggeredSquareCell, radius: u32, out: &mut Vec<StaggeredSquareCell>) {
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

// =============================================================================
// Staggered hex (offset view of hex)
// =============================================================================

/// Hex grid exposed in offset `(col, row)` coordinates.
///
/// All distance / neighbor math goes through the underlying axial
/// representation; offset is the storage view convenient for
/// 2D-array gameplay code.
#[derive(Copy, Clone, Debug)]
pub struct StaggeredHexGrid {
    pub hex: HexGrid,
    pub variant: HexOffset,
}

impl StaggeredHexGrid {
    pub fn new(hex: HexGrid, variant: HexOffset) -> Self {
        Self { hex, variant }
    }
}

impl GridMath for StaggeredHexGrid {
    type Cell = OffsetCell;

    fn world_to_cell(&self, world: Vec2) -> OffsetCell {
        let axial = hex::world_to_axial(&self.hex, world);
        axial_to_offset(axial, self.variant)
    }

    fn cell_to_world_center(&self, cell: OffsetCell) -> Vec2 {
        let axial = offset_to_axial(cell, self.variant);
        hex::axial_to_world(&self.hex, axial)
    }

    fn cell_to_world_vertices(&self, cell: OffsetCell, out: &mut Vec<Vec2>) {
        let axial = offset_to_axial(cell, self.variant);
        hex::axial_to_world_vertices(&self.hex, axial, out);
    }

    fn neighbors(&self, cell: OffsetCell, out: &mut Vec<OffsetCell>) {
        out.clear();
        let axial = offset_to_axial(cell, self.variant);
        // Push axial neighbors, convert each back to offset.
        for d in hex::HEX_DIRECTIONS.iter() {
            let n_axial = HexCell {
                q: axial.q + d.q,
                r: axial.r + d.r,
            };
            out.push(axial_to_offset(n_axial, self.variant));
        }
    }

    fn distance(&self, a: OffsetCell, b: OffsetCell) -> u32 {
        let aa = offset_to_axial(a, self.variant);
        let bb = offset_to_axial(b, self.variant);
        hex::hex_distance(aa, bb)
    }

    fn line(&self, a: OffsetCell, b: OffsetCell, out: &mut Vec<OffsetCell>) {
        let aa = offset_to_axial(a, self.variant);
        let bb = offset_to_axial(b, self.variant);
        // Need an intermediate axial buffer — caller's buffer is
        // typed as OffsetCell. Use a small temporary; allocation
        // happens only on first call (then grows up to line length).
        let mut tmp: Vec<HexCell> = Vec::new();
        hex::hex_line(aa, bb, &mut tmp);
        out.clear();
        out.reserve(tmp.len());
        for c in tmp {
            out.push(axial_to_offset(c, self.variant));
        }
    }

    fn range(&self, center: OffsetCell, radius: u32, out: &mut Vec<OffsetCell>) {
        let center_axial = offset_to_axial(center, self.variant);
        let mut tmp: Vec<HexCell> = Vec::new();
        hex::hex_range(center_axial, radius, &mut tmp);
        out.clear();
        out.reserve(tmp.len());
        for c in tmp {
            out.push(axial_to_offset(c, self.variant));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::OffsetCell;

    #[test]
    fn staggered_square_round_trip() {
        let g =
            StaggeredSquareGrid::new(2.0, 1.0, StaggerParity::OddRows, SquareNeighborhood::Von4);
        for i in -3..=3 {
            for j in -3..=3 {
                let c = (i, j);
                let center = g.cell_to_world_center(c);
                assert_eq!(g.world_to_cell(center), c, "round-trip at {c:?}");
            }
        }
    }

    #[test]
    fn staggered_square_odd_rows_shift_right() {
        let g =
            StaggeredSquareGrid::new(2.0, 1.0, StaggerParity::OddRows, SquareNeighborhood::Von4);
        // Row 0 (even): no shift. Cell (0, 0) center at (1.0, 0.5).
        let c0 = g.cell_to_world_center((0, 0));
        assert!((c0[0] - 1.0).abs() < 1e-6 && (c0[1] - 0.5).abs() < 1e-6);
        // Row 1 (odd): shifted by +1.0 (= cell_w/2). Cell (0, 1)
        // center at (2.0, 1.5).
        let c1 = g.cell_to_world_center((0, 1));
        assert!((c1[0] - 2.0).abs() < 1e-6 && (c1[1] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn staggered_hex_round_trip() {
        let hex_grid = HexGrid::pointy(1.0);
        let g = StaggeredHexGrid::new(hex_grid, HexOffset::OddR);
        for col in -3..=3 {
            for row in -3..=3 {
                let cell = OffsetCell::new(col, row);
                let center = g.cell_to_world_center(cell);
                // Round-trip through world.
                assert_eq!(g.world_to_cell(center), cell, "round-trip at {cell:?}");
            }
        }
    }

    #[test]
    fn staggered_hex_neighbors_are_six_with_dist_one() {
        let hex_grid = HexGrid::pointy(1.0);
        let g = StaggeredHexGrid::new(hex_grid, HexOffset::OddR);
        let center = OffsetCell::new(3, 4);
        let mut out = Vec::new();
        g.neighbors(center, &mut out);
        assert_eq!(out.len(), 6);
        for n in &out {
            assert_eq!(g.distance(center, *n), 1);
        }
    }

    #[test]
    fn staggered_hex_distance_independent_of_variant() {
        let hex_grid = HexGrid::flat(1.0);
        // The two axial cells (0,0) and (3,-2) are 3 apart in hex.
        // Same pair viewed in different offset variants must yield
        // the same distance.
        let axial_a = HexCell::new(0, 0);
        let axial_b = HexCell::new(3, -2);
        for variant in [
            HexOffset::OddR,
            HexOffset::EvenR,
            HexOffset::OddQ,
            HexOffset::EvenQ,
        ] {
            let g = StaggeredHexGrid::new(hex_grid, variant);
            let a = axial_to_offset(axial_a, variant);
            let b = axial_to_offset(axial_b, variant);
            assert_eq!(g.distance(a, b), 3, "variant {variant:?}");
        }
    }
}
