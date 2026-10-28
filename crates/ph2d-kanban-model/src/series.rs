//! ⭐ **Uma ocorrência de uma série** (v7) — o «This event · This and following events · All events»
//! do Google Calendar e do Outlook. Uma ocorrência é dita pelo início que a REGRA lhe dá (`at`):
//!
//! - **This**: a ocorrência sai da série (uma [`Exception`]) e, se mudou, nasce um evento avulso.
//! - **Following**: a série acaba antes de `at` (`until`) e, se mudou, uma série nova começa nela
//!   (as excepções de depois vão com ela).
//! - **All**: a série inteira anda o que a ocorrência andou; as excepções andam com ela.
//!
//! Cada função devolve o lote de UM gesto (um passo de desfazer).

use crate::agenda::{Event, Exception};
use crate::kanban::Kanban;
use crate::ops::{KanbanOp, Record, RecordId};
use crate::records::{EventId, Meta, Moment};

/// O alcance de uma mudança numa ocorrência.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    This,
    Following,
    All,
}

fn shifted(x: &Exception, by: i64) -> Exception {
    Exception {
        at: Moment(x.at.0 + by),
        becomes: x.becomes,
    }
}

/// A série sem a excepção de `at` (a que há, se há).
fn without(series: &Event, at: Moment) -> Event {
    let mut s = series.clone();
    let hit = series.exception_at(at).copied();
    s.exceptions.retain(|x| Some(*x) != hit);
    s
}

/// Os evento avulsos das excepções `xs` que ainda vivem (os que um apagar leva consigo).
fn replacements(k: &Kanban, xs: &[Exception]) -> Vec<KanbanOp> {
    xs.iter()
        .filter_map(|x| x.becomes)
        .filter(|id| k.events.get(id).is_some_and(|e| !e.meta.deleted))
        .map(|id| KanbanOp::Delete(RecordId::Event(id)))
        .collect()
}

/// ⭐ O lote que leva a ocorrência `at` da série `series` (no Kanban `k`) a ficar como `to` (o id
/// de `to` não conta). Vazio = nada mudou.
#[must_use]
pub fn change_occurrence(
    k: &Kanban,
    series: &Event,
    at: Moment,
    to: &Event,
    scope: Scope,
) -> Vec<KanbanOp> {
    let shift = to.start.0 - at.0;
    let len = to.end.0 - to.start.0;
    let scope = match scope {
        Scope::Following if at <= series.start => Scope::All,
        s => s,
    };
    let mut fresh = to.clone();
    fresh.id = EventId(k.mint());
    fresh.meta = Meta::default();
    match scope {
        Scope::All => {
            let mut n = to.clone();
            n.id = series.id;
            n.meta = series.meta.clone();
            n.start = Moment(series.start.0 + shift);
            n.end = Moment(n.start.0 + len);
            n.until = series.until.map(|u| Moment(u.0 + shift));
            n.exceptions = series
                .exceptions
                .iter()
                .map(|x| shifted(x, shift))
                .collect();
            if n == *series {
                return Vec::new();
            }
            vec![KanbanOp::put(Record::Event(n))]
        }
        Scope::This => {
            if to.start == at && len == series.end.0 - series.start.0 && same_but_dates(to, series)
            {
                return Vec::new();
            }
            fresh.repeat = None;
            fresh.until = None;
            fresh.exceptions.clear();
            let mut s = without(series, at);
            s.exceptions.push(Exception {
                at,
                becomes: Some(fresh.id),
            });
            vec![
                KanbanOp::put(Record::Event(fresh)),
                KanbanOp::put(Record::Event(s)),
            ]
        }
        Scope::Following => {
            let mut s = series.clone();
            s.until = Some(at);
            s.exceptions.retain(|x| x.at < at);
            fresh.until = series.until.map(|u| Moment(u.0 + shift));
            fresh.exceptions = series
                .exceptions
                .iter()
                .filter(|x| x.at >= at)
                .map(|x| shifted(x, shift))
                .collect();
            vec![
                KanbanOp::put(Record::Event(s)),
                KanbanOp::put(Record::Event(fresh)),
            ]
        }
    }
}

/// `to` diz o mesmo que `series` fora das datas e do que é da série (título, cor, pessoas…).
fn same_but_dates(to: &Event, series: &Event) -> bool {
    let mut p = to.clone();
    p.id = series.id;
    p.meta = series.meta.clone();
    (p.start, p.end) = (series.start, series.end);
    p.until = series.until;
    p.exceptions.clone_from(&series.exceptions);
    p == *series
}

/// ⭐ O lote que apaga a ocorrência `at` da série `series` com o alcance `scope`.
#[must_use]
pub fn delete_occurrence(k: &Kanban, series: &Event, at: Moment, scope: Scope) -> Vec<KanbanOp> {
    let scope = match scope {
        Scope::Following if at <= series.start => Scope::All,
        s => s,
    };
    match scope {
        Scope::All => {
            let mut ops = vec![KanbanOp::Delete(RecordId::Event(series.id))];
            ops.extend(replacements(k, &series.exceptions));
            ops
        }
        Scope::This => {
            let mut s = without(series, at);
            s.exceptions.push(Exception { at, becomes: None });
            vec![KanbanOp::put(Record::Event(s))]
        }
        Scope::Following => {
            let (keep, gone): (Vec<Exception>, Vec<Exception>) =
                series.exceptions.iter().partition(|x| x.at < at);
            let mut s = series.clone();
            s.until = Some(at);
            s.exceptions = keep;
            let mut ops = vec![KanbanOp::put(Record::Event(s))];
            ops.extend(replacements(k, &gone));
            ops
        }
    }
}

#[cfg(test)]
#[path = "series_tests.rs"]
mod tests;
