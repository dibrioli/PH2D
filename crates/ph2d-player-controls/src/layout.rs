//! A geometria do painel — pura, a MESMA para a pintura e para o clique.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::widget::scrollbar_thumb_rect;
use ph2d_editor_core::zones::Rect;

use crate::screen::{Hit, Row};

/// A coluna do nome e cada caixa de tecla, em alturas de campo («Left Arrow» e «Press a key» cabem a `VALUE_PX`).
const NAME_W: f32 = m::FIELD_H * 4.0; // LITERAL-PX-OK: quatro alturas de campo
const SLOT_W: f32 = m::FIELD_H * 3.5; // LITERAL-PX-OK: três alturas e meia de campo
/// O aviso da tecla repetida («Left Arrow is also Move right.» cabe a `LABEL_PX`); só existe com um aviso.
const NOTE_W: f32 = m::FIELD_H * 6.0; // LITERAL-PX-OK: seis alturas de campo
const KEY_H: f32 = m::FIELD_H - m::SEG_INSET;
const ROW_H: f32 = m::FIELD_H;
/// O indicador de rolagem, fino como nas listas de cartões: metade do recuo, no meio da margem direita.
const BAR_W: f32 = m::SEG_INSET * 0.5; // LITERAL-PX-OK: meio recuo

/// Onde fica cada coisa do painel, em pixels de ecrã.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub title: Rect,
    /// Por linha À VISTA: a faixa da linha, a coluna do nome e as caixas de tecla. A primeira é a `first`.
    pub rows: Vec<RowLayout>,
    /// O índice (nas linhas da tela) da primeira linha à vista.
    pub first: usize,
    /// Rola: `(trilha, polegar)` do indicador.
    pub bar: Option<(Rect, Rect)>,
    pub reset: Rect,
    pub hint: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RowLayout {
    pub band: Rect,
    pub name: Rect,
    pub slots: Vec<Rect>,
    /// Onde vai o aviso da tecla repetida (largura zero quando nenhuma linha avisa).
    pub note: Rect,
}

impl Layout {
    /// O painel centrado em `area`, do topo da lista, uma linha por acção e a de «Reset to defaults» no fim.
    #[must_use]
    pub fn of(area: Rect, rows: &[Row]) -> Self {
        Self::scrolled(area, rows, 0, 0)
    }

    /// O mesmo, rolado: começa em `top` e acerta-o para a linha `selected` ficar à vista (`selected == rows.len()`
    /// é «Reset to defaults», sempre à vista em baixo).
    #[must_use]
    pub fn scrolled(area: Rect, rows: &[Row], selected: usize, top: usize) -> Self {
        let cols = rows.iter().map(Row::slots).max().unwrap_or(crate::SLOTS);
        let note_w = if rows.iter().any(|r| r.also.is_some()) {
            NOTE_W + m::FIELD_GAP
        } else {
            0.0
        };
        let inner_w = NAME_W + (SLOT_W + m::FIELD_GAP) * cols as f32 + note_w;
        let w = (inner_w + m::CARD_PAD_X * 2.0).min(area.w);
        let fixed = m::CARD_PAD_TOP
            + m::CARD_HEAD_H
            + ROW_H
            + m::FIELD_ROW_GAP
            + ROW_H
            + m::CARD_PAD_BOTTOM;
        let fit = (((area.h - fixed) / ROW_H).floor().max(1.0)) as usize;
        let visible = rows.len().min(fit);
        let mut first = top;
        if selected < rows.len() {
            if selected < first {
                first = selected;
            } else if selected >= first + visible {
                first = selected + 1 - visible;
            }
        }
        let first = first.min(rows.len() - visible);
        let h = (fixed + ROW_H * visible as f32).min(area.h);
        let panel = Rect::new(
            area.x + (area.w - w) * 0.5,
            area.y + (area.h - h) * 0.5,
            w,
            h,
        );
        let x0 = panel.x + m::CARD_PAD_X;
        let iw = panel.w - m::CARD_PAD_X * 2.0;
        let title = Rect::new(x0, panel.y + m::CARD_PAD_TOP, iw, m::CARD_HEAD_H);
        let list_y = title.y + title.h;
        let mut y = list_y;
        let out_rows = rows[first..first + visible]
            .iter()
            .map(|r| {
                let band = Rect::new(x0, y, iw, ROW_H);
                y += ROW_H;
                RowLayout::of(band, r.slots(), cols, note_w)
            })
            .collect();
        let bar = (visible < rows.len()).then(|| {
            let track = Rect::new(
                panel.x + panel.w - (m::CARD_PAD_X + BAR_W) * 0.5,
                list_y,
                BAR_W,
                ROW_H * visible as f32,
            );
            let thumb = scrollbar_thumb_rect(
                track,
                ROW_H * first as f32,
                ROW_H * rows.len() as f32,
                track.h,
            );
            (track, thumb)
        });
        let reset = Rect::new(x0, y, iw, ROW_H);
        let hint = Rect::new(x0, y + ROW_H + m::FIELD_ROW_GAP, iw, ROW_H);
        Self {
            panel,
            title,
            rows: out_rows,
            first,
            bar,
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
        for (i, r) in self.rows.iter().enumerate() {
            if let Some(slot) = r.slots.iter().position(|s| s.contains(x, y)) {
                return Some(Hit::Slot {
                    row: self.first + i,
                    slot,
                });
            }
        }
        if self.reset.contains(x, y) {
            return Some(Hit::Reset);
        }
        Some(Hit::Panel)
    }
}

impl RowLayout {
    fn of(band: Rect, slots: usize, cols: usize, note_w: f32) -> Self {
        let x0 = band.x;
        let name = Rect::new(x0 + m::SEG_INSET, band.y, NAME_W - m::SEG_INSET, ROW_H);
        let ky = band.y + (ROW_H - KEY_H) * 0.5;
        let slots = (0..slots)
            .map(|i| {
                Rect::new(
                    x0 + NAME_W + (SLOT_W + m::FIELD_GAP) * i as f32,
                    ky,
                    SLOT_W,
                    KEY_H,
                )
            })
            .collect();
        let nx = x0 + NAME_W + (SLOT_W + m::FIELD_GAP) * cols as f32;
        let note = Rect::new(nx, band.y, (note_w - m::FIELD_GAP).max(0.0), ROW_H);
        Self {
            band,
            name,
            slots,
            note,
        }
    }
}
