//! Os `NodeId` de widget que esta crate LÊ — declarados AQUI, e não na `ph2d-editor-core`.
//!
//! ⭐ **Desceram da fundação em 2026-09-12** (auditoria de arquitectura A5b): em 30 dias, 221 dos 437
//! commits à `ph2d-editor-core/src` tocaram `ids/`, e cada um recompilava as 43 crates que dependem
//! dela. O dono de um id é a crate MAIS BAIXA que todo leitor dele vê (`scripts/censo-ids.py`), e
//! um ficheiro de ids — um ASSUNTO — desce inteiro.
//!
//! ⚠️ **UMA definição e ZERO re-exportações**: quem lê de fora nomeia `ph2d_tool_flip::ids::X`. As colisões
//! de slug vigia-as o censo DERIVADO (`ph2d-editor-core/tests/it/node_id_collisions.rs`), que lê os
//! literais da workspace inteira — o id não precisa de morar ao lado dele.

mod flip;
pub use flip::*;
