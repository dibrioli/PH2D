// ph2d-chrome-sync:z=100 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → Display unit cascade: open submenu + pick Meters/Pixels.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::project::DisplayUnit;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_UNIT {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsUnitSubmenu,
        });
        return true;
    }
    // Display-unit submenu options — write to `project.display_unit`
    // and close the menu. Inspector / Grid Settings / Gizmo readouts
    // read the project setting on the next paint.
    let unit_pick = if id == ids::CTX_MENU_UNIT_METERS {
        DisplayUnit::Meters
    } else if id == ids::CTX_MENU_UNIT_PIXELS {
        DisplayUnit::Pixels
    } else {
        return false;
    };
    hero.project.display_unit = unit_pick;
    hero.store.close_context_menu();
    true
}
