//! Vector Style panel event router (thin forwarder).
//!
//! Same forwarder shape as the `panel_seam!`-generated code (Padding /
//! BgRemoval / …), hand-written here so the Width slider can be **registered at
//! a non-neutral initial value** in `populate.rs` (the macro hardcodes `0.5`,
//! which would render the knob at ~10 px instead of the tool's 3 px default).
//!
//! - Width slider drag (or mirror from the chip) → `ToolPanelEvent::SetValue`
//!   carrying the live track `0..1`; the tool projects it to px.
//! - Width chip `ValueChanged` → swallowed (the dispatch mirror already fired
//!   the slider's `ValueChanged`, handled above — avoids a double notify).
//! - Fill "None" `Click` → `ToolPanelEvent::Click` (tool clears the fill).
//! - Close (X) `Click` → `CancelActiveTool` (deactivates the tool, mirror of
//!   the Padding panel's Cancel).
//!
//! The two colour swatches never reach here — their Down opens the shared OKLCH
//! picker via the generic `is_picker_swatch` dispatch (short-circuits in
//! pointer.rs); the shell's `vector_bridge` reads the pick back into the tool.

use crate::ids;
use crate::state::VectorPanelState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;

pub(crate) fn apply_event(
    _state: &mut VectorPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_WIDTH => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Polygon Sides slider — same shape as Width (track 0..1 → tool projects
        // to a side count).
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_SIDES => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Chip edits already mirrored to their slider (which fires its own
        // ValueChanged, handled above): swallow to avoid a double notify.
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_WIDTH_NUM => true,
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_SIDES_NUM => true,
        WidgetEvent::Click(id) if id == ids::VECTOR_FILL_NONE => {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Draw-mode buttons + Boolean buttons: forward the Click over the generic
        // tool channel. Mode clicks land on `VectorTool::handle_panel_event`
        // (sets the mode); Boolean clicks are picked up by the shell drain, which
        // applies the op to the document (they are not Style edits, so the tool
        // ignores them). Same forwarder shape either way.
        WidgetEvent::Click(id)
            if id == ids::VECTOR_MODE_PEN
                || id == ids::VECTOR_MODE_RECT
                || id == ids::VECTOR_MODE_ELLIPSE
                || id == ids::VECTOR_MODE_POLYGON
                || id == ids::VECTOR_BOOL_UNION
                || id == ids::VECTOR_BOOL_SUBTRACT
                || id == ids::VECTOR_BOOL_INTERSECT =>
        {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) if id == ids::VECTOR_CLOSE => {
            seam_reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
