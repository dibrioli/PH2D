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

/// Quantas FAMÍLIAS de easing a tabela de ids endereça.
///
/// ⚠️ **Não é um teto que se escolhe: é a CONTAGEM de `EasingFamily::ALL`** — e o gate que os
/// compara **não pode morar aqui**, porque editor-core não depende de `ph2d-anim` (a mesma
/// fronteira que faz o menu da timeline nomear as curvas com literais próprios). Ele mora no
/// painel, que vê os dois lados. Uma família além daqui seria pintada e **inalcançável**, que é
/// a armadilha que o `MAX_STATE_ROLES` já documenta.
pub const MAX_EASING_FAMILIES: usize = 11;

/// Quantos MODOS de easing a tabela de ids endereça — a contagem de `EasingMode::ALL`.
pub const MAX_EASING_MODES: usize = 3;

/// A FAMÍLIA de easing `i` — o chip que escolhe a forma da curva da transição.
///
/// ⚠️ Derivado do ÍNDICE em `EasingFamily::ALL`, e não do nome: o id vive um frame (o mesmo
/// racional do `vector_state_record_id`), e indexar pelo rótulo faria uma renomeação de vocabulário
/// mover a chave de registo de um widget.
#[must_use]
pub fn vector_easing_family_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.state.ease.family.{i}"))
}

/// O MODO de easing `i` (In / Out / In-Out).
///
/// ⚠️ A fileira dele **não é pintada para toda família**: `Linear` ignora o modo (o `eval` devolve
/// `u` antes de o olhar), então oferecê-lo ali daria três chips que desenham a mesma curva. Quem
/// responde *"esta família usa o modo?"* é o próprio enum (`EasingFamily::uses_mode`), medido por
/// gate — nunca uma lista de exceções neste ficheiro.
#[must_use]
pub fn vector_easing_mode_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.state.ease.mode.{i}"))
}
