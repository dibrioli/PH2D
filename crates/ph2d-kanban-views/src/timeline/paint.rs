//! A Linha do tempo pintada: a barra de ferramentas, as faixas (nomes à esquerda, barras na grelha),
//! a escala por cima, a gaveta à direita e o que voa na mão. Guarda em [`Painted`] o que pintou.

use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, paint_text_title_elided,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::{Card, CardId, Hue, Kanban, LabelColor, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

use super::drawer;
use super::gesture::{Part, moved};
use super::layout::{self, Lane, LaneKey, Ruler, Span};
use super::metrics as m;
use super::scale::{self, to_vrect};
use super::{Bar, Button, Grab, GroupBy, Painted, TimelineState, Zoom};
use crate::common::{Ctx, avatar_hue, c, initials, label_colors, text_top};

/// A cor de um cartão na linha do tempo: a 1.ª etiqueta com cor, senão a da lista (a lei do
/// calendário, [`crate::ItemTone`]); e a tinta que a lê.
pub(super) fn card_colors(theme: Theme, k: &Kanban, card: &Card) -> (Color, Color) {
    let label = card
        .labels
        .iter()
        .filter_map(|l| k.labels.get(l))
        .filter(|l| !l.meta.deleted)
        .find_map(|l| l.color);
    let l = label.unwrap_or(LabelColor {
        hue: k
            .lists
            .get(&card.list)
            .and_then(|l| l.color)
            .unwrap_or(Hue::Blue),
        shade: Shade::Normal,
    });
    label_colors(theme, l)
}

fn lane_h(l: &Lane) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let rows = l.rows.len().max(1) as f32;
    rows * m::ROW_H + 2.0 * m::LANE_PAD_Y
}

/// Um grupo de botões juntos (o seletor segmentado); devolve o x do fim.
fn segmented(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    (x, y): (f32, f32),
    items: &[(Button, &str, bool)],
    p: &mut Painted,
) -> f32 {
    let widths: Vec<f32> = items
        .iter()
        .map(|(_, t, _)| ts.prefix_width(t, m::TEXT_PX) + 2.0 * m::BUTTON_PAD_X)
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 2.0 * m::BUTTON_GAP;
    let frame = Rect::new(x, y, total, m::BUTTON_H + 2.0 * m::BUTTON_GAP);
    fill_rounded_rect(
        out,
        frame,
        m::RADIUS + m::BUTTON_GAP,
        c(theme, T::KanbanSlot),
    );
    let mut bx = x + m::BUTTON_GAP;
    let ty = text_top(ts, y + m::BUTTON_GAP, m::BUTTON_H, m::TEXT_PX);
    for ((b, text, on), w) in items.iter().zip(widths) {
        let r = Rect::new(bx, y + m::BUTTON_GAP, w, m::BUTTON_H);
        if *on {
            fill_rounded_rect(out, r, m::RADIUS, c(theme, T::KanbanCard));
        }
        let fg = c(
            theme,
            if *on {
                T::KanbanCardFg
            } else {
                T::KanbanListMeta
            },
        );
        paint_text_title_elided(ts, out, text, bx + m::BUTTON_PAD_X, ty, m::TEXT_PX, w, fg);
        p.buttons.push((*b, r));
        bx += w;
    }
    x + total
}

/// Um botão solto (Hoje, a gaveta); devolve o rect.
fn button(out: &mut VectorScene, ts: &mut TextSystem, theme: Theme, r: Rect, text: &str, on: bool) {
    fill_rounded_rect(
        out,
        r,
        m::RADIUS,
        c(theme, if on { T::KanbanCard } else { T::KanbanSlot }),
    );
    let ty = text_top(ts, r.y, r.h, m::TEXT_PX);
    let fg = c(theme, T::KanbanListFg);
    paint_text_title_elided(
        ts,
        out,
        text,
        r.x + m::BUTTON_PAD_X,
        ty,
        m::TEXT_PX,
        r.w,
        fg,
    );
}

fn toolbar(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &TimelineState,
    undated: usize,
    p: &mut Painted,
) {
    let y = area.y + (m::TOOLBAR_H - m::BUTTON_H) / 2.0 - m::BUTTON_GAP;
    let mut x = area.x + m::TOOLBAR_PAD;
    let label = tr("kanban.views.timeline.group_by");
    let ty = text_top(ts, y, m::BUTTON_H + 2.0 * m::BUTTON_GAP, m::TEXT_PX);
    let lw = ts.prefix_width(label, m::TEXT_PX);
    paint_text_elided(
        ts,
        out,
        label,
        x,
        ty,
        m::TEXT_PX,
        lw + m::BUTTON_GAP,
        c(theme, T::KanbanListMeta),
    );
    x += lw + m::BUTTON_PAD_X;
    let groups: Vec<_> = GroupBy::ALL
        .iter()
        .map(|g| (Button::Group(*g), tr(g.key()), *g == st.group))
        .collect();
    x = segmented(out, ts, theme, (x, y), &groups, p) + m::GROUP_GAP;
    let zooms: Vec<_> = Zoom::ALL
        .iter()
        .map(|z| (Button::Zoom(*z), tr(z.key()), *z == st.zoom))
        .collect();
    x = segmented(out, ts, theme, (x, y), &zooms, p) + m::GROUP_GAP;
    let today = tr("kanban.views.timeline.today");
    let bh = m::BUTTON_H + 2.0 * m::BUTTON_GAP;
    let r = Rect::new(
        x,
        y,
        ts.prefix_width(today, m::TEXT_PX) + 2.0 * m::BUTTON_PAD_X,
        bh,
    );
    button(out, ts, theme, r, today, false);
    p.buttons.push((Button::Today, r));
    let text = tr_with("kanban.views.timeline.no_dates_n", &[("n", &undated)]);
    let w = ts.prefix_width(&text, m::TEXT_PX) + 2.0 * m::BUTTON_PAD_X;
    let r = Rect::new(area.x + area.w - m::TOOLBAR_PAD - w, y, w, bh);
    if r.x > x {
        button(out, ts, theme, r, &text, st.drawer_open);
        p.buttons.push((Button::Drawer, r));
    }
}

/// O nome de uma faixa na coluna da esquerda (com a amostra da lista, da etiqueta ou o avatar).
fn lane_label(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    lane: &Lane,
    r: Rect,
) {
    let mut x = r.x + m::LANE_LABEL_PAD_X;
    let row = Rect::new(r.x, r.y + m::LANE_PAD_Y, r.w, m::ROW_H);
    let swatch = |hue, shade| label_colors(theme, LabelColor { hue, shade });
    let mark = match &lane.key {
        LaneKey::List(l) => k
            .lists
            .get(l)
            .and_then(|l| l.color)
            .map(|h| swatch(h, Shade::Normal)),
        LaneKey::Label(l) => k
            .labels
            .get(l)
            .and_then(|l| l.color)
            .map(|lc| label_colors(theme, lc)),
        LaneKey::Member(who) => {
            let a = Rect::new(x, row.y + (row.h - m::BAR_H) / 2.0, m::BAR_H, m::BAR_H);
            let (bg, _) = swatch(avatar_hue(who), Shade::Normal);
            fill_rounded_rect(out, a, a.w / 2.0, bg);
            let s = initials(who);
            let w = ts.prefix_width(&s, m::SMALL_PX);
            let ty = text_top(ts, a.y, a.h, m::SMALL_PX);
            paint_text_elided(
                ts,
                out,
                &s,
                a.x + (a.w - w) / 2.0,
                ty,
                m::SMALL_PX,
                a.w,
                c(theme, T::KanbanAvatarInk),
            );
            x += a.w + m::BUTTON_PAD_X;
            None
        }
        LaneKey::NoMember | LaneKey::NoLabel => None,
    };
    if let Some((bg, _)) = mark {
        let s = Rect::new(
            x,
            row.y + (row.h - m::HANDLE_GRIP_H) / 2.0,
            m::HANDLE_W,
            m::HANDLE_GRIP_H,
        );
        fill_rounded_rect(out, s, m::BAR_RADIUS, bg);
        x += s.w + m::BUTTON_PAD_X;
    }
    let count = lane.count().to_string();
    let cw = ts.prefix_width(&count, m::SMALL_PX);
    let ty = text_top(ts, row.y, row.h, m::TEXT_PX);
    let max = r.x + r.w - x - cw - 2.0 * m::LANE_LABEL_PAD_X;
    paint_text_title_elided(
        ts,
        out,
        &lane.title,
        x,
        ty,
        m::TEXT_PX,
        max,
        c(theme, T::KanbanListFg),
    );
    let ty = text_top(ts, row.y, row.h, m::SMALL_PX);
    let cx = r.x + r.w - m::LANE_LABEL_PAD_X - cw;
    paint_text_elided(
        ts,
        out,
        &count,
        cx,
        ty,
        m::SMALL_PX,
        cw + m::LINE,
        c(theme, T::KanbanListMeta),
    );
}

/// Uma barra: a cor do cartão, o título dentro (ou à direita até `room`, se não cabe — o Trello) e
/// as pegas ao pairar.
#[allow(clippy::too_many_arguments)]
fn bar(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    card: &Card,
    r: Rect,
    hover: Option<Part>,
    room: f32,
) {
    let (mut bg, mut ink) = card_colors(theme, k, card);
    if card.due.is_some_and(|d| d.done) {
        bg = bg.multiply_alpha(m::DONE_ALPHA);
        ink = ink.multiply_alpha(m::DONE_ALPHA);
    }
    fill_rounded_rect(out, r, m::BAR_RADIUS, bg);
    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
    let tw = ts.prefix_width(&card.title, m::SMALL_PX);
    let outside = r.x + r.w + m::BAR_PAD_X;
    let fits = tw + 2.0 * m::BAR_PAD_X <= r.w;
    if !fits && room - outside >= m::BAR_MIN_TEXT_W.max(r.w) {
        let fg = c(theme, T::KanbanCardFg);
        paint_text_title_elided(
            ts,
            out,
            &card.title,
            outside,
            ty,
            m::SMALL_PX,
            room - outside,
            fg,
        );
    } else if r.w >= m::BAR_MIN_TEXT_W {
        let w = r.w - 2.0 * m::BAR_PAD_X;
        paint_text_title_elided(
            ts,
            out,
            &card.title,
            r.x + m::BAR_PAD_X,
            ty,
            m::SMALL_PX,
            w,
            ink,
        );
    }
    if hover.is_some() && r.w >= 3.0 * m::HANDLE_W {
        let grip = ink.multiply_alpha(m::DONE_ALPHA);
        let gy = r.y + (r.h - m::HANDLE_GRIP_H) / 2.0;
        for gx in [
            r.x + (m::HANDLE_W - m::HANDLE_GRIP_W) / 2.0,
            r.x + r.w - (m::HANDLE_W + m::HANDLE_GRIP_W) / 2.0,
        ] {
            let g = Rect::new(gx, gy, m::HANDLE_GRIP_W, m::HANDLE_GRIP_H);
            fill_rounded_rect(out, g, m::HANDLE_GRIP_W / 2.0, grip);
        }
    }
}

fn bar_rect(r: &Ruler, s: &Span, y: f32) -> Rect {
    let (x, w) = r.bar_x(s);
    Rect::new(
        x + m::LINE,
        y + (m::ROW_H - m::BAR_H) / 2.0,
        (w - 2.0 * m::LINE).max(m::LINE),
        m::BAR_H,
    )
}

fn shadow(out: &mut VectorScene, theme: Theme, r: Rect) {
    let s = Rect::new(r.x, r.y + m::FLIGHT_SHADOW, r.w, r.h);
    fill_rounded_rect(out, s, m::BAR_RADIUS, c(theme, T::KanbanShadowFlight));
}

/// ⭐ A linha do tempo inteira.
#[allow(clippy::too_many_lines)]
pub(super) fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut TimelineState,
    k: &Kanban,
    cx: &Ctx<'_>,
) {
    let mut p = Painted::default();
    let (lanes, undated) = layout::lanes(k, cx, st.group);
    toolbar(out, ts, theme, area, st, undated.len(), &mut p);
    let drawer_w = if st.drawer_open {
        m::DRAWER_W.min(area.w / 2.0)
    } else {
        0.0
    };
    let top = area.y + m::TOOLBAR_H;
    let main = Rect::new(
        area.x,
        top,
        area.w - drawer_w,
        (area.h - m::TOOLBAR_H).max(0.0),
    );
    let scale_h = m::SCALE_TOP_H + m::SCALE_BOTTOM_H;
    let label_w = m::LANE_LABEL_W.min(main.w / 2.0);
    let grid = Rect::new(
        main.x + label_w,
        main.y + scale_h,
        (main.w - label_w).max(0.0),
        (main.h - scale_h).max(0.0),
    );
    super::settle_anchor(st, cx, grid.w);
    let content: f32 = lanes.iter().map(lane_h).sum();
    st.scroll_y = st.scroll_y.clamp(0.0, (content - grid.h).max(0.0));
    let ruler = Ruler {
        grid,
        anchor: st
            .anchor
            .unwrap_or_else(|| ph2d_kanban_dates::day_of(cx.now.0, cx.zone)),
        ppd: st.zoom.ppd(),
        scroll_x: st.scroll_x,
    };
    let body = Rect::new(main.x, grid.y, main.w, grid.h);
    out.fill_rect(to_vrect(grid), c(theme, T::KanbanCard));
    out.push_clip(&to_vrect(grid));
    scale::columns(out, theme, &ruler, st.zoom, grid, cx.week_start);
    scale::today_line(out, theme, &ruler, cx, grid);
    out.pop_layer();
    let drag = st.drag.filter(|d| d.moved);
    let mut flight: Option<(CardId, Rect)> = None;
    let line = c(theme, T::Border);
    out.push_clip(&to_vrect(body));
    let mut y = grid.y - st.scroll_y;
    for lane in &lanes {
        let h = lane_h(lane);
        let band = Rect::new(main.x, y, main.w, h);
        p.lanes.push((lane.key.clone(), band));
        if y + h >= body.y && y <= body.y + body.h {
            lane_label(out, ts, theme, k, lane, Rect::new(main.x, y, label_w, h));
            out.push_clip(&to_vrect(grid));
            for (ri, row) in lane.rows.iter().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let ry = y + m::LANE_PAD_Y + ri as f32 * m::ROW_H;
                for (si, s) in row.iter().enumerate() {
                    let Some(card) = k.cards.get(&s.card) else {
                        continue;
                    };
                    let r = bar_rect(&ruler, s, ry);
                    let room = row
                        .get(si + 1)
                        .map_or(grid.x + grid.w, |n| bar_rect(&ruler, n, ry).x)
                        - m::BAR_PAD_X;
                    p.bars.push(Bar {
                        card: s.card,
                        rect: r,
                        lane: lane.key.clone(),
                        span: *s,
                    });
                    let held = drag.and_then(|d| match d.grab {
                        Grab::Card(id, part) if id == s.card && part != Part::Drawer => {
                            Some((d, part))
                        }
                        _ => None,
                    });
                    let Some((d, part)) = held else {
                        let hover = st.hover.filter(|h| h.0 == s.card).map(|h| h.1);
                        bar(out, ts, theme, k, card, r, hover, room);
                        continue;
                    };
                    let delta = ruler.index_at(d.at.0) - ruler.index_at(d.from.0);
                    let mut nr = bar_rect(&ruler, &moved(s, part, delta), ry);
                    if part == Part::Body {
                        fill_rounded_rect(out, r, m::BAR_RADIUS, c(theme, T::KanbanSlot));
                        nr.y += d.at.1 - d.from.1;
                        if flight.is_none() {
                            flight = Some((s.card, nr));
                        }
                    } else {
                        bar(out, ts, theme, k, card, nr, Some(part), room);
                    }
                }
            }
            out.pop_layer();
        }
        y += h;
        out.fill_rect(
            to_vrect(Rect::new(main.x, y - m::LINE, main.w, m::LINE)),
            line,
        );
    }
    out.push_clip(&to_vrect(grid));
    if let Some((id, r)) = flight
        && let Some(card) = k.cards.get(&id)
    {
        shadow(out, theme, r);
        bar(
            out,
            ts,
            theme,
            k,
            card,
            r,
            Some(Part::Body),
            grid.x + grid.w,
        );
    }
    out.pop_layer();
    out.pop_layer();
    out.fill_rect(
        to_vrect(Rect::new(grid.x - m::LINE, main.y, m::LINE, main.h)),
        line,
    );
    let head = Rect::new(grid.x, main.y, grid.w, scale_h);
    scale::header(out, ts, theme, &ruler, st.zoom, head, cx);
    let corner = Rect::new(main.x, main.y, label_w, scale_h);
    let title = tr(st.group.key());
    let ty = text_top(
        ts,
        corner.y + m::SCALE_TOP_H,
        m::SCALE_BOTTOM_H,
        m::SMALL_PX,
    );
    paint_text_title_elided(
        ts,
        out,
        title,
        corner.x + m::LANE_LABEL_PAD_X,
        ty,
        m::SMALL_PX,
        corner.w,
        c(theme, T::KanbanListMeta),
    );
    out.fill_rect(
        to_vrect(Rect::new(
            main.x,
            main.y + scale_h - m::LINE,
            label_w,
            m::LINE,
        )),
        line,
    );
    if st.drawer_open {
        let r = Rect::new(main.x + main.w, top, drawer_w, main.h);
        drawer::paint(out, ts, theme, r, st, k, &undated, &mut p);
    }
    if let Some(d) = drag
        && let Grab::Card(id, Part::Drawer) = d.grab
        && let Some(card) = k.cards.get(&id)
    {
        let r = if grid.contains(d.at.0, d.at.1) {
            let day = ruler.day_at(d.at.0);
            let s = Span {
                card: id,
                start: day,
                end: day,
            };
            let mut r = bar_rect(&ruler, &s, d.at.1 - m::ROW_H / 2.0);
            r.w = r.w.max(m::BAR_MIN_TEXT_W);
            r
        } else {
            let w = m::DRAWER_W - 2.0 * m::DRAWER_PAD;
            Rect::new(
                d.at.0 - w / 2.0,
                d.at.1 - m::DRAWER_CARD_H / 2.0,
                w,
                m::DRAWER_CARD_H,
            )
        };
        shadow(out, theme, r);
        bar(out, ts, theme, k, card, r, None, r.x + r.w);
    }
    p.ruler = Some(ruler);
    p.body = body;
    p.content_h = content;
    st.painted = p;
}

/// O ícone de fechar (a gaveta).
pub(super) fn close_icon(out: &mut VectorScene, theme: Theme, r: Rect) {
    let ir = Rect::new(
        r.x + (r.w - m::TEXT_PX) / 2.0,
        r.y + (r.h - m::TEXT_PX) / 2.0,
        m::TEXT_PX,
        m::TEXT_PX,
    );
    paint_icon(
        out,
        IconId::Close,
        ir,
        c(theme, T::KanbanListMeta),
        StrokeToken::Default.px(),
    );
}
