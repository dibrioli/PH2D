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
//! The panel still owns the slider ↔ chip MIRROR (the chip's stored
//! number value follows a slider drag, and the slider's stored value
//! follows a chip commit) — that is UI-state-local to the widget store,
//! not authoritative tool state.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_padding::params::{px_to_slider, slider_to_px};

use crate::state::PaddingPanelState;

/// Chip id paired with an edge slider id.
fn chip_for_slider(slider: NodeId) -> Option<NodeId> {
    if slider == ids::PAD_TOP {
        Some(ids::PAD_TOP_NUM)
    } else if slider == ids::PAD_RIGHT {
        Some(ids::PAD_RIGHT_NUM)
    } else if slider == ids::PAD_BOTTOM {
        Some(ids::PAD_BOTTOM_NUM)
    } else if slider == ids::PAD_LEFT {
        Some(ids::PAD_LEFT_NUM)
    } else {
        None
    }
}

/// Slider id paired with an edge px chip id.
fn slider_for_chip(chip: NodeId) -> Option<NodeId> {
    if chip == ids::PAD_TOP_NUM {
        Some(ids::PAD_TOP)
    } else if chip == ids::PAD_RIGHT_NUM {
        Some(ids::PAD_RIGHT)
    } else if chip == ids::PAD_BOTTOM_NUM {
        Some(ids::PAD_BOTTOM)
    } else if chip == ids::PAD_LEFT_NUM {
        Some(ids::PAD_LEFT)
    } else {
        None
    }
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
        // Edge slider dragged — read the live track, forward it as-is to
        // the tool (which maps to px), and mirror the px into the paired
        // chip's stored value so the chip's painted number matches the
        // slider thumb in real time.
        WidgetEvent::ValueChanged(id) if chip_for_slider(id).is_some() => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    track as f64,
                )));
            if let Some(chip) = chip_for_slider(id) {
                host.store_mut()
                    .set_number_value(chip, slider_to_px(track) as f64);
            }
            true
        }
        // Edge px chip edited (keyboard commit or drag-scrub) — read the
        // px value, forward it as-is to the tool, and mirror the
        // normalized track onto the paired slider's stored value so the
        // thumb follows the chip.
        WidgetEvent::ValueChanged(id) if slider_for_chip(id).is_some() => {
            let px = host.store().number_value(id).unwrap_or(0.0).round() as i32;
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, px as f64,
                )));
            if let Some(slider) = slider_for_chip(id)
                && let Some(InteractiveState::Slider { value, .. }) =
                    host.store_mut().get_mut(slider)
            {
                *value = px_to_slider(px);
            }
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
