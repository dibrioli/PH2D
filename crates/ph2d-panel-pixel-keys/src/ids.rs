//! **Os ids da janela KEYBOARD SHORTCUTS.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_pixel_keys::{Chord, PixelAction, Section};
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

pub const DRAG: NodeId = hash_node_id("pixel_keys.drag");
pub const EDGE: NodeId = hash_node_id("pixel_keys.edge");
pub const SEC_TOP: NodeId = hash_node_id("pixel_keys.sec.top");
pub const SEARCH: NodeId = hash_node_id("pixel_keys.search");
pub const CONFIRM: NodeId = hash_node_id("pixel_keys.confirm");
pub const CANCEL: NodeId = hash_node_id("pixel_keys.cancel");
pub const REPLACE: NodeId = hash_node_id("pixel_keys.replace");
pub const RESET_ALL: NodeId = hash_node_id("pixel_keys.reset_all");
pub const IMPORT: NodeId = hash_node_id("pixel_keys.import");
pub const EXPORT: NodeId = hash_node_id("pixel_keys.export");
pub const CLOSE: NodeId = hash_node_id("pixel_keys.close");

/// O `CLOSE` saiu da janela (P12: fecha-se pelo menu, como a SLICES); o id fica (append-only).
pub const BOTOES: [NodeId; 7] = [CONFIRM, CANCEL, REPLACE, RESET_ALL, IMPORT, EXPORT, DRAG];

#[must_use]
pub fn secao(s: Section) -> NodeId {
    hash_node_id_runtime(&format!("pixel_keys.sec.{}", s.name_key()))
}

#[must_use]
/// O campo vazio da acção (a tecla nova; era o `+`).
pub fn add(a: PixelAction) -> NodeId {
    hash_node_id_runtime(&format!("pixel_keys.add.{}", a.id()))
}

#[must_use]
pub fn reset(a: PixelAction) -> NodeId {
    hash_node_id_runtime(&format!("pixel_keys.reset.{}", a.id()))
}

/// O campo de uma tecla da acção (o clique captura a que a substitui).
#[must_use]
pub fn ficha(a: PixelAction, k: Chord) -> NodeId {
    hash_node_id_runtime(&format!("pixel_keys.chip.{}.{k}", a.id()))
}

/// O nome da acção na fileira (o clique abre o campo vazio em captura).
#[must_use]
pub fn nome(a: PixelAction) -> NodeId {
    hash_node_id_runtime(&format!("pixel_keys.name.{}", a.id()))
}
