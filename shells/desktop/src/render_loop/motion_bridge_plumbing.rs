//! Engine-managed `pre` plumbing for the Motion graph (docs/Motion Nodes/03, O1).
//!
//! The substrate's only legal state feedback is the 1-tick `pre` edge
//! (ADR-0032) — and gold-standard authoring never makes the artist DRAW that
//! loop (doc 04: POP Solver's green inputs, Bifrost's feedback ports, ICE's
//! Add Forces, Cavalry's Modifiers). So the editor owns it:
//!
//! - A node whose manifest declares an input named `state` or `forces` with
//!   output 0's type is a **feedback host** (append-only convention).
//! - Feedback port empty → self-loop `out --pre--> port`, so integrate/spring
//!   land alive (ballistic motion out of the box) instead of frozen.
//! - A chain wired into the port → the host's previous output enters the
//!   chain at its dangling head(s): `out --pre--> head.in0`. Disconnects and
//!   deletions re-heal; plumbing into nodes that left the branch is removed
//!   by a before/after diff, so a user-authored `pre` anywhere else (the
//!   expert `WouldCycle` path in `apply_connect`) is never touched.
//!
//! The stored graph keeps the exact same edge topology the M2 demos wired by
//! hand — replay, memoization and every dynamics test are untouched; only WHO
//! creates the edges (this module) and how they render (portal badges, not
//! splines — `ph2d-panel-motion-graph::paint`) changed.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::collections::BTreeSet;

/// The feedback input of `id`'s manifest: an input named `state` or `forces`
/// whose type equals output 0's. `None` for ordinary nodes.
fn feedback_port_of(graph: &Graph, id: NodeId, registry: &NodeRegistry) -> Option<u16> {
    let man = graph
        .node(id)
        .and_then(|n| registry.resolve(n.type_id()))
        .map(|op| op.manifest())?;
    let out0 = man.outputs.first()?;
    man.inputs
        .iter()
        .enumerate()
        .find(|(_, p)| matches!(p.name, "state" | "forces") && p.ty == out0.ty)
        .map(|(i, _)| i as u16)
}

/// Every feedback host in the graph, in `nodes()` (insertion) order.
fn feedback_hosts(graph: &Graph, registry: &NodeRegistry) -> Vec<(NodeId, u16)> {
    graph
        .nodes()
        .iter()
        .filter_map(|n| feedback_port_of(graph, n.id, registry).map(|p| (n.id, p)))
        .collect()
}

/// The node feeding `(node, port)` through a forward (non-`pre`) edge.
fn forward_feeder(graph: &Graph, node: NodeId, port: u16) -> Option<NodeId> {
    graph
        .edges()
        .iter()
        .find(|e| !e.delayed && e.to == (node, port))
        .map(|e| e.from.0)
}

fn has_edge_into(graph: &Graph, node: NodeId, port: u16) -> bool {
    graph.edges().iter().any(|e| e.to == (node, port))
}

/// A host's force branch: the upstream closure (forward edges only) of the
/// node feeding its feedback port. Empty when the port is free or self-looped
/// (the self-loop is delayed, so it is no feeder).
fn branch_of(graph: &Graph, host: NodeId, fb: u16) -> BTreeSet<NodeId> {
    let mut set = BTreeSet::new();
    let Some(feeder) = forward_feeder(graph, host, fb) else {
        return set;
    };
    let mut stack = vec![feeder];
    while let Some(n) = stack.pop() {
        if n == host || !set.insert(n) {
            continue;
        }
        for e in graph.edges() {
            if !e.delayed && e.to.0 == n {
                stack.push(e.from.0);
            }
        }
    }
    set
}

/// Is the edge into `(node, port)` engine-managed plumbing? True for a `pre`
/// from a feedback host's output 0 into its own feedback port (the self-loop)
/// or into input 0 of a node inside that host's current force branch.
pub(super) fn is_managed_pre(
    graph: &Graph,
    registry: &NodeRegistry,
    node: NodeId,
    port: u16,
) -> bool {
    let Some(edge) = graph.edges().iter().find(|e| e.to == (node, port)) else {
        return false;
    };
    if !edge.delayed || edge.from.1 != 0 {
        return false;
    }
    let host = edge.from.0;
    let Some(fb) = feedback_port_of(graph, host, registry) else {
        return false;
    };
    (node == host && port == fb) || (port == 0 && branch_of(graph, host, fb).contains(&node))
}

/// Clear an engine-managed `pre` occupying `(node, port)` so a user's forward
/// wire can land there; the caller runs [`reconcile_after`] once the connect
/// commits, which re-plumbs the state entry at the branch's new head. A `pre`
/// that is NOT ours (expert feedback) is left alone — the connect then fails
/// with the ordinary occupied-port refusal, which is the honest outcome.
pub(super) fn clear_managed_pre_at(
    graph: &mut Graph,
    registry: &NodeRegistry,
    node: NodeId,
    port: u16,
) -> bool {
    is_managed_pre(graph, registry, node, port) && graph.disconnect(node, port).is_some()
}

/// Re-derive the plumbing after a structural edit. `before` is the graph as it
/// stood BEFORE the edit; the before/after branch diff keeps the pass surgical
/// (only edges this module itself would have created are ever removed).
/// Idempotent: a second run is a no-op.
pub(super) fn reconcile_after(graph: &mut Graph, registry: &NodeRegistry, before: &Graph) {
    for (host, fb) in feedback_hosts(graph, registry) {
        let b_before = branch_of(before, host, fb);
        let b_after = branch_of(graph, host, fb);
        // Nodes that left the branch lose their host `pre` (nothing else does).
        for n in b_before.difference(&b_after) {
            let managed = graph
                .edges()
                .iter()
                .any(|e| e.delayed && e.from == (host, 0) && e.to == (*n, 0));
            if managed {
                graph.disconnect(*n, 0);
            }
        }
        if b_after.is_empty() {
            // Sequential-node template: an empty feedback port self-loops.
            if !has_edge_into(graph, host, fb) {
                let _ = graph.connect(Edge {
                    from: (host, 0),
                    to: (host, fb),
                    delayed: true,
                });
            }
        } else {
            // The host's previous output enters the chain at its dangling
            // head(s): every branch node with a declared, still-free input 0.
            // A head fed by anything else (another host's plumbing, an expert
            // `pre`, a stream source) is respected as-is.
            for n in &b_after {
                let takes_stream = graph
                    .node(*n)
                    .and_then(|inst| registry.resolve(inst.type_id()))
                    .is_some_and(|op| !op.manifest().inputs.is_empty());
                if takes_stream && !has_edge_into(graph, *n, 0) {
                    let _ = graph.connect(Edge {
                        from: (host, 0),
                        to: (*n, 0),
                        delayed: true,
                    });
                }
            }
        }
    }
}
