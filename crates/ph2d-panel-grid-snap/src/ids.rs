//! Os `NodeId` do painel Grid & Snap que só ele lê.
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Este módulo re-exportava
//! `ph2d_editor_core::grid_snap::ids` inteiro por um glob, dizendo que os ids ficavam lá porque o
//! renderizador do canvas e a barra do topo os lêem — e os que eles lêem ficaram, e quem os usa
//! nomeia `ph2d_editor_core::grid_snap::ids::X` (o corpo do painel, `ph2d_editor_core::ids::GS_PANEL`).
//! Mora aqui só o que este painel lê sozinho.

mod inspector;
pub use inspector::*;
