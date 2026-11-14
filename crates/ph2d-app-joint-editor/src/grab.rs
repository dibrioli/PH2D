//! **O painel que se arruma** (dono 13/11): a divisória à esquerda muda a largura (da do Inspector a metade
//! da aba), a barra arrasta-se para rolar e cada cartão recolhe pelo título. Tudo guardado no doc enquanto a
//! aba vive.

use crate::doc::{Grab, Hit, JointEditorDoc};

impl JointEditorDoc {
    /// A largura mínima e a máxima do painel: o piso dos painéis do app (o Inspector estreita até ele,
    /// dono 13/11) e metade da aba.
    #[must_use]
    pub fn panel_bounds(&self) -> (f32, f32) {
        let min = ph2d_tokens::PANEL_MIN_W_PX;
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
