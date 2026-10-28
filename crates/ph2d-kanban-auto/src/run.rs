//! **O executor** — determinístico: o mesmo projecto, os mesmos acontecimentos, o mesmo «agora» dão
//! o MESMO [`Plan`]. As regras de cada Kanban correm pela ordem do painel ([`crate::specs`]); os
//! acontecimentos pela ordem em que nasceram (fila).
//!
//! **Anti-laço** (ver `docs/TrelloClone/loop/MEDICOES.md` §W8): uma regra nunca dispara por
//! acontecimentos da sua PRÓPRIA cadeia (A → B → A pára em A), a cadeia não passa de
//! [`MAX_DEPTH`] regras e um gesto não corre mais de [`MAX_RUNS`] automações. Os dois tectos são de
//! SEGURANÇA (laço e leque), nunca de hardware.

use std::collections::{BTreeMap, VecDeque};

use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{AutomationId, Card, CardId, KanbanId, KanbanSet, Member, Moment, Plan};

use crate::act::{self, Target, Work};
use crate::cond::{Cx, all_hold, by_ok, fires};
use crate::event::Event;
use crate::spec::{Kind, Scope, Spec, specs};

/// Quantas regras uma cadeia encadeia, no máximo (o gesto da pessoa é a profundidade 0).
pub const MAX_DEPTH: usize = 16;
/// Quantas automações um gesto (ou uma passagem da agenda) corre, no máximo.
pub const MAX_RUNS: usize = 1000;

/// Uma automação que correu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fired {
    pub kanban: KanbanId,
    pub automation: AutomationId,
    /// O cartão do gatilho (regra, botão de cartão); `None` num comando ou botão de quadro.
    pub card: Option<CardId>,
    /// 0 = disparada pela pessoa (ou pela agenda); n = pela n-ésima regra da cadeia.
    pub depth: usize,
}

/// Porque o executor parou antes de acabar a fila.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Depth,
    Runs,
}

/// O que o executor fez: o plano (aplique-o com o autor [`Member::butler`]), o que correu e se
/// parou num tecto.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub plan: Plan,
    pub fired: Vec<Fired>,
    pub stopped: Option<Stop>,
}

type Chain = Vec<(KanbanId, AutomationId)>;

pub(crate) struct Engine<'z> {
    pub work: Work,
    now: Moment,
    zone: &'z Zone,
    fired: Vec<Fired>,
    stopped: Option<Stop>,
    queue: VecDeque<(Event, Chain, Member)>,
    /// As regras decodificadas de cada Kanban (as acções nunca mudam automações).
    rules: BTreeMap<KanbanId, Vec<(AutomationId, Spec)>>,
}

impl<'z> Engine<'z> {
    pub(crate) fn new(set: &KanbanSet, now: Moment, zone: &'z Zone) -> Self {
        Self {
            work: Work {
                set: set.clone(),
                plan: Plan::default(),
            },
            now,
            zone,
            fired: Vec::new(),
            stopped: None,
            queue: VecDeque::new(),
            rules: BTreeMap::new(),
        }
    }

    pub(crate) fn cx(&self, me: Member) -> Cx<'z> {
        Cx {
            now: self.now,
            zone: self.zone,
            me,
        }
    }

    /// Corre `spec` (a automação `id` do Kanban `kanban`) nos `targets`; `me` = quem o pediu.
    pub(crate) fn run(
        &mut self,
        kanban: KanbanId,
        id: AutomationId,
        spec: &Spec,
        mut targets: Vec<Target>,
        me: &Member,
        chain: &[(KanbanId, AutomationId)],
    ) {
        if self.fired.len() >= MAX_RUNS {
            self.stopped = Some(Stop::Runs);
            return;
        }
        let card = match spec.kind {
            Kind::Rule { .. } | Kind::CardButton => targets.first().map(|t| t.card),
            _ => None,
        };
        self.fired.push(Fired {
            kanban,
            automation: id,
            card,
            depth: chain.len(),
        });
        let cx = self.cx(me.clone());
        let mut events = Vec::new();
        for a in &spec.actions {
            if a.on_board() {
                events.extend(act::board(&mut self.work, a, kanban, &cx));
            } else {
                for t in &mut targets {
                    events.extend(act::card(&mut self.work, a, t, &cx));
                }
            }
        }
        let mut next = chain.to_vec();
        next.push((kanban, id));
        for e in events {
            self.queue.push_back((e, next.clone(), me.clone()));
        }
    }

    pub(crate) fn push(&mut self, e: Event) {
        let me = e.by.clone();
        self.queue.push_back((e, Vec::new(), me));
    }

    /// Esvazia a fila: cada acontecimento dispara as regras que casam.
    pub(crate) fn drain(&mut self) {
        while let Some((e, chain, me)) = self.queue.pop_front() {
            if self.stopped == Some(Stop::Runs) {
                return;
            }
            let Some(k) = self.work.set.get(e.kanban) else {
                continue;
            };
            let Some(card) = live(k.cards.get(&e.card)) else {
                continue;
            };
            let cx = self.cx(me.clone());
            let rules = self.rules.entry(e.kanban).or_insert_with(|| specs(k));
            let due: Vec<(AutomationId, Spec)> = rules
                .iter()
                .filter(|(id, s)| {
                    let Kind::Rule {
                        trigger,
                        by,
                        conditions,
                    } = &s.kind
                    else {
                        return false;
                    };
                    s.enabled
                        && !chain.contains(&(e.kanban, *id))
                        && by_ok(by, &e.by)
                        && fires(trigger, &e, k)
                        && all_hold(conditions, card, k, &cx)
                })
                .cloned()
                .collect();
            if !due.is_empty() && chain.len() >= MAX_DEPTH {
                self.stopped.get_or_insert(Stop::Depth);
                continue;
            }
            for (id, spec) in due {
                let t = Target {
                    kanban: e.kanban,
                    card: e.card,
                };
                self.run(e.kanban, id, &spec, vec![t], &me, &chain);
            }
        }
    }

    pub(crate) fn finish(self) -> Outcome {
        Outcome {
            plan: self.work.plan,
            fired: self.fired,
            stopped: self.stopped,
        }
    }
}

fn live(c: Option<&Card>) -> Option<&Card> {
    c.filter(|c| !c.meta.deleted && !c.template)
}

/// As regras que os acontecimentos `events` (de um gesto já aplicado a `set`, lidos por
/// [`crate::Seen`]) disparam, e as que essas disparam em cadeia.
#[must_use]
pub fn react(set: &KanbanSet, events: Vec<Event>, now: Moment, zone: &Zone) -> Outcome {
    let mut en = Engine::new(set, now, zone);
    for e in events {
        en.push(e);
    }
    en.drain();
    en.finish()
}

/// Os cartões em que um comando ou botão de quadro age: das listas do `scope` (vazio = todas, pela
/// ordem do Kanban), à vista, pela ordem da lista, e que passam nas condições.
#[must_use]
pub fn scope_cards(set: &KanbanSet, kanban: KanbanId, scope: &Scope, cx: &Cx) -> Vec<CardId> {
    let Some(k) = set.get(kanban) else {
        return Vec::new();
    };
    k.lists_in_order()
        .into_iter()
        .filter(|l| scope.lists.is_empty() || scope.lists.contains(&l.id))
        .flat_map(|l| k.cards_in(l.id))
        .filter(|c| !c.template && all_hold(&scope.conditions, c, k, cx))
        .map(|c| c.id)
        .collect()
}

fn spec_of(set: &KanbanSet, kanban: KanbanId, id: AutomationId) -> Option<Spec> {
    specs(set.get(kanban)?)
        .into_iter()
        .find(|(i, s)| *i == id && s.enabled)
        .map(|(_, s)| s)
}

/// Carregar no botão de cartão `button` no cartão `card`, por `by`.
#[must_use]
pub fn press_card_button(
    set: &KanbanSet,
    kanban: KanbanId,
    button: AutomationId,
    card: CardId,
    by: &Member,
    now: Moment,
    zone: &Zone,
) -> Outcome {
    let mut en = Engine::new(set, now, zone);
    if let Some(spec) = spec_of(set, kanban, button).filter(|s| s.kind == Kind::CardButton)
        && set
            .get(kanban)
            .and_then(|k| live(k.cards.get(&card)))
            .is_some()
    {
        let t = Target { kanban, card };
        en.run(kanban, button, &spec, vec![t], by, &[]);
        en.drain();
    }
    en.finish()
}

/// Carregar no botão de quadro `button`, por `by`.
#[must_use]
pub fn press_board_button(
    set: &KanbanSet,
    kanban: KanbanId,
    button: AutomationId,
    by: &Member,
    now: Moment,
    zone: &Zone,
) -> Outcome {
    let mut en = Engine::new(set, now, zone);
    if let Some(spec) = spec_of(set, kanban, button)
        && let Kind::BoardButton { scope } = &spec.kind
    {
        let cx = en.cx(by.clone());
        let targets = scope_cards(set, kanban, scope, &cx)
            .into_iter()
            .map(|card| Target { kanban, card })
            .collect();
        en.run(kanban, button, &spec, targets, by, &[]);
        en.drain();
    }
    en.finish()
}
