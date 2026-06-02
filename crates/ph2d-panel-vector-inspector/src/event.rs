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
        _ => false,
    })
}
