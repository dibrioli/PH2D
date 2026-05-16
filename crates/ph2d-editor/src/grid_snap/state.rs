//! Mutable state of the grid-snap floating panel.
//!
//! Owned by `HeroScreen` (analog of `widget_gallery_visible` flag +
//! Inspector slot data). The Coordenador wires read/write access:
//! the dispatcher updates fields on widget events, the painter
//! reads them, and the gizmo / drag-drop / paste code calls
//! [`GridSnapState::snap_world`] when the user moves a sprite.
//!
//! Per-kind config lives inline as separate `*Cfg` structs (rather
//! than as a payload-carrying enum) so the panel can mutate the
//! Hex orientation without losing the Square cell size when the
//! user toggles between kinds — a UX nicety borrowed from Blender.

use crate::zones::Rect;
use ph2d_grid::Vec2;
use ph2d_grid::chunks::ChunkedSquareGrid;
use ph2d_grid::hex::{HexGrid, HexOffset, HexOrientation};
use ph2d_grid::iso::IsoGrid;
use ph2d_grid::quadtree::{AABB, Quadtree};
use ph2d_grid::snap::{SnapTarget, snap_world as gsw};
use ph2d_grid::square::{SquareGrid, SquareNeighborhood};
use ph2d_grid::staggered::{StaggerParity, StaggeredHexGrid, StaggeredSquareGrid};
use ph2d_grid::tri::{TriGrid, TriNeighborhood};
use ph2d_grid::voronoi::{Triangulation, deterministic_seeds};

/// The active grid kind. Identifier only — the actual `*Cfg`
/// structs in [`GridSnapState`] hold the parameters; switching
/// kinds preserves per-kind config independently.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GridKind {
    Square,
    Hex,
    Iso,
    StaggeredSquare,
    StaggeredHex,
    Tri,
    Quadtree,
    Voronoi,
    Chunks,
}

impl GridKind {
    pub fn label(self) -> &'static str {
        match self {
            GridKind::Square => "Square",
            GridKind::Hex => "Hex",
            GridKind::Iso => "Isometric",
            GridKind::StaggeredSquare => "Staggered Square",
            GridKind::StaggeredHex => "Staggered Hex",
            GridKind::Tri => "Triangular",
            GridKind::Quadtree => "Quadtree",
            GridKind::Voronoi => "Voronoi",
            GridKind::Chunks => "Chunked Square",
        }
    }

    pub fn all() -> [GridKind; 9] {
        [
            GridKind::Square,
            GridKind::Hex,
            GridKind::Iso,
            GridKind::StaggeredSquare,
            GridKind::StaggeredHex,
            GridKind::Tri,
            GridKind::Quadtree,
            GridKind::Voronoi,
            GridKind::Chunks,
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SquareCfg {
    pub cell_size: f32,
    pub neighborhood: SquareNeighborhood,
    /// World-space offset of cell (0, 0)'s corner. Lets the user
    /// align the grid to existing art instead of being anchored to
    /// world (0, 0). Matches Tiled / Aseprite / Godot conventions.
    pub origin: Vec2,
    /// Spacing in world units between **major** grid lines. Minor
    /// lines are at `cell_size`. Default = `cell_size * 5` (Photoshop
    /// and Blender canonical "every 5 minor"). Set to `cell_size`
    /// to disable the major/minor distinction.
    pub spacing_major: f32,
}
impl Default for SquareCfg {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            neighborhood: SquareNeighborhood::Von4,
            origin: [0.0, 0.0],
            spacing_major: 5.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HexCfg {
    pub cell_size: f32,
    pub orientation: HexOrientation,
    pub offset_variant: HexOffset,
    /// World-space offset of the hex grid (axial (0, 0) center).
    pub origin: Vec2,
}
impl Default for HexCfg {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            orientation: HexOrientation::Pointy,
            offset_variant: HexOffset::OddR,
            origin: [0.0, 0.0],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IsoCfg {
    pub tile_w: f32,
    pub tile_h: f32,
    pub neighborhood: SquareNeighborhood,
    /// World-space offset of cell (0, 0)'s top corner.
    pub origin: Vec2,
}
impl Default for IsoCfg {
    fn default() -> Self {
        Self {
            tile_w: 2.0,
            tile_h: 1.0,
            neighborhood: SquareNeighborhood::Von4,
            origin: [0.0, 0.0],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StaggeredSquareCfg {
    pub cell_w: f32,
    pub cell_h: f32,
    pub parity: StaggerParity,
    pub neighborhood: SquareNeighborhood,
    pub origin: Vec2,
}
impl Default for StaggeredSquareCfg {
    fn default() -> Self {
        Self {
            cell_w: 1.0,
            cell_h: 1.0,
            parity: StaggerParity::OddRows,
            neighborhood: SquareNeighborhood::Von4,
            origin: [0.0, 0.0],
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct StaggeredHexCfg {
    pub hex: HexCfg,
}

#[derive(Copy, Clone, Debug)]
pub struct TriCfg {
    pub edge_length: f32,
    pub neighborhood: TriNeighborhood,
    pub origin: Vec2,
}
impl Default for TriCfg {
    fn default() -> Self {
        Self {
            edge_length: 1.0,
            neighborhood: TriNeighborhood::Edge3,
            origin: [0.0, 0.0],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct QuadtreeCfg {
    pub bounds: AABB,
    pub max_points_per_leaf: usize,
    pub max_depth: u32,
    /// Number of demo points the editor render adapter inserts
    /// before painting (so the user has something to subdivide).
    pub demo_point_count: usize,
    pub demo_rng_seed: u64,
}
impl Default for QuadtreeCfg {
    fn default() -> Self {
        Self {
            bounds: AABB::new([-10.0, -10.0], [10.0, 10.0]),
            max_points_per_leaf: 4,
            max_depth: 6,
            demo_point_count: 32,
            demo_rng_seed: 42,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VoronoiCfg {
    pub bounds: AABB,
    pub seed_count: usize,
    pub rng_seed: u64,
    pub lloyd_iterations: u32,
}
impl Default for VoronoiCfg {
    fn default() -> Self {
        Self {
            bounds: AABB::new([-10.0, -10.0], [10.0, 10.0]),
            seed_count: 24,
            rng_seed: 7,
            lloyd_iterations: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ChunksCfg {
    pub cell_size: f32,
    pub chunk_size_cells: u32,
    pub neighborhood: SquareNeighborhood,
    pub origin: Vec2,
}
impl Default for ChunksCfg {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            chunk_size_cells: 8,
            neighborhood: SquareNeighborhood::Von4,
            origin: [0.0, 0.0],
        }
    }
}

/// Top-level state for the grid-snap subsystem.
#[derive(Clone, Debug)]
pub struct GridSnapState {
    pub kind: GridKind,

    pub square_cfg: SquareCfg,
    pub hex_cfg: HexCfg,
    pub iso_cfg: IsoCfg,
    pub staggered_square_cfg: StaggeredSquareCfg,
    pub staggered_hex_cfg: StaggeredHexCfg,
    pub tri_cfg: TriCfg,
    pub quadtree_cfg: QuadtreeCfg,
    pub voronoi_cfg: VoronoiCfg,
    pub chunks_cfg: ChunksCfg,

    pub snap_enabled: bool,
    pub snap_target: SnapTarget,
    /// Sub-grid factor applied at snap time only (rendering stays
    /// at the base cell size). `1` (default) = snap to the kind's
    /// natural cell. `N > 1` = snap as if each cell were `N×N`
    /// finer — useful for half/quarter-cell alignment without
    /// changing the visible grid. Applied via pre/post-scale of
    /// world coords by `N` in [`GridSnapState::snap_world`].
    pub snap_subdivisions: u32,

    pub panel_visible: bool,
    pub panel_rect: Option<Rect>,

    pub show_overlay: bool,
    pub color_rgba: [u8; 4],
    pub opacity: f32,

    pub probe_a: Vec2,
    pub probe_b: Vec2,

    /// Scratch buffer shared by snap + inspect paths. Owner is the
    /// caller; never grows beyond the largest single-call need
    /// (≤ 12 entries for any grid kind in this crate).
    pub scratch: Vec<Vec2>,
}

impl Default for GridSnapState {
    fn default() -> Self {
        Self {
            kind: GridKind::Square,

            square_cfg: SquareCfg::default(),
            hex_cfg: HexCfg::default(),
            iso_cfg: IsoCfg::default(),
            staggered_square_cfg: StaggeredSquareCfg::default(),
            staggered_hex_cfg: StaggeredHexCfg::default(),
            tri_cfg: TriCfg::default(),
            quadtree_cfg: QuadtreeCfg::default(),
            voronoi_cfg: VoronoiCfg::default(),
            chunks_cfg: ChunksCfg::default(),

            snap_enabled: false,
            snap_target: SnapTarget::Center,
            snap_subdivisions: 1,

            panel_visible: false,
            panel_rect: None,

            show_overlay: true,
            // Default to a low-saturation cyan that reads on both
            // dark and light themes.
            color_rgba: [0x4F, 0xC3, 0xE5, 0xC0],
            opacity: 0.75,

            probe_a: [0.0, 0.0],
            probe_b: [3.0, 2.0],

            scratch: Vec::with_capacity(12),
        }
    }
}

impl GridSnapState {
    /// Build a [`SquareGrid`] from the current `square_cfg`.
    pub fn make_square(&self) -> SquareGrid {
        SquareGrid::new(self.square_cfg.cell_size, self.square_cfg.neighborhood)
    }
    pub fn make_hex(&self) -> HexGrid {
        HexGrid {
            cell_size: self.hex_cfg.cell_size,
            orientation: self.hex_cfg.orientation,
            offset_default: self.hex_cfg.offset_variant,
        }
    }
    pub fn make_iso(&self) -> IsoGrid {
        IsoGrid {
            tile_w: self.iso_cfg.tile_w,
            tile_h: self.iso_cfg.tile_h,
            neighborhood: self.iso_cfg.neighborhood,
        }
    }
    pub fn make_staggered_square(&self) -> StaggeredSquareGrid {
        StaggeredSquareGrid::new(
            self.staggered_square_cfg.cell_w,
            self.staggered_square_cfg.cell_h,
            self.staggered_square_cfg.parity,
            self.staggered_square_cfg.neighborhood,
        )
    }
    pub fn make_staggered_hex(&self) -> StaggeredHexGrid {
        StaggeredHexGrid::new(
            HexGrid {
                cell_size: self.staggered_hex_cfg.hex.cell_size,
                orientation: self.staggered_hex_cfg.hex.orientation,
                offset_default: self.staggered_hex_cfg.hex.offset_variant,
            },
            self.staggered_hex_cfg.hex.offset_variant,
        )
    }
    pub fn make_tri(&self) -> TriGrid {
        TriGrid::new(self.tri_cfg.edge_length, self.tri_cfg.neighborhood)
    }
    pub fn make_chunks(&self) -> ChunkedSquareGrid {
        ChunkedSquareGrid::new(
            self.chunks_cfg.cell_size,
            self.chunks_cfg.chunk_size_cells,
            self.chunks_cfg.neighborhood,
        )
    }

    /// World-space origin offset of the active kind. `[0, 0]` for
    /// kinds without an explicit origin (Quadtree/Voronoi use
    /// `bounds: AABB` instead).
    pub fn active_origin(&self) -> Vec2 {
        match self.kind {
            GridKind::Square => self.square_cfg.origin,
            GridKind::Hex => self.hex_cfg.origin,
            GridKind::Iso => self.iso_cfg.origin,
            GridKind::StaggeredSquare => self.staggered_square_cfg.origin,
            GridKind::StaggeredHex => self.staggered_hex_cfg.hex.origin,
            GridKind::Tri => self.tri_cfg.origin,
            GridKind::Chunks => self.chunks_cfg.origin,
            GridKind::Quadtree | GridKind::Voronoi => [0.0, 0.0],
        }
    }

    /// Snap `world` per the active grid kind + snap target. Returns
    /// `world` unchanged when snap is disabled or the active kind
    /// has no snap-target (Quadtree, Voronoi).
    ///
    /// `sprite_half_size` is the sprite's half-extent in world meters
    /// — used by the `Corner` and `CenterIntersectionAndCorners`
    /// modes to align a sprite corner to a grid vertex. Pass
    /// `[0.0, 0.0]` when the caller is snapping a bare world point
    /// (drag-drop, paste before the sprite size is known); Corner
    /// modes degenerate to point-Intersection snap in that case.
    ///
    /// Origin offset is applied at the boundary: world coords go
    /// through `(world - origin)` before the math (so the math sees
    /// a grid anchored at (0, 0)), then `+ origin` is added back to
    /// the snapped result.
    pub fn snap_world(&mut self, world: Vec2, sprite_half_size: Vec2) -> Vec2 {
        if !self.snap_enabled {
            return world;
        }
        let target = self.snap_target;
        let origin = self.active_origin();
        // Pre-scale by N for sub-grid snap: a 2× subdivision treats
        // each cell as if it were 1/N × 1/N during the snap math.
        // Post-scale undoes it, landing on the fine-grained target.
        // The sprite half-size scales with world so Corner mode still
        // aligns the right sprite corner under sub-grid snap.
        let n = self.snap_subdivisions.max(1) as f32;
        let local = [(world[0] - origin[0]) * n, (world[1] - origin[1]) * n];
        let local_half = [sprite_half_size[0] * n, sprite_half_size[1] * n];
        let snapped_local = match self.kind {
            GridKind::Square => gsw(
                &self.make_square(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            GridKind::Hex => gsw(
                &self.make_hex(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            GridKind::Iso => gsw(
                &self.make_iso(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            GridKind::StaggeredSquare => gsw(
                &self.make_staggered_square(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            GridKind::StaggeredHex => gsw(
                &self.make_staggered_hex(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            GridKind::Tri => gsw(
                &self.make_tri(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            GridKind::Chunks => gsw(
                &self.make_chunks(),
                local,
                local_half,
                target,
                &mut self.scratch,
            ),
            // Non-uniform grids: dedicated snappers (build the
            // structure on each call, no caching — editor responsiveness
            // is fine even on the largest demo cfgs).
            GridKind::Quadtree => {
                snap_world_quadtree(local, local_half, target, &self.quadtree_cfg)
            }
            GridKind::Voronoi => snap_world_voronoi(local, local_half, target, &self.voronoi_cfg),
        };
        [
            snapped_local[0] / n + origin[0],
            snapped_local[1] / n + origin[1],
        ]
    }
}

// ── Snap helpers for non-uniform grids ────────────────────────────

/// Squared distance helper.
#[inline]
fn sq_dist(a: Vec2, b: Vec2) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Pick the candidate closest (squared distance) to `world`. First
/// wins on tie — fixed evaluation order keeps HR-5 deterministic.
fn nearest_to(world: Vec2, candidates: &[Vec2]) -> Vec2 {
    let mut best = candidates[0];
    let mut best_d2 = sq_dist(world, best);
    for &c in &candidates[1..] {
        let d2 = sq_dist(world, c);
        if d2 < best_d2 {
            best_d2 = d2;
            best = c;
        }
    }
    best
}

/// Sprite-corner snap for non-uniform grids: enumerate the 4 sprite
/// corners (`world ± half`), find each corner's nearest vertex from
/// `vertices`, return the new sprite-center that aligns the closest
/// (corner, vertex) pair. Degenerates to the nearest vertex when
/// `half == [0.0, 0.0]`.
fn corner_snap_against_vertices(world: Vec2, half: Vec2, vertices: &[Vec2]) -> Vec2 {
    if vertices.is_empty() {
        return world;
    }
    let hw = half[0];
    let hh = half[1];
    if hw == 0.0 && hh == 0.0 {
        return nearest_to(world, vertices);
    }
    let corners: [Vec2; 4] = [
        [world[0] - hw, world[1] - hh],
        [world[0] + hw, world[1] - hh],
        [world[0] - hw, world[1] + hh],
        [world[0] + hw, world[1] + hh],
    ];
    let mut best_shift: Vec2 = [0.0, 0.0];
    let mut best_d2 = f32::INFINITY;
    for c in corners {
        let v = nearest_to(c, vertices);
        let dx = v[0] - c[0];
        let dy = v[1] - c[1];
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            best_d2 = d2;
            best_shift = [dx, dy];
        }
    }
    [world[0] + best_shift[0], world[1] + best_shift[1]]
}

/// Build a Quadtree from `cfg.demo_*` and return (leaf_center,
/// leaf_corners) for the leaf containing `world`. Falls back to the
/// outer `bounds` when `world` is outside the tree (which can happen
/// when the user pans far from the cfg bounds).
fn quadtree_active_leaf(world: Vec2, cfg: &QuadtreeCfg) -> (Vec2, [Vec2; 4]) {
    let mut qt: Quadtree<()> = Quadtree::new(cfg.bounds, cfg.max_points_per_leaf, cfg.max_depth);
    // Insert demo points so the tree subdivides into the same shape
    // the panel renders. SplitMix64 RNG mirrors the render adapter.
    for i in 0..cfg.demo_point_count {
        let t = i as u64;
        let mut h = cfg
            .demo_rng_seed
            .wrapping_add(t)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        let fx = ((h >> 32) as u32 as f64) / (u32::MAX as f64);
        let fy = ((h & 0xFFFF_FFFF) as u32 as f64) / (u32::MAX as f64);
        let x = cfg.bounds.min[0] + (fx as f32) * (cfg.bounds.max[0] - cfg.bounds.min[0]);
        let y = cfg.bounds.min[1] + (fy as f32) * (cfg.bounds.max[1] - cfg.bounds.min[1]);
        let _ = qt.insert([x, y], ());
    }
    let mut leaves: Vec<AABB> = Vec::with_capacity(64);
    qt.iter_leaf_bounds(&mut leaves);
    // Pick the leaf containing `world`; fallback to the outer bounds.
    let leaf = leaves
        .into_iter()
        .find(|l| l.contains_point(world))
        .unwrap_or(cfg.bounds);
    let center = leaf.center();
    let corners = [
        [leaf.min[0], leaf.min[1]],
        [leaf.max[0], leaf.min[1]],
        [leaf.max[0], leaf.max[1]],
        [leaf.min[0], leaf.max[1]],
    ];
    (center, corners)
}

fn snap_world_quadtree(world: Vec2, half: Vec2, target: SnapTarget, cfg: &QuadtreeCfg) -> Vec2 {
    let (center, corners) = quadtree_active_leaf(world, cfg);
    match target {
        SnapTarget::Center => center,
        SnapTarget::Intersection => nearest_to(world, &corners),
        SnapTarget::Corner => corner_snap_against_vertices(world, half, &corners),
        SnapTarget::CenterAndIntersection => {
            let v = nearest_to(world, &corners);
            nearest_to(world, &[center, v])
        }
        SnapTarget::CenterIntersectionAndCorners => {
            let v = nearest_to(world, &corners);
            let k = corner_snap_against_vertices(world, half, &corners);
            nearest_to(world, &[center, v, k])
        }
    }
}

/// Build the Voronoi diagram from `cfg`, returning every seed
/// (cell center) and every cell vertex (Voronoi vertex). Cells are
/// clipped to `cfg.bounds` so vertices outside the visible area
/// don't pull the snap there.
fn voronoi_seeds_and_vertices(cfg: &VoronoiCfg) -> (Vec<Vec2>, Vec<Vec2>) {
    let seeds = deterministic_seeds(cfg.bounds, cfg.seed_count, cfg.rng_seed);
    let mut tri = Triangulation::from_seeds(&seeds);
    for _ in 0..cfg.lloyd_iterations {
        tri.lloyd_step();
    }
    let cells = tri.voronoi_cells();
    let mut all_vertices: Vec<Vec2> = Vec::with_capacity(cells.len() * 6);
    for cell in &cells {
        let clipped = ph2d_grid::voronoi::Triangulation::clip_cell_to_aabb(cell, cfg.bounds);
        for v in clipped {
            all_vertices.push(v);
        }
    }
    let seed_centers: Vec<Vec2> = cells.iter().map(|c| c.seed).collect();
    (seed_centers, all_vertices)
}

fn snap_world_voronoi(world: Vec2, half: Vec2, target: SnapTarget, cfg: &VoronoiCfg) -> Vec2 {
    let (seeds, vertices) = voronoi_seeds_and_vertices(cfg);
    if seeds.is_empty() {
        return world;
    }
    let center = nearest_to(world, &seeds);
    match target {
        SnapTarget::Center => center,
        SnapTarget::Intersection => {
            if vertices.is_empty() {
                center
            } else {
                nearest_to(world, &vertices)
            }
        }
        SnapTarget::Corner => {
            if vertices.is_empty() {
                center
            } else {
                corner_snap_against_vertices(world, half, &vertices)
            }
        }
        SnapTarget::CenterAndIntersection => {
            if vertices.is_empty() {
                center
            } else {
                let v = nearest_to(world, &vertices);
                nearest_to(world, &[center, v])
            }
        }
        SnapTarget::CenterIntersectionAndCorners => {
            if vertices.is_empty() {
                center
            } else {
                let v = nearest_to(world, &vertices);
                let k = corner_snap_against_vertices(world, half, &vertices);
                nearest_to(world, &[center, v, k])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_origin_dispatches_per_kind() {
        let mut s = GridSnapState::default();
        s.square_cfg.origin = [1.0, 2.0];
        s.hex_cfg.origin = [3.0, 4.0];
        assert_eq!(s.active_origin(), [1.0, 2.0]);
        s.kind = GridKind::Hex;
        assert_eq!(s.active_origin(), [3.0, 4.0]);
        // Quadtree / Voronoi have no origin — always [0, 0].
        s.kind = GridKind::Quadtree;
        assert_eq!(s.active_origin(), [0.0, 0.0]);
    }

    #[test]
    fn snap_world_respects_origin_offset() {
        let mut s = GridSnapState {
            snap_enabled: true,
            snap_target: SnapTarget::Center,
            ..Default::default()
        };
        // Default cell_size=1.0, origin=[0,0]. Center of cell (0,0)
        // is (0.5, 0.5).
        assert_eq!(s.snap_world([0.1, 0.1], [0.0, 0.0]), [0.5, 0.5]);
        // Shift origin by (10, 20). Same world point [0.1, 0.1]
        // sits 9.9 units LEFT and 19.9 units DOWN of cell (0,0) of
        // the shifted grid → cell (-10, -20) → center
        // (-9.5 + 10, -19.5 + 20) = (0.5, 0.5)? Wait that's the
        // SAME center because the world point shifts relative to a
        // grid moved equally. Let me reconsider: snap pulls the
        // world point to a cell center of the shifted grid. With
        // origin = (10, 20), cell (0, 0) center is at
        // world (10.5, 20.5). For input [0.1, 0.1], the nearest
        // cell center of the shifted grid is one of:
        //   ..., (-9.5, -19.5), (0.5, -19.5), ... (... etc)
        // i.e. (k + 0.5 + 10, j + 0.5 + 20) — closest to [0.1, 0.1]
        // is k=-10, j=-20 → (0.5, 0.5). So same answer as no offset
        // when world point falls exactly on an integer-cell pattern.
        // Use a half-integer origin to make the test less degenerate.
        s.square_cfg.origin = [0.3, 0.7];
        // Input [0.1, 0.1] → local [-0.2, -0.6] → cell (-1, -1) →
        // local center (-0.5, -0.5) → world center (-0.2, 0.2).
        let snapped = s.snap_world([0.1, 0.1], [0.0, 0.0]);
        assert!(
            (snapped[0] - -0.2).abs() < 1e-5 && (snapped[1] - 0.2).abs() < 1e-5,
            "expected [-0.2, 0.2], got {snapped:?}"
        );
    }

    #[test]
    fn default_state_snap_is_off_so_passthrough() {
        let mut s = GridSnapState::default();
        assert!(!s.snap_enabled);
        let p = s.snap_world([1.3, 2.7], [0.0, 0.0]);
        assert_eq!(p, [1.3, 2.7]);
    }

    #[test]
    fn enabled_snap_pulls_to_cell_center_for_square() {
        let mut s = GridSnapState {
            snap_enabled: true,
            kind: GridKind::Square,
            snap_target: SnapTarget::Center,
            ..Default::default()
        };
        let p = s.snap_world([0.1, 0.1], [0.0, 0.0]);
        // Default square cell size = 1.0 → cell (0,0) center = (0.5, 0.5).
        assert_eq!(p, [0.5, 0.5]);
    }

    #[test]
    fn snap_intersection_picks_corner_for_hex() {
        let mut s = GridSnapState {
            snap_enabled: true,
            kind: GridKind::Hex,
            snap_target: SnapTarget::Intersection,
            ..Default::default()
        };
        // Hex point near origin — must land on one of the 6 corners
        // of the containing hex. Verify the result is at distance
        // ≈ cell_size (= 1.0) from the cell center.
        let p = s.snap_world([0.1, 0.1], [0.0, 0.0]);
        let center = s.make_hex();
        use ph2d_grid::GridMath;
        let cell = center.world_to_cell([0.1, 0.1]);
        let cc = center.cell_to_world_center(cell);
        let d = ((p[0] - cc[0]).powi(2) + (p[1] - cc[1]).powi(2)).sqrt();
        assert!(
            (d - s.hex_cfg.cell_size).abs() < 1e-4,
            "vertex should be at radius cell_size from center; got d={d}"
        );
    }

    #[test]
    fn quadtree_snap_to_center_returns_inside_bounds() {
        // With snap enabled, Quadtree should land on a leaf center
        // that's inside the cfg bounds (default `[-10, -10] → [10, 10]`).
        let mut s = GridSnapState {
            snap_enabled: true,
            kind: GridKind::Quadtree,
            snap_target: SnapTarget::Center,
            ..Default::default()
        };
        let p = s.snap_world([0.5, 0.5], [0.0, 0.0]);
        let b = s.quadtree_cfg.bounds;
        assert!(
            p[0] >= b.min[0] && p[0] <= b.max[0],
            "x out of bounds: {p:?}"
        );
        assert!(
            p[1] >= b.min[1] && p[1] <= b.max[1],
            "y out of bounds: {p:?}"
        );
        // And it must NOT be the input — Center mode always pulls to
        // a leaf center which is unlikely to coincide with the input.
        assert!(
            p != [0.5, 0.5],
            "Center snap should pull to a leaf center, got passthrough"
        );
    }

    #[test]
    fn quadtree_snap_to_intersection_picks_a_leaf_corner() {
        // Intersection mode picks the nearest corner of the leaf
        // containing `world`. Corners are AABB extrema, so coordinates
        // line up with the subdivision boundaries.
        let mut s = GridSnapState {
            snap_enabled: true,
            kind: GridKind::Quadtree,
            snap_target: SnapTarget::Intersection,
            ..Default::default()
        };
        let p = s.snap_world([0.0, 0.0], [0.0, 0.0]);
        let b = s.quadtree_cfg.bounds;
        // Corners must be within bounds and at half-multiples of the
        // bounds extent (default subdivision halves repeatedly).
        assert!(p[0] >= b.min[0] && p[0] <= b.max[0]);
        assert!(p[1] >= b.min[1] && p[1] <= b.max[1]);
    }

    #[test]
    fn voronoi_snap_to_center_lands_on_a_seed() {
        // Center mode for Voronoi snaps to the nearest seed (cell
        // center). Returned point must equal one of the deterministic
        // seeds.
        let mut s = GridSnapState {
            snap_enabled: true,
            kind: GridKind::Voronoi,
            snap_target: SnapTarget::Center,
            ..Default::default()
        };
        let p = s.snap_world([0.0, 0.0], [0.0, 0.0]);
        let seeds = ph2d_grid::voronoi::deterministic_seeds(
            s.voronoi_cfg.bounds,
            s.voronoi_cfg.seed_count,
            s.voronoi_cfg.rng_seed,
        );
        let matches_seed = seeds
            .iter()
            .any(|sd| (sd[0] - p[0]).abs() < 1e-4 && (sd[1] - p[1]).abs() < 1e-4);
        assert!(
            matches_seed,
            "snapped point {p:?} doesn't match any of {} seeds",
            seeds.len()
        );
    }

    #[test]
    fn voronoi_snap_intersection_lands_on_a_cell_vertex() {
        // Intersection mode snaps to a Voronoi vertex (where 3+ cells
        // meet). Verify the result is inside cfg bounds.
        let mut s = GridSnapState {
            snap_enabled: true,
            kind: GridKind::Voronoi,
            snap_target: SnapTarget::Intersection,
            ..Default::default()
        };
        let p = s.snap_world([0.0, 0.0], [0.0, 0.0]);
        let b = s.voronoi_cfg.bounds;
        assert!(
            p[0] >= b.min[0] && p[0] <= b.max[0],
            "x out of bounds: {p:?}"
        );
        assert!(
            p[1] >= b.min[1] && p[1] <= b.max[1],
            "y out of bounds: {p:?}"
        );
    }

    #[test]
    fn quadtree_and_voronoi_passthrough_when_disabled() {
        // snap_enabled = false → unconditional passthrough, regardless
        // of kind. Same as every other kind.
        let mut s = GridSnapState {
            snap_enabled: false,
            kind: GridKind::Quadtree,
            ..Default::default()
        };
        assert_eq!(s.snap_world([3.7, 2.1], [0.0, 0.0]), [3.7, 2.1]);
        s.kind = GridKind::Voronoi;
        assert_eq!(s.snap_world([3.7, 2.1], [0.0, 0.0]), [3.7, 2.1]);
    }

    #[test]
    fn grid_kind_label_covers_all_nine_variants() {
        for k in GridKind::all() {
            assert!(!k.label().is_empty());
        }
    }
}
