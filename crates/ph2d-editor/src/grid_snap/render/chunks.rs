//! Chunked-square render adapter — thin inner cell lines + thick
//! chunk-boundary lines drawn over the same region.

use super::util::{WorldBounds, stroke_path, world_bounds, world_to_screen};
use crate::grid::GridView;
use crate::grid_snap::state::ChunksCfg;
use ph2d_vector::{BezPath, Color, VectorScene};

pub fn paint(scene: &mut VectorScene, view: &GridView, color: Color, cfg: &ChunksCfg) {
    if view.canvas.w <= 0.0
        || view.canvas.h <= 0.0
        || cfg.cell_size <= 0.0
        || cfg.chunk_size_cells == 0
    {
        return;
    }
    let (bounds, _) = world_bounds(view);
    let cell = cfg.cell_size;
    let chunk = cell * cfg.chunk_size_cells as f32;
    let origin = cfg.origin;

    // Pass 1: thin lines at every cell boundary, skipping ones that
    // overlap a chunk line (avoids double-paint where chunk wins).
    let mut thin_path = BezPath::new();
    push_lines(&mut thin_path, &bounds, view, cell, origin, Some(chunk));
    stroke_path(scene, &thin_path, 0.6, color);

    // Pass 2: thicker lines at chunk boundaries.
    let mut thick_path = BezPath::new();
    push_lines(&mut thick_path, &bounds, view, chunk, origin, None);
    stroke_path(scene, &thick_path, 1.6, color);
}

fn push_lines(
    path: &mut BezPath,
    bounds: &WorldBounds,
    view: &GridView,
    spacing: f32,
    origin: ph2d_grid::Vec2,
    skip_multiples_of: Option<f32>,
) {
    let first_v = ((bounds.left - origin[0]) / spacing).ceil() as i32;
    let last_v = ((bounds.right - origin[0]) / spacing).floor() as i32;
    for i in first_v..=last_v {
        let wx = i as f32 * spacing + origin[0];
        if let Some(major) = skip_multiples_of {
            let q = (wx - origin[0]) / major;
            if (q.round() - q).abs() < 1e-3 {
                continue;
            }
        }
        let top = world_to_screen([wx, bounds.top], bounds, view);
        let bot = world_to_screen([wx, bounds.bottom], bounds, view);
        path.move_to((top[0] as f64, top[1] as f64));
        path.line_to((bot[0] as f64, bot[1] as f64));
    }
    let first_h = ((bounds.bottom - origin[1]) / spacing).ceil() as i32;
    let last_h = ((bounds.top - origin[1]) / spacing).floor() as i32;
    for j in first_h..=last_h {
        let wy = j as f32 * spacing + origin[1];
        if let Some(major) = skip_multiples_of {
            let q = (wy - origin[1]) / major;
            if (q.round() - q).abs() < 1e-3 {
                continue;
            }
        }
        let left = world_to_screen([bounds.left, wy], bounds, view);
        let right = world_to_screen([bounds.right, wy], bounds, view);
        path.move_to((left[0] as f64, left[1] as f64));
        path.line_to((right[0] as f64, right[1] as f64));
    }
}
