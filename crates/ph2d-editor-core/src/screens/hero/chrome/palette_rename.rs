// ph2d-chrome-sync:z=160 (dispatch priority, ADR-0107; lower = earlier)
//! Color-picker palette rename modal: commit the centered dialog's field to the active palette.
//! Reads the in-dialog `TextInput` on the Rename button click (mirrors the scene-picker pattern).
//! The field is the shared `BLENDER_PALETTE_NAME` (so Enter routes through the picker's existing
//! rename handler in `dispatch::key`); this handler covers the button path.

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::CTX_MENU_PALETTE_RENAME {
        return false;
    }
    let name = hero
        .store
        .get(ids::BLENDER_PALETTE_NAME)
        .and_then(|s| match s {
            InteractiveState::TextInput { text, .. } => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_default();
    hero.store
        .blender_rename_active_palette(ids::INSP_BLENDER_PICKER, &name);
    // Normalise (trim) the shown name, then close the modal.
    hero.store
        .sync_blender_palette_name_buffer(ids::INSP_BLENDER_PICKER);
    hero.store.close_context_menu();
    true
}
