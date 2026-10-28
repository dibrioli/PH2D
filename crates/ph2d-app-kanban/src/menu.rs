//! ⭐ **Os menus do quadro** — o «…» de uma lista (criar cartão, recolher, arquivar, a cor) e o «…»
//! do Kanban (o arquivo, recolher/abrir todas). Disposição numa função só: pintar e clicar leem-na.

use crate::metrics as m;
use crate::paint::{c, label, text_top};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{Hue, LabelColor, ListId, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Menu {
    List(ListId),
    Board,
    /// W4 — «Show on board» com o Kanban em `n` quadros: escolher um.
    Boards(usize),
}

/// Uma linha de um menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    AddCard,
    Collapse,
    ArchiveList,
    ArchiveCards,
    /// Uma cor de lista (`None` = tirar a cor).
    Color(Option<Hue>),
    Archived,
    CollapseAll,
    ExpandAll,
    /// W4 — ir ao Quadro onde o Kanban está, e pôr uma referência nova no Quadro activo.
    ShowOnBoard,
    PlaceOnBoard,
    /// Um dos quadros onde ele está (o índice em `Tab::placements`).
    Board(usize),
    /// W6 — os diálogos da lista e o envelhecimento do Kanban.
    CopyList,
    MoveList,
    MoveAll,
    Sort,
    Aging,
    /// J2 — o painel de automação (o Butler).
    Automation,
}

impl Row {
    fn key(self) -> &'static str {
        match self {
            Row::AddCard => "kanban.menu.add_card",
            Row::Collapse => "kanban.menu.collapse",
            Row::ArchiveList => "kanban.menu.archive_list",
            Row::ArchiveCards => "kanban.menu.archive_cards",
            Row::Color(None) => "kanban.menu.no_color",
            Row::Color(Some(_)) => "",
            Row::Archived => "kanban.menu.archived",
            Row::CollapseAll => "kanban.menu.collapse_all",
            Row::ExpandAll => "kanban.menu.expand_all",
            Row::ShowOnBoard => "kanban.menu.show_on_board",
            Row::PlaceOnBoard => "kanban.menu.place_on_board",
            Row::Board(_) => "",
            Row::CopyList => "kanban.w6.menu.copy_list",
            Row::MoveList => "kanban.w6.menu.move_list",
            Row::MoveAll => "kanban.w6.menu.move_all",
            Row::Sort => "kanban.w6.menu.sort",
            Row::Aging => "kanban.w6.menu.aging",
            Row::Automation => "kanban.auto.ui.menu",
        }
    }
}

/// O menu aberto, disposto.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placed {
    pub rect: Rect,
    pub title: Rect,
    pub rows: Vec<(Row, Rect)>,
    /// O título da secção das cores (só no menu de lista).
    pub colors: Option<Rect>,
}

const SWATCH_COLS: usize = 5;

/// Dispõe `menu` por baixo de `anchor`, dentro de `area`.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn place(menu: Menu, anchor: Rect, area: Rect) -> Placed {
    let x = (anchor.x)
        .min(area.x + area.w - m::MENU_W - m::BOARD_PAD)
        .max(area.x);
    let mut y = anchor.y + anchor.h + m::LABEL_GAP;
    let top = y;
    let inner_x = x + m::MENU_PAD;
    let inner_w = m::MENU_W - 2.0 * m::MENU_PAD;
    let title = Rect::new(x, y, m::MENU_W, m::MENU_ROW_H + m::MENU_PAD / 2.0);
    y += title.h;
    let mut rows = Vec::new();
    let row = |rows: &mut Vec<(Row, Rect)>, y: &mut f32, r: Row| {
        rows.push((r, Rect::new(x, *y, m::MENU_W, m::MENU_ROW_H)));
        *y += m::MENU_ROW_H;
    };
    let mut colors = None;
    match menu {
        Menu::List(_) => {
            for r in [
                Row::AddCard,
                Row::CopyList,
                Row::MoveList,
                Row::MoveAll,
                Row::Sort,
                Row::Collapse,
                Row::ArchiveList,
                Row::ArchiveCards,
            ] {
                row(&mut rows, &mut y, r);
            }
            y += m::MENU_PAD / 2.0;
            colors = Some(Rect::new(inner_x, y, inner_w, m::MENU_ROW_H));
            y += m::MENU_ROW_H;
            let sw = (inner_w - (SWATCH_COLS as f32 - 1.0) * m::SWATCH_GAP) / SWATCH_COLS as f32;
            for (i, h) in Hue::ALL.iter().enumerate() {
                let (col, rowi) = (i % SWATCH_COLS, i / SWATCH_COLS);
                let r = Rect::new(
                    inner_x + col as f32 * (sw + m::SWATCH_GAP),
                    y + rowi as f32 * (m::SWATCH_H + m::SWATCH_GAP),
                    sw,
                    m::SWATCH_H,
                );
                rows.push((Row::Color(Some(*h)), r));
            }
            y += 2.0 * (m::SWATCH_H + m::SWATCH_GAP);
            row(&mut rows, &mut y, Row::Color(None));
        }
        Menu::Board => {
            for r in [
                Row::Archived,
                Row::CollapseAll,
                Row::ExpandAll,
                Row::ShowOnBoard,
                Row::PlaceOnBoard,
                Row::Aging,
                Row::Automation,
            ] {
                row(&mut rows, &mut y, r);
            }
        }
        Menu::Boards(n) => {
            for i in 0..n {
                row(&mut rows, &mut y, Row::Board(i));
            }
        }
    }
    y += m::MENU_PAD / 2.0;
    Placed {
        rect: Rect::new(x, top, m::MENU_W, y - top),
        title,
        rows,
        colors,
    }
}

/// O que está em `(x, y)` no menu: `Some(None)` = dentro do menu, sem linha.
pub(crate) fn hit(p: &Placed, x: f32, y: f32) -> Option<Option<Row>> {
    if !p.rect.contains(x, y) {
        return None;
    }
    Some(
        p.rows
            .iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(row, _)| *row),
    )
}

/// Pinta o menu; `current` = a cor da lista agora (marcada), `hover` = a linha sob o rato, `boards` =
/// os nomes dos quadros onde o Kanban está (as linhas `Row::Board`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    menu: Menu,
    p: &Placed,
    current: Option<Hue>,
    hover: Option<Row>,
    boards: &[(u64, u64, String)],
) {
    crate::paint::shadow(
        scene,
        p.rect,
        m::MENU_RADIUS,
        c(theme, T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    fill_rounded_rect(scene, p.rect, m::MENU_RADIUS, c(theme, T::KanbanCard));
    let fg = c(theme, T::KanbanCardFg);
    let meta = c(theme, T::KanbanCardMeta);
    let title = tr(match menu {
        Menu::List(_) => "kanban.menu.list_title",
        Menu::Board => "kanban.menu.board_title",
        Menu::Boards(_) => "kanban.menu.boards_title",
    });
    let w = ts.prefix_width(title, m::BUTTON_TEXT_PX);
    let ty = text_top(ts, p.title.y, p.title.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(
        ts,
        scene,
        title,
        p.title.x + (p.title.w - w) / 2.0,
        ty,
        m::BUTTON_TEXT_PX,
        p.title.w,
        meta,
    );
    if let Some(r) = p.colors {
        let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
        paint_text_title_elided(
            ts,
            scene,
            tr("kanban.menu.color"),
            r.x,
            ty,
            m::SMALL_PX,
            r.w,
            meta,
        );
    }
    for (row, r) in &p.rows {
        match row {
            Row::Color(Some(h)) => {
                let (fill, _) = label(
                    theme,
                    LabelColor {
                        hue: *h,
                        shade: Shade::Subtle,
                    },
                );
                fill_rounded_rect(scene, *r, m::LABEL_RADIUS, fill);
                if current == Some(*h) || hover == Some(*row) {
                    stroke_rounded_rect(
                        scene,
                        *r,
                        m::LABEL_RADIUS,
                        m::FOCUS_RING,
                        c(theme, T::Accent),
                    );
                }
            }
            _ => {
                if hover == Some(*row) {
                    fill_rounded_rect(scene, *r, 0.0, c(theme, T::KanbanCardRaised));
                }
                let ty = text_top(ts, r.y, r.h, m::BUTTON_TEXT_PX);
                let text = match row {
                    Row::Board(i) => boards.get(*i).map_or("", |b| b.2.as_str()),
                    _ => tr(row.key()),
                };
                paint_text_elided(
                    ts,
                    scene,
                    text,
                    r.x + m::MENU_PAD,
                    ty,
                    m::BUTTON_TEXT_PX,
                    r.w - 2.0 * m::MENU_PAD,
                    fg,
                );
            }
        }
    }
}
