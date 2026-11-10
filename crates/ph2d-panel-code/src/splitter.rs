//! ⭐ O separador ↔ entre uma tabela e a coluna do lado (smoke do dono 09/11, cena 14: «coloque um
//! separador vertical onde o cursor vai pegar»): uma faixa vertical; arrastá-la de lado muda a largura
//! da coluna do lado e a tabela fica com o resto, cada uma presa ao seu mínimo. Moram cá a medida, a
//! pintura e o arrasto; a Sheet do depurador (Call stack) e a grelha da Bancada (o mapa «Where») usam-nos.

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

/// Pinta a linha do separador na faixa que começa em `x` (de `y`, altura `h`) e devolve onde
/// agarra. `hot` (o rato por cima ou a arrastá-lo) = o acento, mais grosso.
pub fn paint_band(
    hot: bool,
    (x, y, h): (f32, f32, f32),
    scene: &mut VectorScene,
    theme: Theme,
) -> Rect {
    let b = band();
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
    Rect::new(x + hair, y, b - hair, h)
}

/// O separador de uma vista: a largura que o artista escolheu, o arrasto em curso, o «por cima»
/// (vale um quadro: o `look` volta a escrevê-lo) e o pintado no último quadro.
#[derive(Debug, Default)]
pub struct Splitter {
    pub side_w: Option<f32>,
    pub dragging: Option<(f32, f32)>,
    pub hover: std::cell::Cell<bool>,
    pub painted: Option<Split>,
}

impl Splitter {
    /// Pinta `s` na faixa que começa em `x` (de `y`, altura `h`) e guarda onde agarra.
    pub fn paint(
        &mut self,
        mut s: Split,
        at: (f32, f32, f32),
        scene: &mut VectorScene,
        theme: Theme,
    ) {
        let hot = self.dragging.is_some() || self.hover.replace(false);
        s.hit = paint_band(hot, at, scene, theme);
        self.painted = Some(s);
    }

    /// Carregar no separador começa o arrasto. `true` = tomado.
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        let Some(s) = self.painted.filter(|s| s.hit.contains(x, y)) else {
            return false;
        };
        self.dragging = Some((x, s.side_w));
        true
    }

    /// O arrasto: para a esquerda a coluna do lado cresce. `true` = havia arrasto.
    pub fn drag_to(&mut self, x: f32) -> bool {
        self.drag_by(x, -1.0)
    }

    /// [`Self::drag_to`] com a coluna do lado à ESQUERDA: para a direita ela cresce.
    pub fn drag_right_to(&mut self, x: f32) -> bool {
        self.drag_by(x, 1.0)
    }

    fn drag_by(&mut self, x: f32, sign: f32) -> bool {
        let Some((x0, w0)) = self.dragging else {
            return false;
        };
        let s = self.painted.unwrap_or_default();
        self.side_w = Some(safe_clamp(w0 + sign * (x - x0), s.min, s.max));
        true
    }

    /// O Esc a meio do arrasto: a largura volta à do carregar. `true` = havia arrasto.
    pub fn cancel(&mut self) -> bool {
        let Some((_, w0)) = self.dragging.take() else {
            return false;
        };
        self.side_w = Some(w0);
        true
    }

    /// O rato está sobre o separador, ou a arrastá-lo? (Lembra-o para a pintura o acender.)
    pub fn look(&self, x: f32, y: f32) -> bool {
        let over = self.painted.is_some_and(|s| s.hit.contains(x, y));
        self.hover.set(over);
        over || self.dragging.is_some()
    }
}
