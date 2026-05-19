//! View overlay toggles — mirror UI / stats HUD / grid overlay.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_MIRROR_UI {
        hero.view.ui_mirrored = !hero.view.ui_mirrored;
        hero.store.close_context_menu();
        return true;
    }
    if id == ids::CTX_MENU_SHOW_STATS {
        hero.view.stats_visible = !hero.view.stats_visible;
        hero.store.close_context_menu();
        return true;
    }
    if id == ids::CTX_MENU_SHOW_GRID {
        hero.view.grid_visible = !hero.view.grid_visible;
        hero.store.close_context_menu();
        return true;
    }
    false
}
