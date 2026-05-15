//! Shared projection / stroke helpers for the render adapters.
//!
//! Re-derives the same `world → screen` math the editor's existing
//! [`crate::grid`] module uses, against the [`crate::grid::GridView`]
//! struct. Kept local to `grid_snap::render` so each per-kind
//! adapter stays free of cross-module knowledge.

use crate::grid::GridView;
use ph2d_grid::Vec2;
use ph2d_vector::{Affine, BezPath, Brush, Color, Stroke, VectorScene};

/// Visible world rectangle derived from the GridView's camera.
#[derive(Copy, Clone, Debug)]
pub struct WorldBounds {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

/// `(world_bounds, pixels_per_meter_y)`.
pub fn world_bounds(view: &GridView) -> (WorldBounds, f32) {
    let aspect = view.window_w / view.window_h;
    let half_h = view.camera_height_world * 0.5;
    let half_w = half_h * aspect;
    let cx = view.camera_center[0];
    let cy = view.camera_center[1];
    let bounds = WorldBounds {
        left: cx - half_w,
        right: cx + half_w,
        bottom: cy - half_h,
        top: cy + half_h,
    };
    let ppm_y = view.window_h / view.camera_height_world;
    (bounds, ppm_y)
}

/// Project a world point onto screen-space pixel coords (Y-flip
/// matches editor convention: world Y-up → screen Y-down).
pub fn world_to_screen(p: Vec2, bounds: &WorldBounds, view: &GridView) -> [f32; 2] {
    let tx = (p[0] - bounds.left) / (bounds.right - bounds.left);
    let ty = (p[1] - bounds.bottom) / (bounds.top - bounds.bottom);
    [tx * view.window_w, view.window_h - ty * view.window_h]
}

/// Stroke a path. Skip empty paths to avoid Vello no-op work.
pub fn stroke_path(scene: &mut VectorScene, path: &BezPath, width: f32, color: Color) {
    if path.is_empty() {
        return;
    }
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, path);
}

/// Append a polygon (closed loop) to `path` in screen-space, given
/// world-space vertices.
pub fn polygon_to_path_world(
    path: &mut BezPath,
    verts: &[Vec2],
    bounds: &WorldBounds,
    view: &GridView,
) {
    if verts.len() < 2 {
        return;
    }
    let p0 = world_to_screen(verts[0], bounds, view);
    path.move_to((p0[0] as f64, p0[1] as f64));
    for v in &verts[1..] {
        let p = world_to_screen(*v, bounds, view);
        path.line_to((p[0] as f64, p[1] as f64));
    }
    path.close_path();
}
