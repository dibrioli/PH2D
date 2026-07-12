//! Waypoint gestures (Motion Nodes F2, doc 44) — split out of `interact` for the panel's
//! LOC cap. Declared there as a `#[path]` sibling, so `super` is `interact`.
//!
//! ## The gestures, and why these
//!
//! - **Double-click a wire → add a point there.** The wire is the thing you want to bend, so
//!   the wire is what you click. (Nuke puts a dot on a wire with a modifier-click; Blender
//!   reroutes with a menu. A double-click needs no modifier to remember and cannot be
//!   confused with the alt-press that already *deletes* a wire.)
//! - **Drag a point → move it.** One undo step for the gesture, bracketed like a node drag.
//! - **Double-click a point → remove it.** The same gesture that made it, on the thing it
//!   made: nothing new to learn, and no third modifier.

use super::{View, push_intent};
use crate::route;
use crate::snapshot::{GraphIntent, GraphViewSnapshot};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::interaction::{GesturePhase, GraphGesture};
use ph2d_editor_core::zones::Rect;

/// A double-click on a wire: add a routing point **where it was clicked**, in the leg of the
/// route it was dropped on (`route::insert_index` — a naive push would tie the wire in a
/// knot, sending it out to the last point and back).
pub(super) fn add_on_wire(
    state: &MotionGraphPanelState,
    g: GraphGesture,
    rect: Rect,
    snap: &GraphViewSnapshot,
    to_node: u32,
    to_port: u16,
) {
    let view = View::new(rect, state.view);
    let Some(e) = snap
        .edges
        .iter()
        .find(|e| e.to_node == to_node && e.to_port == to_port && !e.delayed)
    else {
        return; // a `pre` edge has no spline, so it can hold no routing
    };
    let Some(r) = route::route(snap, e, &view) else {
        return;
    };
    let index = route::insert_index(&r, (g.x, g.y));
    let (x, y) = view.graph(g.x, g.y);
    push_intent(GraphIntent::AddWaypoint {
        to_node,
        to_port,
        index,
        x,
        y,
    });
}

/// The waypoint handle's own gestures: **drag** moves it (one undo step, bracketed on the
/// first real movement so a stray click mints nothing), **double-click** removes it.
pub(super) fn apply_handle(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    rect: Rect,
    to_node: u32,
    to_port: u16,
    index: usize,
) {
    match g.phase {
        GesturePhase::DoubleClick => {
            push_intent(GraphIntent::RemoveWaypoint {
                to_node,
                to_port,
                index,
            });
            state.interaction = Interaction::Idle;
        }
        GesturePhase::Begin => {
            state.interaction = Interaction::DragWaypoint {
                to_node,
                to_port,
                index,
                last: (g.x, g.y),
                started: false,
            };
        }
        GesturePhase::Update => {
            let view = View::new(rect, state.view);
            if let Interaction::DragWaypoint {
                to_node,
                to_port,
                index,
                last,
                started,
            } = &mut state.interaction
            {
                let (dx, dy) = ((g.x - last.0) / view.zoom, (g.y - last.1) / view.zoom);
                *last = (g.x, g.y);
                if dx == 0.0 && dy == 0.0 {
                    return; // no movement, no undo bracket, no intent
                }
                if !*started {
                    *started = true;
                    push_intent(GraphIntent::BeginDrag);
                }
                push_intent(GraphIntent::MoveWaypoint {
                    to_node: *to_node,
                    to_port: *to_port,
                    index: *index,
                    dx,
                    dy,
                });
            }
        }
        GesturePhase::End | GesturePhase::Click => {
            if let Interaction::DragWaypoint { started: true, .. } = state.interaction {
                push_intent(GraphIntent::EndDrag);
            }
            state.interaction = Interaction::Idle;
        }
    }
}
