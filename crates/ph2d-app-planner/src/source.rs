//! ⭐ **A fonte do Calendário do projecto** — todas as datas do projecto para a vista Calendário da
//! W7 (plano §8.1): início e entrega dos cartões de TODOS os Kanbans, os itens de checklist com data
//! e os eventos próprios da agenda (com as ocorrências das recorrências). Arrastar um item devolve o
//! lote que muda a data DELE — no mesmo documento que o quadro mostra; o de uma ocorrência de uma
//! série fica por responder em [`ProjectSource::asked`] (o alcance, [`crate::scope`]).

use std::cell::RefCell;
use std::collections::BTreeSet;

use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{
    BlockId, Card, CardRef, CheckItemId, Event, EventId, Hue, Kanban, KanbanId, KanbanOp, KanbanSet,
    LabelColor, Member, Moment, Plan, Record, Shade, splitmix64,
};
use ph2d_kanban_views::calendar::Target;
use ph2d_kanban_views::{Ctx, Item, ItemKey, ItemTone, KanbanSource, Source};

use crate::recur;
use crate::scope::{Ask, Change};

/// A cor dos itens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorBy {
    /// A cor do Kanban (o fundo dele) — o de nascença: vários quadros, uma cor cada.
    #[default]
    Kanban,
    /// A primeira etiqueta com cor do cartão.
    Label,
}

/// O que o calendário mostra (da VISTA, nunca do documento).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Show {
    /// Só os itens desta pessoa (membro do cartão, responsável do item, pessoa do evento).
    pub person: Option<Member>,
    /// Kanbans escondidos.
    pub hidden: BTreeSet<KanbanId>,
    /// Só os cartões com uma etiqueta deste nome (os eventos saem).
    pub label: Option<String>,
    pub color_by: ColorBy,
    /// Esconder os eventos próprios.
    pub no_events: bool,
    /// O Planner: os blocos de tempo desta pessoa também.
    pub blocks_of: Option<Member>,
}

/// Os eventos sem cor pintam-se neste matiz (o da agenda).
pub const AGENDA_HUE: Hue = Hue::Sky;

const TAG_ITEM: u64 = 0x4954_454D;
const TAG_EVENT: u64 = 0x4556_454E;
const TAG_BLOCK: u64 = 0x424C_4F43;

/// Os blocos do Planner pintam-se neste matiz.
pub const BLOCK_HUE: Hue = Hue::Purple;

/// A chave de um bloco do Planner.
#[must_use]
pub fn block_key(id: BlockId) -> ItemKey {
    ItemKey::Event(splitmix64(id.0 ^ TAG_BLOCK))
}

/// A chave da ocorrência de `id` que a regra põe em `at` (cada uma a sua: arrasta-se UMA).
#[must_use]
pub fn event_key(id: EventId, at: Moment) -> ItemKey {
    ItemKey::Event(splitmix64(splitmix64(id.0 ^ TAG_EVENT) ^ at.0.cast_unsigned()))
}

/// Até onde se procura a ocorrência de uma chave à volta de um instante: a grelha do mês mostra
/// seis semanas, e nem um arrasto nem um clique saem dela.
pub(crate) const REACH_MS: i64 = 7 * 7 * 86_400_000;

/// A chave de um item de checklist com data.
#[must_use]
pub fn item_key(k: KanbanId, id: CheckItemId) -> ItemKey {
    ItemKey::Event(splitmix64(splitmix64(k.0 ^ TAG_ITEM) ^ id.0))
}

/// A cor de um Kanban: a do fundo, ou uma das dez pela ordem em que nasceu.
#[must_use]
pub fn kanban_hue(set: &KanbanSet, k: &Kanban) -> Hue {
    if let ph2d_kanban_model::Background::Color(h) = k.header.background
        && h != Hue::Blue
    {
        return h;
    }
    let i = set.iter().position(|x| x.id == k.id).unwrap_or(0);
    Hue::ALL[i % Hue::ALL.len()]
}

fn normal(hue: Hue) -> ItemTone {
    ItemTone::Label(LabelColor {
        hue,
        shade: Shade::Normal,
    })
}

/// ⭐ A fonte do projecto inteiro.
pub struct ProjectSource<'a> {
    pub set: &'a KanbanSet,
    pub show: &'a Show,
    pub zone: &'a Zone,
    pub cx: Ctx<'a>,
    /// O arrasto de uma ocorrência de uma série: a pergunta do alcance (o lote fica vazio).
    pub asked: RefCell<Option<Ask>>,
}

impl<'a> ProjectSource<'a> {
    #[must_use]
    pub fn new(set: &'a KanbanSet, show: &'a Show, cx: Ctx<'a>) -> Self {
        Self {
            set,
            show,
            zone: cx.zone,
            cx,
            asked: RefCell::new(None),
        }
    }

    fn label_ok(&self, k: &Kanban, c: &Card) -> bool {
        let Some(want) = &self.show.label else {
            return true;
        };
        c.labels
            .iter()
            .filter_map(|l| k.labels.get(l))
            .any(|l| !l.meta.deleted && l.name.eq_ignore_ascii_case(want))
    }

    fn tone(&self, k: &Kanban, c: &Card) -> ItemTone {
        let label = || {
            c.labels
                .iter()
                .filter_map(|l| k.labels.get(l))
                .filter(|l| !l.meta.deleted)
                .find_map(|l| l.color)
        };
        match (self.show.color_by, label()) {
            (ColorBy::Label, Some(l)) => ItemTone::Label(l),
            _ => normal(kanban_hue(self.set, k)),
        }
    }

    /// Os cartões vivos do Kanban `k` que o filtro deixa ver (pela ordem do quadro).
    fn cards<'k>(&self, k: &'k Kanban) -> Vec<&'k Card> {
        k.lists_in_order()
            .into_iter()
            .flat_map(|l| k.cards_in(l.id))
            .filter(|c| !c.template)
            .filter(|c| self.show.person.as_ref().is_none_or(|p| c.members.contains(p)))
            .filter(|c| self.label_ok(k, c))
            .collect()
    }

    fn card_items(&self, k: &Kanban, out: &mut Vec<Item>) {
        for c in self.cards(k) {
            let Some(end) = c.due.map(|d| d.at).or(c.start) else {
                continue;
            };
            out.push(Item {
                key: ItemKey::Card(CardRef {
                    kanban: k.id,
                    card: c.id,
                }),
                title: c.title.clone(),
                start: c.start.filter(|s| c.due.is_some() && *s < end),
                end,
                done: c.due.is_some_and(|d| d.done),
                tone: self.tone(k, c),
            });
        }
    }

    fn checklist_items(&self, k: &Kanban, out: &mut Vec<Item>) {
        let cards = self.cards(k);
        for it in k.items.values().filter(|i| !i.meta.deleted) {
            let Some(due) = it.due else { continue };
            let Some(card) = k
                .checklists
                .get(&it.checklist)
                .filter(|cl| !cl.meta.deleted)
                .and_then(|cl| cards.iter().find(|c| c.id == cl.card))
            else {
                continue;
            };
            if let Some(p) = &self.show.person
                && it.member.as_ref() != Some(p)
                && !card.members.contains(p)
            {
                continue;
            }
            out.push(Item {
                key: item_key(k.id, it.id),
                title: it.text.clone(),
                start: None,
                end: due,
                done: it.done,
                tone: self.tone(k, card),
            });
        }
    }

    fn event_items(&self, from: Moment, to: Moment, out: &mut Vec<Item>) {
        if self.show.no_events || self.show.label.is_some() {
            return;
        }
        for e in self.events() {
            if let Some(p) = &self.show.person
                && !e.people.contains(p)
            {
                continue;
            }
            for (s, end) in recur::occurrences(e, from, to, self.zone) {
                out.push(Item {
                    key: event_key(e.id, s),
                    title: e.title.clone(),
                    start: Some(s),
                    // O dia inteiro termina no último dia (o `end` do evento é exclusivo).
                    end: if e.all_day { Moment(end.0 - 1) } else { end },
                    done: false,
                    tone: normal(e.color.unwrap_or(AGENDA_HUE)),
                });
            }
        }
    }

    /// Os eventos vivos da agenda e dos Kanbans (os de uma sessão também).
    fn events(&self) -> impl Iterator<Item = &'a Event> {
        std::iter::once(self.set.agenda())
            .chain(self.set.iter())
            .flat_map(|k| k.events.values())
            .filter(|e| !e.meta.deleted)
    }

    fn visible(&self) -> impl Iterator<Item = &'a Kanban> {
        self.set
            .iter()
            .filter(|k| !self.show.hidden.contains(&k.id))
    }

    /// A ocorrência da chave `key` perto de `near`: o Kanban, o evento e o início dela.
    pub fn event_at(&self, key: ItemKey, near: Moment) -> Option<(KanbanId, &'a Event, Moment)> {
        let (from, to) = (Moment(near.0 - REACH_MS), Moment(near.0 + REACH_MS));
        std::iter::once(self.set.agenda())
            .chain(self.set.iter())
            .flat_map(|k| k.events.values().map(move |e| (k.id, e)))
            .filter(|(_, e)| !e.meta.deleted)
            .find_map(|(k, e)| {
                recur::occurrences(e, from, to, self.zone)
                    .into_iter()
                    .find(|(s, _)| event_key(e.id, *s) == key)
                    .map(|(s, _)| (k, e, s))
            })
    }

    fn item_at(&self, key: ItemKey) -> Option<(KanbanId, CheckItemId)> {
        self.set.iter().find_map(|k| {
            k.items
                .keys()
                .find(|i| item_key(k.id, **i) == key)
                .map(|i| (k.id, *i))
        })
    }
}

impl Source for ProjectSource<'_> {
    fn items(&self, from: Moment, to: Moment) -> Vec<Item> {
        let mut out = Vec::new();
        for k in self.visible() {
            self.card_items(k, &mut out);
            self.checklist_items(k, &mut out);
        }
        self.event_items(from, to, &mut out);
        if let Some(me) = &self.show.blocks_of {
            for b in crate::personal::blocks_of(self.set, me, from, to) {
                out.push(Item {
                    key: block_key(b.id),
                    title: crate::personal::label_of(self.set, b.of).unwrap_or_default(),
                    start: Some(b.start),
                    end: Moment(b.start.0 + i64::from(b.minutes) * 60_000),
                    done: false,
                    tone: normal(BLOCK_HUE),
                });
            }
        }
        out.retain(|i| i.start.unwrap_or(i.end) < to && i.end >= from);
        out
    }

    fn reschedule(&self, key: ItemKey, start: Option<Moment>, end: Moment) -> Plan {
        let mut plan = Plan::default();
        match key {
            ItemKey::Card(r) => return KanbanSource::new(self.set, r.kanban, &self.cx)
                .reschedule(key, start, end),
            ItemKey::Event(_) => {}
        }
        if let Some(b) = self
            .set
            .agenda()
            .blocks
            .values()
            .find(|b| !b.meta.deleted && block_key(b.id) == key)
        {
            let start = start.unwrap_or(b.start);
            let minutes = u32::try_from((end.0 - start.0) / 60_000).unwrap_or(b.minutes);
            return crate::personal::move_block(self.set, b.id, start, minutes);
        }
        if let Some((k, e, at)) = self.event_at(key, start.unwrap_or(end)) {
            // A vista dá o fim do dia inteiro no último dia (o do evento é exclusivo).
            let to = start.unwrap_or(end);
            let len = e.end.0 - e.start.0;
            let end = if e.all_day { Moment(end.0 + 1) } else { end };
            let end = if start.is_some() && end > to { end } else { Moment(to.0 + len) };
            if e.repeat.is_some() {
                *self.asked.borrow_mut() = Some(Ask {
                    kanban: k,
                    series: e.id,
                    at,
                    change: Change::Move { start: to, end },
                });
                return plan;
            }
            let mut n = e.clone();
            (n.start, n.end) = (to, end);
            if n != *e {
                plan.push(k, KanbanOp::put(Record::Event(n)));
            }
        } else if let Some((k, id)) = self.item_at(key) {
            let it = &self.set.get(k).expect("achado acima").items[&id];
            let mut n = it.clone();
            n.due = Some(end);
            if n != *it {
                plan.push(k, KanbanOp::put(Record::CheckItem(n)));
            }
        }
        plan
    }

    fn targets(&self) -> Vec<Target> {
        self.visible()
            .flat_map(|k| {
                KanbanSource::new(self.set, k.id, &self.cx)
                    .targets()
                    .into_iter()
                    .map(|mut t| {
                        t.name = ph2d_i18n::tr_with(
                            "planner.target",
                            &[("kanban", &k.header.name), ("list", &t.name)],
                        );
                        t
                    })
            })
            .collect()
    }

    fn create(&self, at: &Target, title: &str, due: Moment) -> Plan {
        KanbanSource::new(self.set, at.list.kanban, &self.cx).create(at, title, due)
    }
}
