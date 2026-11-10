//! A geometria do painel — pura, a MESMA para a pintura e para o clique.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::zones::Rect;

use crate::screen::{Hit, Row};

/// A coluna do nome e cada caixa de tecla, em alturas de campo («Left Arrow» e «Press a key» cabem a `VALUE_PX`).
const NAME_W: f32 = m::FIELD_H * 4.0; // LITERAL-PX-OK: quatro alturas de campo
const SLOT_W: f32 = m::FIELD_H * 3.5; // LITERAL-PX-OK: três alturas e meia de campo
const KEY_H: f32 = m::FIELD_H - m::SEG_INSET;
const ROW_H: f32 = m::FIELD_H;

/// Onde fica cada coisa do painel, em pixels de ecrã.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub title: Rect,
    /// Por linha: a faixa da linha, a coluna do nome e as caixas de tecla.
    pub rows: Vec<RowLayout>,
    pub reset: Rect,
    pub hint: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RowLayout {
    pub band: Rect,
    pub name: Rect,
    pub slots: Vec<Rect>,
}

impl Layout {
    /// O painel centrado em `area`, uma linha por acção e a de «Reset to defaults» no fim.
    #[must_use]
    pub fn of(area: Rect, rows: &[Row]) -> Self {
        let cols = rows.iter().map(Row::slots).max().unwrap_or(crate::SLOTS);
        let inner_w = NAME_W + (SLOT_W + m::FIELD_GAP) * cols as f32;
        let w = (inner_w + m::CARD_PAD_X * 2.0).min(area.w);
        let h = (m::CARD_PAD_TOP
            + m::CARD_HEAD_H
            + ROW_H * (rows.len() + 1) as f32
            + m::FIELD_ROW_GAP
            + ROW_H
            + m::CARD_PAD_BOTTOM)
            .min(area.h);
        let panel = Rect::new(
            area.x + (area.w - w) * 0.5,
            area.y + (area.h - h) * 0.5,
            w,
            h,
        );
        let x0 = panel.x + m::CARD_PAD_X;
        let iw = panel.w - m::CARD_PAD_X * 2.0;
        let title = Rect::new(x0, panel.y + m::CARD_PAD_TOP, iw, m::CARD_HEAD_H);
        let mut y = title.y + title.h;
        let out_rows = rows
            .iter()
            .map(|r| {
                let band = Rect::new(x0, y, iw, ROW_H);
                y += ROW_H;
                let name = Rect::new(x0 + m::SEG_INSET, band.y, NAME_W - m::SEG_INSET, ROW_H);
                let ky = band.y + (ROW_H - KEY_H) * 0.5;
                let slots = (0..r.slots())
                    .map(|i| {
                        Rect::new(
                            x0 + NAME_W + (SLOT_W + m::FIELD_GAP) * i as f32,
                            ky,
                            SLOT_W,
                            KEY_H,
                        )
                    })
                    .collect();
                RowLayout { band, name, slots }
            })
            .collect();
        let reset = Rect::new(x0, y, iw, ROW_H);
        let hint = Rect::new(x0, y + ROW_H + m::FIELD_ROW_GAP, iw, ROW_H);
        Self {
            panel,
            title,
            rows: out_rows,
            reset,
            hint,
        }
    }

    /// O alvo sob `(x, y)`; `None` = fora do painel.
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<Hit> {
        if !self.panel.contains(x, y) {
            return None;
        }
        for (row, r) in self.rows.iter().enumerate() {
            if let Some(slot) = r.slots.iter().position(|s| s.contains(x, y)) {
                return Some(Hit::Slot { row, slot });
            }
        }
        if self.reset.contains(x, y) {
            return Some(Hit::Reset);
        }
        Some(Hit::Panel)
    }
}
