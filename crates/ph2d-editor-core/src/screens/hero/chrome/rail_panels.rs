// ph2d-chrome-sync:z=70 (dispatch priority, ADR-0107; lower = earlier)
//! Left rail panel-visibility toggles (Inspector / Hierarchy).

use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::RAIL_SHOW_INSPECTOR {
        let new_visible = !hero.is_panel_visible("inspector");
        hero.panel_visibility.insert("inspector", new_visible);
        if let Some(InteractiveState::Button { state }) =
            hero.store.get_mut(ids::RAIL_SHOW_INSPECTOR)
        {
            *state = if new_visible {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
        return true;
    }
    if id == ids::RAIL_SHOW_HIERARCHY {
        let new_visible = !hero.is_panel_visible("hierarchy");
        hero.panel_visibility.insert("hierarchy", new_visible);
        if let Some(InteractiveState::Button { state }) =
            hero.store.get_mut(ids::RAIL_SHOW_HIERARCHY)
        {
            *state = if new_visible {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
        return true;
    }
    false
}
