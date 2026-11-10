//! **O âmbar do «mudou»** e a tinta dos números do depurador: o matiz e o croma do âmbar canónico
//! (`forge`) à luminosidade do tema activo. Medição e porquê:
//! `docs/CodeEditor/14_plano_quadros.md` §6.

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
/// MATIZ do acento (plano 14 §6).
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
