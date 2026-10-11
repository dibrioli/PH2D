//! Onde fica cada controlo das barras do quadro: a barra curta, a grelha das formas e a barra de
//! estilo por cima da selecção (módulo filho de [`super`]).

use super::{Item, Selected, TOOLBAR, style_groups};
use crate::zones::Rect;
use ph2d_board_model::ShapeType;

// ⭐ A GEOMETRIA e o BOTÃO são do widget genérico ([`super::super::float_bar`]) — o Vetor pinta a
// coluna e a barra da seleção dele pela MESMA conta. Aqui fica só a tabela do Quadro.
pub(super) use super::super::float_bar::{button_px as btn_px, gap};

/// O rectângulo de cada atalho da barra curta, de cima para baixo, encostado à esquerda de `area`.
#[must_use]
pub fn toolbar_rects(area: Rect) -> Vec<(Item, Rect)> {
    TOOLBAR
        .iter()
        .copied()
        .zip(super::super::float_bar::column_rects(
            area,
            &[TOOLBAR.len()],
        ))
        .collect()
}

/// A grelha de todas as formas, à direita do botão «mais formas» (três colunas).
#[must_use]
pub fn shapes_rects(area: Rect) -> Vec<(Item, Rect)> {
    let more = toolbar_rects(area)
        .into_iter()
        .find(|(it, _)| *it == Item::MoreShapes)
        .map(|(_, r)| r)
        .expect("a barra tem o «mais formas»");
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
#[must_use]
pub fn style_rects_of(area: Rect, sel: Rect, groups: Vec<Vec<Item>>) -> Vec<(Item, Rect)> {
    if groups.is_empty() {
        return Vec::new();
    }
    let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
    groups
        .into_iter()
        .flatten()
        .zip(super::super::float_bar::strip_rects(
            area,
            sel,
            &sizes,
            super::super::float_bar::strip_button_px(),
        ))
        .collect()
}
