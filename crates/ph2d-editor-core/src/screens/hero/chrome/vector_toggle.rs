// ph2d-chrome-sync:z=270 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Vector pill — **toggle** for the single Vector drawing tool
//! (ADR-0108 cutover). Clicking activates `vector` when inactive; clicking
//! again deactivates. The shell drain in `render_loop::mod` performs the actual
//! `ToolRegistry::set_active` / `default_tool_id` swap.
//!
//! Central wiring: `ids::TOPBAR_VECTOR` (ids/chrome/topbar.rs) + the pill in
//! `screens/hero/fixture.rs` (`IconId::Vector`) + registration in
//! `topbar/mod.rs::populate`.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR {
        let currently_active = matches!(
            hero.store.get(ids::TOPBAR_VECTOR),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus
                .push(EditorAction::ActivateTool { tool_id: "vector" });
        }
        return true;
    }
    false
}
