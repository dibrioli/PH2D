//! ⭐ O divisor da coluna da esquerda (dono 10/11): o vão entre a coluna dos Scripts + «In this
//! script» e a do documento arrasta-se de lado e muda a largura da da esquerda. O mesmo separador da
//! grelha ([`ph2d_panel_code::splitter`]): ↔ no vão, a linha de acento por cima ou a arrastar, Esc
//! devolve a largura do carregar. A largura vive na vista (a sessão), como a do mapa.
//!
//! Limites (medidos 10/11 no teste `divisor`, área 1000 × 700, `TextSystem` sem fontes do sistema:
//! mínimo 118,6 px — o título «IN THIS SCRIPT» espaçado; era 95,6 sem os títulos —, máximo 712,8 px,
//! documento 263,2 px — a barra):
//! - **mínimo** = a largura em que a lista ([`ScriptListView::fit_w`]: título · botões, nome ·
//!   contagem · lixo) e o «In this script» ([`BancadaView::outline_fit_w`]: título, nome · `ln N`)
//!   cabem sem cortar, os títulos medidos como se pintam (`title_tracked_width`), nunca acima
//!   da de fábrica ([`ScriptListView::width`], `PANEL_MIN_W_PX` = 220): um nome mais longo que ela
//!   continua a elidir-se como antes, e o artista pode alargar para o ler.
//! - **máximo** = o que deixa à coluna do documento o seu mínimo: os menus e a engrenagem da barra
//!   ([`crate::tab_menu::BarPainted::min_w`]), nunca menos que o painel mais estreito
//!   (`PANEL_MIN_W_PX`). Na cena 11 a coluna nasce com 220 px e o arrasto real de 120 px dá 340.

use crate::docs::CodeDocs;
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::splitter::{Split, band, paint_band};
use ph2d_panel_code::{ScriptListView, cards};
use ph2d_tokens::{PANEL_MIN_W_PX, Theme};
use ph2d_vector::VectorScene;

#[cfg(doc)]
use ph2d_panel_code::bancada::BancadaView;

impl CodeDocs {
    /// A coluna da esquerda aberta em `a` (a área já sem a margem): a largura e os limites dela, e o
    /// vão até ao documento como alvo do divisor.
    pub(crate) fn left_split(&self, a: Rect) -> Split {
        let g = cards::margin();
        let default = ScriptListView::width(true);
        let min = self
            .list
            .view
            .fit_w()
            .max(self.bench.outline_fit_w())
            .min(default);
        let doc_min = self.bar.painted.min_w.max(PANEL_MIN_W_PX);
        let max = (a.w - g - doc_min).max(min);
        let w = safe_clamp(self.left.side_w.unwrap_or(default), min, max);
        Split {
            hit: Rect::new(a.x + w, a.y, g, a.h),
            side_w: w,
            min,
            max,
        }
    }

    /// Guarda onde o divisor agarra neste quadro e acende-o (por cima ou a arrastar).
    pub(crate) fn paint_left_split(&mut self, scene: &mut VectorScene, theme: &Theme) {
        if !self.list.open {
            self.left.painted = None;
            return;
        }
        let s = self.left_split(cards::inset(self.area));
        if self.left.dragging.is_some() || self.left.hover.replace(false) {
            let x = s.hit.x + (s.hit.w - band()) * 0.5;
            let _ = paint_band(true, (x, s.hit.y, s.hit.h), scene, *theme);
        }
        self.left.painted = Some(s);
    }

    /// A largura da coluna da esquerda em `a`: a do divisor aberta, a de um botão recolhida.
    pub(crate) fn left_w(&self, a: Rect) -> f32 {
        if self.list.open {
            self.left_split(a).side_w
        } else {
            ScriptListView::width(false)
        }
    }

    /// O largar acaba o arrasto do divisor (o duplo-clique no vão volta à largura de fábrica).
    /// `true` = havia um.
    pub(crate) fn left_release(&mut self) -> bool {
        self.left.release()
    }
}
