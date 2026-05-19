//! Scene picker popover — row click sets the current scene name.

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::{HeroScreen, fixture};

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let Some(slot) = ids::CTX_SCENE_ROWS.iter().position(|x| *x == id) else {
        return false;
    };
    // Re-filter the scene list with the same query the painter used so
    // index→name maps correctly.
    let query = hero
        .store
        .get(ids::CTX_SCENE_SEARCH)
        .and_then(|s| {
            if let InteractiveState::TextInput { text, .. } = s {
                Some(text.as_str())
            } else {
                None
            }
        })
        .unwrap_or("");
    let lower_q = query.to_lowercase();
    let visible: Vec<&'static str> = fixture::scenes()
        .iter()
        .copied()
        .filter(|s| lower_q.is_empty() || s.to_lowercase().contains(&lower_q))
        .take(ids::CTX_SCENE_ROWS.len())
        .collect();
    if let Some(name) = visible.get(slot) {
        hero.store.set_current_scene_name(*name);
    }
    hero.store.close_context_menu();
    true
}
