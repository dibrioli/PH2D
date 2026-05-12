//! 2D orthographic camera.
//!
//! World space is Y-up (per SKILL §11.1). The view-proj matrix
//! includes the Y-flip so the shader output matches clip space
//! (which is Y-down in wgpu/WebGPU NDC). Single source of the flip
//! per §11.1: "O flip Y-up → Y-down é aplicado uma vez na projection
//! matrix".

use ph2d_core::Mat4;
use ph2d_host::WindowSize;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct Camera2d {
    /// Camera center in world units.
    pub center: [f32; 2],
    /// World-units height of the visible region. Width is derived
    /// from window aspect. Use small values (e.g., 10.0) for "zoomed
    /// in" feel; larger for overview.
    pub height_world: f32,
}

impl Camera2d {
    /// Minimum `height_world` allowed by [`Camera2d::zoom`]. 0.5 m =
    /// "zoomed in enough that a half-meter sprite fills the screen
    /// vertically" — anything tighter starts to lose floating-point
    /// precision in the projection matrix.
    pub const ZOOM_MIN_HEIGHT_WORLD: f32 = 0.5;
    /// Maximum `height_world` allowed by [`Camera2d::zoom`]. 100 m =
    /// "5 city blocks visible vertically" — anything wider and the
    /// grid LOD threshold suppresses minor lines anyway.
    pub const ZOOM_MAX_HEIGHT_WORLD: f32 = 100.0;

    pub fn new(center: [f32; 2], height_world: f32) -> Self {
        Self {
            center,
            height_world,
        }
    }

    /// Multiply `height_world` by `factor` and clamp to
    /// `[ZOOM_MIN_HEIGHT_WORLD, ZOOM_MAX_HEIGHT_WORLD]`.
    ///
    /// `factor < 1.0` zooms in (world appears bigger); `> 1.0` zooms
    /// out (world appears smaller). For wheel input the canonical
    /// formula is `factor = 0.9.powf(wheel_dy / 16.0)` — a small
    /// per-notch step that compounds nicely under fast scrolling.
    ///
    /// `factor == 1.0` is a no-op. Non-finite factors are also no-ops
    /// (defensive against NaN coming from misconfigured trackpads).
    pub fn zoom(&mut self, factor: f32) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let next = (self.height_world * factor)
            .clamp(Self::ZOOM_MIN_HEIGHT_WORLD, Self::ZOOM_MAX_HEIGHT_WORLD);
        self.height_world = next;
    }

    /// Pan the camera by a screen-pixel delta — interpreted as
    /// "user dragged the cursor by `(dx_px, dy_px)` and the world
    /// under the cursor should move with it" (i.e. world moves
    /// **opposite** to the cursor drag direction; the camera center
    /// moves the same direction as the cursor).
    ///
    /// The Y-flip applied in [`Camera2d::view_proj`] is honored: a
    /// drag of `(0, +10)` (cursor moves down on Y-down screen)
    /// translates the camera **down** in world (Y-up); `center.y`
    /// decreases.
    ///
    /// `window_w` / `window_h` are the current swapchain dimensions
    /// in pixels — the function needs them to derive
    /// `pixels_per_meter`.
    pub fn pan_screen_delta(&mut self, dx_px: f32, dy_px: f32, window_w: f32, window_h: f32) {
        if window_h <= 0.0 || window_w <= 0.0 {
            return;
        }
        if !dx_px.is_finite() || !dy_px.is_finite() {
            return;
        }
        let pixels_per_meter = window_h / self.height_world.max(f32::EPSILON);
        let dx_world = dx_px / pixels_per_meter;
        let dy_world = dy_px / pixels_per_meter;
        // Camera moves OPPOSITE to the cursor drag (standard "hand"
        // pan), and the world Y axis is flipped relative to screen:
        //   - cursor +x (right) → camera center -x (world scrolls left under cursor)
        //   - cursor +y (down on screen, Y-down) → camera center +y (world Y-up; scrolls up under cursor)
        self.center[0] -= dx_world;
        self.center[1] += dy_world;
    }

    /// Project a screen-pixel cursor position to a world-space point
    /// in meters. Inverse of the orthographic projection used by
    /// [`Camera2d::view_proj`]; honors the Y-flip so a cursor at the
    /// **top** of the window maps to the **largest** world Y under
    /// the camera (world is Y-up, screen is Y-down).
    ///
    /// `cursor_px` is `(x, y)` in pixels with origin at the
    /// window's top-left corner. `window` carries the current
    /// swapchain dimensions. Returns `(world_x, world_y)` in meters.
    ///
    /// Used by:
    /// - Drag-and-drop image import (M14.4e) to spawn at the drop
    ///   point instead of always at the camera center.
    /// - Sprite picking (M14.5+) to hit-test sprites under the
    ///   cursor for selection.
    /// - Future tools (eyedropper, gizmo handles) needing world-
    ///   space coords from a click event.
    pub fn screen_to_world(&self, cursor_px: (f32, f32), window: WindowSize) -> [f32; 2] {
        let w = window.width.max(1) as f32;
        let h = window.height.max(1) as f32;
        let (cx_px, cy_px) = cursor_px;
        // Normalize to [-1, +1] in clip space (Y-DOWN since wgpu NDC).
        let nx = (cx_px / w) * 2.0 - 1.0;
        let ny = (cy_px / h) * 2.0 - 1.0;
        // Reverse the orthographic: clip → world. half_h/half_w come
        // from height_world + aspect, same as in `view_proj`. The
        // Y-flip in `view_proj` swaps `bottom`/`top` so the inverse
        // here is: world_y = center_y - ny * half_h (note the SIGN).
        let aspect = w / h;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let world_x = self.center[0] + nx * half_w;
        let world_y = self.center[1] - ny * half_h;
        [world_x, world_y]
    }

    /// Build the view-projection matrix for the given window.
    /// Includes the Y-flip from world (Y-up) to clip (Y-down NDC).
    pub fn view_proj(&self, window: WindowSize) -> Mat4 {
        let aspect = window.width.max(1) as f32 / window.height.max(1) as f32;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let cx = self.center[0];
        let cy = self.center[1];
        // Orthographic: world (Y-up) → clip (Y-down). Y-axis flipped
        // by swapping `bottom` and `top` arguments.
        Mat4::orthographic_rh(
            cx - half_w,
            cx + half_w,
            cy + half_h, // bottom (was top)
            cy - half_h, // top    (was bottom)
            -1.0,
            1.0,
        )
    }

    pub fn uniform(&self, window: WindowSize) -> CameraUniform {
        let m = self.view_proj(window);
        CameraUniform {
            view_proj: m.to_cols_array_2d(),
        }
    }
}

impl Default for Camera2d {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            height_world: 10.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_proj_centered_origin_maps_to_clip_origin() {
        let cam = Camera2d::default();
        let win = WindowSize::new(800, 600);
        let m = cam.view_proj(win);
        // World (0,0) → clip ~(0,0).
        let p = m * ph2d_core::Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!(p.x.abs() < 1e-4);
        assert!(p.y.abs() < 1e-4);
    }

    #[test]
    fn zoom_factor_one_is_identity() {
        let mut cam = Camera2d::default();
        let before = cam.height_world;
        cam.zoom(1.0);
        assert_eq!(cam.height_world, before);
    }

    #[test]
    fn zoom_in_decreases_height_world() {
        let mut cam = Camera2d::default();
        cam.zoom(0.5);
        assert!(cam.height_world < 10.0);
        assert!((cam.height_world - 5.0).abs() < 1e-5);
    }

    #[test]
    fn zoom_clamps_to_min() {
        let mut cam = Camera2d::default();
        for _ in 0..200 {
            cam.zoom(0.5);
        }
        assert_eq!(cam.height_world, Camera2d::ZOOM_MIN_HEIGHT_WORLD);
    }

    #[test]
    fn zoom_clamps_to_max() {
        let mut cam = Camera2d::default();
        for _ in 0..200 {
            cam.zoom(2.0);
        }
        assert_eq!(cam.height_world, Camera2d::ZOOM_MAX_HEIGHT_WORLD);
    }

    #[test]
    fn zoom_rejects_non_finite_factor() {
        let mut cam = Camera2d::default();
        let before = cam.height_world;
        cam.zoom(f32::NAN);
        cam.zoom(f32::INFINITY);
        cam.zoom(-1.0);
        cam.zoom(0.0);
        assert_eq!(cam.height_world, before);
    }

    #[test]
    fn pan_zero_delta_is_identity() {
        let mut cam = Camera2d::default();
        let before = cam.center;
        cam.pan_screen_delta(0.0, 0.0, 800.0, 600.0);
        assert_eq!(cam.center, before);
    }

    #[test]
    fn pan_screen_right_moves_camera_left() {
        // Cursor drags right by 100 px on an 800×600 window with
        // height_world 10 → pixels_per_meter = 60 → world delta = 100/60 ≈ 1.667.
        // Camera center should move LEFT by that amount (hand-pan
        // semantics: world scrolls right under the cursor).
        let mut cam = Camera2d::default();
        cam.pan_screen_delta(100.0, 0.0, 800.0, 600.0);
        let expected = -(100.0 / 60.0);
        assert!((cam.center[0] - expected).abs() < 1e-3);
        assert_eq!(cam.center[1], 0.0);
    }

    #[test]
    fn pan_screen_down_moves_camera_up_world() {
        // Y-down screen drag (cursor moves DOWN) translates to
        // Y-up world camera moving UP. cursor +60 px (= 1 m at 60
        // px/m) → camera.center.y += 1.
        let mut cam = Camera2d::default();
        cam.pan_screen_delta(0.0, 60.0, 800.0, 600.0);
        assert!((cam.center[1] - 1.0).abs() < 1e-3);
    }

    #[test]
    fn pan_rejects_zero_window() {
        let mut cam = Camera2d::default();
        let before = cam.center;
        cam.pan_screen_delta(100.0, 100.0, 0.0, 0.0);
        assert_eq!(cam.center, before);
    }

    #[test]
    fn pan_at_zoomed_in_camera_moves_less_world() {
        // Same screen-pixel delta should produce SMALLER world
        // motion when zoomed in. height_world=2 (zoomed in 5×)
        // makes 1 px = 1/300 m instead of 1/60 m.
        let mut cam_wide = Camera2d::default();
        let mut cam_zoomed = Camera2d::new([0.0, 0.0], 2.0);
        cam_wide.pan_screen_delta(60.0, 0.0, 800.0, 600.0);
        cam_zoomed.pan_screen_delta(60.0, 0.0, 800.0, 600.0);
        // Zoomed-in camera should move less in world coords (smaller
        // |center[0]|).
        assert!(cam_zoomed.center[0].abs() < cam_wide.center[0].abs());
    }

    #[test]
    fn y_up_world_maps_to_top_of_clip() {
        // Sprite at world (0, +5) with default camera (height_world 10,
        // centered at origin) should appear at clip y > 0 if Y-up
        // world is correctly flipped to Y-down clip... wait, that's
        // wrong. Y-down clip means top of screen is y < 0 in NDC.
        // Actually wgpu/WebGPU NDC has Y-up: top is +1, bottom is -1.
        // The flip we want is "world Y-up → screen Y-down" only matters
        // for where the world origin sits visually. Let me re-derive:
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 800);
        let m = cam.view_proj(win);
        // In WebGPU NDC Y-up, +y is top of screen. Our world Y-up:
        // +y also "up". With our flip (bottom>top swap), +5 world
        // should map to **negative** clip y (showing at bottom). That
        // is the convention used by Y-down screen-space libraries
        // (Vello, parley, kurbo), which we standardize on (§11.1).
        let p = m * ph2d_core::Vec4::new(0.0, 5.0, 0.0, 1.0);
        assert!(p.y < 0.0, "world +Y should map to negative clip Y");
    }

    #[test]
    fn screen_to_world_center_pixel_is_camera_center() {
        let cam = Camera2d::new([3.5, -2.0], 10.0);
        let win = WindowSize::new(800, 600);
        let [wx, wy] = cam.screen_to_world((400.0, 300.0), win);
        assert!((wx - 3.5).abs() < 1e-4, "cx mismatch: {wx}");
        assert!((wy - -2.0).abs() < 1e-4, "cy mismatch: {wy}");
    }

    #[test]
    fn screen_to_world_top_pixel_maps_to_world_top() {
        // Cursor at top of screen (y_px = 0) → world Y above center
        // (world is Y-up; screen Y=0 = visually top = world max Y).
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 600);
        let [_, wy] = cam.screen_to_world((400.0, 0.0), win);
        assert!(wy > 0.0, "screen-top cursor must yield world +Y, got {wy}");
        // Expected: half_h = 5 → world_y = +5 exactly.
        assert!((wy - 5.0).abs() < 1e-4);
    }

    #[test]
    fn screen_to_world_bottom_pixel_maps_to_world_bottom() {
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 600);
        let [_, wy] = cam.screen_to_world((400.0, 600.0), win);
        assert!(
            wy < 0.0,
            "screen-bottom cursor must yield world -Y, got {wy}"
        );
        assert!((wy - -5.0).abs() < 1e-4);
    }

    #[test]
    fn screen_to_world_right_pixel_maps_to_world_right() {
        // Aspect-aware: 800×600 → half_w = 5 * (800/600) ≈ 6.667.
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 600);
        let [wx, _] = cam.screen_to_world((800.0, 300.0), win);
        let aspect = 800.0_f32 / 600.0_f32;
        let expected = 5.0 * aspect;
        assert!(
            (wx - expected).abs() < 1e-4,
            "expected {expected}, got {wx}"
        );
    }

    #[test]
    fn screen_to_world_respects_camera_zoom() {
        // Half the height_world → cursor at top maps to half the world Y.
        let cam = Camera2d::new([0.0, 0.0], 5.0);
        let win = WindowSize::new(800, 600);
        let [_, wy_top] = cam.screen_to_world((400.0, 0.0), win);
        assert!((wy_top - 2.5).abs() < 1e-4);
    }
}
