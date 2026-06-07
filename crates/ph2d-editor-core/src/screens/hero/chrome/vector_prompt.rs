//! LLM vector prompt dialog (P4, ADR-0061): the Generate button reads the prompt
//! field and hands it to the shell, which runs the LLM generation off-thread.
//! Mirrors `settings_api_key.rs` (read the in-menu `TextInput` on a button click).

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_VECTOR_PROMPT_GENERATE {
        let prompt = hero
            .store
            .get(ids::CTX_MENU_VECTOR_PROMPT_INPUT)
            .and_then(|s| {
                if let InteractiveState::TextInput { text, .. } = s {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        // An empty prompt is a no-op — keep the dialog open so the user can type.
        if prompt.trim().is_empty() {
            return true;
        }
        hero.bus
            .push(EditorAction::GenerateVectorFromPrompt(prompt));
        hero.store.close_context_menu();
        return true;
    }
    false
}
