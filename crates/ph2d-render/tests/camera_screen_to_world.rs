//! Integration tests for `Camera2d::screen_to_world` (M14.4e).
//!
//! Lives outside `camera.rs` mod tests so the M14.4e drag-and-drop
//! contract — "cursor at (px, py) maps to (wx, wy) given (camera,
//! window)" — is exercised through the public API surface that the
//! desktop shell consumes. Mirrors the math the shell uses to spawn
//! a dropped image at the cursor instead of at the camera center.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

#[test]
fn drop_at_window_center_spawns_at_camera_center() {
    // Canonical case: drag from outside the window, release in the
    // middle. World coords should match `camera.center` exactly.
    let cam = Camera2d::new([100.0, -50.0], 10.0);
    let win = WindowSize::new(1920, 1080);
    let cursor = (win.width as f32 * 0.5, win.height as f32 * 0.5);
    let [wx, wy] = cam.screen_to_world(cursor, win);
    assert!((wx - 100.0).abs() < 1e-3, "wx mismatch: {wx}");
    assert!((wy - -50.0).abs() < 1e-3, "wy mismatch: {wy}");
}

#[test]
fn drop_at_top_of_window_spawns_above_camera_center() {
    // Drag-and-drop semantic: cursor near the top of the canvas
    // means "spawn at the top of the visible world" (world Y-up).
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let cursor = (400.0, 0.0); // top edge, centered horizontally
    let [_, wy] = cam.screen_to_world(cursor, win);
    assert!(wy > 0.0, "screen-top drop must yield world +Y, got {wy}");
    assert!((wy - 5.0).abs() < 1e-3, "expected wy=+5, got {wy}");
}

#[test]
fn drop_after_camera_pan_tracks_camera() {
    // Camera panned to (50, 10), drop at top-left corner — world
    // coords should reflect both camera offset and screen position.
    let mut cam = Camera2d::new([0.0, 0.0], 10.0);
    cam.center = [50.0, 10.0];
    let win = WindowSize::new(800, 600);
    let [wx_center, wy_center] = cam.screen_to_world((400.0, 300.0), win);
    assert!((wx_center - 50.0).abs() < 1e-3);
    assert!((wy_center - 10.0).abs() < 1e-3);
    // Top-left should be `(center.x - half_w, center.y + half_h)`
    // with half_w = 5 * (800/600) and half_h = 5.
    let [wx_tl, wy_tl] = cam.screen_to_world((0.0, 0.0), win);
    let half_w = 5.0_f32 * (800.0 / 600.0);
    assert!((wx_tl - (50.0 - half_w)).abs() < 1e-3);
    assert!((wy_tl - 15.0).abs() < 1e-3);
}

#[test]
fn drop_after_camera_zoom_tracks_zoom() {
    // At 2× zoom (height_world halved) the same screen position
    // maps to a smaller world delta.
    let mut cam = Camera2d::new([0.0, 0.0], 10.0);
    cam.zoom(0.5); // height_world → 5
    let win = WindowSize::new(800, 600);
    let [_, wy_top] = cam.screen_to_world((400.0, 0.0), win);
    // half_h is now 2.5 (was 5).
    assert!(
        (wy_top - 2.5).abs() < 1e-3,
        "expected wy=+2.5, got {wy_top}"
    );
}

#[test]
fn drop_at_window_corners_is_symmetric_around_center() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [tl_x, tl_y] = cam.screen_to_world((0.0, 0.0), win);
    let [tr_x, tr_y] = cam.screen_to_world((800.0, 0.0), win);
    let [bl_x, bl_y] = cam.screen_to_world((0.0, 600.0), win);
    let [br_x, br_y] = cam.screen_to_world((800.0, 600.0), win);
    // Mirror symmetry around camera center (0,0):
    assert!((tl_x + tr_x).abs() < 1e-3, "tl/tr X must mirror");
    assert!((bl_x + br_x).abs() < 1e-3, "bl/br X must mirror");
    assert!((tl_y + bl_y).abs() < 1e-3, "tl/bl Y must mirror");
    assert!((tr_y + br_y).abs() < 1e-3, "tr/br Y must mirror");
}
