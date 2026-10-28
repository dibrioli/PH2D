//! Os popovers do calendário: o COMPOSITOR do «+» (título + lista, como o «Add a card» do Trello)
//! e a lista de um dia (o «+N more»). Ancorados à célula do dia, presos à área da vista.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, stroke_rounded_rect};
use ph2d_editor_core::widget::paint_dropdown;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_dates::CivilDate;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use super::Composer;
use super::metrics as m;
use super::paint::item;
use crate::common::{c, text_top};
use crate::datepick::day_text;
use crate::item::{Item, ItemKey};

/// As peças do popover aberto (o rato lê-as).
#[derive(Clone, Debug, Default)]
pub(crate) struct PopRects {
    pub panel: Rect,
    pub chip: Rect,
    pub add: Rect,
    pub cancel: Rect,
    pub rows: Vec<(Rect, ItemKey)>,
}

/// O painel de altura `h` junto à célula de `day`, dentro de `area`.
fn anchor(area: Rect, cells: &[(Rect, CivilDate)], day: CivilDate, h: f32) -> Rect {
    let cell = cells
        .iter()
        .find(|(_, d)| *d == day)
        .map_or(area, |(r, _)| *r);
    let w = m::POP_W.min(area.w - 2.0 * m::PAD);
    let x = cell.x.min(area.x + area.w - m::PAD - w).max(area.x + m::PAD);
    let mut y = cell.y + m::DAY_NUM_H;
    if y + h > area.y + area.h - m::PAD {
        y = (area.y + area.h - m::PAD - h).max(area.y + m::PAD);
    }
    Rect::new(x, y, w, h)
}

fn panel(out: &mut VectorScene, theme: Theme, r: Rect) {
    let sh = Rect::new(r.x, r.y + m::POP_SHADOW, r.w, r.h);
    fill_rounded_rect(out, sh, m::POP_RADIUS, c(theme, T::KanbanShadowHover));
    fill_rounded_rect(out, r, m::POP_RADIUS, c(theme, T::KanbanCard));
    stroke_rounded_rect(out, r, m::POP_RADIUS, m::GRID_LINE, c(theme, T::Border));
}

fn label(out: &mut VectorScene, ts: &mut TextSystem, theme: Theme, r: Rect, text: &str, t: T) {
    let y = text_top(ts, r.y, r.h, m::BUTTON_PX);
    paint_text_elided(ts, out, text, r.x, y, m::BUTTON_PX, r.w, c(theme, t));
}

/// Um botão de texto: cheio (`strong`, o azul do Trello) ou só a palavra.
fn text_button(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    at: (f32, f32),
    text: &str,
    strong: bool,
) -> Rect {
    let w = ts.prefix_width(text, m::BUTTON_PX) + 2.0 * m::BUTTON_PAD_X;
    let r = Rect::new(at.0, at.1, w, m::BUTTON_H);
    let ink = if strong {
        fill_rounded_rect(out, r, m::BUTTON_RADIUS, c(theme, T::KanbanBlueBold));
        T::KanbanLabelInkBold
    } else {
        T::KanbanCardMeta
    };
    let y = text_top(ts, r.y, r.h, m::BUTTON_PX);
    paint_text_elided(ts, out, text, r.x + m::BUTTON_PAD_X, y, m::BUTTON_PX, w, c(theme, ink));
    r
}

/// ⭐ O compositor: «Add a card» · o título (a caixa com o cursor) · a lista · Add card / Cancel.
pub(crate) fn composer(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    cells: &[(Rect, CivilDate)],
    comp: &Composer,
) -> PopRects {
    let h = 2.0 * m::POP_PAD + m::POP_TITLE_H + 2.0 * m::FIELD_H + 2.0 * m::POP_GAP + m::BUTTON_H;
    let r = anchor(area, cells, comp.day, h);
    panel(out, theme, r);
    let inner = Rect::new(r.x + m::POP_PAD, r.y + m::POP_PAD, r.w - 2.0 * m::POP_PAD, m::POP_TITLE_H);
    label(out, ts, theme, inner, tr("kanban.views.calendar.add"), T::KanbanCardFg);
    let field = Rect::new(inner.x, inner.y + inner.h, inner.w, m::FIELD_H);
    fill_rounded_rect(out, field, m::BUTTON_RADIUS, c(theme, T::KanbanCard));
    stroke_rounded_rect(out, field, m::BUTTON_RADIUS, m::FOCUS_STROKE, c(theme, T::FocusRing));
    let tx = field.x + m::ITEM_PAD_X;
    let ty = text_top(ts, field.y, field.h, m::FIELD_PX);
    let tw = field.w - 2.0 * m::ITEM_PAD_X;
    if comp.field.text.is_empty() {
        let ph = tr("kanban.views.calendar.placeholder");
        paint_text_elided(ts, out, ph, tx, ty, m::FIELD_PX, tw, c(theme, T::KanbanCardMeta));
    } else {
        let t = &comp.field.text;
        paint_text_elided(ts, out, t, tx, ty, m::FIELD_PX, tw, c(theme, T::KanbanCardFg));
    }
    let pre = comp.field.text.get(..comp.field.caret).unwrap_or_default();
    let cx = tx + ts.prefix_width(pre, m::FIELD_PX);
    let caret = Rect::new(cx, field.y + m::CELL_PAD, m::CARET_W, field.h - 2.0 * m::CELL_PAD);
    out.fill_rect(ph2d_editor_core::paint::rect_to_vello(caret), c(theme, T::KanbanCardFg));
    let chip = Rect::new(inner.x, field.y + field.h + m::POP_GAP, inner.w, m::FIELD_H);
    let by = chip.y + chip.h + m::POP_GAP;
    let add = text_button(out, ts, theme, (inner.x, by), tr("kanban.views.calendar.add_card"), true);
    let cancel = text_button(
        out,
        ts,
        theme,
        (add.x + add.w + m::BUTTON_GAP, by),
        tr("kanban.views.calendar.cancel"),
        false,
    );
    paint_dropdown(&comp.list, chip, out, ts, theme);
    PopRects {
        panel: r,
        chip,
        add,
        cancel,
        rows: Vec::new(),
    }
}

/// A lista de um dia: o dia por título, um item por linha, e «Add a card» no pé.
#[allow(clippy::too_many_arguments)]
pub(crate) fn day(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    cells: &[(Rect, CivilDate)],
    date: CivilDate,
    today: CivilDate,
    items: &[&Item],
) -> PopRects {
    let pitch = m::POP_ROW_H + m::LANE_GAP;
    let fixed = 2.0 * m::POP_PAD + m::POP_TITLE_H + m::POP_GAP + m::BUTTON_H;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let fit = ((area.h - 2.0 * m::PAD - fixed) / pitch).max(0.0).floor() as usize;
    let shown = items.len().min(fit);
    #[allow(clippy::cast_precision_loss)]
    let h = fixed + shown as f32 * pitch;
    let r = anchor(area, cells, date, h);
    panel(out, theme, r);
    let inner = Rect::new(r.x + m::POP_PAD, r.y + m::POP_PAD, r.w - 2.0 * m::POP_PAD, m::POP_TITLE_H);
    label(out, ts, theme, inner, &day_text(date, today), T::KanbanCardFg);
    let mut rows = Vec::new();
    let mut y = inner.y + inner.h;
    for it in items.iter().take(shown) {
        let rr = Rect::new(inner.x, y, inner.w, m::POP_ROW_H);
        item(out, ts, theme, rr, it, (false, true, true), &it.title);
        rows.push((rr, it.key));
        y += pitch;
    }
    let add = text_button(
        out,
        ts,
        theme,
        (inner.x, y + m::POP_GAP - m::LANE_GAP),
        tr("kanban.views.calendar.add"),
        false,
    );
    PopRects {
        panel: r,
        add,
        rows,
        ..PopRects::default()
    }
}
