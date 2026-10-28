//! ⭐ O calendário pintado: o cabeçalho, a grelha, os itens (barras e pastilhas), o «+», o «+N more»,
//! o fantasma do arrasto e os popovers — pelas cores `kanban-*` e as medidas de [`super::metrics`].

use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, fill_rounded_rect_radii, paint_text_elided,
    paint_text_title_elided, rect_to_vello, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{CivilDate, civil, day_of, start_of_day};
use ph2d_kanban_model::{Hue, LabelColor, Moment, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Color, RoundedRect, VectorScene};

use super::layout::{self, Day, Geo, Mode, Seg};
use super::metrics as m;
use super::{CalendarState, Painted, pop};
use crate::common::{Ctx, c, label_colors, text_top};
use crate::datepick::{day_text, month_name, time_text, weekday_name};
use crate::item::{Item, ItemTone, Source};

/// As cores de um item: a faixa, a pastilha suave e a sua tinta, a barra e a sua tinta.
pub(crate) struct Tone {
    pub strip: Color,
    pub soft: Color,
    pub soft_ink: Color,
    pub bar: Color,
    pub bar_ink: Color,
}

pub(crate) fn tone(theme: Theme, t: ItemTone) -> Tone {
    let hue_of = |hue: Hue, shade: Shade| {
        let (bar, bar_ink) = label_colors(theme, LabelColor { hue, shade });
        let (soft, soft_ink) = label_colors(
            theme,
            LabelColor {
                hue,
                shade: Shade::Subtle,
            },
        );
        Tone {
            strip: bar,
            soft,
            soft_ink,
            bar,
            bar_ink,
        }
    };
    match t {
        ItemTone::Label(l) => hue_of(l.hue, l.shade),
        ItemTone::List(h) => hue_of(h.unwrap_or(Hue::Blue), Shade::Normal),
        ItemTone::Token(tok) => Tone {
            strip: c(theme, tok),
            soft: c(theme, T::KanbanSlot),
            soft_ink: c(theme, T::KanbanCardFg),
            bar: c(theme, tok),
            bar_ink: c(theme, T::KanbanLabelInk),
        },
    }
}

/// Um botão do cabeçalho: o corpo, a moldura e o texto (ou nada, para as setas).
fn button(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: Rect,
    text: &str,
    on: bool,
) {
    let (fill, ink) = if on {
        (
            c(theme, T::KanbanBlueBold),
            c(theme, T::KanbanLabelInkBold),
        )
    } else {
        (c(theme, T::KanbanCard), c(theme, T::KanbanCardFg))
    };
    fill_rounded_rect(out, r, m::BUTTON_RADIUS, fill);
    if !on {
        stroke_rounded_rect(out, r, m::BUTTON_RADIUS, m::GRID_LINE, c(theme, T::Border));
    }
    if !text.is_empty() {
        let w = ts.prefix_width(text, m::BUTTON_PX);
        let y = text_top(ts, r.y, r.h, m::BUTTON_PX);
        paint_text_elided(ts, out, text, r.x + (r.w - w) / 2.0, y, m::BUTTON_PX, r.w, ink);
    }
}

fn chevron(out: &mut VectorScene, r: Rect, dir: f32, color: Color) {
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let k = m::CHEVRON;
    let pts = [
        (cx + dir * k / 2.0, cy - k),
        (cx - dir * k / 2.0, cy),
        (cx + dir * k / 2.0, cy + k),
    ];
    stroke_polyline(out, &pts, m::CHEVRON_STROKE, color);
}

fn title_text(mode: Mode, days: &[Day], anchor: CivilDate, today: CivilDate) -> String {
    match (mode, days.first(), days.last()) {
        (Mode::Week, Some(a), Some(b)) => tr_with(
            "kanban.views.calendar.week_title",
            &[("a", &day_text(a.date, today)), ("b", &day_text(b.date, today))],
        ),
        _ => tr_with(
            "kanban.views.calendar.title",
            &[("month", &month_name(anchor.month, true)), ("y", &anchor.year)],
        ),
    }
}

/// O cabeçalho: «Today ‹ › Outubro 2026» à esquerda, «Month | Week» à direita.
#[allow(clippy::too_many_arguments)]
fn header(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    h: Rect,
    st_mode: Mode,
    title: &str,
    p: &mut Painted,
) {
    let y = h.y + (h.h - m::BUTTON_H) / 2.0;
    let today = tr("kanban.views.calendar.today");
    let tw = ts.prefix_width(today, m::BUTTON_PX) + 2.0 * m::BUTTON_PAD_X;
    p.today = Rect::new(h.x, y, tw, m::BUTTON_H);
    button(out, ts, theme, p.today, today, false);
    p.prev = Rect::new(p.today.x + tw + m::BUTTON_GAP, y, m::ARROW_W, m::BUTTON_H);
    p.next = Rect::new(p.prev.x + m::ARROW_W + m::BUTTON_GAP, y, m::ARROW_W, m::BUTTON_H);
    let ink = c(theme, T::KanbanCardFg);
    for (r, dir) in [(p.prev, 1.0), (p.next, -1.0)] {
        button(out, ts, theme, r, "", false);
        chevron(out, r, dir, ink);
    }
    let tx = p.next.x + m::ARROW_W + m::PAD;
    let ty = text_top(ts, h.y, h.h, m::TITLE_PX);
    let fg = c(theme, T::KanbanListFg);
    paint_text_title_elided(ts, out, title, tx, ty, m::TITLE_PX, h.w / 2.0, fg);
    let labels = [
        (tr("kanban.views.calendar.month"), Mode::Month),
        (tr("kanban.views.calendar.week"), Mode::Week),
    ];
    let mut x = h.x + h.w;
    for (text, mode) in labels.iter().rev() {
        let w = ts.prefix_width(text, m::BUTTON_PX) + 2.0 * m::BUTTON_PAD_X;
        x -= w;
        let r = Rect::new(x, y, w, m::BUTTON_H);
        button(out, ts, theme, r, text, *mode == st_mode);
        match mode {
            Mode::Month => p.month = r,
            Mode::Week => p.week = r,
        }
        x -= m::BUTTON_GAP;
    }
}

/// O retângulo de um pedaço na grelha.
fn seg_rect(geo: &Geo, s: &Seg, lane_h: f32) -> Rect {
    let a = geo.cell(s.row * 7 + s.c0);
    let b = geo.cell(s.row * 7 + s.c1);
    let x0 = a.x + m::CELL_PAD;
    let x1 = b.x + b.w - m::CELL_PAD;
    #[allow(clippy::cast_precision_loss)]
    let y = a.y + m::DAY_NUM_H + s.lane as f32 * (lane_h + m::LANE_GAP);
    Rect::new(x0, y, (x1 - x0).max(0.0), lane_h)
}

/// Um item: barra (vários dias) ou pastilha (um dia), com a marca de concluído.
#[allow(clippy::too_many_arguments)]
pub(crate) fn item(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: Rect,
    it: &Item,
    bar: (bool, bool, bool),
    text: &str,
) {
    let (is_bar, starts, ends) = bar;
    let t = tone(theme, it.tone);
    let rad = |on: bool| if on { m::ITEM_RADIUS } else { 0.0 };
    let mut x = r.x + m::ITEM_PAD_X;
    let ink = if is_bar {
        fill_rounded_rect_radii(
            out,
            r,
            (rad(starts), rad(ends), rad(ends), rad(starts)),
            t.bar,
        );
        t.bar_ink
    } else {
        fill_rounded_rect(out, r, m::ITEM_RADIUS, t.soft);
        let strip = Rect::new(r.x, r.y, m::STRIP_W, r.h);
        fill_rounded_rect_radii(out, strip, (m::ITEM_RADIUS, 0.0, 0.0, m::ITEM_RADIUS), t.strip);
        t.soft_ink
    };
    if it.done && (!is_bar || starts) {
        let (cx, cy) = (x + m::DONE_R, r.y + r.h / 2.0);
        fill_circle(out, cx, cy, m::DONE_R, c(theme, T::KanbanDueDone));
        let pts = m::DONE_TICK.map(|(dx, dy)| (cx + dx * m::DONE_R, cy + dy * m::DONE_R));
        stroke_polyline(out, &pts, m::DONE_STROKE, c(theme, T::KanbanDueDoneFg));
        x += 2.0 * m::DONE_R + m::ITEM_PAD_X / 2.0;
    }
    if !text.is_empty() {
        let y = text_top(ts, r.y, r.h, m::ITEM_PX);
        let w = (r.x + r.w - m::ITEM_PAD_X - x).max(0.0);
        paint_text_elided(ts, out, text, x, y, m::ITEM_PX, w, ink);
    }
}

/// O texto de um item: o título, e na semana a hora da entrega à frente (pastilhas).
pub(crate) fn item_text(mode: Mode, it: &Item, is_bar: bool, cx: &Ctx<'_>) -> String {
    if mode == Mode::Week && !is_bar {
        let t = civil(it.end.0, cx.zone);
        tr_with(
            "kanban.views.calendar.at",
            &[("time", &time_text(t.hour, t.minute)), ("title", &it.title)],
        )
    } else {
        it.title.clone()
    }
}

/// A grelha: o fundo, os dias fora do mês, as linhas, os números (hoje num círculo).
fn grid(out: &mut VectorScene, ts: &mut TextSystem, theme: Theme, geo: &Geo, days: &[Day], p: &mut Painted) {
    let g = geo.grid;
    fill_rounded_rect(out, g, m::GRID_RADIUS, c(theme, T::KanbanCard));
    out.push_clip(&RoundedRect::new(
        f64::from(g.x),
        f64::from(g.y),
        f64::from(g.x + g.w),
        f64::from(g.y + g.h),
        f64::from(m::GRID_RADIUS),
    ));
    let line = c(theme, T::Border);
    for (i, d) in days.iter().enumerate() {
        let r = geo.cell(i);
        p.cells.push((r, d.date));
        if !d.in_month {
            out.fill_rect(rect_to_vello(r), c(theme, T::KanbanList));
        }
        out.fill_rect(
            rect_to_vello(Rect::new(r.x, r.y, r.w, m::GRID_LINE)),
            line,
        );
        out.fill_rect(
            rect_to_vello(Rect::new(r.x, r.y, m::GRID_LINE, r.h)),
            line,
        );
        let num = d.date.day.to_string();
        let w = ts.prefix_width(&num, m::DAY_NUM_PX);
        let (nx, ny) = (r.x + r.w - m::CELL_PAD - m::TODAY_R, r.y + m::DAY_NUM_H / 2.0);
        let ink = if d.today {
            fill_circle(out, nx, ny, m::TODAY_R, c(theme, T::KanbanBlueBold));
            c(theme, T::KanbanLabelInkBold)
        } else if d.in_month {
            c(theme, T::KanbanCardFg)
        } else {
            c(theme, T::KanbanCardMeta)
        };
        let ty = text_top(ts, r.y, m::DAY_NUM_H, m::DAY_NUM_PX);
        paint_text_elided(ts, out, &num, nx - w / 2.0, ty, m::DAY_NUM_PX, r.w, ink);
    }
    out.pop_layer();
    stroke_rounded_rect(out, g, m::GRID_RADIUS, m::GRID_LINE, line);
}

/// O «+» de um dia, no canto de cima à esquerda da célula sob o rato.
fn plus(out: &mut VectorScene, theme: Theme, cell: Rect) -> Rect {
    let r = Rect::new(
        cell.x + m::CELL_PAD,
        cell.y + (m::DAY_NUM_H - m::PLUS_W) / 2.0,
        m::PLUS_W,
        m::PLUS_W,
    );
    fill_rounded_rect(out, r, m::ITEM_RADIUS, c(theme, T::KanbanSlot));
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let ink = c(theme, T::KanbanCardFg);
    stroke_polyline(out, &[(cx - m::PLUS_ARM, cy), (cx + m::PLUS_ARM, cy)], m::CHEVRON_STROKE, ink);
    stroke_polyline(out, &[(cx, cy - m::PLUS_ARM), (cx, cy + m::PLUS_ARM)], m::CHEVRON_STROKE, ink);
    r
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut CalendarState,
    src: &dyn Source,
    cx: &Ctx<'_>,
) {
    let today = day_of(cx.now.0, cx.zone);
    let anchor = st.anchor_or_today(cx);
    let days = layout::days(st.mode, anchor, today, cx.week_start);
    let (Some(first), Some(last)) = (days.first(), days.last()) else {
        return;
    };
    let geo = Geo::of(area, days.len());
    let from = start_of_day(first.date, cx.zone).unwrap_or(cx.now.0);
    let to = start_of_day(last.date.add_days(1), cx.zone).unwrap_or(cx.now.0);
    let items = src.items(Moment(from), Moment(to));
    let lane_h = match st.mode {
        Mode::Month => m::LANE_H,
        Mode::Week => m::LANE_H_WEEK,
    };
    let placed = layout::place(&days, &items, cx.zone, geo.cell_h(), lane_h);
    let mut p = Painted {
        anchor: Some(anchor),
        ..Painted::default()
    };
    let title = title_text(st.mode, &days, anchor, today);
    header(out, ts, theme, geo.header, st.mode, &title, &mut p);
    let wd = geo.weekdays;
    #[allow(clippy::cast_precision_loss)]
    for (i, d) in days.iter().take(7).enumerate() {
        let cw = wd.w / 7.0;
        let name = weekday_name(d.date.weekday());
        let w = ts.prefix_width(name, m::WEEKDAY_PX);
        let y = text_top(ts, wd.y, wd.h, m::WEEKDAY_PX);
        let x = wd.x + i as f32 * cw + (cw - w) / 2.0;
        paint_text_elided(ts, out, name, x, y, m::WEEKDAY_PX, cw, c(theme, T::KanbanListMeta));
    }
    grid(out, ts, theme, &geo, &days, &mut p);
    let dragging = st.press.as_ref().filter(|pr| pr.moved);
    let hover_cell = st.hover.and_then(|(x, y)| {
        p.cells
            .iter()
            .position(|(r, _)| r.contains(x, y))
    });
    if let (Some(_), Some(i)) = (dragging, hover_cell) {
        let r = geo.cell(i);
        out.fill_rect(rect_to_vello(r), c(theme, T::KanbanSlot));
    }
    let popped = st.composer.is_some() || st.day_pop.is_some();
    if let (None, false, Some(i)) = (dragging, popped, hover_cell) {
        p.plus = Some((plus(out, theme, geo.cell(i)), days[i].date));
    }
    for s in &placed.segs {
        let r = seg_rect(&geo, s, lane_h);
        let it = &items[s.item];
        let is_bar = s.c0 != s.c1 || !s.starts || !s.ends;
        let text = if is_bar && !s.starts && s.c0 != 0 {
            String::new()
        } else {
            item_text(st.mode, it, is_bar, cx)
        };
        item(out, ts, theme, r, it, (is_bar, s.starts, s.ends), &text);
        p.segs.push((r, s.item));
    }
    for (i, n) in &placed.more {
        let cell = geo.cell(*i);
        let r = Rect::new(
            cell.x + m::CELL_PAD,
            cell.y + cell.h - m::CELL_PAD - m::MORE_H,
            cell.w - 2.0 * m::CELL_PAD,
            m::MORE_H,
        );
        let text = tr_with("kanban.views.calendar.more", &[("n", n)]);
        let y = text_top(ts, r.y, r.h, m::ITEM_PX);
        paint_text_elided(ts, out, &text, r.x + m::ITEM_PAD_X, y, m::ITEM_PX, r.w, c(theme, T::KanbanCardMeta));
        p.more.push((r, days[*i].date, *n));
    }
    p.items = items;
    if let (Some(pr), Some((hx, hy))) = (dragging, st.hover) {
        let r = Rect::new(hx - pr.grab.0, hy - pr.grab.1, pr.size.0, pr.size.1);
        let sh = Rect::new(r.x, r.y + m::POP_SHADOW, r.w, r.h);
        fill_rounded_rect(out, sh, m::ITEM_RADIUS, c(theme, T::KanbanShadowFlight));
        let text = item_text(st.mode, &pr.item, false, cx);
        item(out, ts, theme, r, &pr.item, (false, true, true), &text);
    }
    if let Some(comp) = st.composer.as_ref() {
        p.pop = pop::composer(out, ts, theme, area, &p.cells, comp);
    } else if let Some(day) = st.day_pop {
        let list = layout::on_day(&p.items, day, cx.zone);
        p.pop = pop::day(out, ts, theme, area, &p.cells, day, today, &list);
    }
    st.painted = p;
}
