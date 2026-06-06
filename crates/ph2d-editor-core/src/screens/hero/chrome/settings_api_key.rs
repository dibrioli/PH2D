//! Settings → Anthropic API key cascade (P4, ADR-0061): open the submenu, then
//! commit the field's buffer to the shell on Save. Mirrors `settings_filter.rs`
//! (open-on-row-click) + `scene_picker.rs` (read the in-menu `TextInput`).
//!
//! The shell owns the secret — it persists the key to the user config dir
//! (`llm_vector::save_api_key`) and the LLM-vector feature resolves it lazily;
//! this handler just relays the entered value via `EditorAction::SetApiKey`.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // Top-level row → open the API-key popover beside it.
    if id == ids::CTX_MENU_SETTINGS_API_KEY {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsApiKeySubmenu,
        });
        return true;
    }
    // Save row → read the field's buffer, hand it to the shell, close the menu.
    if id == ids::CTX_MENU_API_KEY_SAVE {
        let key = hero
            .store
            .get(ids::CTX_MENU_API_KEY_INPUT)
            .and_then(|s| {
                if let InteractiveState::TextInput { text, .. } = s {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        hero.bus.push(EditorAction::SetApiKey(key));
        hero.store.close_context_menu();
        return true;
    }
    false
}
