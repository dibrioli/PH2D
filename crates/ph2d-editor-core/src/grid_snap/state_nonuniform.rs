//! **Os encaixes das grelhas não-uniformes** (Quadtree e Voronoi) e o portão de magnetismo que os
//! filtra, irmão de `state.rs` por tecto de LOC.
//!
//! Corte mecânico: as funções saíram inteiras, verbatim; abriu-se só a visibilidade das três que o
//! `GridSnapState::snap_world` chama.

use super::*;

/// Magnetism gate for the non-uniform-grid paths. Returns `snapped`
/// when it sits within `radius` of `world`, otherwise returns `world`
/// unchanged (pass-through during free drag). `radius <= 0.0` keeps
/// the always-snap semantic the legacy tests depended on.
#[inline]
pub(super) fn gate_by_magnetism(world: Vec2, snapped: Vec2, radius: f32) -> Vec2 {
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

pub(super) fn snap_world_quadtree(
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

pub(super) fn snap_world_voronoi(
    world: Vec2,
    half: Vec2,
    target: SnapTarget,
    cfg: &VoronoiCfg,
) -> Vec2 {
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
