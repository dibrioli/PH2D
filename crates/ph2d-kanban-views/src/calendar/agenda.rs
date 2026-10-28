//! ⭐ A **agenda**: os próximos itens (a partir do dia que fixa a vista, quatro semanas), por ordem
//! da entrega, agrupados por dia. Um clique abre o cartão; a roda rola.

use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello,
    stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{CivilDate, civil, day_of, start_of_day};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use super::hours::{Kind, kind};
use super::metrics as m;
use super::paint::tone;
use super::{CalendarState, Painted};
use crate::common::{Ctx, c, text_top};
use crate::datepick::{day_text, time_text, weekday_name};
use crate::item::Item;

fn hm(ms: i64, cx: &Ctx<'_>) -> String {
    let t = civil(ms, cx.zone);
    time_text(t.hour, t.minute)
}

/// A coluna da hora: «10:00 AM – 11:30 AM», «3:00 PM», «Oct 27 – Nov 1» ou «All day».
fn when(it: &Item, today: CivilDate, cx: &Ctx<'_>) -> String {
    let span =
        |a: String, b: String| tr_with("kanban.views.calendar.span", &[("a", &a), ("b", &b)]);
    match (kind(it, cx.zone), it.start) {
        (
            Kind::Timed {
                has_start: true,
                top,
            },
            _,
        ) => span(hm(top.0, cx), hm(it.end.0, cx)),
        (Kind::Timed { .. }, _) => hm(it.end.0, cx),
        (Kind::AllDay, Some(s)) => span(
            day_text(day_of(s.0, cx.zone), today),
            day_text(day_of(it.end.0, cx.zone), today),
        ),
        (Kind::AllDay, None) => tr("kanban.views.calendar.all_day").to_owned(),
    }
}

/// Os índices dos itens da agenda, pela ordem (a entrega, depois o título), a partir de `from`.
#[must_use]
pub(crate) fn order(items: &[Item], from: i64) -> Vec<usize> {
    let mut v: Vec<usize> = (0..items.len())
        .filter(|i| items[*i].end.0 >= from)
        .collect();
    v.sort_by(|a, b| {
        items[*a]
            .end
            .cmp(&items[*b].end)
            .then(items[*a].title.cmp(&items[*b].title))
    });
    v
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut CalendarState,
    p: &mut Painted,
    items: Vec<Item>,
    cx: &Ctx<'_>,
) {
    let today = day_of(cx.now.0, cx.zone);
    let anchor = p.anchor.unwrap_or(today);
    let from = start_of_day(anchor, cx.zone).unwrap_or(cx.now.0);
    let ids = order(&items, from);
    fill_rounded_rect(out, area, m::GRID_RADIUS, c(theme, T::KanbanCard));
    let lc = c(theme, T::Border);
    let mut groups: Vec<(CivilDate, Vec<usize>)> = Vec::new();
    for i in ids {
        let d = day_of(items[i].end.0, cx.zone);
        match groups.last_mut() {
            Some((g, v)) if *g == d => v.push(i),
            _ => groups.push((d, vec![i])),
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let content: f32 = groups
        .iter()
        .map(|(_, v)| m::AGENDA_HEAD_H + v.len() as f32 * m::AGENDA_ROW_H)
        .sum();
    let max = (content - area.h + 2.0 * m::CELL_PAD).max(0.0);
    st.agenda_scroll = st.agenda_scroll.clamp(0.0, max);
    p.agenda_view = area;
    p.agenda_h = content;
    if groups.is_empty() {
        let t = tr("kanban.views.calendar.nothing");
        let w = ts.prefix_width(t, m::AGENDA_PX);
        let y = text_top(ts, area.y, area.h, m::AGENDA_PX);
        paint_text_elided(
            ts,
            out,
            t,
            area.x + (area.w - w) / 2.0,
            y,
            m::AGENDA_PX,
            area.w,
            c(theme, T::KanbanCardMeta),
        );
    }
    let whens: Vec<(usize, String)> = groups
        .iter()
        .flat_map(|(_, v)| v.iter())
        .map(|i| (*i, when(&items[*i], today, cx)))
        .collect();
    let time_w = whens
        .iter()
        .map(|(_, t)| ts.prefix_width(t, m::AGENDA_PX) + m::POP_PAD)
        .fold(m::AGENDA_TIME_W, f32::max);
    out.push_clip(&rect_to_vello(area));
    let x0 = area.x + m::POP_PAD;
    let w = area.w - 2.0 * m::POP_PAD;
    let mut y = area.y + m::CELL_PAD - st.agenda_scroll;
    for (d, v) in &groups {
        let head = Rect::new(x0, y, w, m::AGENDA_HEAD_H);
        let title = tr_with(
            "kanban.views.calendar.day_title",
            &[
                ("wd", &weekday_name(d.weekday())),
                ("day", &day_text(*d, today)),
            ],
        );
        let ink = if *d == today {
            c(theme, T::KanbanBlueBold)
        } else {
            c(theme, T::KanbanCardFg)
        };
        let ty = text_top(ts, head.y, head.h, m::AGENDA_PX);
        paint_text_title_elided(ts, out, &title, head.x, ty, m::AGENDA_PX, w, ink);
        out.fill_rect(
            rect_to_vello(Rect::new(
                head.x,
                head.y + head.h - m::GRID_LINE,
                w,
                m::GRID_LINE,
            )),
            lc,
        );
        y += m::AGENDA_HEAD_H;
        for i in v {
            let it = &items[*i];
            let r = Rect::new(x0, y, w, m::AGENDA_ROW_H);
            let ty = text_top(ts, r.y, r.h, m::AGENDA_PX);
            let meta = c(theme, T::KanbanCardMeta);
            let w_text = whens
                .iter()
                .find(|(j, _)| j == i)
                .map_or("", |(_, t)| t.as_str());
            paint_text_elided(ts, out, w_text, r.x, ty, m::AGENDA_PX, time_w, meta);
            let (dx, dy) = (r.x + time_w + m::AGENDA_DOT_R, r.y + r.h / 2.0);
            if it.done {
                fill_circle(out, dx, dy, m::DONE_R, c(theme, T::KanbanDueDone));
                let pts = m::DONE_TICK.map(|(a, b)| (dx + a * m::DONE_R, dy + b * m::DONE_R));
                stroke_polyline(out, &pts, m::DONE_STROKE, c(theme, T::KanbanDueDoneFg));
            } else {
                fill_circle(out, dx, dy, m::AGENDA_DOT_R, tone(theme, it.tone).strip);
            }
            let tx = dx + m::AGENDA_DOT_R + m::ITEM_PAD_X * 2.0;
            let fg = if it.done {
                meta
            } else {
                c(theme, T::KanbanCardFg)
            };
            paint_text_elided(ts, out, &it.title, tx, ty, m::AGENDA_PX, r.x + r.w - tx, fg);
            if r.y + r.h > area.y && r.y < area.y + area.h {
                p.agenda.push((r, *i));
            }
            y += m::AGENDA_ROW_H;
        }
    }
    out.pop_layer();
    stroke_rounded_rect(out, area, m::GRID_RADIUS, m::GRID_LINE, lc);
    p.items = items;
}
