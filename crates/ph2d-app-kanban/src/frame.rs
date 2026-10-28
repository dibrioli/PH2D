//! ⭐ **O quadro da família** — chamado pela shell uma vez por quadro de imagem: segue o conjunto de
//! Kanbans do projecto (outro projecto larga as abas), o relógio do chrome e a lei das molas (o
//! carácter e o «movimento reduzido» do hero), e a aba activa (a que sai grava o campo aberto).

use crate::docs::KanbanDocs;
use ph2d_kanban_model::KanbanId;
use std::collections::BTreeMap;
use ph2d_editor_core::motion::Role;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_video::kanban as kb;

pub fn frame(hero: &mut HeroScreen) {
    let shared = hero.documents.kanbans_shared();
    let now = hero.store.clock_ns();
    let law = hero.motion.law(Role::Travel);
    let active = hero.documents.surface_active();
    let ours = active.and_then(|t| {
        hero.documents
            .surfaces_mut()
            .get(t.family)
            .is_some_and(|f| f.as_any().is::<KanbanDocs>())
            .then_some(t.id)
    });
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<KanbanDocs>() else {
        return;
    };
    docs.follow_project(shared);
    docs.set_clock(now, law);
    docs.follow_active(ours);
    // W4 — o canvas do Quadro: as vistas dos elementos leem o relógio e o «eu» deste quadro.
    {
        let mut c = docs.canvas.borrow_mut();
        c.cx = docs.view_ctx();
        c.cx.floor = None;
        c.set = Some(docs.set());
    }
    for k in kb::take_open_requests() {
        docs.open(k);
    }
    for k in kb::take_background_requests() {
        let focus = docs.focus;
        docs.open(k);
        docs.focus = focus;
    }
    // «Show on board»: onde cada Kanban aberto (na aba ou no canvas) está posto.
    let mut open = docs.open_kanbans();
    open.extend(docs.canvas.borrow().tabs.values().map(|t| t.view.kanban));
    open.sort_unstable();
    open.dedup();
    let placed: BTreeMap<KanbanId, Vec<(u64, u64, String)>> = open
        .into_iter()
        .map(|k| {
            let p = kb::link::placements(&hero.documents, k)
                .into_iter()
                .map(|p| (p.board.0, p.element.0, p.board_name))
                .collect();
            (k, p)
        })
        .collect();
    if let Some(docs) = hero.documents.surfaces_mut().find_mut::<KanbanDocs>() {
        let mut c = docs.canvas.borrow_mut();
        for t in docs.tabs.iter_mut().chain(c.tabs.values_mut()) {
            t.placements = placed.get(&t.view.kanban).cloned().unwrap_or_default();
        }
    }
    kb::link::serve(hero);
}

impl KanbanDocs {
    /// A aba activa deste quadro (`None` = outra família, um quadro ou a Cena): a que deixou de o
    /// ser grava o campo aberto e larga o gesto.
    pub(crate) fn follow_active(&mut self, active: Option<u64>) {
        // A aba que pediu o foco é a deste quadro: o hero só a activa ao pintar, depois deste
        // `frame` (senão o nome de um Kanban acabado de criar fechava antes de se ver).
        let active = self.focus.or(active);
        let left: Vec<u64> = self
            .tabs
            .iter()
            .map(crate::tab::Tab::id)
            .filter(|id| Some(*id) != active)
            .collect();
        for id in left {
            let busy = self.tabs.iter().any(|t| {
                t.id() == id && (t.field.is_some() || t.drag.is_some() || t.press.is_some())
            });
            if busy {
                self.leave(id);
            }
        }
    }
}
