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
pub mod compositor;
pub mod game_rt;
pub mod individual;
pub mod instance_buffer;
pub mod picking;
pub mod pipeline;
pub mod registry;
pub mod renderer;
pub mod sprite;
pub mod tonemap;
pub mod vello_pass;

pub use atlas::{
    ATLAS_DEFAULT_SIZE_PX, AtlasInsertError, AtlasRegion, DEMO_TILE_COUNT, DEMO_TILE_PX,
    FIRST_IMPORT_KEY, TextureAtlas,
};
pub use camera::{Camera2d, CameraUniform};
pub use compositor::Compositor;
pub use game_rt::GameRt;
pub use individual::{IndividualTextureError, IndividualTextureStore};
pub use instance_buffer::InstanceBuffer;
pub use picking::{WorldBbox, pick_sprite_at_world, pick_sprites_at_world, selection_bbox_world};
pub use pipeline::SpritePipeline;
pub use registry::register_render_components;
pub use renderer::SpriteRenderer;
pub use sprite::{QuadVertex, RenderInstance, Sprite, SpriteSource};
pub use tonemap::Tonemap;
pub use vello_pass::VelloPass;
