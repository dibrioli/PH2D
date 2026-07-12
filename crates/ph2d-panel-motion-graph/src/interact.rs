//! Gesture interpretation (Motion Nodes M1.E4–E7, Phase 1b). Drains the
//! `GraphSurface` channel the M0 dispatch fills (pointer gestures, anchored zoom,
//! graph keys) and turns it into ephemeral view/selection/drag/menu state, plus
//! the [`GraphIntent`]s the shell applies (doc mutations only).
//!
//! Coverage: pan (drag empty canvas), anchored wheel zoom, F = fit, Esc =
//! deselect / cancel, click/shift-select, multi-drag (one `MoveNodes` undo at
//! End), socket→socket **connect** (with a live compatibility ghost; the shell
//! validates for real), alt-press a wire = **disconnect**, R-press (anywhere) /
//! `A` = **add-node** menu, and Delete = **delete selection**.

use crate::geom::{self, View};
use crate::snapshot::{GraphIntent, GraphViewSnapshot, current_catalog, push_intent};
use crate::state::{AddMenu, Interaction, MotionGraphPanelState};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{
    GesturePhase, GraphGesture, GraphHitKind, GraphKey, GraphZoom,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;

/// Drain this frame's graph input and fold it into `state` (+ push doc intents).
/// Called before drawing so the render reflects the latest gestures. `snap` is
/// the snapshot `paint` already fetched — reused for socket/wire hit-testing.
pub(crate) fn process(
    state: &mut MotionGraphPanelState,
    ctx: &mut PaintCtx,
    rect: Rect,
    center: Rect,
    snap: &GraphViewSnapshot,
) {
    let panel = ids::MOTION_GRAPH_PANEL;

    if let Some(z) = ctx.host.store_mut().take_graph_zoom(panel) {
        apply_zoom(state, rect, z);
    }
    let keys: Vec<GraphKey> = ctx.host.store_mut().drain_graph_keys().collect();
    for k in keys {
        apply_key(state, k, rect);
    }
    let gestures: Vec<GraphGesture> = ctx.host.store_mut().drain_graph_gestures().collect();
    for g in gestures {
        apply_gesture(state, g, rect, center, snap);
    }
}

// Wheel-zoom tuning (canvas-interaction constants, not chrome tokens).
const ZOOM_WHEEL_DIV: f32 = 240.0; // LITERAL-PX-OK: wheel-notch → zoom-factor sensitivity divisor
const ZOOM_MIN: f32 = 0.2; // LITERAL-PX-OK: min graph zoom
const ZOOM_MAX: f32 = 2.5; // LITERAL-PX-OK: max graph zoom

/// Anchored zoom: keep the cursor's graph point fixed while scaling.
fn apply_zoom(state: &mut MotionGraphPanelState, rect: Rect, z: GraphZoom) {
    let old = state.view.zoom;
    let factor = (z.delta / ZOOM_WHEEL_DIV).exp();
    let new = (old * factor).clamp(ZOOM_MIN, ZOOM_MAX); // CLAMP-OK: const bounds, min<max, non-NaN
    let f = new / old;
    // screen = base + pan + graph*zoom ⇒ hold `anchor` ⇒ pan' = (anchor-base)(1-f) + pan*f.
    state.view.pan_x = (z.anchor_x - rect.x) * (1.0 - f) + state.view.pan_x * f;
    state.view.pan_y = (z.anchor_y - rect.y) * (1.0 - f) + state.view.pan_y * f;
    state.view.zoom = new;
}

fn apply_key(state: &mut MotionGraphPanelState, k: GraphKey, rect: Rect) {
    match k {
        // Re-fit on the next paint (the draw pass owns the fit math).
        GraphKey::Fit => state.fitted = false,
        GraphKey::Escape => {
            state.selected.clear();
            state.selected_backdrop = None;
            state.interaction = Interaction::Idle;
            state.add_menu = None;
        }
        // Delete the selection (orphan edges go with the nodes, shell-side).
        // Empty selection → no intent (idempotent against the double key
        // dispatch: M0 focus gate + the shell's cursor push). Node and backdrop
        // selection are mutually exclusive, so Delete is never ambiguous — and a
        // deleted backdrop takes nothing with it (it owns no nodes; it draws
        // around them).
        GraphKey::Delete => {
            if !state.selected.is_empty() {
                push_intent(GraphIntent::DeleteSelection {
                    nodes: state.selected.iter().copied().collect(),
                });
                state.selected.clear();
            } else if let Some(id) = state.selected_backdrop.take() {
                push_intent(GraphIntent::DeleteBackdrop { id });
            }
            state.interaction = Interaction::Idle;
        }
        // `A` opens the add-node menu at the canvas center (the keyboard verb
        // carries no cursor position). Idempotent: a second `A` (menu already
        // open) falls through to the no-op arm below.
        GraphKey::Add if state.add_menu.is_none() => {
            let center = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
            let spawn = View::new(rect, state.view).graph(center.0, center.1);
            state.add_menu = Some(AddMenu {
                screen: center,
                spawn,
            });
        }
        // Space — toggle transport play/pause (the shell owns the transport).
        GraphKey::TogglePlay => push_intent(GraphIntent::TogglePlay),
        // Duplicate / knife / probe land later.
        _ => {}
    }
}

fn apply_gesture(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    rect: Rect,
    center: Rect,
    snap: &GraphViewSnapshot,
) {
    // Right-button opens the add-node menu at the cursor — on the PRESS (Begin),
    // over ANY hit (background, node, socket, wire). Doing it on Begin (not the
    // release Click) makes it movement-independent: a right-click that drifts a
    // pixel is classified End by the dispatch, which would otherwise never open
    // (or would dismiss) the menu. All secondary phases are absorbed here so a
    // right-drag / right-release never pans, selects, or dismisses.
    if g.button == PointerButton::Secondary {
        if g.phase == GesturePhase::Begin {
            let spawn = View::new(rect, state.view).graph(g.x, g.y);
            state.add_menu = Some(AddMenu {
                screen: (g.x, g.y),
                spawn,
            });
            state.interaction = Interaction::Idle;
        }
        return;
    }
    match g.kind {
        GraphHitKind::Background => apply_background(state, g, rect),
        GraphHitKind::Node { node } => apply_node(state, g, node as u32),
        GraphHitKind::SocketOut { node, port } => {
            apply_socket_out(state, g, node as u32, port, rect, snap)
        }
        // A wire: alt + press removes the edge (identified by its unique target
        // input, decoded from the opaque handle). On Begin (the press), not the
        // release, so a click that drifts a pixel — classified End by the
        // dispatch — still deletes (same robustness as the R-press add-menu).
        // Plain presses on a wire are inert (fall through to the no-op arm).
        GraphHitKind::Wire { edge } if g.phase == GesturePhase::Begin && g.mods.alt => {
            let (to_node, to_port) = crate::paint::wire_target(edge);
            push_intent(GraphIntent::Disconnect { to_node, to_port });
        }
        // Split divider (E9): drag maps the pointer to a split fraction against
        // the full center band (scene `center` + graph `rect`). Begin/Update both
        // emit so the split tracks the cursor live; the shell clamps `t`.
        GraphHitKind::SplitDivider
            if matches!(g.phase, GesturePhase::Begin | GesturePhase::Update) =>
        {
            let vertical = rect.x > center.x + 0.5;
            let t = if vertical {
                (g.x - center.x) / (rect.x + rect.w - center.x).max(1.0)
            } else {
                (g.y - center.y) / (rect.y + rect.h - center.y).max(1.0)
            };
            push_intent(GraphIntent::SetSplit { t });
        }
        // Toolbar chips (E9): SplitH / SplitV flip orientation; Fit re-fits;
        // Backdrop frames the selection (F2).
        GraphHitKind::Chrome { id } if g.phase == GesturePhase::Click => match id {
            crate::paint_chrome::CHROME_SPLIT_H => {
                push_intent(GraphIntent::SetSplitVertical { vertical: false })
            }
            crate::paint_chrome::CHROME_SPLIT_V => {
                push_intent(GraphIntent::SetSplitVertical { vertical: true })
            }
            crate::paint_chrome::CHROME_FIT => state.fitted = false,
            crate::paint_chrome::CHROME_BACKDROP => add_backdrop(state, rect, snap),
            _ => {}
        },
        // A backdrop's header: select it (a backdrop and a node are never selected
        // together) and drag the whole group — the region plus every node it
        // frames, captured now (see `Interaction::DragBackdrop`).
        GraphHitKind::Backdrop { id } => apply_backdrop(state, g, id as u32, snap),
        GraphHitKind::BackdropResize { id } => apply_backdrop_resize(state, g, id as u32),
        // Input sockets (the reverse-drag of an occupied input) land later.
        _ => {}
    }
}

/// The Backdrop chip: frame the current selection, or drop a default block at the
/// view centre when nothing is selected (Nuke's two behaviours from one button).
/// The panel computes the rect; the shell mints the id.
fn add_backdrop(state: &MotionGraphPanelState, rect: Rect, snap: &GraphViewSnapshot) {
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
fn apply_backdrop(
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

/// The bottom-right gripper: resize in place (the framed nodes do NOT move — a
/// resize changes what the region covers, which is the whole point).
fn apply_backdrop_resize(state: &mut MotionGraphPanelState, g: GraphGesture, id: u32) {
    match g.phase {
        GesturePhase::Begin => {
            state.selected.clear();
            state.selected_backdrop = Some(id);
            state.interaction = Interaction::ResizeBackdrop {
                id,
                last: (g.x, g.y),
                started: false,
            };
        }
        GesturePhase::Update => {
            let zoom = state.view.zoom;
            if let Interaction::ResizeBackdrop { id, last, started } = &mut state.interaction {
                let (dw, dh) = ((g.x - last.0) / zoom, (g.y - last.1) / zoom);
                *last = (g.x, g.y);
                if !*started {
                    push_intent(GraphIntent::BeginDrag);
                    *started = true;
                }
                push_intent(GraphIntent::ResizeBackdrop { id: *id, dw, dh });
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

/// Empty-canvas gestures (primary button only — the secondary button is fully
/// handled in [`apply_gesture`]): pan, selection clear, and resolving the
/// add-node menu the right-press opened.
fn apply_background(state: &mut MotionGraphPanelState, g: GraphGesture, rect: Rect) {
    match g.phase {
        GesturePhase::Begin => {
            // A press while the menu is open consumes (no pan); the release
            // resolves it. Otherwise begin a pan.
            if state.add_menu.is_some() {
                state.interaction = Interaction::Idle;
            } else {
                state.interaction = Interaction::Pan { last: (g.x, g.y) };
            }
        }
        GesturePhase::Update => {
            if let Interaction::Pan { last } = &mut state.interaction {
                state.view.pan_x += g.x - last.0;
                state.view.pan_y += g.y - last.1;
                *last = (g.x, g.y);
            }
        }
        GesturePhase::Click => {
            if let Some(menu) = state.add_menu.take() {
                // A primary click while the menu is open closes it; a click on a
                // row also adds that node at the menu's spawn point.
                resolve_add_menu(&menu, rect, g.x, g.y);
            } else {
                // A plain tap on empty canvas clears the selection — including a
                // selected backdrop (the tap goes THROUGH its click-through body).
                state.selected.clear();
                state.selected_backdrop = None;
            }
            state.interaction = Interaction::Idle;
        }
        GesturePhase::End | GesturePhase::DoubleClick => {
            // A primary drag over empty canvas dismisses an open menu.
            state.add_menu = None;
            state.interaction = Interaction::Idle;
        }
    }
}

/// Resolve a primary click at `(x, y)` against the open add-menu: if it lands on
/// a catalog row, emit `AddNode` at the menu's graph-space spawn point.
fn resolve_add_menu(menu: &AddMenu, rect: Rect, x: f32, y: f32) {
    let catalog = current_catalog();
    let panel = geom::add_menu_panel(menu, catalog.len(), rect);
    for (i, c) in catalog.iter().enumerate() {
        if geom::add_menu_row(panel, i).contains(x, y) {
            push_intent(GraphIntent::AddNode {
                type_name: c.type_name,
                x: menu.spawn.0,
                y: menu.spawn.1,
            });
            return;
        }
    }
}

/// Node-body gestures: select on press, multi-drag with a live `MoveNodes`.
fn apply_node(state: &mut MotionGraphPanelState, g: GraphGesture, node: u32) {
    match g.phase {
        GesturePhase::Begin => {
            state.selected_backdrop = None; // one subject at a time (see the state docs)
            select_on_press(state, node, g.mods.shift);
            state.interaction = Interaction::DragNodes {
                nodes: state.selected.iter().copied().collect(),
                last: (g.x, g.y),
                started: false,
            };
        }
        GesturePhase::Update => {
            let zoom = state.view.zoom;
            if let Interaction::DragNodes {
                nodes,
                last,
                started,
            } = &mut state.interaction
            {
                let (dx, dy) = ((g.x - last.0) / zoom, (g.y - last.1) / zoom);
                *last = (g.x, g.y);
                if dx != 0.0 || dy != 0.0 {
                    if !*started {
                        push_intent(GraphIntent::BeginDrag);
                        *started = true;
                    }
                    // Applied live by the shell → the node tracks the cursor (no
                    // end-jump); one undo step for the whole drag.
                    push_intent(GraphIntent::MoveNodes {
                        nodes: nodes.clone(),
                        dx,
                        dy,
                    });
                }
            }
        }
        GesturePhase::End => {
            if let Interaction::DragNodes { started, .. } = std::mem::take(&mut state.interaction)
                && started
            {
                push_intent(GraphIntent::EndDrag);
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.interaction = Interaction::Idle;
        }
    }
}

/// Output-socket gestures: drag begins a wire; the ghost tracks the pointer and
/// snaps its validity to the hovered input; the drop emits `Connect`.
fn apply_socket_out(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    node: u32,
    port: u16,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.interaction = Interaction::DrawWire {
                from_node: node,
                from_port: port,
                cur: (g.x, g.y),
                target: None,
            };
        }
        GesturePhase::Update => {
            let view = View::new(rect, state.view);
            let target = target_socket(snap, &view, node, port, g.x, g.y);
            if let Interaction::DrawWire { cur, target: t, .. } = &mut state.interaction {
                *cur = (g.x, g.y);
                *t = target;
            }
        }
        GesturePhase::End => {
            if let Interaction::DrawWire {
                from_node,
                from_port,
                target,
                ..
            } = std::mem::take(&mut state.interaction)
                && let Some((to_node, to_port, _compat)) = target
            {
                // Emit regardless of the local compatibility flag — the shell is
                // the authority (cycle / occupied / typing / membrane) and raises
                // the refusal toast. Dropping in empty space (target None) is a
                // no-op (smart-connect is deferred).
                push_intent(GraphIntent::Connect {
                    from_node,
                    from_port,
                    to_node,
                    to_port,
                });
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.interaction = Interaction::Idle;
        }
    }
}

/// The input socket under `(x, y)`, with whether it is locally type-compatible
/// with the source output (domain + dim + clock — `connects_directly` minus the
/// membrane, which the shell checks). `None` when the pointer is over no input.
fn target_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    from_node: u32,
    from_port: u16,
    x: f32,
    y: f32,
) -> Option<(u32, u16, bool)> {
    let (to_node, to_port) = geom::hit_input_socket(snap, view, x, y)?;
    let out = snap
        .nodes
        .iter()
        .find(|n| n.id == from_node)
        .and_then(|n| n.outputs.get(from_port as usize));
    let inp = snap
        .nodes
        .iter()
        .find(|n| n.id == to_node)
        .and_then(|n| n.inputs.get(to_port as usize));
    let compat = match (out, inp) {
        (Some(o), Some(i)) => o.domain == i.domain && o.dim == i.dim && o.clock == i.clock,
        _ => false,
    };
    Some((to_node, to_port, compat))
}

/// Selection on node press: plain click selects only this node (unless it is
/// already part of the selection — then keep it, so a multi-drag works); Shift
/// toggles it into/out of the selection.
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

#[cfg(test)]
#[path = "interact_tests.rs"]
mod tests;
