//! ⭐ **As linhas dos três submenus do TEXTO da interface e o valor de cada uma** (2026-10-01) — a
//! única cópia da ligação linha ↔ valor, lida pelo clique (`chrome/settings_font.rs`) e pela marca
//! do valor activo (`menu_row_mark`). ⚠️ Mora fora do `chrome/` porque os módulos de lá são
//! gerados privados (`ph2d-chrome-sync`) e a marca também a lê.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_tokens::{UiFont, UiScale, UiTextSize, UiWeight};

/// A linha de cada fonte.
pub(crate) const FONTS: [(NodeId, UiFont); 3] = [
    (ids::CTX_MENU_FONT_INTER, UiFont::Inter),
    (ids::CTX_MENU_FONT_NOTO_SANS, UiFont::NotoSans),
    (ids::CTX_MENU_FONT_ATKINSON, UiFont::AtkinsonHyperlegible),
];

/// A linha de cada peso.
pub(crate) const WEIGHTS: [(NodeId, UiWeight); 3] = [
    (ids::CTX_MENU_WEIGHT_LIGHT, UiWeight::Light),
    (ids::CTX_MENU_WEIGHT_NORMAL, UiWeight::Normal),
    (ids::CTX_MENU_WEIGHT_STRONG, UiWeight::Strong),
];

/// A linha de cada tamanho.
pub(crate) const SIZES: [(NodeId, UiTextSize); 3] = [
    (ids::CTX_MENU_SIZE_SMALL, UiTextSize::Small),
    (ids::CTX_MENU_SIZE_NORMAL, UiTextSize::Normal),
    (ids::CTX_MENU_SIZE_LARGE, UiTextSize::Large),
];

/// A linha de cada degrau da escala da interface inteira.
pub(crate) const SCALES: [(NodeId, UiScale); 7] = [
    (ids::CTX_MENU_SCALE_80, UiScale::P80),
    (ids::CTX_MENU_SCALE_90, UiScale::P90),
    (ids::CTX_MENU_SCALE_100, UiScale::P100),
    (ids::CTX_MENU_SCALE_125, UiScale::P125),
    (ids::CTX_MENU_SCALE_150, UiScale::P150),
    (ids::CTX_MENU_SCALE_175, UiScale::P175),
    (ids::CTX_MENU_SCALE_200, UiScale::P200),
];

/// ⭐ **A linha `id` é o valor activo?** — lido do estilo PUBLICADO neste quadro.
#[must_use]
pub(crate) fn is_current(id: NodeId) -> bool {
    let s = ph2d_text::active_text_style();
    FONTS.iter().any(|(r, f)| *r == id && *f == s.font)
        || WEIGHTS.iter().any(|(r, w)| *r == id && *w == s.weight)
        || SIZES.iter().any(|(r, z)| *r == id && *z == s.size)
        || SCALES
            .iter()
            .any(|(r, z)| *r == id && *z == crate::ui_scale::active())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **Cada tabela cobre o seu eixo inteiro, pela ordem do menu** — um valor novo num enum que
    /// não ganhe linha seria uma escolha inalcançável.
    #[test]
    fn every_value_has_its_row_in_menu_order() {
        assert_eq!(FONTS.map(|(_, f)| f), UiFont::ALL);
        assert_eq!(WEIGHTS.map(|(_, w)| w), UiWeight::ALL);
        assert_eq!(SIZES.map(|(_, z)| z), UiTextSize::ALL);
        assert_eq!(SCALES.map(|(_, z)| z), UiScale::ALL);
    }
}
