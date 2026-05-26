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
//! Slider ↔ chip mirror is the panel's job (chip lives in natural unit,
//! slider in track `0..1`; no `link_slider_number` because that helper
//! couples both in the same `0..1` space). Slider drag writes the chip
//! factor; chip commit clamps to `[MIN_SCALE_FACTOR, SCALE_FULL_SCALE]`
//! and writes the slider track. The forwarded `PanelEvent::SetValue`
//! always carries the track (the tool expects track and maps via
//! `slider_to_scale`).

use crate::ids;
use crate::state::UpscalePanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_upscale::params::{
    MIN_SCALE_FACTOR, SCALE_FULL_SCALE, scale_to_slider, slider_to_scale,
};

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
        // Scale slider dragged — read track, mirror factor into chip,
        // forward track to the tool.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE => {
            let track = host
                .store()
                .slider(ids::UPS_SCALE)
                .map(|(_, v)| v)
                .unwrap_or(0.0);
            host.store_mut()
                .set_number_value(ids::UPS_SCALE_NUM, slider_to_scale(track) as f64);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    ids::UPS_SCALE,
                    track as f64,
                )));
            true
        }
        // Scale chip committed (Enter / blur / drag-scrub) — read the
        // natural factor, clamp into the supported range, mirror the
        // clamped factor back into the chip + the projected track into
        // the slider, then forward the track to the tool.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE_NUM => {
            let raw_factor = host
                .store()
                .number_value(ids::UPS_SCALE_NUM)
                .unwrap_or(ph2d_tool_upscale::params::DEFAULT_SCALE_FACTOR as f64)
                as f32;
            let factor = raw_factor.clamp(MIN_SCALE_FACTOR, SCALE_FULL_SCALE);
            let track = scale_to_slider(factor);
            // If the user typed an out-of-range number ("999"), normalize
            // the chip back to the clamped value so the displayed buffer
            // and the slider thumb agree.
            if (factor - raw_factor).abs() > f32::EPSILON {
                host.store_mut()
                    .set_number_value(ids::UPS_SCALE_NUM, factor as f64);
            }
            if let Some(InteractiveState::Slider { value, .. }) =
                host.store_mut().get_mut(ids::UPS_SCALE)
            {
                *value = track;
            }
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    ids::UPS_SCALE,
                    track as f64,
                )));
            true
        }
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
