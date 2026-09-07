//! Gesture interpretation (Motion Nodes M1.E4–E7, Phase 1b). Drains the
//! `GraphSurface` channel the M0 dispatch fills (pointer gestures, anchored zoom,
//! graph keys) and turns it into ephemeral view/selection/drag/menu state, plus
//! the [`GraphIntent`]s the shell applies (doc mutations only).
//!
//! Coverage: **middle-drag = pan** (from anywhere on the surface), **left-drag on
//! empty canvas = rubber-band select** (Shift = additive, Ctrl = subtract), anchored wheel zoom,
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
// `Menu` is re-exported to the child modules (`interact_menu`, `interact_key`), which reach it as
// `super::Menu` — this file no longer builds one itself (that moved to `interact_drop`).
use crate::state::{Interaction, Menu, MotionGraphPanelState};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{GesturePhase, GraphGesture, GraphHitKind, GraphKey};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;

#[path = "interact_backdrop.rs"]
mod backdrop_gesture;

#[path = "interact_socket.rs"]
mod socket;

#[path = "interact_drop.rs"]
mod drop_gesture;

#[path = "interact_menu.rs"]
mod menu;
use menu::{drag_menu_thumb, grab_menu_thumb, resolve_menu, scroll_menu};

#[path = "interact_subgraph.rs"]
mod subgraph_gesture;
pub(crate) use subgraph_gesture::{GroupVerb, verb as group_verb};

#[path = "interact_key.rs"]
mod key;
// `dedup_double_dispatch` lives with the other keyboard-verb logic in `key` (it dedups the
// graph KEYS the store double-delivers); it is re-exported here for the drain loop below.
use key::{apply_key, dedup_double_dispatch, select_on_press, select_wire_on_press};

#[path = "interact_zoom.rs"]
mod zoom;
use zoom::apply_zoom;

/// Drain this frame's graph input and fold it into `state` (+ push doc intents).
/// Called before drawing so the render reflects the latest gestures. `snap` is
/// the snapshot `paint` already fetched — reused for socket/wire hit-testing.
pub(crate) fn process(
    state: &mut MotionGraphPanelState,
    ctx: &mut PaintCtx,
    rect: Rect,
    band: Rect,
    snap: &GraphViewSnapshot,
) {
    let panel = ids::MOTION_GRAPH_PANEL;

    settle_pending_detach(state);

    // The shell hands back a selection after it mints ids the panel could not know
    // (Ctrl+D's duplicates). Taken FIRST, so this frame's gestures act on the copies.
    if let Some(nodes) = crate::snapshot::take_selection_request() {
        state.selected = nodes.into_iter().collect();
        state.selected_backdrop = None;
        state.selected_wires.clear();
    }

    // The wheel over an OPEN add-menu scrolls its list; only otherwise does it zoom the canvas.
    if let Some(z) = ctx.host.store_mut().take_graph_zoom(panel)
        && !scroll_menu(state, rect, snap, &z)
    {
        apply_zoom(state, rect, z);
    }
    // A verb can arrive TWICE in one frame: the graph's keys reach the store through
    // BOTH the focus gate (`dispatch_key`) and the shell's cursor router
    // (`input_handlers`: "one map, two readers"). Idempotent verbs (Delete/SelectAll)
    // shrug that off, but a NON-idempotent one — Paste, Duplicate — would run twice
    // (two copies from one Ctrl+V). Collapsed here so no verb has to be idempotent to
    // survive the double.
    let keys = dedup_double_dispatch(ctx.host.store_mut().drain_graph_keys().collect());
    for k in keys {
        apply_key(state, k, rect, snap);
    }
    let gestures: Vec<GraphGesture> = ctx.host.store_mut().drain_graph_gestures().collect();
    for g in gestures {
        apply_gesture(state, g, rect, band, snap);
    }

    crate::rename::settle_focus(state, ctx);
    crate::param_edit::settle_focus(state, ctx);
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

fn apply_gesture(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    rect: Rect,
    band: Rect,
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
        // ⭐ **Arrastar um param no cartão** (ciclo 1) — o número do Blender.
        GraphHitKind::ParamRow { node, row } => {
            apply_param_row(state, g, node as u32, row, rect, snap);
        }
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
        // A left-press on a wire SELECTS it (one subject at a time — clears node/backdrop): plain
        // makes it the sole wire, Shift toggles it in/out of the set (the node idiom), and `Delete`
        // drops every selected wire. Not draggable, so this leaves Idle (as an inert press did).
        GraphHitKind::Wire { edge } if g.phase == GesturePhase::Begin => {
            state.selected.clear();
            state.selected_backdrop = None;
            select_wire_on_press(state, crate::paint::wire_target(edge), g.mods.shift);
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
        // ⭐⭐ **Split divider (E9): a fracção mede-se contra a BANDA, que o layout publica.**
        //
        // ⛔⛔ Ela era reconstruída somando `center_viewport + motion_graph` — verdade até a
        // timeline docar DENTRO do split e comer o fundo do grafo. A partir daí o denominador do
        // arrasto era `chrome_h − altura_da_timeline` e o do layout era `chrome_h`: **offset** de
        // ~1,32 e **tremor**, porque a altura da timeline depende da do grafo, que depende do `t`
        // que se está a escrever. Ver `HeroLayout::split_band`.
        //
        // Begin/Update ambos emitem, para o divisor seguir o cursor ao vivo; quem clampa é o shell.
        GraphHitKind::SplitDivider
            if matches!(g.phase, GesturePhase::Begin | GesturePhase::Update) =>
        {
            push_intent(GraphIntent::SetSplit {
                t: crate::split::split_fraction(band, rect, (g.x, g.y)),
            });
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
            crate::paint_chrome::CHROME_FIT => state.request_fit(),
            crate::paint_chrome::CHROME_BACKDROP => {
                backdrop_gesture::add_backdrop(state, rect, snap)
            }
            crate::paint_chrome::CHROME_KNIFE => state.knife_armed = !state.knife_armed,
            crate::paint_chrome::CHROME_PROBE => state.probe_armed = !state.probe_armed,
            crate::paint_chrome::CHROME_GROUP => subgraph_gesture::chip(state, snap),
            // Auto-arrange: a document edit (positions), so it crosses as an intent —
            // the shell owns the doc and the undo bracket.
            crate::paint_chrome::CHROME_ARRANGE => push_intent(GraphIntent::ArrangeLayout),
            // Node help on/off (ADR-0155): the shell owns the flag, so the panel does not
            // flip a local bool — it emits the ABSOLUTE value it wants against the one the
            // shell last published (`!node_help()`), and the next frame reflects it.
            crate::paint_chrome::CHROME_NODE_HELP => {
                push_intent(GraphIntent::SetNodeHelp(!crate::snapshot::node_help()))
            }
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
        // The header preview toggle (doc 86): a Click moves the stamp above↔below — panel-local view state, not a doc edit (so no `GraphIntent`, no undo step).
        GraphHitKind::PreviewToggle { node } if g.phase == GesturePhase::Click => {
            state.toggle_preview_position(node as u32)
        }
        // The ⚠ inert badge (ADR-0155): a Click ASKS the shell to fix this node — the panel only
        // knows it is flagged (it has the snapshot, not the graph). The shell fixes it (an
        // integrator forgotten) or explains it (a choice only the artist can make); either way it
        // leads somewhere, so the badge is never a dead control.
        GraphHitKind::InertBadge { node } if g.phase == GesturePhase::Click => {
            crate::snapshot::push_intent(GraphIntent::FixInert { node: node as u32 });
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
                    subtract: g.mods.cmd,
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
            } else if state.editor.is_some() {
                // ⭐ **Um clique fora do editor rico FECHA-O.** Ele chega aqui pelo escudo de
                // fundo; os widgets do próprio editor estão registados por cima do escudo, logo
                // um clique numa alça nunca passa por este braço.
                state.editor = None;
            } else if let Some(menu) = state.menu.take() {
                // A primary click while the menu is open closes it; a click on a
                // row also adds that node at the menu's spawn point.
                resolve_menu(state, &menu, rect, snap, g.x, g.y);
            } else {
                // A plain tap on empty canvas clears the selection — including a
                // selected backdrop (the tap goes THROUGH its click-through body).
                state.selected.clear();
                state.selected_backdrop = None;
                state.selected_wires.clear();
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
                    resolve_menu(state, &menu, rect, snap, g.x, g.y);
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
                        subtract,
                    } => {
                        let hit = geom::nodes_in_box(snap, &view, geom::band_rect(anchor, cur));
                        if subtract {
                            // Ctrl-drag REMOVES the covered nodes — refine a select-all / linked
                            // island down to what you want, without re-picking from scratch.
                            for id in hit {
                                state.selected.remove(&id);
                            }
                        } else {
                            if !additive {
                                state.selected.clear();
                            }
                            state.selected.extend(hit);
                        }
                        // A band selects NODES; a backdrop is picked by its header
                        // alone (its body is click-through — the band swept over it).
                        state.selected_backdrop = None;
                        state.selected_wires.clear();
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
            state.selected_wires.clear();
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

#[cfg(test)]
#[path = "interact_tests.rs"]
mod tests;

/// Os gates do arrasto de um param no cartão (ciclo 1) — irmão por responsabilidade.
#[cfg(test)]
#[path = "interact_param_row_tests.rs"]
mod param_row_tests;

/// **OS GESTOS DENTRO DE UM CARTÃO** — irmão cortado no tecto de LOC (600) e por
/// RESPONSABILIDADE: este ficheiro trata dos gestos sobre o GRAFO (mover um nó, puxar um fio,
/// laçar, cortar) e aquele dos gestos que o cartão passou a ter **dentro de si** (arrastar um
/// número, alternar um estado, dobrar uma secção). Crescem por razões diferentes.
#[path = "interact_param_row.rs"]
pub(crate) mod param_row;
use param_row::apply_param_row;

/// **PUXAR UM FIO** — irmão cortado no mesmo tecto e pela mesma régua: este ficheiro decide
/// *que gesto é este*, aquele executa o único que tem uma máquina de estados própria (o fio
/// vivo, o alvo magnético, e o que fazer quando ele é largado longe de um pino).
#[path = "interact_wire_drag.rs"]
mod wire_drag;
use wire_drag::apply_socket_out;
pub(super) use wire_drag::target_socket;

#[cfg(test)]
#[path = "interact_drop_tests.rs"]
mod drop_tests;

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
#[path = "interact_context_menu_tests.rs"]
mod context_menu_tests;

#[cfg(test)]
#[path = "interact_wire_tests.rs"]
mod wire_tests;

#[cfg(test)]
#[path = "interact_subgraph_tests.rs"]
mod subgraph_tests;
