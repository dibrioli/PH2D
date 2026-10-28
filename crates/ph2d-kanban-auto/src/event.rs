//! **O que aconteceu a um cartão** — os gatilhos lidos das ops, comparando o alvo de cada uma ANTES
//! e DEPOIS ([`Seen`]). A [`ph2d_kanban_model::Activity`] não basta: o Trello não regista etiquetas,
//! campos nem comentários na atividade, e o Butler dispara por eles.

use std::collections::BTreeSet;

use ph2d_kanban_model::{
    CardId, CheckItemId, ChecklistId, CommentId, FieldId, KanbanId, KanbanOp, KanbanSet, LabelId,
    ListId, Member, Moment, Plan, Record, RecordId,
};

/// Um acontecimento num cartão, por quem.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub kanban: KanbanId,
    pub card: CardId,
    pub by: Member,
    pub what: What,
}

#[derive(Clone, Debug, PartialEq)]
pub enum What {
    Created { list: ListId },
    MovedIn { list: ListId, from: ListId },
    MovedOut { list: ListId },
    LabelAdded(LabelId),
    LabelRemoved(LabelId),
    MemberAdded(Member),
    MemberRemoved(Member),
    DueSet(Moment),
    DueRemoved,
    DueDone(bool),
    ItemChecked { item: CheckItemId, done: bool },
    ChecklistComplete(ChecklistId),
    Commented(CommentId),
    Archived(bool),
    FieldChanged(FieldId),
}

/// Os alvos de um lote ANTES de ele se aplicar; [`Seen::events`] lê-os de novo DEPOIS.
#[derive(Clone, Debug, Default)]
pub struct Seen {
    before: Vec<(KanbanId, RecordId, Option<Record>)>,
}

impl Seen {
    /// Antes do lote `ops` no Kanban `kanban`.
    #[must_use]
    pub fn ops(set: &KanbanSet, kanban: KanbanId, ops: &[KanbanOp]) -> Self {
        let mut s = Self::default();
        s.add(set, kanban, ops);
        s
    }

    /// Antes de um gesto entre Kanbans.
    #[must_use]
    pub fn plan(set: &KanbanSet, plan: &Plan) -> Self {
        let mut s = Self::default();
        for k in plan.kanbans() {
            s.add(set, k, plan.ops(k));
        }
        s
    }

    fn add(&mut self, set: &KanbanSet, kanban: KanbanId, ops: &[KanbanOp]) {
        let Some(k) = set.get(kanban) else {
            return;
        };
        for id in ops.iter().filter_map(KanbanOp::target) {
            if !self
                .before
                .iter()
                .any(|(kk, i, _)| *kk == kanban && *i == id)
            {
                self.before.push((kanban, id, k.record(id)));
            }
        }
    }

    /// O que mudou, em nome de `by`, pela ordem das ops (sem repetidos).
    #[must_use]
    pub fn events(&self, set: &KanbanSet, by: &Member) -> Vec<Event> {
        let mut out: Vec<Event> = Vec::new();
        for (kanban, id, before) in &self.before {
            let Some(k) = set.get(*kanban) else {
                continue;
            };
            let mut push = |card: CardId, what: What| {
                let e = Event {
                    kanban: *kanban,
                    card,
                    by: by.clone(),
                    what,
                };
                if !out.contains(&e) {
                    out.push(e);
                }
            };
            match (before, k.record(*id)) {
                (b, Some(Record::Card(a))) if !a.meta.deleted && !a.template => {
                    let b = match b {
                        Some(Record::Card(b)) if !b.meta.deleted => Some(b),
                        _ => None,
                    };
                    let Some(b) = b else {
                        push(a.id, What::Created { list: a.list });
                        continue;
                    };
                    if b.list != a.list {
                        push(a.id, What::MovedOut { list: b.list });
                        push(
                            a.id,
                            What::MovedIn {
                                list: a.list,
                                from: b.list,
                            },
                        );
                    }
                    for l in a.labels.iter().filter(|l| !b.labels.contains(l)) {
                        push(a.id, What::LabelAdded(*l));
                    }
                    for l in b.labels.iter().filter(|l| !a.labels.contains(l)) {
                        push(a.id, What::LabelRemoved(*l));
                    }
                    for m in a.members.iter().filter(|m| !b.members.contains(m)) {
                        push(a.id, What::MemberAdded(m.clone()));
                    }
                    for m in b.members.iter().filter(|m| !a.members.contains(m)) {
                        push(a.id, What::MemberRemoved(m.clone()));
                    }
                    match (b.due, a.due) {
                        (_, Some(ad)) if b.due.map(|d| d.at) != Some(ad.at) => {
                            push(a.id, What::DueSet(ad.at));
                        }
                        (Some(_), None) => push(a.id, What::DueRemoved),
                        (Some(bd), Some(ad)) if bd.done != ad.done => {
                            push(a.id, What::DueDone(ad.done));
                        }
                        _ => {}
                    }
                    if b.archived != a.archived {
                        push(a.id, What::Archived(a.archived));
                    }
                    let keys = a.fields.keys().chain(b.fields.keys());
                    let changed: BTreeSet<FieldId> = keys
                        .filter(|f| a.fields.get(f) != b.fields.get(f))
                        .copied()
                        .collect();
                    for f in changed {
                        push(a.id, What::FieldChanged(f));
                    }
                }
                (Some(Record::CheckItem(b)), Some(Record::CheckItem(a)))
                    if !a.meta.deleted && !b.meta.deleted && a.done != b.done =>
                {
                    let Some(card) = k.checklists.get(&a.checklist).map(|c| c.card) else {
                        continue;
                    };
                    push(
                        card,
                        What::ItemChecked {
                            item: a.id,
                            done: a.done,
                        },
                    );
                    let mut items = k
                        .items
                        .values()
                        .filter(|i| i.checklist == a.checklist && !i.meta.deleted);
                    if a.done && items.all(|i| i.done) {
                        push(card, What::ChecklistComplete(a.checklist));
                    }
                }
                (b, Some(Record::Comment(a))) if !a.meta.deleted => {
                    let was = matches!(b, Some(Record::Comment(c)) if !c.meta.deleted);
                    if !was {
                        push(a.card, What::Commented(a.id));
                    }
                }
                _ => {}
            }
        }
        out
    }
}
