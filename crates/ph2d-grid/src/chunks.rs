//! Chunked square grid — Square grid with a coarse "chunk" overlay
//! used by tilemap renderers to batch cell ranges and stream chunks
//! in/out of memory.
//!
//! # Why specialize to Square
//!
//! Tilemap chunking is overwhelmingly used with square cells in the
//! wild (Minecraft chunks, Stardew Valley areas, Terraria sections,
//! etc.). Other grid kinds (Hex / Iso / Tri) can be chunked by
//! wrapping their own `(col, row)`-like coords in the same scheme;
//! we keep this implementation Square-specific for v1 and revisit
//! once a concrete non-square chunking use case appears.

use crate::square::{SquareGrid, SquareNeighborhood};
use crate::{GridMath, Vec2};

/// Square grid with an additional coarse chunk grid laid on top.
///
/// `chunk_size_cells` is the number of inner cells per chunk side
/// (so each chunk covers `chunk_size_cells × chunk_size_cells` cells
/// = `chunk_size_cells * cell_size` world units per side).
#[derive(Copy, Clone, Debug)]
pub struct ChunkedSquareGrid {
    pub inner: SquareGrid,
    /// Number of inner cells per chunk side. Must be ≥ 1.
    pub chunk_size_cells: u32,
}

/// Chunk coordinate `(chunk_x, chunk_y)`.
pub type ChunkCoord = (i32, i32);

impl ChunkedSquareGrid {
    pub fn new(cell_size: f32, chunk_size_cells: u32, neighborhood: SquareNeighborhood) -> Self {
        debug_assert!(chunk_size_cells > 0);
        Self {
            inner: SquareGrid::new(cell_size, neighborhood),
            chunk_size_cells,
        }
    }

    /// Chunk containing the cell.
    pub fn chunk_of(&self, cell: (i32, i32)) -> ChunkCoord {
        let cs = self.chunk_size_cells as i32;
        (cell.0.div_euclid(cs), cell.1.div_euclid(cs))
    }

    /// World-space AABB of the chunk: `(min, max)`.
    pub fn chunk_to_world_rect(&self, chunk: ChunkCoord) -> (Vec2, Vec2) {
        let s = self.inner.cell_size;
        let cs = self.chunk_size_cells as f32;
        let x0 = chunk.0 as f32 * cs * s;
        let y0 = chunk.1 as f32 * cs * s;
        ([x0, y0], [x0 + cs * s, y0 + cs * s])
    }

    /// Bottom-left inner cell of `chunk`. Used by renderers to
    /// iterate the chunk's contents.
    pub fn chunk_origin_cell(&self, chunk: ChunkCoord) -> (i32, i32) {
        let cs = self.chunk_size_cells as i32;
        (chunk.0 * cs, chunk.1 * cs)
    }
}

impl GridMath for ChunkedSquareGrid {
    type Cell = (i32, i32);

    fn world_to_cell(&self, world: Vec2) -> Self::Cell {
        self.inner.world_to_cell(world)
    }

    fn cell_to_world_center(&self, cell: Self::Cell) -> Vec2 {
        self.inner.cell_to_world_center(cell)
    }

    fn cell_to_world_vertices(&self, cell: Self::Cell, out: &mut Vec<Vec2>) {
        self.inner.cell_to_world_vertices(cell, out);
    }

    fn neighbors(&self, cell: Self::Cell, out: &mut Vec<Self::Cell>) {
        self.inner.neighbors(cell, out);
    }

    fn distance(&self, a: Self::Cell, b: Self::Cell) -> u32 {
        self.inner.distance(a, b)
    }

    fn line(&self, a: Self::Cell, b: Self::Cell, out: &mut Vec<Self::Cell>) {
        self.inner.line(a, b, out);
    }

    fn range(&self, center: Self::Cell, radius: u32, out: &mut Vec<Self::Cell>) {
        self.inner.range(center, radius, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_of_groups_cells_correctly() {
        let g = ChunkedSquareGrid::new(1.0, 4, SquareNeighborhood::Von4);
        assert_eq!(g.chunk_of((0, 0)), (0, 0));
        assert_eq!(g.chunk_of((3, 3)), (0, 0));
        assert_eq!(g.chunk_of((4, 0)), (1, 0));
        assert_eq!(g.chunk_of((-1, -1)), (-1, -1));
        assert_eq!(g.chunk_of((-4, 0)), (-1, 0));
    }

    #[test]
    fn chunk_world_rect_matches_cell_world_size() {
        let g = ChunkedSquareGrid::new(2.0, 8, SquareNeighborhood::Von4);
        let (min, max) = g.chunk_to_world_rect((0, 0));
        assert_eq!(min, [0.0, 0.0]);
        assert_eq!(max, [16.0, 16.0]); // 8 cells × 2.0 per cell
    }

    #[test]
    fn chunk_origin_cell_is_lowest_corner() {
        let g = ChunkedSquareGrid::new(1.0, 16, SquareNeighborhood::Moore8);
        assert_eq!(g.chunk_origin_cell((0, 0)), (0, 0));
        assert_eq!(g.chunk_origin_cell((1, 0)), (16, 0));
        assert_eq!(g.chunk_origin_cell((-1, -1)), (-16, -16));
    }

    #[test]
    fn delegates_grid_math_to_inner_square() {
        let g = ChunkedSquareGrid::new(1.0, 4, SquareNeighborhood::Von4);
        let mut nb = Vec::new();
        g.neighbors((0, 0), &mut nb);
        assert_eq!(nb.len(), 4);
        assert_eq!(g.distance((0, 0), (3, 4)), 7); // Manhattan
    }
}
