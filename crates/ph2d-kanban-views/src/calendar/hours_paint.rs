//! ⭐ A grelha de horas pintada (semana e dia): o cabeçalho dos dias, a faixa «dia inteiro», as
//! horas com os blocos lado a lado, a linha de agora e o que o gesto em curso mostra.

use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, fill_rounded_rect_radii, paint_text_elided,
    paint_text_title_elided, rect_to_vello, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_kanban_dates::civil;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use super::hours::{self, Block, HourGeo};
use super::layout::{self, Day};
use super::metrics as m;
use super::paint::{item, tone};
use super::{CalendarState, HourDrag, Painted};
use crate::common::{Ctx, c, text_top};
use crate::datepick::{time_text, weekday_name};
use crate::item::Item;

/// «9 AM».
fn hour_text(h: u8) -> String {
    let h12 = match h % 12 {
        0 => 12,
        x => x,
    };
    let key = if h < 12 {
        "kanban.views.calendar.hour_am"
    } else {
        "kanban.views.calendar.hour_pm"
    };
    tr_with(key, &[("h", &h12)])
}

fn hm(ms: i64, cx: &Ctx<'_>) -> String {
    let t = civil(ms, cx.zone);
    time_text(t.hour, t.minute)
}

/// Um bloco de tempo: a pastilha suave com a faixa, o título e (se cabe) a hora.
#[allow(clippy::too_many_arguments)]
fn block(out: &mut VectorScene, ts: &mut TextSystem, theme: Theme, r: Rect, it: &Item, when: &str, line: f32) {
    let t = tone(theme, it.tone);
    fill_rounded_rect(out, r, m::ITEM_RADIUS, t.soft);
    let strip = Rect::new(r.x, r.y, m::STRIP_W, r.h);
    fill_rounded_rect_radii(out, strip, (m::ITEM_RADIUS, 0.0, 0.0, m::ITEM_RADIUS), t.strip);
    let x = r.x + m::STRIP_W + m::ITEM_PAD_X;
    let w = (r.w - m::STRIP_W - 2.0 * m::ITEM_PAD_X).max(0.0);
    let y = r.y + m::BLOCK_PAD;
    paint_text_title_elided(ts, out, &it.title, x, y, m::ITEM_PX, w, t.soft_ink);
    if r.h >= 2.0 * line + 2.0 * m::BLOCK_PAD {
        paint_text_elided(ts, out, when, x, y + line, m::ITEM_PX, w, t.soft_ink);
    }
}

/// O texto da hora de um bloco: «10:00 AM – 11:30 AM», ou só a entrega.
fn when(it: &Item, b: &Block, cx: &Ctx<'_>) -> String {
    match (b.has_start, it.start) {
        (true, Some(s)) => tr_with(
            "kanban.views.calendar.span",
            &[("a", &hm(s.0, cx)), ("b", &hm(it.end.0, cx))],
        ),
        _ => hm(it.end.0, cx),
    }
}

/// O cabeçalho de cada dia: o nome por cima, o número por baixo (hoje num círculo).
fn day_heads(out: &mut VectorScene, ts: &mut TextSystem, theme: Theme, head: Rect, days: &[Day]) {
    #[allow(clippy::cast_precision_loss)]
    let cw = head.w / days.len().max(1) as f32;
    for (i, d) in days.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = head.x + i as f32 * cw;
        let name = weekday_name(d.date.weekday());
        let w = ts.prefix_width(name, m::WEEKDAY_PX);
        let half = head.h / 2.0;
        let y = text_top(ts, head.y, half, m::WEEKDAY_PX);
        paint_text_elided(ts, out, name, x + (cw - w) / 2.0, y, m::WEEKDAY_PX, cw, c(theme, T::KanbanListMeta));
        let num = d.date.day.to_string();
        let w = ts.prefix_width(&num, m::DAY_HEAD_NUM_PX);
        let (nx, ny) = (x + cw / 2.0, head.y + half + half / 2.0);
        let ink = if d.today {
            fill_circle(out, nx, ny, m::DAY_HEAD_R, c(theme, T::KanbanBlueBold));
            c(theme, T::KanbanLabelInkBold)
        } else {
            c(theme, T::KanbanListFg)
        };
        let ty = text_top(ts, ny - half / 2.0, half, m::DAY_HEAD_NUM_PX);
        paint_text_title_elided(ts, out, &num, nx - w / 2.0, ty, m::DAY_HEAD_NUM_PX, cw, ink);
    }
}

/// A rolagem das horas: a de antes, ou a de abertura (as 8:00, ou uma hora antes de agora se hoje
/// está à vista e já passou delas), sempre dentro do dia.
fn scroll(st: &mut CalendarState, days: &[Day], body_h: f32, cx: &Ctx<'_>) -> f32 {
    let max = (24.0 * m::HOUR_H - body_h).max(0.0);
    let s = st.scroll.unwrap_or_else(|| {
        let now = hours::minutes(cx.now.0, cx.zone) / 60.0;
        let open = if days.iter().any(|d| d.today) && now - 1.0 > m::OPEN_HOUR {
            now - 1.0
        } else {
            m::OPEN_HOUR
        };
        open * m::HOUR_H
    });
    let s = s.clamp(0.0, max);
    st.scroll = Some(s);
    s
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut CalendarState,
    p: &mut Painted,
    days: &[Day],
    items: Vec<Item>,
    cx: &Ctx<'_>,
) {
    let line = ts.layout("Ag", m::ITEM_PX, f32::INFINITY).height();
    let min_minutes = (line + 2.0 * m::BLOCK_PAD) / m::HOUR_H * 60.0;
    let (blocks, all_day) = hours::place(days, &items, cx.zone, min_minutes);
    let band_items: Vec<Item> = all_day.iter().map(|i| items[*i].clone()).collect();
    let pitch = m::LANE_H + m::LANE_GAP;
    let head = Rect::new(area.x + m::GUTTER_W, area.y, area.w - m::GUTTER_W, m::DAY_HEAD_H);
    #[allow(clippy::cast_precision_loss)]
    let need = layout::place(days, &band_items, cx.zone, f32::MAX / 4.0, m::LANE_H)
        .segs
        .iter()
        .map(|s| s.lane + 1)
        .max()
        .unwrap_or(0) as f32;
    let cap = ((area.h - m::DAY_HEAD_H) * m::ALLDAY_FRACTION / pitch).floor().max(1.0);
    let over = need > cap;
    let lanes_h = need.clamp(1.0, cap) * pitch;
    let more_h = if over { m::MORE_H } else { 0.0 };
    let band_cell_h = m::DAY_NUM_H + m::CELL_PAD + lanes_h + more_h;
    let band = Rect::new(head.x, head.y + head.h, head.w, 2.0 * m::CELL_PAD + lanes_h + more_h);
    let body_y = band.y + band.h;
    let body = Rect::new(head.x, body_y, head.w, (area.y + area.h - body_y).max(0.0));
    let s = scroll(st, days, body.h, cx);
    #[allow(clippy::cast_precision_loss)]
    let cw = head.w / days.len().max(1) as f32;
    #[allow(clippy::cast_precision_loss)]
    let cols: Vec<(Rect, _)> = days
        .iter()
        .enumerate()
        .map(|(i, d)| (Rect::new(head.x + i as f32 * cw, body.y, cw, body.h), d.date))
        .collect();
    for (r, d) in &cols {
        p.cells.push((Rect::new(r.x, head.y, r.w, area.y + area.h - head.y), *d));
    }
    let g = HourGeo {
        body,
        top: body.y - s,
        cols,
        zone: Some(cx.zone.clone()),
    };
    day_heads(out, ts, theme, head, days);
    let panel = Rect::new(band.x, band.y, band.w, band.h + body.h);
    fill_rounded_rect(out, panel, m::GRID_RADIUS, c(theme, T::KanbanCard));
    let lc = c(theme, T::Border);
    // A faixa «dia inteiro».
    let placed = layout::place(days, &band_items, cx.zone, band_cell_h, m::LANE_H);
    for sg in &placed.segs {
        let (a, b) = (&g.cols[sg.c0].0, &g.cols[sg.c1].0);
        #[allow(clippy::cast_precision_loss)]
        let y = band.y + m::CELL_PAD + sg.lane as f32 * pitch;
        let r = Rect::new(a.x + m::CELL_PAD, y, b.x + b.w - a.x - 2.0 * m::CELL_PAD, m::LANE_H);
        let it = &band_items[sg.item];
        let is_bar = sg.c0 != sg.c1 || !sg.starts || !sg.ends;
        item(out, ts, theme, r, it, (is_bar, sg.starts, sg.ends), &it.title);
        p.segs.push((r, all_day[sg.item]));
    }
    for (i, n) in &placed.more {
        let col = g.cols[*i].0;
        let r = Rect::new(col.x + m::CELL_PAD, band.y + band.h - m::CELL_PAD - m::MORE_H, col.w - 2.0 * m::CELL_PAD, m::MORE_H);
        let text = tr_with("kanban.views.calendar.more", &[("n", n)]);
        let y = text_top(ts, r.y, r.h, m::ITEM_PX);
        paint_text_elided(ts, out, &text, r.x + m::ITEM_PAD_X, y, m::ITEM_PX, r.w, c(theme, T::KanbanCardMeta));
        p.more.push((r, days[*i].date, *n));
    }
    out.fill_rect(rect_to_vello(Rect::new(band.x, body.y - m::GRID_LINE, band.w, m::GRID_LINE)), lc);
    // As horas, recortadas à janela.
    out.push_clip(&rect_to_vello(Rect::new(area.x, body.y, area.w, body.h)));
    for h in 0u8..24 {
        let y = g.y_of(f32::from(h) * 60.0);
        out.fill_rect(rect_to_vello(Rect::new(body.x, y, body.w, m::GRID_LINE)), lc);
        let th = ts.layout("Ag", m::HOUR_PX, f32::INFINITY).height();
        if h > 0 && y - th / 2.0 >= body.y {
            let t = hour_text(h);
            let w = ts.prefix_width(&t, m::HOUR_PX);
            let x = area.x + m::GUTTER_W - m::CELL_PAD * 2.0 - w;
            paint_text_elided(ts, out, &t, x, y - th / 2.0, m::HOUR_PX, m::GUTTER_W, c(theme, T::KanbanListMeta));
        }
    }
    for (r, _) in g.cols.iter().skip(1) {
        out.fill_rect(rect_to_vello(Rect::new(r.x, body.y, m::GRID_LINE, body.h)), lc);
    }
    let moving = match st.hold.as_ref().filter(|h| h.moved).map(|h| &h.drag) {
        Some(HourDrag::Move { item, .. } | HourDrag::Resize { item, .. }) => Some(item.key),
        _ => None,
    };
    for b in &blocks {
        let r = hours::block_rect(&g, b);
        let it = &items[b.item];
        if Some(it.key) != moving {
            block(out, ts, theme, r, it, &when(it, b, cx), line);
        }
        p.blocks.push((r, *b));
    }
    preview(out, ts, theme, st, &g, &blocks, &items, min_minutes, line, cx);
    if let Some((r, _)) = g.cols.iter().zip(days).find(|(_, d)| d.today).map(|(c, _)| c) {
        let y = g.y_of(hours::minutes(cx.now.0, cx.zone));
        let red = c(theme, T::KanbanDueOver);
        out.fill_rect(rect_to_vello(Rect::new(r.x, y - m::NOW_STROKE / 2.0, r.w, m::NOW_STROKE)), red);
        fill_circle(out, r.x, y, m::NOW_R, red);
    }
    out.pop_layer();
    stroke_rounded_rect(out, panel, m::GRID_RADIUS, m::GRID_LINE, lc);
    p.items = items;
    p.hours = Some(g);
}

/// O gesto em curso: o bloco onde vai cair (mover, esticar) ou o bloco novo a nascer (criar).
#[allow(clippy::too_many_arguments)]
fn preview(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    st: &CalendarState,
    g: &HourGeo,
    blocks: &[Block],
    items: &[Item],
    min_minutes: f32,
    line: f32,
    cx: &Ctx<'_>,
) {
    let Some((col, top, bot)) = super::hours_input::preview(st) else {
        return;
    };
    let b = Block {
        item: 0,
        col,
        top_min: top,
        bot_min: bot.max(top + min_minutes),
        lane: 0,
        lanes: 1,
        has_start: true,
    };
    let r = hours::block_rect(g, &b);
    let dragged = st.hold.as_ref().and_then(|h| match &h.drag {
        HourDrag::Move { item, .. } | HourDrag::Resize { item, .. } => Some(item.clone()),
        HourDrag::Create { .. } => None,
    });
    if let Some(it) = dragged {
        let sh = Rect::new(r.x, r.y + m::POP_SHADOW, r.w, r.h);
        fill_rounded_rect(out, sh, m::ITEM_RADIUS, c(theme, T::KanbanShadowFlight));
        let had = blocks
            .iter()
            .find(|x| items.get(x.item).is_some_and(|i| i.key == it.key))
            .is_none_or(|x| x.has_start);
        let date = g.cols.get(col).map(|c| c.1);
        let span = date.and_then(|d| {
            let zone = g.zone.as_ref()?;
            let a = hours::at(d, hours::snap(top), zone)?;
            let e = hours::at(d, hours::snap(bot), zone)?;
            Some((a, e))
        });
        let text = match span {
            Some((a, e)) if had => tr_with(
                "kanban.views.calendar.span",
                &[("a", &hm(a.0, cx)), ("b", &hm(e.0, cx))],
            ),
            Some((a, _)) => hm(a.0, cx),
            None => String::new(),
        };
        block(out, ts, theme, r, &it, &text, line);
    } else {
        fill_rounded_rect(out, r, m::ITEM_RADIUS, c(theme, T::KanbanSlot));
        stroke_rounded_rect(out, r, m::ITEM_RADIUS, m::FOCUS_STROKE, c(theme, T::FocusRing));
    }
}
