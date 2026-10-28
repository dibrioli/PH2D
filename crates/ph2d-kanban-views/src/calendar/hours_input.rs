//! O rato na grelha de horas: mover um bloco (outra hora, outro dia; a duração fica), esticar a
//! borda de baixo (só o fim) e arrastar no vazio (um bloco novo: o compositor abre com a hora).
//! Tudo encaixado a 15 min.

use ph2d_kanban_dates::{CivilDate, day_of};
use ph2d_kanban_model::Moment;

use super::hours::{at, mins_ms, snap};
use super::input::open_composer;
use super::metrics as m;
use super::{CalendarState, Hold, HourDrag};
use crate::common::Reply;
use crate::item::{ItemKey, Source};

/// Carregar na grelha de horas: num bloco (mover ou, na borda de baixo, esticar) ou no vazio.
pub(crate) fn down(st: &mut CalendarState, x: f32, y: f32) -> Reply {
    let Some(g) = st.painted.hours.as_ref() else {
        return Reply::default();
    };
    if !g.body.contains(x, y) {
        return Reply::default();
    }
    let hit = st.painted.blocks.iter().rev().find(|(r, _)| r.contains(x, y));
    let drag = if let Some((r, b)) = hit {
        let Some(item) = st.painted.items.get(b.item).cloned() else {
            return Reply::default();
        };
        let top = if b.has_start {
            item.start.unwrap_or(item.end)
        } else {
            item.end
        };
        if y >= r.y + r.h - m::RESIZE_H {
            HourDrag::Resize { item, top }
        } else {
            HourDrag::Move {
                item,
                top,
                has_start: b.has_start,
                grab: g.min_of(y) - b.top_min,
            }
        }
    } else {
        let Some((_, day)) = g.col_at(x) else {
            return Reply::default();
        };
        HourDrag::Create {
            day,
            from: g.min_of(y),
        }
    };
    st.hold = Some(Hold {
        drag,
        at: (x, y),
        moved: false,
    });
    Reply::taken()
}

/// O que o gesto em curso mostra: a coluna e os minutos de cima e de baixo (sem a altura mínima).
pub(crate) fn preview(st: &CalendarState) -> Option<(usize, f32, f32)> {
    let h = st.hold.as_ref().filter(|h| h.moved)?;
    let g = st.painted.hours.as_ref()?;
    let (hx, hy) = st.hover?;
    let zone = g.zone.as_ref()?;
    #[allow(clippy::cast_precision_loss)]
    let f = |n: i64| n as f32;
    match &h.drag {
        HourDrag::Move { item, top, grab, .. } => {
            let (col, _) = g.col_at(hx)?;
            let t = f(snap(g.min_of(hy) - grab));
            let dur = f((item.end.0 - top.0) / mins_ms(1));
            Some((col, t, t + dur))
        }
        HourDrag::Resize { top, .. } => {
            let d = day_of(top.0, zone);
            let col = g.cols.iter().position(|(_, c)| *c == d)?;
            let t = super::hours::minutes(top.0, zone);
            Some((col, t, f(snap(g.min_of(hy))).max(t + f(m::SNAP_MIN))))
        }
        HourDrag::Create { day, from } => {
            let col = g.cols.iter().position(|(_, c)| c == day)?;
            let (a, b) = (snap(*from), snap(g.min_of(hy)));
            let (lo, hi) = (a.min(b), a.max(b).max(a.min(b) + m::SNAP_MIN));
            Some((col, f(lo), f(hi)))
        }
    }
}

fn open(key: ItemKey) -> Reply {
    match key {
        ItemKey::Card(c) => Reply::open(c),
        ItemKey::Event(_) => Reply::taken(),
    }
}

/// Largar: o lote da mudança (mover, esticar) ou o compositor aberto (criar).
pub(crate) fn up(st: &mut CalendarState, src: &dyn Source, x: f32, y: f32) -> Reply {
    let Some(h) = st.hold.take() else {
        return Reply::default();
    };
    let Some(g) = st.painted.hours.clone() else {
        return Reply::taken();
    };
    let Some(zone) = g.zone.clone() else {
        return Reply::taken();
    };
    match h.drag {
        HourDrag::Move { item, .. } | HourDrag::Resize { item, .. } if !h.moved => open(item.key),
        HourDrag::Move {
            item,
            top,
            has_start,
            grab,
        } => {
            let Some((_, date)) = g.col_at(x) else {
                return Reply::taken();
            };
            let Some(t) = at(date, snap(g.min_of(y) - grab), &zone) else {
                return Reply::taken();
            };
            let end = Moment(t.0 + (item.end.0 - top.0));
            if t == top {
                return Reply::taken();
            }
            Reply::with_plan(src.reschedule(item.key, has_start.then_some(t), end))
        }
        HourDrag::Resize { item, top } => {
            let date = day_of(top.0, &zone);
            let floor = Moment(top.0 + mins_ms(m::SNAP_MIN));
            let Some(end) = at(date, snap(g.min_of(y)), &zone).map(|e| e.max(floor)) else {
                return Reply::taken();
            };
            if end == item.end {
                return Reply::taken();
            }
            Reply::with_plan(src.reschedule(item.key, Some(top), end))
        }
        HourDrag::Create { day, from } => {
            let (a, b) = (snap(from), snap(g.min_of(y)));
            let span = if h.moved {
                let (lo, hi) = (a.min(b), a.max(b).max(a.min(b) + m::SNAP_MIN));
                (at(day, lo, &zone), at(day, hi, &zone))
            } else {
                (None, at(day, a, &zone))
            };
            let below = g.y_of(f32_of(a.max(b)));
            create(st, src, day, span, below)
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn f32_of(n: i64) -> f32 {
    n as f32
}

fn create(
    st: &mut CalendarState,
    src: &dyn Source,
    day: CivilDate,
    span: (Option<Moment>, Option<Moment>),
    y: f32,
) -> Reply {
    if span.1.is_none() {
        return Reply::taken();
    }
    open_composer(st, src, day, span, Some(y));
    Reply::taken()
}
