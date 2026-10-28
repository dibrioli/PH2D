//! **A atividade** (v2) — a história de um Kanban, o «Atividade» do verso do cartão no Trello.
//!
//! Nasce das [`KanbanOp`] aplicadas, nunca à mão: [`activity_for`] lê o alvo de uma op ANTES e
//! DEPOIS e diz o que aconteceu (pura e determinística); [`apply_logged`] aplica um lote e grava uma
//! [`Activity`] por acontecimento.
//!
//! **O desfazer não apaga a história** (a lei do Trello: reverter é outra ação, e a atividade nunca
//! se reescreve). As entradas ficam FORA do lote inverso; desfazer pelo [`crate::History`] com
//! `undo_logged` regista o que o desfazer fez (desfazer «moveu A→B» regista «moveu B→A»).
//!
//! Comentários não entram aqui: são registos próprios e a vista intercala-os pela hora.

use serde::{Deserialize, Serialize};

use crate::kanban::{Header, Kanban};
use crate::ops::{KanbanOp, Record, RecordId};
use crate::records::{
    ActivityId, AttachmentId, Card, CardId, CheckItemId, ChecklistId, List, ListId, Member, Meta,
    Moment,
};

/// De quem é a entrada: o Kanban, uma lista ou um cartão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Subject {
    Kanban,
    List(ListId),
    Card(CardId),
}

/// O que aconteceu.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Happened {
    /// Uma lista nasceu (`list: None`) ou um cartão nasceu na lista `list`.
    Created {
        list: Option<ListId>,
    },
    /// O cartão nasceu de um item da checklist do cartão `from`.
    Converted {
        from: CardId,
    },
    Moved {
        from: ListId,
        to: ListId,
    },
    /// Título do cartão, da lista ou nome do Kanban.
    Renamed {
        from: String,
    },
    DescriptionChanged,
    StartSet(Option<Moment>),
    DueSet(Option<Moment>),
    DueDone(bool),
    MemberAdded(Member),
    MemberRemoved(Member),
    Archived,
    Unarchived,
    Deleted,
    /// Um apagado voltou (o desfazer de um apagar).
    Revived,
    ChecklistAdded {
        checklist: ChecklistId,
        title: String,
    },
    ChecklistRemoved {
        checklist: ChecklistId,
        title: String,
    },
    ItemChecked {
        item: CheckItemId,
        text: String,
        done: bool,
    },
    Attached {
        attachment: AttachmentId,
        name: String,
    },
    Detached {
        attachment: AttachmentId,
        name: String,
    },
}

/// Uma entrada da atividade: quem, quando, de quem e o quê.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    pub id: ActivityId,
    pub subject: Subject,
    pub by: Member,
    pub at: Moment,
    pub what: Happened,
    pub meta: Meta,
}

/// Quem faz o gesto e a hora do relógio de parede (a crate não tem relógio).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub by: Member,
    pub at: Moment,
}

/// O alvo de uma op num instante: o registo (`None` = nunca existiu) ou o cabeçalho.
#[derive(Clone, Debug, PartialEq)]
pub enum Snapshot {
    Record(Option<Record>),
    Header(Header),
}

impl Snapshot {
    /// O alvo `target` de uma op ([`KanbanOp::target`]) tal como está em `k`.
    #[must_use]
    pub fn at(target: Option<RecordId>, k: &Kanban) -> Self {
        match target {
            Some(id) => Self::Record(k.record(id)),
            None => Self::Header(k.header.clone()),
        }
    }
}

/// O que uma op fez, lido do alvo `before` e `after` (`k` = o documento DEPOIS, para achar o cartão
/// de um item). Vazio = nada que o Trello registe (etiquetas, ordem, campos, comentários).
#[must_use]
pub fn activity_for(before: &Snapshot, after: &Snapshot, k: &Kanban) -> Vec<(Subject, Happened)> {
    let mut out = Vec::new();
    match (before, after) {
        (Snapshot::Header(b), Snapshot::Header(a)) if b.name != a.name => out.push((
            Subject::Kanban,
            Happened::Renamed {
                from: b.name.clone(),
            },
        )),
        (Snapshot::Record(b), Snapshot::Record(Some(a))) => record(b.as_ref(), a, k, &mut out),
        _ => {}
    }
    out
}

fn record(b: Option<&Record>, a: &Record, k: &Kanban, out: &mut Vec<(Subject, Happened)>) {
    match a {
        Record::Card(a) => {
            let b = b.and_then(|r| match r {
                Record::Card(c) => Some(c),
                _ => None,
            });
            card(b, a, out);
        }
        Record::List(a) => {
            let b = b.and_then(|r| match r {
                Record::List(l) => Some(l),
                _ => None,
            });
            list(b, a, out);
        }
        Record::Checklist(a) => {
            let was = b.is_some_and(|r| !meta(r).deleted);
            let s = Subject::Card(a.card);
            let (checklist, title) = (a.id, a.title.clone());
            match (was, a.meta.deleted) {
                (false, false) => out.push((s, Happened::ChecklistAdded { checklist, title })),
                (true, true) => out.push((s, Happened::ChecklistRemoved { checklist, title })),
                _ => {}
            }
        }
        Record::CheckItem(a) => {
            let Some(Record::CheckItem(b)) = b else {
                return;
            };
            let card = k.checklists.get(&a.checklist).map(|c| c.card);
            if let Some(card) = card
                && !b.meta.deleted
                && !a.meta.deleted
                && b.done != a.done
            {
                out.push((
                    Subject::Card(card),
                    Happened::ItemChecked {
                        item: a.id,
                        text: a.text.clone(),
                        done: a.done,
                    },
                ));
            }
        }
        Record::Attachment(a) => {
            let was = b.is_some_and(|r| !meta(r).deleted);
            let s = Subject::Card(a.card);
            let (attachment, name) = (a.id, a.name.clone());
            match (was, a.meta.deleted) {
                (false, false) => out.push((s, Happened::Attached { attachment, name })),
                (true, true) => out.push((s, Happened::Detached { attachment, name })),
                _ => {}
            }
        }
        Record::Label(_)
        | Record::Comment(_)
        | Record::Field(_)
        | Record::Activity(_)
        | Record::Mirror(_)
        | Record::Automation(_) => {}
    }
}

fn meta(r: &Record) -> Meta {
    match r {
        Record::List(r) => r.meta,
        Record::Card(r) => r.meta,
        Record::Label(r) => r.meta,
        Record::Checklist(r) => r.meta,
        Record::CheckItem(r) => r.meta,
        Record::Comment(r) => r.meta,
        Record::Attachment(r) => r.meta,
        Record::Field(r) => r.meta,
        Record::Activity(r) => r.meta,
        Record::Mirror(r) => r.meta,
        Record::Automation(r) => r.meta,
    }
}

/// Nascer, morrer, reviver. `true` = o registo estava vivo antes e continua (há diferenças a ler).
fn birth(was: Option<bool>, deleted: bool, created: Happened, out: &mut Vec<Happened>) -> bool {
    match (was, deleted) {
        (Some(true), false) => return true,
        (Some(true), true) => out.push(Happened::Deleted),
        (Some(false), false) => out.push(Happened::Revived),
        (None, false) => out.push(created),
        _ => {}
    }
    false
}

fn card(b: Option<&Card>, a: &Card, out: &mut Vec<(Subject, Happened)>) {
    let mut h = Vec::new();
    let created = match a.origin {
        Some(from) => Happened::Converted { from },
        None => Happened::Created { list: Some(a.list) },
    };
    let alive = birth(b.map(|b| !b.meta.deleted), a.meta.deleted, created, &mut h);
    if let (true, Some(b)) = (alive, b) {
        if b.list != a.list {
            h.push(Happened::Moved {
                from: b.list,
                to: a.list,
            });
        }
        if b.title != a.title {
            h.push(Happened::Renamed {
                from: b.title.clone(),
            });
        }
        if b.description != a.description {
            h.push(Happened::DescriptionChanged);
        }
        if b.start != a.start {
            h.push(Happened::StartSet(a.start));
        }
        let (bat, aat) = (b.due.map(|d| d.at), a.due.map(|d| d.at));
        if bat != aat {
            h.push(Happened::DueSet(aat));
        } else if let (Some(bd), Some(ad)) = (b.due, a.due)
            && bd.done != ad.done
        {
            h.push(Happened::DueDone(ad.done));
        }
        for m in a.members.iter().filter(|m| !b.members.contains(m)) {
            h.push(Happened::MemberAdded(m.clone()));
        }
        for m in b.members.iter().filter(|m| !a.members.contains(m)) {
            h.push(Happened::MemberRemoved(m.clone()));
        }
        archive(b.archived, a.archived, &mut h);
    }
    out.extend(h.into_iter().map(|w| (Subject::Card(a.id), w)));
}

fn list(b: Option<&List>, a: &List, out: &mut Vec<(Subject, Happened)>) {
    let mut h = Vec::new();
    let created = Happened::Created { list: None };
    let alive = birth(b.map(|b| !b.meta.deleted), a.meta.deleted, created, &mut h);
    if let (true, Some(b)) = (alive, b) {
        if b.title != a.title {
            h.push(Happened::Renamed {
                from: b.title.clone(),
            });
        }
        archive(b.archived, a.archived, &mut h);
    }
    out.extend(h.into_iter().map(|w| (Subject::List(a.id), w)));
}

fn archive(before: bool, after: bool, h: &mut Vec<Happened>) {
    match (before, after) {
        (false, true) => h.push(Happened::Archived),
        (true, false) => h.push(Happened::Unarchived),
        _ => {}
    }
}

/// Aplica um lote como [`crate::apply_batch`] e grava a atividade dele em nome de `actor`. Devolve
/// o lote inverso SEM as entradas da atividade: desfazer não apaga a história.
pub fn apply_logged(k: &mut Kanban, ops: Vec<KanbanOp>, actor: &Actor) -> Vec<KanbanOp> {
    let mut inverse = Vec::new();
    let mut log = Vec::new();
    for op in ops {
        let target = op.target();
        let before = Snapshot::at(target, k);
        if let Some(inv) = op.apply(k) {
            log.extend(activity_for(&before, &Snapshot::at(target, k), k));
            inverse.push(inv);
        }
    }
    inverse.reverse();
    for (subject, what) in log {
        let entry = Activity {
            id: ActivityId(k.mint()),
            subject,
            by: actor.by.clone(),
            at: actor.at,
            what,
            meta: Meta::default(),
        };
        let _ = KanbanOp::put(Record::Activity(entry)).apply(k);
    }
    inverse
}

impl Kanban {
    /// A atividade viva de `subject`, da mais antiga para a mais nova (a hora; o id desempata).
    #[must_use]
    pub fn activity_of(&self, subject: Subject) -> Vec<&Activity> {
        let mut v: Vec<&Activity> = self
            .activities
            .values()
            .filter(|a| a.subject == subject && !a.meta.deleted)
            .collect();
        v.sort_by(|a, b| a.at.cmp(&b.at).then(a.id.cmp(&b.id)));
        v
    }
}

#[cfg(test)]
#[path = "activity_tests.rs"]
mod tests;
