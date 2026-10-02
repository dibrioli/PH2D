// ph2d-chrome-sync:z=130 (dispatch priority, ADR-0107; lower = earlier)
//! ⭐⭐ **Settings ▸ Interface font / Font weight / Font size** — os três eixos do texto da interface
//! ([`ph2d_tokens::UiTextStyle`], ordem do dono 2026-10-01), cada um num submenu de escolha.
//!
//! Espelha o `settings_motion.rs` na forma: escolher escreve em `HeroScreen.text_style`, que é o
//! dono do facto; a próxima pintura publica-o (`ph2d_text::set_active_text_style`) e a shell
//! persiste-o ao notar a diferença (`shells/desktop/src/prefs.rs`).
//!
//! ⚠️ **A ligação linha ↔ valor vive em TRÊS TABELAS, e é a única cópia**
//! ([`crate::screens::hero::text_style_rows`]): o clique ([`apply`]) e a marca do valor activo
//! leem-nas. Duas respostas à mesma pergunta divergem no
//! dia em que uma linha nova chegar a uma só delas — e um menu que grava e não marca é a família
//! que o `every_choice_submenu_marks_its_active_value` existe para apanhar.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::screens::hero::text_style_rows::{FONTS, SCALES, SIZES, WEIGHTS};
use ph2d_a11y::NodeId;

/// As três categorias do Settings e o submenu que cada uma abre.
const CATEGORIES: [(NodeId, ContextMenuKind); 4] = [
    (
        ids::CTX_MENU_SETTINGS_FONT,
        ContextMenuKind::SettingsFontSubmenu,
    ),
    (
        ids::CTX_MENU_SETTINGS_WEIGHT,
        ContextMenuKind::SettingsWeightSubmenu,
    ),
    (
        ids::CTX_MENU_SETTINGS_SIZE,
        ContextMenuKind::SettingsSizeSubmenu,
    ),
    (
        ids::CTX_MENU_SETTINGS_SCALE,
        ContextMenuKind::SettingsScaleSubmenu,
    ),
];

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if let Some((_, kind)) = CATEGORIES.iter().find(|(c, _)| *c == id) {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store
            .open_context_menu(ContextMenuRequest { x, y, kind: *kind });
        return true;
    }
    if let Some((_, z)) = SCALES.iter().find(|(r, _)| *r == id) {
        hero.ui_scale = *z;
        hero.store.close_context_menu();
        return true;
    }
    let style = &mut hero.text_style;
    if let Some((_, f)) = FONTS.iter().find(|(r, _)| *r == id) {
        style.font = *f;
    } else if let Some((_, w)) = WEIGHTS.iter().find(|(r, _)| *r == id) {
        style.weight = *w;
    } else if let Some((_, z)) = SIZES.iter().find(|(r, _)| *r == id) {
        style.size = *z;
    } else {
        return false;
    }
    hero.store.close_context_menu();
    true
}
