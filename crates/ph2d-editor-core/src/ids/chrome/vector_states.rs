//! **Os ids dos ESTADOS de UI** (plano UI/UX W7) — irmão do [`super::vector_widget`] pelo teto de
//! LOC.
//!
//! O corte é por ASSUNTO: aqui mora *que poses esta forma tem, e como ela transita entre elas*.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

/// A duração da transição, em segundos — o slider e o chip que o espelha.
pub const VECTOR_STATE_DURATION: NodeId = hash_node_id("vector.state.duration");

/// **O MODO DE PREVIEW** (W7r) — o interruptor que faz a UI desenhada responder ao rato.
///
/// ⚠️ **Um id só, e não um por papel:** o modo não escolhe um papel, ele entrega os papéis ao
/// rato. Um chip por papel seria a segunda forma de pedir o que o botão *Show* já pede, e as duas
/// discordariam no dia em que uma delas ganhasse um caso especial.
/// **A MOLA** — o checkbox que troca *duração + curva* por *rigidez + amortecimento*.
///
/// ⚠️ **Ela TROCA as linhas, não as soma.** Rigidez e amortecimento respondem a mesma pergunta que
/// duração e curva (*quanto tempo, e com que forma*), e oferecer as quatro pediria ao artista que
/// mantivesse dois modelos de acordo. Só uma família vive de cada vez — o par
/// `Width: Auto | Fixed` do texto, e o `Mass: Auto | Manual` do editor de áudio.
pub const VECTOR_STATE_SPRING: NodeId = hash_node_id("vector.state.spring");
pub const VECTOR_STATE_STIFFNESS: NodeId = hash_node_id("vector.state.stiffness");

pub const VECTOR_STATE_DAMPING: NodeId = hash_node_id("vector.state.damping");

pub const VECTOR_STATE_PREVIEW: NodeId = hash_node_id("vector.state.preview");

/// **Mover o widget carregando TODOS os estados** (Enio, 2026-08-07).
///
/// ⚠️ Marcado, relocar o hospedeiro desloca a pose dele em cada estado gravado — o widget muda de
/// lugar no canvas e continua **perfeitamente animado**. Desmarcado, mover re-autora só a pose
/// atual, que é o que se quer quando a intenção é corrigir UM estado.
pub const VECTOR_STATE_MOVE_ALL: NodeId = hash_node_id("vector.state.move.all");
