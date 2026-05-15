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
use ph2d_grid::quadtree::AABB;
use ph2d_grid::snap::{SnapTarget, snap_world as gsw};
use ph2d_grid::square::{SquareGrid, SquareNeighborhood};
use ph2d_grid::staggered::{StaggerParity, StaggeredHexGrid, StaggeredSquareGrid};
use ph2d_grid::tri::{TriGrid, TriNeighborhood};

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
}
impl Default for SquareCfg {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            neighborhood: SquareNeighborhood::Von4,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HexCfg {
    pub cell_size: f32,
    pub orientation: HexOrientation,
    pub offset_variant: HexOffset,
}
impl Default for HexCfg {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            orientation: HexOrientation::Pointy,
            offset_variant: HexOffset::OddR,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IsoCfg {
    pub tile_w: f32,
    pub tile_h: f32,
    pub neighborhood: SquareNeighborhood,
}
impl Default for IsoCfg {
    fn default() -> Self {
        Self {
            tile_w: 2.0,
            tile_h: 1.0,
            neighborhood: SquareNeighborhood::Von4,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StaggeredSquareCfg {
    pub cell_w: f32,
    pub cell_h: f32,
    pub parity: StaggerParity,
    pub neighborhood: SquareNeighborhood,
}
impl Default for StaggeredSquareCfg {
    fn default() -> Self {
        Self {
            cell_w: 1.0,
            cell_h: 1.0,
            parity: StaggerParity::OddRows,
            neighborhood: SquareNeighborhood::Von4,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StaggeredHexCfg {
    pub hex: HexCfg,
}
impl Default for StaggeredHexCfg {
    fn default() -> Self {
        Self {
            hex: HexCfg::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TriCfg {
    pub edge_length: f32,
    pub neighborhood: TriNeighborhood,
}
impl Default for TriCfg {
    fn default() -> Self {
        Self {
            edge_length: 1.0,
            neighborhood: TriNeighborhood::Edge3,
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
}
impl Default for ChunksCfg {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            chunk_size_cells: 8,
            neighborhood: SquareNeighborhood::Von4,
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

    /// Snap `world` per the active grid kind + snap target. Returns
    /// `world` unchanged when snap is disabled or the active kind
    /// has no snap-target (Quadtree, Voronoi).
    pub fn snap_world(&mut self, world: Vec2) -> Vec2 {
        if !self.snap_enabled {
            return world;
        }
        let target = self.snap_target;
        match self.kind {
            GridKind::Square => gsw(&self.make_square(), world, target, &mut self.scratch),
            GridKind::Hex => gsw(&self.make_hex(), world, target, &mut self.scratch),
            GridKind::Iso => gsw(&self.make_iso(), world, target, &mut self.scratch),
            GridKind::StaggeredSquare => {
                gsw(&self.make_staggered_square(), world, target, &mut self.scratch)
            }
            GridKind::StaggeredHex => {
                gsw(&self.make_staggered_hex(), world, target, &mut self.scratch)
            }
            GridKind::Tri => gsw(&self.make_tri(), world, target, &mut self.scratch),
            GridKind::Chunks => gsw(&self.make_chunks(), world, target, &mut self.scratch),
            // Non-uniform cells have no canonical snap target.
            GridKind::Quadtree | GridKind::Voronoi => world,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_snap_is_off_so_passthrough() {
        let mut s = GridSnapState::default();
        assert!(!s.snap_enabled);
        let p = s.snap_world([1.3, 2.7]);
        assert_eq!(p, [1.3, 2.7]);
    }

    #[test]
    fn enabled_snap_pulls_to_cell_center_for_square() {
        let mut s = GridSnapState::default();
        s.snap_enabled = true;
        s.kind = GridKind::Square;
        s.snap_target = SnapTarget::Center;
        let p = s.snap_world([0.1, 0.1]);
        // Default square cell size = 1.0 → cell (0,0) center = (0.5, 0.5).
        assert_eq!(p, [0.5, 0.5]);
    }

    #[test]
    fn snap_intersection_picks_corner_for_hex() {
        let mut s = GridSnapState::default();
        s.snap_enabled = true;
        s.kind = GridKind::Hex;
        s.snap_target = SnapTarget::Intersection;
        // Hex point near origin — must land on one of the 6 corners
        // of the containing hex. Verify the result is at distance
        // ≈ cell_size (= 1.0) from the cell center.
        let p = s.snap_world([0.1, 0.1]);
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
    fn quadtree_and_voronoi_passthrough_even_when_enabled() {
        let mut s = GridSnapState::default();
        s.snap_enabled = true;
        s.kind = GridKind::Quadtree;
        assert_eq!(s.snap_world([3.7, 2.1]), [3.7, 2.1]);
        s.kind = GridKind::Voronoi;
        assert_eq!(s.snap_world([3.7, 2.1]), [3.7, 2.1]);
    }

    #[test]
    fn grid_kind_label_covers_all_nine_variants() {
        for k in GridKind::all() {
            assert!(!k.label().is_empty());
        }
    }
}
