//! ⭐ **O gesto da família pela porta única** ([`ph2d_kanban_hooks`]): toda escrita da pessoa —
//! a aba, o elemento do Quadro, o verso, as vistas, os painéis da W6 — chega ao documento por aqui,
//! e o Butler e os scripts `ph2d.kanban.on` reagem a ela. O desfazer/refazer NÃO passa aqui.
//!
//! Duas entradas, a MESMA lei: um [`Plan`] aplica-se e reage já ([`write`]); o lote de um campo no
//! lugar (`actions::Doc::apply`, que só tem o histórico do seu Kanban) aplica-se, anota o que mudou
//! ([`noted`]) e reage no fim da MESMA entrada ([`settle`], chamado pela superfície e pelo quadro).

mod probe;
pub use probe::CanvasW6;

use std::cell::RefCell;

use crate::docs::KanbanDocs;
use ph2d_editor_core::documents::undo_tick;
use ph2d_kanban_auto::{Event, Seen};
use ph2d_kanban_hooks::{Agenda, Reaction, Who};
use ph2d_kanban_model::{Kanban, KanbanOp, KanbanSet, Member, Plan};

thread_local! {
    /// O que os lotes de campo mudaram e ainda não teve reação (o autor entra no [`settle`]).
    static PENDING: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) };
    static AGENDA: RefCell<Agenda> = RefCell::new(Agenda::default());
}

/// Um lote já aplicado por outra porta do modelo: o que ele mudou fica à espera do [`settle`].
pub(crate) fn noted(seen: &Seen, set: &KanbanSet) {
    let events = seen.events(set, &Member(String::new()));
    if !events.is_empty() {
        PENDING.with(|p| p.borrow_mut().extend(events));
    }
}

fn who(docs: &KanbanDocs) -> (Member, ph2d_kanban_model::Moment) {
    (docs.me.clone(), crate::actions::now())
}

fn reacted(docs: &mut KanbanDocs, r: &Reaction) {
    if r.stopped.is_some() {
        docs.notices
            .push(ph2d_i18n::tr("kanban.hooks.stopped").to_owned());
    }
}

/// ⭐ Um gesto da pessoa: o plano em nome de «eu», o Butler e os scripts. `true` = escreveu.
pub(crate) fn write(docs: &mut KanbanDocs, plan: Option<Plan>) -> bool {
    let Some(plan) = plan.filter(|p| !p.is_empty()) else {
        return false;
    };
    settle(docs);
    let (me, now) = who(docs);
    let zone = crate::back::dates::zone();
    let w = Who {
        me,
        now,
        zone: &zone,
    };
    let r = ph2d_kanban_hooks::gesture(
        &mut docs.set.borrow_mut(),
        &mut docs.histories,
        plan,
        &w,
        &mut undo_tick,
    );
    reacted(docs, &r);
    // Um gesto que não mudou nada ainda «fez» (o painel fecha); o modelo não guarda passo vazio.
    true
}

/// O lote que `f` monta sobre o Kanban da vista `id`, como um gesto ([`write`]).
pub(crate) fn write_in(
    docs: &mut KanbanDocs,
    id: u64,
    f: impl FnOnce(&Kanban) -> Option<Vec<KanbanOp>>,
) -> bool {
    let Some(kid) = docs.kanban_of(id) else {
        return false;
    };
    let ops = docs.set.borrow().get(kid).and_then(f);
    let Some(ops) = ops.filter(|o| !o.is_empty()) else {
        return false;
    };
    let before = docs.histories.get(&kid).and_then(ph2d_kanban_model::History::undo_at);
    let mut plan = Plan::default();
    plan.extend(kid, ops);
    write(docs, Some(plan));
    docs.histories.get(&kid).and_then(ph2d_kanban_model::History::undo_at) != before
}

/// O que o Butler fez por um botão ou pelo painel (já com a cascata das regras, a lei do
/// `ph2d-kanban-auto`): aplica-se em nome dele e só os scripts reagem. `true` = escreveu.
pub(crate) fn butler(docs: &mut KanbanDocs, plan: Plan) -> bool {
    if plan.is_empty() {
        return false;
    }
    settle(docs);
    let (me, now) = who(docs);
    let zone = crate::back::dates::zone();
    let w = Who {
        me,
        now,
        zone: &zone,
    };
    let r = ph2d_kanban_hooks::butler(
        &mut docs.set.borrow_mut(),
        &mut docs.histories,
        plan,
        &w,
        &mut undo_tick,
    );
    reacted(docs, &r);
    true
}

/// ⭐ O fim de uma entrada: as reações ao que os lotes de campo mudaram.
pub(crate) fn settle(docs: &mut KanbanDocs) {
    let mut events = PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()));
    if events.is_empty() {
        return;
    }
    let (me, now) = who(docs);
    for e in &mut events {
        e.by = me.clone();
    }
    let zone = crate::back::dates::zone();
    let w = Who {
        me,
        now,
        zone: &zone,
    };
    let r = ph2d_kanban_hooks::react(
        &mut docs.set.borrow_mut(),
        &mut docs.histories,
        events,
        &w,
        &mut undo_tick,
    );
    reacted(docs, &r);
}

/// O quadro: as reações que ficaram, os scripts que o editor pediu para correr e a agenda.
pub(crate) fn frame(docs: &mut KanbanDocs) {
    settle(docs);
    let (me, now) = who(docs);
    let zone = crate::back::dates::zone();
    let w = Who {
        me,
        now,
        zone: &zone,
    };
    for req in ph2d_kanban_hooks::take_runs() {
        let at = undo_tick();
        let _ = ph2d_kanban_hooks::run_script(
            &req,
            &mut docs.set.borrow_mut(),
            &mut docs.histories,
            &w,
            at,
        );
    }
    let r = AGENDA.with(|a| a.borrow_mut().tick(&mut docs.set.borrow_mut(), &w));
    if let Some(r) = r {
        reacted(docs, &r);
    }
}
