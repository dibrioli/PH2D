//! ⭐ **A lista suspensa de um campo «Dropdown»** (dono 30/10): o clique no valor abre a lista
//! INTEIRA por baixo dele (ou por cima, se há mais espaço) — «None» e cada opção, a actual com o
//! visto — e rola pela roda, pela barra e pelas setas ([`ScrollList`]); escolher fecha.

use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{Back, BackHit, Pop, PopHit, ROW};
use crate::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::widget::{SCROLLBAR_W, ScrollList, scrollbar_is_needed};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{FieldId, FieldKind, FieldOption, FieldValue};
use ph2d_tokens::{ColorToken as T, StrokeToken};

/// A lista aberta: o campo, a linha realçada pelas setas e a rolagem.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Choice {
    pub field: FieldId,
    pub cursor: usize,
    pub s: ScrollList,
}

impl Choice {
    pub(crate) fn new(field: FieldId, cursor: usize) -> Self {
        let s = ScrollList::default();
        Self { field, cursor, s }
    }

    /// As setas: a linha realçada anda e fica à vista.
    pub(crate) fn step(&mut self, delta: isize, rows: usize) {
        self.cursor = self
            .cursor
            .saturating_add_signed(delta)
            .min(rows.saturating_sub(1));
        #[allow(clippy::cast_precision_loss)]
        let top = u::SMALL_GAP + self.cursor as f32 * ROW;
        self.s.reveal(top, ROW, u::SMALL_GAP);
    }
}

/// As opções do campo `f` do Kanban (vazio se não é uma lista suspensa).
pub(crate) fn options(k: &ph2d_kanban_model::Kanban, f: FieldId) -> &[FieldOption] {
    match k.fields.get(&f).map(|d| &d.kind) {
        Some(FieldKind::Dropdown(o)) => o,
        _ => &[],
    }
}

/// A linha `i` da lista: `0` = «nenhum», depois cada opção.
pub(crate) fn value_at(opts: &[FieldOption], i: usize) -> Option<FieldValue> {
    i.checked_sub(1)
        .and_then(|j| opts.get(j))
        .map(|o| FieldValue::Option(o.id))
}

/// A linha do valor actual (`0` = nenhum).
pub(crate) fn row_of(opts: &[FieldOption], v: Option<&FieldValue>) -> usize {
    match v {
        Some(FieldValue::Option(o)) => opts.iter().position(|x| x.id == *o).map_or(0, |i| i + 1),
        _ => 0,
    }
}

/// Onde a lista de `content_h` cabe por baixo (ou por cima) do valor `a`, dentro de `area`.
fn place(area: Rect, a: Rect, content_h: f32) -> Rect {
    let w = a.w.max(u::SIDE_W);
    let x = a.x.min(area.x + area.w - w - u::GAP).max(area.x + u::GAP);
    let below = (area.y + area.h - u::GAP) - (a.y + a.h + u::SMALL_GAP);
    let above = (a.y - u::SMALL_GAP) - (area.y + u::GAP);
    if content_h <= below || below >= above {
        Rect::new(
            x,
            a.y + a.h + u::SMALL_GAP,
            w,
            content_h.min(below).max(ROW),
        )
    } else {
        let h = content_h.min(above).max(ROW);
        Rect::new(x, a.y - u::SMALL_GAP - h, w, h)
    }
}

/// ⭐ Pinta a lista aberta por baixo do valor `anchor` (se há).
pub(crate) fn paint(ui: &mut Ui<'_>, back: &mut Back, cx: &Ctx<'_>, anchor: Option<Rect>) {
    let area = back.area;
    let Some(Pop::Choice(c)) = back.pop.as_mut() else {
        return;
    };
    let Some(anchor) = anchor else {
        return;
    };
    let opts = options(cx.k, c.field);
    let current = row_of(opts, cx.card.fields.get(&c.field));
    let rows = opts.len() + 1;
    #[allow(clippy::cast_precision_loss)]
    let content_h = rows as f32 * ROW + 2.0 * u::SMALL_GAP;
    let r = place(area, anchor, content_h);
    c.s.set(r, content_h);
    crate::paint::shadow(
        ui.scene,
        r,
        m::MENU_RADIUS,
        ui.c(T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    let bg = ui.c(T::KanbanCard);
    fill_rounded_rect(ui.scene, r, m::MENU_RADIUS, bg);
    let ring = ui.c(T::KanbanSlot);
    stroke_rounded_rect(ui.scene, r, m::MENU_RADIUS, StrokeToken::Default.px(), ring);
    ui.hit(r, BackHit::Pop(PopHit::Inside));
    let bar = scrollbar_is_needed(content_h, r.h);
    let rw = r.w - if bar { SCROLLBAR_W + u::SMALL_GAP } else { 0.0 };
    ui.scene
        .push_clip(&ph2d_editor_core::paint::rect_to_vello(r));
    for i in 0..rows {
        #[allow(clippy::cast_precision_loss)]
        let y = r.y + u::SMALL_GAP + i as f32 * ROW - c.s.scroll;
        if y + ROW < r.y || y > r.y + r.h {
            continue;
        }
        let row = Rect::new(r.x + u::SMALL_GAP, y, rw - 2.0 * u::SMALL_GAP, ROW);
        #[allow(clippy::cast_possible_truncation)]
        let h = BackHit::Pop(PopHit::Choice(i as u16));
        if ui.hover == Some(h) || i == c.cursor {
            fill_rounded_rect(ui.scene, row, m::BUTTON_RADIUS, ui.c(T::KanbanSlot));
        }
        let tx = row.x + m::BUTTON_PAD_X;
        match i.checked_sub(1).and_then(|j| opts.get(j)) {
            Some(opt) => super::paint::option_chip(ui, opt, tx, y),
            None => {
                let meta = ui.meta();
                let w = row.w - 2.0 * m::BUTTON_PAD_X - ROW;
                let none = tr("kanban.back.field_none");
                ui.line(none, u::Band::new(tx, y, w, ROW), u::BODY_PX, meta);
            }
        }
        if i == current {
            let fg = ui.fg();
            let at = Rect::new(row.x + row.w - ROW, y, ROW, ROW);
            crate::paint::icon(ui.scene, IconId::Check, at, fg);
        }
        let top = row.y.max(r.y);
        let bottom = (row.y + row.h).min(r.y + r.h);
        ui.hit(Rect::new(row.x, top, row.w, bottom - top), h);
    }
    ui.scene.pop_layer();
    let body = Rect::new(
        r.x,
        r.y + u::SMALL_GAP,
        r.w - u::SMALL_GAP,
        r.h - 2.0 * u::SMALL_GAP,
    );
    let hovered = ui.hover == Some(BackHit::Pop(PopHit::ChoiceBar));
    if let Some(track) = c.s.paint_bar(body, hovered, ui.scene, ui.theme) {
        ui.hit(track, BackHit::Pop(PopHit::ChoiceBar));
    }
}
