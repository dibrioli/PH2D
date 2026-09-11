//! Integration tests for `Camera2d::screen_to_world` (M14.4e).
//!
//! Standard convention (M14.4e v2 onward, no projection Y-flip):
//! cursor at screen TOP maps to world +Y (top of camera view).

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

#[test]
fn drop_at_window_center_spawns_at_camera_center() {
    let cam = Camera2d::new([100.0, -50.0], 10.0);
    let win = WindowSize::new(1920, 1080);
    let cursor = (win.width as f32 * 0.5, win.height as f32 * 0.5);
    let [wx, wy] = cam.screen_to_world(cursor, win);
    assert!((wx - 100.0).abs() < 1e-3, "wx mismatch: {wx}");
    assert!((wy - -50.0).abs() < 1e-3, "wy mismatch: {wy}");
}

#[test]
fn drop_at_top_of_window_spawns_at_visual_top() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [_, wy] = cam.screen_to_world((400.0, 0.0), win);
    assert!((wy - 5.0).abs() < 1e-3, "expected wy=+5, got {wy}");
}

#[test]
fn drop_at_bottom_of_window_spawns_at_visual_bottom() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [_, wy] = cam.screen_to_world((400.0, 600.0), win);
    assert!((wy - -5.0).abs() < 1e-3, "expected wy=-5, got {wy}");
}

#[test]
fn drop_after_camera_pan_tracks_camera() {
    let mut cam = Camera2d::new([0.0, 0.0], 10.0);
    cam.center = [50.0, 10.0];
    let win = WindowSize::new(800, 600);
    let [wx_c, wy_c] = cam.screen_to_world((400.0, 300.0), win);
    assert!((wx_c - 50.0).abs() < 1e-3);
    assert!((wy_c - 10.0).abs() < 1e-3);
    // Top-left: x = center.x - half_w, y = center.y + half_h.
    let [wx_tl, wy_tl] = cam.screen_to_world((0.0, 0.0), win);
    let half_w = 5.0_f32 * (800.0 / 600.0);
    assert!((wx_tl - (50.0 - half_w)).abs() < 1e-3);
    assert!(
        (wy_tl - 15.0).abs() < 1e-3,
        "expected wy_tl=+15, got {wy_tl}"
    );
}

#[test]
fn drop_after_camera_zoom_tracks_zoom() {
    let mut cam = Camera2d::new([0.0, 0.0], 10.0);
    cam.zoom(0.5);
    let win = WindowSize::new(800, 600);
    let [_, wy_top] = cam.screen_to_world((400.0, 0.0), win);
    assert!(
        (wy_top - 2.5).abs() < 1e-3,
        "expected wy=+2.5 at half-zoom, got {wy_top}"
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
    assert!((tl_x + tr_x).abs() < 1e-3, "tl/tr X must mirror");
    assert!((bl_x + br_x).abs() < 1e-3, "bl/br X must mirror");
    assert!((tl_y + bl_y).abs() < 1e-3, "tl/bl Y must mirror");
    assert!((tr_y + br_y).abs() < 1e-3, "tr/br Y must mirror");
}
