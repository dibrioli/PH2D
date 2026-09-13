//! Os `NodeId`s da tira.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui; os que a `ph2d-editor-core` também lê (o corpo do painel, que o *z-order walk*
//! percorre) ficaram lá, e quem os usa nomeia `ph2d_editor_core::ids::X` — nunca por este módulo. O
//! teste de colisão já não os precisa na fundação: ele lê os literais da workspace inteira.

mod flip;
pub use flip::*;
