//! **Input-socket gestures** (Motion Nodes F2, doc 45) — dragging a wire out of an INPUT.
//! Split from `interact` for the panel LOC cap; `super` is `interact`.
//!
//! Until now an input socket did nothing: you could only ever draw a wire forwards, out of
//! an output. That is half an editor. A graph is read in both directions — sometimes you
//! know what a node *needs* and go looking for the thing that provides it — and it is the
//! only way to fix a wire that landed on the wrong port without deleting it and starting
//! over.
//!
//! An input socket has **two** gestures, and which one you get depends on whether a wire is
//! already there:
//!
//! - **Empty input → draw BACKWARDS.** A ghost wire follows the cursor, hunting for an
//!   output. Drop it on one — or anywhere on a node's BODY, taking its first compatible output —
//!   and the connection is made, exactly as if it had been drawn the other way.
//! - **Occupied input → GRAB THE WIRE'S END.** The wire lets go of the socket and follows
//!   the cursor, still attached to its source. Drop it on another input and it has *moved*;
//!   drop it on empty canvas and it is unplugged.
//!
//! The second one is the reason this matters. It is how every node editor works (Blender,
//! Nuke, Houdini), and it is what makes an input socket feel like a *plug* rather than a
//! painted-on dot.

use super::{View, geom, push_intent, target_socket};
use crate::snapshot::{GraphIntent, GraphViewSnapshot};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::interaction::{GesturePhase, GraphGesture};
use ph2d_editor_core::zones::Rect;

/// The wire landing on this input, as `(from_node, from_port)` — `None` when the socket is
/// empty. An input holds at most one edge (the graph's own invariant), so this is exact.
///
/// A `pre` edge is skipped: the state loop is plumbing the editor owns, and a `pre` head has
/// no spline to grab in the first place (it draws as a portal badge).
fn wire_into(snap: &GraphViewSnapshot, node: u32, port: u16) -> Option<(u32, u16)> {
    snap.edges
        .iter()
        .find(|e| e.to_node == node && e.to_port == port && !e.delayed)
        .map(|e| (e.from_node, e.from_port))
}

/// The gesture on an input socket.
pub(super) fn apply_socket_in(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    node: u32,
    port: u16,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.interaction = match wire_into(snap, node, port) {
                // OCCUPIED: pull the wire's end off this socket. It keeps its source, so
                // from here on it is an ordinary forward wire-draw — the whole `DrawWire`
                // machinery (ghost, target highlight, compatibility preview) works unchanged.
                // `detached` is what turns the drop into a MOVE rather than a second copy.
                Some((from_node, from_port)) => Interaction::DrawWire {
                    from_node,
                    from_port,
                    cur: (g.x, g.y),
                    target: None,
                    detached: Some((node, port)),
                },
                // EMPTY: draw backwards, hunting for an output.
                None => Interaction::DrawWireBack {
                    to_node: node,
                    to_port: port,
                    cur: (g.x, g.y),
                    target: None,
                },
            };
        }
        GesturePhase::Update => {
            let view = View::new(rect, state.view);
            match &mut state.interaction {
                // The pulled-off end behaves exactly like a forward draw.
                Interaction::DrawWire {
                    from_node,
                    from_port,
                    cur,
                    target,
                    ..
                } => {
                    let t = target_socket(snap, &view, *from_node, *from_port, g.x, g.y);
                    *cur = (g.x, g.y);
                    *target = t;
                }
                Interaction::DrawWireBack {
                    to_node,
                    to_port,
                    cur,
                    target,
                } => {
                    let t = output_target(snap, &view, *to_node, *to_port, g.x, g.y);
                    *cur = (g.x, g.y);
                    *target = t;
                }
                _ => {}
            }
        }
        // The landing is resolved from the DROP's own coordinates — not from the last
        // `Update`'s cached target. Where the button came up is where the wire lands, and a
        // gesture that ends without a preceding move (or a hair away from the last one) must
        // not fall back to a stale answer: reading a `None` there would UNPLUG a wire the
        // artist merely touched.
        GesturePhase::End => {
            let view = View::new(rect, state.view);
            match std::mem::take(&mut state.interaction) {
                // A wire whose end was grabbed: it MOVES. Dropped on an input, it lands there;
                // dropped on empty canvas, it is unplugged. Either way, one undo step — and
                // the shell keeps the original if the new landing is illegal.
                Interaction::DrawWire {
                    from_node,
                    from_port,
                    detached: Some((old_to_node, old_to_port)),
                    ..
                } => {
                    let new_to = target_socket(snap, &view, from_node, from_port, g.x, g.y)
                        .map(|(n, p, _)| (n, p));
                    // Dropped on a collapsed CARD's body: which port inside does the end
                    // move to (doc 57 §5)? Nothing is pushed until the artist says — and
                    // the wire stays drawn where it is, because it has not moved yet.
                    if new_to.is_none()
                        && let Some(menu) = super::subgraph_gesture::card_port_menu(
                            snap,
                            &view,
                            (from_node, from_port),
                            true,
                            Some((old_to_node, old_to_port)),
                            g.x,
                            g.y,
                        )
                    {
                        state.menu = Some(menu);
                        return;
                    }
                    // KEEP HIDING the old wire until the shell answers. The shell applies
                    // intents at the top of the NEXT frame, so this frame would otherwise
                    // paint the wire back onto the socket it was just torn off — a snap-back
                    // to a place the wire no longer is. Dropped back home is the exception:
                    // nothing changed, so hiding it would only blink it out for a frame.
                    if new_to != Some((old_to_node, old_to_port)) {
                        state.pending_detach = Some((old_to_node, old_to_port));
                    }
                    push_intent(GraphIntent::MoveWireEnd {
                        from_node,
                        from_port,
                        old_to_node,
                        old_to_port,
                        new_to,
                    });
                }
                // Drawn backwards onto an output: the same connection, made from the other
                // end. The shell is the authority (cycle / occupied / typing / membrane).
                //
                // Dropped on nothing: no wire, no intent, no undo step. (Smart-connect is the
                // forward gesture's answer to that — it knows what the wire would FEED. A
                // backwards drop would have to guess what should feed it, which is a menu of
                // the whole library: the add-menu the artist already has.)
                Interaction::DrawWireBack {
                    to_node, to_port, ..
                } => {
                    if let Some((from_node, from_port, _compat)) =
                        output_target(snap, &view, to_node, to_port, g.x, g.y)
                    {
                        push_intent(GraphIntent::Connect {
                            from_node,
                            from_port,
                            to_node,
                            to_port,
                        });
                    } else if let Some(menu) = super::subgraph_gesture::card_port_menu(
                        snap,
                        &view,
                        (to_node, to_port),
                        false,
                        None,
                        g.x,
                        g.y,
                    ) {
                        // Dropped on a collapsed CARD: which of the group's outputs feeds
                        // this input? The backwards gesture had no answer for a card either
                        // — it was a silent no-op, the very thing it exists to prevent.
                        state.menu = Some(menu);
                    } else if let Some((from_node, from_port)) =
                        super::drop_gesture::node_body_source(snap, &view, to_node, to_port, g.x, g.y)
                    {
                        // Dropped on a regular node's BODY: connect its first compatible OUTPUT —
                        // the mirror of the forward drop-on-node (doc 63.3), so a backward drag is
                        // as forgiving as a forward one.
                        push_intent(GraphIntent::Connect {
                            from_node,
                            from_port,
                            to_node,
                            to_port,
                        });
                    }
                }
                _ => {}
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.interaction = Interaction::Idle;
        }
    }
}

/// The OUTPUT socket under `(x, y)`, with whether it is locally type-compatible with the
/// input we are dragging out of — the mirror of `interact::target_socket`.
fn output_target(
    snap: &GraphViewSnapshot,
    view: &View,
    to_node: u32,
    to_port: u16,
    x: f32,
    y: f32,
) -> Option<(u32, u16, bool)> {
    let (from_node, from_port) = geom::nearest_output_socket(snap, view, x, y)?;
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
    Some((from_node, from_port, compat))
}
