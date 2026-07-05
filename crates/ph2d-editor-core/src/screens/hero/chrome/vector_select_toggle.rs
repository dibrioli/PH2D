// ph2d-chrome-sync:z=280 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Vector Select pill — **toggle** (mirror of `vector_pencil_toggle.rs`).
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_t23_select_coord.md`
//!
//! 1. `ids.rs`: `pub const TOPBAR_VECTOR_SELECT`.
//! 2. `chrome/mod.rs`: `mod vector_select_toggle;` + call `apply` in the chain.
//! 3. `fixture.rs`: pill SELECT (`IconId::VectorSelect`) in the `vector_tools` cluster.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR_SELECT {
        let active = matches!(
            hero.store.get(ids::TOPBAR_VECTOR_SELECT),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus.push(EditorAction::ActivateTool {
                tool_id: "vector_select",
            });
        }
        return true;
    }
    false
}
