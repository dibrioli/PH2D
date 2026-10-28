//! **Abrir o verso de um cartão** a pedido de outra família (o Planner, o sino): quem pede não
//! conhece a aba do Kanban; ela esvazia os pedidos a cada quadro ([`take_open_cards`]).

use std::cell::RefCell;

use ph2d_kanban_model::CardRef;

thread_local! {
    static OPEN: RefCell<Vec<CardRef>> = const { RefCell::new(Vec::new()) };
}

/// Pede o verso de `card` numa aba do Kanban dele.
pub fn request_open_card(card: CardRef) {
    OPEN.with(|q| q.borrow_mut().push(card));
}

/// Os cartões pedidos desde a última vez, por ordem.
#[must_use]
pub fn take_open_cards() -> Vec<CardRef> {
    OPEN.with(|q| std::mem::take(&mut *q.borrow_mut()))
}
