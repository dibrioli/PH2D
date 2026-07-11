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

/// The transport + "+Track" buttons (Click → `PanelEvent::Click`; the shell
/// maps transport ids to a Playhead command and "+Track" ids to a Bind of the
/// selected sprite).
fn is_button(id: NodeId) -> bool {
    id == ids::TIMELINE_PLAY
        || id == ids::TIMELINE_ADD_MARKER
        || id == ids::TIMELINE_GO_START
        || id == ids::TIMELINE_GO_END
        || id == ids::TIMELINE_PREV_FRAME
        || id == ids::TIMELINE_NEXT_FRAME
        || ids::ADDPROP_BUTTONS.iter().any(|(bid, _)| *bid == id)
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
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        // Close (X) — hide the panel (mirror of the other docked panels).
        WidgetEvent::Click(id) if id == ids::TIMELINE_CLOSE => {
            host.set_panel_visible(TimelinePanel::ID, false);
            EventOutcome::Consumed
        }
        // Ruler scrub: the slider value (0..1 over the visible span) maps back to
        // an absolute time via the span `paint` stored; forward it as a Scrub.
        WidgetEvent::ValueChanged(id) if id == ids::TIMELINE_RULER => {
            let v = host
                .store()
                .slider(id)
                .map(|(_, v)| f64::from(v))
                .unwrap_or(0.0);
            let time = state.view_start_s + v * state.view_span_s;
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::SetValue(
                    id, time,
                )));
            EventOutcome::Consumed
        }
        // Scrollbar drag: dispatch's vertical slider reads `1.0` at the TOP of
        // its track, so the scroll fraction is `1 - v` (panel-local; no intent).
        WidgetEvent::ValueChanged(id) if id == ids::TIMELINE_SCROLLBAR => {
            let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
            state.scroll_y = crate::scrollbar::value_to_fraction(v) * state.scroll_max;
            EventOutcome::Consumed
        }
        // "+Track" opens/closes the property dropdown (panel-local; no intent).
        WidgetEvent::Click(id) if id == ids::TIMELINE_ADD_TRACK => {
            state.add_track_open = !state.add_track_open;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if is_button(id) => {
            // Picking a property from the dropdown closes it.
            if ids::ADDPROP_BUTTONS.iter().any(|(bid, _)| *bid == id) {
                state.add_track_open = false;
            }
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
        // Speed-graph view toggle (W5) — panel-local view state, NOT a document
        // command: flip it here (no bus/intent), like the +Track dropdown. The
        // transport bar mirrors `speed_view` back into the store's switch each
        // paint, so the painted toggle follows this flip. Any in-flight band
        // gesture maps its pointer through the OLD view's value range — drop it
        // (and close its undo bracket) rather than let it resolve as garbage
        // (mirror of the hide-panel cleanup in `paint`).
        WidgetEvent::Toggled(id) if id == ids::TIMELINE_SPEED => {
            state.speed_view = !state.speed_view;
            let handle = state.handle_drag.take().is_some();
            let anchor = state.anchor_drag.take().is_some();
            if handle || anchor {
                crate::state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Toggled(id) if is_toggle(id) => {
            let on = host.store().toggle(id).map(|(_, on)| on).unwrap_or(false);
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::Toggle(id, on)));
            EventOutcome::Consumed
        }
        // Marker rename field (W4.T3). Enter → Submit, click-away → Blur both
        // commit (the `take` inside makes the Enter→Submit+Blur pair idempotent);
        // Esc → Cancel abandons it.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id)
            if id == ids::TIMELINE_MARKER_RENAME_INPUT =>
        {
            crate::marker_rename::commit(state, host.store());
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::TIMELINE_MARKER_RENAME_INPUT => {
            crate::marker_rename::cancel(state);
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
