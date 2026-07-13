//! **The view fold** (Motion Nodes doc 57) — the one place that turns the FLAT
//! graph into the level the artist is looking at.
//!
//! The graph never nests. The document carries a membership map (node → subgraph),
//! and this module answers, for the level being viewed:
//!
//! - which nodes are **direct** members (drawn as themselves),
//! - which are folded into a **card** (drawn as that card, wires terminating on its
//!   derived sockets),
//! - which are **across the boundary** and touch it (drawn as read-only **ghosts** —
//!   Houdini's *indirect input*, but naming the real node instead of a numbered stub).
//!
//! **The card's ports are DERIVED from the edges that cross it**, and they are
//! derived HERE, by [`card_ports`] — which is also what the intent decode
//! ([`super::subgraph::resolve`]) calls to map a socket back to the real port it
//! stands for. One derivation, two readers: a second one is exactly how a socket
//! comes to mean a different port than the one it drew
//! ([[feedback_derived_coordinate_seed_must_match_sample]]).

use super::MotionState;
use ph2d_motion_doc::Holder;
use ph2d_motion_doc::subgraph;
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::{
    ChoiceTarget, Crumb, GraphEdgeView, GraphNodeView, GraphViewSnapshot, HiddenPorts,
    NodeViewKind, PortChoice, SUBGRAPH_VIEW_TAG,
};
use std::collections::{BTreeMap, BTreeSet};

/// What the root crumb is called. English (app UI).
const ROOT_CRUMB: &str = "Root";
/// What an unnamed group is called on its card. English (app UI).
pub(super) const DEFAULT_TITLE: &str = "Group";

/// The view id of a subgraph card (tagged — the node and subgraph id spaces are
/// independent, so an untagged id would alias).
pub(super) fn view_id(sid: u32) -> u32 {
    sid | SUBGRAPH_VIEW_TAG
}

/// The subgraph a view id names, or `None` if it names a node.
pub(super) fn subgraph_of(view: u32) -> Option<u32> {
    (view & SUBGRAPH_VIEW_TAG != 0).then_some(view & !SUBGRAPH_VIEW_TAG)
}

/// The real ports a card's sockets stand for, in slot order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CardPorts {
    /// Slot k ← the real input port `(node, port)` an outside wire feeds.
    pub inputs: Vec<(NodeId, u16)>,
    /// Slot k ← the real output port `(node, port)` that feeds outside. ONE slot per
    /// source port however many outside targets it feeds — a port is a port, and
    /// drawing it twice would say the group had two of them.
    pub outputs: Vec<(NodeId, u16)>,
}

/// Derive a card's interface from the edges crossing its boundary — the same rule
/// Blender ("Group Input and Group Output nodes will be created to represent
/// connections to unselected nodes outside the group") and Unreal ("these pins are
/// automatically generated when the nodes are collapsed") apply at collapse time.
/// We apply it CONTINUOUSLY, because we have no proxy node to freeze it into (doc
/// 57 §3) — so a wire drawn across the seam grows the card a socket, and cutting it
/// takes the socket away.
///
/// Deterministic: sorted by `(node, port)`, so slot k means the same port on every
/// frame, in every session, on every machine.
pub(super) fn card_ports(motion: &MotionState, sid: u32) -> CardPorts {
    let inside: BTreeSet<NodeId> =
        subgraph::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid)
            .into_iter()
            .collect();
    let mut inputs = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    let mut cross = |from: NodeId, from_port: u16, to: NodeId, to_port: u16| {
        match (inside.contains(&from), inside.contains(&to)) {
            (false, true) => {
                inputs.insert((to, to_port));
            }
            (true, false) => {
                outputs.insert((from, from_port));
            }
            _ => {} // wholly inside, or wholly outside: not this card's business
        }
    };
    for e in motion.doc.graph.edges() {
        cross(e.from.0, e.from.1, e.to.0, e.to.1);
    }
    // **A driven param is an edge too** (doc 58), and it crosses boundaries like one. Missed
    // here, grouping a node whose param is driven from outside would make the wire VANISH
    // from the view — the cook would go on reading it, and the canvas would be lying.
    for (node, sources) in motion.doc.graph.all_param_sources() {
        let Some(base) = manifest_of(motion, *node).map(|m| m.inputs.len()) else {
            continue;
        };
        for (k, (_, (src, src_port))) in sources.iter().enumerate() {
            cross(*src, *src_port, *node, (base + k) as u16);
        }
    }
    CardPorts {
        inputs: inputs.into_iter().collect(),
        outputs: outputs.into_iter().collect(),
    }
}

/// The node's manifest, or `None` for a type the registry does not know.
pub(super) fn manifest_of(
    motion: &MotionState,
    node: NodeId,
) -> Option<&'static ph2d_nodegraph::node::NodeManifest> {
    manifest_in(&motion.doc.graph, &motion.registry, node)
}

/// The same, against a bare graph — the rewire path works on a TRIAL clone, not on the
/// document, so it cannot ask the `MotionState`.
pub(super) fn manifest_in(
    graph: &ph2d_nodegraph::graph::Graph,
    registry: &ph2d_node_registry::NodeRegistry,
    node: NodeId,
) -> Option<&'static ph2d_nodegraph::node::NodeManifest> {
    use ph2d_nodegraph::cook::OpResolver;
    let inst = graph.node(node)?;
    registry.resolve(inst.type_id()).map(|op| op.manifest())
}

/// The slot a real port occupies on a card, or `None` when it does not cross.
fn slot_of(slots: &[(NodeId, u16)], node: NodeId, port: u16) -> Option<u16> {
    slots
        .iter()
        .position(|s| *s == (node, port))
        .map(|i| i as u16)
}

/// **Fold the full snapshot down to the level being viewed.** Called after the
/// readouts are stamped (so a card can aggregate what its members are doing) and
/// before the panel ever sees it.
///
/// A document with no subgraphs takes the early exit and pays a breadcrumb — the
/// fold is not a tax on the graphs that never group anything.
pub(super) fn fold(motion: &MotionState, snap: &mut GraphViewSnapshot) {
    let level = motion.level;
    snap.level = level;
    snap.breadcrumb = breadcrumb(motion);
    if motion.doc.subgraphs.is_empty() {
        // No nesting to fold — but EVERY node still offers its params to a dropped wire
        // (doc 58), and most graphs have no groups at all. Publishing an empty map here (as
        // the first cut did) silently turned the feature off for exactly those graphs: the
        // gate that caught it is `a_wire_dropped_on_a_node_drives_one_of_its_params`.
        ph2d_panel_motion_graph::set_card_hidden_ports(node_param_targets(motion, &snap.nodes));
        return;
    }
    let (subs, members) = (&motion.doc.subgraphs, &motion.doc.members);
    let holder = |n: u32| subgraph::holder_at(subs, members, NodeId(n), level);

    // The cards standing at THIS level, each with its derived interface.
    let cards: Vec<u32> = subs
        .iter()
        .filter(|s| s.parent == level)
        .map(|s| s.id)
        .collect();
    let ports: BTreeMap<u32, CardPorts> = cards
        .iter()
        .map(|sid| (*sid, card_ports(motion, *sid)))
        .collect();

    // ── Edges: lift every endpoint to what stands for it here ─────────────────
    let mut ghosts: BTreeSet<u32> = BTreeSet::new();
    let mut edges: Vec<GraphEdgeView> = Vec::with_capacity(snap.edges.len());
    for e in &snap.edges {
        let (hf, ht) = (holder(e.from_node), holder(e.to_node));
        // Wholly across the boundary, or wholly inside ONE card: this level cannot
        // see it, and drawing it would be drawing somebody else's graph.
        if matches!(hf, Holder::Outside) && matches!(ht, Holder::Outside) {
            continue;
        }
        if let (Holder::Card(a), Holder::Card(b)) = (hf, ht)
            && a == b
        {
            continue;
        }
        let mut end = |h: Holder, node: u32, port: u16, input: bool| -> Option<(u32, u16)> {
            match h {
                Holder::Direct => Some((node, port)),
                Holder::Card(sid) => {
                    let p = ports.get(&sid)?;
                    let slots = if input { &p.inputs } else { &p.outputs };
                    slot_of(slots, NodeId(node), port).map(|k| (view_id(sid), k))
                }
                // Across the boundary and wired to something here: it becomes a
                // GHOST of the real node, keeping its real id and its real port —
                // so the wire lands on the socket it actually comes out of.
                Holder::Outside => {
                    ghosts.insert(node);
                    Some((node, port))
                }
            }
        };
        let from = end(hf, e.from_node, e.from_port, false);
        let to = end(ht, e.to_node, e.to_port, true);
        if let (Some((fnode, fport)), Some((tnode, tport))) = (from, to) {
            edges.push(GraphEdgeView {
                from_node: fnode,
                from_port: fport,
                to_node: tnode,
                to_port: tport,
                delayed: e.delayed,
                out_domain: e.out_domain,
            });
        }
    }

    // ── Nodes: the direct members, the ghosts, and one card per child group ───
    let full: Vec<GraphNodeView> = std::mem::take(&mut snap.nodes);
    // What each card is HIDING — the rows of the menu a wire dropped on its body opens
    // (doc 57 §5). Derived from the same `full` views the panel would have drawn and the
    // same raw edges, before either is folded away.
    let mut drop_targets: BTreeMap<u32, HiddenPorts> = cards
        .iter()
        .map(|sid| {
            (
                view_id(*sid),
                hidden_ports(motion, *sid, &ports[sid], &full, &snap.edges),
            )
        })
        .collect();
    // …and every ordinary node AT THIS LEVEL offers its own PARAMS (doc 58). A card hides
    // ports; a node hides parameters; the wire asks the same question of both.
    let here: Vec<GraphNodeView> = full
        .iter()
        .filter(|v| matches!(holder(v.id), Holder::Direct))
        .cloned()
        .collect();
    drop_targets.extend(node_param_targets(motion, &here));
    ph2d_panel_motion_graph::set_card_hidden_ports(drop_targets);
    let mut nodes: Vec<GraphNodeView> = full
        .iter()
        .filter_map(|n| match holder(n.id) {
            Holder::Direct => Some(n.clone()),
            Holder::Outside if ghosts.contains(&n.id) => Some(GraphNodeView {
                kind: NodeViewKind::Ghost,
                ..n.clone()
            }),
            _ => None,
        })
        .collect();
    for sid in &cards {
        nodes.push(card_view(motion, *sid, &ports[sid], &full));
    }
    snap.nodes = nodes;
    snap.edges = edges;

    // Decoration has a level too: a backdrop added inside a group is drawn there,
    // and nowhere else.
    snap.backdrops
        .retain(|b| motion.doc.backdrop_members.get(&b.id).copied() == level);

    // **The probe follows what it is reading, up onto the card that hides it.** Probing
    // a collapsed group points the readout at the node the group EMITS from
    // (`subgraph::probe_target`) — a node which, at this level, is not drawn. The HUD
    // hangs off a card, so without this it would hang off nothing: the artist arms P,
    // clicks the card, and the editor silently shows them nothing at all.
    if let Some(p) = &mut snap.probe
        && let Holder::Card(sid) = holder(p.node)
    {
        p.node = view_id(sid);
    }
}

/// **What a card is hiding** (doc 57 §5) — every port inside the group that has no socket
/// on the card, and could therefore never be reached from outside without going in.
///
/// The two sides are not symmetric, and the asymmetry is the graph's own:
///
/// - an **input** holds at most one edge, so it is offerable only when NOTHING feeds it. A
///   `pre` (the editor's own state-loop plumbing) does not count as fed — it is not a wire
///   the artist drew, and `apply_connect` clears the managed one anyway; the panel's own
///   `wire_into` skips it for exactly the same reason.
/// - an **output** fans out freely, so it is offerable unless the card ALREADY exposes it —
///   in which case the socket is right there and the menu would be offering a second door
///   into the same room.
fn hidden_ports(
    motion: &MotionState,
    sid: u32,
    exposed: &CardPorts,
    full: &[GraphNodeView],
    raw: &[GraphEdgeView],
) -> HiddenPorts {
    let mut out = HiddenPorts::default();
    for n in subgraph::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid) {
        let Some(v) = full.iter().find(|v| v.id == n.0) else {
            continue;
        };
        let choice = |port: u16, pv: &ph2d_panel_motion_graph::PortView| PortChoice {
            node: n.0,
            target: ChoiceTarget::Port(port),
            label: format!("{}: {}", v.display_name, pv.name),
            category: v.category,
            port_type: pv.clone(),
        };
        for (i, pv) in v.inputs.iter().enumerate() {
            let port = i as u16;
            let fed = raw
                .iter()
                .any(|e| e.to_node == n.0 && e.to_port == port && !e.delayed);
            if !fed {
                out.inputs.push(choice(port, pv));
            }
        }
        for (i, pv) in v.outputs.iter().enumerate() {
            let port = i as u16;
            if !exposed.outputs.contains(&(n, port)) {
                out.outputs.push(choice(port, pv));
            }
        }
        // The members' undriven PARAMS are hidden by the card too (doc 58): a wire dropped
        // on a closed group can drive a parameter three storeys down without opening a door.
        out.inputs.extend(param_choices(motion, n, v));
    }
    out
}

/// The drop-targets of ordinary nodes: their undriven params, keyed by node id (which IS the
/// view id for a node — only a card's is tagged).
fn node_param_targets(motion: &MotionState, nodes: &[GraphNodeView]) -> BTreeMap<u32, HiddenPorts> {
    nodes
        .iter()
        .filter_map(|v| {
            let params = param_choices(motion, NodeId(v.id), v);
            (!params.is_empty()).then(|| {
                (
                    v.id,
                    HiddenPorts {
                        inputs: params,
                        outputs: Vec::new(),
                    },
                )
            })
        })
        .collect()
}

/// **The parameters of `n` that no wire drives** (doc 58) — the rows a wire dropped on its
/// body offers.
///
/// A param has no port and never will (`NodeManifest.inputs` is frozen), so this is the ONLY
/// way a wire reaches one. An already-driven param is not offered: its socket is right there
/// on the card, and re-driving it is a drop on that socket.
///
/// The type is the one a value node emits (`Instances/Scalar`, the node's own clock), so the
/// drop's compatibility filter does the rest: only a value wire can reach a parameter, which
/// is exactly the rule the cook enforces anyway (`driven_value` reads the `"v"` column).
fn param_choices(motion: &MotionState, n: NodeId, v: &GraphNodeView) -> Vec<PortChoice> {
    use ph2d_panel_motion_graph::PortView;
    let Some(manifest) = manifest_of(motion, n) else {
        return Vec::new();
    };
    let driven = motion.doc.graph.param_sources(n);
    let hints = motion
        .doc
        .graph
        .node(n)
        .and_then(|i| motion.registry.param_ui(i.type_id()));
    manifest
        .params
        .iter()
        .filter(|p| driven.is_none_or(|d| !d.contains_key(p.name)))
        .map(|p| {
            // The artist reads the label the params panel shows them ("Strength"), not the
            // canonical key ("strength") — the same name in both places, or the menu is
            // offering something they cannot find.
            let label = hints
                .and_then(|hs| hs.iter().find(|h| h.param == p.name))
                .map_or(p.name, |h| h.label);
            PortChoice {
                node: n.0,
                target: ChoiceTarget::Param(p.name),
                label: format!("{}: {}", v.display_name, label),
                category: v.category,
                port_type: PortView {
                    name: p.name,
                    domain: ph2d_nodegraph::port::Domain::Instances,
                    dim: ph2d_nodegraph::port::Dim::Scalar,
                    clock: manifest.clock,
                },
            }
        })
        .collect()
}

/// One collapsed card: its derived sockets, and what its contents are DOING —
/// aggregated from the members' own readouts, which were stamped before the fold.
fn card_view(
    motion: &MotionState,
    sid: u32,
    ports: &CardPorts,
    full: &[GraphNodeView],
) -> GraphNodeView {
    let s = subgraph::find(&motion.doc.subgraphs, sid).expect("card is a subgraph at this level");
    let by_id = |n: NodeId| full.iter().find(|v| v.id == n.0);
    let port_view = |(n, p): &(NodeId, u16), input: bool| {
        by_id(*n).and_then(|v| {
            let list = if input { &v.inputs } else { &v.outputs };
            list.get(*p as usize).cloned()
        })
    };
    let inside = subgraph::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid);
    // The stream leaving the group by its FIRST output — its mass sets the wire's
    // width and its stamp shows what comes out. A card with no output says nothing,
    // which is the truth about a group nothing consumes.
    let head = ports.outputs.first().and_then(|(n, _)| by_id(*n));
    GraphNodeView {
        id: view_id(sid),
        kind: NodeViewKind::Subgraph,
        display_name: if s.title.is_empty() {
            DEFAULT_TITLE.to_string()
        } else {
            s.title.clone()
        },
        category: ph2d_node_registry::NodeUiCategory::Utility,
        silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        x: s.x,
        y: s.y,
        inputs: ports
            .inputs
            .iter()
            .filter_map(|p| port_view(p, true))
            .collect(),
        outputs: ports
            .outputs
            .iter()
            .filter_map(|p| port_view(p, false))
            .collect(),
        // Not a cook readout — a card does not cook. It is how much is folded in
        // here, which is the number the artist actually wants off a closed door.
        readout: Some(format!("{} nodes", inside.len())),
        count: head.and_then(|v| v.count),
        hot: inside.iter().any(|n| by_id(*n).is_some_and(|v| v.hot)),
        is_sink: inside.iter().any(|n| by_id(*n).is_some_and(|v| v.is_sink)),
        preview: head.and_then(|v| v.preview.clone()),
    }
}

/// The path from the root to the current level, root first. Always at least the
/// root crumb — the breadcrumb is a place to stand, not a thing that appears.
fn breadcrumb(motion: &MotionState) -> Vec<Crumb> {
    let mut out = vec![Crumb {
        level: None,
        title: ROOT_CRUMB.to_string(),
    }];
    let Some(level) = motion.level else {
        return out;
    };
    let subs = &motion.doc.subgraphs;
    let mut chain = vec![level];
    chain.extend(subgraph::ancestors(subs, level));
    chain.reverse(); // root-most first
    for id in chain {
        let title = subgraph::find(subs, id)
            .map(|s| s.title.clone())
            .unwrap_or_default();
        out.push(Crumb {
            level: Some(id),
            title: if title.is_empty() {
                DEFAULT_TITLE.to_string()
            } else {
                title
            },
        });
    }
    out
}
