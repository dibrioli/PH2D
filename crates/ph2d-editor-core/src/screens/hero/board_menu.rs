//! W23 B5 — **o menu do botão direito numa forma do quadro** (o Miro): o botão direito sobre um
//! elemento selecciona-o (se não estava) e abre *Copy · Paste · Duplicate · Bring to front · Send to
//! back · Lock · Delete*, que agem na selecção. A linha clicada enfileira o comando; ele corre na
//! pintura seguinte, onde há o moldador de texto que o editor pede.

use super::HeroScreen;
use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_board_edit::{Command, Order};
use ph2d_board_model::TextTurn;
use ph2d_host::Modifiers;
use ph2d_text::TextSystem;

/// As linhas do menu e o comando de cada uma.
const ROWS: [(ph2d_a11y::NodeId, Command); 9] = [
    (ids::CTX_MENU_BOARD_EL_COPY, Command::Copy),
    (ids::CTX_MENU_BOARD_EL_PASTE, Command::Paste),
    (ids::CTX_MENU_BOARD_EL_DUPLICATE, Command::Duplicate),
    (ids::CTX_MENU_BOARD_EL_FRONT, Command::Order(Order::Front)),
    (ids::CTX_MENU_BOARD_EL_BACK, Command::Order(Order::Back)),
    (ids::CTX_MENU_BOARD_EL_LOCK, Command::Lock),
    (ids::CTX_MENU_BOARD_EL_DELETE, Command::Delete),
    (
        ids::CTX_MENU_BOARD_EL_TEXT_ALONG,
        Command::TextTurn(TextTurn::Along),
    ),
    (
        ids::CTX_MENU_BOARD_EL_TEXT_LEVEL,
        Command::TextTurn(TextTurn::Level),
    ),
];

/// ⭐ O botão direito no quadro activo (a porta do quadro, `board_view::canvas_down`, chama-o): sobre
/// um elemento selecciona-o (se não estava) e abre o menu dele; no vazio, nada.
pub(super) fn open(hero: &mut HeroScreen, x: f32, y: f32) {
    let theme = hero.theme;
    let canvas = hero.last_canvas;
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let area = super::board_view::area_of(canvas);
    let p = super::board_view::world_pointer(board, area, x, y, Modifiers::default());
    let ed = super::board_view::editor(&mut live.editor, theme);
    let Some(id) = ed.hit(&board.doc, p) else {
        return;
    };
    if !ed.selection().contains(&id) {
        ed.select(&board.doc, [id]);
    }
    // Numa seta, a linha que deita os textos dela do outro modo.
    let kind = match board.doc.get(id).and_then(|el| el.connector()) {
        Some(c) => ContextMenuKind::BoardArrow {
            along: c.turn == TextTurn::Along,
        },
        None => ContextMenuKind::BoardElement,
    };
    hero.store
        .open_context_menu(ContextMenuRequest { x, y, kind });
}

/// W23 B15 — as linhas que mandam a selecção a outro quadro: `true` = copiar, `false` = mover.
const SEND_ROWS: [(ph2d_a11y::NodeId, bool); 2] = [
    (ids::CTX_MENU_BOARD_EL_COPY_TO, true),
    (ids::CTX_MENU_BOARD_EL_MOVE_TO, false),
];

/// As linhas do menu (o `document_tabs_menu::populate` regista-as com as do menu das abas).
pub(super) fn row_ids() -> impl Iterator<Item = ph2d_a11y::NodeId> {
    ROWS.into_iter()
        .map(|(id, _)| id)
        .chain(SEND_ROWS.into_iter().map(|(id, _)| id))
}

/// Uma linha do menu: fecha-o e enfileira o comando (ou abre a escolha do quadro). Corre no
/// pré-despacho.
pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    if let Some(&(_, keep)) = SEND_ROWS.iter().find(|(row, _)| *row == id) {
        hero.store.close_context_menu();
        hero.store.consume_last_context_menu();
        if let Some(from) = hero.documents.active() {
            super::send_to_board::open_selection(&mut hero.store, from, keep);
        }
        return true;
    }
    let Some(&(_, c)) = ROWS.iter().find(|(row, _)| *row == id) else {
        return false;
    };
    hero.store.close_context_menu();
    hero.store.consume_last_context_menu();
    hero.documents.live.menu_command = Some(c);
    true
}

/// Corre o comando que o menu enfileirou (na pintura, com o moldador). A cópia vai também para fora do
/// app, como a do `Ctrl+C`; colar cola o que o quadro copiou.
pub(crate) fn run_pending(hero: &mut HeroScreen, ts: &mut TextSystem) {
    let Some(c) = hero.documents.live.menu_command.take() else {
        return;
    };
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let history = live.histories.entry(board.id).or_default();
    let ed = super::board_view::editor(&mut live.editor, theme);
    if c == Command::Paste {
        ed.give_clip_files(&mut board.files);
        let centre = [board.camera.center_x, board.camera.center_y];
        ed.paste(&mut board.doc, history, Some(centre));
        return;
    }
    ed.command(&mut board.doc, history, ts, c);
    if c == Command::Copy {
        ed.keep_clip_files(&board.files);
        if let Some(t) = ed.copied_text() {
            hero.store.set_clipboard_copy(t.to_owned());
        }
    }
}
