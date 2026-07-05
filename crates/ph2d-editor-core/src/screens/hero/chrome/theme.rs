// ph2d-chrome-sync:z=10 (dispatch priority, ADR-0107; lower = earlier)
//! Theme menu — 4 theme picks (Forge / Workshop / Sunstone / Blueprint).

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use ph2d_tokens::Theme;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let new_theme = if id == ids::CTX_MENU_THEME_FORGE {
        Theme::Forge
    } else if id == ids::CTX_MENU_THEME_PAINT {
        Theme::Workshop
    } else if id == ids::CTX_MENU_THEME_SUNSTONE {
        Theme::Sunstone
    } else if id == ids::CTX_MENU_THEME_BLUEPRINT {
        Theme::Blueprint
    } else {
        return false;
    };
    hero.theme = new_theme;
    hero.store.close_context_menu();
    true
}
