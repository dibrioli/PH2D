//! Vector inspector event router (thin forwarder).

use crate::VectorInspectorPanelState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};

pub(crate) fn apply_event(
    _state: &mut VectorInspectorPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(match ev {
        WidgetEvent::Click(id) if id == ph2d_editor_core::ids::VECTOR_INSPECTOR_CLOSE => {
            host.set_panel_visible("vector_inspector", false);
            true
        }
        // Shape-kind option click (§4.2): record the chosen index; the shell
        // bridge drains it into `VectorShapeTool::set_kind`. Index is the
        // option's position in `ShapeKind::ALL` order (see `ids::SHAPE_OPTION_IDS`).
        WidgetEvent::Click(id) if crate::ids::SHAPE_OPTION_IDS.contains(&id) => {
            let index = crate::ids::SHAPE_OPTION_IDS
                .iter()
                .position(|&o| o == id)
                .expect("id is in SHAPE_OPTION_IDS (matched by the guard above)");
            crate::state::set_pending_shape(index as u8);
            true
        }
        _ => false,
    })
}
