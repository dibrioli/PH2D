// ph2d-chrome-sync:z=290 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar Vector Direct Select pill — **toggle** (mirror of `vector_select_toggle.rs`).
//!
//! ## ⚠ Central wiring required (Coord) — see `docs/HANDOFF_vector_w2_t23_select_coord.md`
//!
//! 1. `ids.rs`: `pub const TOPBAR_VECTOR_DIRECT`.
//! 2. `chrome/mod.rs`: `mod vector_direct_toggle;` + call `apply` in the chain.
//! 3. `fixture.rs`: pill DIRECT (`IconId::VectorDirect`) in the `vector_tools` cluster.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOPBAR_VECTOR_DIRECT {
        let active = matches!(
            hero.store.get(ids::TOPBAR_VECTOR_DIRECT),
            Some(InteractiveState::Button {
                state: ButtonState::Pressed,
            })
        );
        if active {
            hero.bus.push(EditorAction::CancelActiveTool);
        } else {
            hero.bus.push(EditorAction::ActivateTool {
                tool_id: "vector_direct",
            });
        }
        return true;
    }
    false
}
