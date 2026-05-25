//! Settings → Display cascade: open submenu + pick VSync / Immediate.
//! Mirrors `settings_filter.rs` 1:1.
//!
//! Picking a mode raises `EditorAction::SetPresentMode` so the shell —
//! the owner of the swap chain — reconfigures the surface present mode.
//! VSync (`Fifo`) is perfectly smooth (hardware-paced); Immediate is
//! non-blocking and kills the mouse-move stutter (the demo scene
//! animates every frame, so the render loop is continuous and `Fifo`'s
//! `acquire_frame` stalls under load — see the 2026-05-21 stutter fix).

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_DISPLAY {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsDisplaySubmenu,
        });
        return true;
    }
    // Display submenu options — push the present-mode change to the
    // shell, mirror in store so the menu's "selected" bullet can
    // find the active row, and close the menu.
    let vsync = if id == ids::CTX_MENU_DISPLAY_VSYNC {
        true
    } else if id == ids::CTX_MENU_DISPLAY_IMMEDIATE {
        false
    } else {
        return false;
    };
    hero.store.set_present_vsync(vsync);
    hero.bus.push(EditorAction::SetPresentMode { vsync });
    hero.store.close_context_menu();
    true
}
