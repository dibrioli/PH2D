//! W4 do Kanban, lote V — **as ligações entre a aba e o quadro** (ADR-0195 §3.2): «Show on board» e
//! «Place on board» pedidos da aba (o canal [`request`] → [`serve`], molde do `install_tab_opener`: a
//! família pede, o editor serve no quadro seguinte, sem ciclo), e o desfazer de «inserir Kanban» que
//! leva o documento com a referência se ele ficou intocado ([`born`] → [`after_history`]).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};

use ph2d_board_model::{BoardDoc, BoardId, ElementId, KanbanId};
use ph2d_kanban_model::{Kanban, KanbanSet};

use crate::documents::{Documents, SharedKanbans};
use crate::screens::hero::HeroScreen;
use crate::screens::hero::board_view::{area, area_of, editor};

/// O que a aba pede ao quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardRequest {
    /// Ir ao quadro `board` e centrar a câmara no elemento `element` (seleccionado).
    Show { board: BoardId, element: ElementId },
    /// Uma referência NOVA ao Kanban no quadro activo (o último que o foi), ou no primeiro; sem
    /// quadros, um quadro novo.
    Place { kanban: KanbanId },
}

/// Onde um Kanban está posto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub board: BoardId,
    pub element: ElementId,
    /// O nome do quadro (o popover da escolha mostra-o).
    pub board_name: String,
}

/// Um Kanban que nasceu com a referência dele.
struct Born {
    set: Weak<RefCell<KanbanSet>>,
    pristine: Kanban,
    /// Levado por um desfazer (o refazer repõe-no).
    stash: Option<Kanban>,
}

thread_local! {
    static REQUESTS: RefCell<Vec<BoardRequest>> = const { RefCell::new(Vec::new()) };
    static BORN: RefCell<BTreeMap<KanbanId, Born>> = const { RefCell::new(BTreeMap::new()) };
    static LAST_BOARD: Cell<Option<BoardId>> = const { Cell::new(None) };
}

/// A aba pede algo ao quadro (servido no quadro seguinte por [`serve`]).
pub fn request(r: BoardRequest) {
    REQUESTS.with(|q| q.borrow_mut().push(r));
}

/// Os quadros onde o Kanban `kanban` está posto, pela ordem das abas.
#[must_use]
pub fn placements(docs: &Documents, kanban: KanbanId) -> Vec<Placement> {
    let mut out = Vec::new();
    for b in docs.boards().boards() {
        for el in b.doc.live() {
            if el.kanban().is_some_and(|k| k.kanban == kanban) {
                out.push(Placement {
                    board: b.id,
                    element: el.id,
                    board_name: b.name.clone(),
                });
            }
        }
    }
    out
}

/// ⭐ Serve os pedidos da aba (a família chama-o a cada quadro de imagem). `true` = serviu algum.
pub fn serve(hero: &mut HeroScreen) -> bool {
    if let Some(b) = hero.documents.active() {
        LAST_BOARD.with(|l| l.set(Some(b)));
    }
    let asks = REQUESTS.with(|q| std::mem::take(&mut *q.borrow_mut()));
    let any = !asks.is_empty();
    for r in asks {
        match r {
            BoardRequest::Show { board, element } => show(hero, board, element),
            BoardRequest::Place { kanban } => {
                place(hero, kanban);
            }
        }
    }
    any
}

/// Vai ao quadro `board`, centra a câmara no elemento e selecciona-o.
fn show(hero: &mut HeroScreen, board: BoardId, element: ElementId) {
    hero.documents.activate(Some(board));
    let theme = hero.theme;
    let Some((b, live)) = hero.documents.active_parts() else {
        return;
    };
    let Some(el) = b.doc.get(element) else {
        return;
    };
    b.camera.center_x = el.x + el.w / 2.0;
    b.camera.center_y = el.y + el.h / 2.0;
    editor(&mut live.editor, theme).select(&b.doc, [element]);
}

/// ⭐ Uma referência nova ao Kanban no quadro activo, ou no último que o foi, ou no primeiro (um
/// quadro novo se não há nenhum); o quadro fica à vista com ela seleccionada. UM passo de desfazer.
pub(crate) fn place(hero: &mut HeroScreen, kanban: KanbanId) -> Option<(BoardId, ElementId)> {
    let boards = hero.documents.boards();
    let last = LAST_BOARD.with(Cell::get).filter(|b| boards.get(*b).is_some());
    let target = hero
        .documents
        .active()
        .or(last)
        .or_else(|| boards.boards().first().map(|b| b.id));
    let target = match target {
        Some(b) => b,
        None => {
            let n = boards.boards().len() + 1;
            let name = ph2d_i18n::tr_with("board.tab.default_name", &[("n", &n)]);
            let b = hero.documents.create(name);
            crate::screens::hero::document_tabs::register_board(&mut hero.store, b);
            b
        }
    };
    hero.documents.activate(Some(target));
    // Posta noutro sítio, a referência já não é só do gesto que a criou.
    BORN.with(|b| b.borrow_mut().remove(&kanban));
    let view = hero.last_layout.map(|l| area_of(area(&l)));
    let theme = hero.theme;
    let (board, live) = hero.documents.active_parts()?;
    let z = board.camera.zoom;
    let fit = view.map_or([f64::INFINITY; 2], |v| [v[2] / z / 2.0, v[3] / z / 2.0]);
    let center = [board.camera.center_x, board.camera.center_y];
    let history = live.histories.entry(board.id).or_default();
    let ed = editor(&mut live.editor, theme);
    let el = ed.insert_kanban(&mut board.doc, history, kanban, center, fit);
    Some((board.id, el))
}

/// O Kanban `k` acabou de nascer com a referência dele.
pub(crate) fn born(set: &SharedKanbans, k: KanbanId) {
    let Some(pristine) = set.borrow().get(k).cloned() else {
        return;
    };
    let b = Born {
        set: Rc::downgrade(set),
        pristine,
        stash: None,
    };
    BORN.with(|m| m.borrow_mut().insert(k, b));
}

/// Os Kanbans referidos no quadro.
#[must_use]
pub(crate) fn refs(doc: &BoardDoc) -> BTreeSet<KanbanId> {
    doc.live().filter_map(|e| e.kanban().map(|k| k.kanban)).collect()
}

/// ⭐ Depois de um desfazer/refazer do quadro (`before` = os referidos antes): o Kanban nascido com a
/// referência que o desfazer tirou, ainda intocado, sai do projecto com ela; o refazer repõe-no.
pub(crate) fn after_history(doc: &BoardDoc, before: &BTreeSet<KanbanId>) {
    let now = refs(doc);
    BORN.with(|m| {
        let mut m = m.borrow_mut();
        m.retain(|_, b| b.set.strong_count() > 0);
        for k in before.difference(&now) {
            let Some(b) = m.get_mut(k) else { continue };
            let Some(set) = b.set.upgrade() else { continue };
            let untouched = set.borrow().get(*k) == Some(&b.pristine);
            if untouched && b.stash.is_none() {
                b.stash = set.borrow_mut().remove(*k);
            }
        }
        for k in now.difference(before) {
            let Some(b) = m.get_mut(k) else { continue };
            if let (Some(set), Some(kb)) = (b.set.upgrade(), b.stash.take()) {
                set.borrow_mut().insert(kb);
            }
        }
    });
}
