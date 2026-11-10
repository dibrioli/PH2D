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

/// A UV final de uma sprite: de que rect do atlas a célula se corta e onde cai o meio-texel.
mod cell_uv;
mod component;
mod image_of;
mod instance;
/// As folhas extra do Animated Sprite no ecrã (plano GEF 26).
mod sheets;
/// ⭐ **ONDE a sprite lê os pixels** — a célula da folha como RECTÂNGULO (o irmão devolve a UV;
/// este devolve os pixels), com o terceiro leitor: quem prende a imagem ao esqueleto.
mod source_cells;
/// Que pedaço da textura uma sprite amostra — a célula da folha e a região (dois leitores: o
/// extract da shell e o retrato de um prefab).
mod subrect;
mod vertex;

#[cfg(test)]
mod tests;

pub use cell_uv::{CellSource, CellUv, CellUvInput, sprite_cell_uv};
pub use component::{Sprite, SpriteSource};
pub use image_of::{OverrideLaw, drawn_texture, images_of, texture_of, texture_of_entity};
pub use instance::{GpuTexRun, RenderInstance};
pub use sheets::{
    SheetTextures, SheetView, drawn_geometry, nudged, open_preview_cell, resolve_sheet_textures,
    sheet_geometry, sheet_sprite, sheet_view, upload_sheet_image,
};
pub use source_cells::SourceCells;
pub use subrect::{inset_half_texel, region_subrect, sprite_sheet_subrect, sprite_wraps};
pub use vertex::QuadVertex;
