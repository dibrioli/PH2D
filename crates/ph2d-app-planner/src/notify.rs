//! ⭐ **As notificações** (o sino do Trello) — DERIVADAS do documento, nunca guardadas: menções a
//! mim nos comentários, cartões a que me juntaram, mudanças nos cartões/listas/Kanbans que sigo (a
//! [`ph2d_kanban_model::Activity`]) e lembretes a tocar (das entregas e dos eventos). O que é guardado
//! é só o que EU já li ([`ph2d_kanban_model::Seen`], pessoal, na agenda).

use ph2d_kanban_dates::{Zone, reminder_due};
use ph2d_kanban_model::{
    Card, CardId, CommentId, Happened, Kanban, KanbanId, KanbanOp, KanbanSet, Member, Meta, Moment,
    Record, Seen, SeenId, Subject, splitmix64,
};

use crate::recur;

/// O que aconteceu.
#[derive(Clone, Debug, PartialEq)]
pub enum What {
    /// `@eu` num comentário.
    Mention { comment: CommentId, text: String },
    /// Juntaram-me ao cartão.
    Assigned,
    /// Um cartão (ou lista, ou Kanban) que sigo mudou.
    Changed(Happened),
    /// O lembrete da entrega do cartão.
    DueReminder { due: Moment },
    /// O lembrete de um evento da agenda.
    EventReminder { title: String, start: Moment },
}

/// Uma notificação: a chave estável (para «lida»), a hora, quem, onde e o quê.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub key: u64,
    pub at: Moment,
    pub by: Option<Member>,
    pub kanban: KanbanId,
    pub card: Option<CardId>,
    /// O título do cartão (ou do Kanban) — o «em …» da linha.
    pub place: String,
    pub what: What,
    pub unread: bool,
}

impl Notice {
    /// Um lembrete (o que pede um aviso na hora).
    #[must_use]
    pub fn is_reminder(&self) -> bool {
        matches!(
            self.what,
            What::DueReminder { .. } | What::EventReminder { .. }
        )
    }
}

fn key(parts: &[u64]) -> u64 {
    parts.iter().fold(0x4E4F_5449, |h, p| splitmix64(h ^ p))
}

/// `text` menciona `who` (`@Nome`, sem distinguir maiúsculas, seguido de fim ou de um não-alfanumérico).
#[must_use]
pub fn mentions(text: &str, who: &Member) -> bool {
    let name = who.0.trim().to_lowercase();
    if name.is_empty() {
        return false;
    }
    let lower = text.to_lowercase();
    lower.match_indices('@').any(|(i, _)| {
        let rest = &lower[i + 1..];
        rest.starts_with(&name)
            && rest[name.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric())
    })
}

/// Sigo o cartão `c` (ele, a lista dele ou o Kanban)?
fn follows(k: &Kanban, c: &Card, me: &Member) -> bool {
    c.watchers.contains(me)
        || k.lists
            .get(&c.list)
            .is_some_and(|l| l.watchers.contains(me))
        || k.header.watchers.contains(me)
}

fn card_notices(k: &Kanban, me: &Member, out: &mut Vec<Notice>) {
    let place = |id: CardId| {
        k.cards
            .get(&id)
            .map_or_else(String::new, |c| c.title.clone())
    };
    for cm in k.comments.values().filter(|c| !c.meta.deleted) {
        if &cm.author != me && mentions(&cm.text, me) {
            out.push(Notice {
                key: key(&[1, k.id.0, cm.id.0]),
                at: cm.at,
                by: Some(cm.author.clone()),
                kanban: k.id,
                card: Some(cm.card),
                place: place(cm.card),
                what: What::Mention {
                    comment: cm.id,
                    text: cm.text.clone(),
                },
                unread: true,
            });
        }
    }
    for a in k
        .activities
        .values()
        .filter(|a| !a.meta.deleted && &a.by != me)
    {
        let Subject::Card(cid) = a.subject else {
            if a.subject == Subject::Kanban && k.header.watchers.contains(me) {
                out.push(Notice {
                    key: key(&[3, k.id.0, a.id.0]),
                    at: a.at,
                    by: Some(a.by.clone()),
                    kanban: k.id,
                    card: None,
                    place: k.header.name.clone(),
                    what: What::Changed(a.what.clone()),
                    unread: true,
                });
            }
            continue;
        };
        let Some(c) = k.cards.get(&cid) else { continue };
        let what = match &a.what {
            Happened::MemberAdded(m) if m == me => What::Assigned,
            _ if follows(k, c, me) => What::Changed(a.what.clone()),
            _ => continue,
        };
        out.push(Notice {
            key: key(&[2, k.id.0, a.id.0]),
            at: a.at,
            by: Some(a.by.clone()),
            kanban: k.id,
            card: Some(cid),
            place: c.title.clone(),
            what,
            unread: true,
        });
    }
}

fn reminders(set: &KanbanSet, me: &Member, now: Moment, zone: &Zone, out: &mut Vec<Notice>) {
    for k in set.iter() {
        for c in k.cards.values() {
            let Some(d) = c.due else { continue };
            let Some(before) = d.remind_before_min else {
                continue;
            };
            let mine = c.members.contains(me) || c.watchers.contains(me);
            if c.meta.deleted
                || c.archived
                || d.done
                || !mine
                || !reminder_due(d.at.0, before, now.0)
            {
                continue;
            }
            out.push(Notice {
                key: key(&[4, k.id.0, c.id.0, d.at.0.cast_unsigned(), before.into()]),
                at: Moment(ph2d_kanban_dates::remind_at(d.at.0, before)),
                by: None,
                kanban: k.id,
                card: Some(c.id),
                place: c.title.clone(),
                what: What::DueReminder { due: d.at },
                unread: true,
            });
        }
    }
    let agenda = set.agenda();
    for e in agenda.events.values().filter(|e| !e.meta.deleted) {
        let Some(before) = e.remind_before_min else {
            continue;
        };
        if !e.people.is_empty() && !e.people.contains(me) {
            continue;
        }
        // A ocorrência que está a começar (ou a última que começou há menos de um dia).
        let from = Moment(now.0 - 86_400_000);
        let Some(start) = recur::next_start(e, from, zone) else {
            continue;
        };
        if !reminder_due(start.0, before, now.0) {
            continue;
        }
        out.push(Notice {
            key: key(&[5, e.id.0, start.0.cast_unsigned(), before.into()]),
            at: Moment(ph2d_kanban_dates::remind_at(start.0, before)),
            by: None,
            kanban: KanbanId::AGENDA,
            card: None,
            place: e.title.clone(),
            what: What::EventReminder {
                title: e.title.clone(),
                start,
            },
            unread: true,
        });
    }
}

/// ⭐ As notificações de `me` agora, da mais nova para a mais antiga, com «não lida» do registo dele.
#[must_use]
pub fn notices(set: &KanbanSet, me: &Member, now: Moment, zone: &Zone) -> Vec<Notice> {
    let mut out = Vec::new();
    for k in set.iter() {
        card_notices(k, me, &mut out);
    }
    reminders(set, me, now, zone, &mut out);
    let seen = set.agenda().seen_of(me);
    for n in &mut out {
        n.unread = seen.is_none_or(|s| n.at > s.before && !s.read.contains(&n.key));
    }
    out.sort_by(|a, b| b.at.cmp(&a.at).then(a.key.cmp(&b.key)));
    out
}

fn seen_or_new(agenda: &Kanban, me: &Member) -> Seen {
    agenda.seen_of(me).cloned().unwrap_or_else(|| Seen {
        id: SeenId(agenda.mint()),
        owner: me.clone(),
        before: Moment(i64::MIN),
        read: Vec::new(),
        meta: Meta::default(),
    })
}

/// Marca a notificação `key` como lida (o op na agenda; `None` = já estava).
#[must_use]
pub fn mark_read(set: &KanbanSet, me: &Member, key: u64) -> Option<KanbanOp> {
    let mut s = seen_or_new(set.agenda(), me);
    if s.read.contains(&key) {
        return None;
    }
    s.read.push(key);
    Some(KanbanOp::put(Record::Seen(s)))
}

/// «Marcar todas como lidas»: tudo até `now`; as chaves soltas saem (já estão debaixo da linha).
#[must_use]
pub fn mark_all_read(set: &KanbanSet, me: &Member, now: Moment) -> KanbanOp {
    let mut s = seen_or_new(set.agenda(), me);
    s.before = now;
    s.read.clear();
    KanbanOp::put(Record::Seen(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mention_is_the_name_after_an_at_and_never_a_longer_name() {
        let ana = Member("Ana".into());
        assert!(mentions("olá @ana, vê isto", &ana));
        assert!(mentions("@Ana", &ana));
        assert!(!mentions("@Anabela vê", &ana));
        assert!(!mentions("ana sem arroba", &ana));
        assert!(mentions("cc @Ana Maria", &Member("Ana Maria".into())));
    }
}
