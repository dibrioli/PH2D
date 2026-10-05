//! As abas de DOCUMENTO da barra de menus (MiroClone) — as duas fixas. A aba de cada quadro tem id
//! DERIVADO do `BoardId` (`screens::hero::document_tabs::tab_node_id`).
use super::{NodeId, hash_node_id};

/// A aba `Cena`.
pub const DOC_TAB_SCENE: NodeId = hash_node_id("doc_tab_scene");
/// O `+` que cria um quadro.
pub const DOC_TAB_NEW: NodeId = hash_node_id("doc_tab_new");
