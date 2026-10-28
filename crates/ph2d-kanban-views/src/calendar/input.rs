//! O rato, a roda e o teclado do calendário: navegar, arrastar um item para outro dia, o «+» e o
//! compositor, o «+N more» e o popover do dia. Tudo sobre as peças do último quadro pintado.

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, FieldKey, LineField};
use ph2d_i18n::tr;
use ph2d_kanban_dates::{CivilDate, add_days, instant};
use ph2d_kanban_model::Moment;

use super::layout::Mode;
use super::metrics as m;
use super::{CalendarState, Composer, Press};
use crate::common::{Ctx, PointerAt, PointerKind, Reply};
use crate::datepick::node;
use crate::item::{ItemKey, Source};

/// O sal das ids a11y do compositor.
const COMPOSER_SALT: u64 = 0xCA1E_0000;

/// Anda `dir` meses (ou semanas) a partir do que está à vista.
pub(crate) fn step(st: &mut CalendarState, from: CivilDate, dir: i64) {
    let next = match st.mode {
        Mode::Week => from.add_days(7 * dir),
        Mode::Day => from.add_days(dir),
        Mode::Agenda => from.add_days(m::AGENDA_DAYS * dir),
        Mode::Month => {
            let months = i64::from(from.year) * 12 + i64::from(from.month) - 1 + dir;
            let (y, mo) = (months.div_euclid(12), months.rem_euclid(12) + 1);
            match (i16::try_from(y), u8::try_from(mo)) {
                (Ok(y), Ok(mo)) => CivilDate::new(y, mo, 1).unwrap_or(from),
                _ => from,
            }
        }
    };
    st.anchor = Some(next);
    st.day_pop = None;
    st.composer = None;
}

fn shown(st: &CalendarState, cx: &Ctx<'_>) -> CivilDate {
    st.painted.anchor.unwrap_or_else(|| st.anchor_or_today(cx))
}

pub(crate) fn cell_at(st: &CalendarState, x: f32, y: f32) -> Option<CivilDate> {
    st.painted
        .cells
        .iter()
        .find(|(r, _)| r.contains(x, y))
        .map(|(_, d)| *d)
}

/// Abre o compositor do dia `day` (`span` = o início e a entrega do bloco, na grelha de horas;
/// `at_y` = onde o painel abre).
pub(crate) fn open_composer(
    st: &mut CalendarState,
    src: &dyn Source,
    day: CivilDate,
    span: (Option<Moment>, Option<Moment>),
    at_y: Option<f32>,
) {
    let targets = src.targets();
    let options = targets
        .iter()
        .enumerate()
        .map(|(i, t)| DropdownOption::new(node(COMPOSER_SALT ^ (i as u64 + 2)), i, t.name.clone()))
        .collect();
    let mut list = Dropdown::new(
        node(COMPOSER_SALT ^ 1),
        tr("kanban.views.calendar.list"),
        options,
    );
    if !targets.is_empty() {
        list.select(0);
    }
    st.composer = Some(Composer {
        day,
        field: LineField::default(),
        list,
        targets,
        start: span.0,
        due: span.1,
        at_y,
    });
    st.day_pop = None;
}

/// O compositor confirmado: o cartão novo na lista escolhida, com a entrega ao meio-dia do dia (ou o
/// bloco da grelha de horas).
fn commit(st: &mut CalendarState, src: &dyn Source, cx: &Ctx<'_>) -> Reply {
    let Some(c) = st.composer.take() else {
        return Reply::default();
    };
    let target = c.list.selected.and_then(|i| c.targets.get(i));
    let due = c
        .due
        .or_else(|| instant(c.day, m::NEW_DUE_HOUR, 0, cx.zone).map(Moment));
    match (target, due) {
        (Some(t), Some(due)) if !c.field.text.trim().is_empty() => {
            Reply::with_plan(src.create_span(t, &c.field.text, c.start, due))
        }
        _ => Reply::taken(),
    }
}

/// Um clique com o compositor aberto.
fn composer_down(st: &mut CalendarState, src: &dyn Source, cx: &Ctx<'_>, x: f32, y: f32) -> Reply {
    let r = st.painted.pop.clone();
    let Some(c) = st.composer.as_mut() else {
        return Reply::default();
    };
    if c.list.open {
        c.list.open = false;
        if let Some(i) = (0..c.list.options.len()).find(|i| c.list.option_rect(r.chip, *i).contains(x, y)) {
            c.list.select(i);
        }
        return Reply::taken();
    }
    if r.chip.contains(x, y) {
        c.list.open = true;
        return Reply::taken();
    }
    if r.add.contains(x, y) {
        return commit(st, src, cx);
    }
    if r.cancel.contains(x, y) || !r.panel.contains(x, y) {
        st.composer = None;
    }
    Reply::taken()
}

/// Um clique com o popover de um dia aberto.
fn day_pop_down(st: &mut CalendarState, src: &dyn Source, x: f32, y: f32) -> Reply {
    let r = st.painted.pop.clone();
    if let Some((_, key)) = r.rows.iter().find(|(rr, _)| rr.contains(x, y)) {
        st.day_pop = None;
        return match key {
            ItemKey::Card(c) => Reply::open(*c),
            ItemKey::Event(_) => Reply::taken(),
        };
    }
    if let Some(day) = st.day_pop
        && r.add.contains(x, y)
    {
        open_composer(st, src, day, (None, None), None);
        return Reply::taken();
    }
    st.day_pop = None;
    Reply::taken()
}

fn down(st: &mut CalendarState, src: &dyn Source, cx: &Ctx<'_>, x: f32, y: f32) -> Reply {
    if st.composer.is_some() {
        return composer_down(st, src, cx, x, y);
    }
    if st.day_pop.is_some() {
        return day_pop_down(st, src, x, y);
    }
    let p = st.painted.clone();
    let from = shown(st, cx);
    if p.prev.contains(x, y) {
        step(st, from, -1);
        return Reply::taken();
    }
    if p.next.contains(x, y) {
        step(st, from, 1);
        return Reply::taken();
    }
    if p.today.contains(x, y) {
        st.anchor = None;
        return Reply::taken();
    }
    if let Some((_, mode)) = p.modes.iter().find(|(r, _)| r.contains(x, y)) {
        st.mode = *mode;
        st.hold = None;
        return Reply::taken();
    }
    if let Some((r, day)) = p.plus
        && r.contains(x, y)
    {
        open_composer(st, src, day, (None, None), None);
        return Reply::taken();
    }
    if let Some((_, i)) = p.agenda.iter().find(|(r, _)| r.contains(x, y)) {
        return match p.items.get(*i).map(|it| it.key) {
            Some(ItemKey::Card(c)) => Reply::open(c),
            _ => Reply::taken(),
        };
    }
    if let Some((_, day, _)) = p.more.iter().find(|(r, _, _)| r.contains(x, y)) {
        st.day_pop = Some(*day);
        return Reply::taken();
    }
    if let Some((r, i)) = p.segs.iter().rev().find(|(r, _)| r.contains(x, y))
        && let (Some(item), Some(day)) = (p.items.get(*i), cell_at(st, x, y))
    {
        st.press = Some(Press {
            item: item.clone(),
            day,
            at: (x, y),
            grab: (x - r.x, y - r.y),
            size: (r.w, r.h),
            moved: false,
        });
        return Reply::taken();
    }
    if st.mode.hours() {
        return super::hours_input::down(st, x, y);
    }
    Reply::default()
}

/// O arrasto largado: o item anda os dias entre a célula agarrada e a do rato, à MESMA hora.
fn up(st: &mut CalendarState, src: &dyn Source, cx: &Ctx<'_>, x: f32, y: f32) -> Reply {
    if st.hold.is_some() {
        return super::hours_input::up(st, src, x, y);
    }
    let Some(p) = st.press.take() else {
        return Reply::default();
    };
    if !p.moved {
        return match p.item.key {
            ItemKey::Card(c) => Reply::open(c),
            ItemKey::Event(_) => Reply::taken(),
        };
    }
    let Some(to) = cell_at(st, x, y) else {
        return Reply::taken();
    };
    let d = p.day.days_until(to);
    if d == 0 {
        return Reply::taken();
    }
    let start = p.item.start.map(|s| Moment(add_days(s.0, d, cx.zone)));
    let end = Moment(add_days(p.item.end.0, d, cx.zone));
    Reply::with_plan(src.reschedule(p.item.key, start, end))
}

pub(crate) fn pointer(st: &mut CalendarState, src: &dyn Source, cx: &Ctx<'_>, p: PointerAt) -> Reply {
    match p.kind {
        PointerKind::Down => down(st, src, cx, p.x, p.y),
        PointerKind::Move => {
            let changed = st.hover != Some((p.x, p.y));
            st.hover = Some((p.x, p.y));
            if let Some(h) = st.hold.as_mut() {
                if (p.x - h.at.0).hypot(p.y - h.at.1) > m::DRAG_SLOP {
                    h.moved = true;
                }
                return Reply::taken();
            }
            if let Some(press) = st.press.as_mut() {
                let (dx, dy) = (p.x - press.at.0, p.y - press.at.1);
                if dx.hypot(dy) > m::DRAG_SLOP {
                    press.moved = true;
                }
                return Reply::taken();
            }
            Reply {
                redraw: changed,
                ..Reply::default()
            }
        }
        PointerKind::Up => up(st, src, cx, p.x, p.y),
    }
}

pub(crate) fn wheel(st: &mut CalendarState, dy: f32) -> Reply {
    let Some(from) = st.painted.anchor else {
        return Reply::default();
    };
    match st.mode {
        Mode::Week | Mode::Day => {
            st.scroll = Some(st.scroll.unwrap_or_default() + dy);
            return Reply::taken();
        }
        Mode::Agenda => {
            st.agenda_scroll += dy;
            return Reply::taken();
        }
        Mode::Month => {}
    }
    st.wheel += dy;
    let mut dir = 0;
    while st.wheel >= m::WHEEL_STEP {
        st.wheel -= m::WHEEL_STEP;
        dir += 1;
    }
    while st.wheel <= -m::WHEEL_STEP {
        st.wheel += m::WHEEL_STEP;
        dir -= 1;
    }
    if dir == 0 {
        return Reply::taken();
    }
    step(st, from, dir);
    Reply::taken()
}

pub(crate) fn key(
    st: &mut CalendarState,
    src: &dyn Source,
    cx: &Ctx<'_>,
    input: &KeyInput<'_>,
) -> Reply {
    if !input.pressed {
        return Reply::default();
    }
    if let Some(c) = st.composer.as_mut() {
        return match c.field.key(input.key, input.text) {
            FieldKey::Commit => commit(st, src, cx),
            FieldKey::Cancel => {
                st.composer = None;
                Reply::taken()
            }
            FieldKey::Edited | FieldKey::Moved | FieldKey::Ignored => Reply::taken(),
        };
    }
    let from = shown(st, cx);
    match input.key {
        Key::Escape if st.day_pop.is_some() || st.press.is_some() => {
            st.day_pop = None;
            st.press = None;
            Reply::taken()
        }
        Key::Left | Key::PageUp => {
            step(st, from, -1);
            Reply::taken()
        }
        Key::Right | Key::PageDown => {
            step(st, from, 1);
            Reply::taken()
        }
        _ => Reply::default(),
    }
}
