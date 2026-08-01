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

/// **Ctrl+C** — snapshot the selection into the graph clipboard (`motion.clip`): each
/// node's type, params, text params and position, plus the wires **between them**.
/// A READ — it never touches the document, so no undo step and no re-cook.
///
/// The captured set mirrors [`duplicate`] exactly (internal wires only; a wire from
/// OUTSIDE is not part of the thing being copied), but into a PORTABLE clip: the
/// edges reference node INDICES, not ids, so [`paste`] can land them in any level or
/// after the originals are gone. `pre` (feedback) edges are dropped — the paste
/// re-plumbs the copies' self-loops through `reconcile`, like a dropped node.
pub(super) fn copy_selection(motion: &mut MotionState, nodes: Vec<u32>, cards: Vec<u32>) {
    let sources: Vec<NodeId> = nodes
        .iter()
        .map(|id| NodeId(*id))
        .filter(|id| motion.doc.graph.node(*id).is_some())
        .collect();
    if sources.is_empty() {
        return; // a copy of nothing leaves the clipboard as it was
    }
    // source id → its index in the clip's node list, so the edges below become
    // index pairs the paste can re-point at the copies it mints.
    let index: std::collections::BTreeMap<NodeId, usize> =
        sources.iter().enumerate().map(|(i, id)| (*id, i)).collect();

    // The groups to carry: every subgraph inside a copied card, at any depth. A copy
    // of loose nodes has no cards, so this is empty and the clip is byte-identical to
    // the pre-nesting one. Sorted ascending (BTreeSet, deterministic) → a stable clip
    // index the node/parent references below point at.
    let relevant: std::collections::BTreeSet<u32> = cards
        .iter()
        .flat_map(|c| ph2d_motion_doc::subgraph::descendants(&motion.doc.subgraphs, *c))
        .collect();
    let sub_index: std::collections::BTreeMap<u32, usize> = relevant
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();
    let clip_subgraphs: Vec<crate::motion_state::ClipSubgraph> = relevant
        .iter()
        .filter_map(|sid| ph2d_motion_doc::subgraph::find(&motion.doc.subgraphs, *sid))
        .map(|s| crate::motion_state::ClipSubgraph {
            title: s.title.clone(),
            // A parent OUTSIDE the copied set becomes a top-level group of the clip
            // (`None`), so the paste hangs it from whatever level it lands in.
            parent: s.parent.and_then(|p| sub_index.get(&p).copied()),
            x: s.x,
            y: s.y,
        })
        .collect();

    let clip_nodes: Vec<crate::motion_state::ClipNode> = sources
        .iter()
        .map(|src| {
            let inst = motion
                .doc
                .graph
                .node(*src)
                .expect("filtered to live nodes above");
            crate::motion_state::ClipNode {
                type_name: inst.type_name.clone(),
                params: motion
                    .doc
                    .graph
                    .node_param_overrides(*src)
                    .cloned()
                    .unwrap_or_default(),
                texts: motion
                    .doc
                    .graph
                    .node_text_param_overrides(*src)
                    .cloned()
                    .unwrap_or_default(),
                pos: motion.doc.graph.pos(*src).unwrap_or(Pos { x: 0.0, y: 0.0 }),
                // The clip index of the group holding it, or loose. A node whose live
                // group is not among the copied cards' subtrees pastes loose.
                subgraph: motion
                    .doc
                    .members
                    .get(src)
                    .and_then(|sid| sub_index.get(sid).copied()),
            }
        })
        .collect();

    let clip_edges: Vec<crate::motion_state::ClipEdge> = motion
        .doc
        .graph
        .edges()
        .iter()
        .filter(|e| !e.delayed)
        .filter_map(|e| {
            let from = *index.get(&NodeId(e.from.0.0))?;
            let to = *index.get(&NodeId(e.to.0.0))?;
            Some(crate::motion_state::ClipEdge {
                from: (from, e.from.1),
                to: (to, e.to.1),
            })
        })
        .collect();

    motion.clip = Some(crate::motion_state::GraphClip {
        nodes: clip_nodes,
        edges: clip_edges,
        subgraphs: clip_subgraphs,
        pastes: 0,
    });
}

/// **Ctrl+V** — mint fresh nodes from the graph clipboard into the CURRENT graph,
/// re-wire the internal links between them, and hand the pastes back as the
/// selection (so the drag that follows moves the copies, not whatever was selected).
/// One undo step; inert when nothing was copied.
///
/// Each paste cascades one offset further than the last (`GraphClip::pastes`), so
/// pasting the same clip repeatedly does not stack every copy on one spot. The
/// membership + `pre` plumbing ride on the shared [`super::reconcile`], so a paste
/// into a group joins that group and a pasted sequential node arrives alive — exactly
/// like a node dropped from the add-menu.
///
/// **A copied group pastes as a group** — its member nodes' wires AND the collapsed
/// cards that held them are rebuilt (`subgraph::paste_nesting`), so Ctrl+V matches
/// Ctrl+D on a group instead of exploding it flat. A copy of loose nodes carries no
/// subgraphs, so that path is byte-identical to before.
pub(super) fn paste(motion: &mut MotionState) {
    let Some(clip) = motion.clip.clone() else {
        return; // nothing copied yet
    };
    if clip.nodes.is_empty() {
        return;
    }
    let pre = motion.doc.clone();
    let step = clip.pastes as f32 + 1.0;
    let (ox, oy) = (OFFSET_X * step, OFFSET_Y * step);

    // Mint one node per clip entry; `new_ids[i]` is the copy of `clip.nodes[i]`, so
    // the index-based edges below re-point at the copies.
    let mut new_ids: Vec<NodeId> = Vec::with_capacity(clip.nodes.len());
    for cn in &clip.nodes {
        let dst = motion.doc.graph.add_node(cn.type_name.clone());
        motion.doc.graph.set_pos(
            dst,
            Pos {
                x: cn.pos.x + ox,
                y: cn.pos.y + oy,
            },
        );
        for (name, value) in &cn.params {
            motion.doc.graph.set_param(dst, name.clone(), *value);
        }
        for (name, value) in &cn.texts {
            motion
                .doc
                .graph
                .set_text_param(dst, name.clone(), value.clone());
        }
        new_ids.push(dst);
    }
    for ce in &clip.edges {
        let _ = motion.doc.graph.connect(Edge {
            from: (new_ids[ce.from.0], ce.from.1),
            to: (new_ids[ce.to.0], ce.to.1),
            delayed: false,
        });
    }

    super::reconcile(motion, &pre.graph);
    // Rebuild the copied groups AFTER reconcile (it re-homes the nested nodes over the
    // current-level membership reconcile just gave them), and inside the same undo
    // step. The selection it returns is cards + loose nodes — for a loose-node clip it
    // is every node, exactly as before.
    let selection = super::subgraph::paste_nesting(motion, &clip, &new_ids, ox, oy);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();

    ph2d_panel_motion_graph::request_graph_selection(selection);

    // Cascade the NEXT paste one offset further (the clip stays, so paste-many works).
    if let Some(c) = motion.clip.as_mut() {
        c.pastes += 1;
    }
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

/// The probe's presentation of a [`super::readout::Reading`] — a label and a raw
/// number, where the readout row wants a formatted string. The DECISION they
/// share lives in `reading_of`; only this differs.
fn reading_label(r: super::readout::Reading) -> (String, f32) {
    match r {
        super::readout::Reading::Value(v) => ("value".to_string(), v),
        super::readout::Reading::Instances(n) => ("instances".to_string(), n as f32),
    }
}

/// Fold one reading into the sparkline's ring.
fn push_probe_sample(motion: &mut MotionState, value: f32) {
    let ring = &mut motion.probe_ring;
    if ring.len() >= ph2d_panel_motion_graph::PROBE_SAMPLES {
        ring.remove(0);
    }
    ring.push(value);
}

/// Read the probed node and fold the reading into its ring — the sparkline's data.
///
/// ⚠️ **Called once per frame BEFORE the cooks**, not after. The old note here
/// said "AFTER the cook", and it is worth being exact because two seams turned on
/// it: `sample_probe` runs in the snapshot block, while `cook_gpu` and the pump's
/// `advance_or_scrub_scoped` are both further down `dispatch`. What made the CPU
/// reading fresh anyway is that this calls `cook`, not `peek` — so on that path
/// the probe performs the tick's FIRST evaluation and the pump then reuses the
/// memo, which is why probing still costs one evaluation of the chain and never
/// two. (The readout row next door peeks, and is therefore a frame behind.)
///
/// **What the number is** — the `v` scalar for a VALUE stream, the instance count
/// otherwise — is decided by [`super::readout::reading_of`], shared with the
/// readout row. The two used to decide it separately, under a comment promising
/// they would never disagree.
pub(super) fn sample_probe(
    motion: &mut MotionState,
    playhead: f64,
    tapped: Option<&std::collections::BTreeMap<NodeId, ph2d_nodegraph::attr::Stream>>,
) -> Option<ph2d_panel_motion_graph::ProbeView> {
    let node = motion.probe?;
    if motion.doc.graph.node(node).is_none() {
        // The probed node was deleted — the probe goes with it.
        motion.probe = None;
        motion.probe_ring.clear();
        return None;
    }
    // A GPU-resident cook does not feed the CPU memo this probe reads, and the
    // `cook` below is NOT a memo lookup when the memo is empty — it would run a
    // full CPU cook of the chain, every frame, beside a GPU that is already
    // drawing it. At the fountain's 1,2 M that is ~50 ms of the frame, and the
    // number it returns is cooked from a `pre` state the pump never marched: a
    // simulation that did not happen. Both halves are unacceptable, and they
    // have the same remedy — do not cook here.
    //
    // ⚠️ `gpu_live` is LAST frame's (this runs before `cook_gpu`, which sets it).
    // On the frame the route flips, the probe therefore pays one CPU cook — the
    // cost it used to pay on EVERY frame, so this is bounded and never worse
    // than the status quo. Moving the sampling after `cook_gpu` would make it
    // exact ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]);
    // that is a publish-order change, noted rather than smuggled in here.
    if motion.gpu_live {
        // **The tap answers instead** (`readout::take_tap`): 48 strided rows per
        // staged node for a measured +0,075 ms, so the probe reads the device's
        // real numbers rather than announcing that it cannot.
        //
        // ⚠️ **The count comes from `CookShape`, never from the tapped stream** —
        // it carries 48 rows whatever the node emitted, and the probe's whole
        // output is a number the artist quotes. Same trap as the readout row's.
        //
        // ⚠️ **One frame behind, and here that is a CHANGE**: the CPU branch below
        // calls `cook`, so it is fresh, while the tap reads the previous cook.
        // The alternative was cooking the chain on the CPU beside a GPU already
        // drawing it (~50 ms at 1,2 M, from a `pre` state the pump never marched
        // — a simulation that did not happen). A real reading one tick old beats
        // both that and the blank this replaces; making it exact is the same
        // publish-order change the note below already names.
        let sampled = tapped.and_then(|t| t.get(&node));
        // A node the device did not stage (a CPU boundary in a hybrid plan) still
        // has a memo entry from the prefix cook. `peek`, never `cook` — see above.
        let fallback = motion.pump.cook.peek(node).and_then(|o| o.first());
        let (label, value) = match (sampled, fallback) {
            (Some(s), _) => reading_label(super::readout::reading_of(
                s,
                motion.gpu_cook.node_count(node).unwrap_or(0),
            )),
            (None, Some(v)) => {
                let st = v.as_stream();
                let n = st.count() as u32;
                reading_label(super::readout::reading_of(st, n))
            }
            // Staged nowhere and cooked nowhere: the node is genuinely not
            // running. Saying so beats inventing a zero.
            (None, None) => {
                motion.probe_ring.clear();
                return Some(ph2d_panel_motion_graph::ProbeView {
                    node: node.0,
                    label: "idle".to_string(),
                    value: None,
                    samples: Vec::new(),
                });
            }
        };
        push_probe_sample(motion, value);
        return Some(ph2d_panel_motion_graph::ProbeView {
            node: node.0,
            label,
            value: Some(value),
            samples: motion.probe_ring.clone(),
        });
    }
    let out = motion
        .pump
        .cook
        .cook(&motion.doc.graph, &motion.registry, node, playhead)
        .ok()?;
    let stream = out.first()?.as_stream();
    let n = stream.count() as u32;
    let (label, value) = reading_label(super::readout::reading_of(stream, n));
    push_probe_sample(motion, value);
    Some(ph2d_panel_motion_graph::ProbeView {
        node: node.0,
        label,
        value: Some(value),
        samples: motion.probe_ring.clone(),
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
