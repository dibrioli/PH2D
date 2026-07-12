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
/// How long a strip of an EMPTY clip is: a clip with no keys has no duration, and
/// a strip of zero seconds paints as nothing and cannot be grabbed to fix.
///
/// It floors ONLY the empty case. Padding a short clip's span to a second used to
/// look harmless and was not: a 0.4 s clip in a 1 s box is a strip playing at 0.4x
/// before anyone asked it to (`slice == span * speed`), and the first stretch would
/// have snapped its rate to match.
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
    // The clip stack answers first, through ONE door (`stack_event`): its chrome,
    // its two menus and its weight field all speak to the STACK rather than to the
    // sheet, and folding them into one guard is also what keeps `apply_event` under
    // its LOC cap — the cap noticing that the stack had grown into its own subject.
    if let Some(out) = stack_event(state, ev, host) {
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

/// Everything the clip stack answers: its chrome, its two right-click menus, and
/// the lane weight field. `None` means "not ours" — the caller falls through to
/// the sheet.
fn stack_event(
    state: &mut TimelinePanelState,
    ev: WidgetEvent,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    match ev {
        WidgetEvent::Click(id) => stack_click(id)
            .or_else(|| strip_menu_click(id, host))
            .or_else(|| lane_menu_click(id, host)),
        // Grabbing (or clicking into) the weight field OPENS the undo bracket.
        //
        // Dispatch emits a `ValueChanged` for every Move of a number body-drag, and
        // each one, unbracketed, is its own atomic undo step: sliding the weight
        // across its range left dozens of Ctrl+Z steps behind it. Every other
        // document-mutating gesture in this panel brackets (`strip_drag`, `key_drag`,
        // `anchor_drag`); this was the one that did not.
        WidgetEvent::Focus(id) => {
            let lane = ids::TIMELINE_LANE_WEIGHT.iter().position(|&w| w == id)?;
            if state.weight_edit.is_none() {
                state.weight_edit = Some(lane);
                state::push_intent(TimelineIntent::BeginEdit);
            }
            Some(EventOutcome::Consumed)
        }
        // …and letting go closes it. Dispatch guarantees the Blur: it fires on
        // pointer-up for a body drag and on click-away for a typed edit. A gesture
        // that changed nothing commits no step (`commit_if_changed`).
        WidgetEvent::Blur(id) | WidgetEvent::Submit(id)
            if ids::TIMELINE_LANE_WEIGHT.contains(&id) =>
        {
            if state.weight_edit.take().is_some() {
                state::push_intent(TimelineIntent::EndEdit);
            }
            Some(EventOutcome::Consumed)
        }
        // The lane weight is a bounded field, so its edit arrives as a ValueChanged.
        WidgetEvent::ValueChanged(id) => {
            let lane = ids::TIMELINE_LANE_WEIGHT.iter().position(|&w| w == id)?;
            // A lane gone from the snapshot addresses nothing — the field is
            // registered for all MAX_LANES, because the store is populated once.
            if lane < crate::state::current_snapshot().lanes.len() {
                let weight = host.store().number_value(id).unwrap_or(1.0);
                state::push_intent(TimelineIntent::SetLaneWeight { lane, weight });
            }
            Some(EventOutcome::Consumed)
        }
        _ => None,
    }
}

/// The lane's right-click menu (ADR-0115 B5): how it blends, and whether it stays.
///
/// Same contract as `strip_menu_click` — read the PARKED request, confirm the lane
/// still exists, spend the request. `Delete Lane` lives here rather than on the row
/// because the row has no width for a third button; that is a layout fact, not a
/// judgement about how often a lane gets deleted.
fn lane_menu_click(
    id: ph2d_editor_core::NodeId,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_timeline::LaneMode;

    if !ids::TIMELINE_LANE_MENU.iter().any(|(r, _, _)| *r == id) {
        return None;
    }
    let req = host
        .store()
        .context_menu()
        .or_else(|| host.store().last_context_menu());
    let Some(ContextMenuKind::TimelineLane { lane }) = req.map(|r| r.kind) else {
        return Some(EventOutcome::Consumed);
    };
    if lane < crate::state::current_snapshot().lanes.len() {
        let intent = if id == ids::CTX_MENU_TL_LANE_DELETE {
            TimelineIntent::RemoveLane { lane }
        } else {
            // The two modes. `Additive` is named explicitly and `Override` is the
            // fallback, so a row added to the table without an arm here lands on
            // Override — which the seam test refuses to let pass silently.
            let mode = if id == ids::CTX_MENU_TL_LANE_ADDITIVE {
                LaneMode::Additive
            } else {
                LaneMode::Override
            };
            TimelineIntent::SetLaneMode { lane, mode }
        };
        state::push_intent(intent);
    }
    host.store_mut().close_context_menu();
    host.store_mut().consume_last_context_menu();
    Some(EventOutcome::Consumed)
}

/// The strip's right-click menu (ADR-0115 B6). `None` means "not one of ours".
///
/// Same two gotchas the track menu documents, for the same reasons: the Down that
/// preceded this Click already CLOSED the menu (so read
/// `context_menu().or_else(last_context_menu())` — reading only the open one
/// ships a menu that does nothing), and the request is CONSUMED after it lands
/// (so a later stray Click on the id cannot duplicate the strip a second time).
///
/// The request names the strip by its stable id, and the snapshot is asked to
/// confirm it still exists: a strip deleted between the menu opening and the row
/// being clicked resolves to nothing. The action expires with its target.
fn strip_menu_click(
    id: ph2d_editor_core::NodeId,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_timeline::{StripId, StripLoop};

    if !ids::TIMELINE_STRIP_MENU.iter().any(|(r, _, _)| *r == id) {
        return None;
    }
    let req = host
        .store()
        .context_menu()
        .or_else(|| host.store().last_context_menu());
    let Some(ContextMenuKind::TimelineStrip { lane, strip }) = req.map(|r| r.kind) else {
        return Some(EventOutcome::Consumed);
    };
    let id_ = StripId(strip);
    let snap = crate::state::current_snapshot();
    let live = snap
        .lanes
        .get(lane)
        .is_some_and(|l| l.strips.iter().any(|s| s.id == id_));
    if live {
        let intent = if id == ids::CTX_MENU_TL_STRIP_DUPLICATE {
            TimelineIntent::DuplicateStrip { lane, id: id_ }
        } else if id == ids::CTX_MENU_TL_STRIP_DELETE {
            TimelineIntent::RemoveStrip { lane, id: id_ }
        } else if id == ids::CTX_MENU_TL_STRIP_RESET_SPEED {
            TimelineIntent::SetStripSpeed {
                lane,
                id: id_,
                speed: 1.0,
            }
        } else {
            // The three source modes. Exhaustive over what remains of the table —
            // and if a row is ever added without landing here, `strip_menu_click`
            // would silently set Once, so the seam test proves each row raises the
            // intent it names rather than merely raising SOMETHING.
            let loop_mode = if id == ids::CTX_MENU_TL_STRIP_LOOP {
                StripLoop::Loop
            } else if id == ids::CTX_MENU_TL_STRIP_PINGPONG {
                StripLoop::PingPong
            } else {
                StripLoop::Once
            };
            TimelineIntent::SetStripLoop {
                lane,
                id: id_,
                loop_mode,
            }
        };
        state::push_intent(intent);
    }
    host.store_mut().close_context_menu();
    host.store_mut().consume_last_context_menu();
    Some(EventOutcome::Consumed)
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
            let len = if snap.duration_seconds > 0.0 {
                snap.duration_seconds
            } else {
                MIN_NEW_STRIP_S
            };
            crate::state::push_intent(ph2d_timeline::TimelineIntent::AddStrip {
                lane,
                clip: snap.active_clip,
                t_start: t,
                t_end: t + len,
            });
        }
        return Some(EventOutcome::Consumed);
    }
    None
}
