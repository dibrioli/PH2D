//! Widget `NodeId`s for the Physics world panel.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui; os que a `ph2d-editor-core` também lê (o corpo do painel, que o *z-order walk*
//! percorre) ficaram lá, e quem os usa nomeia `ph2d_editor_core::ids::X` — nunca por este módulo.
//! Ele re-exportava a fundação «para não bifurcar a fonte da verdade»: com uma definição só não há o
//! que bifurcar, e o censo de colisões (`node_id_collisions`) lê os literais da workspace inteira,
//! não uma lista na fundação.

// ⚠️ **`PHYSICS_SLEEP_SPIN_NUM` saiu da lista de re-exportação que este ficheiro tinha, em
// 2026-08-30,** e a ausência é a nota: ele era o
// chip numérico do slider *Spin*, e o slider virou um interruptor (`paint::body`), que não tem
// chip. ✅ O remate que esta nota pedia aconteceu em 2026-09-12: o const órfão saiu da
// `ph2d-editor-core`, e a tabela à mão do `node_id_collisions` que o mantinha «lido» morreu com o
// censo derivado.

mod physics;
pub use physics::*;
