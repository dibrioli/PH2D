//! Gesture interpretation (Motion Nodes M1.E4/E5, Phase 1b-1). Drains the
//! `GraphSurface` channel the M0 dispatch fills (pointer gestures + anchored zoom
//! + graph keys) and turns it into ephemeral view/selection/drag state and — for
//! doc mutations only — [`GraphIntent`]s the shell applies.
//!
//! Coverage here: pan (drag on empty canvas), anchored wheel zoom, F = fit,
//! Esc = deselect, click/shift-select nodes, and multi-drag (one `MoveNodes`
//! undo at End). Connect / add-node / delete land in Phase 1b-2.

use crate::snapshot::{GraphIntent, push_intent};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{GesturePhase, GraphGesture, GraphHitKind, GraphKey, GraphZoom};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;

/// Drain this frame's graph input and fold it into `state` (+ push doc intents).
/// Call before drawing so the render reflects the latest gestures.
pub(crate) fn process(state: &mut MotionGraphPanelState, ctx: &mut PaintCtx, rect: Rect) {
    let panel = ids::MOTION_GRAPH_PANEL;

    if let Some(z) = ctx.host.store_mut().take_graph_zoom(panel) {
        apply_zoom(state, rect, z);
    }
    let keys: Vec<GraphKey> = ctx.host.store_mut().drain_graph_keys().collect();
    for k in keys {
        apply_key(state, k);
    }
    let gestures: Vec<GraphGesture> = ctx.host.store_mut().drain_graph_gestures().collect();
    for g in gestures {
        apply_gesture(state, g);
    }
}

/// Anchored zoom: keep the cursor's graph point fixed while scaling.
fn apply_zoom(state: &mut MotionGraphPanelState, rect: Rect, z: GraphZoom) {
    let old = state.view.zoom;
    let new = (old * (z.delta / 240.0).exp()).clamp(0.2, 2.5);
    let f = new / old;
    // screen = base + pan + graph*zoom ⇒ hold `anchor` ⇒ pan' = (anchor-base)(1-f) + pan*f.
    state.view.pan_x = (z.anchor_x - rect.x) * (1.0 - f) + state.view.pan_x * f;
    state.view.pan_y = (z.anchor_y - rect.y) * (1.0 - f) + state.view.pan_y * f;
    state.view.zoom = new;
}

fn apply_key(state: &mut MotionGraphPanelState, k: GraphKey) {
    match k {
        // Re-fit on the next paint (the draw pass owns the fit math).
        GraphKey::Fit => state.fitted = false,
        GraphKey::Escape => {
            state.selected.clear();
            state.interaction = Interaction::Idle;
        }
        // Delete / add / duplicate / knife / probe land in Phase 1b-2.
        _ => {}
    }
}

fn apply_gesture(state: &mut MotionGraphPanelState, g: GraphGesture) {
    match g.kind {
        GraphHitKind::Background => match g.phase {
            GesturePhase::Begin => state.interaction = Interaction::Pan { last: (g.x, g.y) },
            GesturePhase::Update => {
                if let Interaction::Pan { last } = &mut state.interaction {
                    state.view.pan_x += g.x - last.0;
                    state.view.pan_y += g.y - last.1;
                    *last = (g.x, g.y);
                }
            }
            // A tap on empty canvas clears the selection; any release ends the pan.
            GesturePhase::Click => {
                state.selected.clear();
                state.interaction = Interaction::Idle;
            }
            GesturePhase::End | GesturePhase::DoubleClick => {
                state.interaction = Interaction::Idle;
            }
        },
        GraphHitKind::Node { node } => {
            let node = node as u32;
            match g.phase {
                GesturePhase::Begin => {
                    select_on_press(state, node, g.mods.shift);
                    state.interaction = Interaction::DragNodes {
                        nodes: state.selected.iter().copied().collect(),
                        last: (g.x, g.y),
                        graph_dx: 0.0,
                        graph_dy: 0.0,
                    };
                }
                GesturePhase::Update => {
                    if let Interaction::DragNodes {
                        last,
                        graph_dx,
                        graph_dy,
                        ..
                    } = &mut state.interaction
                    {
                        *graph_dx += (g.x - last.0) / state.view.zoom;
                        *graph_dy += (g.y - last.1) / state.view.zoom;
                        *last = (g.x, g.y);
                    }
                }
                GesturePhase::End => {
                    if let Interaction::DragNodes {
                        nodes,
                        graph_dx,
                        graph_dy,
                        ..
                    } = std::mem::take(&mut state.interaction)
                        && (graph_dx != 0.0 || graph_dy != 0.0)
                        && !nodes.is_empty()
                    {
                        push_intent(GraphIntent::MoveNodes {
                            nodes,
                            dx: graph_dx,
                            dy: graph_dy,
                        });
                    }
                }
                GesturePhase::Click | GesturePhase::DoubleClick => {
                    state.interaction = Interaction::Idle;
                }
            }
        }
        // Sockets / wires / backdrops interact in Phase 1b-2.
        _ => {}
    }
}

/// Selection on node press: plain click selects only this node (unless it is
/// already part of the selection — then keep it, so a multi-drag works);
/// Shift toggles it into/out of the selection.
fn select_on_press(state: &mut MotionGraphPanelState, node: u32, shift: bool) {
    if shift {
        if !state.selected.insert(node) {
            state.selected.remove(&node);
        }
    } else if !state.selected.contains(&node) {
        state.selected.clear();
        state.selected.insert(node);
    }
}
