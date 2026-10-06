//! As abas de DOCUMENTO da barra de menus (MiroClone) — as duas fixas, o campo de renomear e o
//! menu do botão direito. A aba de cada quadro tem id DERIVADO do `BoardId`
//! (`screens::hero::document_tabs::tab_node_id`).
use super::{NodeId, hash_node_id};

/// A aba `Cena`.
pub const DOC_TAB_SCENE: NodeId = hash_node_id("doc_tab_scene");
/// O `+` que cria um quadro.
pub const DOC_TAB_NEW: NodeId = hash_node_id("doc_tab_new");
/// O campo que ocupa o lugar da aba enquanto o quadro se renomeia (um de cada vez).
pub const DOC_TAB_RENAME_INPUT: NodeId = hash_node_id("doc_tab_rename_input");
/// Menu do botão direito numa aba de quadro.
pub const CTX_MENU_BOARD_RENAME: NodeId = hash_node_id("ctx_menu_board_rename");
pub const CTX_MENU_BOARD_DUPLICATE: NodeId = hash_node_id("ctx_menu_board_duplicate");
pub const CTX_MENU_BOARD_DELETE: NodeId = hash_node_id("ctx_menu_board_delete");
/// Os dois botões da pergunta *«apagar este quadro?»*.
pub const CTX_MENU_BOARD_DELETE_CONFIRM: NodeId = hash_node_id("ctx_menu_board_delete_confirm");
pub const CTX_MENU_BOARD_DELETE_CANCEL: NodeId = hash_node_id("ctx_menu_board_delete_cancel");
