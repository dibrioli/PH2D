//! **Ids das operações BOOLEANAS do vetor** — as oito do Pathfinder e o modo VIVO delas.
//!
//! Irmão de `vector.rs` pelo teto de LOC (HR-18), cortado por ASSUNTO: aqui vive *o que COMBINA
//! formas*, e lá o que descreve UMA forma. O `pub use` no `mod.rs` mantém todo caminho de
//! chamador intacto.

use crate::ids::hash_node_id;
use ph2d_a11y::NodeId;

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
