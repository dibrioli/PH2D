//! Staggered render adapters — square cells (shifted) and hex cells
//! viewed under offset coords. Hex variant delegates to
//! [`super::hex`] since the underlying geometry is identical.

use super::util::{polygon_to_path_world_with_origin, stroke_path, world_bounds};
use crate::grid::GridView;
use crate::grid_snap::state::{StaggeredHexCfg, StaggeredSquareCfg};
use ph2d_grid::GridMath;
use ph2d_grid::staggered::StaggeredSquareGrid;
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint_square(
    scene: &mut VectorScene,
    view: &GridView,
    color: Color,
    cfg: &StaggeredSquareCfg,
) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 || cfg.cell_w <= 0.0 || cfg.cell_h <= 0.0 {
        return;
    }
    let grid = StaggeredSquareGrid::new(cfg.cell_w, cfg.cell_h, cfg.parity, cfg.neighborhood);
    let (bounds, _) = world_bounds(view);
    let origin = cfg.origin;

    let corners = [
        [bounds.left - origin[0], bounds.bottom - origin[1]],
        [bounds.right - origin[0], bounds.bottom - origin[1]],
        [bounds.right - origin[0], bounds.top - origin[1]],
        [bounds.left - origin[0], bounds.top - origin[1]],
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
    x_min -= 2;
    x_max += 2;
    y_min -= 2;
    y_max += 2;

    let mut verts = Vec::with_capacity(4);
    let mut path = BezPath::new();
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            grid.cell_to_world_vertices((x, y), &mut verts);
            polygon_to_path_world_with_origin(&mut path, &verts, origin, &bounds, view);
        }
    }
    stroke_path(scene, &path, 0.8, color);
}

pub fn paint_hex(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &StaggeredHexCfg) {
    // Visually identical to a regular hex grid; the staggered "view"
    // is purely a coord-system relabeling.
    super::hex::paint(scene, view, color, &cfg.hex);
}
