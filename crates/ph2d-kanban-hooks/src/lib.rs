//! `ph2d-kanban-hooks` — ⭐ **a porta ÚNICA de escrita de um gesto num Kanban** (TrelloClone J3).
//!
//! A lei: os alvos ANTES ([`Seen`]) → aplicar (um passo de desfazer, a atividade em nome de quem
//! fez) → o Butler reage ([`ph2d_kanban_auto::react`], o plano dele num passo PRÓPRIO, autor Butler)
//! → os handlers `ph2d.kanban.on` dos scripts carregados reagem ao gesto E ao Butler (um passo por
//! corrida). O desfazer/refazer NÃO passa aqui (desfazer não é gesto: reagir de novo repetia o
//! disparo). A agenda ([`Agenda`]) corre FORA do desfazer — desfazer a hora gravada repetia-a.
//!
//! Baixa de propósito: a aba e o elemento do Quadro (`ph2d-app-kanban`), o Planner
//! (`ph2d-app-planner`) e o editor de scripts (`ph2d-app-code`) chamam-na sem se conhecerem. Os
//! scripts carregados vivem por fio ([`with_scripts`]): a VM Luau não é `Send`, e o app desenha e
//! escreve num só fio.

mod open;
mod scripts;

use std::collections::BTreeMap;

use ph2d_kanban_auto::{Event, Fired, Kind, Seen, Stop, react as butler_react, specs};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{
    Actor, History, KanbanId, KanbanOp, KanbanSet, Member, Moment, Plan, apply_logged,
};

pub use open::{request_open_card, take_open_cards};
pub use ph2d_kanban_script::{Ctx, KanbanScripts, Report, ScriptError};
pub use scripts::{
    Line, RunRequest, is_kanban_script, request_run, run as run_script, take_lines, take_runs,
    unload, with_scripts,
};

/// Quem fez o gesto, quando, em que fuso.
#[derive(Clone, Debug)]
pub struct Who<'z> {
    pub me: Member,
    pub now: Moment,
    pub zone: &'z Zone,
}

/// O que reagiu a um gesto (ou à agenda).
#[derive(Clone, Debug, Default)]
pub struct Reaction {
    /// O gesto mudou algum Kanban.
    pub wrote: bool,
    /// As automações do Butler que correram (as dos scripts incluídas).
    pub butler: Vec<Fired>,
    /// Se a cascata parou num tecto (`MAX_DEPTH`/`MAX_RUNS`).
    pub stopped: Option<Stop>,
    /// Quantos handlers de script correram.
    pub handlers: usize,
}

/// ⭐ Um gesto da pessoa: `plan` aplicado em nome de `who.me` (um passo por Kanban tocado, à hora
/// `tick()`), e depois as reações ([`react`]). `tick` dá a hora ÚNICA de cada passo de desfazer
/// (o `undo_tick` do editor): o gesto, o Butler e cada corrida de script desfazem-se um a um.
pub fn gesture(
    set: &mut KanbanSet,
    histories: &mut BTreeMap<KanbanId, History>,
    plan: Plan,
    who: &Who<'_>,
    tick: &mut dyn FnMut() -> u64,
) -> Reaction {
    if plan.is_empty() {
        return Reaction::default();
    }
    let seen = Seen::plan(set, &plan);
    let before = stamps(histories);
    let actor = Actor {
        by: who.me.clone(),
        at: who.now,
    };
    set.apply_plan(plan, histories, tick(), &actor);
    let wrote = stamps(histories) != before;
    let events = seen.events(set, &who.me);
    let mut r = react(set, histories, events, who, tick);
    r.wrote |= wrote;
    r
}

/// Os alvos de um lote de UM Kanban antes de ele se aplicar e o que mudou depois, para quem aplica
/// por outra porta do modelo (`History::apply` de um campo da vista) e deixa a reação para o fim da
/// entrada ([`react`]).
#[must_use]
pub fn seen_ops(set: &KanbanSet, kanban: KanbanId, ops: &[KanbanOp]) -> Seen {
    Seen::ops(set, kanban, ops)
}

/// ⭐ As reações a acontecimentos já aplicados: o Butler (um passo, autor Butler) e os scripts
/// carregados (um passo por corrida). Sem regras nem handlers não toca em nada.
pub fn react(
    set: &mut KanbanSet,
    histories: &mut BTreeMap<KanbanId, History>,
    mut events: Vec<Event>,
    who: &Who<'_>,
    tick: &mut dyn FnMut() -> u64,
) -> Reaction {
    let mut r = Reaction::default();
    if events.is_empty() {
        return r;
    }
    let out = butler_react(set, events.clone(), who.now, who.zone);
    r.butler = out.fired;
    r.stopped = out.stopped;
    if !out.plan.is_empty() {
        let seen = Seen::plan(set, &out.plan);
        let butler = Actor {
            by: Member::butler(),
            at: who.now,
        };
        set.apply_plan(out.plan, histories, tick(), &butler);
        r.wrote = true;
        events.extend(seen.events(set, &Member::butler()));
    }
    run_handlers(set, histories, events, who, tick(), &mut r);
    r
}

/// O que o Butler fez por um botão ou pelo painel (o plano de `press_*_button`, já com a cascata
/// das regras): aplica-se em nome dele, num passo, e só os scripts reagem — o Butler não volta a
/// correr sobre o que ele mesmo fez (a lei do `ph2d-kanban-auto`).
pub fn butler(
    set: &mut KanbanSet,
    histories: &mut BTreeMap<KanbanId, History>,
    plan: Plan,
    who: &Who<'_>,
    tick: &mut dyn FnMut() -> u64,
) -> Reaction {
    let mut r = Reaction::default();
    if plan.is_empty() {
        return r;
    }
    let seen = Seen::plan(set, &plan);
    let before = stamps(histories);
    let actor = Actor {
        by: Member::butler(),
        at: who.now,
    };
    set.apply_plan(plan, histories, tick(), &actor);
    r.wrote = stamps(histories) != before;
    let events = seen.events(set, &Member::butler());
    run_handlers(set, histories, events, who, tick(), &mut r);
    r
}

fn run_handlers(
    set: &mut KanbanSet,
    histories: &mut BTreeMap<KanbanId, History>,
    events: Vec<Event>,
    who: &Who<'_>,
    at: u64,
    r: &mut Reaction,
) {
    if events.is_empty() {
        return;
    }
    let cx = Ctx {
        now: who.now,
        zone: who.zone,
        me: who.me.clone(),
        at,
    };
    let report = with_scripts(|s| {
        (!s.loaded().is_empty()).then(|| s.react(set, histories, events, &cx))
    });
    if let Some(rep) = report {
        r.wrote |= rep.changed;
        r.handlers += rep.handlers;
        r.butler.extend(rep.butler.iter().cloned());
        r.stopped = r.stopped.or(rep.stopped);
        scripts::log(&rep);
    }
}

fn stamps(h: &BTreeMap<KanbanId, History>) -> Vec<Option<u64>> {
    h.values().map(History::undo_at).collect()
}

/// ⭐ A agenda do Butler (comandos de calendário e de prazo), a correr no quadro do app: só quando
/// algum Kanban tem um comando agendado, no máximo uma vez por [`AGENDA_STEP_MS`], e o plano FORA
/// do desfazer (autor Butler). Os handlers dos scripts reagem também, fora do desfazer.
#[derive(Clone, Debug, Default)]
pub struct Agenda {
    last: Option<Moment>,
}

/// O passo da agenda. A menor unidade de um comando agendado é o MINUTO (`TimeOfDay`), e uma
/// verificação sem nada a fazer custa ~µs (`docs/TrelloClone/loop/MEDICOES.md` §J3): 1 s dá a hora
/// certa ao segundo sem pagar a decodificação das automações a cada quadro.
pub const AGENDA_STEP_MS: i64 = 1_000;

impl Agenda {
    /// Um quadro: corre a agenda se é altura. `None` = não correu.
    pub fn tick(&mut self, set: &mut KanbanSet, who: &Who<'_>) -> Option<Reaction> {
        if self
            .last
            .is_some_and(|l| who.now.0.saturating_sub(l.0) < AGENDA_STEP_MS)
        {
            return None;
        }
        self.last = Some(who.now);
        has_scheduled(set).then(|| run_scheduled(set, who))
    }
}

/// Algum Kanban tem um comando de calendário ou de prazo.
#[must_use]
pub fn has_scheduled(set: &KanbanSet) -> bool {
    set.iter().any(|k| {
        !k.automations.is_empty()
            && specs(k)
                .iter()
                .any(|(_, s)| matches!(s.kind, Kind::Calendar { .. } | Kind::DueDate { .. }))
    })
}

/// Corre a agenda agora: o plano aplica-se SEM passo de desfazer (autor Butler).
pub fn run_scheduled(set: &mut KanbanSet, who: &Who<'_>) -> Reaction {
    let out = ph2d_kanban_auto::run_scheduled(set, who.now, who.zone);
    let mut r = Reaction {
        butler: out.fired,
        stopped: out.stopped,
        ..Reaction::default()
    };
    if out.plan.is_empty() {
        return r;
    }
    let seen = Seen::plan(set, &out.plan);
    let butler = Actor {
        by: Member::butler(),
        at: who.now,
    };
    for (k, ops) in out.plan.into_steps() {
        if let Some(doc) = set.doc_mut(k) {
            r.wrote |= !apply_logged(doc, ops, &butler).is_empty();
        }
    }
    let events = seen.events(set, &Member::butler());
    // Fora do desfazer: os passos dos handlers vão para um histórico que se deita fora.
    let mut scratch = BTreeMap::new();
    run_handlers(set, &mut scratch, events, who, 0, &mut r);
    r
}

#[cfg(test)]
mod tests;
