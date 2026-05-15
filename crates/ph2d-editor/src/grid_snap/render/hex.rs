//! Hex render adapter — enumerates visible hex cells via the
//! pointy/flat projection and emits each hexagon's 6 edges.

use super::util::{polygon_to_path_world, stroke_path, world_bounds};
use crate::grid::GridView;
use crate::grid_snap::state::HexCfg;
use ph2d_grid::GridMath;
use ph2d_grid::hex::{HexCell, HexGrid};
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &HexCfg) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 || cfg.cell_size <= 0.0 {
        return;
    }
    let grid = HexGrid {
        cell_size: cfg.cell_size,
        orientation: cfg.orientation,
        offset_default: cfg.offset_variant,
    };
    let (bounds, _) = world_bounds(view);

    // Conservative axial-range bounds: take the AABB's 4 corners,
    // convert each to axial, and expand by ±2 to catch hexes that
    // straddle the boundary.
    let corners = [
        [bounds.left, bounds.bottom],
        [bounds.right, bounds.bottom],
        [bounds.right, bounds.top],
        [bounds.left, bounds.top],
    ];
    let mut q_min = i32::MAX;
    let mut q_max = i32::MIN;
    let mut r_min = i32::MAX;
    let mut r_max = i32::MIN;
    for c in &corners {
        let cell = grid.world_to_cell(*c);
        q_min = q_min.min(cell.q);
        q_max = q_max.max(cell.q);
        r_min = r_min.min(cell.r);
        r_max = r_max.max(cell.r);
    }
    q_min -= 2;
    q_max += 2;
    r_min -= 2;
    r_max += 2;

    let mut verts = Vec::with_capacity(6);
    let mut path = BezPath::new();
    for r in r_min..=r_max {
        for q in q_min..=q_max {
            let cell = HexCell::new(q, r);
            grid.cell_to_world_vertices(cell, &mut verts);
            polygon_to_path_world(&mut path, &verts, &bounds, view);
        }
    }
    stroke_path(scene, &path, 0.8, color);
}

/// Test helper: cell count drawn for the given view + config.
#[cfg(test)]
pub fn count_visible_cells(view: &GridView, cfg: &HexCfg) -> u32 {
    let grid = HexGrid {
        cell_size: cfg.cell_size,
        orientation: cfg.orientation,
        offset_default: cfg.offset_variant,
    };
    let (bounds, _) = world_bounds(view);
    let corners = [
        [bounds.left, bounds.bottom],
        [bounds.right, bounds.bottom],
        [bounds.right, bounds.top],
        [bounds.left, bounds.top],
    ];
    let (mut q_min, mut q_max, mut r_min, mut r_max) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    for c in &corners {
        let cell = grid.world_to_cell(*c);
        q_min = q_min.min(cell.q);
        q_max = q_max.max(cell.q);
        r_min = r_min.min(cell.r);
        r_max = r_max.max(cell.r);
    }
    q_min -= 2;
    q_max += 2;
    r_min -= 2;
    r_max += 2;
    ((q_max - q_min + 1) * (r_max - r_min + 1)) as u32
}
