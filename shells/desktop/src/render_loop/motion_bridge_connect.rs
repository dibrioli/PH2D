//! The **connect authority** (Motion Nodes M1) — the shell half of a wire the artist drags,
//! split out of `motion_bridge` (shell LOC cap). Declared there as a `#[path]` sibling, so
//! `super` is `render_loop::motion_bridge`.
//!
//! The panel PROPOSES a connection and the shell DISPOSES: `Graph::connect` (structure —
//! cycle, occupied input) then `Graph::validate` (typing, membrane) on a trial clone, kept
//! only if both pass, refused with a toast that says which rule it broke. The panel's live
//! compatibility preview is a courtesy; this is the authority.

use super::{MotionState, plumbing};
use ph2d_editor::ToastQueue;

pub(super) fn apply_connect(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    from_node: u32,
    from_port: u16,
    to_node: u32,
    to_port: u16,
) {
    use ph2d_editor::Toast;
    use ph2d_nodegraph::graph::{Edge, EdgeError, NodeId};
    let fwd = Edge {
        from: (NodeId(from_node), from_port),
        to: (NodeId(to_node), to_port),
        delayed: false,
    };
    let mut trial = motion.doc.graph.clone();
    // A feedback port (or a plumbed branch head) holds an engine-managed `pre`;
    // the user's forward wire takes precedence — clear it here and let the
    // post-commit reconcile re-plumb the state entry at the branch's new head.
    // An expert (non-managed) `pre` is NOT cleared: the connect then fails with
    // the ordinary occupied-port refusal.
    plumbing::clear_managed_pre_at(&mut trial, &motion.registry, NodeId(to_node), to_port);
    let (attempt, edge) = match trial.connect(fwd) {
        Err(EdgeError::WouldCycle) => {
            let pre_edge = Edge {
                delayed: true,
                ..fwd
            };
            (trial.connect(pre_edge), pre_edge)
        }
        other => (other, fwd),
    };
    match attempt {
        Err(e) => {
            toasts.push(Toast::warning(connect_err_msg(e)));
        }
        Ok(()) => {
            let rejected = match trial.validate(&motion.registry) {
                Ok(()) => false,
                Err(viols) => viols.iter().any(|v| violation_blocks_edge(v, edge)),
            };
            if rejected {
                toasts.push(Toast::warning("Can't connect: incompatible ports"));
            } else if scopes_a_sequential_node(&trial) {
                // A spring / integrate upstream of a Time Remap would be asked
                // to integrate a recurrence on a rewritten clock — the cook
                // refuses it (`SequentialInTimeScope`) and the scene goes dark.
                // Refuse the WIRE instead, with the reason (M2.N1).
                toasts.push(Toast::warning(
                    "Can't connect: Time Remap can't rewrite time for a spring or integrate upstream",
                ));
            } else {
                let pre = motion.doc.clone();
                motion.doc.graph = trial;
                plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
                if edge.delayed {
                    toasts.push(Toast::info("Connected as 1-tick feedback (pre)"));
                }
            }
        }
    }
}

/// Does any `motion.time_remap` in `graph` now have a **sequential** node (a
/// spring / integrate, i.e. one fed by a `pre` edge) in its upstream subtree?
/// Such a node integrates a recurrence over the outer tick and has no defined
/// behaviour on a rewritten clock (`CookError::SequentialInTimeScope`), so the
/// editor refuses the wire that would create the situation rather than let the
/// cook drop the sink and the scene go dark (M2.N1, plan §1.5).
pub(super) fn scopes_a_sequential_node(graph: &ph2d_nodegraph::graph::Graph) -> bool {
    graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ph2d_node_motion_time_remap::MANIFEST.name)
        .any(|n| ph2d_node_motion_time_remap::scopes_a_sequential_node(graph, n.id))
}

/// The refusal toast for a rejected edge — which RULE it broke, in English (app UI), so the
/// artist learns the graph's grammar from the refusal instead of guessing at it.
fn connect_err_msg(e: ph2d_nodegraph::graph::EdgeError) -> &'static str {
    use ph2d_nodegraph::graph::EdgeError;
    match e {
        EdgeError::WouldCycle => "Can't connect: would create a cycle",
        EdgeError::InputAlreadyConnected => "Can't connect: input already wired",
        EdgeError::UnknownNode => "Can't connect: unknown node",
    }
}

/// Does a validation violation reject *this* just-added edge (as opposed to a
/// pre-existing problem elsewhere)? Only a type mismatch or a membrane crossing
/// on the same endpoints blocks the connect.
fn violation_blocks_edge(
    v: &ph2d_nodegraph::graph::Violation,
    edge: ph2d_nodegraph::graph::Edge,
) -> bool {
    use ph2d_nodegraph::graph::Violation;
    match v {
        Violation::TypeMismatch { from, to } | Violation::Membrane { from, to } => {
            *from == edge.from && *to == edge.to
        }
        _ => false,
    }
}
