//! A tabela pintada: o fundo do quadro, a folha branca com o cabeçalho fixo, SÓ as linhas visíveis
//! (a rolagem virtual), a barra de rolagem e o popover aberto. Cores pelos tokens `kanban-*`.

use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, paint_text_title_elided, rect_to_vello,
    stroke_rounded_rect,
};
use ph2d_editor_core::paint_shapes::fill_polygon;
use ph2d_editor_core::widget::{
    ScrollbarState, paint_date_picker, paint_dropdown, paint_scrollbar, resolve_opaque,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{DueTone, civil, due_tone};
use ph2d_kanban_model::{
    Card, CardId, FieldKind, FieldValue, Kanban, Label, LabelColor, Member, Shade,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{RoundedRect, VectorScene};

use super::edit::{Pop, pop_row};
use super::layout::{Column, Grid, Sort};
use super::metrics as m;
use crate::common::{Ctx, avatar_hue, c, initials, label_colors, list_token, people, text_top};
use crate::datepick::{day_text, time_text};

/// O que o estado da vista muda na pintura.
pub(super) struct Look {
    pub hover: Option<(usize, Column)>,
    pub sort: Option<Sort>,
    pub thumb: ScrollbarState,
}

/// O nome de uma coluna no cabeçalho.
fn title(k: &Kanban, col: Column) -> String {
    match col {
        Column::Card => tr("kanban.views.table.col.card").to_owned(),
        Column::List => tr("kanban.views.table.col.list").to_owned(),
        Column::Labels => tr("kanban.views.table.col.labels").to_owned(),
        Column::Members => tr("kanban.views.table.col.members").to_owned(),
        Column::Due => tr("kanban.views.table.col.due").to_owned(),
        Column::Field(f) => k.fields.get(&f).map(|d| d.name.clone()).unwrap_or_default(),
    }
}

fn line(
    ts: &mut TextSystem,
    out: &mut VectorScene,
    text: &str,
    cell: Rect,
    px: f32,
    color: ph2d_vector::Color,
) {
    let y = text_top(ts, cell.y, cell.h, px);
    paint_text_elided(
        ts,
        out,
        text,
        cell.x + m::CELL_PAD,
        y,
        px,
        (cell.w - 2.0 * m::CELL_PAD).max(0.0),
        color,
    );
}

/// ⭐ A folha da tabela.
#[allow(clippy::too_many_arguments)]
pub(super) fn sheet(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    cx: &Ctx<'_>,
    g: &Grid,
    rows: &[CardId],
    drawn: std::ops::Range<usize>,
    look: &Look,
) {
    let area = Rect::new(
        g.sheet.x - m::PAD,
        g.sheet.y - m::PAD,
        g.sheet.w + 2.0 * m::PAD,
        g.sheet.h + 2.0 * m::PAD,
    );
    out.fill_rect(rect_to_vello(area), c(theme, T::KanbanBoardBg));
    fill_rounded_rect(out, g.sheet, m::RADIUS, c(theme, T::KanbanCard));
    let clip = RoundedRect::from_rect(rect_to_vello(g.sheet), f64::from(m::RADIUS));
    out.push_clip(&clip);
    for i in drawn {
        let Some(card) = rows.get(i).and_then(|id| k.cards.get(id)) else {
            continue;
        };
        let r = g.row(i);
        if look.hover.is_some_and(|h| h.0 == i) {
            out.fill_rect(rect_to_vello(r), c(theme, T::KanbanList));
        }
        for &(col, x, w) in &g.cols {
            let cell = Rect::new(x, r.y, w, r.h);
            let hot = look.hover == Some((i, col));
            cell_paint(out, ts, theme, k, cx, card, col, cell, hot);
        }
        let rule = Rect::new(r.x, r.y + r.h - m::RULE_W, r.w, m::RULE_W);
        out.fill_rect(rect_to_vello(rule), c(theme, T::KanbanSlot));
    }
    if rows.is_empty() {
        let b = g.body;
        let msg = tr("kanban.views.no_cards");
        let w = ts.prefix_width(msg, m::BODY_PX);
        let y = text_top(ts, b.y, m::ROW_H * 2.0, m::BODY_PX);
        paint_text_elided(
            ts,
            out,
            msg,
            b.x + (b.w - w).max(0.0) / 2.0,
            y,
            m::BODY_PX,
            b.w,
            c(theme, T::KanbanCardMeta),
        );
    }
    header(out, ts, theme, k, g, look.sort);
    out.pop_layer();
    paint_scrollbar(
        g.body,
        g.scroll_y,
        g.content_h(),
        g.body.h,
        (look.thumb, 1.0),
        out,
        theme,
    );
}

fn header(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    g: &Grid,
    sort: Option<Sort>,
) {
    out.fill_rect(rect_to_vello(g.header), c(theme, T::KanbanList));
    let ink = c(theme, T::KanbanListFg);
    for &(col, x, w) in &g.cols {
        let cell = Rect::new(x, g.header.y, w, g.header.h);
        let t = title(k, col);
        let y = text_top(ts, cell.y, cell.h, m::SMALL_PX);
        let budget = (w - 3.0 * m::CELL_PAD).max(0.0);
        paint_text_title_elided(ts, out, &t, x + m::CELL_PAD, y, m::SMALL_PX, budget, ink);
        if let Some(s) = sort.filter(|s| s.by == col) {
            let tw = ts.prefix_width(&t, m::SMALL_PX).min(budget);
            let ax = x + m::CELL_PAD + tw + m::CELL_PAD / 2.0 + m::SORT_ARROW;
            let ay = cell.y + cell.h / 2.0;
            let a = m::SORT_ARROW;
            let pts = if s.desc {
                [
                    (ax - a, ay - a / 2.0),
                    (ax + a, ay - a / 2.0),
                    (ax, ay + a / 2.0),
                ]
            } else {
                [
                    (ax - a, ay + a / 2.0),
                    (ax + a, ay + a / 2.0),
                    (ax, ay - a / 2.0),
                ]
            };
            fill_polygon(out, &pts, ink);
        }
        let rule = Rect::new(
            x + w - m::RULE_W,
            cell.y + m::RULE_INSET,
            m::RULE_W,
            cell.h - 2.0 * m::RULE_INSET,
        );
        out.fill_rect(rect_to_vello(rule), c(theme, T::KanbanSlot));
    }
}

/// Uma célula.
#[allow(clippy::too_many_arguments)]
fn cell_paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    cx: &Ctx<'_>,
    card: &Card,
    col: Column,
    cell: Rect,
    hot: bool,
) {
    let fg = c(theme, T::KanbanCardFg);
    match col {
        Column::Card => {
            let y = text_top(ts, cell.y, cell.h, m::BODY_PX);
            let wb = (cell.w - 2.0 * m::CELL_PAD).max(0.0);
            paint_text_title_elided(
                ts,
                out,
                &card.title,
                cell.x + m::CELL_PAD,
                y,
                m::BODY_PX,
                wb,
                fg,
            );
            if hot {
                let tw = ts.prefix_width(&card.title, m::BODY_PX).min(wb);
                let u = Rect::new(
                    cell.x + m::CELL_PAD,
                    y + m::BODY_PX + m::RULE_W * 2.0,
                    tw,
                    m::RULE_W,
                );
                out.fill_rect(rect_to_vello(u), fg);
            }
        }
        Column::List => {
            let Some(l) = k.lists.get(&card.list) else {
                return;
            };
            let dot = Rect::new(
                cell.x + m::CELL_PAD,
                cell.y + (cell.h - m::DOT) / 2.0,
                m::DOT,
                m::DOT,
            );
            fill_rounded_rect(out, dot, m::CHECK_RADIUS, c(theme, list_token(l.color)));
            let t = Rect::new(
                dot.x + dot.w - m::DOT_GAP,
                cell.y,
                cell.w - dot.w - m::CHECK,
                cell.h,
            );
            line(ts, out, &l.title, t, m::BODY_PX, fg);
            if hot {
                chevron(out, theme, cell);
            }
        }
        Column::Labels => labels(out, ts, theme, k, card, cell),
        Column::Members => avatars(out, ts, theme, &card.members, cell),
        Column::Due => due(out, ts, theme, cx, card, cell, hot),
        Column::Field(f) => {
            let Some(def) = k.fields.get(&f) else {
                return;
            };
            match (&def.kind, card.fields.get(&f)) {
                (FieldKind::Checkbox, v) => check(
                    out,
                    theme,
                    cell,
                    matches!(v, Some(FieldValue::Checkbox(true))),
                ),
                (FieldKind::Dropdown(opts), Some(FieldValue::Option(o))) => {
                    if let Some(o) = opts.iter().find(|x| x.id == *o) {
                        match o.color {
                            Some(hue) => {
                                let color = LabelColor {
                                    hue,
                                    shade: Shade::Subtle,
                                };
                                pill(out, ts, theme, color, &o.text, cell.x + m::CELL_PAD, cell);
                            }
                            None => line(ts, out, &o.text, cell, m::BODY_PX, fg),
                        }
                    }
                    if hot {
                        chevron(out, theme, cell);
                    }
                }
                (_, Some(FieldValue::Text(t))) => line(ts, out, t, cell, m::BODY_PX, fg),
                (_, Some(FieldValue::Number(n))) => {
                    line(ts, out, &format!("{n}"), cell, m::BODY_PX, fg)
                }
                (_, Some(FieldValue::Date(d))) => {
                    let today = civil(cx.now.0, cx.zone).date;
                    line(
                        ts,
                        out,
                        &day_text(civil(d.0, cx.zone).date, today),
                        cell,
                        m::BODY_PX,
                        fg,
                    );
                }
                (FieldKind::Dropdown(_), None) if hot => chevron(out, theme, cell),
                _ => {}
            }
        }
    }
}

fn chevron(out: &mut VectorScene, theme: Theme, cell: Rect) {
    let s = m::CHECK;
    let r = Rect::new(
        cell.x + cell.w - m::CELL_PAD - s,
        cell.y + (cell.h - s) / 2.0,
        s,
        s,
    );
    paint_icon(
        out,
        IconId::ChevronDown,
        r,
        c(theme, T::KanbanCardMeta),
        StrokeToken::Default.px(),
    );
}

fn check(out: &mut VectorScene, theme: Theme, cell: Rect, on: bool) {
    let r = Rect::new(
        cell.x + m::CELL_PAD,
        cell.y + (cell.h - m::CHECK) / 2.0,
        m::CHECK,
        m::CHECK,
    );
    if on {
        fill_rounded_rect(out, r, m::CHECK_RADIUS, c(theme, T::Accent));
        paint_icon(
            out,
            IconId::Check,
            r,
            c(theme, T::AccentFg),
            StrokeToken::Default.px(),
        );
    } else {
        stroke_rounded_rect(
            out,
            r,
            m::CHECK_RADIUS,
            StrokeToken::Default.px(),
            c(theme, T::KanbanCardMeta),
        );
    }
}

/// Uma etiqueta em pílula a partir de `x`; devolve a largura gasta.
#[allow(clippy::too_many_arguments)]
fn pill(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    color: LabelColor,
    name: &str,
    x: f32,
    cell: Rect,
) -> f32 {
    let (bg, ink) = label_colors(theme, color);
    let tw = if name.is_empty() {
        m::PILL_H
    } else {
        ts.prefix_width(name, m::SMALL_PX) + 2.0 * m::PILL_PAD_X
    };
    let room = (cell.x + cell.w - m::CELL_PAD - x).max(0.0);
    let w = tw.min(room);
    let r = Rect::new(x, cell.y + (cell.h - m::PILL_H) / 2.0, w, m::PILL_H);
    fill_rounded_rect(out, r, m::PILL_RADIUS, bg);
    if !name.is_empty() {
        let y = text_top(ts, r.y, r.h, m::SMALL_PX);
        paint_text_elided(
            ts,
            out,
            name,
            r.x + m::PILL_PAD_X,
            y,
            m::SMALL_PX,
            (w - 2.0 * m::PILL_PAD_X).max(0.0),
            ink,
        );
    }
    w
}

fn labels(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    card: &Card,
    cell: Rect,
) {
    let ls: Vec<&Label> = card
        .labels
        .iter()
        .filter_map(|l| k.labels.get(l))
        .filter(|l| !l.meta.deleted)
        .collect();
    let mut x = cell.x + m::CELL_PAD;
    let end = cell.x + cell.w - m::CELL_PAD;
    let more_w = ts.prefix_width("+99", m::SMALL_PX);
    for (i, l) in ls.iter().enumerate() {
        let want = if l.name.is_empty() {
            m::PILL_H
        } else {
            ts.prefix_width(&l.name, m::SMALL_PX) + 2.0 * m::PILL_PAD_X
        };
        let last = i + 1 == ls.len();
        if !last && x + want + m::PILL_GAP + more_w > end && i > 0 {
            let rest = tr_with("kanban.views.table.more", &[("n", &(ls.len() - i))]);
            let y = text_top(ts, cell.y, cell.h, m::SMALL_PX);
            paint_text_elided(
                ts,
                out,
                &rest,
                x,
                y,
                m::SMALL_PX,
                (end - x).max(0.0),
                c(theme, T::KanbanCardMeta),
            );
            return;
        }
        let color = l.color.unwrap_or(LabelColor {
            hue: ph2d_kanban_model::Hue::Black,
            shade: Shade::Subtle,
        });
        x += pill(out, ts, theme, color, &l.name, x, cell) + m::PILL_GAP;
        if x >= end {
            return;
        }
    }
}

/// Os avatares de `who`, a partir da esquerda, sobrepostos como na frente do cartão.
pub(super) fn avatars(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    who: &[Member],
    cell: Rect,
) {
    let mut x = cell.x + m::CELL_PAD;
    let end = cell.x + cell.w - m::CELL_PAD;
    let ring_c = c(theme, T::KanbanCard);
    for p in who {
        if x + m::AVATAR > end {
            break;
        }
        let r = Rect::new(x, cell.y + (cell.h - m::AVATAR) / 2.0, m::AVATAR, m::AVATAR);
        avatar(out, ts, theme, p, r, ring_c);
        x += m::AVATAR - m::AVATAR_OVERLAP;
    }
}

fn avatar(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    p: &Member,
    r: Rect,
    ring: ph2d_vector::Color,
) {
    let g = Rect::new(
        r.x - m::AVATAR_RING,
        r.y - m::AVATAR_RING,
        r.w + 2.0 * m::AVATAR_RING,
        r.h + 2.0 * m::AVATAR_RING,
    );
    fill_rounded_rect(out, g, g.w / 2.0, ring);
    let (bg, _) = label_colors(
        theme,
        LabelColor {
            hue: avatar_hue(p),
            shade: Shade::Normal,
        },
    );
    fill_rounded_rect(out, r, r.w / 2.0, bg);
    let s = initials(p);
    let w = ts.prefix_width(&s, m::AVATAR_PX);
    let y = text_top(ts, r.y, r.h, m::AVATAR_PX);
    paint_text_title_elided(
        ts,
        out,
        &s,
        r.x + (r.w - w) / 2.0,
        y,
        m::AVATAR_PX,
        r.w,
        c(theme, T::KanbanAvatarInk),
    );
}

/// A entrega em distintivo (as cores do Trello: atrasada, a menos de 24 h, concluída).
fn due(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    cx: &Ctx<'_>,
    card: &Card,
    cell: Rect,
    hot: bool,
) {
    let Some(d) = card.due else {
        if hot {
            chevron(out, theme, cell);
        }
        return;
    };
    let today = civil(cx.now.0, cx.zone).date;
    let at = civil(d.at.0, cx.zone);
    let text = tr_with(
        "kanban.back.day_time",
        &[
            ("day", &day_text(at.date, today)),
            ("time", &time_text(at.hour, at.minute)),
        ],
    );
    let tone = due_tone(Some(d.at.0), d.done, cx.now.0);
    let pair = match tone {
        Some(DueTone::Overdue) => Some((T::KanbanDueOver, T::KanbanDueOverFg)),
        Some(DueTone::Soon) => Some((T::KanbanDueSoon, T::KanbanDueSoonFg)),
        Some(DueTone::Done) => Some((T::KanbanDueDone, T::KanbanDueDoneFg)),
        _ => None,
    };
    match pair {
        Some((bg, fg)) => {
            let w = (ts.prefix_width(&text, m::SMALL_PX) + 2.0 * m::PILL_PAD_X)
                .min(cell.w - 2.0 * m::CELL_PAD);
            let r = Rect::new(
                cell.x + m::CELL_PAD,
                cell.y + (cell.h - m::PILL_H) / 2.0,
                w,
                m::PILL_H,
            );
            fill_rounded_rect(out, r, m::PILL_RADIUS, c(theme, bg));
            let y = text_top(ts, r.y, r.h, m::SMALL_PX);
            paint_text_elided(
                ts,
                out,
                &text,
                r.x + m::PILL_PAD_X,
                y,
                m::SMALL_PX,
                (w - 2.0 * m::PILL_PAD_X).max(0.0),
                c(theme, fg),
            );
        }
        None => line(ts, out, &text, cell, m::BODY_PX, c(theme, T::KanbanCardFg)),
    }
}

/// As etiquetas vivas de `k`, pela ordem do popover (o nome, e a id desempata).
pub(super) fn pop_labels(k: &Kanban) -> Vec<&Label> {
    let mut v: Vec<&Label> = k.labels.values().filter(|l| !l.meta.deleted).collect();
    v.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    v
}

fn frame(out: &mut VectorScene, ts: &mut TextSystem, theme: Theme, at: Rect, title: &str) {
    let shadow = Rect::new(at.x, at.y + m::POP_GAP / 2.0, at.w, at.h);
    fill_rounded_rect(out, shadow, m::POP_RADIUS, c(theme, T::KanbanShadowHover));
    fill_rounded_rect(out, at, m::POP_RADIUS, resolve_opaque(T::KanbanCard, theme));
    let r = pop_row(at, 0);
    let w = ts.prefix_width(title, m::SMALL_PX);
    let y = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_title_elided(
        ts,
        out,
        title,
        r.x + (r.w - w).max(0.0) / 2.0,
        y,
        m::SMALL_PX,
        r.w,
        c(theme, T::KanbanCardMeta),
    );
}

fn tick(out: &mut VectorScene, theme: Theme, r: Rect, on: bool) {
    let b = Rect::new(
        r.x + r.w - m::CHECK - m::POP_PAD,
        r.y + (r.h - m::CHECK) / 2.0,
        m::CHECK,
        m::CHECK,
    );
    if on {
        paint_icon(
            out,
            IconId::Check,
            b,
            c(theme, T::KanbanCardFg),
            StrokeToken::Default.px(),
        );
    }
}

/// O popover aberto.
pub(super) fn pop(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    cx: &Ctx<'_>,
    pop: &Pop,
) {
    let Some(card) = k.cards.get(&pop.card()) else {
        return;
    };
    match pop {
        Pop::List { dd, chip, .. } => paint_dropdown(dd, *chip, out, ts, theme),
        Pop::Option { dd, chip, .. } => paint_dropdown(dd, *chip, out, ts, theme),
        Pop::Dates { picker, at, .. } => paint_date_picker(&picker.widget, *at, out, ts, theme),
        Pop::Labels { at, .. } => {
            frame(out, ts, theme, *at, tr("kanban.back.labels"));
            for (i, l) in pop_labels(k).into_iter().enumerate() {
                let r = pop_row(*at, i + 1);
                let bar = Rect::new(
                    r.x,
                    r.y + (r.h - m::PILL_H - m::PILL_GAP) / 2.0,
                    r.w - m::CHECK - 2.0 * m::POP_PAD,
                    m::PILL_H + m::PILL_GAP,
                );
                let color = l.color.unwrap_or(LabelColor {
                    hue: ph2d_kanban_model::Hue::Black,
                    shade: Shade::Subtle,
                });
                let (bg, ink) = label_colors(theme, color);
                fill_rounded_rect(out, bar, m::PILL_RADIUS, bg);
                let y = text_top(ts, bar.y, bar.h, m::BODY_PX);
                paint_text_title_elided(
                    ts,
                    out,
                    &l.name,
                    bar.x + m::PILL_PAD_X,
                    y,
                    m::BODY_PX,
                    (bar.w - 2.0 * m::PILL_PAD_X).max(0.0),
                    ink,
                );
                tick(out, theme, r, card.labels.contains(&l.id));
            }
        }
        Pop::Members { at, .. } => {
            frame(out, ts, theme, *at, tr("kanban.back.members"));
            let ring = resolve_opaque(T::KanbanCard, theme);
            for (i, p) in people(k, cx).iter().enumerate() {
                let r = pop_row(*at, i + 1);
                let a = Rect::new(
                    r.x + m::AVATAR_RING,
                    r.y + (r.h - m::AVATAR) / 2.0,
                    m::AVATAR,
                    m::AVATAR,
                );
                avatar(out, ts, theme, p, a, ring);
                let t = Rect::new(a.x + a.w, r.y, r.w - a.w - m::CHECK - 2.0 * m::POP_PAD, r.h);
                line(ts, out, &p.0, t, m::BODY_PX, c(theme, T::KanbanCardFg));
                tick(out, theme, r, card.members.contains(p));
            }
        }
    }
}
