//! **Os ids da secção GATILHO** — a mão de quem joga (suplente #24).
//!
//! # ⚠️ A lista + UM editor, como a vigia
//!
//! Um `SignalOnAction` guarda até [`ph2d_ecs::ACTION_TRIGGERS_MAX`] linhas e cada uma tem **três**
//! campos. São menos que os cinco da vigia — e ainda assim não cabem numa linha da coluna do
//! Inspector, que o dono corre a `220,9` px de largura (medido no `~/.ph2d/layout.txt` dele, e é a
//! largura que o gate `every_label_this_panel_paints_fits_its_column` julga). ⇒ o molde é o mesmo:
//! a **lista** escolhe qual linha está aberta, e um editor só, abaixo dela, mostra os campos dessa.
//!
//! ⚠️ **A escolha da linha NÃO vai ao barramento** — qual linha se edita é um facto da UI e vive no
//! `InspectorState`, como no `Timers` e na vigia.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// `+ Add Trigger`.
pub const INSP_TRIGGER_ADD: NodeId = hash_node_id("insp_trigger_add");

/// `x Remove Trigger` — apaga a que está aberta.
pub const INSP_TRIGGER_REMOVE: NodeId = hash_node_id("insp_trigger_remove");

/// O **nome da acção** do Input Map que a linha ouve. ⚠️ Não é o nome da tecla: é o nome que o
/// artista escreveu em *Settings → Input Map…*, e é ele que sobrevive a um remapeamento.
pub const INSP_TRIGGER_ACTION: NodeId = hash_node_id("insp_trigger_action");

/// O nome do sinal publicado. **Vazio = calada.**
pub const INSP_TRIGGER_SIGNAL: NodeId = hash_node_id("insp_trigger_signal");

/// **O CHIP da aresta** — o único `Dropdown` desta secção no store.
///
/// ⚠️ A máquina é a do chip da vigia: as opções são BOTÕES ([`INSP_TRIGGER_EDGE_OPT`]) e o chip
/// guarda só o `open`; o rect do popover viaja por uma ranhura até ao passe diferido, para ele se
/// pintar **por cima** de tudo.
/// ⭐⭐⭐ **CRIAR a acção que falta**, ali mesmo. ⚠️ Ele só é PINTADO quando a fileira já diz que a
/// acção não existe — *um botão que oferece criar uma acção que já existe é ruído*, e o gate
/// afirma as duas metades.
pub const INSP_TRIGGER_CREATE_ACTION: NodeId = hash_node_id("insp_trigger_create_action");

pub const INSP_TRIGGER_EDGE_PICK: NodeId = hash_node_id("insp_trigger_edge_pick");

/// As três opções da aresta — premir, largar, segurar.
///
/// ⚠️ **O comprimento deste array É o número de variantes do `ph2d_ecs::ActionEdge`**, e há gate na
/// shell a prendê-los (a única crate que vê o painel e o motor). *Uma quarta aresta no motor sem
/// uma quarta entrada aqui é uma aresta que o artista nunca escolhe.*
pub const INSP_TRIGGER_EDGE_OPT: [NodeId; 3] = [
    hash_node_id("insp_trigger_edge_opt_0"),
    hash_node_id("insp_trigger_edge_opt_1"),
    hash_node_id("insp_trigger_edge_opt_2"),
];

/// **As linhas da lista** — uma por gatilho, até ao cap de [`ph2d_ecs::ACTION_TRIGGERS_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate na shell a prendê-los: *um modelo
/// que aceita o que o painel não mostra produz estado inalcançável por gesto nenhum.*
pub const INSP_TRIGGER_ROW: [NodeId; 16] = [
    hash_node_id("insp_trigger_row_00"),
    hash_node_id("insp_trigger_row_01"),
    hash_node_id("insp_trigger_row_02"),
    hash_node_id("insp_trigger_row_03"),
    hash_node_id("insp_trigger_row_04"),
    hash_node_id("insp_trigger_row_05"),
    hash_node_id("insp_trigger_row_06"),
    hash_node_id("insp_trigger_row_07"),
    hash_node_id("insp_trigger_row_08"),
    hash_node_id("insp_trigger_row_09"),
    hash_node_id("insp_trigger_row_10"),
    hash_node_id("insp_trigger_row_11"),
    hash_node_id("insp_trigger_row_12"),
    hash_node_id("insp_trigger_row_13"),
    hash_node_id("insp_trigger_row_14"),
    hash_node_id("insp_trigger_row_15"),
];
