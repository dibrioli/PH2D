//! Quadtree render adapter — builds the demo quadtree from the
//! config + inserts deterministic seed points, then strokes each
//! leaf AABB as a rectangle outline.

use super::util::{stroke_path, world_bounds, world_to_screen};
use crate::grid::GridView;
use crate::grid_snap::state::QuadtreeCfg;
use ph2d_grid::quadtree::{AABB, Quadtree};
use ph2d_grid::voronoi::deterministic_seeds;
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &QuadtreeCfg) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 {
        return;
    }
    let qt = build_demo(cfg);
    let (bounds, _) = world_bounds(view);

    let mut leaves: Vec<AABB> = Vec::new();
    qt.iter_leaf_bounds(&mut leaves);

    // Pass 1: thin outline of every leaf box.
    let mut path = BezPath::new();
    for leaf in &leaves {
        // Skip leaves entirely outside the visible AABB to avoid
        // wasted strokes on heavily-zoomed-in views.
        if leaf.max[0] < bounds.left
            || leaf.min[0] > bounds.right
            || leaf.max[1] < bounds.bottom
            || leaf.min[1] > bounds.top
        {
            continue;
        }
        let tl = world_to_screen([leaf.min[0], leaf.max[1]], &bounds, view);
        let tr = world_to_screen([leaf.max[0], leaf.max[1]], &bounds, view);
        let br = world_to_screen([leaf.max[0], leaf.min[1]], &bounds, view);
        let bl = world_to_screen([leaf.min[0], leaf.min[1]], &bounds, view);
        path.move_to((tl[0] as f64, tl[1] as f64));
        path.line_to((tr[0] as f64, tr[1] as f64));
        path.line_to((br[0] as f64, br[1] as f64));
        path.line_to((bl[0] as f64, bl[1] as f64));
        path.close_path();
    }
    stroke_path(scene, &path, 0.8, color);

    // Pass 2: dot at each inserted demo point (small cross marker).
    let mut dot_path = BezPath::new();
    let mut entries: Vec<ph2d_grid::quadtree::Entry<()>> = Vec::new();
    qt.query_rect(cfg.bounds, &mut entries);
    for e in &entries {
        let p = world_to_screen(e.point, &bounds, view);
        dot_path.move_to((p[0] as f64 - 2.0, p[1] as f64));
        dot_path.line_to((p[0] as f64 + 2.0, p[1] as f64));
        dot_path.move_to((p[0] as f64, p[1] as f64 - 2.0));
        dot_path.line_to((p[0] as f64, p[1] as f64 + 2.0));
    }
    stroke_path(scene, &dot_path, 1.0, color);
}

fn build_demo(cfg: &QuadtreeCfg) -> Quadtree<()> {
    let mut qt: Quadtree<()> = Quadtree::new(cfg.bounds, cfg.max_points_per_leaf, cfg.max_depth);
    if cfg.demo_point_count == 0 {
        return qt;
    }
    let pts = deterministic_seeds(cfg.bounds, cfg.demo_point_count, cfg.demo_rng_seed);
    for p in pts {
        qt.insert(p, ());
    }
    qt
}
