#![forbid(unsafe_code)]
//! ph2d-render — sprite render skeleton (M5).
//!
//! Builds the minimal "draw N textured quads from instance data" path
//! end-to-end: explicit pipeline layout (LLM1 audit §10.5: never trust
//! `layout: PipelineLayoutDescriptor::Auto`), explicit bind groups
//! (`@group(0)` frame, `@group(1)` material; per-instance via vertex
//! attributes), dynamic instance buffer, procedural dummy atlas.
//!
//! Two-world boundary (ADR-0021) is honored: [`Sprite`] is a
//! `SimComponent` (canonical state in [`ph2d_ecs::SimWorld`]),
//! [`RenderInstance`] is a `PresentComponent` derived per-frame via
//! [`ph2d_ecs::extract!`] from `Sprite + WorldPos` and uploaded to the
//! instance buffer.
//!
//! M5 explicitly does NOT include: render graph, transient pool
//! integration, batched draws by atlas, sprite sorting, MSAA. Those
//! land in M6+ as the asset pipeline + scene scale demand them.

pub mod atlas;
pub mod camera;
pub(crate) mod clip_pass;
pub mod compositor;
/// Compressed cooked-texture upload path (KTX2 Fase 2, W2.T3). One
/// shared pipeline uploads BC/ASTC/ETC2/RGBA8 (all sample as
/// filterable-float in wgpu 28) with block-aligned `write_texture`. See
/// [`docs/plans/2026-05-texture-compression-waves.md`](../../../docs/plans/2026-05-texture-compression-waves.md).
pub mod compressed_pipeline;
/// Renderer-side cache of cooked KTX2 textures — the W2.T4 loader path that
/// makes [`SpriteSource::CookedTexture`] render. See
/// [`docs/plans/2026-05-texture-compression-waves.md`](../../../docs/plans/2026-05-texture-compression-waves.md).
pub mod cooked_texture;
pub mod game_rt;
pub mod image_filter;
pub mod individual;
pub mod instance_buffer;
/// KTX2 → wgpu texture-format mapping (KTX2 Fase 2, W2.T1). See
/// [`docs/plans/2026-05-texture-compression-waves.md`](../../../docs/plans/2026-05-texture-compression-waves.md).
pub mod ktx2_format;
/// GPU layer compositor (Painter W3, Block 2) — real-time sibling of the CPU
/// reference `ph2d_tool_painter::compositor`. 22 W3C blend modes + groups via a
/// per-pixel stack machine; texture-array cache + dirty-rect dispatch. See
/// [`docs/Painter_projeto/02_layers.md`](../../../docs/Painter_projeto/02_layers.md).
pub mod layer_compositor;
pub mod picking;
pub mod pipeline;
pub mod premul;
/// Straight → premultiplied GPU blit bridging the layer compositor's straight
/// output into the premultiplied Painter preview slot (ADR-0045 Phase 3).
pub mod preview_premul;
pub mod registry;
pub mod renderer;
pub mod sprite;
/// Sprite Inspector v2 — W0 frozen (schema v3 baseline). The W1
/// schema bump v3→v4 + ABI changes live behind this module. See
/// [`docs/HANDOFF_sprite_inspector_v2.md`](../../../docs/HANDOFF_sprite_inspector_v2.md)
/// for the read order, [`docs/Sprite_projeto/`](../../../docs/Sprite_projeto/)
/// for the ratified spec, and ADR-0070 + ADR-0070-amendment-2 for
/// the wrapper-enum back-compat contract.
pub mod sprite_versioned;
pub mod tonemap;
pub mod vello_pass;

pub use atlas::{
    ATLAS_DEFAULT_SIZE_PX, AtlasInsertError, AtlasRegion, DEMO_TILE_COUNT, DEMO_TILE_PX,
    FIRST_IMPORT_KEY, TextureAtlas,
};
pub use camera::{Camera2d, CameraUniform};
pub use compositor::Compositor;
pub use compressed_pipeline::{
    COMPRESSED_TEXTURE_CACHE_BUDGET_MB, CompressedTexturePipeline, CompressedUploadError,
    MipUploadLayout, UploadedCompressedTexture, compressed_size_per_format,
};
pub use cooked_texture::{CookedTextureError, CookedTextureStore};
pub use game_rt::GameRt;
pub use image_filter::{ImageFilterMode, create_sprite_sampler, wgpu_filter};
pub use individual::{IndividualTextureError, IndividualTextureStore};
pub use instance_buffer::InstanceBuffer;
pub use ktx2_format::{CompressionFeatureSet, FormatError, wgpu_format_from_ktx2_format};
pub use layer_compositor::{
    GpuOpScratch, HARD_CAP_LAYERS, LAYER_CACHE_BUDGET_BYTES, LayerCompositeError, LayerCompositor,
    LayerOp, LayerPixelProvider, LayerPixels, MAX_BLUR_HALF, Region, SPATIAL_BLOOM, SPATIAL_CHROMA,
    SPATIAL_GAUSSIAN, SPATIAL_MOTION, SPATIAL_SHADOWS_HIGHLIGHTS, SPATIAL_SHARPEN,
    flatten_layer_ops, gaussian_weights, has_spatial, max_layers_for_budget, motion_weights,
};
pub use picking::{
    WorldBbox, pick_sprite_at_world, pick_sprites_at_world, pick_sprites_in_world_rect,
    selection_bbox_world, sprite_world_to_uv,
};
pub use pipeline::SpritePipeline;
pub use premul::{
    AlphaMode, SpriteImage, premultiply_rgba8, premultiply_rgba8_in_linear, unpremultiply_rgba8,
};
pub use preview_premul::PreviewPremul;
pub use registry::register_render_components;
pub use renderer::SpriteRenderer;
pub use sprite::{QuadVertex, RenderInstance, Sprite, SpriteSource};
// The wrapper enum + the canonical load path (`load_sprite` +
// `LoadError`, ADR-0070-amendment-2 §4) are re-exported at crate root —
// `SpriteV3` stays internal migrator machinery (`#[doc(hidden)]` on the
// struct). Tests reach it via `ph2d_render::sprite_versioned::SpriteV3`.
pub use sprite_versioned::{LoadError, SpriteVersioned, load_sprite};
pub use tonemap::Tonemap;
pub use vello_pass::VelloPass;
