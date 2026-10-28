//! **O popover do evento** — criar ou editar um evento próprio do Calendário: o título, a duração
//! (30 min · 1 h · 2 h · dia inteiro), a recorrência, e o seletor de datas da W7 (o MESMO do verso do
//! cartão) para o dia, a hora e o lembrete. «Start date» ligado = um evento de vários dias (do início
//! à data).

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, stroke_rounded_rect};
use ph2d_editor_core::widget::{
    DatePickerEvent, DatePickerLayout, FieldKey, LineField, Popover, paint_date_picker,
    paint_popover,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_dates::{CivilDate, Zone, civil, day_of, start_of_day};
use ph2d_kanban_model::{
    Event, EventId, KanbanId, KanbanOp, KanbanSet, Moment, Plan, Record, RecordId, Repeat,
};
use ph2d_kanban_views::Ctx;
use ph2d_kanban_views::datepick::Picker;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use crate::metrics as m;
use crate::ui::{c, text_top};

/// As durações oferecidas (minutos); a última é o dia inteiro.
pub const DURATIONS: [u32; 3] = [30, 60, 120];
const REPEATS: [Option<Repeat>; 5] = [
    None,
    Some(Repeat::Daily),
    Some(Repeat::Weekly),
    Some(Repeat::Monthly),
    Some(Repeat::Yearly),
];

/// Um sítio do popover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopHit {
    Title,
    /// 0..=2 = [`DURATIONS`], 3 = o dia inteiro.
    Duration(u8),
    Repeat(u8),
    Picker,
}

/// O popover aberto.
#[derive(Clone, Debug)]
pub struct EventPop {
    pub base: Option<Event>,
    pub title: LineField,
    pub editing: bool,
    pub picker: Picker,
    pub minutes: u32,
    pub all_day: bool,
    pub repeat: Option<Repeat>,
    pub rect: Rect,
}

fn repeat_key(r: Option<Repeat>) -> &'static str {
    match r {
        None => "planner.repeat.none",
        Some(Repeat::Daily) => "planner.repeat.daily",
        Some(Repeat::Weekly) => "planner.repeat.weekly",
        Some(Repeat::Monthly) => "planner.repeat.monthly",
        Some(Repeat::Yearly) => "planner.repeat.yearly",
    }
}

impl EventPop {
    /// O popover de `e` (`None` = um evento novo que começa em `slot`).
    #[must_use]
    pub fn new(e: Option<Event>, slot: Moment, cx: &Ctx<'_>) -> Self {
        let (start, minutes, all_day, repeat, remind) = e.as_ref().map_or(
            (slot, DURATIONS[1], false, None, None),
            |e| {
                let mins = u32::try_from((e.end.0 - e.start.0) / 60_000).unwrap_or(60);
                (e.start, mins, e.all_day, e.repeat, e.remind_before_min)
            },
        );
        let zone = if all_day { Zone::utc() } else { cx.zone.clone() };
        let multi = all_day && minutes > 24 * 60;
        let last = e.as_ref().map_or(start, |e| Moment(e.end.0 - 1));
        let mut picker = Picker::new(
            0x5045_4F50,
            multi.then_some(start),
            Some((if multi { last } else { start }, remind)),
            cx.now,
            &zone,
            cx.week_start,
        );
        // O seletor do verso fala de «entrega»; aqui é o dia do evento.
        let l = &mut picker.widget.labels;
        l.title = tr("planner.event.dates").to_owned();
        l.start = tr("planner.event.first_day").to_owned();
        l.due = tr("planner.event.day").to_owned();
        l.reminder = tr("planner.event.reminder").to_owned();
        l.remove = tr("planner.event.delete").to_owned();
        Self {
            title: LineField::new(e.as_ref().map_or_else(String::new, |e| e.title.clone())),
            editing: e.is_none(),
            base: e,
            picker,
            minutes,
            all_day,
            repeat,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    fn rows(&self) -> (Rect, Rect, Rect, Rect) {
        let r = self.rect;
        let x = r.x + m::PAD;
        let w = r.w - 2.0 * m::PAD;
        let title = Rect::new(x, r.y + m::PAD, w, m::FIELD_H);
        let dur = Rect::new(x, title.y + title.h + m::GAP, w, m::BUTTON_H);
        let rep = Rect::new(x, dur.y + dur.h + m::GAP, w, m::BUTTON_H);
        let pick_y = rep.y + rep.h + m::GAP;
        let picker = Rect::new(r.x, pick_y, r.w, DatePickerLayout::preferred_height());
        (title, dur, rep, picker)
    }

    /// A altura que o popover pede.
    #[must_use]
    pub fn height() -> f32 {
        m::PAD + m::FIELD_H + 2.0 * (m::GAP + m::BUTTON_H) + m::GAP
            + DatePickerLayout::preferred_height()
    }

    fn chips(row: Rect, n: usize) -> Vec<Rect> {
        #[allow(clippy::cast_precision_loss)]
        let w = (row.w - m::GAP * (n as f32 - 1.0)) / n as f32;
        #[allow(clippy::cast_precision_loss)]
        (0..n)
            .map(|i| Rect::new(row.x + i as f32 * (w + m::GAP), row.y, w, row.h))
            .collect()
    }

    /// Os sítios de clique.
    #[must_use]
    pub fn hits(&self) -> Vec<(PopHit, Rect)> {
        let (title, dur, rep, picker) = self.rows();
        let mut v = vec![(PopHit::Title, title), (PopHit::Picker, picker)];
        for (i, r) in Self::chips(dur, 4).into_iter().enumerate() {
            v.push((PopHit::Duration(u8::try_from(i).unwrap_or(0)), r));
        }
        for (i, r) in Self::chips(rep, REPEATS.len()).into_iter().enumerate() {
            v.push((PopHit::Repeat(u8::try_from(i).unwrap_or(0)), r));
        }
        v
    }

    pub fn paint(&self, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
        let (title, dur, rep, picker) = self.rows();
        let frame = Rect::new(self.rect.x, self.rect.y, self.rect.w, picker.y - self.rect.y);
        paint_popover(&Popover::new(crate::POP_NODE), frame, scene, theme);
        fill_rounded_rect(scene, title, m::RADIUS, c(theme, T::Bg2));
        if self.editing {
            stroke_rounded_rect(scene, title, m::RADIUS, 1.0, c(theme, T::Accent));
        }
        let (text, ink) = if self.title.text.is_empty() {
            (tr("planner.event.title_hint"), T::Text2)
        } else {
            (self.title.text.as_str(), T::Text1)
        };
        let ty = text_top(ts, title.y, title.h, m::TEXT_PX);
        paint_text_elided(ts, scene, text, title.x + m::BUTTON_PAD_X, ty, m::TEXT_PX, title.w - 2.0 * m::BUTTON_PAD_X, c(theme, ink));
        let durs = Self::chips(dur, 4);
        for (i, r) in durs.into_iter().enumerate() {
            let on = if i == 3 { self.all_day } else { !self.all_day && self.minutes == DURATIONS[i] };
            let label = if i == 3 {
                tr("planner.event.all_day").to_owned()
            } else {
                ph2d_i18n::tr_with("planner.event.minutes", &[("n", &DURATIONS[i])])
            };
            chip(scene, ts, theme, r, &label, on);
        }
        for (i, r) in Self::chips(rep, REPEATS.len()).into_iter().enumerate() {
            chip(scene, ts, theme, r, tr(repeat_key(REPEATS[i])), self.repeat == REPEATS[i]);
        }
        paint_date_picker(&self.picker.widget, picker, scene, ts, theme);
    }

    fn zone(&self, cx: &Ctx<'_>) -> Zone {
        if self.all_day { Zone::utc() } else { cx.zone.clone() }
    }

    /// Um clique. `Some(plan)` = o popover fechou (o plano pode estar vazio: «Close»).
    pub fn click(&mut self, set: &KanbanSet, cx: &Ctx<'_>, x: f32, y: f32) -> Option<Plan> {
        let hit = self.hits().into_iter().find(|(_, r)| r.contains(x, y)).map(|(h, _)| h);
        self.editing = hit == Some(PopHit::Title);
        match hit? {
            PopHit::Title => None,
            PopHit::Duration(i) => {
                let i = usize::from(i);
                self.all_day = i == 3;
                if i < 3 {
                    self.minutes = DURATIONS[i];
                }
                None
            }
            PopHit::Repeat(i) => {
                self.repeat = REPEATS[usize::from(i)];
                None
            }
            PopHit::Picker => {
                let (_, _, _, rect) = self.rows();
                let events = self.picker.widget.pointer_down(rect, x, y);
                self.picker_events(set, cx, events)
            }
        }
    }

    fn picker_events(&mut self, set: &KanbanSet, cx: &Ctx<'_>, events: Vec<DatePickerEvent>) -> Option<Plan> {
        let zone = self.zone(cx);
        let today = day_of(cx.now.0, &zone);
        for ev in events {
            match ev {
                DatePickerEvent::Month(e) => self.picker.month_event(e, cx.now, &zone),
                DatePickerEvent::Pick { slot, date } => self.picker.picked(slot, date),
                DatePickerEvent::TextCommitted { field, text } => {
                    self.picker.typed(field, &text, today);
                }
                DatePickerEvent::Toggled { .. } | DatePickerEvent::Reminder(_) => {}
                DatePickerEvent::Save => return Some(self.save(set, cx)),
                DatePickerEvent::Remove => return Some(self.remove()),
                DatePickerEvent::Close => return Some(Plan::default()),
            }
        }
        None
    }

    /// O evento que o popover descreve agora (`None` = sem data).
    #[must_use]
    pub fn event(&self, set: &KanbanSet, cx: &Ctx<'_>) -> Option<Event> {
        let zone = self.zone(cx);
        let (first, at) = self.picker.read(&zone);
        let (at, remind) = at?;
        let id = self.base.as_ref().map_or_else(|| EventId(set.agenda().mint()), |e| e.id);
        let title = self.title.text.trim();
        let title = if title.is_empty() { tr("planner.event.untitled") } else { title };
        let (start, end) = if self.all_day {
            let last: CivilDate = civil(at.0, &zone).date;
            let first = first.map_or(last, |f| day_of(f.0, &zone)).min(last);
            let s = start_of_day(first, &zone)?;
            let e = start_of_day(last.add_days(1), &zone)?;
            (Moment(s), Moment(e))
        } else {
            (at, Moment(at.0 + i64::from(self.minutes) * 60_000))
        };
        let mut e = self.base.clone().unwrap_or_else(|| Event::new(id, title, start, end, self.all_day));
        e.title = title.to_owned();
        (e.start, e.end, e.all_day) = (start, end, self.all_day);
        e.repeat = self.repeat;
        e.remind_before_min = remind;
        Some(e)
    }

    fn save(&self, set: &KanbanSet, cx: &Ctx<'_>) -> Plan {
        let mut plan = Plan::default();
        if let Some(e) = self.event(set, cx) {
            plan.push(KanbanId::AGENDA, KanbanOp::put(Record::Event(e)));
        }
        plan
    }

    fn remove(&self) -> Plan {
        let mut plan = Plan::default();
        if let Some(e) = &self.base {
            plan.push(KanbanId::AGENDA, KanbanOp::Delete(RecordId::Event(e.id)));
        }
        plan
    }

    /// Uma tecla: o título em edição, ou a caixa do seletor. `Some(plan)` = fechou.
    pub fn key(&mut self, set: &KanbanSet, cx: &Ctx<'_>, input: &KeyInput<'_>) -> (bool, Option<Plan>) {
        if !input.pressed {
            return (true, None);
        }
        if self.editing {
            return match self.title.key(input.key, input.text) {
                FieldKey::Commit => (true, Some(self.save(set, cx))),
                FieldKey::Cancel => (true, Some(Plan::default())),
                _ => (true, None),
            };
        }
        if input.key == Key::Escape {
            return (true, Some(Plan::default()));
        }
        let (taken, ev) = self.picker.widget.key(input);
        let plan = ev.and_then(|e| self.picker_events(set, cx, vec![e]));
        (taken, plan)
    }
}

/// Um botão de escolha (aceso = o escolhido).
pub(crate) fn chip(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, r: Rect, label: &str, on: bool) {
    let (bg, fg) = if on { (T::Accent, T::AccentFg) } else { (T::Bg2, T::Text1) };
    fill_rounded_rect(scene, r, m::RADIUS, c(theme, bg));
    let w = ts.prefix_width(label, m::SMALL_PX).min(r.w);
    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_elided(ts, scene, label, r.x + (r.w - w) / 2.0, ty, m::SMALL_PX, r.w, c(theme, fg));
}
