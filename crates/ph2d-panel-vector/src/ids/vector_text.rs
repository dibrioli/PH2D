//! Os ids da seção **TEXT** do painel do vetor — irmão de `vector` pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, e ele é o mesmo que os outros `vector_*` já fazem: aqui vive
//! *o que um texto DIZ e como ele se dispõe* (tamanho, peso, fonte, alinhamento, entrelinha,
//! tracking, o refluxo) — e no pai fica *o que uma forma É*. `VECTOR_MODE_TEXT` **não** vem
//! junto de propósito: ele é um MODO, e mora com os outros três.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_text.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// Value chip paired with [`VECTOR_TEXT_SIZE`].
pub const VECTOR_TEXT_SIZE_NUM: NodeId = hash_node_id("vector.text.size_num");

/// Value chip paired with [`VECTOR_TEXT_WEIGHT`].
pub const VECTOR_TEXT_WEIGHT_NUM: NodeId = hash_node_id("vector.text.weight_num");

pub const VECTOR_TEXT_LINE_HEIGHT_NUM: NodeId = hash_node_id("vector.text.line_height_num");

pub const VECTOR_TEXT_TRACKING_NUM: NodeId = hash_node_id("vector.text.tracking_num");

pub const VECTOR_TEXT_WRAP_W_NUM: NodeId = hash_node_id("vector.text.wrap_w_num");

/// Stable [`NodeId`] for the `index`-th family row in the open font dropdown
/// (index into the shell's pickable list `[bundled] ++ imported ++ system`). Runtime
/// `format!` (the family count is only known at runtime); the FNV twin keeps it in
/// the same id space as the `hash_node_id` consts. Mirrors the Painter option-id
/// fatories (`painter_brush_*_option_id`).
#[must_use]
pub fn vector_text_font_option_id(index: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.text.fontopt.{index}"))
}
