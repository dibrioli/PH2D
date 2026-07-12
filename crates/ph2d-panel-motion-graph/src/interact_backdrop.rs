//! Backdrop gestures (Motion Nodes F2) — add / drag / resize the group regions, split out of
//! `interact` for the panel LOC cap. Declared there as a `#[path]` sibling, so `super` is
//! `interact`.

use super::{View, push_intent};
use crate::snapshot::{GraphIntent, GraphViewSnapshot};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::interaction::{GesturePhase, GraphGesture};
use ph2d_editor_core::zones::Rect;

/// The Backdrop chip: frame the current selection, or drop a default block at the
/// view centre when nothing is selected (Nuke's two behaviours from one button).
/// The panel computes the rect; the shell mints the id.
pub(super) fn add_backdrop(state: &MotionGraphPanelState, rect: Rect, snap: &GraphViewSnapshot) {
    let framed: Vec<&crate::snapshot::GraphNodeView> = snap
        .nodes
        .iter()
        .filter(|n| state.selected.contains(&n.id))
        .collect();
    let (x, y, w, h) = crate::backdrop::wrap_of(&framed).unwrap_or_else(|| {
        let view = View::new(rect, state.view);
        let (cx, cy) = view.graph(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        (
            cx - crate::backdrop::NEW_W * 0.5,
            cy - crate::backdrop::NEW_H * 0.5,
            crate::backdrop::NEW_W,
            crate::backdrop::NEW_H,
        )
    });
    push_intent(GraphIntent::AddBackdrop { x, y, w, h });
}

/// Header gestures: select on press, drag the group (region + framed nodes) as one
/// undo step. The framed set is captured at Begin — see `Interaction::DragBackdrop`.
pub(super) fn apply_backdrop(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    id: u32,
    snap: &GraphViewSnapshot,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.selected.clear(); // a backdrop and nodes are never co-selected
            state.selected_backdrop = Some(id);
            let nodes = snap
                .backdrops
                .iter()
                .find(|b| b.id == id)
                .map(|b| {
                    snap.nodes
                        .iter()
                        .filter(|n| crate::backdrop::frames_node(b, n))
                        .map(|n| n.id)
                        .collect()
                })
                .unwrap_or_default();
            state.interaction = Interaction::DragBackdrop {
                id,
                nodes,
                last: (g.x, g.y),
                started: false,
            };
        }
        GesturePhase::Update => {
            let zoom = state.view.zoom;
            if let Interaction::DragBackdrop {
                id,
                nodes,
                last,
                started,
            } = &mut state.interaction
            {
                let (dx, dy) = ((g.x - last.0) / zoom, (g.y - last.1) / zoom);
                *last = (g.x, g.y);
                if !*started {
                    push_intent(GraphIntent::BeginDrag);
                    *started = true;
                }
                push_intent(GraphIntent::MoveBackdrop { id: *id, dx, dy });
                if !nodes.is_empty() {
                    push_intent(GraphIntent::MoveNodes {
                        nodes: nodes.clone(),
                        dx,
                        dy,
                    });
                }
            }
        }
        // Release (a drag that moved, a tap that did not, or a double-tap): close
        // the undo bracket if one was opened. The selection made on press stays.
        GesturePhase::End | GesturePhase::Click | GesturePhase::DoubleClick => {
            if let Interaction::DragBackdrop { started: true, .. } = state.interaction {
                push_intent(GraphIntent::EndDrag);
            }
            state.interaction = Interaction::Idle;
        }
    }
}

/// Either bottom gripper: resize in place (the framed nodes do NOT move — a resize
/// changes what the region covers, which is the whole point). `left` is the corner
/// grabbed; the opposite edge stays anchored (shell-side).
pub(super) fn apply_backdrop_resize(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    id: u32,
    left: bool,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.selected.clear();
            state.selected_backdrop = Some(id);
            state.interaction = Interaction::ResizeBackdrop {
                id,
                left,
                last: (g.x, g.y),
                started: false,
            };
        }
        GesturePhase::Update => {
            let zoom = state.view.zoom;
            if let Interaction::ResizeBackdrop {
                id,
                left,
                last,
                started,
            } = &mut state.interaction
            {
                let (dx, dy) = ((g.x - last.0) / zoom, (g.y - last.1) / zoom);
                *last = (g.x, g.y);
                if !*started {
                    push_intent(GraphIntent::BeginDrag);
                    *started = true;
                }
                push_intent(GraphIntent::ResizeBackdrop {
                    id: *id,
                    left: *left,
                    dx,
                    dy,
                });
            }
        }
        GesturePhase::End | GesturePhase::Click | GesturePhase::DoubleClick => {
            if let Interaction::ResizeBackdrop { started: true, .. } = state.interaction {
                push_intent(GraphIntent::EndDrag);
            }
            state.interaction = Interaction::Idle;
        }
    }
}
