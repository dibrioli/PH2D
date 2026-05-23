//! Upscale panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-C: the panel no longer holds the semantic mapping (slider
//! track → factor, algorithm-segment NodeId → `UpscaleAlgorithm`) — that
//! lives in `UpscaleTool::handle_panel_event` over in
//! `ph2d-tool-upscale`. Here we just classify each `WidgetEvent` into a
//! tool-agnostic [`PanelEvent`], push it through
//! `EditorAction::ToolPanelEvent`, and let the action-bus drain in the
//! shell call `Tool::handle_panel_event` on the active tool. Cancel maps
//! to `EditorAction::CancelActiveTool`.
//!
//! The panel still owns the slider ↔ chip MIRROR (the chip's stored
//! number value follows a slider drag, and the slider's stored value
//! follows a chip commit) — that is UI-state-local to the widget store,
//! not authoritative tool state.

use crate::ids;
use crate::state::UpscalePanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_upscale::params::{scale_to_slider, slider_to_scale};

pub(crate) fn apply_event(
    _state: &mut UpscalePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Algorithm segmented buttons — three Click variants, each
        // forwarded as a `PanelEvent::Click` so the tool routes to
        // `apply_ui_edit(SetAlgorithm(_))`.
        WidgetEvent::Click(id)
            if id == ids::UPS_ALGO_LANCZOS3
                || id == ids::UPS_ALGO_NEAREST
                || id == ids::UPS_ALGO_XBR =>
        {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Scale slider — read the live normalized track, forward to the
        // tool, mirror the projected factor into the chip's stored value.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    track as f64,
                )));
            let chip_val = slider_to_scale(track) as f64;
            host.store_mut()
                .set_number_value(ids::UPS_SCALE_NUM, chip_val);
            true
        }
        // Scale chip — read the raw factor, forward, mirror back onto
        // the slider's stored track.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE_NUM => {
            let factor = host.store().number_value(id).unwrap_or(0.0) as f32;
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    factor as f64,
                )));
            if let Some(InteractiveState::Slider { value, .. }) =
                host.store_mut().get_mut(ids::UPS_SCALE)
            {
                *value = scale_to_slider(factor);
            }
            true
        }
        // Apply — bake at full resolution.
        WidgetEvent::Click(id) if id == ids::UPS_APPLY => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Cancel — abandon + deactivate. Shell switches back to the
        // default tool, hiding the panel.
        WidgetEvent::Click(id) if id == ids::UPS_CANCEL => {
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
