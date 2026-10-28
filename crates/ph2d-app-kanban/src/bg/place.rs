//! A gaveta disposta — numa função só, que o pintor e o clique leem.

use super::{Drawer, Row, Target};
use crate::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{Background, Hue, PatternKind};

/// Uma secção: o rótulo (chave i18n) e onde se escreve.
pub(crate) type Section = (&'static str, Rect);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placed {
    pub rect: Rect,
    pub title: Rect,
    pub close: Rect,
    /// A parte que rola (o recorte do conteúdo).
    pub body: Rect,
    pub sections: Vec<Section>,
    pub rows: Vec<(Row, Rect)>,
    /// O seletor da cor livre aberto: o quadrado e a faixa do matiz.
    pub sv: Option<Rect>,
    pub hue: Option<Rect>,
    pub max_scroll: f32,
}

/// `n` caixas de altura `h` em `cols` colunas a partir de `(x, *y)`; `*y` passa para baixo delas.
#[allow(clippy::cast_precision_loss)]
fn grid(x: f32, y: &mut f32, w: f32, cols: usize, n: usize, h: f32) -> Vec<Rect> {
    let g = m::SWATCH_GAP * 2.0;
    let cw = (w - (cols as f32 - 1.0) * g) / cols as f32;
    let out: Vec<Rect> = (0..n)
        .map(|i| {
            let (c, r) = (i % cols, i / cols);
            Rect::new(x + c as f32 * (cw + g), *y + r as f32 * (h + g), cw, h)
        })
        .collect();
    let rows = n.div_ceil(cols) as f32;
    *y += rows * (h + g);
    out
}

/// ⭐ A gaveta à direita de `board`, com o fundo `bg` (o gravado ou a pré-visualização).
#[allow(clippy::too_many_lines)]
pub(crate) fn place(d: &Drawer, bg: Background, board: Rect) -> Placed {
    let rect = Rect::new(board.x + board.w - m::PANEL_W, board.y, m::PANEL_W, board.h);
    let x = rect.x + m::MENU_PAD;
    let w = rect.w - 2.0 * m::MENU_PAD;
    let close = Rect::new(
        rect.x + rect.w - m::MENU_PAD - m::BUTTON_H,
        rect.y + m::MENU_PAD,
        m::BUTTON_H,
        m::BUTTON_H,
    );
    let title = Rect::new(x, close.y, close.x - x, m::BUTTON_H);
    let top = close.y + close.h + m::MENU_PAD / 2.0;
    let body = Rect::new(rect.x, top, rect.w, (rect.y + rect.h - top).max(0.0));
    let mut y = top - d.scroll;
    let mut sections = Vec::new();
    let mut rows = Vec::new();
    let (mut sv, mut hue) = (None, None);
    let section = |key: &'static str, y: &mut f32, sections: &mut Vec<Section>| {
        sections.push((key, Rect::new(x, *y, w, m::BG_SECTION_H)));
        *y += m::BG_SECTION_H;
    };
    let mut picker = |target: Target, y: &mut f32| {
        if d.custom != Some(target) {
            return;
        }
        *y += m::SWATCH_GAP;
        sv = Some(Rect::new(x, *y, w, m::BG_SV_H));
        *y += m::BG_SV_H + m::MENU_PAD / 2.0;
        hue = Some(Rect::new(x, *y, w, m::BG_HUE_H));
        *y += m::BG_HUE_H + m::MENU_PAD / 2.0;
    };
    // As fotografias: «Upload…» e a actual.
    section("kanban.bg.photos", &mut y, &mut sections);
    let photo = matches!(bg, Background::Image(_));
    let tiles = grid(x, &mut y, w, 2, 1 + usize::from(photo), m::BG_PHOTO_H);
    rows.push((Row::Upload, tiles[0]));
    if photo {
        rows.push((Row::Photo, tiles[1]));
    }
    // As cores: os dez matizes, os nove gradientes do Trello e a cor livre.
    section("kanban.bg.colors", &mut y, &mut sections);
    let n = ph2d_board_render::KANBAN_GRADIENTS.len();
    let boxes = grid(x, &mut y, w, 5, Hue::ALL.len() + n + 1, m::BG_THUMB_H);
    for (i, r) in boxes.into_iter().enumerate() {
        let row = if i < Hue::ALL.len() {
            Row::Color(Hue::ALL[i])
        } else if i < Hue::ALL.len() + n {
            #[allow(clippy::cast_possible_truncation)]
            Row::Gradient((i - Hue::ALL.len()) as u16)
        } else {
            Row::Custom(Target::Solid)
        };
        rows.push((row, r));
    }
    picker(Target::Solid, &mut y);
    // Os padrões e, com um escolhido, as duas cores dele.
    section("kanban.bg.patterns", &mut y, &mut sections);
    let boxes = grid(x, &mut y, w, 4, PatternKind::ALL.len(), m::BG_THUMB_H);
    for (k, r) in PatternKind::ALL.into_iter().zip(boxes) {
        rows.push((Row::Pattern(k), r));
    }
    if matches!(bg, Background::Pattern(_)) {
        for (target, key) in [
            (Target::Base, "kanban.bg.pattern_base"),
            (Target::Ink, "kanban.bg.pattern_ink"),
        ] {
            section(key, &mut y, &mut sections);
            let boxes = grid(x, &mut y, w, 6, Hue::ALL.len() + 1, m::BG_SWATCH_H);
            for (i, r) in boxes.into_iter().enumerate() {
                let row = Hue::ALL
                    .get(i)
                    .map_or(Row::Custom(target), |h| Row::Tint(target, *h));
                rows.push((row, r));
            }
            picker(target, &mut y);
        }
    }
    let bottom = y + d.scroll + m::MENU_PAD;
    Placed {
        rect,
        title,
        close,
        body,
        sections,
        rows,
        sv,
        hue,
        max_scroll: (bottom - (body.y + body.h)).max(0.0),
    }
}

/// O que está em `(x, y)`: `None` = fora da gaveta.
pub(crate) fn hit(p: &Placed, x: f32, y: f32) -> Option<Row> {
    if !p.rect.contains(x, y) {
        return None;
    }
    if p.close.contains(x, y) {
        return Some(Row::Close);
    }
    if !p.body.contains(x, y) {
        return Some(Row::Inside);
    }
    if p.sv.is_some_and(|r| r.contains(x, y)) {
        return Some(Row::Sv);
    }
    if p.hue.is_some_and(|r| r.contains(x, y)) {
        return Some(Row::HueStrip);
    }
    Some(
        p.rows
            .iter()
            .find(|(_, r)| r.contains(x, y))
            .map_or(Row::Inside, |(row, _)| *row),
    )
}
