//! ⭐ **J7 — a porta entre a sala do QUADRO e a família Kanban** (duas famílias que não se conhecem;
//! o gate `no_dependency_climbs_a_layer`). Por fio, como a fila `open_in_background` do editor: o
//! app desenha e serve a rede num só fio, e cada ecrã dos testes vive no seu.
//!
//! - a família Kanban diz onde estou ([`set_here`], ids LOCAIS) e lê quem está nos Kanbans da sala
//!   do Quadro ([`with_room`]);
//! - a sala do Quadro lê o [`here`] e publica a [`Room`];
//! - um convite colado no «Join» errado vai para o lado certo ([`post_kanban_link`] /
//!   [`post_board_link`]).

use std::cell::RefCell;
use std::collections::BTreeMap;

use ph2d_kanban_model::{CardId, KanbanId};

use crate::presence::Viewers;

/// Onde estou: o Kanban à frente (id LOCAL), o cartão aberto e o cursor no conteúdo.
pub type Here = (KanbanId, Option<CardId>, Option<[f64; 2]>);

/// O que a sala do Quadro sabe de quem está nos Kanbans dela.
#[derive(Clone, Debug, Default)]
pub struct Room {
    pub viewers: Viewers,
    /// Quem é cada par.
    pub names: BTreeMap<u64, String>,
    /// Os Kanbans da sala: `(id da sessão, id local)`.
    pub pairs: Vec<(KanbanId, KanbanId)>,
    /// O relógio da sala (o dos cursores).
    pub now: u64,
    /// A sala caiu e está a voltar.
    pub reconnecting: bool,
}

#[derive(Default)]
struct Port {
    here: Option<Here>,
    room: Option<Room>,
    kanban_link: Option<String>,
    board_link: Option<String>,
}

thread_local! {
    static PORT: RefCell<Port> = RefCell::new(Port::default());
}

/// A família Kanban: onde estou agora (`None` = nenhum Kanban à frente).
pub fn set_here(h: Option<Here>) {
    PORT.with_borrow_mut(|p| p.here = h);
}

/// A sala do Quadro: onde está esta pessoa.
#[must_use]
pub fn here() -> Option<Here> {
    PORT.with_borrow(|p| p.here)
}

/// A sala do Quadro: quem está nos Kanbans dela (`None` = a sala acabou).
pub fn set_room(r: Option<Room>) {
    PORT.with_borrow_mut(|p| p.room = r);
}

/// A família Kanban: lê a sala do Quadro.
pub fn with_room<R>(f: impl FnOnce(Option<&Room>) -> R) -> R {
    PORT.with_borrow(|p| f(p.room.as_ref()))
}

/// Um convite de KANBAN colado no «Join» do Quadro: a família Kanban abre-o.
pub fn post_kanban_link(link: String) {
    PORT.with_borrow_mut(|p| p.kanban_link = Some(link));
}

#[must_use]
pub fn take_kanban_link() -> Option<String> {
    PORT.with_borrow_mut(|p| p.kanban_link.take())
}

/// Um convite de QUADRO colado no «Join» do Kanban: o Quadro entra com ele.
pub fn post_board_link(link: String) {
    PORT.with_borrow_mut(|p| p.board_link = Some(link));
}

#[must_use]
pub fn take_board_link() -> Option<String> {
    PORT.with_borrow_mut(|p| p.board_link.take())
}

/// Os testes: a porta deste fio volta a vazia.
pub fn reset_for_tests() {
    PORT.with_borrow_mut(|p| *p = Port::default());
}
