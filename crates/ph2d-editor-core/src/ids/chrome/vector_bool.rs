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

// ── O VERBO DE UMA FORMA dentro da booleana viva (2026-08-22) ────────────────
// Os quatro acima agem sobre a SELEÇÃO e criam ou re-miram o grupo inteiro; estes quatro são
// **propriedade de UMA forma** — o verbo com que ela dobra sobre o resultado das anteriores. É o
// compound shape vivo do Illustrator, em que cada componente guarda o seu Shape Mode.
//
// ⚠️ **Ids próprios, e não os quatro de cima reaproveitados.** Os dois conjuntos convivem na
// mesma seção e fazem coisas diferentes sobre a mesma seleção; partilhar o id faria um clique
// significar as duas, e qual delas venceria dependeria da ordem do dispatch.
//
// ⛔ São QUATRO, não oito: `MinusBack`/`Trim`/`Crop`/`Merge` são afirmações sobre a PILHA INTEIRA
// (*"cada forma menos a união do que está acima dela"*), e não cabem numa forma só.
pub const VECTOR_BOOL_SHAPE_UNION: NodeId = hash_node_id("vector.bool.shape.union");
/// **Subtract** como verbo desta forma.
pub const VECTOR_BOOL_SHAPE_SUBTRACT: NodeId = hash_node_id("vector.bool.shape.subtract");
/// **Intersect** como verbo desta forma.
pub const VECTOR_BOOL_SHAPE_INTERSECT: NodeId = hash_node_id("vector.bool.shape.intersect");
/// **Exclude** como verbo desta forma.
pub const VECTOR_BOOL_SHAPE_EXCLUDE: NodeId = hash_node_id("vector.bool.shape.exclude");
