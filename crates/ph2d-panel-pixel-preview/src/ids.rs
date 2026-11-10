//! **Os ids da Preview do Pixel** — os que ESTA crate pinta e despacha. O rect do painel é o
//! `ph2d_editor_core::ids::PIXEL_PREVIEW_PANEL`.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O `×` do título.
pub const PREVIEW_CLOSE: NodeId = hash_node_id("pixel_preview.close");
/// Tocar / parar a Preview (o ícone vira *Stop* enquanto toca).
pub const PLAY: NodeId = hash_node_id("pixel_preview.play");
/// A alça de arrasto (a banda do título).
pub const DRAG: NodeId = hash_node_id("pixel_preview.drag");
/// A pega de redimensionar do canto de baixo à direita.
pub const RESIZE: NodeId = hash_node_id("pixel_preview.resize");
/// A pega de redimensionar do canto de baixo à esquerda.
pub const RESIZE_BL: NodeId = hash_node_id("pixel_preview.resize_bl");
