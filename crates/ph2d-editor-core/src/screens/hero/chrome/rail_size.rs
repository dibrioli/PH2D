// ph2d-chrome-sync:z=30 (dispatch priority, ADR-0107; lower = earlier)
//! Rail-button size presets — Small / Medium / Large. Added
//! 2026-05-24 so the user can pick between the new (Small, 36 px),
//! pre-2026-05-24 (Large, 44 px), and halfway (Medium, 40 px) chip
//! edges from the Themes menu. The store keeps the active preset;
//! the layout re-computes rail width each frame from it.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use crate::widget::RailButtonSize;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let size = if id == ids::CTX_MENU_RAIL_SIZE_SMALL {
        RailButtonSize::Small
    } else if id == ids::CTX_MENU_RAIL_SIZE_MEDIUM {
        RailButtonSize::Medium
    } else if id == ids::CTX_MENU_RAIL_SIZE_LARGE {
        RailButtonSize::Large
    } else {
        return false;
    };
    hero.store.set_rail_button_size(size);
    hero.store.close_context_menu();
    true
}
