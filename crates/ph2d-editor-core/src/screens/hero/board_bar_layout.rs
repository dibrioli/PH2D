//! Onde fica cada controlo das barras do quadro: a barra curta, a grelha das formas e a barra de
//! estilo por cima da selecção (módulo filho de [`super`]).

use super::{Item, Selected, TOOLBAR, TOOLBAR_TAIL, style_groups};
use crate::zones::Rect;
use ph2d_board_model::ShapeType;
use ph2d_tokens::Spacing;

// ⭐ A GEOMETRIA e o BOTÃO são do widget genérico ([`super::super::float_bar`]) — o Vetor pinta a
// coluna e a barra da seleção dele pela MESMA conta. Aqui fica só a tabela do Quadro.
pub(super) use super::super::float_bar::{button_px as btn_px, gap};

/// O rectângulo de cada atalho da barra curta, de cima para baixo, encostado à esquerda de `area`.
#[must_use]
///
/// Numa janela baixa a coluna parte-se em colunas lado a lado, de alturas iguais (W23 A6): nenhum
/// botão sai da área. Com lugar, é a coluna do [`super::super::float_bar::column_rects`].
pub fn toolbar_rects(area: Rect) -> Vec<(Item, Rect)> {
    let n = TOOLBAR.len() + TOOLBAR_TAIL.len();
    let (s, g, m) = (btn_px(), gap(), Spacing::Md.px());
    let fits = (((area.h - 2.0 * m + g) / (s + g)).floor().max(1.0)) as usize;
    let rows = n.div_ceil(n.div_ceil(fits));
    TOOLBAR
        .iter()
        .chain(&TOOLBAR_TAIL)
        .copied()
        .enumerate()
        .map(|(i, it)| {
            let (c, r) = ((i / rows) as f32, (i % rows) as f32);
            (
                it,
                Rect::new(area.x + m + c * (s + g), area.y + m + r * (s + g), s, s),
            )
        })
        .collect()
}

/// O botão `item` da barra visto pelos painéis que abrem À DIREITA dele: na última coluna da barra
/// (um painel aberto de um botão da 1.ª coluna não tapa a 2.ª).
pub(super) fn anchor(area: Rect, item: Item) -> Option<Rect> {
    let rects = toolbar_rects(area);
    let right = rects.iter().map(|(_, r)| r.x).fold(f32::MIN, f32::max);
    rects
        .into_iter()
        .find(|(it, _)| *it == item)
        .map(|(_, r)| Rect::new(right, r.y, r.w, r.h))
}

/// A grelha de todas as formas, à direita do botão «mais formas» (três colunas).
#[must_use]
pub fn shapes_rects(area: Rect) -> Vec<(Item, Rect)> {
    let more = anchor(area, Item::MoreShapes).expect("a barra tem o «mais formas»");
    let grid = super::super::float_bar::grid_rects(area, more, ShapeType::ALL.len(), 3);
    ShapeType::ALL
        .iter()
        .map(|t| Item::Pick(*t))
        .zip(grid)
        .collect()
}

/// A barra de estilo por cima de `sel` (o rectângulo da selecção no ecrã), presa dentro de `area`;
/// por baixo da selecção quando não cabe por cima.
#[must_use]
pub fn style_rects(area: Rect, sel: Rect, what: Selected) -> Vec<(Item, Rect)> {
    style_rects_of(area, sel, style_groups(what))
}

/// O mesmo, para os grupos que a selecção de AGORA pinta (sem os controlos que não teriam efeito).
/// Fica à direita da barra curta e, sem lugar numa linha, parte-se em linhas (a da forma, com a
/// letra e o Organizar, passava da área a 1366 — foto da cena 28, 22/10).
#[must_use]
pub fn style_rects_of(area: Rect, sel: Rect, groups: Vec<Vec<Item>>) -> Vec<(Item, Rect)> {
    if groups.is_empty() {
        return Vec::new();
    }
    let left = toolbar_rects(area)
        .iter()
        .map(|(_, r)| r.x + r.w)
        .fold(area.x, f32::max)
        + Spacing::Md.px();
    let room = Rect::new(left, area.y, (area.x + area.w - left).max(0.0), area.h);
    let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
    groups
        .into_iter()
        .flatten()
        .zip(super::super::float_bar::strip_rects_fitting(
            room,
            sel,
            &sizes,
            super::super::float_bar::strip_button_px(),
            Spacing::Lg.px(),
        ))
        .collect()
}
