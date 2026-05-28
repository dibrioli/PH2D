//! Padding panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-C: the panel no longer holds the semantic mapping
//! (slider track → signed px via `slider_to_px`, then construction of a
//! typed `PaddingUiEdit::Top(px)`) — that lives in
//! `PaddingTool::handle_panel_event` over in `ph2d-tool-padding`.
//! Here we just classify each `WidgetEvent` into a tool-agnostic
//! [`PanelEvent`], push it through `EditorAction::ToolPanelEvent`, and
//! let the action-bus drain in the shell call `Tool::handle_panel_event`
//! on the active tool. Cancel maps to `EditorAction::CancelActiveTool`.
//!
//! 2026-05-27 refactor: slider ↔ chip mirror moved into the dispatch
//! via [`WidgetStore::link_slider_number_mapped`] (see `populate.rs`).
//! Both `ValueChanged(slider)` AND `ValueChanged(chip)` fire when one
//! side mutates, so we forward exactly once — keyed off the slider id —
//! and always carry the slider's track value (the tool projects to px).

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

use crate::state::PaddingPanelState;

/// True iff `id` is one of the four edge sliders. Only slider events
/// drive the forward — the dispatch already mirrors chip→slider via
/// the registered mapping, so a chip ValueChanged is redundant.
fn is_edge_slider(id: NodeId) -> bool {
    id == ids::PAD_TOP || id == ids::PAD_RIGHT || id == ids::PAD_BOTTOM || id == ids::PAD_LEFT
}

pub(crate) fn apply_event(
    _state: &mut PaddingPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Edge slider value changed (drag, OR mirror from chip commit /
        // stepper / drag-scrub via `link_slider_number_mapped`). Read
        // the live track and forward it as the canonical SetValue —
        // the tool's apply_ui_edit projects to px.
        WidgetEvent::ValueChanged(id) if is_edge_slider(id) => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    track as f64,
                )));
            true
        }
        // Chip ValueChanged — dispatch already mirrored to the slider,
        // which fires its own ValueChanged handled above. Swallow so the
        // tool isn't double-notified for the same edit.
        WidgetEvent::ValueChanged(id)
            if id == ids::PAD_TOP_NUM
                || id == ids::PAD_RIGHT_NUM
                || id == ids::PAD_BOTTOM_NUM
                || id == ids::PAD_LEFT_NUM =>
        {
            true
        }
        // Pivot-mode toggle — flips recenter/keep. Reset the pressed
        // state; the active look is the per-frame snapshot in `paint`.
        WidgetEvent::Click(id) if id == ids::PAD_PIVOT_RECENTER => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Apply button — bake at full resolution.
        WidgetEvent::Click(id) if id == ids::PAD_APPLY => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Reset-all button — snap every per-edge field back to 0 +
        // pivot-recenter to default. Routes to the tool's
        // `apply_ui_edit::ResetAll`.
        WidgetEvent::Click(id) if id == ids::PAD_RESET => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Cancel — abandon + deactivate the tool. Shell switches back to
        // the default tool via `CancelActiveTool`, hiding this panel and
        // restoring the Inspector.
        WidgetEvent::Click(id) if id == ids::PAD_CANCEL => {
            reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    }
}

fn reset_button(host: &mut dyn PanelHostInternal, id: NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
