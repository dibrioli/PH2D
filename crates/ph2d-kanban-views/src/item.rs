//! ⭐ **O que um calendário mostra** — itens genéricos com datas, de uma [`Source`]. A vista
//! Calendário de UM Kanban lê a [`KanbanSource`]; o Calendário do projecto (W9, plano §8.1) junta
//! todos os Kanbans e os eventos próprios noutra `Source`, sem reescrever a vista.

use ph2d_kanban_model::{
    Card, CardRef, Due, FracKey, Hue, KanbanId, KanbanOp, KanbanSet, LabelColor, ListRef, Meta,
    Moment, Plan, Record,
};
use ph2d_tokens::ColorToken;

use crate::common::{Ctx, visible_cards};

/// De onde vem um item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKey {
    Card(CardRef),
    /// Um evento que não é cartão (a W9: reunião, marco, feriado), pela id da `Source` dele.
    Event(u64),
}

/// A cor de um item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemTone {
    /// A primeira etiqueta do cartão.
    Label(LabelColor),
    /// A cor da lista (`None` = sem cor).
    List(Option<Hue>),
    /// Um token dado pela `Source` (os eventos próprios da W9).
    Token(ColorToken),
}

/// Um item com datas: um dia (`start == None`) ou um intervalo `start → end`.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub key: ItemKey,
    pub title: String,
    pub start: Option<Moment>,
    /// A entrega (ou o próprio dia, se o item só tem início).
    pub end: Moment,
    pub done: bool,
    pub tone: ItemTone,
}

/// Um sítio onde um item novo pode nascer (uma lista de um Kanban, com o nome à vista).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub list: ListRef,
    pub name: String,
}

/// ⭐ A fonte de um calendário.
pub trait Source {
    /// Os itens que tocam `[from, to)`.
    fn items(&self, from: Moment, to: Moment) -> Vec<Item>;
    /// Mudar as datas de `key` (o arrasto): o lote a aplicar.
    fn reschedule(&self, key: ItemKey, start: Option<Moment>, end: Moment) -> Plan;
    /// Onde um item novo pode nascer (o «+» de um dia); o primeiro é a escolha por omissão.
    fn targets(&self) -> Vec<Target>;
    /// Um cartão novo com o título `title` e a entrega `due`, em `at`.
    fn create(&self, at: &Target, title: &str, due: Moment) -> Plan;
    /// Um cartão novo com início E entrega (o bloco arrastado na grelha de horas). Por omissão,
    /// só a entrega.
    fn create_span(&self, at: &Target, title: &str, start: Option<Moment>, due: Moment) -> Plan {
        let _ = start;
        self.create(at, title, due)
    }
}

/// A fonte de UM Kanban: os cartões à vista (o filtro da aba) com início ou entrega.
pub struct KanbanSource<'a> {
    set: &'a KanbanSet,
    kanban: KanbanId,
    cx: Ctx<'a>,
}

impl<'a> KanbanSource<'a> {
    #[must_use]
    pub fn new(set: &'a KanbanSet, kanban: KanbanId, cx: &Ctx<'a>) -> Self {
        Self {
            set,
            kanban,
            cx: *cx,
        }
    }

    fn item(&self, c: &Card) -> Option<Item> {
        let k = self.set.get(self.kanban)?;
        let end = c.due.map(|d| d.at).or(c.start)?;
        let start = c.start.filter(|s| c.due.is_some() && *s < end);
        let tone = c
            .labels
            .iter()
            .filter_map(|l| k.labels.get(l))
            .filter(|l| !l.meta.deleted)
            .find_map(|l| l.color)
            .map_or_else(
                || ItemTone::List(k.lists.get(&c.list).and_then(|l| l.color)),
                ItemTone::Label,
            );
        Some(Item {
            key: ItemKey::Card(CardRef {
                kanban: self.kanban,
                card: c.id,
            }),
            title: c.title.clone(),
            start,
            end,
            done: c.due.is_some_and(|d| d.done),
            tone,
        })
    }
}

impl Source for KanbanSource<'_> {
    fn items(&self, from: Moment, to: Moment) -> Vec<Item> {
        let Some(k) = self.set.get(self.kanban) else {
            return Vec::new();
        };
        visible_cards(k, &self.cx)
            .into_iter()
            .filter_map(|c| self.item(c))
            .filter(|i| i.start.unwrap_or(i.end) < to && i.end >= from)
            .collect()
    }

    fn reschedule(&self, key: ItemKey, start: Option<Moment>, end: Moment) -> Plan {
        let mut plan = Plan::default();
        let ItemKey::Card(r) = key else {
            return plan;
        };
        let Some(c) = self.set.get(r.kanban).and_then(|k| k.cards.get(&r.card)) else {
            return plan;
        };
        let mut n = c.clone();
        match n.due.as_mut() {
            Some(d) => {
                d.at = end;
                n.start = start.or(n.start.filter(|s| *s < end));
            }
            None => n.start = Some(end),
        }
        if n != *c {
            plan.push(r.kanban, KanbanOp::put(Record::Card(n)));
        }
        plan
    }

    fn targets(&self) -> Vec<Target> {
        let Some(k) = self.set.get(self.kanban) else {
            return Vec::new();
        };
        k.lists_in_order()
            .into_iter()
            .map(|l| Target {
                list: ListRef {
                    kanban: self.kanban,
                    list: l.id,
                },
                name: l.title.clone(),
            })
            .collect()
    }

    fn create(&self, at: &Target, title: &str, due: Moment) -> Plan {
        let mut plan = Plan::default();
        let title = title.trim();
        let Some(k) = self.set.get(at.list.kanban) else {
            return plan;
        };
        if title.is_empty() || !k.lists.contains_key(&at.list.list) {
            return plan;
        }
        let pos: FracKey = k.pos_after_cards(at.list.list);
        let mut c = Card::new(
            ph2d_kanban_model::CardId(k.mint()),
            at.list.list,
            pos,
            title,
            self.cx.now,
        );
        c.due = Some(Due {
            at: due,
            done: false,
            remind_before_min: None,
        });
        c.meta = Meta::default();
        plan.push(at.list.kanban, KanbanOp::put(Record::Card(c)));
        plan
    }
    fn create_span(&self, at: &Target, title: &str, start: Option<Moment>, due: Moment) -> Plan {
        self.create_span_kanban(at, title, start, due)
    }
}

impl KanbanSource<'_> {
    /// O cartão de [`Source::create`] com o início `start` (a grelha de horas).
    fn with_start(plan: Plan, start: Option<Moment>) -> Plan {
        let mut out = Plan::default();
        for (k, ops) in plan.into_steps() {
            let ops = ops.into_iter().map(|op| match op {
                KanbanOp::Put(r) => match *r {
                    Record::Card(mut c) => {
                        c.start = start.or(c.start);
                        KanbanOp::put(Record::Card(c))
                    }
                    other => KanbanOp::put(other),
                },
                other => other,
            });
            out.extend(k, ops);
        }
        out
    }

    /// [`Source::create_span`] da fonte de um Kanban (chamada pela implementação do trait).
    fn create_span_kanban(&self, at: &Target, title: &str, start: Option<Moment>, due: Moment) -> Plan {
        Self::with_start(self.create(at, title, due), start.filter(|s| *s < due))
    }
}
