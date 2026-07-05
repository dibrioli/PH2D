// ph2d-chrome-sync:z=250 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Vector Pen pill — **toggle**: clicking activates the Pen
//! tool when inactive; clicking again deactivates (back to default
//! tool, typically Brush). The shell drain in `render_loop::mod`
//! performs the actual `ToolRegistry::set_active` / `default_tool_id`
//! swap.
//!
//! Mirror of `image_tools_toggle.rs` shape — intercepted at Hero
//! level because the bus push must reach the shell's per-frame
//! action drain.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR_PEN {
        // R9: toggle. Pill state Pressed → tool currently active →
        // user wants to deactivate. The per-frame reconcile in
        // `render_loop::mod` keeps the pill state in sync with
        // `ToolRegistry::active()`, so reading the pill state is the
        // authoritative "is Pen active right now" signal accessible
        // from the chrome layer (which has no direct ToolRegistry
        // handle).
        let pen_currently_active = matches!(
            hero.store.get(ids::TOPBAR_VECTOR_PEN),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if pen_currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus.push(EditorAction::ActivateTool {
                tool_id: "vector_pen",
            });
        }
        return true;
    }
    false
}
