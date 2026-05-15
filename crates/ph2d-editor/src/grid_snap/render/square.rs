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
    let minor = cfg.cell_size;
    let major = if cfg.spacing_major > minor {
        cfg.spacing_major
    } else {
        minor
    };

    // Minor pass — skip lines that coincide with major so the major
    // overpaint doesn't fight a duplicate stroke on the same pixel.
    let mut minor_path = BezPath::new();
    let skip = if major > minor { Some(major) } else { None };
    build_lines(&mut minor_path, &bounds, view, minor, cfg.origin, skip);
    stroke_path(scene, &minor_path, 0.6, color);

    // Major pass — drawn over the minor lines with a heavier stroke.
    // Skipped when spacing_major collapses to spacing_minor.
    if major > minor {
        let mut major_path = BezPath::new();
        build_lines(&mut major_path, &bounds, view, major, cfg.origin, None);
        stroke_path(scene, &major_path, 1.4, color);
    }
}

/// Emit vertical + horizontal lines at multiples of `spacing` plus
/// the `origin` offset. When `skip_multiples_of` is `Some(major)`,
/// any line whose world coord is itself a multiple of `major`
/// (within a small tolerance) is omitted to avoid double-painting
/// against a later major pass.
fn build_lines(
    path: &mut BezPath,
    bounds: &WorldBounds,
    view: &GridView,
    spacing: f32,
    origin: ph2d_grid::Vec2,
    skip_multiples_of: Option<f32>,
) {
    // Vertical lines — constant world X = i * spacing + origin.x.
    let first_v = ((bounds.left - origin[0]) / spacing).ceil() as i32;
    let last_v = ((bounds.right - origin[0]) / spacing).floor() as i32;
    for i in first_v..=last_v {
        let wx = i as f32 * spacing + origin[0];
        if let Some(major) = skip_multiples_of {
            // Skip if `wx - origin` is a multiple of major (line
            // coincides with a major-pass line). 1e-3 tolerance
            // absorbs f32 round-trip jitter same as the canonical
            // grid.rs uses.
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
    // Horizontal lines — constant world Y.
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
