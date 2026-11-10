//! ⭐ **«In this script»** (plano 13 Onda B, item 6) — o cartão por baixo da lista de scripts:
//! Properties · Functions · Hears, cada entrada com `ln N` à direita; o clique salta para a linha
//! (o «Filtrar Métodos» do Godot, mais as propriedades e os sinais ouvidos).

use super::{Aim, BancadaView, Who};
use crate::chrome::{self, Ui};
use ph2d_code_viewmodel::Entry;
use ph2d_editor_core::paint::{fill_rounded_rect, frame_radius, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{LineHeight, ROW_H_PX, Radius, Theme};
use ph2d_vector::VectorScene;

/// O passo de uma ENTRADA (`speed`, `init`): a linha folgada do texto, sem a caixa de um controlo —
/// dono 10/11: «os espaços entre as propriedades e as funções podem ser menores». Os títulos das
/// secções ficam em [`ROW_H_PX`].
#[must_use]
pub fn outline_entry_h() -> f32 {
    ph2d_editor_core::ui_scale::ao_pixel(chrome::font_shown() * LineHeight::Loose.ratio())
}

/// A rolagem de uma lista da Bancada: a caixa das linhas e a altura do conteúdo.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Scroll {
    pub rows: Rect,
    pub content_h: f32,
    pub y: f32,
}

impl Scroll {
    pub fn clamp(&mut self) {
        let max = (self.content_h - self.rows.h).max(0.0);
        self.y = ph2d_editor_core::math::safe_clamp(self.y, 0.0, max);
    }

    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        if !self.rows.contains(x, y) || self.content_h <= self.rows.h {
            return false;
        }
        self.y -= dy;
        self.clamp();
        true
    }
}

/// O título de um cartão da Bancada no idioma da lista (MAIÚSCULAS espaçadas nos cartões).
pub(super) fn card_title(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    title: &str,
    r: Rect,
    theme: Theme,
) {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        use ph2d_editor_core::panel::cards::banda::metrics as m;
        ph2d_editor_core::paint::paint_text_title_elided_tracked(
            ts,
            scene,
            &title.to_uppercase(),
            [
                r.x,
                r.y + ph2d_editor_core::paint::folga_do_texto(r.h, m::LIST_TITLE_PX),
            ],
            m::LIST_TITLE_PX,
            r.w,
            ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::KanbanListFg, theme),
            m::LIST_TITLE_TRACKING_EM,
        );
    } else {
        chrome::label(scene, ts, title, r, &Ui::new(theme));
    }
}

impl BancadaView {
    /// Pinta o «In this script» em `area` (o cartão já aberto pela família).
    pub fn paint_outline(
        &mut self,
        who: &Who,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let ui = Ui::new(*theme);
        self.outline_hits.clear();
        scene.fill_rect(rect_to_vello(area), ui.bg);
        let g = ph2d_tokens::card_pad_px();
        let w = (area.w - g * 2.0).max(0.0);
        card_title(
            scene,
            ts,
            tr("code.bancada.outline.title"),
            Rect::new(area.x + g, area.y + g, w, ROW_H_PX),
            *theme,
        );
        let top = area.y + g + ROW_H_PX + g;
        let props: Vec<Entry> = who
            .columns
            .iter()
            .map(|c| Entry(c.name.clone(), c.line))
            .collect();
        let sections = [
            ("code.bancada.outline.properties", props.as_slice()),
            ("code.bancada.outline.functions", who.functions.as_slice()),
            ("code.bancada.outline.hears", who.hears.as_slice()),
        ];
        let shown = sections.iter().filter(|s| !s.1.is_empty()).count();
        let entries: usize = sections.iter().map(|s| s.1.len()).sum();
        self.outline.rows = Rect::new(area.x, top, area.w, (area.y + area.h - g - top).max(0.0));
        self.outline.content_h = if shown == 0 {
            ROW_H_PX
        } else {
            ROW_H_PX * shown as f32 + outline_entry_h() * entries as f32
        };
        self.outline.clamp();
        scene.push_clip(&rect_to_vello(self.outline.rows));
        if shown == 0 {
            chrome::label_in(
                scene,
                ts,
                tr("code.bancada.outline.empty"),
                Rect::new(area.x + g, top, w, ROW_H_PX),
                ui.text3,
            );
        }
        let mut y = top - self.outline.y;
        let radius = frame_radius(*theme, Radius::Sm.px());
        let eh = outline_entry_h();
        for (key, entries) in sections.iter().filter(|s| !s.1.is_empty()) {
            chrome::label_in(
                scene,
                ts,
                tr(key),
                Rect::new(area.x + g, y, w, ROW_H_PX),
                ui.text3,
            );
            y += ROW_H_PX;
            for Entry(name, line) in *entries {
                let r = Rect::new(area.x + g * 0.5, y, (area.w - g).max(0.0), eh);
                if self.aim == Aim::Entry(*line) {
                    fill_rounded_rect(scene, r, radius, ui.hover);
                }
                let ln = tr_with("code.bancada.outline.line", &[("line", &(line + 1))]);
                let lw = chrome::label_w(ts, &ln);
                let rx = r.x + r.w - g * 0.5 - lw;
                chrome::label_in(scene, ts, &ln, Rect::new(rx, y, lw, eh), ui.text3);
                let name_r = Rect::new(r.x + g, y, (rx - g - r.x - g).max(0.0), eh);
                chrome::label_in(scene, ts, name, name_r, ui.text1);
                if r.y + r.h > self.outline.rows.y
                    && r.y < self.outline.rows.y + self.outline.rows.h
                {
                    self.outline_hits.push((Aim::Entry(*line), r));
                }
                y += eh;
            }
        }
        scene.pop_layer();
    }
}
