//! Iso render adapter — diamond cells in 2:1 dimetric projection.

use super::util::{polygon_to_path_world, stroke_path, world_bounds};
use crate::grid::GridView;
use crate::grid_snap::state::IsoCfg;
use ph2d_grid::GridMath;
use ph2d_grid::iso::IsoGrid;
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &IsoCfg) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 || cfg.tile_w <= 0.0 || cfg.tile_h <= 0.0 {
        return;
    }
    let grid = IsoGrid {
        tile_w: cfg.tile_w,
        tile_h: cfg.tile_h,
        neighborhood: cfg.neighborhood,
    };
    let (bounds, _) = world_bounds(view);

    let corners = [
        [bounds.left, bounds.bottom],
        [bounds.right, bounds.bottom],
        [bounds.right, bounds.top],
        [bounds.left, bounds.top],
    ];
    let mut x_min = i32::MAX;
    let mut x_max = i32::MIN;
    let mut y_min = i32::MAX;
    let mut y_max = i32::MIN;
    for c in &corners {
        let cell = grid.world_to_cell(*c);
        x_min = x_min.min(cell.0);
        x_max = x_max.max(cell.0);
        y_min = y_min.min(cell.1);
        y_max = y_max.max(cell.1);
    }
    // Iso diamonds spread further in screen space than a 1:1 box, so
    // pad more aggressively than Square / Hex.
    x_min -= 3;
    x_max += 3;
    y_min -= 3;
    y_max += 3;

    let mut verts = Vec::with_capacity(4);
    let mut path = BezPath::new();
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            grid.cell_to_world_vertices((x, y), &mut verts);
            polygon_to_path_world(&mut path, &verts, &bounds, view);
        }
    }
    stroke_path(scene, &path, 0.8, color);
}
