//! ⭐ **A tabela `id da linha ⇄ tema` do menu de tema** — os oito, das duas famílias.
//!
//! Uma tabela, três leitores: o despacho (`chrome::theme`), a marca de estado do menu
//! (`context_menu_overlay`) e as rows (`menu_rows`). Um `match` em cada um seria o trio que
//! envelhece em separado — e a família moderna (2026-09-04) foi exactamente o dia em que os três
//! teriam de mudar juntos.
//!
//! ⚠️ **Mora num ficheiro PRÓPRIO, e não em `menu_rows.rs`, por causa do gate
//! `every_menu_row_reaches_a_handler`:** ele exige que todo id pintado seja nomeado num sítio de
//! despacho *fora* da tabela de rows, e mede-o lendo o fonte. A tabela que o handler consulta É
//! o sítio de despacho — e aqui ela é visível ao gate.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;

/// `(id da linha do menu de tema, tema)` — os oito, na ordem do menu.
pub const THEME_MENU: [(NodeId, Theme); 8] = [
    (ids::CTX_MENU_THEME_FORGE, Theme::Forge),
    (ids::CTX_MENU_THEME_PAINT, Theme::Workshop),
    (ids::CTX_MENU_THEME_SUNSTONE, Theme::Sunstone),
    (ids::CTX_MENU_THEME_BLUEPRINT, Theme::Blueprint),
    (ids::CTX_MENU_THEME_DARK, Theme::Dark),
    (ids::CTX_MENU_THEME_GRAY, Theme::Gray),
    (ids::CTX_MENU_THEME_LIGHT, Theme::Light),
    (ids::CTX_MENU_THEME_OLED, Theme::Oled),
];

/// O id da linha de menu que escolhe `theme`.
#[must_use]
pub fn theme_menu_id(theme: Theme) -> NodeId {
    THEME_MENU
        .iter()
        .find(|(_, t)| *t == theme)
        .map(|(id, _)| *id)
        .expect("todo tema tem uma linha no menu")
}

/// O tema que a linha de menu `id` escolhe, se for uma.
#[must_use]
pub fn theme_of_menu_id(id: NodeId) -> Option<Theme> {
    THEME_MENU
        .iter()
        .find(|(mid, _)| *mid == id)
        .map(|(_, t)| *t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Os oito temas têm linha, e `theme_of_menu_id` é o inverso exacto de `theme_menu_id`.
    #[test]
    fn every_theme_has_one_row_and_the_map_round_trips() {
        for theme in Theme::ALL {
            let id = theme_menu_id(theme);
            assert_eq!(theme_of_menu_id(id), Some(theme));
        }
        assert_eq!(THEME_MENU.len(), Theme::ALL.len());
        assert_eq!(theme_of_menu_id(ids::TOOL_UNDO), None);
    }
}
