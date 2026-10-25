//! **Os ids do painel PIXEL** — os que ESTA crate pinta e despacha. O rect do painel e o abridor do
//! menu vivem no `ph2d_editor_core::ids` (o chrome lê-os); a barra de rolagem é o
//! `ph2d_editor_core::widget::PIXEL_SCROLLBAR_ID`.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::PixelTool;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O `×` do cabeçalho.
pub const PIXEL_CLOSE: NodeId = hash_node_id("pixel.close");
/// As amostras de frente e de fundo: o chrome conhece-as (a família lê o seletor por elas).
pub use ph2d_editor_core::ids::{PIXEL_BG, PIXEL_FG};
/// **Trocar** frente e fundo (o `X` do Aseprite).
pub const PIXEL_SWAP: NodeId = hash_node_id("pixel.swap");

/// As quatro ferramentas, na ordem da barra: pintar, registar e despachar leem a MESMA tabela.
pub const TOOLS: [(PixelTool, NodeId); 4] = [
    (PixelTool::Pencil, hash_node_id("pixel.tool.pencil")),
    (PixelTool::Eraser, hash_node_id("pixel.tool.eraser")),
    (PixelTool::Eyedropper, hash_node_id("pixel.tool.eyedropper")),
    (PixelTool::Hand, hash_node_id("pixel.tool.hand")),
];

/// **A célula `i` da paleta** — o clique esquerdo (cor de frente). A população é a paleta, sem tecto.
#[must_use]
pub fn palette_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.palette.{i}"))
}

/// O GÉMEO do botão direito da célula `i` (cor de fundo) — `HitIndex::register_with_secondary`.
#[must_use]
pub fn palette_bg_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.palette_bg.{i}"))
}
