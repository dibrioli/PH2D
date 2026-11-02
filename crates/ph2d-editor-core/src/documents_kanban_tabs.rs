//! **A aba de um Kanban do projecto** (ADR-0195, D17): os pedidos de abrir e de fechar, que o editor
//! faz e a família Kanban esvazia a cada quadro. Mora nos documentos porque abrir e apagar um
//! documento são deles ([`super::Documents::open_asset`], apagar pelos Assets); o quadro também os
//! pede (duplo clique, «Open in tab»), e sem ciclo: o ecrã depende dos documentos, nunca o inverso.

use std::cell::{Cell, RefCell};

use ph2d_board_model::KanbanId;

use super::BoardLive;

thread_local! {
    static OPENER: Cell<bool> = const { Cell::new(false) };
    static OPEN: RefCell<Vec<KanbanId>> = const { RefCell::new(Vec::new()) };
    static CLOSE: RefCell<Vec<KanbanId>> = const { RefCell::new(Vec::new()) };
}

/// A frase quando não há quem abra a aba.
pub(crate) const NO_TAB: &str = "board.kanban.no_tab";

/// Quem abre abas de Kanban existe neste fio (esvazia [`take_open_requests`] a cada quadro).
pub fn install_tab_opener() {
    OPENER.with(|o| o.set(true));
}

/// Há quem abra abas de Kanban («Open in tab» fica aceso).
#[must_use]
pub fn can_open_tabs() -> bool {
    OPENER.with(Cell::get)
}

/// Pede o Kanban `k` numa aba; sem quem a abra, o aviso no quadro. `true` = pedido.
pub(crate) fn request_open(live: &mut BoardLive, k: KanbanId) -> bool {
    if !can_open_tabs() {
        live.notice = Some((NO_TAB, std::time::Instant::now()));
        return false;
    }
    OPEN.with(|q| q.borrow_mut().push(k));
    true
}

/// Os Kanbans pedidos numa aba desde a última vez, por ordem.
#[must_use]
pub fn take_open_requests() -> Vec<KanbanId> {
    OPEN.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// O Kanban apagado fecha a aba dele (a família esvazia [`take_close_requests`] a cada quadro).
pub(crate) fn request_close(k: KanbanId) {
    CLOSE.with(|q| q.borrow_mut().push(k));
}

/// Os Kanbans apagados desde a última vez (as abas deles fecham).
#[must_use]
pub fn take_close_requests() -> Vec<KanbanId> {
    CLOSE.with(|q| std::mem::take(&mut *q.borrow_mut()))
}
