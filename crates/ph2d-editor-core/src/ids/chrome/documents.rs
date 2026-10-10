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
/// A busca no quadro (`Ctrl+F`, W5).
pub const BOARD_SEARCH_INPUT: NodeId = hash_node_id("board_search_input");
/// Os botões do campo de busca: o resultado anterior, o seguinte, limpar o campo.
pub const BOARD_SEARCH_PREV: NodeId = hash_node_id("board_search_prev");
pub const BOARD_SEARCH_NEXT: NodeId = hash_node_id("board_search_next");
pub const BOARD_SEARCH_CLEAR: NodeId = hash_node_id("board_search_clear");
/// O campo de pôr uma etiqueta nas notas seleccionadas (W5).
pub const BOARD_TAG_INPUT: NodeId = hash_node_id("board_tag_input");
/// O campo que ocupa a linha de um frame no painel de Frames enquanto ele se renomeia (W5).
pub const BOARD_FRAME_RENAME_INPUT: NodeId = hash_node_id("board_frame_rename_input");
/// W8 — o campo de escrever um comentário (o fio aberto), o do nome de quem escreve, e o do nome de
/// uma versão (guardar uma nova, ou renomear).
pub const BOARD_COMMENT_INPUT: NodeId = hash_node_id("board_comment_input");
pub const BOARD_AUTHOR_INPUT: NodeId = hash_node_id("board_author_input");
pub const BOARD_VERSION_INPUT: NodeId = hash_node_id("board_version_input");
/// Menu do botão direito numa aba de quadro.
pub const CTX_MENU_BOARD_RENAME: NodeId = hash_node_id("ctx_menu_board_rename");
pub const CTX_MENU_BOARD_DUPLICATE: NodeId = hash_node_id("ctx_menu_board_duplicate");
pub const CTX_MENU_BOARD_DELETE: NodeId = hash_node_id("ctx_menu_board_delete");
/// W8 — apresentar, as versões, exportar e importar um `.excalidraw`.
pub const CTX_MENU_BOARD_PRESENT: NodeId = hash_node_id("ctx_menu_board_present");
pub const CTX_MENU_BOARD_VERSIONS: NodeId = hash_node_id("ctx_menu_board_versions");
pub const CTX_MENU_BOARD_EXPORT: NodeId = hash_node_id("ctx_menu_board_export");
pub const CTX_MENU_BOARD_IMPORT: NodeId = hash_node_id("ctx_menu_board_import");
/// Os dois botões da pergunta *«apagar este quadro?»*.
pub const CTX_MENU_BOARD_DELETE_CONFIRM: NodeId = hash_node_id("ctx_menu_board_delete_confirm");
pub const CTX_MENU_BOARD_DELETE_CANCEL: NodeId = hash_node_id("ctx_menu_board_delete_cancel");
