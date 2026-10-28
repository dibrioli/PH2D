//! O que um arrasto na Linha do tempo escreve: mover a barra inteira (início e entrega juntos, à
//! mesma hora local), puxar uma ponta (só o início ou só a entrega, sem se cruzarem), largar um
//! cartão da gaveta num dia, ou mudar de lista ao largar noutra faixa.

use ph2d_kanban_dates::{CivilDate, Zone, add_days, civil, instant, start_of_day};
use ph2d_kanban_model::{Card, CardId, Due, Kanban, KanbanId, KanbanOp, Moment, Plan, Record};

use super::layout::{LaneKey, Span, span};

/// A parte da barra que a mão leva.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Body,
    Start,
    End,
    /// Um cartão sem datas, vindo da gaveta.
    Drawer,
}

/// A hora por omissão de uma entrega nova (a do seletor de datas do Trello: 12:00).
const NOON: (u8, u8) = (12, 0);

/// O deslocamento `delta` aplicado à parte `part` dos dias `s`, preso para as pontas não se
/// cruzarem: a pré-visualização e a escrita usam este mesmo número.
#[must_use]
pub fn clamp_delta(s: &Span, part: Part, delta: i64) -> i64 {
    let len = s.start.days_until(s.end);
    match part {
        Part::Start => delta.min(len),
        Part::End => delta.max(-len),
        Part::Body | Part::Drawer => delta,
    }
}

/// Os dias que a barra mostra durante o arrasto.
#[must_use]
pub fn moved(s: &Span, part: Part, delta: i64) -> Span {
    let d = clamp_delta(s, part, delta);
    match part {
        Part::Body | Part::Drawer => Span {
            start: s.start.add_days(d),
            end: s.end.add_days(d),
            ..*s
        },
        Part::Start => Span {
            start: s.start.add_days(d),
            ..*s
        },
        Part::End => Span {
            end: s.end.add_days(d),
            ..*s
        },
    }
}

fn due_at(day: CivilDate, zone: &Zone) -> Option<Moment> {
    instant(day, NOON.0, NOON.1, zone).map(Moment)
}

/// O cartão `c` depois do gesto (`None` = nada muda).
#[must_use]
pub fn rescheduled(c: &Card, part: Part, delta: i64, day: CivilDate, zone: &Zone) -> Option<Card> {
    let mut n = c.clone();
    if part == Part::Drawer {
        n.start = start_of_day(day, zone).map(Moment);
        n.due = Some(Due {
            at: due_at(day, zone)?,
            done: false,
            remind_before_min: None,
        });
        return (n != *c).then_some(n);
    }
    let s = span(c, zone)?;
    let d = clamp_delta(&s, part, delta);
    if d == 0 {
        return None;
    }
    let shift = |m: Moment| Moment(add_days(m.0, d, zone));
    match part {
        Part::Body => {
            n.start = n.start.map(shift);
            if let Some(due) = n.due.as_mut() {
                due.at = shift(due.at);
            }
        }
        Part::Start => {
            n.start = match n.start {
                Some(st) => Some(shift(st)),
                None => start_of_day(s.start.add_days(d), zone).map(Moment),
            };
        }
        Part::End => match n.due.as_mut() {
            Some(due) => due.at = shift(due.at),
            None => {
                let st = n.start?;
                let hm = civil(st.0, zone);
                let day = s.end.add_days(d);
                let at = if (hm.hour, hm.minute) == (0, 0) {
                    due_at(day, zone)?
                } else {
                    Moment(instant(day, hm.hour, hm.minute, zone)?)
                };
                n.due = Some(Due {
                    at,
                    done: false,
                    remind_before_min: None,
                });
            }
        },
        Part::Drawer => {}
    }
    (n != *c).then_some(n)
}

/// ⭐ O lote de um gesto acabado: `delta` dias à parte `part` do cartão `card` (largado no dia
/// `day`, na faixa `lane` — uma faixa de OUTRA lista muda-lhe a lista, no fim dela).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn plan(
    k: &Kanban,
    kanban: KanbanId,
    card: CardId,
    part: Part,
    delta: i64,
    day: CivilDate,
    lane: Option<&LaneKey>,
    zone: &Zone,
) -> Plan {
    let mut plan = Plan::default();
    let Some(c) = k.cards.get(&card) else {
        return plan;
    };
    let mut n = rescheduled(c, part, delta, day, zone).unwrap_or_else(|| c.clone());
    if matches!(part, Part::Body | Part::Drawer)
        && let Some(LaneKey::List(l)) = lane
        && *l != c.list
        && k.lists
            .get(l)
            .is_some_and(|x| !x.meta.deleted && !x.archived)
    {
        n.list = *l;
        n.pos = k.pos_after_cards(*l);
    }
    if n != *c {
        plan.push(kanban, KanbanOp::put(Record::card(n)));
    }
    plan
}
