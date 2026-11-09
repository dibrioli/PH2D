//! ⭐ O separador da Sheet entre a tabela e a coluna do lado (smoke do dono 09/11, cena 14: «coloque
//! um separador vertical onde o cursor vai pegar»): uma faixa vertical à direita da calha da barra da
//! tabela; arrastá-la de lado muda a largura da coluna do lado (Call stack, Waiting here) e a tabela
//! fica com o resto, cada uma presa ao seu mínimo. Só a Sheet tem esta divisão: os Cards são três
//! cartões lado a lado e a Strip uma fila de células.

use super::DebugCardView;
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, PANEL_MIN_W_PX, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// A parte da largura que a coluna do lado toma por omissão (antes de o artista a arrastar).
const SIDE_SHARE: f32 = 0.4; // LITERAL-PX-OK: fração da largura do quadro, não píxeis

/// A faixa do separador: agarra em toda ela e a linha pinta-se ao meio (mais larga que a linha).
#[must_use]
pub fn band() -> f32 {
    ph2d_tokens::PANEL_RESIZE_HANDLE_SIZE_PX
}

/// O separador pintado: onde agarra, a largura da coluna do lado e os limites dela.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Split {
    pub hit: Rect,
    pub side_w: f32,
    pub min: f32,
    pub max: f32,
}

impl Split {
    /// A largura da coluna do lado num corpo `body_w`: a do artista (ou a de omissão), presa entre
    /// `side_min` e o que deixa `table_min` à tabela (num corpo estreito demais ganha a do lado).
    #[must_use]
    pub fn measure(user: Option<f32>, body_w: f32, table_min: f32, side_min: f32) -> Self {
        let max = (body_w - band() - table_min).max(side_min);
        let want = user.unwrap_or_else(|| PANEL_MIN_W_PX.min(body_w * SIDE_SHARE));
        Self {
            hit: Rect::default(),
            side_w: safe_clamp(want, side_min, max),
            min: side_min,
            max,
        }
    }
}

/// Pinta o separador na faixa que começa em `x` (de `y`, altura `h`) e guarda onde agarra. Aceso
/// (o acento, mais grosso) com o rato por cima ou a arrastá-lo.
pub(super) fn paint(
    st: &mut DebugCardView,
    mut s: Split,
    (x, y, h): (f32, f32, f32),
    scene: &mut VectorScene,
    theme: Theme,
) {
    let b = band();
    let hot = st.splitting.is_some() || st.split_hover.get();
    let (line_w, line_tone, mark_tone) = if hot {
        (
            StrokeToken::Thick.px(),
            ColorToken::Accent,
            ColorToken::Accent,
        )
    } else {
        (
            StrokeToken::Thin.px(),
            ColorToken::Border,
            ColorToken::Text3,
        )
    };
    let line = Rect::new(x + (b - line_w) * 0.5, y, line_w, h);
    fill_rounded_rect(scene, line, 0.0, resolve(line_tone, theme));
    // A pega ao meio: um traço curto e grosso, como o do puxador de cima.
    let (mw, mh) = (StrokeToken::Thick.px(), Spacing::Xl4.px().min(h));
    let mark = Rect::new(x + (b - mw) * 0.5, y + (h - mh) * 0.5, mw, mh);
    fill_rounded_rect(scene, mark, mw * 0.5, resolve(mark_tone, theme));
    // Agarra à direita da calha da barra da tabela (as duas zonas nunca se tocam).
    let hair = StrokeToken::Thin.px();
    s.hit = Rect::new(x + hair, y, b - hair, h);
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
