//! **O que o card `Line` AUTORA no Sketchy** — os cinco controles do tipo (plano 38 W3).
//!
//! Separado do [`super::sketchy_deposit`] por assunto: aqui mora *o que o artista escolheu*, lá *o
//! que o depósito faz com a escolha*.
//!
//! ⚠️ **Os cinco escrevem nos MESMOS slots que o `set_line_kind`** ([`super::impasto_settings::RELIEF_SLOTS`]).
//! O tipo de linha é um assunto do TRAÇO, não da ferramenta de relevo em que a mão está: sem o
//! fan-out, escolher Sketchy no Deposit e afinar a densidade lá deixaria a Faca com o tipo armado e
//! a densidade do outro slot — o defeito medido que o `toggle_brush_impasto` já paga.

use crate::tool::PainterTool;
use ph2d_painter_brush::line_kind::{SKETCHY_REACH_MAX, SKETCHY_WIDTH_MAX_PX};

/// Escreve `f` no pincel vivo e nos slots que compartilham o assunto.
macro_rules! fan_out {
    ($self:ident, $field:ident, $v:expr) => {{
        let v = $v;
        $self.paint.brush.$field = v;
        for mode in super::impasto_settings::RELIEF_SLOTS {
            $self.paint.brush_by_mode[mode.slot()].$field = v;
        }
    }};
}

impl PainterTool {
    /// **Reach** a partir da pista `0..1` do slider — o alcance em DIÂMETROS de pincel.
    pub fn set_sketchy_reach_norm(&mut self, t: f32) {
        fan_out!(self, sketchy_reach, t.clamp(0.0, 1.0) * SKETCHY_REACH_MAX);
    }

    /// **Density** a partir da pista `0..1`.
    ///
    /// ⚠️ A pista inteira é o orçamento: o teto é `SKETCHY_DENSITY_MAX`, não `1.0`, porque densidade
    /// cheia deposita ~50× o arco do traço (W0.3). Um slider que fosse até 1 seria um controle cujo
    /// topo o produto não sustenta — ver o doc da constante.
    pub fn set_sketchy_density_norm(&mut self, t: f32) {
        let max = ph2d_painter_brush::line_kind::SKETCHY_DENSITY_MAX;
        fan_out!(self, sketchy_density, t.clamp(0.0, 1.0) * max);
    }

    /// **Line Width** a partir da pista `0..1`, em pixels.
    pub fn set_sketchy_width_norm(&mut self, t: f32) {
        fan_out!(
            self,
            sketchy_width_px,
            t.clamp(0.0, 1.0) * SKETCHY_WIDTH_MAX_PX
        );
    }

    /// **Opacity** de um fio (`0..=1`) — a pista É o valor.
    pub fn set_sketchy_opacity(&mut self, t: f32) {
        fan_out!(self, sketchy_opacity, t.clamp(0.0, 1.0));
    }

    /// **Magnetify** — alterna.
    pub fn toggle_sketchy_magnetify(&mut self) {
        fan_out!(self, sketchy_magnetify, !self.paint.brush.sketchy_magnetify);
    }
}
