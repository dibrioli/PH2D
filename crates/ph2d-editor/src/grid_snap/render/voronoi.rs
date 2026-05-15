//! Voronoi render adapter — stub for Stage 8 (lands in Stage 9).

use crate::grid::GridView;
use crate::grid_snap::state::VoronoiCfg;
use ph2d_vector::{Color, VectorScene};

pub fn paint(_scene: &mut VectorScene, _view: &GridView, _color: Color, _cfg: &VoronoiCfg) {
    // Stage 9 fills in: deterministic_seeds, build Triangulation,
    // run lloyd_step N times, derive voronoi_cells, clip each to
    // view AABB, stroke polygons.
}
