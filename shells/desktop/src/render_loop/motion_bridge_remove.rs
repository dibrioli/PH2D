//! The **removal authority** (Motion Nodes M1) — the destructive half of the intent
//! funnel (`Disconnect`/`DeleteSelection`) and the sink query, split out of
//! `motion_bridge` (shell LOC cap), the counterpart of `motion_bridge_connect`. Declared
//! there as a `#[path]` sibling and re-exported, so every `motion_bridge::…` call site is
//! unchanged and `super` is `render_loop::motion_bridge`.
//!
//! Everything is fully qualified (`super::…` / crate paths): the two intent handlers are
//! `panel-motion-graph`-gated but `output_nodes` is always compiled (the render loop reads
//! the sinks every frame), so a top-level `use` would leave an unused import behind when
//! the feature is off.

/// The graph's render sinks: **every** `motion.output` node, in node-id order
/// (deterministic). Each lowers onto the same instance buffer, so a document can
/// hold several independent scenes — the default one holds a grid rig and a
/// particle fountain. Empty (nothing renders) until a chain is wired into an
/// Output node.
pub(super) fn output_nodes(
    graph: &ph2d_nodegraph::graph::Graph,
) -> Vec<ph2d_nodegraph::graph::NodeId> {
    // `"motion.output"` is the node's canonical type name (as authored in the
    // default graph); the shell addresses node types by name, like the tool id.
    let mut ids: Vec<_> = graph
        .nodes()
        .iter()
        .filter(|inst| inst.type_name == "motion.output")
        .map(|inst| inst.id)
        .collect();
    ids.sort();
    ids
}

/// Apply a `Disconnect` intent. An engine-managed `pre` (the plumbing badge) is not
/// hand-deletable — it would re-derive on the next reconcile anyway — so the gesture steers
/// the user to the edit that DOES change topology. Everything else disconnects, the plumbing
/// re-heals (a chain pulled off a `forces` port gets its host's self-loop back; a chain split
/// mid-way moves the state entry to the new dangling head).
#[cfg(feature = "panel-motion-graph")]
pub(super) fn apply_disconnect(
    motion: &mut super::MotionState,
    toasts: &mut ph2d_editor::ToastQueue,
    to_node: u32,
    to_port: u16,
) {
    use ph2d_nodegraph::graph::NodeId;
    if super::plumbing::is_managed_pre(
        &motion.doc.graph,
        &motion.registry,
        NodeId(to_node),
        to_port,
    ) {
        toasts.push(ph2d_editor::Toast::info(
            "State wiring is automatic - disconnect the chain from the forces port instead",
        ));
        return;
    }
    let pre = motion.doc.clone();
    if super::subgraph::unplug(motion, NodeId(to_node), to_port) {
        super::reconcile(motion, &pre.graph);
        motion.history.push_undo(pre);
        motion.pump.mark_dirty();
    }
}

/// Apply a `DeleteSelection` intent. A mid-chain node is **healed out**
/// (delete-and-reconnect, Blender Ctrl+X): its port-0 source is bridged to its
/// port-0 targets, so the chain gets shorter, not severed — a deleted force keeps
/// the branch's head and its managed state entry (`plumbing` re-derives the rest).
/// When there is nothing to bridge (an end node, or a heal that would not validate)
/// it falls back to plain removal. The sinks are re-resolved from the Output nodes
/// each frame (before the cook), so deleting one cleanly stops its scene — no manual
/// sink bookkeeping here.
#[cfg(feature = "panel-motion-graph")]
pub(super) fn apply_delete_selection(
    motion: &mut super::MotionState,
    nodes: Vec<u32>,
    toasts: &mut ph2d_editor::ToastQueue,
) {
    let pre = motion.doc.clone();
    let mut changed = false;
    // **A GHOST is not deletable from here** (doc 57): it is a node from outside this
    // level, drawn because a wire reaches it. You can grab it and tidy it (a node has
    // ONE position), but deleting it would reach into a canvas the artist is not on —
    // and a node vanishing from a room you cannot see is the definition of a surprise.
    let (dead, foreign): (Vec<u32>, Vec<u32>) = nodes.into_iter().partition(|id| {
        super::subgraph::subgraph_of(*id).is_some()
            || !matches!(
                ph2d_motion_doc::subgraph::holder_at(
                    &motion.doc.subgraphs,
                    &motion.doc.members,
                    ph2d_nodegraph::graph::NodeId(*id),
                    motion.level,
                ),
                ph2d_motion_doc::Holder::Outside
            )
    });
    if !foreign.is_empty() {
        toasts.push(ph2d_editor::Toast::info(
            "That node lives outside this group - leave the group to delete it",
        ));
    }
    for id in dead {
        match super::subgraph::target(id) {
            // A collapsed card IS its contents (Nuke: "the original nodes are
            // replaced with the Group node"), so deleting it deletes them — every
            // member, at every depth, and the decoration that lived in there.
            super::subgraph::Target::Card(sid) => {
                changed |= super::subgraph::delete_deep(motion, sid)
            }
            super::subgraph::Target::Node(nid) => {
                // Delete-and-reconnect (Blender Ctrl+X): a node removed from the middle of a
                // chain leaves the chain HEALED, not severed. `heal_deleted_node` removes it on
                // success; otherwise fall back to the plain removal. Processing `dead` in order
                // and re-reading the live graph each time composes — deleting two adjacent nodes
                // heals the whole span (grid -> a -> b -> out becomes grid -> out).
                let removed = super::rewire::heal_deleted_node(motion, nid)
                    || motion.doc.graph.remove_node(nid);
                if removed {
                    motion.doc.forget_nodes(&[nid]);
                    changed = true;
                }
            }
        }
    }
    if changed {
        super::reconcile(motion, &pre.graph);
        motion.history.push_undo(pre);
        motion.pump.mark_dirty();
    }
}
