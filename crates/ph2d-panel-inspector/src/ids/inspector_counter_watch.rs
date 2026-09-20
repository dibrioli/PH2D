//! **Os ids da secção COUNTER WATCH** — a vigia do contador.
//!
//! # ⚠️ A lista + UM editor, e não seis controlos por linha
//!
//! Um `CounterWatch` guarda até [`ph2d_ecs::WATCHES_MAX`] regras e cada uma tem **cinco** campos.
//! Desenhar os cinco em cada linha custaria `5 × 16 = 80` ids e uma coluna que não cabe na largura
//! do Inspector. ⇒ o molde é o do `Timers`: a **lista** escolhe qual regra está aberta, e um editor
//! só, abaixo dela, mostra os campos dessa.
//!
//! ⚠️ **A escolha da linha NÃO vai ao barramento** — qual regra se edita é um facto da UI e vive no
//! `InspectorState`, como no `Timers`. Um `CounterWatch` não tem *«a regra actual»*.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// `+ Add Rule`.
pub const INSP_WATCH_ADD: NodeId = hash_node_id("insp_watch_add");

/// `x Remove Rule` — apaga a que está aberta.
pub const INSP_WATCH_REMOVE: NodeId = hash_node_id("insp_watch_remove");

/// O **nome do contador** que a regra vigia. ⚠️ Não é o nome desta regra nem o do sinal.
pub const INSP_WATCH_COUNTER: NodeId = hash_node_id("insp_watch_counter");

/// O limiar.
pub const INSP_WATCH_VALUE: NodeId = hash_node_id("insp_watch_value");

/// O nome do sinal publicado na travessia. **Vazio = calada.**
pub const INSP_WATCH_SIGNAL: NodeId = hash_node_id("insp_watch_signal");

/// Só da primeira vez.
pub const INSP_WATCH_ONCE: NodeId = hash_node_id("insp_watch_once");

/// A caixa **Only this object** — a vida de UM inimigo em vez do placar da cena.
pub const INSP_WATCH_SCOPE: NodeId = hash_node_id("insp_watch_scope");

/// **O CHIP da comparação** — o único `Dropdown` desta secção no store.
///
/// ⚠️ **As opções são BOTÕES** ([`INSP_WATCH_CMP_OPT`]), e o chip guarda só o `open`: é a máquina
/// que o selector da cutscene já usa, e o rect do popover viaja por uma ranhura até ao passe
/// diferido, para ele se pintar **por cima** de tudo.
pub const INSP_WATCH_CMP_PICK: NodeId = hash_node_id("insp_watch_cmp_pick");

/// As três opções da comparação — `<=`, `>=`, `==`.
///
/// ⚠️ **O comprimento deste array É o número de variantes do `ph2d_ecs::Compare`**, e há gate na
/// shell a prendê-los (a única crate que vê o painel e o motor). *Uma quarta comparação no motor
/// sem uma quarta entrada aqui é uma comparação que o artista nunca escolhe.*
pub const INSP_WATCH_CMP_OPT: [NodeId; 3] = [
    hash_node_id("insp_watch_cmp_opt_0"),
    hash_node_id("insp_watch_cmp_opt_1"),
    hash_node_id("insp_watch_cmp_opt_2"),
];

/// **As linhas da lista** — uma por regra, até ao cap de [`ph2d_ecs::WATCHES_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate na shell a prendê-los: *um modelo
/// que aceita o que o painel não mostra produz estado inalcançável por gesto nenhum.*
pub const INSP_WATCH_ROW: [NodeId; 16] = [
    hash_node_id("insp_watch_row_00"),
    hash_node_id("insp_watch_row_01"),
    hash_node_id("insp_watch_row_02"),
    hash_node_id("insp_watch_row_03"),
    hash_node_id("insp_watch_row_04"),
    hash_node_id("insp_watch_row_05"),
    hash_node_id("insp_watch_row_06"),
    hash_node_id("insp_watch_row_07"),
    hash_node_id("insp_watch_row_08"),
    hash_node_id("insp_watch_row_09"),
    hash_node_id("insp_watch_row_10"),
    hash_node_id("insp_watch_row_11"),
    hash_node_id("insp_watch_row_12"),
    hash_node_id("insp_watch_row_13"),
    hash_node_id("insp_watch_row_14"),
    hash_node_id("insp_watch_row_15"),
];
