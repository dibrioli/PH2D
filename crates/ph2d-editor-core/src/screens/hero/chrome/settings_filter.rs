// ph2d-chrome-sync:z=110 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → Image filter cascade: open submenu + pick Pixel Art /
//! Smooth. Mirrors `settings_unit.rs` 1:1.
//!
//! Picking a mode writes `project.image_filter` (so the menu state +
//! any editor-side readout is correct on the next paint) AND raises
//! `EditorAction::SetImageFilter` so the shell — the owner of the GPU
//! sampler state — rebuilds the atlas + individual samplers and selects
//! the matching `peniko::ImageQuality` for the Vello preview. This is
//! the single global image-filter mode applied to EVERYTHING.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::project::ImageFilterMode;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_FILTER {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsFilterSubmenu,
        });
        return true;
    }
    // Image-filter submenu options — write `project.image_filter`, push
    // the change to the shell, and close the menu.
    let pick = if id == ids::CTX_MENU_FILTER_PIXELART {
        ImageFilterMode::PixelArt
    } else if id == ids::CTX_MENU_FILTER_SMOOTH {
        ImageFilterMode::Smooth
    } else {
        return false;
    };
    hero.project.image_filter = pick;
    hero.bus.push(EditorAction::SetImageFilter { mode: pick });
    hero.store.close_context_menu();
    true
}
