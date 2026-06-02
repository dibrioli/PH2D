//! TopBar Vector Shape pill — **toggle** (mirror of `vector_pencil_toggle.rs`).
//! Clicking activates the Shape tool when inactive; clicking again
//! deactivates. The shell drain in `render_loop::mod` performs the actual
//! `ToolRegistry::set_active` / `default_tool_id` swap.
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_shape_coord.md`
//!
//! 1. `ids.rs`: `pub const TOPBAR_VECTOR_SHAPE` (mirror `TOPBAR_VECTOR_PENCIL`).
//! 2. `chrome/mod.rs`: `mod vector_shape_toggle;` + call `apply` in the chain.
//! 3. `screens/hero/fixture.rs`: add the pill to the `vector_tools` cluster
//!    (`IconId::VectorShape`).

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR_SHAPE {
        let shape_currently_active = matches!(
            hero.store.get(ids::TOPBAR_VECTOR_SHAPE),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if shape_currently_active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus.push(EditorAction::ActivateTool {
                tool_id: "vector_shape",
            });
        }
        return true;
    }
    false
}
