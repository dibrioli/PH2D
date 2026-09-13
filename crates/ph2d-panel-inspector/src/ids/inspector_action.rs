//! **Os ids da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_timer`], de quem esta secção é o gémeo estrutural: lista + um editor.
//!
//! ⚠️ **A POSIÇÃO NO ARRAY DOS VERBOS É A TAG** — o despacho deriva-a de
//! `position(|&o| o == id)` e o modelo lê-a com `SignalVerb::from_tag`. ⛔ Reordenar
//! [`INSP_ACTION_VERB`] faria um clique escrever outro verbo, **e compila**.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector_action.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// `+ Add Action`.
pub const INSP_ACTION_ADD: NodeId = hash_node_id("insp_action_add");

/// `x Remove Action` — apaga a que está aberta.
pub const INSP_ACTION_REMOVE: NodeId = hash_node_id("insp_action_remove");

/// O nome do sinal que dispara esta linha. **Vazio = nunca.**
pub const INSP_ACTION_ON: NodeId = hash_node_id("insp_action_on");

/// O NOME do objecto que sofre a acção. **Vazio = este objecto.**
pub const INSP_ACTION_TARGET: NodeId = hash_node_id("insp_action_target");

/// O parâmetro do verbo — hoje, o nome do timer. **Vazio = todos.**
pub const INSP_ACTION_ARG: NodeId = hash_node_id("insp_action_arg");

/// **O verbo — o CHIP do seletor.** As entradas dele são [`INSP_ACTION_VERB`].
///
/// ⚠️ **Ele é o único id desta família registado como `Dropdown`**: o `open` do popover é o
/// estado dele, e a ESCOLHA nunca vive aqui — ela é do snapshot, relida a cada quadro. *O seed é
/// dono do valor, o dispatch é dono do estado* (a lei que a §12 já paga).
pub const INSP_ACTION_VERB_PICK: NodeId = hash_node_id("insp_action_verb_pick");
