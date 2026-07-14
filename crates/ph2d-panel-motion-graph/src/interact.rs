//! Gesture interpretation (Motion Nodes M1.E4–E7, Phase 1b). Drains the
//! `GraphSurface` channel the M0 dispatch fills (pointer gestures, anchored zoom,
//! graph keys) and turns it into ephemeral view/selection/drag/menu state, plus
//! the [`GraphIntent`]s the shell applies (doc mutations only).
//!
//! Coverage: **middle-drag = pan** (from anywhere on the surface), **left-drag on
//! empty canvas = rubber-band select** (Shift = additive), anchored wheel zoom,
//! F = fit, Esc = deselect / cancel, click/shift-select, multi-drag (one
//! `MoveNodes` undo at End), socket→socket **connect** (with a live compatibility
//! ghost; the shell validates for real), alt-press a wire = **disconnect**,
//! R-press (anywhere) / `A` = **add-node** menu, Delete = **delete selection**,
//! and the backdrop gestures (header drag / corner resize).
//!
//! **The buttons (Enio, smoke 2026-07-12).** Left used to pan the canvas, which
//! left multi-select with nowhere to live. The node-editor convention — Blender,
//! Nuke, Houdini — is middle-pans / left-selects, and that is what this is now.

use crate::geom::{self, View};
use crate::snapshot::{GraphIntent, GraphViewSnapshot, push_intent};
use crate::state::MenuBody;
use crate::state::{Interaction, Menu, MotionGraphPanelState};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{
    GesturePhase, GraphGesture, GraphHitKind, GraphKey, GraphZoom,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;

#[path = "interact_backdrop.rs"]
mod backdrop_gesture;

#[path = "interact_socket.rs"]
mod socket;

#[path = "interact_menu.rs"]
mod menu;
use menu::{drag_menu_thumb, grab_menu_thumb, resolve_menu, scroll_menu};

#[path = "interact_subgraph.rs"]
mod subgraph_gesture;
pub(crate) use subgraph_gesture::{GroupVerb, verb as group_verb};

#[path = "interact_key.rs"]
mod key;
use key::apply_key;

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

    settle_pending_detach(state);

    // The shell hands back a selection after it mints ids the panel could not know
    // (Ctrl+D's duplicates). Taken FIRST, so this frame's gestures act on the copies.
    if let Some(nodes) = crate::snapshot::take_selection_request() {
        state.selected = nodes.into_iter().collect();
        state.selected_backdrop = None;
    }

    crate::menu_search::mirror_query(state, ctx);

    // The wheel over an OPEN add-menu scrolls its list; only otherwise does it zoom the canvas.
    if let Some(z) = ctx.host.store_mut().take_graph_zoom(panel)
        && !scroll_menu(state, rect, snap, &z)
    {
        apply_zoom(state, rect, z);
    }
    let keys: Vec<GraphKey> = ctx.host.store_mut().drain_graph_keys().collect();
    for k in keys {
        apply_key(state, k, rect, snap);
    }
    let gestures: Vec<GraphGesture> = ctx.host.store_mut().drain_graph_gestures().collect();
    for g in gestures {
        apply_gesture(state, g, rect, center, snap);
    }

    crate::menu_search::settle_focus(state, ctx);
    crate::rename::settle_focus(state, ctx);
}

/// **The frame boundary a released wire-end has to survive** (doc 45.1).
///
/// The shell applies the panel's intents at the TOP of a frame and republishes the snapshot
/// after — so the frame in which the drop happens still paints from a snapshot where the wire
/// is plugged in exactly where the artist just tore it out of. Suppressing it only while the
/// pointer is down made the end **snap back to its old socket for one frame** before vanishing.
///
/// So the suppression outlives the gesture by one frame, and dies HERE, on the next pass: by
/// now the snapshot IS the shell's answer — moved, unplugged, or (an illegal landing) refused
/// with the original wire intact — and the answer is the truth to paint, whatever it is.
fn settle_pending_detach(state: &mut MotionGraphPanelState) {
    state.pending_detach = None;
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
        // What a right-press MEANS depends on what is under it (doc 62) — the node library over
        // the canvas, the tint palette over a backdrop's header. It lives in `interact_menu`,
        // which owns the popups; this file was at the 600-LOC panel cap.
        menu::open_on_right_press(state, g, rect, snap);
        return;
    }
    // The MIDDLE button pans, from anywhere on the surface — over a card, a wire, a
    // backdrop, empty canvas. The node-editor convention (Blender / Nuke / Houdini),
    // and it frees the left button for selecting, which is what it is for.
    if g.button == PointerButton::Middle {
        apply_pan(state, g);
        return;
    }
    match g.kind {
        GraphHitKind::Background => apply_background(state, g, rect, snap),
        GraphHitKind::Node { node } => apply_node(state, g, node as u32, snap),
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
        // **Double-click a wire → splice a REROUTE node into it** (doc 45). The dot is a
        // node: it bends the wire AND you can drag a new wire out of it. The shell picks the
        // reroute type that fits this wire and re-wires source → dot → target.
        GraphHitKind::Wire { edge } if g.phase == GesturePhase::DoubleClick => {
            let (to_node, to_port) = crate::paint::wire_target(edge);
            let view = View::new(rect, state.view);
            let (x, y) = view.graph(g.x, g.y);
            push_intent(GraphIntent::SpliceReroute {
                to_node,
                to_port,
                x,
                y,
            });
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
            crate::paint_chrome::CHROME_BACKDROP => {
                backdrop_gesture::add_backdrop(state, rect, snap)
            }
            crate::paint_chrome::CHROME_KNIFE => state.knife_armed = !state.knife_armed,
            crate::paint_chrome::CHROME_PROBE => state.probe_armed = !state.probe_armed,
            crate::paint_chrome::CHROME_GROUP => subgraph_gesture::chip(state, snap),
            // The breadcrumb (doc 57): crumb `i` rides the ordinal `CHROME_CRUMB_BASE + i`.
            id if id >= crate::paint_chrome::CHROME_CRUMB_BASE => {
                subgraph_gesture::go_to_crumb(
                    snap,
                    (id - crate::paint_chrome::CHROME_CRUMB_BASE) as usize,
                );
            }
            _ => {}
        },
        // A backdrop's header: select it (a backdrop and a node are never selected
        // together) and drag the whole group — the region plus every node it
        // frames, captured now (see `Interaction::DragBackdrop`).
        GraphHitKind::Backdrop { id } => {
            backdrop_gesture::apply_backdrop(state, g, id as u32, snap)
        }
        // Either bottom corner resizes; the handle says which (the panel packed it).
        GraphHitKind::BackdropResize { id } => {
            let (id, left) = crate::backdrop::resize_target(id);
            backdrop_gesture::apply_backdrop_resize(state, g, id, left)
        }
        // An INPUT socket: draw a wire backwards out of an empty one, or grab the END of the
        // wire already in it and move it (doc 45). Until now this did nothing at all.
        GraphHitKind::SocketIn { node, port } => {
            socket::apply_socket_in(state, g, node as u32, port, rect, snap)
        }
        _ => {}
    }
}

/// Empty-canvas gestures (primary button only — the secondary button is fully
/// handled in [`apply_gesture`]): pan, selection clear, and resolving the
/// add-node menu the right-press opened.
/// Middle-button pan — any hit kind, any part of the surface.
fn apply_pan(state: &mut MotionGraphPanelState, g: GraphGesture) {
    match g.phase {
        GesturePhase::Begin => state.interaction = Interaction::Pan { last: (g.x, g.y) },
        GesturePhase::Update => {
            if let Interaction::Pan { last } = &mut state.interaction {
                state.view.pan_x += g.x - last.0;
                state.view.pan_y += g.y - last.1;
                *last = (g.x, g.y);
            }
        }
        _ => state.interaction = Interaction::Idle,
    }
}

fn apply_background(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    match g.phase {
        GesturePhase::Begin => {
            // The scrollbar takes the press before anything else: reaching for it must not shut
            // the very menu you are trying to scroll (doc 54).
            if let Some(grab) = grab_menu_thumb(state, rect, snap, g.x, g.y) {
                state.interaction = Interaction::MenuScroll { grab };
            } else if state.menu.is_some() {
                // A press anywhere else while the menu is open consumes it (no band); the release
                // resolves the menu. Otherwise start a rubber band — the LEFT button selects
                // (panning is the middle button's job).
                state.interaction = Interaction::Idle;
            } else if state.knife_armed {
                state.interaction = Interaction::Knife {
                    anchor: (g.x, g.y),
                    cur: (g.x, g.y),
                };
            } else {
                state.interaction = Interaction::BoxSelect {
                    anchor: (g.x, g.y),
                    cur: (g.x, g.y),
                    additive: g.mods.shift,
                };
            }
        }
        GesturePhase::Update => {
            if let Interaction::MenuScroll { grab } = state.interaction {
                drag_menu_thumb(state, rect, snap, g.y, grab);
            } else if let Interaction::BoxSelect { cur, .. } | Interaction::Knife { cur, .. } =
                &mut state.interaction
            {
                *cur = (g.x, g.y);
            }
        }
        GesturePhase::Click => {
            // A tap on the scrollbar scrolled; it chose nothing, and it must not close the menu.
            if matches!(state.interaction, Interaction::MenuScroll { .. }) {
                state.interaction = Interaction::Idle;
            } else if let Some(menu) = state.menu.take() {
                // A primary click while the menu is open closes it; a click on a
                // row also adds that node at the menu's spawn point.
                resolve_menu(&menu, rect, snap, g.x, g.y);
            } else {
                // A plain tap on empty canvas clears the selection — including a
                // selected backdrop (the tap goes THROUGH its click-through body).
                state.selected.clear();
                state.selected_backdrop = None;
            }
            state.interaction = Interaction::Idle;
        }
        GesturePhase::End | GesturePhase::DoubleClick => {
            // A finished scrollbar drag leaves the menu exactly as it was: open, and scrolled.
            if matches!(state.interaction, Interaction::MenuScroll { .. }) {
                state.interaction = Interaction::Idle;
                return;
            }
            // **A release INSIDE the popup is a PICK, not a dismissal** (Enio, smoke
            // 2026-07-13: *"não consigo inserir nenhum nó ao clicar no menu"*).
            //
            // The dispatcher calls a press-release with ANY movement between them an `End`,
            // not a `Click` — and a hand always moves. One pixel of drift and the row the
            // artist pressed became a drag, the menu closed, and nothing was added. Every
            // test sent Down and Up at the same coordinate, which is the one thing a real
            // hand never does.
            //
            // While a menu is open the pointer belongs to the MENU, so where the button
            // comes UP is what it means: over a row, that row; anywhere else, dismiss. (This
            // also gives the popup the press-slide-release gesture every OS menu has.)
            if let Some(menu) = state.menu.take() {
                let rows = crate::snapshot::menu_rows(snap, &menu).len();
                if geom::menu_panel(&menu, rows, rect).contains(g.x, g.y) {
                    resolve_menu(&menu, rect, snap, g.x, g.y);
                }
                state.interaction = Interaction::Idle;
                return;
            }
            // A left-drag over empty canvas dismisses an open menu; otherwise it was
            // a rubber band (select) or a knife stroke (cut) — resolve whichever.
            {
                let view = View::new(rect, state.view);
                match state.interaction {
                    Interaction::BoxSelect {
                        anchor,
                        cur,
                        additive,
                    } => {
                        let hit = geom::nodes_in_box(snap, &view, geom::band_rect(anchor, cur));
                        if !additive {
                            state.selected.clear();
                        }
                        state.selected.extend(hit);
                        // A band selects NODES; a backdrop is picked by its header
                        // alone (its body is click-through — the band swept over it).
                        state.selected_backdrop = None;
                    }
                    Interaction::Knife { anchor, cur } => {
                        let targets = crate::paint::wires_crossed(snap, &view, anchor, cur);
                        if !targets.is_empty() {
                            push_intent(GraphIntent::CutWires { targets });
                        }
                        // The stroke consumes the arming: a knife you have to
                        // remember to put away is a knife that cuts by accident.
                        state.knife_armed = false;
                    }
                    _ => {}
                }
            }
            state.interaction = Interaction::Idle;
        }
    }
}

/// Node-body gestures: select on press, multi-drag with a live `MoveNodes`.
fn apply_node(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    node: u32,
    snap: &GraphViewSnapshot,
) {
    // Probe armed: this press PICKS the node to read instead of selecting/dragging
    // it. The pick disarms (three exits, like the knife).
    if state.probe_armed && g.phase == GesturePhase::Begin {
        state.probe_armed = false;
        state.probe = Some(node);
        push_intent(GraphIntent::SetProbe { node: Some(node) });
        state.interaction = Interaction::Idle;
        return;
    }
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
        // **Double-click a collapsed card → go inside it** (doc 57). On an ordinary
        // node it is inert, and falls through to the same idle reset as a Click.
        GesturePhase::DoubleClick => {
            subgraph_gesture::enter(state, snap, node);
        }
        GesturePhase::Click => {
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
                detached: None,
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
            {
                let Some((to_node, to_port, _compat)) = target else {
                    // Dropped on a collapsed CARD's body: the group has ports this wire
                    // could feed, but no socket for them yet (doc 57 §5). Ask which one.
                    let view = View::new(rect, state.view);
                    if let Some(menu) = subgraph_gesture::card_port_menu(
                        snap,
                        &view,
                        (from_node, from_port),
                        true,
                        None,
                        g.x,
                        g.y,
                    ) {
                        state.menu = Some(menu);
                        return;
                    }
                    // **Smart-connect**: dropped on empty canvas. The gesture already
                    // said what it wanted — a consumer for THIS wire — so open the
                    // add-menu filtered to the node types that can take it, and wire
                    // it on the pick. (Dropping in space used to be a silent no-op:
                    // the artist drew a wire and got nothing back.)
                    let spawn = view.graph(g.x, g.y);
                    state.menu = Some(Menu {
                        scroll: 0.0,
                        screen: (g.x, g.y),
                        spawn,
                        query: String::new(),
                        opened: false,
                        body: MenuBody::Library {
                            connect_from: Some((from_node, from_port)),
                        },
                    });
                    return;
                };
                // Emit regardless of the local compatibility flag — the shell is
                // the authority (cycle / occupied / typing / membrane) and raises
                // the refusal toast.
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
pub(super) fn target_socket(
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

#[cfg(test)]
#[path = "interact_f2_tests.rs"]
mod f2_tests;

#[cfg(test)]
#[path = "interact_f2b_tests.rs"]
mod f2b_tests;

#[cfg(test)]
#[path = "interact_menu_tests.rs"]
mod menu_tests;

#[cfg(test)]
#[path = "interact_subgraph_tests.rs"]
mod subgraph_tests;
