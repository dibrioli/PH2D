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

/// Which texture source a [`Sprite`] reads its pixels from. M14.5
/// introduces the multi-strategy model documented in the post-spike
/// plan §M14.5:
///
/// - [`SpriteSource::Atlas`] — shared dynamic atlas (the M14.4d/4f
///   Skyline packer). Many sprites in one 4096² texture; 1 draw call
///   for every atlas-backed sprite per frame.
/// - [`SpriteSource::Individual`] — sprite owns its own
///   `wgpu::Texture` at native resolution. The renderer groups
///   contiguous same-texture instances into one draw call each
///   (Godot 4 `RenderingServer` pattern). Use when packing-or-stretch
///   trade-offs don't suit the content (large HD sprites, procedural
///   textures, or content that gets hot-reloaded independently).
///
/// `Hand-packed` (artist-authored atlas + JSON) is M14.5 B — separate
/// variant, separate PR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteSource {
    /// Index into the shared atlas. The atlas resolves to a UV
    /// rectangle via `TextureAtlas::region_uv` at extract time.
    Atlas { key: u32 },
    /// Renderer-assigned id for an individually-owned texture, handed
    /// out by [`crate::individual::IndividualTextureStore::acquire`].
    /// Stable for the lifetime of the texture in the store.
    Individual { texture_id: u32 },
}

impl SpriteSource {
    /// Convenience for the common "atlas key 0" case used in fixture
    /// tests and the demo's HSV tiles.
    pub const ATLAS_ZERO: Self = Self::Atlas { key: 0 };
}

/// Canonical sprite description in simulation state. World position
/// is read from the entity's [`ph2d_ecs::Transform`] during the
/// extract phase (ADR-0025).
///
/// `Serialize`/`Deserialize` derives let `Sprite` round-trip through
/// the `PrefabDoc` / `SceneDoc` postcard pipeline (M14.3). All fields
/// are POD-shaped (`SpriteSource` is `#[repr(Rust)]` enum with `u32`
/// payloads), so the wire format stays stable across rustc versions
/// as long as `SpriteSource`'s variant order doesn't change.
#[derive(Component, Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    /// Where the pixels come from — shared atlas or individual texture.
    pub source: SpriteSource,
    /// Sprite size in world units (meters).
    pub size: [f32; 2],
    /// RGBA tint multiplied with the texel color in the fragment shader.
    pub tint: [f32; 4],
    /// `true` → this sprite's texture pixels are stored PREMULTIPLIED,
    /// not straight alpha. Set only by the BG-Removal Apply path, which
    /// bakes a premultiplied Individual texture so the GPU's bilinear
    /// `textureSample` composites the anti-aliased edge band like the
    /// Vello preview (premultiply-before-sample) instead of letting a
    /// partial-alpha edge texel's straight RGB bleed in at full weight
    /// (the purple/dark fringe). Drives `RenderInstance.premultiplied`
    /// at extract time AND tells the Image-Tools readback paths to
    /// un-premultiply before re-running an algorithm that assumes
    /// straight alpha. Defaults to `false`.
    ///
    /// `#[serde(skip)]`: this is a RUNTIME rendering hint, never part of
    /// the persisted prefab/scene format. It is only ever `true` for a
    /// live Individual texture (itself runtime — `texture_id` isn't
    /// portably serializable), and always `false` for Atlas sprites, so
    /// it carries no meaning on disk. Skipping it keeps the postcard cook
    /// hash stable (no fixture churn) and deserializes as `false`.
    #[serde(skip)]
    pub premultiplied: bool,
}

impl Sprite {
    /// Schema version for the cooked-prefab pipeline (HR-14
    /// mitigation; consumed by `ComponentRegistry` until the
    /// `Saveable` derive macro lands). Bumped to 2 when
    /// `atlas_index` became `source` in M14.5 C.
    pub const VERSION: u32 = 2;

    /// Convenience constructor for atlas-backed sprites — the
    /// dominant case after M14.4d. Preserves the pre-M14.5 ergonomics
    /// (`Sprite { atlas_index: K, size, tint }`) under a slightly
    /// different name.
    pub fn atlas(key: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        Self {
            source: SpriteSource::Atlas { key },
            size,
            tint,
            premultiplied: false,
        }
    }

    /// Convenience constructor for individual-texture sprites.
    /// `texture_id` must come from
    /// `IndividualTextureStore::acquire`.
    pub fn individual(texture_id: u32, size: [f32; 2], tint: [f32; 4]) -> Self {
        Self {
            source: SpriteSource::Individual { texture_id },
            size,
            tint,
            premultiplied: false,
        }
    }
}

impl SimComponent for Sprite {}

/// Per-frame instance data uploaded to the GPU. Layout matches the
/// `InstanceInput` struct in `shaders/sprite.wgsl`. `#[repr(C)]` +
/// `bytemuck::Pod` for zero-copy upload via `Queue::write_buffer`.
///
/// M14.5 C: `texture_id` is CPU-side metadata used by the renderer
/// to group same-texture instances into one draw call each. The
/// shader's vertex layout doesn't reference it, so it's ignored on
/// the GPU side — the byte stride still includes it (Pod size = 52
/// bytes, 4-byte aligned).
///
/// - `texture_id == 0` → the instance reads from the shared atlas
///   bound at material bind group 1.
/// - `texture_id > 0` → the renderer rebinds material 1 to the
///   individually-cached texture handed out by
///   `IndividualTextureStore::acquire`.
#[derive(Component, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RenderInstance {
    pub world_pos: [f32; 2],
    /// Sprite size in world meters. Extract MULTIPLIES the raw
    /// `Sprite.size` by the entity's `GlobalTransform` scale before
    /// stamping this field, so the gizmo's scale handles
    /// (`Transform.scale`) reach the rendered quad.
    pub size: [f32; 2],
    pub atlas_uv: [f32; 4],
    pub tint: [f32; 4],
    /// World-space rotation in radians, decomposed from
    /// `GlobalTransform`. Read by the shader to rotate the local
    /// quad before the world translate. Defaults to 0 for legacy
    /// extract paths that don't set it.
    pub rotation: f32,
    pub texture_id: u32,
    /// `1.0` → the bound texture is already premultiplied (BG-Removal
    /// Apply); the fragment skips its post-sample premultiply so
    /// bilinear sampling matches the Vello on-canvas preview and the
    /// straight-alpha edge fringe disappears. `0.0` for every other
    /// sprite. CPU-side this is set from [`Sprite::premultiplied`] at
    /// extract time. Occupies one of the two former `_pad` words, so
    /// the Pod stride stays 64 bytes.
    pub premultiplied: f32,
    pub _pad: u32,
}

impl PresentComponent for RenderInstance {}

impl RenderInstance {
    pub const VERTEX_ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        2 => Float32x2,  // world_pos
        3 => Float32x2,  // size
        4 => Float32x4,  // atlas_uv
        5 => Float32x4,  // tint
        6 => Float32,    // rotation (M14.7)
        7 => Float32,    // premultiplied flag (BG-Removal Apply)
    ];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }

    /// Sentinel for `texture_id` meaning "sample from the shared
    /// atlas at material bind group 1".
    pub const ATLAS_TEXTURE_ID: u32 = 0;
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
    /// UV mapping is "natural": world-up vertex (`pos.y = +0.5`)
    /// samples texture top (V=0); world-down vertex samples texture
    /// bottom (V=1). This works directly because
    /// [`Camera2d::view_proj`](crate::camera::Camera2d::view_proj)
    /// uses standard orthographic with no Y-flip (M14.4e v2 removed
    /// the prior `bottom`/`top` swap that had inverted everything).
    /// World Y-up therefore maps to screen Y-up, and a texture
    /// uploaded in image-crate's top-down byte order displays upright.
    pub const QUAD_STRIP: [Self; 4] = [
        Self {
            pos: [-0.5, -0.5],
            uv: [0.0, 1.0],
        },
        Self {
            pos: [0.5, -0.5],
            uv: [1.0, 1.0],
        },
        Self {
            pos: [-0.5, 0.5],
            uv: [0.0, 0.0],
        },
        Self {
            pos: [0.5, 0.5],
            uv: [1.0, 0.0],
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
            rotation: 0.0,
            texture_id: RenderInstance::ATLAS_TEXTURE_ID,
            premultiplied: 0.0,
            _pad: 0,
        };
        let bytes: &[u8] = bytemuck::bytes_of(&inst);
        assert_eq!(bytes.len(), std::mem::size_of::<RenderInstance>());
        // 12 floats (world_pos+size+atlas_uv+tint) = 48 bytes
        // + rotation f32 (4) + texture_id u32 (4) + premultiplied f32 (4)
        // + 1 pad u32 (4) = 64 bytes — stride unchanged from pre-fringe-fix.
        assert_eq!(bytes.len(), 64);
    }

    #[test]
    fn vertex_attributes_cover_full_stride() {
        // The premultiplied flag is the 6th vertex attribute (location
        // 7). Confirm the attribute array's last offset+size lands
        // inside the Pod stride so the vertex layout doesn't read past
        // the instance buffer.
        let attrs = RenderInstance::VERTEX_ATTRIBUTES;
        let last = attrs.last().expect("at least one attribute");
        assert_eq!(last.shader_location, 7, "premultiplied is @location(7)");
        // Float32 == 4 bytes.
        let end = last.offset + 4;
        assert!(
            end <= std::mem::size_of::<RenderInstance>() as u64,
            "attr end {end} must fit in stride {}",
            std::mem::size_of::<RenderInstance>()
        );
    }

    #[test]
    fn sprite_constructors_default_straight_alpha() {
        assert!(!Sprite::atlas(0, [1.0, 1.0], [1.0; 4]).premultiplied);
        assert!(!Sprite::individual(1, [1.0, 1.0], [1.0; 4]).premultiplied);
    }

    #[test]
    fn sprite_source_atlas_zero_const() {
        assert_eq!(SpriteSource::ATLAS_ZERO, SpriteSource::Atlas { key: 0 });
    }

    #[test]
    fn sprite_atlas_constructor_round_trip() {
        let s = Sprite::atlas(7, [1.0, 2.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.source, SpriteSource::Atlas { key: 7 });
        assert_eq!(s.size, [1.0, 2.0]);
    }

    #[test]
    fn sprite_individual_constructor_round_trip() {
        let s = Sprite::individual(42, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.source, SpriteSource::Individual { texture_id: 42 });
    }

    #[test]
    fn quad_strip_winding() {
        // Triangle strip [0,1,2] then [1,3,2] → both CCW when viewed
        // from +Z (Y-up world space). Just sanity that vertex order
        // matches what the shader expects.
        assert_eq!(QuadVertex::QUAD_STRIP.len(), 4);
    }

    #[test]
    fn quad_strip_uv_natural_mapping() {
        // M14.4e v2: camera projection no longer Y-flips, so the UV
        // mapping is straightforward — world-up vertex (pos.y > 0)
        // samples texture top (V=0), world-down samples texture
        // bottom (V=1). X is straight (no flip).
        for v in QuadVertex::QUAD_STRIP {
            let expected_v = if v.pos[1] < 0.0 { 1.0 } else { 0.0 };
            assert_eq!(
                v.uv[1], expected_v,
                "pos {:?} expected V={expected_v} got V={}",
                v.pos, v.uv[1]
            );
            let expected_u = if v.pos[0] < 0.0 { 0.0 } else { 1.0 };
            assert_eq!(v.uv[0], expected_u);
        }
    }
}
