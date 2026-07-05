// ph2d-chrome-sync:z=130 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → Text rendering cascade: toggle entre Default e
//! Crisp Heavy. Mirrors `settings_present.rs` 1:1.
//!
//! Picking a mode writes `HeroScreen.text_rendering`. The next frame's
//! `paint_hero_screen` calls `paint::set_text_rendering(hero.text_rendering)`,
//! which publishes the choice to the `paint_text*` thread-local. No
//! action bus push is needed — the strategy is core-side state, not
//! a shell-side resource (unlike `SetPresentMode`).

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;
use ph2d_tokens::TextRendering;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_TEXT {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsTextSubmenu,
        });
        return true;
    }
    let chosen = if id == ids::CTX_MENU_TEXT_DEFAULT {
        TextRendering::Default
    } else if id == ids::CTX_MENU_TEXT_CRISP_HEAVY {
        TextRendering::CrispHeavy
    } else if id == ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS {
        TextRendering::CrispHeavyPlus
    } else {
        return false;
    };
    hero.text_rendering = chosen;
    hero.store.close_context_menu();
    true
}
