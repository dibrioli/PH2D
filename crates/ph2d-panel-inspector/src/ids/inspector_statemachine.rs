//! **Os ids da secção STATE MACHINE** (TOP-20 #15, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_action`], de quem esta secção é o **gémeo estrutural**: DUAS listas, cada uma
//! com `+ Add` / `x Remove` e **um editor para a linha aberta**.
//!
//! ⚠️⚠️ **Um editor para a linha ABERTA, e não N linhas de campos** — é o idioma que o
//! `SignalActions` e o `Timers` já usam, e a razão é o dock: 16 estados × 3 campos seriam 48
//! controlos numa coluna que mostra ~30 linhas. *Um modelo que aceita o que o painel não mostra
//! produz estado inalcançável* — e aqui o painel mostra **tudo**, uma linha de cada vez.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// As linhas da lista de ESTADOS — clicar numa abre-a no editor.
///
/// ⚠️ **O tecto é o `STATES_MAX` da lei** (16), e os dois números são o MESMO facto: um estado que
/// o modelo aceita e a lista não mostra é inalcançável.
pub const INSP_SM_STATE_ROW: [NodeId; 16] = [
    hash_node_id("insp_sm_state_row0"),
    hash_node_id("insp_sm_state_row1"),
    hash_node_id("insp_sm_state_row2"),
    hash_node_id("insp_sm_state_row3"),
    hash_node_id("insp_sm_state_row4"),
    hash_node_id("insp_sm_state_row5"),
    hash_node_id("insp_sm_state_row6"),
    hash_node_id("insp_sm_state_row7"),
    hash_node_id("insp_sm_state_row8"),
    hash_node_id("insp_sm_state_row9"),
    hash_node_id("insp_sm_state_row10"),
    hash_node_id("insp_sm_state_row11"),
    hash_node_id("insp_sm_state_row12"),
    hash_node_id("insp_sm_state_row13"),
    hash_node_id("insp_sm_state_row14"),
    hash_node_id("insp_sm_state_row15"),
];

/// `+ Add State`.
pub const INSP_SM_STATE_ADD: NodeId = hash_node_id("insp_sm_state_add");
/// `x Remove State` — apaga o que está aberto.
pub const INSP_SM_STATE_REMOVE: NodeId = hash_node_id("insp_sm_state_remove");
/// O nome do estado aberto.
pub const INSP_SM_STATE_NAME: NodeId = hash_node_id("insp_sm_state_name");
/// O sinal publicado ao ENTRAR. **Vazio = calado.**
pub const INSP_SM_STATE_ON_ENTER: NodeId = hash_node_id("insp_sm_state_on_enter");
/// O sinal publicado ao SAIR. **Vazio = calado.**
pub const INSP_SM_STATE_ON_EXIT: NodeId = hash_node_id("insp_sm_state_on_exit");

/// As linhas da lista de TRANSIÇÕES — o tecto é o `TRANSITIONS_MAX` da lei (32).
pub const INSP_SM_TRANS_ROW: [NodeId; 32] = [
    hash_node_id("insp_sm_trans_row0"),
    hash_node_id("insp_sm_trans_row1"),
    hash_node_id("insp_sm_trans_row2"),
    hash_node_id("insp_sm_trans_row3"),
    hash_node_id("insp_sm_trans_row4"),
    hash_node_id("insp_sm_trans_row5"),
    hash_node_id("insp_sm_trans_row6"),
    hash_node_id("insp_sm_trans_row7"),
    hash_node_id("insp_sm_trans_row8"),
    hash_node_id("insp_sm_trans_row9"),
    hash_node_id("insp_sm_trans_row10"),
    hash_node_id("insp_sm_trans_row11"),
    hash_node_id("insp_sm_trans_row12"),
    hash_node_id("insp_sm_trans_row13"),
    hash_node_id("insp_sm_trans_row14"),
    hash_node_id("insp_sm_trans_row15"),
    hash_node_id("insp_sm_trans_row16"),
    hash_node_id("insp_sm_trans_row17"),
    hash_node_id("insp_sm_trans_row18"),
    hash_node_id("insp_sm_trans_row19"),
    hash_node_id("insp_sm_trans_row20"),
    hash_node_id("insp_sm_trans_row21"),
    hash_node_id("insp_sm_trans_row22"),
    hash_node_id("insp_sm_trans_row23"),
    hash_node_id("insp_sm_trans_row24"),
    hash_node_id("insp_sm_trans_row25"),
    hash_node_id("insp_sm_trans_row26"),
    hash_node_id("insp_sm_trans_row27"),
    hash_node_id("insp_sm_trans_row28"),
    hash_node_id("insp_sm_trans_row29"),
    hash_node_id("insp_sm_trans_row30"),
    hash_node_id("insp_sm_trans_row31"),
];

/// `+ Add Transition`.
pub const INSP_SM_TRANS_ADD: NodeId = hash_node_id("insp_sm_trans_add");
/// `x Remove Transition`.
pub const INSP_SM_TRANS_REMOVE: NodeId = hash_node_id("insp_sm_trans_remove");
/// O índice do estado de PARTIDA da seta aberta.
pub const INSP_SM_TRANS_FROM: NodeId = hash_node_id("insp_sm_trans_from");
/// O NOME do sinal que dispara a seta aberta. **Vazio = nunca.**
pub const INSP_SM_TRANS_ON: NodeId = hash_node_id("insp_sm_trans_on");
/// O índice do estado de CHEGADA.
pub const INSP_SM_TRANS_TO: NodeId = hash_node_id("insp_sm_trans_to");

/// Em que estado a cena abre.
pub const INSP_SM_INITIAL: NodeId = hash_node_id("insp_sm_initial");
