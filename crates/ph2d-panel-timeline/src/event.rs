//! Timeline panel event router (W2.E2).
//!
//! The transport controls are document commands, not tool edits — the panel
//! translates each `WidgetEvent` into an [`EditorAction::TimelinePanelEvent`]
//! carrying a tool-agnostic [`PanelEvent`] (NodeId + payload); the shell drains
//! it, maps the id to a `ph2d_timeline::TimelineIntent`, and applies it (see
//! `render_loop::timeline_bridge::intent_for_transport`). The close (X) button
//! hides the panel directly through the host.

use crate::ids;
use crate::state;
use crate::{TimelinePanel, state::TimelinePanelState};
use ph2d_a11y::NodeId;
/// How long a strip of an EMPTY clip is: a clip with no keys has no duration,
/// and a strip of zero seconds paints as nothing and cannot be grabbed to fix.
const MIN_NEW_STRIP_S: f64 = 1.0;

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_timeline::TimelineIntent;

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

/// The transport toggles routed to the shell (Toggled → `PanelEvent::Toggle`).
/// `TIMELINE_SPEED` is deliberately absent — it is a panel-local VIEW toggle,
/// handled in `apply_event` without reaching the shell.
fn is_toggle(id: NodeId) -> bool {
    id == ids::TIMELINE_LOOP
        || id == ids::TIMELINE_PINGPONG
        || id == ids::TIMELINE_AUTOKEY
        || id == ids::TIMELINE_RECORD
        || id == ids::TIMELINE_SNAP
}

pub(crate) fn apply_event(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // The clip stack's own chrome answers first (see `stack_click`) — lifted out
    // because `apply_event` is at its LOC cap, and because those three controls
    // speak to the STACK rather than to the sheet.
    if let WidgetEvent::Click(id) = ev
        && let Some(out) = stack_click(id)
    {
        return out;
    }
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
        // Track-row context menu: Delete Track. The Down that preceded this
        // Click already CLOSED the menu, parking the request — read
        // `context_menu().or_else(last_context_menu())` (the same gotcha the
        // presets menu documents; reading only the open one ships a menu that
        // does nothing). The request carries the raw `AnimTarget`; the snapshot
        // row resolves it back to the `(entity, prop)` the Unbind intent needs.
        // A row gone from the snapshot since the menu opened (deleted
        // meanwhile) resolves to nothing — the action expires with its target.
        WidgetEvent::Click(id) if id == ids::CTX_MENU_TL_DELETE_TRACK => {
            use ph2d_editor_core::interaction::ContextMenuKind;
            let req = host
                .store()
                .context_menu()
                .or_else(|| host.store().last_context_menu());
            if let Some(ContextMenuKind::TimelineTrack { target }) = req.map(|r| r.kind) {
                if let Some(track) = crate::state::current_snapshot()
                    .tracks
                    .iter()
                    .find(|t| t.target.get() == target)
                {
                    crate::state::push_intent(ph2d_timeline::TimelineIntent::Unbind {
                        entity: track.entity,
                        prop: track.prop,
                    });
                }
                host.store_mut().close_context_menu();
                // Spent: a later stray Click on this id must not delete again.
                host.store_mut().consume_last_context_menu();
            }
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

        // ── Clip selector (W5) ──────────────────────────────────────────────
        // Picking a clip from the open list. The store's `selected_index` is set
        // too, so the chip reads right on the SAME frame — the document round-trip
        // only lands on the next one.
        WidgetEvent::Click(id) if ids::TIMELINE_CLIP_OPT.contains(&id) => {
            if let Some(index) = ids::TIMELINE_CLIP_OPT.iter().position(|&o| o == id) {
                state::push_intent(TimelineIntent::SetActiveClip { index });
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(ids::TIMELINE_CLIP_DD)
                {
                    *open = false;
                    *selected_index = Some(index);
                }
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_ADD_CLIP => {
            state::push_intent(TimelineIntent::AddClip);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_RENAME_CLIP => {
            crate::clip_rename::open(state, &state::current_snapshot());
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_DELETE_CLIP => {
            // The SECOND barrier: the paint does not even hit-register the trash
            // while a single clip remains, but a dimmed control that still
            // dispatches is precisely the bug that guard is for — so refuse here
            // too, and let the document refuse a third time
            // ([[feedback_disabled_button_still_dispatches]]).
            let snap = state::current_snapshot();
            if snap.clips.len() > 1 {
                state::push_intent(TimelineIntent::DeleteClip {
                    index: snap.active_clip,
                });
            }
            EventOutcome::Consumed
        }
        // Clip rename field — same Enter/click-away/Esc contract as the marker's.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id)
            if id == ids::TIMELINE_CLIP_RENAME_INPUT =>
        {
            crate::clip_rename::commit(state, host.store());
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::TIMELINE_CLIP_RENAME_INPUT => {
            crate::clip_rename::cancel(state);
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}

/// The clip stack's chrome (ADR-0115): "+ Lane", and each lane's mute and
/// "+ Strip". `None` means "not one of ours" — the caller falls through.
fn stack_click(id: ph2d_editor_core::NodeId) -> Option<EventOutcome> {
    if id == ids::TIMELINE_ADD_LANE {
        crate::state::push_intent(ph2d_timeline::TimelineIntent::AddLane);
        return Some(EventOutcome::Consumed);
    }
    if let Some(lane) = ids::TIMELINE_LANE_MUTE.iter().position(|&b| b == id) {
        // A lane the snapshot no longer has (deleted since the paint that
        // registered this button) raises nothing: the action expires with its
        // target, exactly as Delete Track's does.
        if let Some(v) = crate::state::current_snapshot().lanes.get(lane) {
            crate::state::push_intent(ph2d_timeline::TimelineIntent::SetLaneMuted {
                lane,
                muted: !v.muted,
            });
        }
        return Some(EventOutcome::Consumed);
    }
    if let Some(lane) = ids::TIMELINE_LANE_ADD_STRIP.iter().position(|&b| b == id) {
        let snap = crate::state::current_snapshot();
        if lane < snap.lanes.len() {
            // The ACTIVE clip, dropped AT THE PLAYHEAD — the clip you are looking
            // at, where you are looking.
            let t = snap.time_seconds.max(0.0);
            crate::state::push_intent(ph2d_timeline::TimelineIntent::AddStrip {
                lane,
                clip: snap.active_clip,
                t_start: t,
                t_end: t + snap.duration_seconds.max(MIN_NEW_STRIP_S),
            });
        }
        return Some(EventOutcome::Consumed);
    }
    None
}
