//! **Rewiring** (Motion Nodes F2, doc 45) — splice a reroute into a wire, and move a wire's
//! end from one input to another. Declared by `motion_bridge` as a `#[path]` sibling, so
//! `super` is `render_loop::motion_bridge`.
//!
//! Both are ONE undo step and both go through the same authority every hand-drawn wire does
//! (`connect` + `validate` on a trial clone). A gesture that half-succeeds — a node spliced
//! in but not wired, a wire unplugged and not re-plugged — would leave the artist with a
//! graph they did not ask for and no single Ctrl+Z to undo it.

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
pub(super) fn splice_reroute(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    to_node: u32,
    to_port: u16,
    x: f32,
    y: f32,
) {
    let (to, port) = (NodeId(to_node), to_port);
    // The wire, and what it carries.
    let Some(edge) = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == to && e.to.1 == port && !e.delayed)
        .copied()
    else {
        return; // no wire there (or a managed `pre`) — nothing to splice into
    };
    let Some(ty) = wire_type(motion, edge.from.0, edge.from.1) else {
        return;
    };
    let Some(type_name) = reroute_for(ty) else {
        toasts.push(ph2d_editor::Toast::info(
            "No reroute exists for this kind of wire",
        ));
        return;
    };

    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();
    let dot = trial.add_node(type_name.to_string());
    trial.set_pos(dot, Pos { x, y });
    trial.disconnect(to, port);
    let ok = trial
        .connect(Edge {
            from: edge.from,
            to: (dot, 0),
            delayed: false,
        })
        .is_ok()
        && trial
            .connect(Edge {
                from: (dot, 0),
                to: (to, port),
                delayed: false,
            })
            .is_ok()
        && trial.validate(&motion.registry).is_ok();

    if !ok {
        // Refused: the ORIGINAL wire stays. A splice that leaves a node dangling and a wire
        // cut is worse than no splice at all.
        toasts.push(ph2d_editor::Toast::info("Can't reroute this wire"));
        return;
    }
    motion.doc.graph = trial;
    plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty(); // a reroute is a NODE — the graph changed
    ph2d_panel_motion_graph::request_graph_selection(vec![dot.0]);
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
    if trial.disconnect(old.0, old.1).is_none() {
        return; // the wire is already gone (a stale gesture) — do nothing, quietly
    }

    if let Some((n, p)) = new_to {
        // Dropped back where it came from: nothing happened. No undo step for a gesture that
        // changed nothing.
        if (n, p) == (old_to_node, old_to_port) {
            return;
        }
        let landed = trial
            .connect(Edge {
                from: (NodeId(from_node), from_port),
                to: (NodeId(n), p),
                delayed: false,
            })
            .is_ok()
            && trial.validate(&motion.registry).is_ok();
        if !landed {
            toasts.push(ph2d_editor::Toast::info(
                "Can't move the wire there - the original stays",
            ));
            return;
        }
    }

    motion.doc.graph = trial;
    plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
}

#[cfg(test)]
#[path = "motion_bridge_rewire_tests.rs"]
mod tests;
