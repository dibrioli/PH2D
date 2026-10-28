//! **A agenda** — os comandos de calendário e de prazo que chegou a hora de correr.
//!
//! O documento guarda, em cada automação, o último instante já corrido (`last_run`): o que caiu em
//! `(last_run, agora]` corre, e correr carimba `agora`. Por isso reabrir o projecto nunca repete um
//! disparo, e o que caiu com o app fechado corre ao abrir:
//! - **calendário**: UMA vez, a ocorrência mais recente que se perdeu (dez dias fechado = uma
//!   «todo dia», não dez — o Butler corre no servidor e nunca acumula atraso);
//! - **prazo**: cada cartão cuja hora passou corre uma vez (cada cartão é um disparo próprio).
//!
//! Uma automação sem `last_run` (não armada) não corre: a primeira passagem só a arma em `agora`.

use ph2d_kanban_dates::{CivilDate, Zone, add_days, day_of, instant};
use ph2d_kanban_model::{AutomationId, CardId, KanbanId, KanbanSet, Moment, Record};

use crate::act::Target;
use crate::cond::Cx;
use crate::run::{Engine, Outcome, scope_cards};
use crate::spec::{DueWhen, Every, Kind, TimeOfDay, Unit, specs};

const HOUR_MS: i64 = 60 * 60 * 1000;
/// Um ano e um dia: todo [`Every`] ocorre pelo menos uma vez nesta janela (o dia além do fim do
/// mês cai no último), por isso procurar mais atrás nunca acha uma ocorrência mais recente.
const SCAN_DAYS: i64 = 367;

/// Um comando a correr agora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Firing {
    pub kanban: KanbanId,
    pub automation: AutomationId,
    /// O instante agendado (o mais recente, se vários cartões).
    pub at: Moment,
    /// Os cartões de um comando de prazo (vazio num de calendário).
    pub cards: Vec<CardId>,
}

fn last_day(year: i16, month: u8) -> u8 {
    (28..=31)
        .rev()
        .find(|d| CivilDate::new(year, month, *d).is_some())
        .unwrap_or(28)
}

/// O dia `d` é um dia de `every`?
#[must_use]
pub fn is_day(every: Every, d: CivilDate) -> bool {
    let clamp = |day: u8| day.min(last_day(d.year, d.month));
    match every {
        Every::Day => true,
        Every::Weekdays(mask) => mask & (1 << d.weekday().monday_index()) != 0,
        Every::MonthDay(n) => d.day == clamp(n),
        Every::YearDay { month, day } => d.month == month && d.day == clamp(day),
    }
}

/// A ocorrência mais recente de `every` às `at` em `(after, upto]`.
#[must_use]
pub fn latest_occurrence(
    every: Every,
    at: TimeOfDay,
    after: Moment,
    upto: Moment,
    zone: &Zone,
) -> Option<Moment> {
    let top = day_of(upto.0, zone);
    let floor = day_of(after.0, zone).max(top.add_days(-SCAN_DAYS));
    let mut d = top;
    while d >= floor {
        if is_day(every, d)
            && let Some(t) = instant(d, at.hour, at.minute, zone)
            && after.0 < t
            && t <= upto.0
        {
            return Some(Moment(t));
        }
        d = d.add_days(-1);
    }
    None
}

/// A hora em que o comando de prazo `when` dispara para uma entrega em `due`.
#[must_use]
pub fn due_moment(when: DueWhen, due: Moment, zone: &Zone) -> Option<Moment> {
    let shift = |n: u32, unit: Unit, sign: i64| match unit {
        Unit::Days => add_days(due.0, sign * i64::from(n), zone),
        Unit::Hours => due.0 + sign * i64::from(n) * HOUR_MS,
    };
    Some(Moment(match when {
        DueWhen::Before { n, unit } => shift(n, unit, -1),
        DueWhen::After { n, unit } => shift(n, unit, 1),
        DueWhen::OnDay { at } => instant(day_of(due.0, zone), at.hour, at.minute, zone)?,
    }))
}

/// O disparo de `spec` em `(last, now]`, ligada ou não.
fn pending(
    set: &KanbanSet,
    k: &ph2d_kanban_model::Kanban,
    id: AutomationId,
    spec: &crate::spec::Spec,
    last: Moment,
    now: Moment,
    zone: &Zone,
) -> Option<Firing> {
    match &spec.kind {
        Kind::Calendar { every, at, .. } => {
            latest_occurrence(*every, *at, last, now, zone).map(|t| Firing {
                kanban: k.id,
                automation: id,
                at: t,
                cards: Vec::new(),
            })
        }
        Kind::DueDate { when, scope } => {
            let cx = Cx {
                now,
                zone,
                me: spec.owner.clone(),
            };
            let mut cards = Vec::new();
            let mut at = last;
            for c in scope_cards(set, k.id, scope, &cx) {
                let Some(due) = k.cards.get(&c).and_then(|c| c.due) else {
                    continue;
                };
                if let Some(t) = due_moment(*when, due.at, zone)
                    && last < t
                    && t <= now
                {
                    cards.push(c);
                    at = at.max(t);
                }
            }
            (!cards.is_empty()).then_some(Firing {
                kanban: k.id,
                automation: id,
                at,
                cards,
            })
        }
        _ => None,
    }
}

/// Os disparos em `(last_run, now]`: `(das ligadas, das desligadas)`, numa passagem.
fn pending_all(set: &KanbanSet, now: Moment, zone: &Zone) -> (Vec<Firing>, Vec<Firing>) {
    let (mut on, mut off) = (Vec::new(), Vec::new());
    for k in set.iter() {
        for (id, spec) in specs(k) {
            let Some(last) = k.automations.get(&id).and_then(|a| a.last_run) else {
                continue;
            };
            let to = if spec.enabled { &mut on } else { &mut off };
            to.extend(pending(set, k, id, &spec, last, now, zone));
        }
    }
    (on, off)
}

/// Os comandos de calendário e de prazo de todos os Kanbans que têm disparo em `(last_run, now]`.
#[must_use]
pub fn due_commands(set: &KanbanSet, now: Moment, zone: &Zone) -> Vec<Firing> {
    pending_all(set, now, zone).0
}

/// Corre a agenda: arma as automações agendadas sem `last_run`, corre cada [`Firing`] e carimba
/// `now` nas que correram (no mesmo plano). Uma DESLIGADA cuja ocorrência chegou é carimbada sem
/// correr: o que caiu enquanto estava desligada nunca corre ao religar. Aplique o plano com o autor Butler e FORA do desfazer
/// da pessoa — desfazer um carimbo faria o disparo repetir.
#[must_use]
pub fn run_scheduled(set: &KanbanSet, now: Moment, zone: &Zone) -> Outcome {
    let mut en = Engine::new(set, now, zone);
    let (firings, skipped) = pending_all(set, now, zone);
    for k in set.iter() {
        for (id, spec) in specs(k) {
            let scheduled = matches!(spec.kind, Kind::Calendar { .. } | Kind::DueDate { .. });
            let Some(a) = k.automations.get(&id) else {
                continue;
            };
            let fired = firings
                .iter()
                .find(|f| f.kanban == k.id && f.automation == id);
            let swallowed = skipped
                .iter()
                .any(|f| f.kanban == k.id && f.automation == id);
            if !scheduled || (a.last_run.is_some() && fired.is_none() && !swallowed) {
                continue;
            }
            let mut stamp = a.clone();
            stamp.last_run = Some(now);
            en.work.put(k.id, Record::Automation(stamp));
            let Some(f) = fired else {
                continue;
            };
            let targets: Vec<CardId> = match &spec.kind {
                Kind::Calendar { scope, .. } => {
                    let cx = en.cx(spec.owner.clone());
                    scope_cards(&en.work.set, k.id, scope, &cx)
                }
                _ => f.cards.clone(),
            };
            let targets = targets
                .into_iter()
                .map(|card| Target { kanban: k.id, card })
                .collect();
            en.run(k.id, id, &spec, targets, &spec.owner, &[]);
        }
    }
    en.drain();
    en.finish()
}
