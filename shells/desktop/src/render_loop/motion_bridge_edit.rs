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
///
/// Returns **source → copy**, not just the copies: a duplicated subgraph has to put
/// each copy into the copy of the group its source was in (doc 57), and that needs
/// the mapping, not the list. The caller may keep mutating in the same undo step —
/// the snapshot was taken before any of this.
pub(super) fn duplicate(
    motion: &mut MotionState,
    nodes: Vec<u32>,
) -> std::collections::BTreeMap<NodeId, NodeId> {
    let sources: Vec<NodeId> = nodes
        .iter()
        .map(|id| NodeId(*id))
        .filter(|id| motion.doc.graph.node(*id).is_some())
        .collect();
    if sources.is_empty() {
        return Default::default();
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
    super::reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();

    let copies: Vec<u32> = copy_of.values().map(|id| id.0).collect();
    ph2d_panel_motion_graph::request_graph_selection(copies);
    copy_of
}

/// **Smart-connect**: the artist dragged a wire into empty space and picked a node
/// type from the filtered menu. Add that node AND wire the dragged output into the
/// first input the wire fits — **one undo step**, because it was one gesture (an
/// add the artist then has to wire by hand is just the add-menu they already had).
///
/// The connect goes through the same authority as a hand-drawn wire (cycle,
/// occupied input, typing, membrane); the menu's filter is a courtesy — it hides
/// what cannot fit — not a licence to skip the check. If the wire is refused, the
/// node still lands (the artist asked for it) and the toast says why.
pub(super) fn smart_connect(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    from_node: u32,
    from_port: u16,
    to_type: &str,
    x: f32,
    y: f32,
) {
    use ph2d_nodegraph::cook::OpResolver;

    let pre = motion.doc.clone();
    let target = motion.doc.graph.add_node(to_type.to_string());
    motion.doc.graph.set_pos(target, Pos { x, y });
    // A sequential node arrives with its `pre` self-loop plumbed, like any other.
    super::reconcile(motion, &pre.graph);

    // The dragged output's type → the first input on the new node that takes it.
    let out_ty = motion
        .doc
        .graph
        .node(NodeId(from_node))
        .and_then(|n| motion.registry.resolve(n.type_id()))
        .and_then(|op| op.manifest().outputs.get(from_port as usize))
        .map(|p| p.ty);
    let port = out_ty.and_then(|ty| {
        motion
            .doc
            .graph
            .node(target)
            .and_then(|n| motion.registry.resolve(n.type_id()))
            .and_then(|op| op.manifest().inputs.iter().position(|i| i.ty == ty))
    });
    if let Some(port) = port {
        let edge = Edge {
            from: (NodeId(from_node), from_port),
            to: (target, port as u16),
            delayed: false,
        };
        let mut trial = motion.doc.graph.clone();
        if trial.connect(edge).is_ok() && trial.validate(&motion.registry).is_ok() {
            motion.doc.graph = trial;
        } else {
            toasts.push(ph2d_editor::Toast::info(
                "That wire cannot land there - the node was added unconnected",
            ));
        }
    }
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
}

/// Read the probed node and fold the reading into its ring — the sparkline's data.
/// Called once per frame, AFTER the cook, so it reads what the graph actually
/// produced this tick.
///
/// It reuses the pump's own `Cook` (whose memo already holds this tick's results
/// and whose `pre` feedback is the live simulation state), so probing a node costs
/// a memo lookup, not a second evaluation of the chain — and it can never see a
/// different tick than the render did.
///
/// **What the number is:** a VALUE stream reads out its scalar (the `v` column,
/// first element); anything else reads out how many instances it carries. Those
/// are the two questions an artist actually asks a wire ("what is it worth?" /
/// "how many are there?"), and the label says which one is on screen.
pub(super) fn sample_probe(
    motion: &mut MotionState,
    playhead: f64,
) -> Option<ph2d_panel_motion_graph::ProbeView> {
    use ph2d_nodegraph::attr::Column;

    let node = motion.probe?;
    if motion.doc.graph.node(node).is_none() {
        // The probed node was deleted — the probe goes with it.
        motion.probe = None;
        motion.probe_ring.clear();
        return None;
    }
    let out = motion
        .pump
        .cook
        .cook(&motion.doc.graph, &motion.registry, node, playhead)
        .ok()?;
    let stream = out.first()?.as_stream();
    let (label, value) = match stream.get("v") {
        Some(Column::Scalar(v)) if !v.is_empty() => ("value".to_string(), v[0]),
        _ => ("instances".to_string(), stream.count() as f32),
    };

    let ring = &mut motion.probe_ring;
    if ring.len() >= ph2d_panel_motion_graph::PROBE_SAMPLES {
        ring.remove(0);
    }
    ring.push(value);
    Some(ph2d_panel_motion_graph::ProbeView {
        node: node.0,
        label,
        value,
        samples: ring.clone(),
    })
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
        // Whichever kind of socket it is: the knife cuts a param wire like any other
        // (doc 58) — it is drawn like one, so it must die like one.
        if super::subgraph::unplug(motion, nid, to_port) {
            cut += 1;
        }
    }
    if refused {
        toasts.push(ph2d_editor::Toast::info(
            "State wiring is automatic - disconnect the chain from the forces port instead",
        ));
    }
    if cut > 0 {
        super::reconcile(motion, &pre.graph);
        motion.history.push_undo(pre);
        motion.pump.mark_dirty();
    }
}
