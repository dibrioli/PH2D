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

/// **Os ids do NAVEGADOR DE ASSETS** (plano `docs/Components/07`) — a grade, as buscas e os
/// chips. A porta de entrada e' o `TOPBAR_RIGHT_ASSETS`, que ja' existia e nao despachava nada.
mod asset_browser;
/// **Os ids do painel AUTORADO** (plano UI/UX W8b.2) — o painel que o artista desenhou,
/// vivo. As rows nao existem em tempo de escrita: os ids delas sao derivados da CHAVE.
mod authored;
mod bgremoval;
mod flip;
mod image_tools;
mod input_map;
mod inspector;
/// **Os ids do painel de MODELAGEM 3D** (ADR-0161) — não confundir com `sculpt3d`.
mod model3d;
mod motion;
mod padding;
mod painter;
mod painter_fill;
mod physics;
/// **Os ids da BARRA DO MODO de edição de receita** — chrome do canvas, não de um painel.
mod prefab;
mod rail;
mod rail_painter;
/// **Os ids do painel da cena 3D** (ADR-0150, W12) — a ferramenta, o pincel, o espelho,
/// a topologia e o sombreamento. Categoria MUNDO, como o de física.
mod sculpt3d;
mod timeline;
/// **Os ids do painel de TOKENS** (plano UI/UX W6) — a tabela de cor do design system,
/// autorável pelo artista. Categoria MUNDO, como o de física.
mod tokens;
mod topbar;
mod vector;
/// ⭐ **Os ids do ESQUELETO** (estudo 42 item 5) — o modo Osso e a seção que prende a forma a ele.
mod vector_bone;
mod wet_tuning;

pub use asset_browser::*;
pub use authored::*;
pub use bgremoval::*;
pub use flip::*;
pub use image_tools::*;
pub use input_map::*;
pub use inspector::*;
pub use model3d::*;
pub use motion::*;
pub use padding::*;
pub use painter::*;
pub use painter_fill::*;
pub use physics::*;
pub use prefab::*;
pub use rail::*;
pub use rail_painter::*;
pub use sculpt3d::*;
pub use timeline::*;
pub use tokens::*;
pub use topbar::*;
pub use vector::*;
pub use vector_bone::*;
pub use wet_tuning::*;
