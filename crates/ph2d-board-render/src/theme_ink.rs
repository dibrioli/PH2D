//! A tinta do tema (módulo filho de [`crate`]: o ficheiro estava no tecto de LOC).

use ph2d_board_model::Rgba;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Color;

use crate::{doc_color, token};

/// ⭐ **A TINTA DO TEMA** — smoke do dono (07/10): *«em temas claros linhas e fontes deveriam por
/// padrão ser pretas ou muito escuras»*. A tinta de nascença do quadro é UMA cor do documento,
/// [`ph2d_board_model::DEFAULT_INK`] (`#1e1e1e`, a do Excalidraw), e nunca a do tema em que a forma
/// nasceu (a cena aberta no escuro gravava tinta clara e ficava ilegível no claro). Desenha-se tal como
/// é num quadro de fundo CLARO, e com o texto do tema (`text-1`) num de fundo ESCURO — o idioma do
/// Excalidraw. Claro ou escuro decide-o a luminância do fundo (`bg-1`), o que vale para os 12 temas.
/// ⚠️ O texto dentro de uma forma PREENCHIDA lê-se contra o preenchimento, não contra o quadro: não muda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeInk {
    dark: bool,
    light: Color,
}

impl Default for ThemeInk {
    fn default() -> Self {
        Self::of(Theme::default())
    }
}

impl ThemeInk {
    #[must_use]
    pub fn of(theme: Theme) -> Self {
        let b = ColorToken::Bg1.resolve(theme);
        Self {
            dark: Rgba([b.r, b.g, b.b, 255]).luminance() < 0.5,
            light: token(ColorToken::Text1, theme),
        }
    }

    /// A cor de um traço (ou de um texto sobre o quadro) com a tinta `c`.
    #[must_use]
    pub fn color(self, c: Rgba) -> Color {
        if self.dark && c.0 == ph2d_board_model::DEFAULT_INK {
            self.light.with_alpha(f32::from(c.0[3]) / 255.0)
        } else {
            doc_color(c)
        }
    }

    /// A cor do texto de uma forma: sobre o quadro segue o tema; sobre um preenchimento, não.
    #[must_use]
    pub fn text(self, st: &ph2d_board_model::Style) -> Color {
        if st.fill.is_some() {
            doc_color(st.text_color)
        } else {
            self.color(st.text_color)
        }
    }
}
