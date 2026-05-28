//! Upscale panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-C: the panel does NOT own the semantic mapping (slider
//! track → factor, algorithm-segment NodeId → `UpscaleAlgorithm`) —
//! that lives in `UpscaleTool::handle_panel_event` over in
//! `ph2d-tool-upscale`. Here we classify each `WidgetEvent` into a
//! tool-agnostic [`PanelEvent`], push it through
//! `EditorAction::ToolPanelEvent`, and let the action-bus drain in the
//! shell call `Tool::handle_panel_event`. Cancel maps to
//! `EditorAction::CancelActiveTool`.
//!
//! 2026-05-27: chip ↔ slider mirror is in dispatch via
//! [`WidgetStore::link_slider_number_mapped`] (see `populate.rs`).
//! Both `ValueChanged(slider)` and `ValueChanged(chip)` fire when one
//! side mutates; we forward exactly once — keyed off the slider id —
//! always carrying the slider's `0..1` track (the tool projects via
//! `slider_to_scale`). Out-of-range chip input ("999") is clamped by
//! the dispatch's `apply_chip_value_with_mirror` re-sync.

use crate::ids;
use crate::state::UpscalePanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

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
        // Scale slider value changed (drag, OR mirror from chip commit
        // / stepper / drag-scrub via `link_slider_number_mapped`).
        // Forward the canonical track value.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE => {
            let track = host
                .store()
                .slider(ids::UPS_SCALE)
                .map(|(_, v)| v)
                .unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    ids::UPS_SCALE,
                    track as f64,
                )));
            true
        }
        // Chip ValueChanged — dispatch already mirrored to the slider,
        // which fires its own ValueChanged handled above. Swallow.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE_NUM => true,
        // Apply — bake at full resolution.
        WidgetEvent::Click(id) if id == ids::UPS_APPLY => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Reset-all — algorithm + scale back to defaults.
        WidgetEvent::Click(id) if id == ids::UPS_RESET => {
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
