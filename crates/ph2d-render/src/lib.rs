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
pub mod instance_buffer;
pub mod pipeline;
pub mod renderer;
pub mod sprite;

pub use atlas::TextureAtlas;
pub use camera::{Camera2d, CameraUniform};
pub use instance_buffer::InstanceBuffer;
pub use pipeline::SpritePipeline;
pub use renderer::SpriteRenderer;
pub use sprite::{QuadVertex, RenderInstance, Sprite};
