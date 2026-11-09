//! ⭐ **O balão «em quem»** (plano 13 Onda B, item 7) — aberto pela marca da margem (pairar abre,
//! clicar prende): a propriedade e, por objecto, o valor e se é o do script («default») ou outro
//! («changed»); num sinal ouvido, quem o ouve e a linha que corre. O botão «Select Hero 2» é do
//! objecto sob o rato (ou do primeiro que mudou).
//!
//! Smoke do dono 09/11: a moldura e o botão são os PADRÃO da casa (`paint_popover`, `paint_button`
//! elevado — os do balão do código, `hint.rs`), não a tinta do cartão (sumia no fundo), e o balão
//! encosta-se ao canto de baixo à direita da marca que o abriu (o caminho do rato é contíguo).

use super::{Aim, BancadaView, Who};
use crate::chrome;
use ph2d_a11y::NodeId;
use ph2d_editor_core::motion::SETTLED;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text_title, resolve,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, Popover, paint_button, paint_popover, pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

impl BancadaView {
    /// Pinta o balão aberto, por cima do código, encostado por baixo da marca e dentro de `bounds`.
    #[allow(clippy::too_many_lines)]
    pub fn paint_popover(
        &mut self,
        who: &Who,
        bounds: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.pop_hits.clear();
        self.pop_rect = None;
        let Some(pop) = self.pop else { return };
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
            (Some(c), _) => (
                who.columns[c].name.clone(),
                tr("code.bancada.pop.inspector"),
            ),
            (None, Some(e)) => (e.0.clone(), tr("code.bancada.pop.heard_by")),
            (None, None) => return,
        };
        let rows: Vec<(usize, String)> = (0..who.holders.len())
            .map(|h| {
                let text = match column.and_then(|c| who.cell(h, c)) {
                    Some(cell) => {
                        let key = if cell.changed {
                            "code.bancada.pop.changed"
                        } else {
                            "code.bancada.pop.default"
                        };
                        tr_with(key, &[("value", &cell.text)])
                    }
                    None => tr_with("code.bancada.pop.hears", &[("line", &(pop.line + 1))]),
                };
                (h, text)
            })
            .collect();
        let hot = match self.aim {
            Aim::Holder(h) | Aim::Select(h) if h < who.holders.len() => Some(h),
            _ => column.and_then(|c| {
                (0..who.holders.len()).find(|&h| who.cell(h, c).is_some_and(|x| x.changed))
            }),
        }
        .or((!who.holders.is_empty()).then_some(0));
        let s = Spacing::Sm.px();
        let dot = super::chip_r();
        let (title_w, hint_w) = (chrome::label_w(ts, &title), chrome::label_w(ts, hint));
        let name_w = who
            .holders
            .iter()
            .map(|h| chrome::label_w(ts, &h.name))
            .fold(title_w, f32::max);
        let value_w = rows
            .iter()
            .map(|r| chrome::label_w(ts, &r.1))
            .fold(hint_w, f32::max);
        let select =
            hot.map(|h| tr_with("code.bancada.pop.select", &[("name", &who.holders[h].name)]));
        let button_w = select.as_deref().map_or(0.0, |t| {
            let f = Button::label_font_px();
            ts.prefix_width_weighted(t, f, FontWeight::MEDIUM) + Spacing::Lg.px() * 2.0
        });
        let w = (dot * 2.0 + s + name_w + s * 2.0 + s + value_w).max(button_w) + s * 2.0;
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
        let radius = frame_radius(*theme, Radius::Sm.px());
        let f = chrome::font();
        ph2d_editor_core::panel::camada_das_listas::por_cima(scene, |scene| {
            paint_popover(&Popover::new(NodeId(0)), r, scene, *theme);
            push_popover_clip(r, scene);
            let inner_x = r.x + s;
            let inner_w = r.w - s * 2.0;
            let mut yy = r.y + s;
            let ty = |y: f32| y + (ROW_H_PX - f) * 0.5;
            let title_ink = ink(ColorToken::Text1);
            paint_text_title(ts, scene, &title, inner_x, ty(yy), f, name_w + s, title_ink);
            let hint_r = Rect::new(inner_x + inner_w - hint_w, yy, hint_w, ROW_H_PX);
            chrome::label_in(scene, ts, hint, hint_r, ink(ColorToken::Text3));
            yy += ROW_H_PX;
            if rows.is_empty() {
                let none_r = Rect::new(inner_x, yy, inner_w, ROW_H_PX);
                let none = tr("code.bancada.pop.none");
                chrome::label_in(scene, ts, none, none_r, ink(ColorToken::Text3));
            }
            let edge = Spacing::Xxs.px();
            for (hh, text) in &rows {
                let row = Rect::new(r.x + edge, yy, r.w - edge * 2.0, ROW_H_PX);
                if hot == Some(*hh) {
                    fill_rounded_rect(scene, row, radius, ink(ColorToken::AccentSoft));
                }
                let holder = &who.holders[*hh];
                let color = ink(ColorToken::layer(holder.color));
                fill_circle(scene, inner_x + dot, yy + ROW_H_PX * 0.5, dot, color);
                let nx = inner_x + dot * 2.0 + s;
                let name_r = Rect::new(nx, yy, name_w, ROW_H_PX);
                chrome::label_in(scene, ts, &holder.name, name_r, ink(ColorToken::Text1));
                let vw = chrome::label_w(ts, text);
                let value_r = Rect::new(inner_x + inner_w - vw, yy, vw, ROW_H_PX);
                chrome::label_in(scene, ts, text, value_r, ink(ColorToken::Text2));
                self.pop_hits.push((Aim::Holder(*hh), row));
                yy += ROW_H_PX;
            }
            if let (Some(hh), Some(label)) = (hot, select.as_deref()) {
                let b = Rect::new(inner_x, yy + s * 0.5, button_w, ROW_H_PX);
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
    }
}
