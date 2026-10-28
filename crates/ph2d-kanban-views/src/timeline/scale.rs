//! A escala da Linha do tempo: as duas filas do cabeçalho (o período em cima, os dias/semanas/meses
//! em baixo), as colunas da grelha (fins de semana, divisórias) e a linha de hoje.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, paint_text_title_elided};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_kanban_dates::{CivilDate, Weekday, civil};
use ph2d_kanban_model::{Hue, LabelColor, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Rect as VRect, VectorScene};

use super::Zoom;
use super::layout::Ruler;
use super::metrics as m;
use crate::common::{Ctx, c, label_colors, text_top};
use crate::datepick::{month_name, weekday_name};

pub(super) fn to_vrect(r: Rect) -> VRect {
    VRect::new(
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.x + r.w),
        f64::from(r.y + r.h),
    )
}

/// O azul de hoje (a etiqueta azul forte) e a tinta dela.
pub(super) fn today_colors(theme: Theme) -> (ph2d_vector::Color, ph2d_vector::Color) {
    label_colors(
        theme,
        LabelColor {
            hue: Hue::Blue,
            shade: Shade::Bold,
        },
    )
}

/// Os dias visíveis na grelha (com um de folga de cada lado).
fn visible(r: &Ruler) -> impl Iterator<Item = CivilDate> + '_ {
    let i0 = r.index_at(r.grid.x) - 1;
    let i1 = r.index_at(r.grid.x + r.grid.w) + 1;
    (i0..=i1).map(move |i| r.anchor.add_days(i))
}

fn quarter(d: CivilDate) -> u8 {
    (d.month - 1) / 3 + 1
}

/// O período da fila de cima a que `d` pertence (muda quando muda o rótulo).
fn period(z: Zoom, d: CivilDate) -> (i16, u8) {
    match z {
        Zoom::Quarter => (d.year, quarter(d)),
        _ => (d.year, d.month),
    }
}

fn period_text(z: Zoom, d: CivilDate) -> String {
    match z {
        Zoom::Quarter => tr_with(
            "kanban.views.timeline.quarter",
            &[("q", &quarter(d)), ("y", &d.year)],
        ),
        _ => tr_with(
            "kanban.views.timeline.month_year",
            &[("month", &month_name(d.month, true)), ("y", &d.year)],
        ),
    }
}

/// A fila de baixo marca `d`? (Dia/Semana: todo dia; Mês: o começo da semana; Trimestre: o
/// começo do mês.)
fn tick(z: Zoom, d: CivilDate, week: Weekday) -> bool {
    match z {
        Zoom::Day | Zoom::Week => true,
        Zoom::Month => d.weekday() == week,
        Zoom::Quarter => d.day == 1,
    }
}

fn tick_text(z: Zoom, d: CivilDate) -> String {
    match z {
        Zoom::Day => tr_with(
            "kanban.views.timeline.day",
            &[("wd", &weekday_name(d.weekday())), ("d", &d.day)],
        ),
        Zoom::Week | Zoom::Month => d.day.to_string(),
        Zoom::Quarter => month_name(d.month, false).to_owned(),
    }
}

fn weekend(d: CivilDate) -> bool {
    matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday)
}

/// As colunas da grelha: o fim de semana sombreado (até ao zoom Mês) e as divisórias.
pub(super) fn columns(
    out: &mut VectorScene,
    theme: Theme,
    r: &Ruler,
    z: Zoom,
    body: Rect,
    week: Weekday,
) {
    let line = c(theme, T::Border);
    for d in visible(r) {
        let x = r.x_of(d);
        if z != Zoom::Quarter && weekend(d) {
            out.fill_rect(
                to_vrect(Rect::new(x, body.y, r.ppd, body.h)),
                c(theme, T::KanbanSlot),
            );
        }
        if tick(z, d, week) && x >= r.grid.x {
            out.fill_rect(to_vrect(Rect::new(x, body.y, m::LINE, body.h)), line);
        }
    }
}

/// A linha de hoje, na hora de agora (pinta-se por cima das faixas, por baixo das barras).
pub(super) fn today_line(out: &mut VectorScene, theme: Theme, r: &Ruler, cx: &Ctx<'_>, body: Rect) {
    let now = civil(cx.now.0, cx.zone);
    let minutes = u16::from(now.hour) * 60 + u16::from(now.minute);
    let x = r.x_of(now.date) + r.ppd * f32::from(minutes) / f32::from(24_u16 * 60);
    if x < r.grid.x || x > r.grid.x + r.grid.w {
        return;
    }
    let (blue, _) = today_colors(theme);
    out.fill_rect(
        to_vrect(Rect::new(x - m::TODAY_W / 2.0, body.y, m::TODAY_W, body.h)),
        blue,
    );
    let dot = Rect::new(
        x - m::TODAY_DOT,
        body.y - m::TODAY_DOT,
        2.0 * m::TODAY_DOT,
        2.0 * m::TODAY_DOT,
    );
    fill_rounded_rect(out, dot, m::TODAY_DOT, blue);
}

/// As duas filas do cabeçalho sobre `head` (a largura da grelha).
pub(super) fn header(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: &Ruler,
    z: Zoom,
    head: Rect,
    cx: &Ctx<'_>,
) {
    out.fill_rect(to_vrect(head), c(theme, T::KanbanCard));
    out.push_clip(&to_vrect(head));
    let fg = c(theme, T::KanbanCardFg);
    let meta = c(theme, T::KanbanCardMeta);
    let line = c(theme, T::Border);
    let top = Rect::new(head.x, head.y, head.w, m::SCALE_TOP_H);
    let bottom = Rect::new(head.x, head.y + m::SCALE_TOP_H, head.w, m::SCALE_BOTTOM_H);
    let today = ph2d_kanban_dates::day_of(cx.now.0, cx.zone);
    let days: Vec<CivilDate> = visible(r).collect();
    // A fila de cima: o rótulo do período cola-se à borda esquerda enquanto o período está à vista.
    let mut i = 0;
    while i < days.len() {
        let p = period(z, days[i]);
        let j = days[i..]
            .iter()
            .position(|d| period(z, *d) != p)
            .map_or(days.len(), |n| i + n);
        let x0 = r.x_of(days[i]);
        let x1 = r.x_of(days[j - 1].add_days(1));
        if x0 > head.x {
            out.fill_rect(to_vrect(Rect::new(x0, top.y, m::LINE, head.h)), line);
        }
        let tx = x0.max(head.x) + m::SCALE_PAD_X;
        let ty = text_top(ts, top.y, top.h, m::SMALL_PX);
        let w = x1 - tx - m::SCALE_PAD_X;
        if w > 0.0 {
            paint_text_title_elided(
                ts,
                out,
                &period_text(z, days[i]),
                tx,
                ty,
                m::SMALL_PX,
                w,
                fg,
            );
        }
        i = j;
    }
    // A fila de baixo: os dias (ou semanas, ou meses), e hoje numa pastilha azul.
    let ty = text_top(ts, bottom.y, bottom.h, m::SMALL_PX);
    let ticks: Vec<CivilDate> = days
        .iter()
        .copied()
        .filter(|d| tick(z, *d, cx.week_start))
        .collect();
    for (n, d) in ticks.iter().enumerate() {
        let x0 = r.x_of(*d);
        let x1 = ticks.get(n + 1).map_or(x0 + r.ppd, |e| r.x_of(*e));
        let text = tick_text(z, *d);
        let tw = ts.prefix_width(&text, m::SMALL_PX);
        let centred = matches!(z, Zoom::Day | Zoom::Week);
        let tx = if centred {
            x0 + (r.ppd - tw) / 2.0
        } else {
            x0 + m::SCALE_PAD_X
        };
        if *d == today && centred {
            let (blue, ink) = today_colors(theme);
            let pill = Rect::new(
                tx - m::SCALE_PAD_X,
                bottom.y + m::BUTTON_GAP,
                tw + 2.0 * m::SCALE_PAD_X,
                bottom.h - 2.0 * m::BUTTON_GAP,
            );
            fill_rounded_rect(out, pill, pill.h / 2.0, blue);
            paint_text_elided(
                ts,
                out,
                &text,
                tx,
                ty,
                m::SMALL_PX,
                tw + m::SCALE_PAD_X,
                ink,
            );
        } else if x1 - tx > 0.0 {
            paint_text_elided(ts, out, &text, tx, ty, m::SMALL_PX, x1 - tx, meta);
        }
    }
    out.fill_rect(
        to_vrect(Rect::new(
            head.x,
            head.y + head.h - m::LINE,
            head.w,
            m::LINE,
        )),
        line,
    );
    out.pop_layer();
}
