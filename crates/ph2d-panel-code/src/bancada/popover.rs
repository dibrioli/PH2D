//! ⭐ **O balão «em quem»** (plano 13 Onda B, item 7; plano 14 Q4 C) — aberto pela marca da margem
//! (pairar abre, clicar prende): a faixa «**speed** · line 1 … default 220» (num sinal ouvido, «heard
//! by») e, sobre a tabela da casa sem cabeçalho, por objecto: o ponto e o nome, o valor e se é o do
//! script («default») ou outro («changed», a linha em âmbar). O botão «Select Hero 2» é do objecto
//! sob o rato (ou do primeiro que mudou).
//!
//! Smoke do dono 09/11: a moldura e o botão são os PADRÃO da casa (`paint_popover`, `paint_button`
//! elevado — os do balão do código, `hint.rs`), não a tinta do cartão (sumia no fundo), e o balão
//! encosta-se ao canto de baixo à direita da marca que o abriu (o caminho do rato é contíguo).

use super::{Aim, BancadaView, Who};
use crate::chrome;
use crate::peso::paint_text_title;
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_a11y::NodeId;
use ph2d_code_viewmodel::Holder;
use ph2d_editor_core::motion::SETTLED;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::{
    Button, ButtonState, Popover, paint_button, paint_popover, pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme};
use ph2d_ui_table::{Cell, Column, Ink, Layout, Metrics, Row, StableWidths};
use ph2d_vector::VectorScene;

/// A tabela do balão (sem cabeçalho): quem (o ponto e o nome), o valor (à direita, mono) e o estado
/// («default»/«changed»); num sinal ouvido o valor fica vazio e o estado diz a linha que corre.
fn table(who: &Who, column: Option<usize>, line: usize) -> (Vec<Column>, Vec<Row>) {
    let cols = vec![
        Column::new("who", String::new()),
        Column::new("value", String::new()).end().mono(),
        Column::new("state", String::new()),
    ];
    let row = |(h, holder): (usize, &Holder)| {
        let name = Cell {
            dot: Some(ColorToken::layer(holder.color)),
            ..Cell::new(holder.name.clone(), Ink::Plain)
        };
        let cell = column.and_then(|c| who.cell(h, c));
        let changed = cell.is_some_and(|x| x.changed);
        let state = match (column, changed) {
            (None, _) => tr_with("code.bancada.pop.hears", &[("line", &(line + 1))]),
            (Some(_), true) => tr("code.bancada.pop.changed").to_owned(),
            (Some(_), false) => tr("code.bancada.pop.default").to_owned(),
        };
        let value = cell.map(|x| x.text.clone()).unwrap_or_default();
        Row {
            key: h.to_string(),
            cells: vec![
                name,
                Cell::new(value, if changed { Ink::Changed } else { Ink::Number }),
                Cell::new(state, Ink::Muted),
            ],
            changed,
            ..Row::default()
        }
    };
    (cols, who.holders.iter().enumerate().map(row).collect())
}

/// A chave da linha de um alvo do balão (o nome de um objecto).
fn holder_key(aim: Option<Aim>) -> Option<String> {
    match aim {
        Some(Aim::Holder(h)) => Some(h.to_string()),
        _ => None,
    }
}

impl BancadaView {
    /// Pinta o balão aberto, por cima do código, encostado por baixo da marca e dentro de `bounds`.
    /// Os alvos (uma linha por objecto, o «Select», o balão inteiro) saem da MESMA geometria; os
    /// valores na mono de `view` (a do código).
    #[allow(clippy::too_many_lines)]
    pub fn paint_popover(
        &mut self,
        who: &Who,
        bounds: Rect,
        view: &mut CodeView,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.pop_hits.clear();
        self.pop_rect = None;
        self.pop_painted = None;
        let Some(pop) = self.pop else {
            self.pop_steady = None;
            return;
        };
        let Some(mark) = self
            .marks
            .iter()
            .find(|(l, r)| *l == pop.line && *r == pop.anchor)
            .or_else(|| self.marks.iter().find(|(l, _)| *l == pop.line))
            .map(|(_, r)| *r)
        else {
            return;
        };
        let ink = |t: ColorToken| resolve(t, *theme);
        let column = who.columns.iter().position(|c| c.line == pop.line);
        let heard = who.hears.iter().find(|e| e.1 == pop.line);
        let (title, hint) = match (column, heard) {
            (Some(c), _) => {
                let default = &who.columns[c].default;
                let hint = tr_with("code.bancada.pop.default_is", &[("value", default)]);
                (who.columns[c].name.clone(), hint)
            }
            (None, Some(e)) => (e.0.clone(), tr("code.bancada.pop.heard_by").to_owned()),
            (None, None) => return,
        };
        let line = tr_with("code.bancada.pop.line", &[("line", &(pop.line + 1))]);
        let (cols, rows) = table(who, column, pop.line);
        let hot = match self.aim {
            Aim::Holder(h) | Aim::Select(h) if h < who.holders.len() => Some(h),
            _ => column.and_then(|c| {
                (0..who.holders.len()).find(|&h| who.cell(h, c).is_some_and(|x| x.changed))
            }),
        }
        .or((!who.holders.is_empty()).then_some(0));
        let s = Spacing::Sm.px();
        let m = Metrics {
            header_h: 0.0,
            ..crate::peso::metricas()
        };
        let mut text = CodeCells {
            view,
            theme: *theme,
        };
        let natural = ph2d_ui_table::natural_widths(&cols, &rows, &m, ts, &mut text);
        // As larguras nascem ao abrir e só crescem enquanto o balão vive na MESMA marca.
        let steady = match &mut self.pop_steady {
            Some((l, st)) if *l == pop.line => st,
            slot => &mut slot.insert((pop.line, StableWidths::default())).1,
        };
        let natural = steady.widths(&cols, &natural);
        let f = chrome::font();
        let gap = s * 0.5;
        let title_w = crate::peso::largura_em(ts, &title, f, crate::peso::TITULO) + gap;
        let (line_w, hint_w) = (chrome::label_w(ts, &line), chrome::label_w(ts, &hint));
        let strip_w = title_w + line_w + s * 2.0 + hint_w;
        let select =
            hot.map(|h| tr_with("code.bancada.pop.select", &[("name", &who.holders[h].name)]));
        let button_w = select.as_deref().map_or(0.0, |t| {
            let f = Button::label_font_px();
            ts.prefix_width_weighted(t, f, FontWeight::MEDIUM) + Spacing::Lg.px() * 2.0
        });
        let w = natural.iter().sum::<f32>().max(strip_w).max(button_w) + s * 2.0;
        let lines = 1 + rows.len().max(1);
        let button_h = if select.is_some() { ROW_H_PX + s } else { 0.0 };
        let h = ROW_H_PX * lines as f32 + button_h + s * 2.0;
        // No canto de baixo à direita da marca que o abriu, encostado: o rato vai dela até ele sem
        // sair dos dois, e a coluna das marcas das linhas de baixo fica à vista.
        let x = (mark.x + mark.w).min(bounds.x + bounds.w - w).max(bounds.x);
        let below = mark.y + mark.h;
        let y = if below + h > bounds.y + bounds.h {
            (mark.y - h).max(bounds.y)
        } else {
            below
        };
        let r = Rect::new(x, y, w, h);
        self.pop_rect = Some(r);
        let (inner_x, inner_w) = (r.x + s, r.w - s * 2.0);
        let body_y = r.y + s + ROW_H_PX;
        let table_r = Rect::new(inner_x, body_y, inner_w, ROW_H_PX * rows.len() as f32);
        // A linha do objecto sob o rato (o nome, ou o «Select» dele) acende-se.
        self.pop_table.hover = match self.aim {
            Aim::Holder(h) | Aim::Select(h) => Some(h.to_string()),
            _ => None,
        };
        let lay = ph2d_ui_table::layout(&cols, &rows, &natural, &mut self.pop_table, table_r, &m);
        let (hover, pressed) = (holder_key(Some(self.aim)), holder_key(self.pressed));
        ph2d_editor_core::panel::camada_das_listas::por_cima(scene, |scene| {
            paint_popover(&Popover::new(NodeId(0)), r, scene, *theme);
            push_popover_clip(r, scene);
            let strip = r.y + s;
            let title_ink = ink(ColorToken::Text1);
            let ty = strip + ph2d_editor_core::paint::folga_do_texto(ROW_H_PX, f);
            paint_text_title(ts, scene, &title, inner_x, ty, f, title_w, title_ink);
            let line_r = Rect::new(inner_x + title_w, strip, line_w, ROW_H_PX);
            chrome::label_in(scene, ts, &line, line_r, ink(ColorToken::Text3));
            let hint_r = Rect::new(inner_x + inner_w - hint_w, strip, hint_w, ROW_H_PX);
            chrome::label_in(scene, ts, &hint, hint_r, ink(ColorToken::Text3));
            if rows.is_empty() {
                let none_r = Rect::new(inner_x, body_y, inner_w, ROW_H_PX);
                let none = tr("code.bancada.pop.none");
                chrome::label_in(scene, ts, none, none_r, ink(ColorToken::Text3));
            }
            let tones = (ColorToken::BgElev, ColorToken::Bg3);
            let st = &self.pop_table;
            ph2d_ui_table::paint(&lay, &cols, &rows, st, tones, scene, ts, &mut text, *theme);
            let (hk, pk) = (hover.as_deref(), pressed.as_deref());
            ph2d_ui_table::paint_row_frames(&lay, &rows, hk, pk, scene, *theme);
            for span in &lay.rows {
                let row = Rect::new(r.x, span.rect.y, r.w, span.rect.h);
                self.pop_hits.push((Aim::Holder(span.index), row));
            }
            let by = body_y + ROW_H_PX * rows.len().max(1) as f32;
            if let (Some(hh), Some(label)) = (hot, select.as_deref()) {
                let b = Rect::new(inner_x, by + s * 0.5, button_w, ROW_H_PX);
                let state = if self.aim == Aim::Select(hh) {
                    ButtonState::Hovered
                } else {
                    ButtonState::Normal
                };
                let button = Button::new(NodeId(0), label)
                    .on_elevated()
                    .visual((state, SETTLED));
                paint_button(&button, b, scene, ts, *theme);
                self.pop_hits.push((Aim::Select(hh), b));
            }
            pop_popover_clip(scene);
        });
        // O balão inteiro é do balão: o rato dentro dele não o fecha nem cai no código por baixo.
        self.pop_hits.push((Aim::Mark(pop.line), r));
        self.pop_painted = Some((lay, rows));
    }

    /// A tabela do balão no último quadro: a geometria e as linhas (`None` = fechado).
    #[must_use]
    pub fn popover_table(&self) -> Option<(&Layout, &[Row])> {
        let (lay, rows) = self.pop_painted.as_ref()?;
        Some((lay, rows.as_slice()))
    }
}
