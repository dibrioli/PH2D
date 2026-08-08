//! **Os ids dos ESTADOS de UI** (plano UI/UX W7) — irmão do [`super::vector_widget`] pelo teto de
//! LOC.
//!
//! O corte é por ASSUNTO: aqui mora *que poses esta forma tem, e como ela transita entre elas*.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;
use super::painter::fnv_node_id_runtime;

/// O cabeçalho da seção **States**.
pub const VECTOR_SECTION_STATES: NodeId = hash_node_id("vector.section.states");

/// Quantos papéis a tabela de ids endereça.
///
/// ⚠️ **Não é um teto que se escolhe: é a CONTAGEM de `StateRole::ALL`**, e o gate a compara com
/// o enum. Um papel além daqui seria pintado e **inalcançável** — a mesma armadilha do
/// `MAX_WIDGET_KINDS`, que por isso exige `>=` em vez de apenas *"os chips existem"*.
pub const MAX_STATE_ROLES: usize = 4;

/// **Record / Update** o papel `i` — grava a pose atual.
///
/// ⚠️ Derivado do ÍNDICE de runtime, nunca do que viaja no documento: este id vive um frame, o
/// mesmo racional do `vector_widget_kind_id`.
#[must_use]
pub fn vector_state_record_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.state.record.{i}"))
}

/// **Clear** o papel `i` — só é pintado onde há o que apagar.
#[must_use]
pub fn vector_state_clear_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.state.clear.{i}"))
}

/// **Apply** o papel `i` — põe a cena nessa pose, para o artista a EDITAR.
///
/// ⚠️ Ele é o que torna a gravação re-editável: sem ele o artista teria de reconstruir a pose de
/// cabeça para regravá-la, e um estado autorado uma vez seria um estado autorado para sempre.
#[must_use]
pub fn vector_state_apply_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.state.apply.{i}"))
}

/// A duração da transição, em segundos — o slider e o chip que o espelha.
///
/// ⚠️ **A CURVA não tem id aqui, e a ausência é decisão medida:** o catálogo de easing são 11
/// famílias × 3 modos e a crate que o possui (`ph2d-anim`) não dá nome a nenhuma combinação. Um
/// dropdown hoje pintaria identificador inglês cru, que é o que o HR-15 proíbe; o knob nasce
/// quando as curvas ganharem nomes, no lugar onde elas moram.
pub const VECTOR_STATE_DURATION: NodeId = hash_node_id("vector.state.duration");
pub const VECTOR_STATE_DURATION_NUM: NodeId = hash_node_id("vector.state.duration.num");

/// **O MODO DE PREVIEW** (W7r) — o interruptor que faz a UI desenhada responder ao rato.
///
/// ⚠️ **Um id só, e não um por papel:** o modo não escolhe um papel, ele entrega os papéis ao
/// rato. Um chip por papel seria a segunda forma de pedir o que o botão *Show* já pede, e as duas
/// discordariam no dia em que uma delas ganhasse um caso especial.
pub const VECTOR_STATE_PREVIEW: NodeId = hash_node_id("vector.state.preview");

/// **Mover o widget carregando TODOS os estados** (Enio, 2026-08-07).
///
/// ⚠️ Marcado, relocar o hospedeiro desloca a pose dele em cada estado gravado — o widget muda de
/// lugar no canvas e continua **perfeitamente animado**. Desmarcado, mover re-autora só a pose
/// atual, que é o que se quer quando a intenção é corrigir UM estado.
pub const VECTOR_STATE_MOVE_ALL: NodeId = hash_node_id("vector.state.move.all");
