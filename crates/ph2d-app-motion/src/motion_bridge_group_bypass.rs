//! **Bypass a group as a UNIT** (the H verb on a group card) — the cook-time graph transform.
//! Declared by `motion_bridge` as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.
//!
//! A group is invisible to the cook: it has no node and no `output[0]`, and its boundary lives
//! only as the edges crossing it (`fold::card_ports`). So "mute the group as a unit" cannot be a
//! node bypass — it is a REWIRE of that boundary, applied to a throwaway clone the cook reads and
//! never to the document (the interior must survive an un-mute). The convention is Houdini/Nuke's,
//! one level up from the graph's own `cook_bypass::bypass_outputs`: the group's output slot 0
//! passes its input slot 0's source, and every other boundary output goes Empty (unplugged).
//!
//! The common case — no bypassed group — returns `None`, and the cook reads `doc.graph` byte for
//! byte, so grouping-WITHOUT-muting stays the no-op the whole subgraph feature promises.
//!
//! v1 scope, stated plainly (each is a smoke-decides follow-up, not a silent gap):
//! - The GPU cook path is skipped while any group is bypassed (its caller forces the CPU pump),
//!   so a bypassed preview is never cooked from the un-rewired graph.
//! - `output_nodes` and `time_scopes` read `doc.graph` — a bypass removes no NODE, so the sinks
//!   and remappers are the same set; only the value flow (which the pump cooks) is rewired.
//! - Two bypassed groups wired in SERIES compose in a way the smoke should judge; independent or
//!   nested groups (outer wins — the inner is already inside the skipped interior) are handled.

use super::MotionState;
use super::{fold, subgraph};
use ph2d_motion_doc::subgraph as model;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::collections::BTreeSet;

/// The graph the cook should see: `doc.graph` with every bypassed group short-circuited, or
/// `None` when none is — the byte-identical common case.
pub(super) fn cook_graph(motion: &MotionState) -> Option<Graph> {
    let bypassed = &motion.doc.bypassed_subgraphs;
    if bypassed.is_empty() {
        return None;
    }
    let mut g = motion.doc.graph.clone();
    for &sid in bypassed {
        // Outer bypass WINS: a group nested inside a bypassed one is already inside the interior
        // the outer one skips, so short-circuiting it again would fight over the same edges.
        if model::ancestors(&motion.doc.subgraphs, sid)
            .iter()
            .any(|a| bypassed.contains(a))
        {
            continue;
        }
        short_circuit(motion, &mut g, sid);
    }
    Some(g)
}

/// Rewire ONE group's boundary on `g`: output slot 0's consumers read input slot 0's source, and
/// every other boundary-output consumer is unplugged (Empty) — for both wire edges and driven
/// params (a driven param is an edge too, doc 58). Every derivation comes from the ORIGINAL
/// `doc.graph` via `motion`, so the order groups are processed in does not matter.
fn short_circuit(motion: &MotionState, g: &mut Graph, sid: u32) {
    let inside: BTreeSet<NodeId> =
        model::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid)
            .into_iter()
            .collect();
    let cp = fold::card_ports(motion, sid);
    let out0 = cp.outputs.first().copied();
    let s0 = input0_source(motion, cp.inputs.first().copied(), &inside);

    // Snapshot the crossing-OUT consumers (from the original graph) before touching `g`.
    let wires: Vec<Edge> = motion
        .doc
        .graph
        .edges()
        .iter()
        .filter(|e| inside.contains(&e.from.0) && !inside.contains(&e.to.0))
        .copied()
        .collect();
    let params: Vec<(NodeId, String, (NodeId, u16))> = motion
        .doc
        .graph
        .all_param_sources()
        .iter()
        .filter(|(node, _)| !inside.contains(node))
        .flat_map(|(node, srcs)| {
            srcs.iter()
                .map(move |(name, src)| (*node, name.clone(), *src))
        })
        .filter(|(_, _, src)| inside.contains(&src.0))
        .collect();

    for e in wires {
        g.disconnect(e.to.0, e.to.1);
        // Only the FIRST boundary output passes through; the rest go Empty (Houdini's convention).
        if Some(e.from) == out0
            && let Some(src) = s0
        {
            // A refused reconnect (it would close a loop) just leaves the slot Empty — safe.
            let _ = g.connect(Edge {
                from: src,
                to: e.to,
                delayed: e.delayed,
            });
        }
    }
    for (node, name, src) in params {
        g.undrive_param(node, &name);
        if Some(src) == out0
            && let Some(s) = s0
        {
            let _ = g.drive_param(node, name, s);
        }
    }
}

/// The EXTERNAL source feeding the group's input slot 0 — the value the bypass passes through.
/// `None` when the group has no input crossing (then output slot 0 goes Empty too). The slot may
/// be a param socket, drawn like a wire and resolved like one (doc 58).
fn input0_source(
    motion: &MotionState,
    in0: Option<(NodeId, u16)>,
    inside: &BTreeSet<NodeId>,
) -> Option<(NodeId, u16)> {
    let (in_node, in_port) = in0?;
    match subgraph::param_at(motion, in_node, in_port) {
        Some(name) => {
            let src = *motion.doc.graph.param_sources(in_node)?.get(&name)?;
            (!inside.contains(&src.0)).then_some(src)
        }
        None => motion
            .doc
            .graph
            .edges()
            .iter()
            .find(|e| e.to == (in_node, in_port) && !inside.contains(&e.from.0))
            .map(|e| e.from),
    }
}

#[cfg(test)]
#[path = "motion_bridge_group_bypass_tests.rs"]
mod tests;
