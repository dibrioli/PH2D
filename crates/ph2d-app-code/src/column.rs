//! ⭐ O divisor da coluna da esquerda (dono 10/11): o vão entre a coluna dos Scripts + «In this
//! script» e a do documento arrasta-se de lado ([`Split::gap_after`], [`Splitter`]).
//!
//! Limites (medidos 10/11 no teste `divisor`, área 1000 × 700: mínimo 118,6 px, máximo 712,8 px,
//! documento 263,2 px): o **mínimo** é o que a lista ([`ScriptListView::fit_w`]) e o «In this
//! script» ([`BancadaView::outline_fit_w`]) pedem para caber, títulos incluídos, nunca acima da de
//! fábrica ([`ScriptListView::width`]); o **máximo** deixa ao documento os menus e a engrenagem da
//! barra ([`crate::tab_menu::BarPainted::min_w`]), nunca menos que `PANEL_MIN_W_PX`.

use crate::docs::CodeDocs;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::splitter::Split;
use ph2d_panel_code::{ScriptListView, cards};
use ph2d_tokens::{PANEL_MIN_W_PX, Theme};
use ph2d_vector::VectorScene;

#[cfg(doc)]
use {ph2d_panel_code::bancada::BancadaView, ph2d_panel_code::splitter::Splitter};

impl CodeDocs {
    /// A coluna da esquerda aberta em `a` (a área já sem a margem): a largura e os limites dela, e o
    /// vão até ao documento como alvo do divisor.
    pub(crate) fn left_split(&self, a: Rect) -> Split {
        let fit = self.list.view.fit_w().max(self.bench.outline_fit_w());
        let doc_min = self.bar.painted.min_w.max(PANEL_MIN_W_PX);
        let limits = (ScriptListView::width(true), fit, doc_min);
        Split::gap_after(a, cards::margin(), self.left.side_w, limits)
    }

    /// Guarda onde o divisor agarra neste quadro e acende-o (por cima ou a arrastar).
    pub(crate) fn paint_left_split(&mut self, scene: &mut VectorScene, theme: &Theme) {
        if !self.list.open {
            self.left.painted = None;
            return;
        }
        let s = self.left_split(cards::inset(self.area));
        self.left.paint_gap(s, scene, *theme);
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
