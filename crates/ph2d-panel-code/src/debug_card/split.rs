//! O separador da Sheet entre a tabela e a coluna do lado (Call stack, Waiting here): a lei mora em
//! [`crate::splitter`]. Só a Sheet tem esta divisão: os Cards são três cartões e a Strip uma fila.

use super::DebugCardView;
use ph2d_editor_core::math::safe_clamp;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

pub use crate::splitter::{Split, band};

/// Pinta o separador na faixa que começa em `x` (de `y`, altura `h`) e guarda onde agarra.
pub(super) fn paint(
    st: &mut DebugCardView,
    mut s: Split,
    at: (f32, f32, f32),
    scene: &mut VectorScene,
    theme: Theme,
) {
    // O «por cima» vale um quadro: o `look` volta a escrevê-lo enquanto o rato lá estiver.
    let hot = st.splitting.is_some() || st.split_hover.replace(false);
    s.hit = crate::splitter::paint_band(hot, at, scene, theme);
    st.painted.split = Some(s);
}

impl DebugCardView {
    /// Carregar no separador começa o arrasto. `true` = tomado.
    pub(super) fn split_press(&mut self, x: f32, y: f32) -> bool {
        let Some(s) = self.painted.split.filter(|s| s.hit.contains(x, y)) else {
            return false;
        };
        self.splitting = Some((x, s.side_w));
        true
    }

    /// O arrasto do separador: para a esquerda a coluna do lado cresce. `true` = havia arrasto.
    pub(super) fn split_drag_to(&mut self, x: f32) -> bool {
        let Some((x0, w0)) = self.splitting else {
            return false;
        };
        let s = self.painted.split.unwrap_or_default();
        self.side_w = Some(safe_clamp(w0 - (x - x0), s.min, s.max));
        true
    }

    /// O rato está sobre o separador, ou a arrastá-lo? (Lembra-o para a pintura o acender.)
    pub(super) fn split_look(&self, x: f32, y: f32) -> bool {
        let over = self.painted.split.is_some_and(|s| s.hit.contains(x, y));
        self.split_hover.set(over);
        over || self.splitting.is_some()
    }
}
