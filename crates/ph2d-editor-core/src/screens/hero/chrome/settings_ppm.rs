//! Settings → Pixels per meter cascade: open submenu + write preset.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // Cascade entry: clicking "Pixels per meter \u{25b6}" REPLACES the
    // top-level menu with the PPM presets submenu. Anchored to the right
    // edge of the clicked cascade row.
    if id == ids::CTX_MENU_SETTINGS_PPM {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsPpmSubmenu,
        });
        return true;
    }
    // PPM presets — writes `project.pixels_per_meter` and closes the
    // menu; the shell reads the new value on the next import.
    let ppm_preset = if id == ids::CTX_MENU_PPM_16 {
        16.0 // LITERAL-PX-OK: pixels-per-meter preset (business value, not UI dimension)
    } else if id == ids::CTX_MENU_PPM_32 {
        32.0 // LITERAL-PX-OK: pixels-per-meter preset
    } else if id == ids::CTX_MENU_PPM_100 {
        100.0 // LITERAL-PX-OK: pixels-per-meter preset
    } else if id == ids::CTX_MENU_PPM_256 {
        256.0 // LITERAL-PX-OK: pixels-per-meter preset
    } else if id == ids::CTX_MENU_PPM_1024 {
        1024.0 // LITERAL-PX-OK: pixels-per-meter preset
    } else {
        return false;
    };
    hero.project.set_pixels_per_meter(ppm_preset);
    hero.store.close_context_menu();
    true
}
