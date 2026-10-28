//! ⭐ **O que é de UMA pessoa** (o «Pessoal» do Trello de 2025): a Inbox (captura rápida), os blocos
//! de tempo do Planner e «My cards». Cada gesto é um [`Plan`] — UM passo de desfazer, mesmo quando
//! toca a agenda e um Kanban (a Inbox que vira cartão).

use ph2d_kanban_dates::{CivilDate, Zone, day_of};
use ph2d_kanban_model::{
    Block, BlockId, BlockOf, Card, CardId, CardRef, Due, InboxId, InboxItem, KanbanId, KanbanOp,
    KanbanSet, ListRef, Member, Meta, Moment, Plan, Record, RecordId,
};

const A: KanbanId = KanbanId::AGENDA;

/// Um item novo na Inbox de `me` (texto vazio = nada).
#[must_use]
pub fn capture(set: &KanbanSet, me: &Member, text: &str, now: Moment) -> Plan {
    let mut plan = Plan::default();
    let text = text.trim();
    if text.is_empty() {
        return plan;
    }
    let a = set.agenda();
    plan.push(
        A,
        KanbanOp::put(Record::Inbox(InboxItem {
            id: InboxId(a.mint()),
            owner: me.clone(),
            text: text.to_owned(),
            created: now,
            pos: a.pos_after_inbox(),
            meta: Meta::default(),
        })),
    );
    plan
}

/// ⭐ «Mover para um Kanban»: o item vira um cartão no FIM da lista `to` (com `me` de membro, como
/// no Trello), sai da Inbox, e os blocos do Planner que o tinham passam a ter o cartão.
#[must_use]
pub fn inbox_to_card(set: &KanbanSet, item: InboxId, to: ListRef, now: Moment) -> Option<Plan> {
    let a = set.agenda();
    let it = a.inbox.get(&item).filter(|i| !i.meta.deleted)?;
    let k = set.get(to.kanban)?;
    k.lists.get(&to.list).filter(|l| !l.meta.deleted)?;
    let mut card = Card::new(
        CardId(k.mint()),
        to.list,
        k.pos_after_cards(to.list),
        &it.text,
        now,
    );
    card.members.push(it.owner.clone());
    let r = CardRef {
        kanban: to.kanban,
        card: card.id,
    };
    let mut plan = Plan::default();
    plan.push(to.kanban, KanbanOp::put(Record::Card(card)));
    plan.push(A, KanbanOp::Delete(RecordId::Inbox(item)));
    for b in a.blocks.values() {
        if !b.meta.deleted && b.of == BlockOf::Inbox(item) {
            let mut n = b.clone();
            n.of = BlockOf::Card(r);
            plan.push(A, KanbanOp::put(Record::Block(n)));
        }
    }
    Some(plan)
}

/// Põe `of` num bloco de tempo do Planner de `me`.
#[must_use]
pub fn schedule(set: &KanbanSet, me: &Member, of: BlockOf, start: Moment, minutes: u32) -> Plan {
    let a = set.agenda();
    let mut plan = Plan::default();
    plan.push(
        A,
        KanbanOp::put(Record::Block(Block {
            id: BlockId(a.mint()),
            owner: me.clone(),
            of,
            start,
            minutes: minutes.max(1),
            meta: Meta::default(),
        })),
    );
    plan
}

/// Move (ou estica) um bloco.
#[must_use]
pub fn move_block(set: &KanbanSet, id: BlockId, start: Moment, minutes: u32) -> Plan {
    let mut plan = Plan::default();
    if let Some(b) = set.agenda().blocks.get(&id).filter(|b| !b.meta.deleted) {
        let mut n = b.clone();
        (n.start, n.minutes) = (start, minutes.max(1));
        if n != *b {
            plan.push(A, KanbanOp::put(Record::Block(n)));
        }
    }
    plan
}

/// Os blocos vivos de `me` que tocam `[from, to)`, pela hora.
#[must_use]
pub fn blocks_of<'a>(set: &'a KanbanSet, me: &Member, from: Moment, to: Moment) -> Vec<&'a Block> {
    let mut v: Vec<&Block> = set
        .agenda()
        .blocks
        .values()
        .filter(|b| !b.meta.deleted && &b.owner == me)
        .filter(|b| b.start < to && b.start.0 + i64::from(b.minutes) * 60_000 > from.0)
        .filter(|b| label_of(set, b.of).is_some())
        .collect();
    v.sort_by_key(|b| (b.start, b.id));
    v
}

/// O texto de um bloco: o do item ou o título do cartão (`None` = já não existe vivo).
#[must_use]
pub fn label_of(set: &KanbanSet, of: BlockOf) -> Option<String> {
    match of {
        BlockOf::Inbox(i) => set
            .agenda()
            .inbox
            .get(&i)
            .filter(|i| !i.meta.deleted)
            .map(|i| i.text.clone()),
        BlockOf::Card(r) => set.card(r).filter(|c| live(c)).map(|c| c.title.clone()),
    }
}

fn live(c: &Card) -> bool {
    !c.meta.deleted && !c.archived && !c.template
}

/// Os cartões vivos onde `me` é membro, de todos os Kanbans, pela ordem do quadro.
#[must_use]
pub fn mine<'a>(set: &'a KanbanSet, me: &Member) -> Vec<(CardRef, &'a Card)> {
    set.iter()
        .flat_map(|k| {
            k.lists_in_order()
                .into_iter()
                .flat_map(move |l| k.cards_in(l.id))
                .filter(|c| live(c) && c.members.contains(me))
                .map(move |c| {
                    (
                        CardRef {
                            kanban: k.id,
                            card: c.id,
                        },
                        c,
                    )
                })
        })
        .collect()
}

/// A coluna do Planner: os meus cartões SEM data e sem bloco (o resto já está no calendário).
#[must_use]
pub fn undated_mine<'a>(set: &'a KanbanSet, me: &Member) -> Vec<(CardRef, &'a Card)> {
    let blocked: Vec<CardRef> = set
        .agenda()
        .blocks
        .values()
        .filter(|b| !b.meta.deleted && &b.owner == me)
        .filter_map(|b| match b.of {
            BlockOf::Card(r) => Some(r),
            BlockOf::Inbox(_) => None,
        })
        .collect();
    mine(set, me)
        .into_iter()
        .filter(|(r, c)| c.start.is_none() && c.due.is_none() && !blocked.contains(r))
        .collect()
}

/// Como «My cards» agrupa (o Trello: por quadro ou por data).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupBy {
    #[default]
    Kanban,
    Date,
}

/// Os grupos por data (a ordem em que se mostram).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateGroup {
    Overdue,
    Today,
    Tomorrow,
    ThisWeek,
    Later,
    NoDate,
    Done,
}

impl DateGroup {
    /// A chave da i18n do título.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Overdue => "planner.group.overdue",
            Self::Today => "planner.group.today",
            Self::Tomorrow => "planner.group.tomorrow",
            Self::ThisWeek => "planner.group.week",
            Self::Later => "planner.group.later",
            Self::NoDate => "planner.group.no_date",
            Self::Done => "planner.group.done",
        }
    }
}

fn date_group(d: Option<Due>, now: Moment, zone: &Zone) -> DateGroup {
    let Some(d) = d else {
        return DateGroup::NoDate;
    };
    if d.done {
        return DateGroup::Done;
    }
    if d.at < now {
        return DateGroup::Overdue;
    }
    let today: CivilDate = day_of(now.0, zone);
    match today.days_until(day_of(d.at.0, zone)) {
        0 => DateGroup::Today,
        1 => DateGroup::Tomorrow,
        2..=6 => DateGroup::ThisWeek,
        _ => DateGroup::Later,
    }
}

/// Um grupo de «My cards»: o título e os cartões.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub title: String,
    pub cards: Vec<CardRef>,
}

/// ⭐ «My cards» agrupado: por Kanban → lista (a ordem do quadro), ou por data (atrasados primeiro,
/// a entrega mais cedo primeiro dentro do grupo).
#[must_use]
pub fn my_cards(set: &KanbanSet, me: &Member, by: GroupBy, now: Moment, zone: &Zone) -> Vec<Group> {
    let cards = mine(set, me);
    let mut out: Vec<Group> = Vec::new();
    match by {
        GroupBy::Kanban => {
            for (r, c) in cards {
                let k = set.get(r.kanban).expect("o cartão veio dele");
                let list = k.lists.get(&c.list).map_or("", |l| l.title.as_str());
                let title =
                    ph2d_i18n::tr_with("planner.target", &[("kanban", &k.header.name), ("list", &list)]);
                match out.last_mut() {
                    Some(g) if g.title == title => g.cards.push(r),
                    _ => out.push(Group {
                        title,
                        cards: vec![r],
                    }),
                }
            }
        }
        GroupBy::Date => {
            let mut keyed: Vec<(DateGroup, Moment, CardRef)> = cards
                .into_iter()
                .map(|(r, c)| {
                    let at = c.due.map_or(Moment(i64::MAX), |d| d.at);
                    (date_group(c.due, now, zone), at, r)
                })
                .collect();
            keyed.sort();
            for (g, _, r) in keyed {
                let title = ph2d_i18n::tr(g.key()).to_owned();
                match out.last_mut() {
                    Some(x) if x.title == title => x.cards.push(r),
                    _ => out.push(Group {
                        title,
                        cards: vec![r],
                    }),
                }
            }
        }
    }
    out
}
