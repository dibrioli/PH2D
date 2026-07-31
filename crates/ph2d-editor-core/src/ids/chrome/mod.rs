//! Stable [`NodeId`] constants for the hero screen's interactive widgets,
//! split per UI domain. Pure organizational split (ADR-0075 isolation
//! hygiene) — every const/fn keeps its exact public path: the parent
//! `ids` module re-exports `chrome::*`, and this module re-exports every
//! submodule, so the whole workspace still reaches them as
//! `ph2d_editor_core::ids::<NAME>`.
//!
//! Submodules group by name prefix (TOPBAR_*, IMAGE_ACTION_*, PAD_*,
//! VGRAPH_*/VECTOR_INSPECTOR_*, TOOL_*/RAIL_*, PAINTER_*, BGR_*, INSP_*).
//! See the parent `ids/mod.rs` doc-comment for the NodeId derivation
//! convention (FNV-1a slug hashing).

// Bring the symbols the submodules reference via `use super::*;` into this
// module's namespace, so each submodule resolves `NodeId` / `hash_node_id`
// through `super` (this module) exactly as the old flat file did via the
// parent `ids` module.
use super::{NodeId, hash_node_id};

mod bgremoval;
mod flip;
mod image_tools;
mod inspector;
mod motion;
mod padding;
mod painter;
mod painter_brush_sections;
mod painter_deform;
mod painter_fill;
mod painter_gradient;
mod painter_impasto;
mod painter_sculpt;
mod painter_selection;
mod painter_shape;
mod painter_stroke_op;
mod painter_studio;
mod painter_symmetry;
mod painter_texture;
mod painter_tiling;
mod painter_watercolor;
mod painter_wetpaint;
mod physics;
mod rail;
mod rail_painter;
mod timeline;
mod topbar;
mod vector;
/// Os ids da seção **Contour** (pesquisa `20_*` #9) — irmão de `vector` pelo teto de 700 LOC.
mod vector_contour;
/// Os ids da seção **Filters** (FX raster, plano 24) — irmão de `vector` pelo teto de 700 LOC.
mod vector_filters;
/// Os ids da seção **Pattern on Path** (plano 23) — irmão de `vector` pelo teto de 700 LOC.
mod vector_patternpath;
/// Os ids da seção **Pencil** (a mão livre, plano 25) — irmão de `vector` pelo teto de 700 LOC.
mod vector_pencil;
/// A lista das seções colapsáveis do painel Vector — irmã de `vector` pelo teto de 700 LOC.
mod vector_sections;
/// Os ids da seção **Text on Path** — irmão de `vector` pelo teto de 700 LOC.
mod vector_textpath;
mod vector_width;
mod wet_tuning;

pub use bgremoval::*;
pub use flip::*;
pub use image_tools::*;
pub use inspector::*;
pub use motion::*;
pub use padding::*;
pub use painter::*;
pub use painter_brush_sections::*;
pub use painter_deform::*;
pub use painter_fill::*;
pub use painter_gradient::*;
pub use painter_impasto::*;
pub use painter_sculpt::*;
pub use painter_selection::*;
pub use painter_shape::*;
pub use painter_stroke_op::*;
pub use painter_studio::*;
pub use painter_symmetry::*;
pub use painter_texture::*;
pub use painter_tiling::*;
pub use painter_watercolor::*;
pub use painter_wetpaint::*;
pub use physics::*;
pub use rail::*;
pub use rail_painter::*;
pub use timeline::*;
pub use topbar::*;
pub use vector::*;
pub use vector_contour::*;
pub use vector_filters::*;
pub use vector_patternpath::*;
pub use vector_pencil::*;
pub use vector_sections::*;
pub use vector_textpath::*;
pub use vector_width::*;
pub use wet_tuning::*;
