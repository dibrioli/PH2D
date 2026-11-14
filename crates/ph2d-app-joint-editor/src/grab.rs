//! **O painel que se arruma** (dono 13/11): a divisória à esquerda muda a largura (da do Inspector a metade
//! da aba), a barra arrasta-se para rolar e cada cartão recolhe pelo título. Tudo guardado no doc enquanto a
//! aba vive.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::zones::Rect;

use crate::doc::{Grab, Hit, JointEditorDoc};

/// ⭐ **O painel mais estreito em que UMA coluna de campos ainda se lê** (dono 13/11, «pode ser mais
/// estreito»): o campo mínimo [`crate::widgets::FIELD_MIN_W`], a folga do cartão dos dois lados e a faixa
/// da barra (a coluna estreita é alta e rola). Abaixo do piso dos painéis (`PANEL_MIN_W_PX`): os pares
/// passam a uma coluna em vez de cortar o valor.
#[must_use]
pub fn panel_min_w() -> f32 {
    let bar = ph2d_editor_core::widget::scrollbar_track_rect(Rect::default());
    crate::widgets::FIELD_MIN_W + m::CARD_PAD_X * 2.0 - bar.x + m::LIST_BAR_GAP
}

impl JointEditorDoc {
    /// A largura mínima e a máxima do painel: [`panel_min_w`] e metade da aba.
    #[must_use]
    pub fn panel_bounds(&self) -> (f32, f32) {
        let min = panel_min_w();
        (min, (self.area.w * 0.5).max(min))
    }

    /// A largura do painel: a escolhida na divisória, presa aos limites; nasce com a do Inspector.
    #[must_use]
    pub fn panel_width(&self) -> f32 {
        let (lo, hi) = self.panel_bounds();
        let w = if self.panel_w <= 0.0 {
            ph2d_tokens::INSPECTOR_W_PX
        } else {
            self.panel_w
        };
        w.clamp(lo, hi)
    }

    /// Escolhe a largura do painel (presa aos limites na pintura).
    pub fn set_panel_width(&mut self, w: f32) {
        self.panel_w = w;
    }

    /// Os alvos pintados neste quadro, na ordem da pintura.
    pub fn hits(&self) -> impl Iterator<Item = (Rect, &Hit)> {
        self.hits.iter().map(|(r, h, _)| (*r, h))
    }

    /// Quanto o painel está rolado.
    #[must_use]
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// O cartão `key` está recolhido.
    #[must_use]
    pub fn is_folded(&self, key: &str) -> bool {
        self.folded.contains(&key)
    }

    pub(crate) fn toggle_fold(&mut self, key: &'static str) {
        if let Some(i) = self.folded.iter().position(|k| *k == key) {
            self.folded.remove(i);
        } else {
            self.folded.push(key);
        }
    }

    /// O Down na divisória ou na barra começa o arrasto. `true` = começou.
    pub(crate) fn grab_start(&mut self, hit: Option<&Hit>, x: f32, y: f32) -> bool {
        self.grab = match hit {
            Some(Hit::Split) => Some(Grab::Split {
                x0: x,
                w0: self.panel_width(),
            }),
            Some(Hit::Bar) => Some(Grab::Bar {
                y0: y,
                s0: self.scroll,
            }),
            _ => None,
        };
        self.grab.is_some()
    }

    /// O arrasto em curso segue o cursor. `true` = havia arrasto.
    pub(crate) fn grab_move(&mut self, x: f32, y: f32) -> bool {
        match self.grab {
            Some(Grab::Split { x0, w0 }) => {
                let (lo, hi) = self.panel_bounds();
                self.panel_w = (w0 + x0 - x).clamp(lo, hi);
            }
            Some(Grab::Bar { y0, s0 }) => {
                let s = s0 + (y - y0) * self.scroll_per_px;
                self.scroll = s.clamp(0.0, self.scroll_max);
            }
            None => return false,
        }
        true
    }
}
