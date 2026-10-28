//! ⭐ O `DocumentSurface` da família: a moldura ([`crate::ui`], [`crate::side`]) responde primeiro
//! (o popover do evento, o sino, os botões); o resto da área é do calendário da W7, que devolve o lote
//! de cada gesto — aplicado aqui, um passo de desfazer.

use std::any::Any;

use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::{
    Compose, DocCommand, DocMenu, DocTab, DocumentSurface, Key, KeyInput, KeyOutcome, PointerInput,
    PointerLook,
};
use ph2d_editor_core::widget::{FieldKey, LineField};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_i18n::tr;
use ph2d_kanban_dates::{instant, start_of_day};
use ph2d_kanban_model::{BlockOf, EventId, KanbanId, Moment};
use ph2d_kanban_views::calendar;
use ph2d_kanban_views::{PointerAt, PointerKind as VKind, Reply};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

use crate::docs::{Hit, Mode, PlannerDocs, SideDrag, TAB};
use crate::dest_pop::DestDone;
use crate::event_pop::{Done, EventPop};
use crate::recur;
use crate::scope::{ScopeDone, ScopePop};
use crate::source::{REACH_MS, event_key};
use crate::metrics as m;
use crate::source::{ColorBy, ProjectSource};

pub const CMD_CALENDAR: NodeId = hash_node_id("planner_cmd_calendar");
pub const CMD_PLANNER: NodeId = hash_node_id("planner_cmd_planner");
pub const CMD_MY_CARDS: NodeId = hash_node_id("planner_cmd_my_cards");
pub const CMD_INBOX: NodeId = hash_node_id("planner_cmd_inbox");

/// A hora a que um bloco largado num dia começa (o dia de hoje: a próxima meia hora).
const DROP_HOUR: u8 = 9;
/// O arrasto da coluna só começa depois de o rato andar isto (um clique não arrasta).
const DRAG_SLOP: f32 = 4.0;

impl PlannerDocs {
    /// Passa um gesto ao calendário; o arrasto de uma ocorrência de uma série pergunta o alcance
    /// junto de `at`.
    fn with_calendar(&mut self, at: (f32, f32), f: impl FnOnce(&mut calendar::CalendarState, &ProjectSource<'_>) -> Reply) -> bool {
        let set = self.set.clone();
        let mut cal = std::mem::take(&mut self.cal);
        let (reply, asked) = {
            let set = set.borrow();
            let cx = self.ctx();
            let show = self.show.clone();
            let src = ProjectSource::new(&set, &show, cx);
            let reply = f(&mut cal, &src);
            (reply, src.asked.take())
        };
        self.cal = cal;
        if let Some(ask) = asked {
            self.ask_scope(ask, at);
        }
        if let Some(open) = reply.open {
            self.open_cards.push(open);
        }
        let taken = reply.taken;
        self.apply(reply.plan);
        taken
    }

    /// A ocorrência de um evento pintada sob `(x, y)`: o Kanban, o evento e o início dela.
    fn event_under(&self, x: f32, y: f32) -> Option<(KanbanId, EventId, Moment)> {
        let cx = self.ctx();
        let day = self.cal.anchor_or_today(&cx);
        let mid = start_of_day(day, &self.zone)?;
        let (from, to) = (Moment(mid - REACH_MS), Moment(mid + REACH_MS));
        let set = self.set.borrow();
        let docs = std::iter::once(set.agenda()).chain(set.iter());
        docs.flat_map(|k| k.events.values().map(move |e| (k.id, e)))
            .filter(|(_, e)| !e.meta.deleted)
            .find_map(|(k, e)| {
                recur::occurrences(e, from, to, &self.zone)
                    .into_iter()
                    .find(|(s, _)| self.cal.item_rects(event_key(e.id, *s)).iter().any(|r| r.contains(x, y)))
                    .map(|(s, _)| (k, e.id, s))
            })
    }

    fn hit_at(&self, x: f32, y: f32) -> Option<Hit> {
        self.hits.iter().rev().find(|(_, r)| r.contains(x, y)).map(|(h, _)| *h)
    }

    /// O dia do calendário sob `(x, y)` (a grelha pintada; procura à volta do dia que a fixa).
    fn day_at(&self, x: f32, y: f32) -> Option<ph2d_kanban_dates::CivilDate> {
        let a = self.cal.anchor_or_today(&self.ctx());
        (-42..=42)
            .map(|n| a.add_days(n))
            .find(|d| self.cal.cell_rect(*d).is_some_and(|r| r.contains(x, y)))
    }

    /// Larga `of` no dia sob o rato: um bloco às 9:00 (hoje, na próxima meia hora).
    fn drop_on_calendar(&mut self, of: BlockOf, x: f32, y: f32) -> bool {
        let Some(day) = self.day_at(x, y) else {
            return false;
        };
        let today = ph2d_kanban_dates::day_of(self.now.0, &self.zone);
        let start = if day == today {
            Some(self.next_slot().0)
        } else {
            instant(day, DROP_HOUR, 0, &self.zone).or_else(|| start_of_day(day, &self.zone))
        };
        start.is_some_and(|s| self.schedule_at(of, Moment(s)))
    }

    fn click(&mut self, hit: Hit) {
        match hit {
            Hit::Mode(mode) => self.open(mode),
            Hit::Bell => self.bell = !self.bell,
            Hit::Export => self.export = Some(self.ics()),
            Hit::NewEvent => self.edit_event(None),
            Hit::ColorBy => {
                self.show.color_by = match self.show.color_by {
                    ColorBy::Kanban => ColorBy::Label,
                    ColorBy::Label => ColorBy::Kanban,
                };
            }
            Hit::Person => {
                self.show.person = match self.show.person {
                    None => Some(self.me.clone()),
                    Some(_) => None,
                };
            }
            Hit::GroupBy => {
                self.group_by = match self.group_by {
                    crate::personal::GroupBy::Kanban => crate::personal::GroupBy::Date,
                    crate::personal::GroupBy::Date => crate::personal::GroupBy::Kanban,
                };
            }
            Hit::MarkAll => {
                self.read_all();
            }
            Hit::Notice(k) => {
                self.read_notice(k);
                self.bell = false;
            }
            Hit::InboxField => {
                self.capture.get_or_insert_with(|| LineField::new(""));
            }
            Hit::InboxToCard(i) => {
                if let Some(r) = self.probe_hit(hit) {
                    self.open_move(i, r);
                }
            }
            Hit::Schedule(of) => {
                let at = self.next_slot();
                self.schedule_at(of, at);
            }
            Hit::Card(r) => self.open_cards.push(r),
            Hit::InboxItem(_) | Hit::Pop(_) => {}
        }
    }

    /// Um clique com a pergunta do alcance aberta (ela é a primeira: está por cima de tudo).
    fn ask_down(&mut self, x: f32, y: f32) {
        let Some(q) = self.ask.as_ref() else { return };
        match q.click(x, y) {
            ScopeDone::Pick(s) => {
                self.answer(s);
            }
            ScopeDone::Cancel => self.ask = None,
            ScopeDone::Stay => {}
        }
    }

    fn dest_done(&mut self, done: DestDone) {
        match done {
            DestDone::Stay => {}
            DestDone::Close => self.dest = None,
            DestDone::Move { to, top } => {
                if let Some(d) = self.dest.take() {
                    self.inbox_to_card(d.item, to, top);
                }
            }
        }
    }

    fn pop_done(&mut self, done: Done, rect: Rect) {
        match done {
            Done::Plan(plan) => {
                self.apply(plan);
            }
            Done::Ask(ask) => self.ask_scope(ask, (rect.x + rect.w / 2.0, rect.y)),
        }
    }

    fn down(&mut self, x: f32, y: f32) -> bool {
        if self.ask.is_some() || self.dest.is_some() {
            self.pressed = Some(Hit::Pop(crate::event_pop::PopHit::Picker));
            if self.ask.is_some() {
                self.ask_down(x, y);
            } else if let Some(mut d) = self.dest.take() {
                let done = d.click(&self.set.borrow(), x, y);
                self.dest = Some(d);
                self.dest_done(done);
            }
            return true;
        }
        if let Some(mut pop) = self.pop.take() {
            // O largar deste clique é do popover, mesmo que ele feche agora.
            self.pressed = Some(Hit::Pop(crate::event_pop::PopHit::Picker));
            if pop.rect.contains(x, y) {
                let done = {
                    let set = self.set.clone();
                    let set = set.borrow();
                    pop.click(&set, &self.ctx(), x, y)
                };
                match done {
                    Some(done) => self.pop_done(done, pop.rect),
                    None => self.pop = Some(pop),
                }
            }
            return true;
        }
        let hit = self.hit_at(x, y);
        if self.bell && !matches!(hit, Some(Hit::Notice(_) | Hit::MarkAll | Hit::Bell)) {
            self.bell = false;
            return true;
        }
        if hit != Some(Hit::InboxField) && self.capture.as_ref().is_some_and(|f| f.text.is_empty()) {
            self.capture = None;
        }
        self.pressed = hit;
        if let Some(h) = hit {
            let of = match h {
                Hit::InboxItem(i) => Some(BlockOf::Inbox(i)),
                Hit::Card(r) if self.mode == Mode::Planner => Some(BlockOf::Card(r)),
                _ => None,
            };
            if let Some(of) = of {
                self.drag = Some(SideDrag { of, from: (x, y), at: (x, y) });
            }
            return true;
        }
        let on_calendar = self.calendar_area.contains(x, y) && self.mode != Mode::MyCards;
        self.cal_down = on_calendar.then_some((x, y));
        on_calendar
            && self.with_calendar((x, y), |st, src| {
                calendar::pointer(st, src, &src.cx, PointerAt { kind: VKind::Down, x, y })
            })
    }

    fn up(&mut self, x: f32, y: f32) -> bool {
        let pressed = self.pressed.take();
        let down = self.cal_down.take();
        if self.pop.is_some() || self.ask.is_some() || self.dest.is_some() || matches!(pressed, Some(Hit::Pop(_))) {
            return self.area.contains(x, y);
        }
        if let Some(g) = self.drag.take() {
            let moved = (g.at.0 - g.from.0).abs().max((g.at.1 - g.from.1).abs()) > DRAG_SLOP;
            if moved {
                if self.calendar_area.contains(x, y) {
                    self.drop_on_calendar(g.of, x, y);
                }
                return true;
            }
        }
        if let Some(h) = pressed {
            if self.hit_at(x, y) == Some(h) {
                self.click(h);
            }
            return true;
        }
        let taken = self.calendar_area.contains(x, y)
            && self.mode != Mode::MyCards
            && self.with_calendar((x, y), |st, src| {
                calendar::pointer(st, src, &src.cx, PointerAt { kind: VKind::Up, x, y })
            });
        // Um clique (sem arrasto) num evento abre-o — a vista só abre cartões.
        let click = down.is_some_and(|d| (d.0 - x).abs().max((d.1 - y).abs()) <= DRAG_SLOP);
        if click
            && self.ask.is_none()
            && let Some((k, id, at)) = self.event_under(x, y)
        {
            self.edit_occurrence(k, id, at);
        }
        taken
    }

    fn moved(&mut self, x: f32, y: f32) -> bool {
        if let Some(g) = self.drag.as_mut() {
            g.at = (x, y);
            return true;
        }
        if let Some(q) = self.ask.as_mut() {
            q.hover = q.hits().into_iter().find(|(_, r)| r.contains(x, y)).map(|(h, _)| h);
            return self.area.contains(x, y);
        }
        if let Some(mut d) = self.dest.take() {
            d.hover = d.hits(&self.set.borrow()).into_iter().find(|(_, r)| r.contains(x, y)).map(|(h, _)| h);
            self.dest = Some(d);
            return self.area.contains(x, y);
        }
        self.hover = self.hit_at(x, y);
        if self.pop.is_some() || self.hover.is_some() {
            return self.area.contains(x, y);
        }
        self.calendar_area.contains(x, y)
            && self.mode != Mode::MyCards
            && self.with_calendar((x, y), |st, src| {
                calendar::pointer(st, src, &src.cx, PointerAt { kind: VKind::Move, x, y })
            })
    }

    fn paint_all(&mut self, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
        self.area = area;
        self.hits.clear();
        let body = crate::ui::top_bar(self, area, scene, ts, theme);
        let cal = match self.mode {
            Mode::MyCards => {
                crate::side::my_cards(self, body, scene, ts, theme);
                Rect::new(body.x, body.y, 0.0, 0.0)
            }
            Mode::Planner => crate::side::planner_column(self, body, scene, ts, theme),
            Mode::Calendar => body,
        };
        self.calendar_area = cal;
        if self.mode != Mode::MyCards {
            scene.fill_rect(ph2d_editor_core::paint::rect_to_vello(cal), crate::ui::c(theme, ph2d_tokens::ColorToken::Bg1));
            let set = self.set.clone();
            let set = set.borrow();
            let show = self.show.clone();
            let mut st = std::mem::take(&mut self.cal);
            {
                let src = ProjectSource::new(&set, &show, self.ctx());
                calendar::paint(scene, ts, theme, cal, &mut st, &src, &src.cx);
            }
            self.cal = st;
        }
        crate::side::ghost(self, scene, ts, theme);
        if let Some(mut pop) = self.pop.take() {
            let anchor = self.probe_hit(Hit::NewEvent).unwrap_or(Rect::new(area.x + area.w, area.y, 0.0, m::TOP_H));
            let h = EventPop::height().min(area.h - m::TOP_H);
            let x = (anchor.x + anchor.w - m::POP_W).max(area.x + m::PAD);
            pop.rect = Rect::new(x, area.y + m::TOP_H, m::POP_W, h);
            pop.paint(scene, ts, theme);
            self.pop = Some(pop);
        }
        if self.bell {
            crate::ui::bell_box(self, area, scene, ts, theme);
        }
        if let Some(mut d) = self.dest.take() {
            let set = self.set.clone();
            let set = set.borrow();
            d.place(&set, area);
            d.paint(&set, scene, ts, theme);
            self.dest = Some(d);
        }
        if let Some(q) = self.ask.as_mut() {
            q.place(area);
            q.paint(scene, ts, theme);
        }
    }

    fn key_in(&mut self, ts: &mut TextSystem, input: &KeyInput<'_>) -> bool {
        let _ = ts;
        if self.ask.is_some() {
            match ScopePop::key(input) {
                ScopeDone::Pick(s) => {
                    self.answer(s);
                }
                ScopeDone::Cancel => self.ask = None,
                ScopeDone::Stay => {}
            }
            return true;
        }
        if let Some(mut d) = self.dest.take() {
            let done = d.key(&self.set.borrow(), input);
            self.dest = Some(d);
            self.dest_done(done);
            return true;
        }
        if let Some(mut pop) = self.pop.take() {
            let (taken, done) = {
                let set = self.set.borrow();
                pop.key(&set, &self.ctx(), input)
            };
            match done {
                Some(done) => self.pop_done(done, pop.rect),
                None => self.pop = Some(pop),
            }
            return taken;
        }
        if let Some(f) = self.capture.as_mut() {
            if !input.pressed {
                return true;
            }
            match f.key(input.key, input.text) {
                FieldKey::Commit => {
                    let text = std::mem::take(&mut f.text);
                    f.caret = 0;
                    self.capture(&text);
                }
                FieldKey::Cancel => self.capture = None,
                _ => {}
            }
            return true;
        }
        if input.pressed && input.key == Key::Escape && self.bell {
            self.bell = false;
            return true;
        }
        let mid = (self.calendar_area.x + self.calendar_area.w / 2.0, self.calendar_area.y + m::TOP_H);
        self.mode != Mode::MyCards
            && self.with_calendar(mid, |st, src| calendar::key(st, src, &src.cx, input))
    }
}

impl DocumentSurface for PlannerDocs {
    fn tabs(&self) -> Vec<DocTab> {
        if !self.open {
            return Vec::new();
        }
        vec![DocTab {
            id: TAB,
            title: tr("planner.tab").to_owned(),
            dirty: false,
        }]
    }

    fn paint(&mut self, _id: u64, area: Rect, scene: &mut VectorScene, text: &mut TextSystem, theme: &Theme) {
        self.paint_all(area, scene, text, *theme);
    }

    fn pointer(&mut self, _id: u64, _input: PointerInput<'_>, kind: PointerKind, _button: PointerButton, x: f32, y: f32) -> bool {
        match kind {
            PointerKind::Down => self.down(x, y),
            PointerKind::Up => self.up(x, y),
            PointerKind::Move => self.moved(x, y),
        }
    }

    fn pointer_move(&mut self, _id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.moved(x, y)
    }

    fn wheel(&mut self, _id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        if let Some(d) = self.dest.as_mut()
            && d.rect.contains(x, y)
        {
            d.wheel(&self.set.borrow(), dy);
            return true;
        }
        self.calendar_area.contains(x, y)
            && self.mode != Mode::MyCards
            && self.with_calendar((x, y), |st, _| calendar::wheel(st, (x, y), dy, shift))
    }

    fn key(&mut self, _id: u64, ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome {
            consumed: self.key_in(ts, &input),
            copy: None,
        }
    }

    fn compose(&mut self, _id: u64, c: Compose<'_>) -> bool {
        let Compose::Commit(ch) = c else { return false };
        let s = ch.to_string();
        if let Some(d) = self.dest.as_mut() {
            d.insert(&self.set.borrow(), &s);
            return true;
        }
        if let Some(pop) = self.pop.as_mut().filter(|p| p.editing) {
            pop.title.insert(&s);
            return true;
        }
        if let Some(f) = self.capture.as_mut() {
            f.insert(&s);
            return true;
        }
        false
    }

    fn close(&mut self, _id: u64) {
        self.open = false;
        self.pop = None;
        self.dest = None;
        self.ask = None;
        self.capture = None;
        self.bell = false;
    }

    fn undo(&mut self, _id: u64, redo: bool) -> bool {
        PlannerDocs::undo(self, redo)
    }

    fn a11y(&self, id: u64) -> Vec<(NodeId, ph2d_a11y::Node)> {
        let a = self.area;
        let node = ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Document)
            .label(tr("planner.tab").to_owned())
            .bounds(f64::from(a.x), f64::from(a.y), f64::from(a.w), f64::from(a.h))
            .build();
        vec![(NodeId(crate::A11Y_SALT ^ id), node)]
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    fn family_commands(&self) -> Vec<DocCommand> {
        [
            (CMD_CALENDAR, "planner.cmd.calendar"),
            (CMD_PLANNER, "planner.cmd.planner"),
            (CMD_MY_CARDS, "planner.cmd.my_cards"),
            (CMD_INBOX, "planner.cmd.inbox"),
        ]
        .into_iter()
        .map(|(id, key)| DocCommand {
            id,
            label: tr(key).to_owned(),
            menu: DocMenu::File,
        })
        .collect()
    }

    fn family_command(&mut self, cmd: NodeId) -> bool {
        let mode = if cmd == CMD_CALENDAR {
            Mode::Calendar
        } else if cmd == CMD_PLANNER || cmd == CMD_INBOX {
            Mode::Planner
        } else if cmd == CMD_MY_CARDS {
            Mode::MyCards
        } else {
            return false;
        };
        self.open(mode);
        if cmd == CMD_INBOX {
            self.capture = Some(LineField::new(""));
        }
        true
    }

    fn pointer_look(&self, _id: u64, x: f32, y: f32) -> Option<PointerLook> {
        if !self.area.contains(x, y) {
            return None;
        }
        Some(match self.hit_at(x, y) {
            Some(Hit::InboxField) => PointerLook::Text,
            Some(_) => PointerLook::Hand,
            None => PointerLook::Arrow,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
