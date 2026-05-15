#![forbid(unsafe_code)]
//! ph2d-grid — grid kinds + snap math, no UI deps.
//!
//! Pure-math primitives for the 11 grid kinds the editor's
//! [grid-snap subsystem](../../ph2d-editor/src/grid_snap/) and any
//! gameplay code can share. Zero deps outside `core`/`std`; safe to
//! link from `ph2d-core` (HR-1: core platform-agnostic).
//!
//! # Buffer-reuse contract (HR-3)
//!
//! Trait methods that return a variable-length collection take a
//! `&mut Vec<Self::Cell>` (or `&mut Vec<Vec2>` for vertices). The
//! function **clears** the buffer before pushing — callers reuse the
//! same `Vec` across frames to keep the steady-state allocation cost
//! at zero. First call grows; subsequent calls within capacity touch
//! no allocator.
//!
//! # Determinism (HR-5)
//!
//! No `HashMap`, no `f32::mul_add` (would be FMA-dependent), no fast-
//! math intrinsics. RNG-driven kinds (Voronoi seeds) take an explicit
//! `u64` seed; identical seeds reproduce identical output across
//! platforms.

pub mod chunks;
pub mod hex;
pub mod iso;
pub mod quadtree;
pub mod square;
pub mod staggered;
pub mod tri;
pub mod voronoi;

/// 2D world position in meters. Matches the editor's existing
/// `[f32; 2]` convention (see `crates/ph2d-editor/src/grid.rs`).
pub type Vec2 = [f32; 2];

/// Common interface every regular grid (Square / Hex / Iso /
/// Staggered / Tri / Chunks) implements.
///
/// Quadtree and Voronoi have non-uniform cells and expose their own
/// APIs — they don't fit a single `Cell` associated type cleanly.
pub trait GridMath {
    /// Stable identifier for one cell. Must be `Copy + Eq + Ord +
    /// Hash` so callers can build deterministic sets / maps over
    /// cells (HR-5 forbids `HashMap` in sim paths, but `BTreeMap` /
    /// `BTreeSet` are fine and need `Ord`).
    type Cell: Copy + Eq + Ord + core::hash::Hash + core::fmt::Debug;

    /// World → cell. The cell whose interior contains `world` (ties
    /// resolved deterministically — see per-impl docs).
    fn world_to_cell(&self, world: Vec2) -> Self::Cell;

    /// Center of `cell` in world coordinates.
    fn cell_to_world_center(&self, cell: Self::Cell) -> Vec2;

    /// Polygon vertices of `cell` in world coordinates, CCW order.
    /// Clears + pushes. For Square: 4 vertices. Hex: 6. Tri: 3.
    fn cell_to_world_vertices(&self, cell: Self::Cell, out: &mut Vec<Vec2>);

    /// Cells adjacent to `cell` under the grid's active
    /// neighborhood policy. Clears + pushes. Order is implementation-
    /// defined but stable for a fixed grid configuration.
    fn neighbors(&self, cell: Self::Cell, out: &mut Vec<Self::Cell>);

    /// Graph distance between two cells (count of edge traversals
    /// along the active neighborhood — Manhattan for Square Von4,
    /// Chebyshev for Moore8, hex distance for Hex, etc.).
    fn distance(&self, a: Self::Cell, b: Self::Cell) -> u32;

    /// Cells on the discrete line from `a` to `b`, inclusive on both
    /// ends. Clears + pushes. Order: a … b.
    fn line(&self, a: Self::Cell, b: Self::Cell, out: &mut Vec<Self::Cell>);

    /// All cells `c` such that `self.distance(center, c) <= radius`,
    /// including `center`. Clears + pushes. Order is implementation-
    /// defined but stable.
    fn range(&self, center: Self::Cell, radius: u32, out: &mut Vec<Self::Cell>);

    /// Snap `world` to the center of its containing cell.
    ///
    /// Default impl composes `cell_to_world_center(world_to_cell(world))`
    /// — overrides should be rare. Zero-alloc (no buffers used).
    fn snap_to_center(&self, world: Vec2) -> Vec2 {
        self.cell_to_world_center(self.world_to_cell(world))
    }

    /// Snap `world` to the nearest cell vertex (corner / intersection).
    ///
    /// Default impl enumerates the containing cell's vertices via
    /// [`cell_to_world_vertices`] and returns the closest one. Uses
    /// a scratch buffer the caller owns; pass an empty `Vec` for
    /// one-off snaps, or reuse across calls for zero alloc.
    ///
    /// [`cell_to_world_vertices`]: GridMath::cell_to_world_vertices
    fn snap_to_nearest_vertex(&self, world: Vec2, scratch: &mut Vec<Vec2>) -> Vec2 {
        let cell = self.world_to_cell(world);
        self.cell_to_world_vertices(cell, scratch);
        debug_assert!(
            !scratch.is_empty(),
            "cell_to_world_vertices must produce ≥ 1 vertex"
        );
        let mut best = scratch[0];
        let mut best_d2 = f32::INFINITY;
        for v in scratch.iter() {
            let d2 = (v[0] - world[0]).powi(2) + (v[1] - world[1]).powi(2);
            if d2 < best_d2 {
                best_d2 = d2;
                best = *v;
            }
        }
        best
    }
}

pub mod astar;
pub mod snap;
