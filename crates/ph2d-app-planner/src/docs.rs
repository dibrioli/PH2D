//! ⭐ **`PlannerDocs` — a família instalada no hero**: UMA aba com três modos (o idioma do Trello de
//! 2025: a barra «Inbox · Planner · Boards» é uma navegação, não três documentos) — **Calendar** (o
//! Calendário do projecto), **Planner** (a semana com a coluna Inbox + cartões sem data) e **My
//! cards**. O sino das notificações vive na barra dela.
//!
//! O documento é o MESMO `Rc` do projecto ([`SharedKanbans`]); cada gesto é um [`Plan`] aplicado por
//! [`KanbanSet::apply_plan`] com a atividade em nome de «eu», e o desfazer da aba desfaz o gesto
//! inteiro em todos os documentos que tocou ([`KanbanSet::undo_gesture`]).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_editor_core::documents::{SharedKanbans, undo_tick};
use ph2d_editor_core::widget::LineField;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{Weekday, Zone};
use ph2d_kanban_filter::Filter;
use ph2d_kanban_model::{
    Actor, BlockOf, CardRef, EventId, History, InboxId, KanbanId, ListRef, Member, Moment, Plan,
};
use ph2d_kanban_views::Ctx;
use ph2d_kanban_views::calendar::CalendarState;

use crate::dest_pop::DestPop;
use crate::event_pop::EventPop;
use crate::notify::Notice;
use crate::personal::GroupBy;
use crate::scope::{Ask, ScopePop};
use crate::source::Show;

/// O id da aba (uma só).
pub const TAB: u64 = 0x504C_414E;

/// O modo da aba.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Calendar,
    Planner,
    MyCards,
}

/// O que está debaixo do rato na moldura da aba (o calendário responde por ele mesmo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Mode(Mode),
    Bell,
    Export,
    NewEvent,
    ColorBy,
    Person,
    GroupBy,
    MarkAll,
    Notice(u64),
    InboxField,
    InboxItem(InboxId),
    /// «To Kanban»: abre o destino do item (Kanban → lista → topo/fundo).
    InboxToCard(InboxId),
    /// Pôr no Planner (um bloco na próxima meia hora livre de hoje).
    Schedule(BlockOf),
    Card(CardRef),
    Pop(crate::event_pop::PopHit),
}

/// Um arrasto da coluna do Planner para o calendário.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SideDrag {
    pub of: BlockOf,
    pub from: (f32, f32),
    pub at: (f32, f32),
}

pub struct PlannerDocs {
    pub(crate) set: SharedKanbans,
    adopted: bool,
    pub(crate) open: bool,
    pub(crate) focus: Option<u64>,
    pub(crate) mode: Mode,
    pub(crate) show: Show,
    pub(crate) group_by: GroupBy,
    pub(crate) cal: CalendarState,
    pub(crate) filter: Filter,
    pub(crate) histories: BTreeMap<KanbanId, History>,
    pub(crate) me: Member,
    pub(crate) team: Vec<Member>,
    pub(crate) now: Moment,
    pub(crate) zone: Zone,
    pub(crate) week_start: Weekday,
    pub(crate) bell: bool,
    pub(crate) notices: Vec<Notice>,
    /// Os lembretes já avisados nesta sessão (o aviso toca UMA vez).
    pub(crate) fired: BTreeSet<u64>,
    /// Avisos e pedidos para o mundo (a shell serve-os no `frame`).
    pub(crate) toasts: Vec<String>,
    pub(crate) open_cards: Vec<CardRef>,
    pub(crate) export: Option<String>,
    /// A captura da Inbox (o campo «Add to Inbox»), em edição.
    pub(crate) capture: Option<LineField>,
    pub(crate) pop: Option<EventPop>,
    /// O destino de «To Kanban» aberto.
    pub(crate) dest: Option<DestPop>,
    /// A pergunta do alcance numa ocorrência de uma série.
    pub(crate) ask: Option<ScopePop>,
    /// Onde o rato carregou no calendário (um clique sem arrasto num evento abre-o).
    pub(crate) cal_down: Option<(f32, f32)>,
    pub(crate) drag: Option<SideDrag>,
    pub(crate) hover: Option<Hit>,
    pub(crate) pressed: Option<Hit>,
    pub(crate) hits: Vec<(Hit, Rect)>,
    pub(crate) area: Rect,
    pub(crate) calendar_area: Rect,
    /// Um relógio fixo (os testes e as fotos); `None` = o de parede.
    pub pinned: Option<Moment>,
}

impl Default for PlannerDocs {
    fn default() -> Self {
        Self {
            set: SharedKanbans::default(),
            adopted: false,
            open: false,
            focus: None,
            mode: Mode::default(),
            show: Show::default(),
            group_by: GroupBy::default(),
            cal: CalendarState::default(),
            filter: Filter::default(),
            histories: BTreeMap::new(),
            me: Member(ph2d_i18n::tr("kanban.back.me").to_owned()),
            team: Vec::new(),
            now: Moment(0),
            zone: Zone::system(),
            week_start: Weekday::Monday,
            bell: false,
            notices: Vec::new(),
            fired: BTreeSet::new(),
            toasts: Vec::new(),
            open_cards: Vec::new(),
            export: None,
            capture: None,
            pop: None,
            dest: None,
            ask: None,
            cal_down: None,
            drag: None,
            hover: None,
            pressed: None,
            hits: Vec::new(),
            area: Rect::new(0.0, 0.0, 0.0, 0.0),
            calendar_area: Rect::new(0.0, 0.0, 0.0, 0.0),
            pinned: None,
        }
    }
}

impl PlannerDocs {
    /// Segue o conjunto do projecto: outro `Rc` = outro projecto (o desfazer do anterior larga-se).
    pub fn follow_project(&mut self, shared: SharedKanbans) {
        if std::rc::Rc::ptr_eq(&self.set, &shared) {
            return;
        }
        if self.adopted {
            self.histories.clear();
            self.pop = None;
            self.dest = None;
            self.ask = None;
            self.capture = None;
            self.fired.clear();
        }
        self.set = shared;
        self.adopted = true;
    }

    /// O relógio de parede e quem sou (o nome da Equipa; «Me» sem nenhum).
    pub fn set_now(&mut self, now: Moment, me: Member, team: Vec<Member>) {
        self.now = now;
        self.me = me;
        self.team = team;
    }

    /// O fuso (o do sistema por omissão; os testes fixam-no).
    pub fn set_zone(&mut self, zone: Zone) {
        self.zone = zone;
    }

    /// Abre (ou activa) a aba no modo `mode`.
    pub fn open(&mut self, mode: Mode) {
        self.open = true;
        self.mode = mode;
        self.show.blocks_of = (mode == Mode::Planner).then(|| self.me.clone());
        self.focus = Some(TAB);
    }

    /// O contexto das vistas da W7 (o filtro do calendário do projecto é o [`Show`], não este).
    pub(crate) fn ctx(&self) -> Ctx<'_> {
        Ctx {
            now: self.now,
            zone: &self.zone,
            me: &self.me,
            team: &self.team,
            filter: &self.filter,
            week_start: self.week_start,
        }
    }

    /// ⭐ Aplica um gesto pela porta única do Kanban (`ph2d_kanban_hooks::gesture`): um passo de
    /// desfazer em cada documento que ele toca, com a atividade em nome de «eu», e o Butler e os
    /// scripts `ph2d.kanban.on` a reagir. `false` = nada a fazer.
    pub fn apply(&mut self, plan: Plan) -> bool {
        if plan.is_empty() {
            return false;
        }
        let who = ph2d_kanban_hooks::Who {
            me: self.me.clone(),
            now: self.now,
            zone: &self.zone,
        };
        ph2d_kanban_hooks::gesture(
            &mut self.set.borrow_mut(),
            &mut self.histories,
            plan,
            &who,
            &mut undo_tick,
        );
        true
    }

    /// O desfazer (ou refazer) do gesto mais recente da aba.
    pub fn undo(&mut self, redo: bool) -> bool {
        let actor = Actor {
            by: self.me.clone(),
            at: self.now,
        };
        let mut set = self.set.borrow_mut();
        if redo {
            set.redo_gesture(&mut self.histories, undo_tick(), &actor)
        } else {
            set.undo_gesture(&mut self.histories, undo_tick(), &actor)
        }
    }

    /// Captura `text` na minha Inbox.
    pub fn capture(&mut self, text: &str) -> bool {
        let plan = crate::personal::capture(&self.set.borrow(), &self.me, text, self.now);
        self.apply(plan)
    }

    /// O item `item` da Inbox vira um cartão no topo (`top`) ou no fim da lista `to`.
    pub fn inbox_to_card(&mut self, item: InboxId, to: ListRef, top: bool) -> bool {
        let plan = crate::personal::inbox_to_card(&self.set.borrow(), item, to, top, self.now);
        plan.is_some_and(|p| self.apply(p))
    }

    /// Abre o destino de «To Kanban» do item `item`, debaixo de `anchor`.
    pub fn open_move(&mut self, item: InboxId, anchor: Rect) {
        self.dest = Some(DestPop::new(&self.set.borrow(), item, anchor));
    }

    /// Pergunta o alcance de `ask` junto de `anchor`.
    pub(crate) fn ask_scope(&mut self, ask: Ask, anchor: (f32, f32)) {
        self.ask = Some(ScopePop::new(ask, anchor));
    }

    /// Responde à pergunta aberta com `scope` (um gesto, um passo de desfazer).
    pub fn answer(&mut self, scope: ph2d_kanban_model::Scope) -> bool {
        let Some(q) = self.ask.take() else {
            return false;
        };
        let plan = q.ask.plan(&self.set.borrow(), scope);
        self.apply(plan)
    }

    /// Abre o popover na ocorrência `at` do evento `id` do Kanban `kanban`.
    pub fn edit_occurrence(&mut self, kanban: KanbanId, id: EventId, at: Moment) {
        let set = self.set.borrow();
        let e = set.get(kanban).and_then(|k| k.events.get(&id)).cloned();
        if e.is_some() {
            self.pop = Some(EventPop::new(
                e,
                kanban,
                Some(at),
                self.next_slot(),
                &self.ctx(),
            ));
        }
    }

    /// A primeira lista do primeiro Kanban (o destino de «Move to Kanban» sem escolha).
    #[must_use]
    pub fn first_list(&self) -> Option<ListRef> {
        let set = self.set.borrow();
        set.iter().find_map(|k| {
            k.lists_in_order().first().map(|l| ListRef {
                kanban: k.id,
                list: l.id,
            })
        })
    }

    /// Põe `of` num bloco que começa em `start`.
    pub fn schedule_at(&mut self, of: BlockOf, start: Moment) -> bool {
        let plan = crate::personal::schedule(
            &self.set.borrow(),
            &self.me,
            of,
            start,
            crate::metrics::BLOCK_MIN,
        );
        self.apply(plan)
    }

    /// A próxima meia hora (o bloco de um clique em «Schedule»).
    #[must_use]
    pub fn next_slot(&self) -> Moment {
        let s = crate::metrics::SLOT_MS;
        Moment((self.now.0.div_euclid(s) + 1) * s)
    }

    /// Lê a notificação `key` (e abre o cartão dela).
    pub fn read_notice(&mut self, key: u64) -> bool {
        let n = self.notices.iter().find(|n| n.key == key).cloned();
        let op = crate::notify::mark_read(&self.set.borrow(), &self.me, key);
        let mut plan = Plan::default();
        if let Some(op) = op {
            plan.push(KanbanId::AGENDA, op);
        }
        if let Some(c) = n.and_then(|n| {
            n.card.map(|card| CardRef {
                kanban: n.kanban,
                card,
            })
        }) {
            self.open_cards.push(c);
        }
        self.apply(plan)
    }

    /// «Mark all as read».
    pub fn read_all(&mut self) -> bool {
        let op = crate::notify::mark_all_read(&self.set.borrow(), &self.me, self.now);
        let mut plan = Plan::default();
        plan.push(KanbanId::AGENDA, op);
        self.apply(plan)
    }

    /// Recalcula as notificações e devolve os lembretes NOVOS (o aviso na hora).
    pub fn refresh_notices(&mut self) -> Vec<Notice> {
        self.notices = crate::notify::notices(&self.set.borrow(), &self.me, self.now, &self.zone);
        let fresh: Vec<Notice> = self
            .notices
            .iter()
            .filter(|n| n.is_reminder() && n.unread && !self.fired.contains(&n.key))
            .cloned()
            .collect();
        self.fired.extend(fresh.iter().map(|n| n.key));
        fresh
    }

    /// Quantas notificações por ler (o número do sino).
    #[must_use]
    pub fn unread(&self) -> usize {
        self.notices.iter().filter(|n| n.unread).count()
    }

    /// Abre o popover de um evento novo (`None`) ou existente.
    pub fn edit_event(&mut self, id: Option<EventId>) {
        let set = self.set.borrow();
        let e = id.and_then(|id| set.agenda().events.get(&id)).cloned();
        self.pop = Some(EventPop::new(
            e,
            KanbanId::AGENDA,
            None,
            self.next_slot(),
            &self.ctx(),
        ));
    }

    /// O `.ics` do projecto inteiro (o que «Export .ics» grava).
    #[must_use]
    pub fn ics(&self) -> String {
        crate::ics::export(
            &self.set.borrow(),
            ph2d_i18n::tr("planner.calendar_name"),
            self.now,
        )
    }

    pub fn set(&self) -> SharedKanbans {
        self.set.clone()
    }

    /// Os pedidos para o mundo desde o último quadro: cartões a abrir, avisos, o `.ics` a gravar.
    pub fn take_requests(&mut self) -> (Vec<CardRef>, Vec<String>, Option<String>) {
        (
            std::mem::take(&mut self.open_cards),
            std::mem::take(&mut self.toasts),
            self.export.take(),
        )
    }

    /// ⚠️ SONDA: onde a moldura pintou `hit` no último desenho.
    #[must_use]
    pub fn probe_hit(&self, hit: Hit) -> Option<Rect> {
        self.hits.iter().find(|(h, _)| *h == hit).map(|(_, r)| *r)
    }

    /// ⚠️ SONDA: tudo o que a moldura pintou com área de clique.
    #[must_use]
    pub fn probe_hits(&self) -> Vec<(Hit, Rect)> {
        self.hits.clone()
    }

    /// ⚠️ SONDA: o modo e a área do calendário no último desenho.
    #[must_use]
    pub fn probe_mode(&self) -> (Mode, Rect) {
        (self.mode, self.calendar_area)
    }

    /// ⚠️ SONDA: as notificações calculadas no último quadro.
    #[must_use]
    pub fn probe_notices(&self) -> &[Notice] {
        &self.notices
    }

    /// ⚠️ SONDA: fecha o popover sem gravar (as fotos passam ao estado seguinte).
    pub fn close_pop_for_photo(&mut self) {
        self.pop = None;
        self.dest = None;
        self.ask = None;
    }

    /// ⚠️ SONDA: o popover do evento aberto.
    #[must_use]
    pub fn probe_pop(&self) -> Option<&EventPop> {
        self.pop.as_ref()
    }

    /// ⚠️ SONDA: o destino de «To Kanban» aberto.
    #[must_use]
    pub fn probe_dest(&self) -> Option<&DestPop> {
        self.dest.as_ref()
    }

    /// ⚠️ SONDA: a pergunta do alcance aberta.
    #[must_use]
    pub fn probe_ask(&self) -> Option<&ScopePop> {
        self.ask.as_ref()
    }

    /// ⚠️ SONDA: o estado do calendário (os testes perguntam-lhe onde pintou um item).
    #[must_use]
    pub fn probe_calendar(&self) -> &CalendarState {
        &self.cal
    }
}
