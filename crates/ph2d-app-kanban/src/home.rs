//! ⭐ **O início «Kanbans»** — a página dos Kanbans do projecto (a «Boards» do Trello): um mosaico
//! por Kanban e «Create new Kanban». Clicar abre-o numa aba.

use crate::metrics as m;
use crate::paint::{c, text_top};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::KanbanId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tile {
    Open(KanbanId),
    Create,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Home {
    pub hover: Option<Tile>,
    pub pressed: Option<Tile>,
    pub tiles: Vec<(Tile, Rect)>,
}

#[allow(clippy::cast_precision_loss)]
fn place(area: Rect, n: usize) -> Vec<Rect> {
    let x0 = area.x + m::HOME_PAD;
    let y0 = area.y + m::HOME_PAD + m::HEADER_H;
    let cols = (((area.w - 2.0 * m::HOME_PAD + m::TILE_GAP) / (m::TILE_W + m::TILE_GAP)).floor() as usize).max(1);
    (0..n)
        .map(|i| {
            let (col, row) = (i % cols, i / cols);
            Rect::new(
                x0 + col as f32 * (m::TILE_W + m::TILE_GAP),
                y0 + row as f32 * (m::TILE_H + m::TILE_GAP),
                m::TILE_W,
                m::TILE_H,
            )
        })
        .collect()
}

pub(crate) fn paint(home: &mut Home, names: &[(KanbanId, String)], area: Rect, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
    scene.fill_rect(rect_to_vello(area), c(theme, T::Bg1));
    let ty = text_top(ts, area.y + m::HOME_PAD, m::HEADER_H, m::HEADER_TITLE_PX);
    paint_text_title_elided(ts, scene, tr("kanban.home.title"), area.x + m::HOME_PAD, ty, m::HEADER_TITLE_PX, area.w, c(theme, T::Text1));
    let rects = place(area, names.len() + 1);
    home.tiles.clear();
    for (i, r) in rects.into_iter().enumerate() {
        let tile = names.get(i).map_or(Tile::Create, |(id, _)| Tile::Open(*id));
        let on = home.hover == Some(tile);
        match tile {
            Tile::Open(_) => {
                crate::paint::shadow(scene, r, m::CARD_RADIUS, c(theme, T::KanbanShadowNear), m::SHADOW_NEAR);
                fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, T::KanbanBoardBg));
                if on {
                    fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, T::KanbanSlot));
                }
                let ty = text_top(ts, r.y + m::CARD_PAD_Y, m::LINE_H, m::BUTTON_TEXT_PX);
                paint_text_title_elided(ts, scene, &names[i].1, r.x + m::CARD_PAD_X, ty, m::BUTTON_TEXT_PX, r.w - 2.0 * m::CARD_PAD_X, c(theme, T::KanbanHeaderFg));
            }
            Tile::Create => {
                let fill = if on { T::Bg3 } else { T::Bg2 };
                fill_rounded_rect(scene, r, m::CARD_RADIUS, c(theme, fill));
                let label = tr("kanban.home.create");
                let w = ts.prefix_width(label, m::BUTTON_TEXT_PX);
                let ty = text_top(ts, r.y, r.h, m::BUTTON_TEXT_PX);
                paint_text_elided(ts, scene, label, r.x + ((r.w - w) / 2.0).max(m::CARD_PAD_X), ty, m::BUTTON_TEXT_PX, r.w, c(theme, T::Text2));
            }
        }
        home.tiles.push((tile, r));
    }
}

pub(crate) fn hit(home: &Home, x: f32, y: f32) -> Option<Tile> {
    home.tiles.iter().find(|(_, r)| r.contains(x, y)).map(|(t, _)| *t)
}
