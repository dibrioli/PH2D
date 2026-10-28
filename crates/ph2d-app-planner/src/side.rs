//! **A coluna do Planner** (Inbox + os meus cartões sem data) e **«My cards»**. Cada linha é um
//! cartão do idioma do Kanban (fundo `kanban-card`, cantos de `8`); o que se arrasta da coluna para o
//! calendário vira um bloco de tempo, e o clique em «Schedule» põe-no na próxima meia hora.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{civil, day_of};
use ph2d_kanban_model::{BlockOf, CardRef};
use ph2d_kanban_views::datepick::{day_text, time_text};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use crate::docs::{Hit, PlannerDocs};
use crate::metrics as m;
use crate::ui::{c, text_top};

fn section(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
) {
    let ty = text_top(ts, y, m::SECTION_H, m::SMALL_PX);
    paint_text_elided(
        ts,
        scene,
        label,
        x,
        ty,
        m::SMALL_PX,
        w,
        c(theme, T::KanbanListMeta),
    );
}

/// Uma linha-cartão: o texto e, à direita, um botão pequeno. Devolve o rect do botão.
#[allow(clippy::too_many_arguments)]
fn row(
    d: &mut PlannerDocs,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: Rect,
    text: &str,
    body: Hit,
    action: Option<(&str, Hit)>,
) {
    fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, T::KanbanCard));
    if d.hover == Some(body) {
        fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, T::KanbanCardRaised));
    }
    d.hits.push((body, r));
    let mut text_w = r.w - 2.0 * m::BUTTON_PAD_X;
    if let Some((label, hit)) = action {
        let w = ts.prefix_width(label, m::SMALL_PX).ceil() + 2.0 * m::BADGE_PAD_X;
        let b = Rect::new(
            r.x + r.w - w - m::GAP / 2.0,
            r.y + (r.h - m::BADGE_H) / 2.0,
            w,
            m::BADGE_H,
        );
        let bg = if d.hover == Some(hit) {
            T::Accent
        } else {
            T::Bg2
        };
        let fg = if d.hover == Some(hit) {
            T::AccentFg
        } else {
            T::Text2
        };
        fill_rounded_rect(scene, b, m::RADIUS, c(theme, bg));
        let ty = text_top(ts, b.y, b.h, m::SMALL_PX);
        paint_text_elided(
            ts,
            scene,
            label,
            b.x + m::BADGE_PAD_X,
            ty,
            m::SMALL_PX,
            w,
            c(theme, fg),
        );
        d.hits.push((hit, b));
        text_w -= w + m::GAP;
    }
    let ty = text_top(ts, r.y, r.h, m::TEXT_PX);
    paint_text_elided(
        ts,
        scene,
        text,
        r.x + m::BUTTON_PAD_X,
        ty,
        m::TEXT_PX,
        text_w,
        c(theme, T::KanbanCardFg),
    );
}

/// A coluna do Planner; devolve a área que sobra para o calendário.
pub(crate) fn planner_column(
    d: &mut PlannerDocs,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> Rect {
    let col = Rect::new(area.x, area.y, m::SIDE_W.min(area.w / 2.0), area.h);
    scene.fill_rect(rect_to_vello(col), c(theme, T::KanbanList));
    let x = col.x + m::PAD;
    let w = col.w - 2.0 * m::PAD;
    let mut y = col.y + m::PAD;
    section(scene, ts, theme, x, y, w, tr("planner.inbox"));
    y += m::SECTION_H;
    // A captura: «Add to Inbox» (Enter grava e deixa o campo aberto para o seguinte).
    let field = Rect::new(x, y, w, m::FIELD_H);
    fill_rounded_rect(scene, field, m::RADIUS, c(theme, T::Bg1));
    let (text, ink) = match &d.capture {
        Some(f) if !f.text.is_empty() => (f.text.clone(), T::Text1),
        _ => (tr("planner.inbox.add").to_owned(), T::Text2),
    };
    let ty = text_top(ts, field.y, field.h, m::TEXT_PX);
    paint_text_elided(
        ts,
        scene,
        &text,
        field.x + m::BUTTON_PAD_X,
        ty,
        m::TEXT_PX,
        field.w - 2.0 * m::BUTTON_PAD_X,
        c(theme, ink),
    );
    if d.capture.is_some() {
        ph2d_editor_core::paint::stroke_rounded_rect(
            scene,
            field,
            m::RADIUS,
            1.0,
            c(theme, T::Accent),
        );
    }
    d.hits.push((Hit::InboxField, field));
    y += m::FIELD_H + m::ROW_GAP;
    let (items, undated) = {
        let set = d.set.borrow();
        let items: Vec<_> = set
            .agenda()
            .inbox_of(&d.me)
            .into_iter()
            .map(|i| (i.id, i.text.clone()))
            .collect();
        let undated: Vec<(CardRef, String)> = crate::personal::undated_mine(&set, &d.me)
            .into_iter()
            .map(|(r, c)| (r, c.title.clone()))
            .collect();
        (items, undated)
    };
    let to_card = tr("planner.inbox.to_card");
    for (id, text) in items {
        let r = Rect::new(x, y, w, m::ROW_H);
        row(
            d,
            scene,
            ts,
            theme,
            r,
            &text,
            Hit::InboxItem(id),
            Some((to_card, Hit::InboxToCard(id))),
        );
        y += m::ROW_H + m::ROW_GAP;
    }
    y += m::GAP;
    section(scene, ts, theme, x, y, w, tr("planner.undated"));
    y += m::SECTION_H;
    let schedule = tr("planner.schedule");
    for (r_, title) in undated {
        let r = Rect::new(x, y, w, m::ROW_H);
        row(
            d,
            scene,
            ts,
            theme,
            r,
            &title,
            Hit::Card(r_),
            Some((schedule, Hit::Schedule(BlockOf::Card(r_)))),
        );
        y += m::ROW_H + m::ROW_GAP;
    }
    Rect::new(col.x + col.w, area.y, (area.w - col.w).max(0.0), area.h)
}

/// O fantasma do arrasto da coluna (por cima do calendário).
pub(crate) fn ghost(d: &PlannerDocs, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
    let Some(g) = d.drag else { return };
    let label = crate::personal::label_of(&d.set.borrow(), g.of).unwrap_or_default();
    let w = m::SIDE_W - 2.0 * m::PAD;
    let r = Rect::new(g.at.0 - w / 2.0, g.at.1 - m::ROW_H / 2.0, w, m::ROW_H);
    fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, T::KanbanCardRaised));
    let ty = text_top(ts, r.y, r.h, m::TEXT_PX);
    paint_text_elided(
        ts,
        scene,
        &label,
        r.x + m::BUTTON_PAD_X,
        ty,
        m::TEXT_PX,
        r.w,
        c(theme, T::KanbanCardFg),
    );
}

/// «My cards»: os grupos e as linhas, numa coluna centrada.
pub(crate) fn my_cards(
    d: &mut PlannerDocs,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    scene.fill_rect(rect_to_vello(area), c(theme, T::Bg1));
    let w = m::LIST_W.min(area.w - 2.0 * m::PAD);
    let x = area.x + (area.w - w) / 2.0;
    let mut y = area.y + m::PAD;
    let groups = crate::personal::my_cards(&d.set.borrow(), &d.me, d.group_by, d.now, &d.zone);
    if groups.is_empty() {
        let ty = text_top(ts, y, m::GROUP_H, m::TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            tr("planner.my_cards.empty"),
            x,
            ty,
            m::TEXT_PX,
            w,
            c(theme, T::Text2),
        );
        return;
    }
    let today = day_of(d.now.0, &d.zone);
    for g in groups {
        let ty = text_top(ts, y, m::GROUP_H, m::TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            &g.title,
            x,
            ty,
            m::TEXT_PX,
            w,
            c(theme, T::Text1),
        );
        y += m::GROUP_H;
        for r in g.cards {
            let (title, due) = {
                let set = d.set.borrow();
                let Some(card) = set.card(r) else { continue };
                let due = card.due.map(|dd| {
                    let ct = civil(dd.at.0, &d.zone);
                    tr_with(
                        "planner.when",
                        &[
                            ("day", &day_text(ct.date, today)),
                            ("time", &time_text(ct.hour, ct.minute)),
                        ],
                    )
                });
                (card.title.clone(), due)
            };
            let rr = Rect::new(x, y, w, m::ROW_H);
            let open = tr("planner.open");
            let text = due.map_or_else(
                || title.clone(),
                |due| tr_with("planner.card_due", &[("title", &title), ("due", &due)]),
            );
            row(
                d,
                scene,
                ts,
                theme,
                rr,
                &text,
                Hit::Card(r),
                Some((open, Hit::Card(r))),
            );
            y += m::ROW_H + m::ROW_GAP;
            if y > area.y + area.h {
                return;
            }
        }
        y += m::GAP;
    }
}
