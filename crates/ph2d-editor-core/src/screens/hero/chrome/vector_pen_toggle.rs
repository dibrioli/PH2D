//! TopBar Vector Pen pill — pushes `EditorAction::ActivateTool { tool_id: "vector_pen" }`
//! on click. The shell drain in `render_loop::mod` performs the
//! actual `ToolRegistry::set_active`.
//!
//! Mirror of `image_tools_toggle.rs` shape — intercepted at Hero
//! level because the bus push must reach the shell's per-frame
//! action drain.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR_PEN {
        hero.bus.push(EditorAction::ActivateTool {
            tool_id: "vector_pen",
        });
        return true;
    }
    false
}
