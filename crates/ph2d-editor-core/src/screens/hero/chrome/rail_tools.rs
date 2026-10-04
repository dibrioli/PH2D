// ph2d-chrome-sync:z=50 (dispatch priority, ADR-0107; lower = earlier)
//! Left rail tool compounds — SPACE flip, VIEW (TOOL_HOME) cycle,
//! and the exclusive Transform tool radio group.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::{HeroScreen, ViewFocusKind};
use crate::widget::ButtonState;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOOL_SPACE {
        let next = !hero.store.tool_space_local();
        hero.store.set_tool_space_local(next);
        return true;
    }
    if id == ids::TOOL_HOME {
        // M14.7 polish: 3-mode cycle (Selected → Camera → All). Each click
        // EXECUTES the current mode and then advances the label so the
        // user can chain actions or see what's next.
        let current = hero.store.tool_view_mode();
        let kind = match current {
            1 => ViewFocusKind::Camera,
            2 => ViewFocusKind::All,
            _ => ViewFocusKind::Selected,
        };
        hero.bus.push(EditorAction::SetViewFocus { kind });
        let next = (current + 1) % 3;
        hero.store.set_tool_view_mode(next);
        return true;
    }
    // Transform tools are an EXCLUSIVE toggle group (a radio group with
    // no off-state): clicking any one activates it and de-activates the
    // others. Mirrors Blender / Unity convention — only one transform
    // tool is "current".
    const TRANSFORM_TOOLS: [ph2d_a11y::NodeId; 4] = [
        ids::TOOL_TRANSLATE,
        ids::TOOL_ROTATE,
        ids::TOOL_SCALE,
        ids::TOOL_PIVOT,
    ];
    if TRANSFORM_TOOLS.contains(&id) {
        for tool_id in TRANSFORM_TOOLS {
            if let Some(InteractiveState::Button { state }) = hero.store.get_mut(tool_id) {
                *state = if tool_id == id {
                    ButtonState::Pressed
                } else {
                    ButtonState::Normal
                };
            }
        }
        return true;
    }
    false
}
