//! ⭐ **O balão «em quem»** (plano 13 Onda B, item 7) — aberto pela marca da margem (pairar abre,
//! clicar prende): a propriedade e, por objecto, o valor e se é o do script («default») ou outro
//! («changed»); num sinal ouvido, quem o ouve e a linha que corre. O botão «Select Hero 2» é do
//! objecto sob o rato (ou do primeiro que mudou).

use super::{Aim, BancadaView, Who};
use crate::chrome::{self, Ui};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text_title, resolve,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

impl BancadaView {
    /// Pinta o balão aberto, por cima do código, encostado à marca e dentro de `bounds`.
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
        let Some(&(_, mark)) = self.marks.iter().find(|(l, _)| *l == pop.line) else {
            return;
        };
        let ui = Ui::new(*theme);
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
        let rows: Vec<(usize, String)> = who
            .holders
            .iter()
            .enumerate()
            .map(|(h, _)| {
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
        let button_w = select.as_deref().map_or(0.0, |t| chrome::button_w(ts, t));
        let w = (dot * 2.0 + s + name_w + s * 2.0 + s + value_w).max(button_w + s * 2.0) + s * 2.0;
        let lines = 1 + rows.len().max(1) + usize::from(select.is_some());
        let h = ROW_H_PX * lines as f32 + s * 2.0;
        // Por baixo da linha, a partir do início do texto (maquete B, foto 2).
        let left = self.text_x.unwrap_or(mark.x + mark.w);
        let x = left.min(bounds.x + bounds.w - w).max(bounds.x);
        let below = mark.y + mark.h;
        let y = if below + h > bounds.y + bounds.h {
            (mark.y - h).max(bounds.y)
        } else {
            below
        };
        let r = Rect::new(x, y, w, h);
        self.pop_rect = Some(r);
        // A moldura do balão do código: o campo, com o fio da borda.
        let radius = frame_radius(*theme, Radius::Sm.px());
        let t = StrokeToken::Thin.px();
        ph2d_editor_core::panel::camada_das_listas::por_cima(scene, |scene| {
            fill_rounded_rect(scene, r, radius, ui.field_edge);
            fill_rounded_rect(
                scene,
                Rect::new(r.x + t, r.y + t, r.w - t * 2.0, r.h - t * 2.0),
                radius,
                ui.field,
            );
            let inner_x = r.x + s;
            let inner_w = r.w - s * 2.0;
            let f = chrome::font();
            let mut yy = r.y + s;
            paint_text_title(
                ts,
                scene,
                &title,
                inner_x,
                yy + (ROW_H_PX - f) * 0.5,
                f,
                name_w + s,
                ui.text1,
            );
            let hw = chrome::label_w(ts, hint);
            chrome::label_in(
                scene,
                ts,
                hint,
                Rect::new(inner_x + inner_w - hw, yy, hw, ROW_H_PX),
                ui.text3,
            );
            yy += ROW_H_PX;
            if rows.is_empty() {
                chrome::label_in(
                    scene,
                    ts,
                    tr("code.bancada.pop.none"),
                    Rect::new(inner_x, yy, inner_w, ROW_H_PX),
                    ui.text3,
                );
            }
            for (hh, text) in &rows {
                let row = Rect::new(r.x + t, yy, r.w - t * 2.0, ROW_H_PX);
                if hot == Some(*hh) {
                    fill_rounded_rect(scene, row, radius, resolve(ColorToken::AccentSoft, *theme));
                }
                let holder = &who.holders[*hh];
                fill_circle(
                    scene,
                    inner_x + dot,
                    yy + ROW_H_PX * 0.5,
                    dot,
                    resolve(ColorToken::layer(holder.color), *theme),
                );
                let nx = inner_x + dot * 2.0 + s;
                chrome::label_in(
                    scene,
                    ts,
                    &holder.name,
                    Rect::new(nx, yy, name_w, ROW_H_PX),
                    ui.text1,
                );
                let vw = chrome::label_w(ts, text);
                chrome::label_in(
                    scene,
                    ts,
                    text,
                    Rect::new(inner_x + inner_w - vw, yy, vw, ROW_H_PX),
                    ui.text2,
                );
                self.pop_hits.push((Aim::Holder(*hh), row));
                yy += ROW_H_PX;
            }
            if let (Some(hh), Some(label)) = (hot, select.as_deref()) {
                let b = Rect::new(
                    inner_x,
                    yy + Spacing::Xxs.px(),
                    button_w,
                    ROW_H_PX - Spacing::Xxs.px() * 2.0,
                );
                // Sempre em relevo: o fundo de botão é o do próprio balão (o campo).
                chrome::text_button(scene, ts, label, b, true, &ui);
                if self.aim == Aim::Select(hh) {
                    let w = StrokeToken::Thin.px();
                    ph2d_editor_core::paint::stroke_rounded_rect(scene, b, radius, w, ui.focus);
                }
                self.pop_hits.push((Aim::Select(hh), b));
            }
        });
        // O balão inteiro é do balão: o rato dentro dele não o fecha nem cai no código por baixo.
        self.pop_hits.push((Aim::Mark(pop.line), r));
    }
}
