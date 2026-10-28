//! A gaveta «No dates»: os cartões à vista sem início nem entrega, prontos a largar num dia.

use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_block, paint_text_elided, paint_text_title_elided,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{CardId, Kanban};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use super::metrics as m;
use super::paint::{card_colors, close_icon};
use super::scale::to_vrect;
use super::{Button, Grab, Painted, Part, TimelineState};
use crate::common::{c, text_top};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: Rect,
    st: &mut TimelineState,
    k: &Kanban,
    undated: &[CardId],
    p: &mut Painted,
) {
    out.fill_rect(to_vrect(r), c(theme, T::KanbanList));
    out.fill_rect(
        to_vrect(Rect::new(r.x, r.y, m::LINE, r.h)),
        c(theme, T::Border),
    );
    let head = Rect::new(r.x, r.y, r.w, m::DRAWER_HEAD_H);
    let ty = text_top(ts, head.y, head.h, m::TEXT_PX);
    let title = tr("kanban.views.timeline.no_dates");
    let x = r.x + m::DRAWER_PAD;
    let tw = ts.prefix_width(title, m::TEXT_PX);
    paint_text_title_elided(
        ts,
        out,
        title,
        x,
        ty,
        m::TEXT_PX,
        tw + m::LINE,
        c(theme, T::KanbanListFg),
    );
    let n = undated.len().to_string();
    paint_text_elided(
        ts,
        out,
        &n,
        x + tw + m::BUTTON_PAD_X,
        ty,
        m::TEXT_PX,
        head.w,
        c(theme, T::KanbanListMeta),
    );
    let close = Rect::new(
        r.x + r.w - m::DRAWER_PAD - m::CLOSE_W,
        head.y + (head.h - m::CLOSE_W) / 2.0,
        m::CLOSE_W,
        m::CLOSE_W,
    );
    close_icon(out, theme, close);
    p.buttons.push((Button::DrawerClose, close));
    let list = Rect::new(r.x, head.y + head.h, r.w, (r.h - head.h).max(0.0));
    let w = r.w - 2.0 * m::DRAWER_PAD;
    let meta = c(theme, T::KanbanListMeta);
    if undated.is_empty() {
        paint_text_block(
            ts,
            out,
            tr("kanban.views.timeline.no_dates_empty"),
            x,
            list.y,
            m::SMALL_PX,
            w,
            meta,
        );
        p.drawer = Some(r);
        return;
    }
    let hint = tr("kanban.views.timeline.no_dates_hint");
    let hint_h = ts.layout(hint, m::SMALL_PX, w).height() + m::DRAWER_GAP;
    #[allow(clippy::cast_precision_loss)]
    let content = undated.len() as f32 * (m::DRAWER_CARD_H + m::DRAWER_GAP) + hint_h;
    st.drawer_scroll = st.drawer_scroll.clamp(0.0, (content - list.h).max(0.0));
    let held = st.drag.filter(|d| d.moved).and_then(|d| match d.grab {
        Grab::Card(id, Part::Drawer) => Some(id),
        _ => None,
    });
    out.push_clip(&to_vrect(list));
    let mut y = list.y - st.drawer_scroll;
    paint_text_block(ts, out, hint, x, y, m::SMALL_PX, w, meta);
    y += hint_h;
    for id in undated {
        let Some(card) = k.cards.get(id) else {
            continue;
        };
        let cr = Rect::new(x, y, w, m::DRAWER_CARD_H);
        p.drawer_cards.push((*id, cr));
        if held == Some(*id) {
            fill_rounded_rect(out, cr, m::BAR_RADIUS, c(theme, T::KanbanSlot));
        } else {
            fill_rounded_rect(out, cr, m::BAR_RADIUS, c(theme, T::KanbanCard));
            let (tone, _) = card_colors(theme, k, card);
            let strip = Rect::new(cr.x, cr.y, m::HANDLE_W / 2.0, cr.h);
            fill_rounded_rect(out, strip, m::BAR_RADIUS / 2.0, tone);
            let ty = text_top(ts, cr.y, cr.h, m::SMALL_PX);
            let fg = c(theme, T::KanbanCardFg);
            let tx = cr.x + m::BAR_PAD_X + m::HANDLE_W / 2.0;
            paint_text_title_elided(
                ts,
                out,
                &card.title,
                tx,
                ty,
                m::SMALL_PX,
                cr.x + cr.w - tx - m::BAR_PAD_X,
                fg,
            );
        }
        y += m::DRAWER_CARD_H + m::DRAWER_GAP;
    }
    out.pop_layer();
    p.drawer_content_h = content;
    p.drawer = Some(r);
}
