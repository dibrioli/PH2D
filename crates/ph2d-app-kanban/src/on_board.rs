//! W4, lote V — **«Show on board» e «Place on board»** no «…» do Kanban (ADR-0195 §3.2): o pedido vai
//! ao editor pelo canal dele (`board_video::kanban::link`), servido no quadro seguinte.

use crate::docs::KanbanDocs;
use crate::menu::{Menu, Row};
use ph2d_board_model::{BoardId, ElementId};
use ph2d_editor_core::screens::hero::board_video::kanban::link::{self, BoardRequest};

/// Uma linha do «…» que liga a aba ao Quadro.
pub(crate) fn row(docs: &mut KanbanDocs, id: u64, row: Row) {
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    let show = |p: &(u64, u64, String)| {
        link::request(BoardRequest::Show {
            board: BoardId(p.0),
            element: ElementId(p.1),
        });
    };
    match row {
        Row::ShowOnBoard => match t.placements.len() {
            0 => {
                let n = ph2d_i18n::tr("kanban.board.nowhere").to_owned();
                docs.notices.push(n);
            }
            1 => show(&t.placements[0]),
            n => t.menu = Some(Menu::Boards(n)),
        },
        Row::Board(i) => {
            if let Some(p) = t.placements.get(i) {
                show(p);
            }
        }
        Row::PlaceOnBoard => link::request(BoardRequest::Place {
            kanban: t.view.kanban,
        }),
        _ => {}
    }
}
