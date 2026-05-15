//! Square-grid render adapter — vertical + horizontal lines spanning
//! the visible world AABB.
//!
//! Pattern mirrors the existing [`crate::grid::paint_grid`] for the
//! orthogonal case, but parameterized by the panel's `SquareCfg`
//! rather than the engine-wide `GridConfig`.

use super::util::{WorldBounds, stroke_path, world_bounds, world_to_screen};
use crate::grid::GridView;
use crate::grid_snap::state::SquareCfg;
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &SquareCfg) {
    if view.canvas.w <= 0.0 || view.canvas.h <= 0.0 || cfg.cell_size <= 0.0 {
        return;
    }
    let (bounds, _ppm) = world_bounds(view);

    let mut path = BezPath::new();
    build_lines(&mut path, &bounds, view, cfg.cell_size);
    stroke_path(scene, &path, 0.8, color);
}

fn build_lines(path: &mut BezPath, bounds: &WorldBounds, view: &GridView, spacing: f32) {
    // Vertical lines (constant world X).
    let first_v = (bounds.left / spacing).ceil() as i32;
    let last_v = (bounds.right / spacing).floor() as i32;
    for i in first_v..=last_v {
        let wx = i as f32 * spacing;
        let top = world_to_screen([wx, bounds.top], bounds, view);
        let bot = world_to_screen([wx, bounds.bottom], bounds, view);
        path.move_to((top[0] as f64, top[1] as f64));
        path.line_to((bot[0] as f64, bot[1] as f64));
    }
    // Horizontal lines (constant world Y).
    let first_h = (bounds.bottom / spacing).ceil() as i32;
    let last_h = (bounds.top / spacing).floor() as i32;
    for j in first_h..=last_h {
        let wy = j as f32 * spacing;
        let left = world_to_screen([bounds.left, wy], bounds, view);
        let right = world_to_screen([bounds.right, wy], bounds, view);
        path.move_to((left[0] as f64, left[1] as f64));
        path.line_to((right[0] as f64, right[1] as f64));
    }
}

/// Test helper: count of visible grid lines (for unit tests that
/// don't have a real `VectorScene` to inspect).
#[cfg(test)]
pub fn count_visible_lines(view: &GridView, cfg: &SquareCfg) -> u32 {
    let (bounds, _) = world_bounds(view);
    let v = (bounds.right / cfg.cell_size).floor() as i32
        - (bounds.left / cfg.cell_size).ceil() as i32
        + 1;
    let h = (bounds.top / cfg.cell_size).floor() as i32
        - (bounds.bottom / cfg.cell_size).ceil() as i32
        + 1;
    (v.max(0) + h.max(0)) as u32
}
