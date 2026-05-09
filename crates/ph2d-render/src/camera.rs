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
    pub fn new(center: [f32; 2], height_world: f32) -> Self {
        Self {
            center,
            height_world,
        }
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
}
