//! As medidas da tabela ([`Metrics`], dos tokens) e o que se mede antes da geometria: a largura
//! natural de cada coluna, a largura mínima e a altura natural.

use crate::{CellText, Column, Row, TableState, swatch_side};
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// As medidas da tabela, dos tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub row_h: f32,
    pub header_h: f32,
    pub pad: f32,
    pub indent: f32,
    pub min_col: f32,
    /// A meia largura do puxador entre colunas.
    pub handle: f32,
    /// A largura natural de um traço.
    pub spark_w: f32,
}

impl Metrics {
    #[must_use]
    pub fn standard() -> Self {
        Self {
            row_h: ROW_H_PX,
            header_h: ROW_H_PX,
            pad: Spacing::Sm.px(),
            indent: Spacing::Lg.px(),
            min_col: ROW_H_PX * 2.0,
            handle: Spacing::Xs.px(),
            spark_w: ROW_H_PX * 4.0, // LITERAL-PX-OK: quatro alturas de linha (token)
        }
    }

    /// A caixa do ▸ (e a reserva dela na 1.ª coluna).
    #[must_use]
    pub fn chevron(&self) -> f32 {
        self.row_h * 0.5 + self.pad
    }

    /// A reserva do ▸ na 1.ª coluna de `rows`: só quando alguma linha o mostra (uma lista plana
    /// encosta o ponto e o texto ao recuo).
    #[must_use]
    pub fn twist(&self, rows: &[Row]) -> f32 {
        if rows.iter().any(|r| r.expandable) {
            self.chevron()
        } else {
            0.0
        }
    }
}

/// A largura mais estreita em que a tabela ainda se lê inteira: a `flex` no `min_col`, as outras à
/// vista na natural (como o [`crate::layout`] as deixa) e a calha da barra (o pior caso: o corpo
/// transborda).
#[must_use]
pub fn min_width(cols: &[Column], natural: &[f32], st: &TableState, m: &Metrics) -> f32 {
    let cells: f32 = cols
        .iter()
        .zip(natural)
        .filter(|(c, _)| !st.hidden.contains(&c.key))
        .map(|(c, w)| if c.flex { m.min_col } else { w.max(m.min_col) })
        .sum();
    cells + bar_gutter()
}

/// A largura natural de cada coluna (o título e a célula mais larga), medida por `text`.
pub fn natural_widths(
    cols: &[Column],
    rows: &[Row],
    m: &Metrics,
    ts: &mut TextSystem,
    text: &mut dyn CellText,
) -> Vec<f32> {
    let twist = m.twist(rows);
    cols.iter()
        .enumerate()
        .map(|(ci, c)| {
            let head = text.width(ts, &c.title, false);
            let cells = rows
                .iter()
                .filter(|r| r.group.is_none())
                .filter_map(|r| r.cells.get(ci).map(|cell| (r, cell)))
                .map(|(r, cell)| {
                    let mut w = if cell.swatch.is_some() {
                        swatch_side()
                    } else {
                        text.width(ts, &cell.text, c.mono)
                    };
                    if let Some(old) = &cell.old {
                        w += text.width(ts, old, c.mono) + m.pad;
                    }
                    if cell.dot.is_some() {
                        w += m.row_h * 0.5;
                    }
                    if cell.spark.is_some() {
                        w = w.max(m.spark_w);
                    }
                    if ci == 0 {
                        w += f32::from(r.depth) * m.indent + twist;
                    }
                    w
                })
                .fold(0.0f32, f32::max);
            let natural = if c.spark { m.spark_w } else { head.max(cells) };
            (natural + m.pad * 2.0).max(m.min_col)
        })
        .collect()
}

/// A calha da barra do corpo quando ele transborda: a trilha da casa e um recuo.
#[must_use]
pub fn bar_gutter() -> f32 {
    ph2d_editor_core::widget::SCROLLBAR_W + Spacing::Sm.px()
}

/// A altura que a tabela pede para mostrar `rows` sem rolar (o cabeçalho e as linhas à vista).
#[must_use]
pub fn natural_height(rows: &[Row], st: &TableState, m: &Metrics) -> f32 {
    m.header_h + st.visible_rows(rows).len() as f32 * m.row_h
}
