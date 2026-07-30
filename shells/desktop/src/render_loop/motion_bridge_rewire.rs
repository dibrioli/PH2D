//! **Rewiring** (Motion Nodes F2, doc 45) — splice a reroute into a wire, and move a wire's
//! end from one input to another. Declared by `motion_bridge` as a `#[path]` sibling, so
//! `super` is `render_loop::motion_bridge`.
//!
//! Both are ONE undo step and both go through the same authority every hand-drawn wire does
//! (`connect` + `validate` on a trial clone). A gesture that half-succeeds — a node spliced
//! in but not wired, a wire unplugged and not re-plugged — would leave the artist with a
//! graph they did not ask for and no single Ctrl+Z to undo it.

use super::subgraph;
use super::{MotionState, plumbing};
use ph2d_editor::ToastQueue;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_nodegraph::port::PortType;

/// The reroute node type that fits a given port type. `None` when the wire carries something
/// no reroute speaks — which cannot happen today (the whole library uses three port types and
/// there is a reroute for each), but a new port type must not silently splice the wrong dot.
fn reroute_for(ty: PortType) -> Option<&'static str> {
    use ph2d_node_util_reroute as reroute;
    for (manifest, name) in [
        (&reroute::MANIFEST_STREAM, reroute::TYPE_STREAM),
        (&reroute::MANIFEST_VALUE, reroute::TYPE_VALUE),
        (&reroute::MANIFEST_PULSE, reroute::TYPE_PULSE),
    ] {
        if manifest.outputs[0].ty == ty {
            return Some(name);
        }
    }
    None
}

/// The port type a wire carries — read off its SOURCE output.
fn wire_type(motion: &MotionState, from: NodeId, port: u16) -> Option<PortType> {
    motion
        .doc
        .graph
        .node(from)
        .and_then(|n| motion.registry.resolve(n.type_id()))
        .and_then(|op| op.manifest().outputs.get(port as usize))
        .map(|p| p.ty)
}

/// **Splice a reroute node into the wire landing on `(to_node, to_port)`** — the dot the
/// artist double-clicked onto it (doc 45).
///
/// The type is chosen from the WIRE, not from the artist: the gesture already said which
/// wire, and a wire knows what it carries. So there is no menu, no wrong choice to make, and
/// the three reroute node types are an implementation detail the artist never meets.
///
/// The reroute is a **pass-through**, so the render must not move a pixel — that is the
/// property that makes this gesture safe to reach for while tidying a live scene.
/// The wire feeding `(to, port)`, if any — the ordinary edge, never a managed `pre`.
fn wire_edge(motion: &MotionState, to: NodeId, port: u16) -> Option<Edge> {
    motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == to && e.to.1 == port && !e.delayed)
        .copied()
}

/// Insert `type_name` into `edge` at (x,y): on a TRIAL clone, add the node, cut the wire and
/// rewire source → new:0 → target; commit only if the whole thing VALIDATES — else `refuse_msg`
/// and the ORIGINAL wire stays (a splice that dangles a node and cuts a wire is worse than no
/// splice). One undo step; re-cooks; selects the new node. The single door the fixed reroute
/// and the artist-chosen node both go through — two callers, one splice.
fn splice_into_wire(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    edge: Edge,
    type_name: &str,
    x: f32,
    y: f32,
    refuse_msg: &str,
) {
    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();
    let node = trial.add_node(type_name.to_string());
    trial.set_pos(node, Pos { x, y });
    trial.disconnect(edge.to.0, edge.to.1);
    let ok = trial
        .connect(Edge {
            from: edge.from,
            to: (node, 0),
            delayed: false,
        })
        .is_ok()
        && trial
            .connect(Edge {
                from: (node, 0),
                to: edge.to,
                delayed: false,
            })
            .is_ok()
        && trial.validate(&motion.registry).is_ok();

    if !ok {
        toasts.push(ph2d_editor::Toast::info(refuse_msg));
        return;
    }
    motion.doc.graph = trial;
    super::reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty(); // a spliced node is IN the graph — the graph changed
    ph2d_panel_motion_graph::request_graph_selection(vec![node.0]);
}

pub(super) fn splice_reroute(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    to_node: u32,
    to_port: u16,
    x: f32,
    y: f32,
) {
    let Some(edge) = wire_edge(motion, NodeId(to_node), to_port) else {
        return; // no wire there (or a managed `pre`) — nothing to splice into
    };
    // The reroute type is derived from what the wire CARRIES (the artist chose nothing).
    let Some(ty) = wire_type(motion, edge.from.0, edge.from.1) else {
        return;
    };
    let Some(type_name) = reroute_for(ty) else {
        toasts.push(ph2d_editor::Toast::info(
            "No reroute exists for this kind of wire",
        ));
        return;
    };
    splice_into_wire(
        motion,
        toasts,
        edge,
        type_name,
        x,
        y,
        "Can't reroute this wire",
    );
}

/// Insert the artist-CHOSEN `type_name` into the wire feeding `(to_node, to_port)` — the
/// R-press-a-wire → add-menu generalisation of [`splice_reroute`]. Same trial-validate-or-refuse
/// (an unfitting type just does not splice, the wire stays), one undo step, one door.
pub(super) fn splice_node(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    to_node: u32,
    to_port: u16,
    type_name: &str,
    x: f32,
    y: f32,
) {
    let Some(edge) = wire_edge(motion, NodeId(to_node), to_port) else {
        return;
    };
    splice_into_wire(
        motion,
        toasts,
        edge,
        type_name,
        x,
        y,
        "Can't insert this node into this wire",
    );
}

/// **Move a wire's end** from the input it was pulled off to wherever it was dropped (doc
/// 45). `new_to = None` → dropped on empty canvas, so the wire is simply unplugged.
///
/// One undo step: unplug + plug. A `Disconnect` followed by a `Connect` would need two
/// Ctrl+Z to put back — and if the second half were refused, would leave the wire destroyed
/// by a gesture that was only ever asking to move it.
///
/// **A refused landing keeps the ORIGINAL wire.** The artist tried to move a wire somewhere
/// it cannot go; the answer to that is "no", not "the wire you had is gone too".
pub(super) fn move_wire_end(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    from_node: u32,
    from_port: u16,
    old_to_node: u32,
    old_to_port: u16,
    new_to: Option<(u32, u16)>,
) {
    let old = (NodeId(old_to_node), old_to_port);
    if plumbing::is_managed_pre(&motion.doc.graph, &motion.registry, old.0, old.1) {
        toasts.push(ph2d_editor::Toast::info(
            "State wiring is automatic - disconnect the chain from the forces port instead",
        ));
        return;
    }

    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();
    // The end being pulled off may be a PARAM socket (doc 58) — it is drawn like a wire, so
    // it moves like a wire. Resolved before the edit, because the answer lives in the graph
    // the edit is about to change.
    let old_param = subgraph::param_at_in(&trial, &motion.registry, old.0, old.1);
    if !subgraph::unplug_in(&mut trial, old_param.as_deref(), old.0, old.1) {
        return; // the wire is already gone (a stale gesture) — do nothing, quietly
    }

    if let Some((n, p)) = new_to {
        // Dropped back where it came from: nothing happened. No undo step for a gesture that
        // changed nothing.
        if (n, p) == (old_to_node, old_to_port) {
            return;
        }
        // Landing on ANOTHER param's socket: re-drive that one. (Landing on a param that has
        // no wire yet is not a socket at all — the drop opens the body menu instead, and the
        // panel sends `DriveParam`.)
        let dst_param = subgraph::param_at_in(&trial, &motion.registry, NodeId(n), p);
        let landed = match &dst_param {
            Some(name) => trial
                .drive_param(NodeId(n), name.as_str(), (NodeId(from_node), from_port))
                .is_ok(),
            None => trial
                .connect(Edge {
                    from: (NodeId(from_node), from_port),
                    to: (NodeId(n), p),
                    delayed: false,
                })
                .is_ok(),
        } && trial.validate(&motion.registry).is_ok();
        if !landed {
            toasts.push(ph2d_editor::Toast::info(
                "Can't move the wire there - the original stays",
            ));
            return;
        }
    }

    motion.doc.graph = trial;
    super::reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
}

/// **Delete-and-reconnect** (Blender's Ctrl+X): before a node leaves the graph, bridge the wire
/// feeding its PRIMARY input (port 0) to every wire leaving its PRIMARY output, so a node
/// removed from the middle of a chain leaves the chain HEALED instead of severed. This is
/// [`splice_into_wire`] run in reverse, through the same trial-validate-or-refuse authority.
///
/// On success the node is removed and `true` is returned; on `false` the node is left in place
/// and the caller removes it the plain way (today's behaviour). It heals NOTHING — leaving the
/// wire cut — when the bridge would be ambiguous or wrong: no ordinary wire feeds port 0 (a
/// source node has nothing to carry through), no wire leaves port 0 (nothing downstream to
/// keep), or the bridged graph does not VALIDATE. The `!e.delayed` filters skip engine-managed
/// `pre` plumbing (an input takes exactly one source, so a bridged target is never a `pre`),
/// which the delete's `reconcile` re-derives regardless.
pub(super) fn heal_deleted_node(motion: &mut MotionState, nid: NodeId) -> bool {
    // The wire feeding port 0 — the stream this node sits on. Its source is what carries through.
    let Some(source) = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == nid && e.to.1 == 0 && !e.delayed)
        .map(|e| e.from)
    else {
        return false; // nothing feeds port 0 (a source node) — no chain to heal
    };
    // Every wire leaving port 0 — the targets that must keep their input after nid is gone.
    let targets: Vec<(NodeId, u16)> = motion
        .doc
        .graph
        .edges()
        .iter()
        .filter(|e| e.from.0 == nid && e.from.1 == 0 && !e.delayed)
        .map(|e| e.to)
        .collect();
    if targets.is_empty() {
        return false; // nothing downstream of port 0 — deleting an end, not a middle
    }

    let mut trial: Graph = motion.doc.graph.clone();
    trial.remove_node(nid); // drops nid and all its edges; targets are now dangling
    for &(tn, tp) in &targets {
        if trial
            .connect(Edge {
                from: source,
                to: (tn, tp),
                delayed: false,
            })
            .is_err()
        {
            return false; // a bridge that will not connect — leave the node for plain removal
        }
    }
    if trial.validate(&motion.registry).is_err() {
        return false; // the healed chain is ill-typed — the caller severs it the plain way
    }
    motion.doc.graph = trial;
    true
}

#[cfg(test)]
#[path = "motion_bridge_rewire_tests.rs"]
mod tests;
