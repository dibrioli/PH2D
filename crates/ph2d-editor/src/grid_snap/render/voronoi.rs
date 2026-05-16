//! Voronoi render adapter — generates seeds, triangulates, runs
//! Lloyd's relaxation N times, then strokes each cell polygon
//! clipped to the visible AABB.

use super::util::{polygon_to_path_world, stroke_path, world_bounds};
use crate::grid::GridView;
use crate::grid_snap::state::VoronoiCfg;
use ph2d_grid::voronoi::{Triangulation, deterministic_seeds};
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &VoronoiCfg) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 || cfg.seed_count < 3 {
        return;
    }
    let seeds = deterministic_seeds(cfg.bounds, cfg.seed_count, cfg.rng_seed);
    let mut t = Triangulation::from_seeds(&seeds);
    for _ in 0..cfg.lloyd_iterations {
        t.lloyd_step();
    }

    let (bounds, _) = world_bounds(view);
    // Use the smaller of (panel bounds, visible world AABB) as the
    // clip rect so boundary cells get capped to the demo region.
    let view_aabb =
        ph2d_grid::quadtree::AABB::new([bounds.left, bounds.bottom], [bounds.right, bounds.top]);
    let clip_rect = intersect_aabb(cfg.bounds, view_aabb);

    let cells = t.voronoi_cells();

    let mut path = BezPath::new();
    for cell in &cells {
        if cell.vertices.len() < 3 {
            continue;
        }
        let clipped = Triangulation::clip_cell_to_aabb(cell, clip_rect);
        if clipped.len() < 3 {
            continue;
        }
        polygon_to_path_world(&mut path, &clipped, &bounds, view);
    }
    stroke_path(scene, &path, 0.8, color);
}

fn intersect_aabb(
    a: ph2d_grid::quadtree::AABB,
    b: ph2d_grid::quadtree::AABB,
) -> ph2d_grid::quadtree::AABB {
    ph2d_grid::quadtree::AABB::new(
        [a.min[0].max(b.min[0]), a.min[1].max(b.min[1])],
        [a.max[0].min(b.max[0]), a.max[1].min(b.max[1])],
    )
}
