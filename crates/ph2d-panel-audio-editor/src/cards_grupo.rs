//! Os grupos de botões num cartão: peças soltas, nas colunas do cartão, com o vão da grelha (`05 §4`). Fora de um
//! cartão, a lei do grupo de sempre (`block_cells`: as peças encostam).

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::widget::card_ink::card_body;
use ph2d_editor_core::widget::{self, GroupCell, GroupPos};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

const SOLTA: GroupCell = GroupCell {
    col: GroupPos::Only,
    row: GroupPos::Only,
};

/// O vão entre duas linhas: o da grelha num cartão, `fora` no painel de antes.
#[must_use]
pub(crate) fn vao(fora: f32) -> f32 {
    if card_body().is_some() {
        m::FIELD_ROW_GAP
    } else {
        fora
    }
}

/// Uma linha de texto curta sobre um controlo, depois de um vão de `vao`: `(topo, corpo, y seguinte)`, com os
/// vãos de um rótulo num cartão. Fora, o `Xs` e o vão de sempre.
#[must_use]
pub(crate) fn linha_de_texto(y: f32) -> (f32, f32, f32) {
    if card_body().is_some() {
        let topo = y + m::LABEL_TOP - m::FIELD_ROW_GAP;
        return (topo, m::LABEL_PX, topo + m::LABEL_PX + m::LABEL_BOTTOM);
    }
    let px = ph2d_tokens::TypeToken::Xs.px();
    (y, px, y + px + ph2d_tokens::control_gap_px())
}

/// O que separa duas peças da mesma fileira.
#[must_use]
pub(crate) fn fio() -> f32 {
    if card_body().is_some() {
        m::FIELD_GAP
    } else {
        widget::SEGMENT_HAIRLINE
    }
}

/// Uma fileira de peças de larguras dadas, soltas.
fn fileira(x: f32, y: f32, h: f32, larguras: &[f32]) -> Vec<(Rect, GroupCell)> {
    let mut cx = x;
    larguras
        .iter()
        .map(|&w| {
            let r = Rect::new(cx, y, w, h);
            cx += w + m::FIELD_GAP;
            (r, SOLTA)
        })
        .collect()
}

/// O `block_cells` do painel.
#[must_use]
pub(crate) fn block_cells(
    origin: Rect,
    cols_per_row: &[usize],
    row_h: f32,
) -> Vec<Vec<(Rect, GroupCell)>> {
    if card_body().is_none() {
        return widget::block_cells(origin, cols_per_row, row_h);
    }
    cols_per_row
        .iter()
        .enumerate()
        .map(|(r, &n)| {
            let cada = (origin.w - m::FIELD_GAP * n.saturating_sub(1) as f32) / n.max(1) as f32;
            let y = origin.y + r as f32 * (row_h + m::FIELD_ROW_GAP);
            fileira(origin.x, y, row_h, &vec![cada; n])
        })
        .collect()
}

/// O `block_cells_of` do painel.
#[must_use]
pub(crate) fn block_cells_of(
    origin: Rect,
    linhas: &[&[f32]],
    row_h: f32,
) -> Vec<Vec<(Rect, GroupCell)>> {
    if card_body().is_none() {
        return widget::block_cells_of(origin, linhas, row_h);
    }
    linhas
        .iter()
        .enumerate()
        .map(|(r, larguras)| {
            let y = origin.y + r as f32 * (row_h + m::FIELD_ROW_GAP);
            fileira(origin.x, y, row_h, larguras)
        })
        .collect()
}

/// A altura de `rows` linhas de um grupo.
#[must_use]
pub(crate) fn grid_height(rows: usize, row_h: f32) -> f32 {
    if card_body().is_none() {
        return widget::grid_height(rows, row_h);
    }
    if rows == 0 {
        return 0.0;
    }
    rows as f32 * row_h + (rows - 1) as f32 * m::FIELD_ROW_GAP
}

/// A largura da peça do meio de `◀ nome ▶`.
#[must_use]
pub(crate) fn stepper_middle_w(total: f32, lado: f32) -> f32 {
    if card_body().is_none() {
        return widget::stepper_middle_w(total, lado);
    }
    (total - 2.0 * (lado + m::FIELD_GAP)).max(1.0)
}

/// A fileira que mede as palavras: colunas iguais se cabem.
#[must_use]
pub(crate) fn segment_rects_for(
    row: Rect,
    rotulos: &[&str],
    font: f32,
    text_system: &mut TextSystem,
) -> Vec<(Rect, GroupCell)> {
    if card_body().is_none() {
        return widget::segment_rects_for(row, rotulos, font, text_system);
    }
    let n = rotulos.len();
    let cada = (row.w - m::FIELD_GAP * n.saturating_sub(1) as f32) / n.max(1) as f32;
    fileira(row.x, row.y, row.h, &vec![cada; n])
}
