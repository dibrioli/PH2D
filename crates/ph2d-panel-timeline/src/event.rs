//! Timeline panel event router (W2.E2).
//!
//! The transport controls are document commands, not tool edits — the panel
//! translates each `WidgetEvent` into an [`EditorAction::TimelinePanelEvent`]
//! carrying a tool-agnostic [`PanelEvent`] (NodeId + payload); the shell drains
//! it, maps the id to a `ph2d_timeline::TimelineIntent`, and applies it (see
//! `render_loop::timeline_bridge::intent_for_transport`). The close (X) button
//! hides the panel directly through the host.

use crate::ids;
use crate::{TimelinePanel, state::TimelinePanelState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;

/// The three transport buttons (Click → `PanelEvent::Click`).
fn is_button(id: NodeId) -> bool {
    id == ids::TIMELINE_PLAY || id == ids::TIMELINE_PREV_FRAME || id == ids::TIMELINE_NEXT_FRAME
}

/// The two transport chips (ValueChanged → `PanelEvent::SetValue`).
fn is_chip(id: NodeId) -> bool {
    id == ids::TIMELINE_TIME_NUM || id == ids::TIMELINE_FRAME_NUM
}

/// The three transport toggles (Toggled → `PanelEvent::Toggle`).
fn is_toggle(id: NodeId) -> bool {
    id == ids::TIMELINE_LOOP || id == ids::TIMELINE_AUTOKEY || id == ids::TIMELINE_SNAP
}

pub(crate) fn apply_event(
    _state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        // Close (X) — hide the panel (mirror of the other docked panels).
        WidgetEvent::Click(id) if id == ids::TIMELINE_CLOSE => {
            host.set_panel_visible(TimelinePanel::ID, false);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if is_button(id) => {
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::Click(id)));
            EventOutcome::Consumed
        }
        WidgetEvent::ValueChanged(id) if is_chip(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::SetValue(
                    id, v,
                )));
            EventOutcome::Consumed
        }
        WidgetEvent::Toggled(id) if is_toggle(id) => {
            let on = host.store().toggle(id).map(|(_, on)| on).unwrap_or(false);
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::Toggle(id, on)));
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
