//! **Os ids da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_timer`], de quem esta secção é o gémeo estrutural: lista + um editor.
//!
//! ⚠️ **A POSIÇÃO NO ARRAY DOS VERBOS É A TAG** — o despacho deriva-a de
//! `position(|&o| o == id)` e o modelo lê-a com `SignalVerb::from_tag`. ⛔ Reordenar
//! [`INSP_ACTION_VERB`] faria um clique escrever outro verbo, **e compila**.

use super::*;

/// A secção SIGNAL ACTIONS — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`].
pub const INSP_LIVE_ACTION_SECTION: NodeId = hash_node_id("insp_live_action_section");
/// SIGNAL ACTIONS — ponto de cor do cabeçalho.
pub const INSP_LIVE_ACTION_COLOR: NodeId = hash_node_id("insp_live_action_color");
