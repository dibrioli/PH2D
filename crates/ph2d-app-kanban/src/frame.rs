//! ⭐ **O quadro da família** — chamado pela shell uma vez por quadro de imagem: segue o conjunto de
//! Kanbans do projecto (outro projecto larga as abas), o relógio do chrome e a lei das molas (o
//! carácter e o «movimento reduzido» do hero), e a aba activa (a que sai grava o campo aberto).

use crate::docs::KanbanDocs;
use ph2d_editor_core::motion::Role;
use ph2d_editor_core::screens::hero::HeroScreen;

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
}

impl KanbanDocs {
    /// A aba activa deste quadro (`None` = outra família, um quadro ou a Cena): a que deixou de o
    /// ser grava o campo aberto e larga o gesto.
    pub(crate) fn follow_active(&mut self, active: Option<u64>) {
        let left: Vec<u64> = self.tabs.iter().map(crate::tab::Tab::id).filter(|id| Some(*id) != active).collect();
        for id in left {
            let busy = self.tabs.iter().any(|t| t.id() == id && (t.field.is_some() || t.drag.is_some() || t.press.is_some()));
            if busy {
                self.leave(id);
            }
        }
    }
}
