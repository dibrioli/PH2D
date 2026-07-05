// ph2d-chrome-sync:z=20 (dispatch priority, ADR-0107; lower = earlier)
//! Radius scale presets — Sharp / Default / Round.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let new_radius_scale = if id == ids::CTX_MENU_RADIUS_SHARP {
        0.2_f32 // LITERAL-PX-OK: radius scale preset (business value, not dimension)
    } else if id == ids::CTX_MENU_RADIUS_DEFAULT {
        1.0_f32
    } else if id == ids::CTX_MENU_RADIUS_ROUND {
        1.6_f32 // LITERAL-PX-OK: radius scale preset (business value)
    } else {
        return false;
    };
    hero.store.set_radius_scale(new_radius_scale);
    hero.store.close_context_menu();
    true
}
