//! Upscale panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-C + Widget Gallery convention (DIRETRIZ §4.2): the
//! panel does NOT own the semantic mapping (slider track → factor,
//! algorithm-segment NodeId → `UpscaleAlgorithm`) — that lives in
//! `UpscaleTool::handle_panel_event` over in `ph2d-tool-upscale`.
//! Here we just classify each `WidgetEvent` into a tool-agnostic
//! [`PanelEvent`], push it through `EditorAction::ToolPanelEvent`,
//! and let the action-bus drain in the shell call
//! `Tool::handle_panel_event` on the active tool. Cancel maps to
//! `EditorAction::CancelActiveTool`.
//!
//! Slider ↔ chip mirror is the dispatch's job, NOT the panel's:
//! `populate` calls `link_slider_number(UPS_SCALE, UPS_SCALE_NUM)`
//! so chip and slider share the same `0..1` track storage. Both
//! `ValueChanged(slider_id)` and `ValueChanged(chip_id)` flow through
//! this forwarder; we read the slider's stored track and forward it
//! as the tool's `UPS_SCALE` event (the chip event would carry the
//! same value via the mirror).

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
    if let WidgetEvent::Click(id) = ev
        && id == ids::UPS_TITLE_COLOR
    {
        let seed = host
            .store()
            .widget_color(id)
            .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral seed
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }
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
        // Scale slider OR chip — both store the canonical track via
        // `link_slider_number`. Forward the slider's `value` (track
        // `0..1`); the tool projects to a factor via
        // `slider_to_scale`.
        WidgetEvent::ValueChanged(id) if id == ids::UPS_SCALE || id == ids::UPS_SCALE_NUM => {
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
