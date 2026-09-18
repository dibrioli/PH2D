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

mod component;
mod instance;
/// ⭐ **ONDE a sprite lê os pixels** — a célula da folha como RECTÂNGULO (o irmão devolve a UV;
/// este devolve os pixels), com o terceiro leitor: quem prende a imagem ao esqueleto.
mod source_cells;
/// Que pedaço da textura uma sprite amostra — a célula da folha e a região (dois leitores: o
/// extract da shell e o retrato de um prefab).
mod subrect;
mod vertex;

#[cfg(test)]
mod tests;

pub use component::{Sprite, SpriteSource};
pub use instance::{GpuTexRun, RenderInstance};
pub use source_cells::SourceCells;
pub use subrect::{region_subrect, sprite_sheet_subrect};
pub use vertex::QuadVertex;
