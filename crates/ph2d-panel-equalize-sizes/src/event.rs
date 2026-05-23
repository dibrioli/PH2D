//! Equalize Sizes panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-C shape: the panel does not own any semantic mapping
//! (NodeId → `EqualizeSizesUiEdit` lives in
//! `EqualizeSizesTool::handle_panel_event`). Here we just classify each
//! `WidgetEvent` into a tool-agnostic [`PanelEvent`], push it through
//! `EditorAction::ToolPanelEvent`, and let the action-bus drain in the
//! shell call `Tool::handle_panel_event` on the active tool. Cancel maps
//! to `EditorAction::CancelActiveTool`.
//!
//! The grid-unit slider ↔ chip mirror is kept locally (UI-state-local in
//! the widget store): drag the slider → chip px follows; type in the
//! chip → slider track follows.

use crate::ids;
use crate::state::EqualizeSizesPanelState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_equalize_sizes::params::{grid_unit_to_slider, slider_to_grid_unit};

pub(crate) fn apply_event(
    _state: &mut EqualizeSizesPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Grid-unit slider dragged — read the track, forward as-is, then
        // mirror the corresponding px into the chip's stored value so
        // the chip's painted number tracks the thumb in real time.
        WidgetEvent::ValueChanged(id) if id == ids::EQS_GRID_UNIT => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    track as f64,
                )));
            host.store_mut()
                .set_number_value(ids::EQS_GRID_UNIT_NUM, slider_to_grid_unit(track) as f64);
            true
        }
        // Grid-unit chip edited — read the px, forward, mirror onto the
        // slider's stored track so the thumb follows.
        WidgetEvent::ValueChanged(id) if id == ids::EQS_GRID_UNIT_NUM => {
            let px = host.store().number_value(id).unwrap_or(1.0).round() as u32;
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, px as f64,
                )));
            if let Some(InteractiveState::Slider { value, .. }) =
                host.store_mut().get_mut(ids::EQS_GRID_UNIT)
            {
                *value = grid_unit_to_slider(px);
            }
            true
        }
        // Fixed-mode W/H chips — raw px, no mirror.
        WidgetEvent::ValueChanged(id) if id == ids::EQS_FIXED_W || id == ids::EQS_FIXED_H => {
            let px = host.store().number_value(id).unwrap_or(1.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, px)));
            true
        }
        // ── Plain click-style buttons (modes, algorithm, toggles, Apply) ──
        WidgetEvent::Click(id)
            if matches!(
                id,
                _ if id == ids::EQS_MODE_MAX
                    || id == ids::EQS_MODE_FIXED
                    || id == ids::EQS_MODE_GRID
                    || id == ids::EQS_UPSCALE_IF_SMALLER
                    || id == ids::EQS_RASTERIZE_AFTER
                    || id == ids::EQS_ALG_LANCZOS
                    || id == ids::EQS_ALG_NEAREST
                    || id == ids::EQS_ALG_XBR
                    || id == ids::EQS_APPLY
            ) =>
        {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Cancel — abandon + deactivate the tool.
        WidgetEvent::Click(id) if id == ids::EQS_CANCEL => {
            reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    }
}

fn reset_button(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
