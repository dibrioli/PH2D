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
use ph2d_grid::snap::{SnapTarget, snap_distance_for_target};
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
    /// World-space radius (meters) within which the cursor / sprite
    /// is ATTRACTED to a snap target. Outside this radius the snap
    /// is a no-op — the host gets the raw world point back so the
    /// drag stays smooth between snap points (Figma / Blender
    /// "magnetic" behavior). `0.0` disables magnetism (back-compat
    /// with the always-snap semantics; equivalent to the legacy
    /// behavior). Scales with subdivisions inside `snap_world` so
    /// the active sub-cell still gets a proportionally smaller
    /// attraction zone.
    pub snap_magnetism_radius: f32,

    /// Persisted floating-panel rect for the Grid Snap chrome
    /// (drag/resize survives close+reopen). Lazy-initialized to
    /// [`super::default_rect_for_panel`] on first paint. Phase C.4
    /// kept this field on `GridSnapState` (rather than splitting it
    /// into a separate `GridSnapPanelState`) because the canvas
    /// renderer never reads it — only the panel chrome does — and
    /// splitting added more friction than it removed. Visibility
    /// migrated to `HeroScreen::panel_visibility` keyed `"grid_snap"`.
    pub panel_rect: Option<Rect>,

    pub show_overlay: bool,
    /// When `true` the grid overlay paints ON TOP of sprites (the
    /// canonical Photoshop / Figma behavior — grid is a reference
    /// affordance for the artist). When `false`, the grid renders
    /// BEHIND sprites so the artwork visually occludes grid lines
    /// where it overlaps. Driven by the "Layer" segmented toggle in
    /// the Grid Settings panel.
    ///
    /// **Visual implementation note (2026-05-15)**: the renderer's
    /// compositor currently runs a single Vello-over-game blend
    /// (`game_rt_ldr` + `vello_intermediate` → swap chain), so the
    /// chrome layer always sits visually on top of the sprite layer.
    /// True "behind" rendering needs a second Vello intermediate
    /// painted under the game pass + a 3-layer compositor shader. The
    /// state field + UI ship now; the renderer wiring lands in a
    /// follow-up commit so the toggle becomes visually meaningful.
    pub grid_in_front: bool,
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
            // Default 0.30 m — feels right for a 1 m default cell:
            // the inner ~60% of the cell is free-drag, the outer
            // 30% ring near each vertex / center is magnetic. Tune
            // via the panel.
            snap_magnetism_radius: 0.30,

            panel_rect: None,

            show_overlay: true,
            // Default to in-front so users see the grid (the
            // canonical artist tool affordance) without needing to
            // discover the toggle.
            grid_in_front: true,
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
        let subdivisions = self.snap_subdivisions.max(1);
        let nf = subdivisions as f32;
        let local = [world[0] - origin[0], world[1] - origin[1]];
        // Magnetism radius in WORLD units. `<= 0.0` disables the
        // threshold (legacy always-snap behavior — kept for the
        // back-compat test suite). The uniform-grid path multiplies
        // `local` by `nf`, so the distance returned from
        // `snap_distance_for_target` is also in scaled space; we
        // pre-multiply the radius by `nf` so the gate threshold lives
        // in the SAME scaled frame and the world-meter semantic is
        // preserved at any subdivision.
        let mag_radius = self.snap_magnetism_radius;
        let snapped_local = match self.kind {
            // Uniform grids: pre-scale by N so each cell is treated
            // as 1/N × 1/N during the snap math, then post-scale to
            // land on the fine-grained target. Sprite half-size scales
            // along so Corner mode keeps aligning the sprite corner.
            GridKind::Square
            | GridKind::Hex
            | GridKind::Iso
            | GridKind::StaggeredSquare
            | GridKind::StaggeredHex
            | GridKind::Tri
            | GridKind::Chunks => {
                let scaled = [local[0] * nf, local[1] * nf];
                let scaled_half = [sprite_half_size[0] * nf, sprite_half_size[1] * nf];
                let scaled_radius = mag_radius * nf;
                let (snapped, dist) = match self.kind {
                    GridKind::Square => snap_distance_for_target(
                        &self.make_square(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    GridKind::Hex => snap_distance_for_target(
                        &self.make_hex(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    GridKind::Iso => snap_distance_for_target(
                        &self.make_iso(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    GridKind::StaggeredSquare => snap_distance_for_target(
                        &self.make_staggered_square(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    GridKind::StaggeredHex => snap_distance_for_target(
                        &self.make_staggered_hex(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    GridKind::Tri => snap_distance_for_target(
                        &self.make_tri(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    GridKind::Chunks => snap_distance_for_target(
                        &self.make_chunks(),
                        scaled,
                        scaled_half,
                        target,
                        &mut self.scratch,
                    ),
                    _ => unreachable!(),
                };
                // Magnetism gate: only commit the snap if the WINNING
                // candidate is within the (scaled) radius. Otherwise
                // pass through the (scaled) local point unchanged.
                let final_scaled = if scaled_radius > 0.0 && dist > scaled_radius {
                    scaled
                } else {
                    snapped
                };
                [final_scaled[0] / nf, final_scaled[1] / nf]
            }
            // Non-uniform grids: dedicated snappers (build the
            // structure on each call, no caching). Subdivisions are
            // honored by subdividing the active leaf into N×N sub-cells
            // (Quadtree) or treated as 1 (Voronoi cells are arbitrary
            // polygons; subdividing them isn't well-defined yet).
            GridKind::Quadtree => {
                let snapped = snap_world_quadtree(
                    local,
                    sprite_half_size,
                    target,
                    &self.quadtree_cfg,
                    subdivisions,
                );
                gate_by_magnetism(local, snapped, mag_radius)
            }
            GridKind::Voronoi => {
                let snapped =
                    snap_world_voronoi(local, sprite_half_size, target, &self.voronoi_cfg);
                gate_by_magnetism(local, snapped, mag_radius)
            }
        };
        [snapped_local[0] + origin[0], snapped_local[1] + origin[1]]
    }
}

/// Magnetism gate for the non-uniform-grid paths. Returns `snapped`
/// when it sits within `radius` of `world`, otherwise returns `world`
/// unchanged (pass-through during free drag). `radius <= 0.0` keeps
/// the always-snap semantic the legacy tests depended on.
#[inline]
fn gate_by_magnetism(world: Vec2, snapped: Vec2, radius: f32) -> Vec2 {
    if radius <= 0.0 {
        return snapped;
    }
    let d = sq_dist(world, snapped).sqrt();
    if d <= radius { snapped } else { world }
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

fn snap_world_quadtree(
    world: Vec2,
    half: Vec2,
    target: SnapTarget,
    cfg: &QuadtreeCfg,
    subdivisions: u32,
) -> Vec2 {
    let (leaf_center, leaf_corners) = quadtree_active_leaf(world, cfg);
    let n = subdivisions.max(1);
    if n == 1 {
        return match target {
            SnapTarget::Center => leaf_center,
            SnapTarget::Intersection => nearest_to(world, &leaf_corners),
            SnapTarget::Corner => corner_snap_against_vertices(world, half, &leaf_corners),
            SnapTarget::CenterAndIntersection => {
                let v = nearest_to(world, &leaf_corners);
                nearest_to(world, &[leaf_center, v])
            }
            SnapTarget::CenterIntersectionAndCorners => {
                let v = nearest_to(world, &leaf_corners);
                let k = corner_snap_against_vertices(world, half, &leaf_corners);
                nearest_to(world, &[leaf_center, v, k])
            }
        };
    }
    // Subdivide the active leaf into N×N uniform sub-cells. The leaf
    // corners are emitted in CCW order from `quadtree_active_leaf`:
    // [min, (max.x, min.y), max, (min.x, max.y)].
    let leaf_min = leaf_corners[0];
    let leaf_max = leaf_corners[2];
    let dx = (leaf_max[0] - leaf_min[0]) / n as f32;
    let dy = (leaf_max[1] - leaf_min[1]) / n as f32;
    let mut centers: Vec<Vec2> = Vec::with_capacity((n * n) as usize);
    let mut vertices: Vec<Vec2> = Vec::with_capacity(((n + 1) * (n + 1)) as usize);
    for j in 0..n {
        for i in 0..n {
            centers.push([
                leaf_min[0] + (i as f32 + 0.5) * dx,
                leaf_min[1] + (j as f32 + 0.5) * dy,
            ]);
        }
    }
    for j in 0..=n {
        for i in 0..=n {
            vertices.push([leaf_min[0] + i as f32 * dx, leaf_min[1] + j as f32 * dy]);
        }
    }
    let active_center = nearest_to(world, &centers);
    match target {
        SnapTarget::Center => active_center,
        SnapTarget::Intersection => nearest_to(world, &vertices),
        SnapTarget::Corner => corner_snap_against_vertices(world, half, &vertices),
        SnapTarget::CenterAndIntersection => {
            let v = nearest_to(world, &vertices);
            nearest_to(world, &[active_center, v])
        }
        SnapTarget::CenterIntersectionAndCorners => {
            let v = nearest_to(world, &vertices);
            let k = corner_snap_against_vertices(world, half, &vertices);
            nearest_to(world, &[active_center, v, k])
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
#[path = "state_tests.rs"]
mod tests;
