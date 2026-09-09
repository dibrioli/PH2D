//! **Os ids da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_timer`], de quem esta secção é o gémeo estrutural: lista + um editor.
//!
//! ⚠️ **A POSIÇÃO NO ARRAY DOS VERBOS É A TAG** — o despacho deriva-a de
//! `position(|&o| o == id)` e o modelo lê-a com `SignalVerb::from_tag`. ⛔ Reordenar
//! [`INSP_ACTION_VERB`] faria um clique escrever outro verbo, **e compila**.

use super::*;

/// A secção SIGNAL ACTIONS — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`].
pub const INSP_LIVE_ACTION_SECTION: NodeId = hash_node_id("insp_live_action_section");
/// SIGNAL ACTIONS — ponto de cor do cabeçalho.
pub const INSP_LIVE_ACTION_COLOR: NodeId = hash_node_id("insp_live_action_color");

/// **As linhas da lista** — uma por acção, até ao cap de [`ph2d_ecs::SIGNAL_ACTIONS_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, com gate na shell: *um modelo que aceita o
/// que o painel não mostra produz estado inalcançável*.
pub const INSP_ACTION_ROW: [NodeId; 16] = [
    hash_node_id("insp_action_row_00"),
    hash_node_id("insp_action_row_01"),
    hash_node_id("insp_action_row_02"),
    hash_node_id("insp_action_row_03"),
    hash_node_id("insp_action_row_04"),
    hash_node_id("insp_action_row_05"),
    hash_node_id("insp_action_row_06"),
    hash_node_id("insp_action_row_07"),
    hash_node_id("insp_action_row_08"),
    hash_node_id("insp_action_row_09"),
    hash_node_id("insp_action_row_10"),
    hash_node_id("insp_action_row_11"),
    hash_node_id("insp_action_row_12"),
    hash_node_id("insp_action_row_13"),
    hash_node_id("insp_action_row_14"),
    hash_node_id("insp_action_row_15"),
];

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

/// **O verbo**, um botão por entrada de `SignalVerb::ALL`.
///
/// ⚠️ **A posição é a tag** — ver o doc do módulo.
pub const INSP_ACTION_VERB: [NodeId; 5] = [
    hash_node_id("insp_action_verb_start"),
    hash_node_id("insp_action_verb_stop"),
    hash_node_id("insp_action_verb_show"),
    hash_node_id("insp_action_verb_hide"),
    hash_node_id("insp_action_verb_toggle"),
];
