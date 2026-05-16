//! Triangular render adapter — alternating up/down triangles tiling
//! each row strip.

use super::util::{polygon_to_path_world_with_origin, stroke_path, world_bounds};
use crate::grid::GridView;
use crate::grid_snap::state::TriCfg;
use ph2d_grid::GridMath;
use ph2d_grid::tri::{TriCell, TriGrid};
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &TriCfg) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 || cfg.edge_length <= 0.0 {
        return;
    }
    let grid = TriGrid::new(cfg.edge_length, cfg.neighborhood);
    let (bounds, _) = world_bounds(view);
    let origin = cfg.origin;

    let corners = [
        [bounds.left - origin[0], bounds.bottom - origin[1]],
        [bounds.right - origin[0], bounds.bottom - origin[1]],
        [bounds.right - origin[0], bounds.top - origin[1]],
        [bounds.left - origin[0], bounds.top - origin[1]],
    ];
    let mut k_min = i32::MAX;
    let mut k_max = i32::MIN;
    let mut r_min = i32::MAX;
    let mut r_max = i32::MIN;
    for c in &corners {
        let cell = grid.world_to_cell(*c);
        k_min = k_min.min(cell.k);
        k_max = k_max.max(cell.k);
        r_min = r_min.min(cell.r);
        r_max = r_max.max(cell.r);
    }
    // Tri cells overlap horizontally by 1 half-col; pad generously.
    k_min -= 3;
    k_max += 3;
    r_min -= 1;
    r_max += 1;

    let mut verts = Vec::with_capacity(3);
    let mut path = BezPath::new();
    for r in r_min..=r_max {
        for k in k_min..=k_max {
            let cell = TriCell::new(k, r);
            grid.cell_to_world_vertices(cell, &mut verts);
            polygon_to_path_world_with_origin(&mut path, &verts, origin, &bounds, view);
        }
    }
    stroke_path(scene, &path, 0.8, color);
}
