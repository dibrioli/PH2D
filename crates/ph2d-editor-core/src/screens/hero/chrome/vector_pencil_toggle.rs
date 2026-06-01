//! TopBar Vector Pencil pill — **toggle**: clicking activates the Pencil
//! tool when inactive; clicking again deactivates (back to the default
//! tool). The shell drain in `render_loop::mod` performs the actual
//! `ToolRegistry::set_active` / `default_tool_id` swap.
//!
//! Mirror of `vector_pen_toggle.rs` — intercepted at Hero level because
//! the bus push must reach the shell's per-frame action drain.
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_pencil_coord.md`
//!
//! Not yet declared/called centrally. To activate:
//! 1. `ids.rs`: add `pub const TOPBAR_VECTOR_PENCIL: WidgetId` (mirror
//!    `TOPBAR_VECTOR_PEN`).
//! 2. `chrome/mod.rs`: `mod vector_pencil_toggle;` + call `apply` in the
//!    chrome dispatch chain (next to `vector_pen_toggle::apply`).
//! 3. `screens/hero/fixture.rs`: add the pill to the `vector_tools`
//!    TopBar cluster (`TopBarCluster::single("PENCIL", IconId::VectorPencil)`).

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR_PENCIL {
        // Toggle: the per-frame reconcile in `render_loop::mod` keeps the
        // pill state synced with `ToolRegistry::active()`, so a Pressed
        // pill means "Pencil is active right now" — the authoritative
        // signal from the chrome layer (no direct ToolRegistry handle).
        let pencil_currently_active = matches!(
            hero.store.get(ids::TOPBAR_VECTOR_PENCIL),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if pencil_currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus.push(EditorAction::ActivateTool {
                tool_id: "vector_pencil",
            });
        }
        return true;
    }
    false
}
