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
    /// Visibility-layer cull mask (Sprite Inspector v2 W3.T3.12). A
    /// sprite renders for this camera only when its
    /// `ph2d_ecs::VisibilityLayer` mask intersects `cull_mask`; absence
    /// of the component = visible. CPU-only (drives the extract cull),
    /// never uploaded to `CameraUniform`. Default `u32::MAX` = every
    /// layer visible (no culling).
    pub cull_mask: u32,
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
            cull_mask: u32::MAX,
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
        // Pan semantics: world content under the cursor follows the
        // cursor drag direction (Photoshop/Figma "hand" pan). With
        // the projection's Y-flip (`view_proj` swaps bottom/top so
        // world Y-up maps to screen Y-down NDC), the perceived
        // screen-down drag corresponds to camera UP in world coords.
        //
        // Empirical result the user validated: cursor down → camera
        // center[1] increases. X is straight (cursor right → camera
        // left in world).
        //
        // History: M14.4e shipped with `center[1] -= dy_world`
        // assuming a different projection convention; the user
        // reported Y still inverted in two consecutive builds, which
        // pointed at the actual Y-flip direction being opposite to
        // the textbook expectation. Sign reverted to `+=` here.
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
        // Cursor coords are Y-down (top of window = cy_px=0). World
        // is Y-up. Cursor at screen TOP must map to world's HIGH Y
        // (top of camera view) — so we subtract `ny * half_h` to
        // invert the Y direction during the projection inverse.
        let nx = (cx_px / w) * 2.0 - 1.0;
        let ny = (cy_px / h) * 2.0 - 1.0;
        let aspect = w / h;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let world_x = self.center[0] + nx * half_w;
        let world_y = self.center[1] - ny * half_h;
        [world_x, world_y]
    }

    /// Project a world-space point (meters) to a screen-pixel position
    /// (origin at the window's top-left, Y-down). Exact inverse of
    /// [`Camera2d::screen_to_world`] — world Y-up maps so the largest
    /// world Y lands at the smallest screen Y (top of the window).
    ///
    /// Used by overlays that must align to a sprite's on-canvas
    /// footprint without a GPU readback — e.g. the Background-Removal
    /// live preview blit.
    pub fn world_to_screen(&self, world: [f32; 2], window: WindowSize) -> (f32, f32) {
        let w = window.width.max(1) as f32;
        let h = window.height.max(1) as f32;
        let aspect = w / h;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        // Inverse of `screen_to_world`'s nx/ny derivation.
        let nx = (world[0] - self.center[0]) / half_w;
        let ny = (self.center[1] - world[1]) / half_h;
        let cx_px = (nx + 1.0) * 0.5 * w;
        let cy_px = (ny + 1.0) * 0.5 * h;
        (cx_px, cy_px)
    }

    /// World-meters → screen-pixel [`vello::kurbo::Affine`] — the matrix
    /// form of [`Camera2d::world_to_screen`], for callers that hand an
    /// `Affine` to a renderer (e.g. the Vector Pen bridge feeding
    /// `vello::Scene::stroke`/`fill`) instead of projecting points one at
    /// a time.
    ///
    /// Single source of the projection: derived from the SAME uniform
    /// scale `k = window.height / height_world` and Y-flip as
    /// `world_to_screen`/`screen_to_world`, so a shell can no longer carry
    /// a hand-rolled copy that silently diverges (Vector audit H5/M3). The
    /// returned affine maps `(world_x, world_y)` to `(screen_x, screen_y)`
    /// with origin at the window's top-left, Y-down.
    ///
    /// Assumes square pixels (uniform `k`); if `Camera2d` ever becomes
    /// anisotropic, this and `world_to_screen` must change together.
    pub fn world_to_screen_affine(&self, window: WindowSize) -> vello::kurbo::Affine {
        use vello::kurbo::Affine;
        let w = window.width.max(1) as f64;
        let h = window.height.max(1) as f64;
        let k = h / (self.height_world as f64).max(1e-6);
        Affine::translate((w * 0.5, h * 0.5))
            * Affine::scale_non_uniform(k, -k)
            * Affine::translate((-(self.center[0] as f64), -(self.center[1] as f64)))
    }

    /// Build the view-projection matrix for the given window.
    /// Standard right-handed orthographic — world Y-up maps to wgpu
    /// clip space Y-up (per WebGPU spec §3.4), which means world top
    /// renders at screen top with no extra inversion.
    ///
    /// History (M14.4e v2): an earlier swap of `bottom`/`top` here
    /// produced a Y-flip ("world Y-up → clip Y-down NDC") that
    /// caused sprites to render mirrored relative to the grid + the
    /// drag cursor — the user reported "mouse e grid descem enquanto
    /// sprites sobem". Removing the swap aligns ALL Y consumers
    /// (sprites, grid painter, screen_to_world inverse, pan delta)
    /// to the same direction, and the QUAD_STRIP UV in
    /// `crates/ph2d-render/src/sprite.rs` is restored to its
    /// pre-M14.4d mapping (world-up → tex-top, V=0) so imported
    /// images still render upright.
    pub fn view_proj(&self, window: WindowSize) -> Mat4 {
        let aspect = window.width.max(1) as f32 / window.height.max(1) as f32;
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let cx = self.center[0];
        let cy = self.center[1];
        Mat4::orthographic_rh(
            cx - half_w,
            cx + half_w,
            cy - half_h,
            cy + half_h,
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

    /// View-projection for rendering into a **sub-rect** of the target (Motion
    /// Nodes M0.T13 — the split viewport⟂graph). The aspect comes from the
    /// sub-rect's pixel dimensions (not the full window), so when the caller
    /// pairs this with `set_viewport`/`set_scissor_rect` the scene fills the
    /// sub-rect undistorted instead of being squashed by the window aspect.
    /// Center + `height_world` are unchanged (same framing, smaller area).
    /// Passing the full window dims yields exactly [`Self::view_proj`].
    pub fn view_proj_for_subrect(&self, subrect_w_px: f32, subrect_h_px: f32) -> Mat4 {
        let aspect = subrect_w_px.max(1.0) / subrect_h_px.max(1.0);
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let cx = self.center[0];
        let cy = self.center[1];
        Mat4::orthographic_rh(cx - half_w, cx + half_w, cy - half_h, cy + half_h, -1.0, 1.0)
    }

    /// [`CameraUniform`] form of [`Self::view_proj_for_subrect`].
    pub fn uniform_for_subrect(&self, subrect_w_px: f32, subrect_h_px: f32) -> CameraUniform {
        CameraUniform {
            view_proj: self
                .view_proj_for_subrect(subrect_w_px, subrect_h_px)
                .to_cols_array_2d(),
        }
    }
}

impl Default for Camera2d {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            height_world: 10.0,
            cull_mask: u32::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cull_mask_defaults_to_all_layers_visible() {
        // W3.T3.12: a fresh camera culls nothing (every VisibilityLayer
        // intersects u32::MAX), so the extract's cull is a no-op until a
        // non-default mask is set.
        assert_eq!(Camera2d::default().cull_mask, u32::MAX);
        assert_eq!(Camera2d::new([1.0, 2.0], 5.0).cull_mask, u32::MAX);
    }

    #[test]
    fn subrect_uniform_uses_the_subrect_aspect() {
        let cam = Camera2d::new([0.0, 0.0], 10.0); // half_h = 5
        // A 2:1 sub-rect → half_w = half_h * 2 = 10. Orthographic scale on x is
        // 2/(right-left) = 1/half_w; on y it's 1/half_h.
        let m = cam.view_proj_for_subrect(400.0, 200.0).to_cols_array_2d();
        assert!((m[0][0] - 0.1).abs() < 1e-6, "x scale = 1/half_w = 0.1");
        assert!((m[1][1] - 0.2).abs() < 1e-6, "y scale = 1/half_h = 0.2");
    }

    #[test]
    fn subrect_with_window_dims_equals_full_window_view_proj() {
        // Passing the whole window through the sub-rect path must be identical to
        // the full-window projection (no special-casing needed at the call site).
        let cam = Camera2d::new([3.0, -2.0], 8.0);
        let win = WindowSize::new(1280, 720);
        let full = cam.view_proj(win).to_cols_array_2d();
        let sub = cam
            .view_proj_for_subrect(win.width as f32, win.height as f32)
            .to_cols_array_2d();
        for c in 0..4 {
            for r in 0..4 {
                assert!((full[c][r] - sub[c][r]).abs() < 1e-6, "mismatch at [{c}][{r}]");
            }
        }
    }

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
    fn world_to_screen_round_trips_screen_to_world() {
        let cam = Camera2d::new([3.0, -2.0], 8.0);
        let win = WindowSize {
            width: 800,
            height: 600,
        };
        for px in [(0.0, 0.0), (400.0, 300.0), (799.0, 599.0), (123.0, 456.0)] {
            let world = cam.screen_to_world(px, win);
            let (sx, sy) = cam.world_to_screen(world, win);
            assert!((sx - px.0).abs() < 1e-3, "x: {sx} vs {}", px.0);
            assert!((sy - px.1).abs() < 1e-3, "y: {sy} vs {}", px.1);
        }
    }

    #[test]
    fn world_to_screen_affine_matches_world_to_screen() {
        // H5/M3: the Affine MUST produce the same screen pixels as the
        // per-point `world_to_screen`, so the Vector shell bridge can drop
        // its hand-rolled copy without any visual drift. Pure scale +
        // translate (no trig) → no libm/determinism concern (HR-5).
        let cam = Camera2d::new([3.0, -2.0], 8.0);
        let win = WindowSize {
            width: 800,
            height: 600,
        };
        let affine = cam.world_to_screen_affine(win);
        for world in [[0.0, 0.0], [3.0, -2.0], [7.5, 4.25], [-12.0, 9.0]] {
            let (ex, ey) = cam.world_to_screen(world, win);
            let p = affine * vello::kurbo::Point::new(world[0] as f64, world[1] as f64);
            assert!((p.x - ex as f64).abs() < 1e-3, "x: {} vs {ex}", p.x);
            assert!((p.y - ey as f64).abs() < 1e-3, "y: {} vs {ey}", p.y);
        }
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
        // With the projection's Y-flip (view_proj swaps bottom/top so
        // world Y-up renders as screen Y-down), cursor moving DOWN on
        // screen corresponds to camera moving UP in world Y-up coords
        // — keeping the visible world point under the cursor (Photoshop
        // "hand" pan). cursor +60 px @ 60 px/m → center.y += 1.
        let mut cam = Camera2d::default();
        cam.pan_screen_delta(0.0, 60.0, 800.0, 600.0);
        assert!(
            (cam.center[1] - 1.0).abs() < 1e-3,
            "expected center.y = +1.0, got {}",
            cam.center[1]
        );
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
    fn y_up_world_maps_to_positive_clip_y() {
        // Standard ortho (no Y-flip post-M14.4e v2): world Y-up
        // maps to clip Y-up (wgpu NDC has +1 at top per WebGPU spec).
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 800);
        let m = cam.view_proj(win);
        let p = m * ph2d_core::Vec4::new(0.0, 5.0, 0.0, 1.0);
        assert!(
            p.y > 0.0,
            "world +Y should map to positive clip Y, got {}",
            p.y
        );
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
    fn screen_to_world_top_pixel_maps_to_positive_world_y() {
        // World Y-up, standard projection (no Y-flip): cursor at
        // screen top maps to the TOP of the camera view in world
        // coords = cy + half_h.
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 600);
        let [_, wy] = cam.screen_to_world((400.0, 0.0), win);
        assert!(wy > 0.0, "screen-top maps to world +Y, got {wy}");
        assert!((wy - 5.0).abs() < 1e-4);
    }

    #[test]
    fn screen_to_world_bottom_pixel_maps_to_negative_world_y() {
        let cam = Camera2d::new([0.0, 0.0], 10.0);
        let win = WindowSize::new(800, 600);
        let [_, wy] = cam.screen_to_world((400.0, 600.0), win);
        assert!(wy < 0.0, "screen-bottom maps to world -Y, got {wy}");
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
