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
mod image_tools;
mod inspector;
mod padding;
mod painter;
mod painter_brush_sections;
mod painter_studio;
mod painter_texture;
mod painter_tiling;
mod rail;
mod topbar;
mod vector;

pub use bgremoval::*;
pub use image_tools::*;
pub use inspector::*;
pub use padding::*;
pub use painter::*;
pub use painter_brush_sections::*;
pub use painter_studio::*;
pub use painter_texture::*;
pub use painter_tiling::*;
pub use rail::*;
pub use topbar::*;
pub use vector::*;
