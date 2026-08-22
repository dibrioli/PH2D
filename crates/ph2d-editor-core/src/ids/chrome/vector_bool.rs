//! **Ids das operações BOOLEANAS do vetor** — as oito do Pathfinder e o modo VIVO delas.
//!
//! Irmão de `vector.rs` pelo teto de LOC (HR-18), cortado por ASSUNTO: aqui vive *o que COMBINA
//! formas*, e lá o que descreve UMA forma. O `pub use` no `mod.rs` mantém todo caminho de
//! chamador intacto.

use crate::ids::hash_node_id;
use ph2d_a11y::NodeId;

// ── Boolean ops (ADR-0108 Fase 1 — edit-time union/subtract/intersect) ───────
// Act on the DOCUMENT (shell-owned `vec_scene`), NOT the tool's Style: the
// panel forwards a `Click` over `ToolPanelEvent` and the shell drain applies
// the op to the two last closed regions (mirror of the U/I/D hotkeys).
pub const VECTOR_BOOL_UNION: NodeId = hash_node_id("vector.bool.union");
pub const VECTOR_BOOL_SUBTRACT: NodeId = hash_node_id("vector.bool.subtract");
pub const VECTOR_BOOL_INTERSECT: NodeId = hash_node_id("vector.bool.intersect");
pub const VECTOR_BOOL_EXCLUDE: NodeId = hash_node_id("vector.bool.exclude");

// ── A BOOLEANA VIVA (plano UI/UX W1) ─────────────────────────────────────────
// O par decide **o que os oito botões acima FAZEM**: `Off` consome os operandos (o mundo de
// sempre), `On` cria um GRUPO cujos filhos se combinam e continuam editáveis. Não é um nono
// botão — é o modo dos oito, e por isso vive acima deles.
pub const VECTOR_BOOL_LIVE_OFF: NodeId = hash_node_id("vector.bool.live.off");
/// **Booleana viva — ligada.**
pub const VECTOR_BOOL_LIVE_ON: NodeId = hash_node_id("vector.bool.live.on");
/// **Consolidar** a booleana viva selecionada: o que está na tela vira caminhos comuns e o grupo
/// morre. Oferecido só com um grupo booleano selecionado — um *Apply* que não aplica nada é pior
/// que *Apply* nenhum.
pub const VECTOR_BOOL_APPLY: NodeId = hash_node_id("vector.bool.apply");

// ── O DIAGRAMA da booleana viva (etapa 2) ────────────────────────────────────
// O card onde a operação passa a ser da LIGAÇÃO. ⚠️ Só três ids, e é de propósito: os círculos e
// os arcos NÃO são widgets — eles são geometria com acerto próprio (`widget::bool_graph`), e dar
// um `NodeId` a cada um significaria um registo por forma por frame, com o mapa a mudar de tamanho
// a cada ligação criada. O que se regista é o card: a banda de título, o X, e o CORPO (que existe
// para o clique parar ali em vez de atravessar para a arte por baixo).

/// **Abrir o diagrama** — o botão da seção Boolean. Oferecido só com um grupo booleano vivo
/// selecionado, pela mesma razão do *Apply*: um botão que abre um diagrama vazio é pior que
/// botão nenhum.
pub const VECTOR_BOOL_GRAPH_OPEN: NodeId = hash_node_id("vector.bool.graph.open");
/// A banda de título do card = a alça de arrasto. Ela para antes do X, para os dois retângulos de
/// acerto nunca partilharem um pixel (um *Down* no X fecha; um *Down* na banda arrasta).
pub const VECTOR_BOOL_GRAPH_HANDLE: NodeId = hash_node_id("vector.bool.graph.handle");
/// O X que fecha o card.
pub const VECTOR_BOOL_GRAPH_CLOSE: NodeId = hash_node_id("vector.bool.graph.close");
/// O CORPO do card — o retângulo que engole o clique.
///
/// ⚠️ Sem ele o ponteiro atravessaria o diagrama e chegaria à arte por baixo: arrastar de um
/// círculo a outro moveria as FORMAS, que é o oposto exato do gesto.
pub const VECTOR_BOOL_GRAPH_BODY: NodeId = hash_node_id("vector.bool.graph.body");
