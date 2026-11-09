//! **O âmbar do «mudou»** (plano 14 do editor de scripts, maquete `14_quadros_do_depurador.html`):
//! o valor que mudou desde o passo anterior e o fundo da linha dele.
//!
//! O `warn` dos temas derivados do Godot é o `warning` dele, `(0,83 0,78 0,62)` — um caqui pálido
//! (`oklch 0,83 0,057 93`) que, a `ROW_SELECTED_ALPHA` sobre o cartão, lê-se CINZENTO (medido na
//! foto da cena 14: `99,100,96`). A maquete aprovada pelo dono usa o âmbar do `tokens.json`
//! (`forge`: `warn` `oklch 0,795 0,125 80`, `warn-soft` `oklch 0,310 0,036 80`).
//!
//! A lei: **o matiz e o croma do âmbar canónico (a fábrica do `forge`), à LUMINOSIDADE do tema
//! activo** — a luminosidade é o que o tema escolheu para se ler sobre o seu fundo (num tema claro
//! o `warn` desce a `0,56`), o matiz e o croma são o que faz âmbar ser âmbar. No `forge` é a
//! identidade.

use crate::color::{Color, ColorToken};
use crate::color_space::srgb_to_oklch;
use crate::theme::Theme;

fn route(token: ColorToken, theme: Theme) -> Color {
    let here = token.resolve(theme);
    let canon = token.factory(Theme::Forge);
    let (l, _, _) = srgb_to_oklch(here.r, here.g, here.b);
    let (_, c, h) = srgb_to_oklch(canon.r, canon.g, canon.b);
    Color::from_oklch(l, c, h)
}

/// A tinta do que mudou (o valor novo, o rótulo, a borda da caixa, o ■ Paused).
#[must_use]
pub fn changed_ink(theme: Theme) -> Color {
    route(ColorToken::Warn, theme)
}

/// A tinta de um VALOR numérico no depurador: a luminosidade e o croma do `info` do tema, no
/// MATIZ do acento — a maquete escreveu os números `#9fd0ff`, o azul claro da família do acento
/// dela; no tema `dark` do Godot isto dá `168,208,253` (o `info` dele, `179,204,255`, lê-se quase
/// branco ao lado do texto, e o `info` canónico, matiz 235, ciano).
#[must_use]
pub fn value_ink(theme: Theme) -> Color {
    let info = ColorToken::Info.resolve(theme);
    let accent = ColorToken::Accent.resolve(theme);
    let (l, c, _) = srgb_to_oklch(info.r, info.g, info.b);
    let (_, _, h) = srgb_to_oklch(accent.r, accent.g, accent.b);
    Color::from_oklch(l, c, h)
}

/// O fundo opaco da linha / célula que mudou.
#[must_use]
pub fn changed_band(theme: Theme) -> Color {
    route(ColorToken::WarnSoft, theme)
}
