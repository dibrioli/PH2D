//! ⭐ **A moldura da aba** — a barra de cima (os três modos, os filtros, «+ Event», «Export .ics» e
//! o sino) e a caixa das notificações. O calendário é a vista da W7 ([`ph2d_kanban_views::calendar`]);
//! a coluna do Planner e «My cards» moram em [`crate::side`]. Cada coisa pintada com área de clique
//! entra em [`PlannerDocs::hits`] (a ordem é a do desenho: a última pintada ganha o clique).

use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello, resolve,
};
use ph2d_editor_core::widget::{Popover, paint_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{civil, day_of};
use ph2d_kanban_model::Happened;
use ph2d_kanban_views::datepick::{day_text, time_text};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Color, VectorScene};

use crate::docs::{Hit, Mode, PlannerDocs};
use crate::metrics as m;
use crate::notify::{Notice, What};
use crate::personal::GroupBy;
use crate::source::ColorBy;

/// A cor de um token no tema.
pub(crate) fn c(theme: Theme, t: T) -> Color {
    resolve(t, theme)
}

/// O topo de uma linha de texto de `px` centrada numa faixa de `row_h` que começa em `y`.
pub(crate) fn text_top(ts: &mut TextSystem, y: f32, row_h: f32, px: f32) -> f32 {
    let h = ts.layout("Ag", px, f32::INFINITY).height();
    y + ((row_h - h) / 2.0).max(0.0)
}

/// Um botão de texto da barra; devolve o rect.
fn button(
    d: &mut PlannerDocs,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    x: f32,
    y: f32,
    label: &str,
    hit: Hit,
    on: bool,
) -> Rect {
    let w = ts.prefix_width(label, m::TEXT_PX).ceil() + 2.0 * m::BUTTON_PAD_X;
    let r = Rect::new(x, y, w, m::BUTTON_H);
    let hover = d.hover == Some(hit);
    let (bg, fg) = match (on, hover) {
        (true, _) => (Some(T::Accent), T::AccentFg),
        (false, true) => (Some(T::Bg3), T::Text1),
        (false, false) => (None, T::Text1),
    };
    if let Some(bg) = bg {
        fill_rounded_rect(scene, r, m::RADIUS, c(theme, bg));
    }
    let ty = text_top(ts, y, m::BUTTON_H, m::TEXT_PX);
    paint_text_elided(
        ts,
        scene,
        label,
        x + m::BUTTON_PAD_X,
        ty,
        m::TEXT_PX,
        w,
        c(theme, fg),
    );
    d.hits.push((hit, r));
    r
}

fn badge(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, right: f32, y: f32, n: usize) {
    let t = n.to_string();
    let w = ts.prefix_width(&t, m::SMALL_PX).ceil() + 2.0 * m::BADGE_PAD_X;
    let w = w.max(m::BADGE_H);
    let r = Rect::new(right - w / 2.0, y - m::BADGE_H / 2.0, w, m::BADGE_H);
    fill_rounded_rect(scene, r, m::BADGE_H / 2.0, c(theme, T::KanbanDueOver));
    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_elided(
        ts,
        scene,
        &t,
        r.x + m::BADGE_PAD_X,
        ty,
        m::SMALL_PX,
        w,
        c(theme, T::KanbanDueOverFg),
    );
}

/// A barra de cima. Devolve a área por baixo dela.
pub(crate) fn top_bar(
    d: &mut PlannerDocs,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> Rect {
    let bar = Rect::new(area.x, area.y, area.w, m::TOP_H);
    scene.fill_rect(rect_to_vello(bar), c(theme, T::Bg2));
    let y = bar.y + (m::TOP_H - m::BUTTON_H) / 2.0;
    let title = tr("planner.title");
    let ty = text_top(ts, bar.y, m::TOP_H, m::TITLE_PX);
    paint_text_title_elided(
        ts,
        scene,
        title,
        bar.x + m::PAD,
        ty,
        m::TITLE_PX,
        bar.w,
        c(theme, T::Text1),
    );
    let mut x = bar.x + m::PAD + ts.prefix_width(title, m::TITLE_PX).ceil() + 2.0 * m::GAP;
    for (mode, key) in [
        (Mode::Calendar, "planner.mode.calendar"),
        (Mode::Planner, "planner.mode.planner"),
        (Mode::MyCards, "planner.mode.my_cards"),
    ] {
        let on = d.mode == mode;
        let r = button(d, scene, ts, theme, x, y, tr(key), Hit::Mode(mode), on);
        x = r.x + r.w + m::GAP / 2.0;
    }
    // Da direita para a esquerda.
    let mut right = bar.x + bar.w - m::PAD;
    let mut place = |d: &mut PlannerDocs,
                     scene: &mut VectorScene,
                     ts: &mut TextSystem,
                     label: String,
                     hit: Hit,
                     on: bool| {
        let w = ts.prefix_width(&label, m::TEXT_PX).ceil() + 2.0 * m::BUTTON_PAD_X;
        right -= w;
        let r = button(d, scene, ts, theme, right, y, &label, hit, on);
        right -= m::GAP;
        r
    };
    let unread = d.unread();
    let bell = place(
        d,
        scene,
        ts,
        tr("planner.bell").to_owned(),
        Hit::Bell,
        d.bell,
    );
    if unread > 0 {
        badge(scene, ts, theme, bell.x + bell.w, bell.y, unread);
    }
    place(
        d,
        scene,
        ts,
        tr("planner.export").to_owned(),
        Hit::Export,
        false,
    );
    if d.mode == Mode::MyCards {
        let key = match d.group_by {
            GroupBy::Kanban => "planner.group_by.kanban",
            GroupBy::Date => "planner.group_by.date",
        };
        place(d, scene, ts, tr(key).to_owned(), Hit::GroupBy, false);
    } else {
        place(
            d,
            scene,
            ts,
            tr("planner.new_event").to_owned(),
            Hit::NewEvent,
            d.pop.is_some(),
        );
        let key = match d.show.color_by {
            ColorBy::Kanban => "planner.color_by.kanban",
            ColorBy::Label => "planner.color_by.label",
        };
        place(d, scene, ts, tr(key).to_owned(), Hit::ColorBy, false);
        let label = d.show.person.as_ref().map_or_else(
            || tr("planner.person.everyone").to_owned(),
            |p| tr_with("planner.person.one", &[("name", &p.0)]),
        );
        place(d, scene, ts, label, Hit::Person, d.show.person.is_some());
    }
    Rect::new(
        area.x,
        area.y + m::TOP_H,
        area.w,
        (area.h - m::TOP_H).max(0.0),
    )
}

fn happened_key(h: &Happened) -> &'static str {
    match h {
        Happened::Created { .. } | Happened::Converted { .. } => "planner.notice.created",
        Happened::Moved { .. } => "planner.notice.moved",
        Happened::Renamed { .. } => "planner.notice.renamed",
        Happened::StartSet(_) | Happened::DueSet(_) => "planner.notice.dates",
        Happened::DueDone(true) => "planner.notice.completed",
        Happened::Archived => "planner.notice.archived",
        Happened::ChecklistAdded { .. }
        | Happened::ChecklistRemoved { .. }
        | Happened::ItemChecked { .. } => "planner.notice.checklist",
        Happened::Attached { .. } | Happened::Detached { .. } => "planner.notice.attached",
        Happened::MemberAdded(_) | Happened::MemberRemoved(_) => "planner.notice.members",
        _ => "planner.notice.updated",
    }
}

/// O texto de uma notificação (a linha de cima) e o pormenor (a de baixo).
#[must_use]
pub fn notice_text(n: &Notice, d: &PlannerDocs) -> (String, String) {
    let by = n.by.as_ref().map_or("", |m| m.0.as_str());
    let when = |at: i64| {
        let ct = civil(at, &d.zone);
        let today = day_of(d.now.0, &d.zone);
        tr_with(
            "planner.when",
            &[
                ("day", &day_text(ct.date, today)),
                ("time", &time_text(ct.hour, ct.minute)),
            ],
        )
    };
    let args = |key: &str| tr_with(key, &[("by", &by), ("card", &n.place)]);
    match &n.what {
        What::Mention { text, .. } => (args("planner.notice.mention"), text.clone()),
        What::Assigned => (args("planner.notice.assigned"), String::new()),
        What::Changed(h) => (args(happened_key(h)), String::new()),
        What::DueReminder { due } => (
            tr_with("planner.notice.due", &[("card", &n.place)]),
            when(due.0),
        ),
        What::EventReminder { title, start } => (
            tr_with("planner.notice.event", &[("title", title)]),
            when(start.0),
        ),
    }
}

/// A caixa das notificações sob o sino.
pub(crate) fn bell_box(
    d: &mut PlannerDocs,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let Some(bell) = d.probe_hit(Hit::Bell) else {
        return;
    };
    let rows = d.notices.len().max(1);
    #[allow(clippy::cast_precision_loss)]
    let want = m::GROUP_H + rows as f32 * m::NOTICE_H + m::PAD;
    let h = want.min(area.h - m::PAD);
    let x = (bell.x + bell.w - m::BELL_W).max(area.x + m::PAD);
    let r = Rect::new(x, bell.y + bell.h + m::GAP, m::BELL_W, h);
    paint_popover(&Popover::new(crate::POP_NODE), r, scene, theme);
    let head = Rect::new(r.x + m::PAD, r.y, r.w - 2.0 * m::PAD, m::GROUP_H);
    let ty = text_top(ts, head.y, head.h, m::TEXT_PX);
    paint_text_elided(
        ts,
        scene,
        tr("planner.bell"),
        head.x,
        ty,
        m::TEXT_PX,
        head.w / 2.0,
        c(theme, T::Text1),
    );
    let all = tr("planner.mark_all");
    let w = ts.prefix_width(all, m::SMALL_PX).ceil();
    let mr = Rect::new(head.x + head.w - w, head.y, w, head.h);
    let ty = text_top(ts, head.y, head.h, m::SMALL_PX);
    paint_text_elided(
        ts,
        scene,
        all,
        mr.x,
        ty,
        m::SMALL_PX,
        w + 1.0,
        c(theme, T::Text2),
    );
    d.hits.push((Hit::MarkAll, mr));
    let mut y = head.y + head.h;
    if d.notices.is_empty() {
        let ty = text_top(ts, y, m::NOTICE_H, m::SMALL_PX);
        paint_text_elided(
            ts,
            scene,
            tr("planner.no_notices"),
            head.x,
            ty,
            m::SMALL_PX,
            head.w,
            c(theme, T::Text2),
        );
        return;
    }
    let notices = d.notices.clone();
    for n in &notices {
        if y + m::NOTICE_H > r.y + r.h {
            break;
        }
        let row = Rect::new(
            r.x + m::GAP,
            y,
            r.w - 2.0 * m::GAP,
            m::NOTICE_H - m::GAP / 2.0,
        );
        let hit = Hit::Notice(n.key);
        if d.hover == Some(hit) {
            fill_rounded_rect(scene, row, m::RADIUS, c(theme, T::Bg3));
        } else if n.unread {
            fill_rounded_rect(scene, row, m::RADIUS, c(theme, T::Bg2));
        }
        let (top, detail) = notice_text(n, d);
        let ink = if n.unread { T::Text1 } else { T::Text2 };
        let x = row.x + m::GAP;
        paint_text_elided(
            ts,
            scene,
            &top,
            x,
            row.y + m::GAP,
            m::TEXT_PX,
            row.w - 2.0 * m::GAP,
            c(theme, ink),
        );
        let ty = row.y + m::GAP + m::TEXT_PX + m::GAP / 2.0;
        paint_text_elided(
            ts,
            scene,
            &detail,
            x,
            ty,
            m::SMALL_PX,
            row.w - 2.0 * m::GAP,
            c(theme, T::Text2),
        );
        d.hits.push((hit, row));
        y += m::NOTICE_H;
    }
}
