//! Graph edits from the F2 gestures — **Ctrl+D duplicate** and the **knife** —
//! split out of `motion_bridge` (shell LOC cap). Declared by the parent as a
//! `#[path]` sibling, so `super` is `render_loop::motion_bridge`.

use super::{MotionState, plumbing};
use ph2d_editor::ToastQueue;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// How far a duplicate lands from its original, in graph units — enough that the
/// copy reads as a separate card sitting on top-right of the source, not as a
/// misdrawn one. The copies arrive SELECTED, so the artist's next drag places them.
const OFFSET_X: f32 = 40.0; // LITERAL-PX-OK: duplicate offset (graph space)
const OFFSET_Y: f32 = 40.0; // LITERAL-PX-OK: duplicate offset (graph space)

/// Duplicate `nodes`: their type, position (offset), params, text params, and the
/// wires **between them**. One undo step.
///
/// **Internal wires only.** An edge from outside the selection is not copied: a
/// duplicate is a new thing to place, not a second consumer silently spliced into
/// somebody else's upstream. (Blender's Ctrl+D and Nuke's copy/paste both keep the
/// internal links and drop the external ones.)
///
/// The copies become the SELECTION (`request_graph_selection`) — only the shell
/// knows the ids it just minted, and if the originals stayed selected, the drag
/// that naturally follows a Ctrl+D would move the originals instead of the copies.
/// Returns the ids it minted (also handed to the panel as the new selection) —
/// so the caller, and the tests, can see exactly what was created.
pub(super) fn duplicate(motion: &mut MotionState, nodes: Vec<u32>) -> Vec<u32> {
    let sources: Vec<NodeId> = nodes
        .iter()
        .map(|id| NodeId(*id))
        .filter(|id| motion.doc.graph.node(*id).is_some())
        .collect();
    if sources.is_empty() {
        return Vec::new();
    }
    let pre = motion.doc.clone();

    // Clone each node, remembering source → copy so the internal wires can be
    // re-pointed at the copies.
    let mut copy_of: std::collections::BTreeMap<NodeId, NodeId> = Default::default();
    for src in &sources {
        let Some(inst) = motion.doc.graph.node(*src) else {
            continue;
        };
        let type_name = inst.type_name.clone();
        let params = motion
            .doc
            .graph
            .node_param_overrides(*src)
            .cloned()
            .unwrap_or_default();
        let texts = motion
            .doc
            .graph
            .node_text_param_overrides(*src)
            .cloned()
            .unwrap_or_default();
        let pos = motion.doc.graph.pos(*src).unwrap_or(Pos { x: 0.0, y: 0.0 });

        let dst = motion.doc.graph.add_node(type_name);
        motion.doc.graph.set_pos(
            dst,
            Pos {
                x: pos.x + OFFSET_X,
                y: pos.y + OFFSET_Y,
            },
        );
        for (name, value) in params {
            motion.doc.graph.set_param(dst, name, value);
        }
        for (name, value) in texts {
            motion.doc.graph.set_text_param(dst, name, value);
        }
        copy_of.insert(*src, dst);
    }

    // The wires with BOTH ends inside the selection, re-pointed at the copies.
    // A `pre` edge is skipped: the sequential-node plumbing owns those, and
    // `reconcile_after` re-plumbs the copies' self-loops below (copying one by hand
    // would fight it).
    let internal: Vec<Edge> = pre
        .graph
        .edges()
        .iter()
        .filter(|e| !e.delayed)
        .filter_map(|e| {
            let from = copy_of.get(&NodeId(e.from.0.0))?;
            let to = copy_of.get(&NodeId(e.to.0.0))?;
            Some(Edge {
                from: (*from, e.from.1),
                to: (*to, e.to.1),
                delayed: false,
            })
        })
        .collect();
    for e in internal {
        let _ = motion.doc.graph.connect(e);
    }

    // A duplicated sequential node needs its `pre` self-loop re-plumbed, exactly as
    // one dropped from the add-menu does.
    plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();

    let copies: Vec<u32> = copy_of.values().map(|id| id.0).collect();
    ph2d_panel_motion_graph::request_graph_selection(copies.clone());
    copies
}

/// Cut every wire the knife crossed — **one undo step for the whole stroke** (a
/// knife that cut five wires and needed five Ctrl+Z would be a trap).
///
/// Managed `pre` wiring is refused, with the same toast the alt-click disconnect
/// raises: the state loop is plumbing the editor owns, not a wire the artist drew.
/// (The panel already skips `pre` edges when testing the stroke — this is the
/// second barrier, at the authority that owns the document.)
pub(super) fn cut_wires(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    targets: Vec<(u32, u16)>,
) {
    let pre = motion.doc.clone();
    let (mut cut, mut refused) = (0usize, false);
    for (to_node, to_port) in targets {
        let nid = NodeId(to_node);
        if plumbing::is_managed_pre(&motion.doc.graph, &motion.registry, nid, to_port) {
            refused = true;
            continue;
        }
        if motion.doc.graph.disconnect(nid, to_port).is_some() {
            cut += 1;
        }
    }
    if refused {
        toasts.push(ph2d_editor::Toast::info(
            "State wiring is automatic - disconnect the chain from the forces port instead",
        ));
    }
    if cut > 0 {
        plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
        motion.history.push_undo(pre);
        motion.pump.mark_dirty();
    }
}
