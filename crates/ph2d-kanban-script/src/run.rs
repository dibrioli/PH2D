//! **A corrida** — um script de Kanban corre o topo (as escritas e os `ph2d.kanban.on`), e os
//! handlers correm depois de cada gesto. Cada FASE (o topo, ou um handler) corre sobre uma cópia
//! do projecto: acabou bem ⇒ as escritas pousam no projecto com o autor «script: <nome>»; falhou ⇒
//! nenhuma pousa (a lei do `ph2d.set`: um gancho que falha perde as dele).
//!
//! **Desfazer:** tudo o que UMA corrida mudou — o topo ou um gesto e a cascata dele (handlers e
//! regras do Butler) — é UM passo do histórico de cada Kanban tocado, todos à hora `Ctx::at`: um
//! `Ctrl+Z` (`KanbanSet::undo_gesture`) devolve a corrida inteira.
//!
//! **Anti-laço:** a lei do Butler (`ph2d-kanban-auto::run`): um handler nunca corre por
//! acontecimentos da sua PRÓPRIA cadeia (nem através das regras que ela disparou); a cadeia não
//! passa de [`MAX_DEPTH`] e uma corrida não corre mais de [`MAX_RUNS`] handlers e regras.

use std::collections::{BTreeMap, VecDeque};
use std::time::Instant;

use mlua::{Function, Lua, MultiValue};
use ph2d_kanban_auto::{Event, Fired, MAX_DEPTH, MAX_RUNS, Seen, Stop, react};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{
    Actor, History, KanbanId, KanbanOp, KanbanSet, Member, Moment, apply_logged,
};
use ph2d_script::HOOK_BUDGET;

use crate::event::{event_name, to_lua};
use crate::lane::{Lane, script_member};

/// O «agora», o fuso, quem corre o script (o `@me`) e a hora do passo de desfazer.
#[derive(Clone, Debug)]
pub struct Ctx<'z> {
    pub now: Moment,
    pub zone: &'z Zone,
    pub me: Member,
    /// O `undo_tick` do gesto (único): a hora do passo no histórico de cada Kanban.
    pub at: u64,
}

/// Um erro de um script, em texto para o console.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptError {
    pub script: String,
    pub message: String,
}

/// O que uma corrida fez.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub errors: Vec<ScriptError>,
    /// `(script, linha)` do `print`.
    pub printed: Vec<(String, String)>,
    /// Quantos handlers correram.
    pub handlers: usize,
    /// As regras do Butler que as escritas dos scripts dispararam.
    pub butler: Vec<Fired>,
    /// Se a cascata parou num tecto.
    pub stopped: Option<Stop>,
    /// Se algum Kanban mudou (há um passo de desfazer).
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Link {
    Handler(usize, usize),
    Butler,
}

struct Loaded {
    name: String,
    lua: Lua,
    handlers: Vec<(String, Function)>,
}

struct Printed(Vec<String>);
struct Deadline(Option<Instant>);

/// Os scripts de Kanban carregados (cada um com a sua VM) e os handlers deles.
#[derive(Default)]
pub struct KanbanScripts {
    loaded: Vec<Loaded>,
}

/// Uma VM de script de Kanban: `ph2d.kanban`, o `print` para o relatório, o prazo e a sandbox.
fn vm() -> mlua::Result<Lua> {
    let lua = Lua::new();
    crate::api::wire(&lua)?;
    lua.set_app_data(Printed(Vec::new()));
    lua.set_app_data(Deadline(None));
    let print = lua.create_function(|lua, args: MultiValue| {
        let line: Vec<String> = args
            .iter()
            .map(|v| v.to_string().unwrap_or_else(|_| v.type_name().to_owned()))
            .collect();
        if let Some(mut p) = lua.app_data_mut::<Printed>() {
            p.0.push(line.join("\t"));
        }
        Ok(())
    })?;
    lua.globals().set("print", print)?;
    let ticks = std::cell::Cell::new(0u32);
    lua.set_interrupt(move |lua| {
        ticks.set(ticks.get().wrapping_add(1));
        if ticks.get() & 0xFF != 0 {
            return Ok(mlua::VmState::Continue);
        }
        let late = lua
            .app_data_ref::<Deadline>()
            .and_then(|d| d.0)
            .is_some_and(|until| Instant::now() > until);
        if late {
            return Err(mlua::Error::RuntimeError(format!(
                "the script took longer than {} ms (a loop that never ends?)",
                HOOK_BUDGET.as_millis().max(1)
            )));
        }
        Ok(mlua::VmState::Continue)
    });
    lua.sandbox(true)?;
    Ok(lua)
}

/// Uma escrita da fase, `(autor, Kanban, lote)`.
type Write = (Member, KanbanId, Vec<KanbanOp>);

/// Uma corrida: o desfazer acumulado por Kanban e o relatório.
struct Pass<'c, 'z> {
    cx: &'c Ctx<'z>,
    pending: BTreeMap<KanbanId, Vec<KanbanOp>>,
    report: Report,
    runs: usize,
    queue: VecDeque<(Event, Vec<Link>)>,
}

impl Pass<'_, '_> {
    /// Pousa as escritas no projecto (actividade em nome de cada autor) e devolve o que mudou.
    fn commit(&mut self, set: &mut KanbanSet, writes: Vec<Write>) -> Vec<Event> {
        let mut events = Vec::new();
        for (by, k, ops) in writes {
            let seen = Seen::ops(set, k, &ops);
            let actor = Actor {
                by: by.clone(),
                at: self.cx.now,
            };
            let Some(doc) = set.doc_mut(k) else { continue };
            let mut inverse = apply_logged(doc, ops, &actor);
            let older = self.pending.entry(k).or_default();
            inverse.append(older);
            *older = inverse;
            events.extend(seen.events(set, &by));
        }
        events
    }

    /// Pousa a fase, põe os acontecimentos na fila e deixa o Butler reagir a eles.
    fn land(&mut self, set: &mut KanbanSet, writes: Vec<Write>, chain: &[Link]) {
        let events = self.commit(set, writes);
        if events.is_empty() {
            return;
        }
        for e in &events {
            self.queue.push_back((e.clone(), chain.to_vec()));
        }
        if self.runs >= MAX_RUNS {
            self.report.stopped = Some(Stop::Runs);
            return;
        }
        let out = react(set, events, self.cx.now, self.cx.zone);
        self.runs += out.fired.len();
        self.report.butler.extend(out.fired);
        if out.stopped.is_some() {
            self.report.stopped = out.stopped;
        }
        let mut butler_chain = chain.to_vec();
        butler_chain.push(Link::Butler);
        let writes: Vec<Write> = out
            .plan
            .into_steps()
            .into_iter()
            .map(|(k, ops)| (Member::butler(), k, ops))
            .collect();
        for e in self.commit(set, writes) {
            self.queue.push_back((e, butler_chain.clone()));
        }
    }
}

impl KanbanScripts {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Os scripts carregados e quantos handlers cada um tem.
    #[must_use]
    pub fn loaded(&self) -> Vec<(&str, usize)> {
        self.loaded
            .iter()
            .map(|l| (l.name.as_str(), l.handlers.len()))
            .collect()
    }

    /// Larga o script `name` e os handlers dele.
    pub fn unload(&mut self, name: &str) {
        self.loaded.retain(|l| l.name != name);
    }

    /// ⭐ Corre o script `name` (substitui o anterior com esse nome): o topo, depois os handlers
    /// que as escritas dele acordarem. Um topo que falha não fica carregado.
    pub fn run(
        &mut self,
        name: &str,
        source: &str,
        set: &mut KanbanSet,
        histories: &mut BTreeMap<KanbanId, History>,
        cx: &Ctx<'_>,
    ) -> Report {
        self.unload(name);
        let mut pass = Pass {
            cx,
            pending: BTreeMap::new(),
            report: Report::default(),
            runs: 0,
            queue: VecDeque::new(),
        };
        let lua = match vm() {
            Ok(l) => l,
            Err(e) => {
                pass.report.errors.push(ScriptError {
                    script: name.to_owned(),
                    message: e.to_string(),
                });
                return pass.report;
            }
        };
        let chunk = format!("={name}");
        let outcome = phase(&lua, name, set, cx, &mut pass.report, |lua| {
            lua.load(source).set_name(chunk).exec()
        });
        if let Some((writes, handlers)) = outcome {
            self.loaded.push(Loaded {
                name: name.to_owned(),
                lua,
                handlers,
            });
            pass.land(set, writes, &[]);
            self.drain(set, &mut pass);
        }
        finish(pass, set, histories)
    }

    /// ⭐ Depois de um gesto de outra pessoa (ou do Butler) já aplicado: os handlers que os
    /// acontecimentos `events` acordam, e a cascata deles.
    pub fn react(
        &mut self,
        set: &mut KanbanSet,
        histories: &mut BTreeMap<KanbanId, History>,
        events: Vec<Event>,
        cx: &Ctx<'_>,
    ) -> Report {
        let mut pass = Pass {
            cx,
            pending: BTreeMap::new(),
            report: Report::default(),
            runs: 0,
            queue: events.into_iter().map(|e| (e, Vec::new())).collect(),
        };
        self.drain(set, &mut pass);
        finish(pass, set, histories)
    }

    fn drain(&mut self, set: &mut KanbanSet, pass: &mut Pass<'_, '_>) {
        while let Some((e, chain)) = pass.queue.pop_front() {
            let name = event_name(&e.what);
            let due: Vec<(usize, usize)> = self
                .loaded
                .iter()
                .enumerate()
                .flat_map(|(si, l)| {
                    l.handlers
                        .iter()
                        .enumerate()
                        .filter(|(_, (n, _))| n == name)
                        .map(move |(hi, _)| (si, hi))
                })
                .filter(|(si, hi)| !chain.contains(&Link::Handler(*si, *hi)))
                .collect();
            if !due.is_empty() && chain.len() >= MAX_DEPTH {
                pass.report.stopped.get_or_insert(Stop::Depth);
                continue;
            }
            for (si, hi) in due {
                if pass.runs >= MAX_RUNS {
                    pass.report.stopped = Some(Stop::Runs);
                    return;
                }
                pass.runs += 1;
                pass.report.handlers += 1;
                let l = &mut self.loaded[si];
                let f = l.handlers[hi].1.clone();
                let ev = match to_lua(&l.lua, &e, set, pass.cx.zone) {
                    Ok(t) => t,
                    Err(err) => {
                        pass.report.errors.push(ScriptError {
                            script: l.name.clone(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let out = phase(&l.lua, &l.name, set, pass.cx, &mut pass.report, |_| {
                    f.call::<()>(ev)
                });
                if let Some((writes, more)) = out {
                    l.handlers.extend(more);
                    let mut next = chain.clone();
                    next.push(Link::Handler(si, hi));
                    pass.land(set, writes, &next);
                }
            }
        }
    }
}

/// Uma fase sobre a cópia; `None` = falhou (o erro vai para o relatório e nada pousa).
fn phase(
    lua: &Lua,
    name: &str,
    set: &KanbanSet,
    cx: &Ctx<'_>,
    report: &mut Report,
    call: impl FnOnce(&Lua) -> mlua::Result<()>,
) -> Option<(Vec<Write>, Vec<(String, Function)>)> {
    lua.set_app_data(Lane {
        set: set.clone(),
        writes: Vec::new(),
        by: script_member(name),
        me: cx.me.clone(),
        now: cx.now,
        zone: cx.zone.clone(),
        handlers: Vec::new(),
    });
    lua.set_app_data(Deadline(Instant::now().checked_add(HOOK_BUDGET)));
    let r = call(lua);
    lua.set_app_data(Deadline(None));
    let lane = lua.remove_app_data::<Lane>();
    if let Some(mut p) = lua.app_data_mut::<Printed>() {
        report
            .printed
            .extend(p.0.drain(..).map(|l| (name.to_owned(), l)));
    }
    match (r, lane) {
        (Ok(()), Some(l)) => Some((l.writes, l.handlers)),
        (Err(e), _) => {
            report.errors.push(ScriptError {
                script: name.to_owned(),
                message: message(&e),
            });
            None
        }
        (Ok(()), None) => None,
    }
}

/// A mensagem do Luau sem o invólucro do callback do mlua.
fn message(e: &mlua::Error) -> String {
    match e {
        mlua::Error::CallbackError { cause, traceback } => {
            let first = traceback.lines().nth(1).map(str::trim).unwrap_or_default();
            let inner = message(cause);
            if first.is_empty() {
                inner
            } else {
                format!("{inner} ({first})")
            }
        }
        other => other.to_string(),
    }
}

/// Um passo de desfazer por Kanban tocado, todos à hora `cx.at`.
fn finish(
    pass: Pass<'_, '_>,
    set: &KanbanSet,
    histories: &mut BTreeMap<KanbanId, History>,
) -> Report {
    let mut report = pass.report;
    for (k, inverse) in pass.pending {
        if let Some(doc) = set.get(k)
            && !inverse.is_empty()
        {
            histories
                .entry(k)
                .or_default()
                .record(inverse, doc, pass.cx.at);
            report.changed = true;
        }
    }
    report
}
