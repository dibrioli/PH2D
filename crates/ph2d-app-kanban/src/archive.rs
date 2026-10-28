//! ⭐ **O arquivo** — o painel à direita com os cartões e as listas arquivados: devolver ao quadro
//! ou apagar de vez (o desfazer devolve também o que se apagou de vez).

use crate::metrics as m;
use crate::paint::{c, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, paint_text_title_elided,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{Kanban, RecordId};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// O painel: que separador está aberto e a rolagem.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Panel {
    pub lists: bool,
    pub scroll: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    Close,
    ShowCards,
    ShowLists,
    Restore(RecordId),
    Delete(RecordId),
    /// Dentro do painel, sem linha.
    Inside,
}

/// Um item arquivado no painel.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Item {
    pub id: RecordId,
    pub title: String,
    pub rect: Rect,
    pub restore: Rect,
    pub delete: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placed {
    pub rect: Rect,
    pub close: Rect,
    pub cards: Rect,
    pub lists: Rect,
    pub items: Vec<Item>,
}

/// Os arquivados, do mais recente para o mais antigo (o maior id primeiro).
fn archived(k: &Kanban, lists: bool) -> Vec<(RecordId, String)> {
    if lists {
        k.lists
            .values()
            .rev()
            .filter(|l| l.archived && !l.meta.deleted)
            .map(|l| (RecordId::List(l.id), l.title.clone()))
            .collect()
    } else {
        k.cards
            .values()
            .rev()
            .filter(|c| c.archived && !c.meta.deleted)
            .map(|c| (RecordId::Card(c.id), c.title.clone()))
            .collect()
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn place(panel: Panel, k: &Kanban, board: Rect, ts: &mut TextSystem) -> Placed {
    let rect = Rect::new(board.x + board.w - m::PANEL_W, board.y, m::PANEL_W, board.h);
    let x = rect.x + m::MENU_PAD;
    let w = rect.w - 2.0 * m::MENU_PAD;
    let close = Rect::new(
        rect.x + rect.w - m::MENU_PAD - m::BUTTON_H,
        rect.y + m::MENU_PAD,
        m::BUTTON_H,
        m::BUTTON_H,
    );
    let tabs_y = close.y + close.h + m::MENU_PAD;
    let cards = Rect::new(x, tabs_y, w / 2.0, m::BUTTON_H);
    let lists = Rect::new(x + w / 2.0, tabs_y, w / 2.0, m::BUTTON_H);
    let mut y = tabs_y + m::BUTTON_H + m::MENU_PAD - panel.scroll;
    let mut items = Vec::new();
    for (id, title) in archived(k, panel.lists) {
        let lines = crate::front::title_lines(ts, &title, w - 2.0 * m::CARD_PAD_X) as f32;
        let h = 2.0 * m::CARD_PAD_Y + lines * m::LINE_H;
        let item = Rect::new(x, y, w, h);
        let by = y + h + m::LABEL_GAP;
        let restore = Rect::new(x, by, w / 2.0 - m::LABEL_GAP, m::BUTTON_H);
        let delete = Rect::new(x + w / 2.0, by, w / 2.0, m::BUTTON_H);
        items.push(Item {
            id,
            title,
            rect: item,
            restore,
            delete,
        });
        y = by + m::BUTTON_H + m::MENU_PAD;
    }
    Placed {
        rect,
        close,
        cards,
        lists,
        items,
    }
}

pub(crate) fn hit(p: &Placed, x: f32, y: f32) -> Option<Row> {
    if !p.rect.contains(x, y) {
        return None;
    }
    if p.close.contains(x, y) {
        return Some(Row::Close);
    }
    if p.cards.contains(x, y) {
        return Some(Row::ShowCards);
    }
    if p.lists.contains(x, y) {
        return Some(Row::ShowLists);
    }
    let row = p.items.iter().find_map(|i| {
        if i.restore.contains(x, y) {
            Some(Row::Restore(i.id))
        } else if i.delete.contains(x, y) {
            Some(Row::Delete(i.id))
        } else {
            None
        }
    });
    Some(row.unwrap_or(Row::Inside))
}

pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    panel: Panel,
    p: &Placed,
    hover: Option<Row>,
) {
    crate::paint::shadow(
        scene,
        p.rect,
        0.0,
        c(theme, T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    fill_rounded_rect(scene, p.rect, 0.0, c(theme, T::KanbanList));
    let fg = c(theme, T::KanbanListFg);
    let meta = c(theme, T::KanbanListMeta);
    let ty = text_top(ts, p.close.y, p.close.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(
        ts,
        scene,
        tr("kanban.archive.title"),
        p.rect.x + m::MENU_PAD,
        ty,
        m::BUTTON_TEXT_PX,
        p.rect.w,
        fg,
    );
    if hover == Some(Row::Close) {
        fill_rounded_rect(scene, p.close, m::BUTTON_RADIUS, c(theme, T::KanbanSlot));
    }
    let ir = Rect::new(
        p.close.x + (p.close.w - m::ICON_PX) / 2.0,
        p.close.y + (p.close.h - m::ICON_PX) / 2.0,
        m::ICON_PX,
        m::ICON_PX,
    );
    paint_icon(scene, IconId::Close, ir, meta, StrokeToken::Default.px());
    for (r, on, key) in [
        (p.cards, !panel.lists, "kanban.archive.cards"),
        (p.lists, panel.lists, "kanban.archive.lists"),
    ] {
        let fill = if on {
            T::KanbanHeaderBtnOn
        } else {
            T::KanbanSlot
        };
        fill_rounded_rect(scene, r, m::BUTTON_RADIUS, c(theme, fill));
        let label = tr(key);
        let w = ts.prefix_width(label, m::BUTTON_TEXT_PX);
        let ty = text_top(ts, r.y, r.h, m::BUTTON_TEXT_PX);
        let ink = if on {
            c(theme, T::KanbanHeaderBtnOnFg)
        } else {
            fg
        };
        paint_text_elided(
            ts,
            scene,
            label,
            r.x + (r.w - w) / 2.0,
            ty,
            m::BUTTON_TEXT_PX,
            r.w,
            ink,
        );
    }
    let body = Rect::new(
        p.rect.x,
        p.cards.y + p.cards.h + m::MENU_PAD / 2.0,
        p.rect.w,
        p.rect.y + p.rect.h - p.cards.y - p.cards.h,
    );
    scene.push_clip(&ph2d_editor_core::paint::rect_to_vello(body));
    if p.items.is_empty() {
        let ty = text_top(ts, body.y, m::MENU_ROW_H, m::BUTTON_TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            tr("kanban.archive.empty"),
            body.x + m::MENU_PAD,
            ty,
            m::BUTTON_TEXT_PX,
            body.w,
            meta,
        );
    }
    for i in &p.items {
        fill_rounded_rect(scene, i.rect, m::CARD_RADIUS, c(theme, T::KanbanCard));
        let ty = text_top(ts, i.rect.y + m::CARD_PAD_Y, m::LINE_H, m::CARD_TITLE_PX);
        ph2d_editor_core::paint::paint_text_block(
            ts,
            scene,
            &i.title,
            i.rect.x + m::CARD_PAD_X,
            ty,
            m::CARD_TITLE_PX,
            i.rect.w - 2.0 * m::CARD_PAD_X,
            c(theme, T::KanbanCardFg),
        );
        for (r, row, key) in [
            (i.restore, Row::Restore(i.id), "kanban.archive.restore"),
            (i.delete, Row::Delete(i.id), "kanban.archive.delete"),
        ] {
            let fill = match (row, hover == Some(row)) {
                (Row::Delete(_), true) => T::DangerSoft,
                (_, true) => T::KanbanHeaderBtn,
                _ => T::KanbanSlot,
            };
            fill_rounded_rect(scene, r, m::BUTTON_RADIUS, c(theme, fill));
            let label = tr(key);
            let w = ts.prefix_width(label, m::BUTTON_TEXT_PX);
            let ty = text_top(ts, r.y, r.h, m::BUTTON_TEXT_PX);
            paint_text_elided(
                ts,
                scene,
                label,
                r.x + (r.w - w) / 2.0,
                ty,
                m::BUTTON_TEXT_PX,
                r.w,
                fg,
            );
        }
    }
    scene.pop_layer();
}
