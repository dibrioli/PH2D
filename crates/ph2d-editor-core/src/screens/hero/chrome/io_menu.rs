// ph2d-chrome-sync:z=80 (dispatch priority, ADR-0107; lower = earlier)
//! File menu items — Import (raises a flag for the host to open the
//! native picker) + Save / Save As / Open Project placeholders until
//! the pilot project wires real I/O.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_IMPORT {
        hero.import_requested = true;
        hero.store.close_context_menu();
        return true;
    }
    if id == ids::CTX_MENU_SAVE || id == ids::CTX_MENU_SAVE_AS || id == ids::CTX_MENU_OPEN_PROJECT {
        hero.store.close_context_menu();
        return true;
    }
    false
}
