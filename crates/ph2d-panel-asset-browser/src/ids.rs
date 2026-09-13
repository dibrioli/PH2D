//! Os `NodeId` de widget do navegador de assets.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui; os que a `ph2d-editor-core` também lê (o corpo do painel, que o *z-order walk*
//! percorre, e o que o despacho compara) ficaram lá, e quem os usa nomeia `ph2d_editor_core::ids::X`
//! — nunca por este módulo. Ele re-exportava a fundação dizendo que tê-los aqui criaria um ciclo: com
//! o id na crate que o lê não há ciclo nenhum, e o censo de colisões (`node_id_collisions`) lê os
//! literais da workspace inteira, não uma lista na fundação.

mod asset_browser;
pub use asset_browser::*;
