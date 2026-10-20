//! ⭐ W23 B15 — **«Copy to board…» / «Move to board…»** (pedido do dono 20/10; doc 17 §2.4): o botão
//! direito numa forma do quadro → o modal do «Send to board» ([`super::send_to_board::open_selection`])
//! → aqui, na pintura seguinte. A selecção sai pela porta do editor ([`ph2d_board_edit::Editor::lift`],
//! a mesma lei de colar noutro quadro), o quadro de destino fica ACTIVO com ela no meio da vista,
//! seleccionada; mover apaga-a na origem. Cada quadro guarda o SEU passo (o `Ctrl+Z` nunca desfaz
//! noutro quadro, doc 02 §1.4) e o aviso «Undo» desfaz os dois ([`super::board_alert`]).

use super::HeroScreen;
use super::board_alert::{Action, Alert};
use super::board_view::editor;
use super::document_tabs::register_board;
use ph2d_board_edit::Command;
use ph2d_board_model::BoardId;
use ph2d_i18n::tr_with;
use ph2d_text::TextSystem;

/// A escolha do modal à espera da pintura: de que quadro, para qual (`None` = um novo), copiar ou mover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Send {
    pub(crate) from: BoardId,
    pub(crate) to: Option<BoardId>,
    pub(crate) keep: bool,
}

/// Corre a escolha enfileirada. Nada muda se a origem já não é o quadro activo, se a selecção está
/// vazia ou se o quadro escolhido foi fechado entretanto.
pub(crate) fn run_pending(hero: &mut HeroScreen, ts: &mut TextSystem) {
    let Some(s) = hero.documents.live.send.take() else {
        return;
    };
    if hero.documents.active() != Some(s.from)
        || s.to.is_some_and(|b| hero.documents.boards().get(b).is_none())
    {
        return;
    }
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let ed = editor(&mut live.editor, theme);
    let parcel = ed.lift(&board.doc, &board.files);
    if parcel.is_empty() {
        return;
    }
    let from_at = (!s.keep)
        .then(|| {
            let h = live.histories.entry(board.id).or_default();
            ed.command(&mut board.doc, h, ts, Command::Delete);
            h.undo_at()
        })
        .flatten();
    let to = match s.to {
        Some(b) => {
            hero.documents.activate(Some(b));
            b
        }
        None => {
            let n = hero.documents.boards().boards().len() + 1;
            let b = hero
                .documents
                .create(tr_with("board.tab.default_name", &[("n", &n)]));
            register_board(&mut hero.store, b);
            b
        }
    };
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let centre = [board.camera.center_x, board.camera.center_y];
    let h = live.histories.entry(to).or_default();
    let ed = editor(&mut live.editor, theme);
    if !ed.land(&mut board.doc, h, &mut board.files, &parcel, centre) {
        return;
    }
    let n = parcel.picked();
    let name = board.name.clone();
    let key = match (s.keep, n) {
        (true, 1) => "board.send.copied_one",
        (true, _) => "board.send.copied",
        (false, 1) => "board.send.moved_one",
        (false, _) => "board.send.moved",
    };
    let text = tr_with(key, &[("count", &n), ("name", &name)]);
    let action = match (from_at, h.undo_at()) {
        (Some(from_at), Some(to_at)) => Some(Action::UndoMove {
            from: s.from,
            from_at,
            to,
            to_at,
        }),
        _ => None,
    };
    live.alert = Some(Alert::new(text, action));
}
