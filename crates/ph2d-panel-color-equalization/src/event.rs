//! Color Equalization panel `apply_event` — thin forwarder.
//!
//! Mirrors the canonical "Speed slider" pattern from the widget gallery
//! showcase: chip + slider share `0..1` and `link_slider_number` glues
//! them — the dispatch handles drag, clamp, and chip↔slider mirror for
//! free. Both `ValueChanged(slider_id)` and `ValueChanged(chip_id)`
//! flow through here; we forward the slider's normalized track value
//! to the tool (whose `params.rs` owns the `0..1 → natural unit`
//! projection). Cancel deactivates the tool.

use crate::ColorEqualizationPanelState;
use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

/// Pair a row's widget id (slider OR chip) → its slider id, so the
/// host always receives `PanelEvent::SetValue(slider_id, track)` with
/// the canonical slider NodeId, no matter whether the user dragged the
/// slider or scrubbed/typed the chip.
fn slider_for_widget(id: NodeId) -> Option<NodeId> {
    if id == ids::CEQ_CLIP_LIMIT || id == ids::CEQ_CLIP_LIMIT_NUM {
        Some(ids::CEQ_CLIP_LIMIT)
    } else if id == ids::CEQ_TILE_GRID || id == ids::CEQ_TILE_GRID_NUM {
        Some(ids::CEQ_TILE_GRID)
    } else if id == ids::CEQ_BRIGHTNESS || id == ids::CEQ_BRIGHTNESS_NUM {
        Some(ids::CEQ_BRIGHTNESS)
    } else if id == ids::CEQ_CONTRAST || id == ids::CEQ_CONTRAST_NUM {
        Some(ids::CEQ_CONTRAST)
    } else if id == ids::CEQ_SATURATION || id == ids::CEQ_SATURATION_NUM {
        Some(ids::CEQ_SATURATION)
    } else {
        None
    }
}

pub(crate) fn apply_event(
    _state: &mut ColorEqualizationPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Slider drag OR chip scrub/commit — both end up with the
        // slider's `value` (0..1) holding the canonical track via
        // `link_slider_number`. Forward it as the tool's `_SLIDER`
        // variant so `params::apply_ui_edit` does the 0..1 → natural
        // unit projection in one place.
        WidgetEvent::ValueChanged(id) if slider_for_widget(id).is_some() => {
            let slider_id = slider_for_widget(id).unwrap();
            let track = host
                .store()
                .slider(slider_id)
                .map(|(_, v)| v)
                .unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    slider_id,
                    track as f64,
                )));
            true
        }
        WidgetEvent::Click(id) if id == ids::CEQ_AUTO_WB => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) if id == ids::CEQ_APPLY => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) if id == ids::CEQ_CANCEL => {
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
