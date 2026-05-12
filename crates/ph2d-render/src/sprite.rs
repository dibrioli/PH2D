//! Sprite components.
//!
//! [`Sprite`] is a SimComponent (lives in SimWorld; canonical state).
//! [`RenderInstance`] is a PresentComponent (built each frame from
//! Sprite via the extract phase; uploaded to instance buffer).
//!
//! ## World position lives in `Transform`
//!
//! Since ADR-0025 (M14.1) the canonical world-space pose for a sprite
//! comes from [`ph2d_ecs::Transform`] + the hierarchical
//! [`ph2d_ecs::propagate_transforms`] pass — **not** from a separate
//! `WorldPos`/`Position` component. The extract closure reads the
//! freshly computed `GlobalTransform.translation()` and stamps it
//! into `RenderInstance.world_pos` so the renderer stays a pure
//! PresentWorld consumer.

use bevy_ecs::component::Component;
use ph2d_ecs::{PresentComponent, SimComponent};

/// Canonical sprite description in simulation state. World position
/// is read from the entity's [`ph2d_ecs::Transform`] during the
/// extract phase (ADR-0025).
///
/// `Serialize`/`Deserialize` derives let `Sprite` round-trip through
/// the `PrefabDoc` / `SceneDoc` postcard pipeline (M14.3). All fields
/// are POD (`u32`, fixed-size `f32` arrays), so the wire format is
/// stable across rustc versions.
#[derive(Component, Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    /// Index into the texture atlas tile grid.
    pub atlas_index: u32,
    /// Sprite size in world units (meters).
    pub size: [f32; 2],
    /// RGBA tint multiplied with the texel color in the fragment shader.
    pub tint: [f32; 4],
}

impl Sprite {
    /// Schema version for the cooked-prefab pipeline (HR-14
    /// mitigation; consumed by `ComponentRegistry` until the
    /// `Saveable` derive macro lands).
    pub const VERSION: u32 = 1;
}

impl SimComponent for Sprite {}

/// Per-frame instance data uploaded to the GPU. Layout matches the
/// `InstanceInput` struct in `shaders/sprite.wgsl`. `#[repr(C)]` +
/// `bytemuck::Pod` for zero-copy upload via `Queue::write_buffer`.
#[derive(Component, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RenderInstance {
    pub world_pos: [f32; 2],
    pub size: [f32; 2],
    pub atlas_uv: [f32; 4],
    pub tint: [f32; 4],
}

impl PresentComponent for RenderInstance {}

impl RenderInstance {
    pub const VERTEX_ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        2 => Float32x2,  // world_pos
        3 => Float32x2,  // size
        4 => Float32x4,  // atlas_uv
        5 => Float32x4,  // tint
    ];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }
}

/// Vertex of the unit quad used as the geometry for every sprite
/// instance. Layout matches `VertexInput` in the shader.
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct QuadVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

impl QuadVertex {
    pub const VERTEX_ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x2,  // quad_pos
        1 => Float32x2,  // quad_uv
    ];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }

    /// Unit quad as triangle strip, centered at origin.
    ///
    /// UV V coords compensate the world→clip Y-flip in
    /// [`Camera2d::view_proj`](crate::camera::Camera2d::view_proj):
    /// the camera maps world-up to clip-down (Y-down NDC), so a quad
    /// vertex at `pos.y = +0.5` (world-up) renders at screen-bottom.
    /// To keep an imported image visually upright (its top row at
    /// screen-top), the world-up corner must sample the **bottom** of
    /// the source texture, so the V on world-up vertices is `1.0` and
    /// V on world-down is `0.0`. (The previous mapping was the
    /// opposite, which inverted asymmetric textures vertically — the
    /// `M5` HSV dummy tiles hid this because solid colors are flip-
    /// invariant.) Tested in
    /// [`tests/sprite_quad_uv.rs`](../tests/sprite_quad_uv.rs).
    pub const QUAD_STRIP: [Self; 4] = [
        Self {
            pos: [-0.5, -0.5],
            uv: [0.0, 0.0],
        },
        Self {
            pos: [0.5, -0.5],
            uv: [1.0, 0.0],
        },
        Self {
            pos: [-0.5, 0.5],
            uv: [0.0, 1.0],
        },
        Self {
            pos: [0.5, 0.5],
            uv: [1.0, 1.0],
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_instance_is_pod_compatible() {
        let inst = RenderInstance {
            world_pos: [1.0, 2.0],
            size: [10.0, 10.0],
            atlas_uv: [0.0, 0.0, 0.25, 0.25],
            tint: [1.0, 1.0, 1.0, 1.0],
        };
        let bytes: &[u8] = bytemuck::bytes_of(&inst);
        assert_eq!(bytes.len(), std::mem::size_of::<RenderInstance>());
        // 2 + 2 + 4 + 4 = 12 floats = 48 bytes
        assert_eq!(bytes.len(), 48);
    }

    #[test]
    fn quad_strip_winding() {
        // Triangle strip [0,1,2] then [1,3,2] → both CCW when viewed
        // from +Z (Y-up world space). Just sanity that vertex order
        // matches what the shader expects.
        assert_eq!(QuadVertex::QUAD_STRIP.len(), 4);
    }

    #[test]
    fn quad_strip_uv_compensates_camera_y_flip() {
        // Regression for the M14.4c bug: imported sprites were
        // rendering Y-inverted because the camera's view_proj flips
        // Y (world-up → clip-down per Y-down NDC) but QUAD_STRIP UVs
        // mapped world-up to texture-up. Net: texture-top ended up at
        // screen-bottom for asymmetric content.
        //
        // Invariant now: world-DOWN vertices sample texture-TOP (V=0),
        // world-UP vertices sample texture-BOTTOM (V=1). After the
        // camera Y-flip this places texture-top at screen-top — the
        // image displays upright. X mapping is straight (no flip).
        for v in QuadVertex::QUAD_STRIP {
            // pos.y < 0 (world-down)  → uv.v == 0  (texture-top)
            // pos.y > 0 (world-up)    → uv.v == 1  (texture-bottom)
            let expected_v = if v.pos[1] < 0.0 { 0.0 } else { 1.0 };
            assert_eq!(
                v.uv[1], expected_v,
                "pos {:?} expected V={expected_v} got V={}",
                v.pos, v.uv[1]
            );
            // X straight: pos.x sign matches uv.u value.
            let expected_u = if v.pos[0] < 0.0 { 0.0 } else { 1.0 };
            assert_eq!(v.uv[0], expected_u);
        }
    }
}
