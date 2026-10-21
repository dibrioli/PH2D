//! ⭐ **OS MENUS DE BOTÃO DIREITO DO QUADRO** (MiroClone): a aba de um quadro e um elemento dele (forma,
//! seta ao longo / na horizontal).
//!
//! ⚠️ **Irmão do [`super::menu_tables`] pelo tecto de 700 linhas** (W23 bloco 6, 20/10). O gate
//! `every_menu_row_reaches_a_handler` lê os ficheiros de tabelas como UMA população — este está na
//! lista dele.

use crate::ids::{self, MenuRow, menu_row};

/// As linhas de `ContextMenuKind::BoardTab { .. }` (MiroClone) — a ordem da linha da Hierarquia:
/// renomear, a cópia, a equipa e a saída (W8), e o destrutivo por último (que ainda pergunta).
pub(super) const BOARD_TAB_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_BOARD_RENAME, "board.menu.rename"),
    menu_row(ids::CTX_MENU_BOARD_DUPLICATE, "board.menu.duplicate"),
    menu_row(ids::CTX_MENU_BOARD_PRESENT, "board.menu.present"),
    menu_row(ids::CTX_MENU_BOARD_VERSIONS, "board.menu.versions"),
    menu_row(ids::CTX_MENU_BOARD_EXPORT, "board.menu.export"),
    menu_row(ids::CTX_MENU_BOARD_IMPORT, "board.menu.import"),
    menu_row(ids::CTX_MENU_BOARD_DELETE, "board.menu.delete"),
];

/// W23 — as linhas de `ContextMenuKind::BoardElement` (o botão direito numa forma do quadro, o Miro):
/// a cópia, a ordem, travar, e o destrutivo por último.
pub(super) const BOARD_ELEMENT_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_BOARD_EL_COPY, "board.el.copy"),
    menu_row(ids::CTX_MENU_BOARD_EL_PASTE, "board.el.paste"),
    menu_row(ids::CTX_MENU_BOARD_EL_DUPLICATE, "board.el.duplicate"),
    menu_row(ids::CTX_MENU_BOARD_EL_COPY_TO, "board.el.copy_to"),
    menu_row(ids::CTX_MENU_BOARD_EL_MOVE_TO, "board.el.move_to"),
    menu_row(ids::CTX_MENU_BOARD_EL_FRONT, "board.el.front"),
    menu_row(ids::CTX_MENU_BOARD_EL_BACK, "board.el.back"),
    menu_row(ids::CTX_MENU_BOARD_EL_LOCK, "board.el.lock"),
    menu_row(ids::CTX_MENU_BOARD_EL_DELETE, "board.el.delete"),
];

/// W23 B5(2) — numa SETA, as mesmas e, antes de travar, a que deita os textos do outro modo (os
/// textos estão ao longo da linha: a linha oferece a horizontal).
pub(super) const BOARD_ARROW_ALONG_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_BOARD_EL_COPY, "board.el.copy"),
    menu_row(ids::CTX_MENU_BOARD_EL_PASTE, "board.el.paste"),
    menu_row(ids::CTX_MENU_BOARD_EL_DUPLICATE, "board.el.duplicate"),
    menu_row(ids::CTX_MENU_BOARD_EL_COPY_TO, "board.el.copy_to"),
    menu_row(ids::CTX_MENU_BOARD_EL_MOVE_TO, "board.el.move_to"),
    menu_row(ids::CTX_MENU_BOARD_EL_FRONT, "board.el.front"),
    menu_row(ids::CTX_MENU_BOARD_EL_BACK, "board.el.back"),
    menu_row(ids::CTX_MENU_BOARD_EL_TEXT_LEVEL, "board.el.text_level"),
    menu_row(ids::CTX_MENU_BOARD_EL_LOCK, "board.el.lock"),
    menu_row(ids::CTX_MENU_BOARD_EL_DELETE, "board.el.delete"),
];

/// W23 B5(2) — os textos estão na horizontal: a linha oferece deitá-los ao longo da linha.
pub(super) const BOARD_ARROW_LEVEL_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_BOARD_EL_COPY, "board.el.copy"),
    menu_row(ids::CTX_MENU_BOARD_EL_PASTE, "board.el.paste"),
    menu_row(ids::CTX_MENU_BOARD_EL_DUPLICATE, "board.el.duplicate"),
    menu_row(ids::CTX_MENU_BOARD_EL_COPY_TO, "board.el.copy_to"),
    menu_row(ids::CTX_MENU_BOARD_EL_MOVE_TO, "board.el.move_to"),
    menu_row(ids::CTX_MENU_BOARD_EL_FRONT, "board.el.front"),
    menu_row(ids::CTX_MENU_BOARD_EL_BACK, "board.el.back"),
    menu_row(ids::CTX_MENU_BOARD_EL_TEXT_ALONG, "board.el.text_along"),
    menu_row(ids::CTX_MENU_BOARD_EL_LOCK, "board.el.lock"),
    menu_row(ids::CTX_MENU_BOARD_EL_DELETE, "board.el.delete"),
];
