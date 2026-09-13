//! Widget `NodeId`s for the Padding panel.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui; os que a `ph2d-editor-core` também lê (o corpo do painel, que o *z-order walk*
//! percorre) ficaram lá, e quem os usa nomeia `ph2d_editor_core::ids::X` — nunca por este módulo.
//! Ele re-exportava a fundação «para não bifurcar a fonte da verdade»: com uma definição só não há o
//! que bifurcar, e o censo de colisões (`node_id_collisions`) lê os literais da workspace inteira,
//! não uma lista na fundação.

mod padding;
pub use padding::*;
