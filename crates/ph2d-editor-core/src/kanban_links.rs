//! W11 do Kanban — **o cartão que abre o objecto** (os «Power-Ups» do PH2D, `ph2d_kanban_links`).
//!
//! O canal sem ciclo (o molde de `board_kanban_link`): o verso do cartão pede [`open`] e o DONO do
//! alvo serve-o. O Quadro serve-se aqui mesmo (com elemento, pelo `BoardRequest::Show` que já centra
//! e selecciona; sem elemento, activando o Quadro em [`serve`]); as famílias que vivem fora do
//! editor (a cena e a Hierarquia, a timeline, o editor de scripts, a biblioteca de macros) drenam o
//! que é delas por [`take`]. Quem tem o mundo publica o catálogo da cena por [`publish_scene`], e o
//! verso resolve as ligações contra [`catalog`] (vivas ou quebradas).
//!
//! E a nota adesiva ↔ cartão ([`notes_to_cards`], [`request_card_to_note`]): um passo de desfazer
//! no Quadro, cujo lado do Kanban [`serve`] acompanha.

use std::cell::RefCell;
use std::collections::BTreeSet;

use ph2d_board_model::{BoardId, ElementId};
pub use ph2d_kanban_links::Catalog;
pub use ph2d_kanban_model::Target;

use crate::documents::Documents;
use crate::screens::hero::HeroScreen;
use crate::screens::hero::board_video::kanban::link::{BoardRequest, request as board_request};

// A nota ↔ cartão mexe no Quadro do hero: mora com ele (`screens::hero::board_kanban_notes`).
pub use crate::screens::hero::board_kanban_notes::{
    Choice, choices, notes_to_cards, request_card_to_note, selected_notes,
};

/// Quem serve um pedido «abrir» fora do editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    /// O objecto da cena: seleccioná-lo na cena e na Hierarquia (pelo `StableId`).
    Scene,
    /// O trecho: a timeline vai ao intervalo (e selecciona o dono).
    Timeline,
    /// O script: abri-lo na aba do editor de scripts.
    Script,
    /// A macro: abri-la no painel Macros.
    Macro,
}

/// A família que serve `t` (`None` = o Quadro, servido aqui).
#[must_use]
pub fn family_of(t: &Target) -> Option<Family> {
    match t {
        Target::Object(_) => Some(Family::Scene),
        Target::Span { .. } => Some(Family::Timeline),
        Target::Script(_) => Some(Family::Script),
        Target::Macro(_) => Some(Family::Macro),
        Target::Board { .. } => None,
    }
}

thread_local! {
    static ASKS: RefCell<Vec<Target>> = const { RefCell::new(Vec::new()) };
    static BOARDS: RefCell<Vec<BoardId>> = const { RefCell::new(Vec::new()) };
    static SCENE: RefCell<Catalog> = RefCell::new(Catalog::default());
    static MACROS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

/// ⭐ Pede «abrir/seleccionar/focar» o alvo de uma ligação ao dono dele.
pub fn open(t: &Target) {
    match *t {
        Target::Board {
            board,
            element: Some(e),
        } => board_request(BoardRequest::Show {
            board: BoardId(board),
            element: ElementId(e),
        }),
        Target::Board {
            board,
            element: None,
        } => BOARDS.with(|q| q.borrow_mut().push(BoardId(board))),
        _ => ASKS.with(|q| q.borrow_mut().push(t.clone())),
    }
}

/// Os pedidos da família `f`, pela ordem (os das outras ficam).
pub fn take(f: Family) -> Vec<Target> {
    ASKS.with(|q| {
        let mut q = q.borrow_mut();
        let (mine, rest) = std::mem::take(&mut *q)
            .into_iter()
            .partition(|t| family_of(t) == Some(f));
        *q = rest;
        mine
    })
}

/// Há pedidos da família `f` por drenar? (o painel Macros abre-se antes de o drenar ao pintar)
#[must_use]
pub fn pending(f: Family) -> bool {
    ASKS.with(|q| q.borrow().iter().any(|t| family_of(t) == Some(f)))
}

/// Quem tem o mundo publica o que a cena tem agora (objectos, scripts, macros, clips); os Quadros
/// vêm do projecto em [`catalog`].
pub fn publish_scene(cat: Catalog) {
    SCENE.with(|s| *s.borrow_mut() = cat);
}

/// A família Macros publica os nomes da biblioteca (somam-se aos de [`publish_scene`]).
pub fn publish_macros(names: BTreeSet<String>) {
    MACROS.with(|m| *m.borrow_mut() = names);
}

/// O catálogo do projecto: o da cena publicado, as macros e os Quadros de `docs`.
#[must_use]
pub fn catalog(docs: &Documents) -> Catalog {
    let mut cat = SCENE.with(|s| s.borrow().clone());
    MACROS.with(|m| cat.macros.extend(m.borrow().iter().cloned()));
    cat.set_boards(docs.boards());
    cat
}

/// ⭐ Serve o que é do editor (a família chama-o a cada imagem, ao lado do `board_kanban_link::serve`):
/// activa o Quadro pedido sem elemento, abre o painel Macros a uma macro pedida e acompanha, no Kanban,
/// o desfazer/refazer das conversões. `true` = mudou algo.
pub fn serve(hero: &mut HeroScreen) -> bool {
    let boards = BOARDS.with(|q| std::mem::take(&mut *q.borrow_mut()));
    let mut any = !boards.is_empty();
    for b in boards {
        if hero.documents.boards().get(b).is_some() {
            hero.documents.activate(Some(b));
        }
    }
    any |= crate::screens::hero::board_kanban_notes::serve(hero);
    // Uma macro pedida abre o painel Macros, que a drena ao pintar (`ph2d-panel-macros`).
    if pending(Family::Macro) {
        let node = crate::ids::MACROS_PANEL;
        hero.panel_visibility.insert("macros", true);
        crate::screens::hero::dock_reopen::reopen_for(hero, node);
        hero.store.bump_panel_z(node);
        any = true;
    }
    any
}

#[cfg(test)]
#[path = "kanban_links_tests.rs"]
mod tests;
