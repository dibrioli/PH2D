//! **The view-snapshot builder** — resolves the live `Graph` + `NodeRegistry` into the
//! panel's paint-ready [`GraphViewSnapshot`]. Split from `snapshot` (the data structs +
//! channels) for the panel LOC cap; `super` is `snapshot`, whose structs it reads through
//! `use super::*`. The shell fills the cook-derived flags (readout/count/hot/is_sink/inert)
//! and the backdrops AFTER this call — this side only sees the graph.

use super::*;
use ph2d_node_registry::{NodeRegistry, NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::port::{Dim, Domain};

/// Build a [`GraphViewSnapshot`] from the live document graph + registry (M1.E10).
/// Resolves each node's UI metadata (falling back to the type name / `Utility` /
/// `Rect` when a node registered no [`ph2d_node_registry::NodeUiManifest`]) and
/// its port types, and every edge's source domain (its wire color).
pub fn snapshot_from(graph: &Graph, registry: &NodeRegistry) -> GraphViewSnapshot {
    use ph2d_nodegraph::cook::OpResolver;

    let nodes = graph
        .nodes()
        .iter()
        .map(|inst| {
            let type_id = inst.type_id();
            let manifest = registry.resolve(type_id).map(|op| op.manifest());
            let ui = registry.ui_manifest(type_id);
            let pos = graph
                .pos(inst.id)
                .unwrap_or(ph2d_nodegraph::graph::Pos { x: 0.0, y: 0.0 });
            GraphNodeView {
                id: inst.id.0,
                // Every node of the graph is a plain node here. The shell FOLDS this
                // full view down to the level being shown (`motion_bridge_subgraph`),
                // turning the nested ones into cards and the boundary-touching
                // outsiders into ghosts — it is the only side that knows the nesting.
                kind: NodeViewKind::Node,
                // **The artist's name wins** (doc 61). Absent, the card says what the node IS
                // (its type's display name) — which is also what the rename box seeds with, so
                // the string you edit is the string you were looking at.
                display_name: graph
                    .label(inst.id)
                    .map(str::to_string)
                    .or_else(|| ui.map(|u| u.display_name.to_string()))
                    .unwrap_or_else(|| inst.type_name.clone()),
                category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
                silhouette: ui.map(|u| u.silhouette).unwrap_or(NodeSilhouette::Rect),
                x: pos.x,
                y: pos.y,
                // The manifest's declared inputs, and then one socket per DRIVEN PARAM
                // (doc 58) — a port the graph does not have, exactly as a subgraph card
                // draws ports the graph does not have (doc 57 §3). Appended, never
                // interleaved: slot `inputs.len() + k` is the k-th driven param, and the
                // shell decodes it back with the same derivation (`param_source::param_at`).
                inputs: manifest
                    .map(|m| {
                        let mut v = port_views(m.inputs);
                        v.extend(driven_param_ports(graph, m, inst.id));
                        v
                    })
                    .unwrap_or_default(),
                outputs: manifest.map(|m| port_views(m.outputs)).unwrap_or_default(),
                // The readout, the stream's mass, whether it changed, and which nodes are
                // sinks all need the COOK (or the shell's sink list), which only the shell
                // owns; it fills them in afterwards, exactly as it does the backdrops below.
                readout: None,
                count: None,
                hot: false,
                is_sink: false,
                preview: None,
                // Read straight from the graph — bypass is authored state, not a cook result.
                bypassed: graph.node_bypassed(inst.id),
                // The diagnosis needs `ph2d_motion_diagnose` + the sink list, which only the
                // shell owns; it fills this afterwards, like the cook-derived flags above.
                inert: false,
                thumbnail: None,
            }
        })
        .collect();

    let mut edges: Vec<GraphEdgeView> = graph
        .edges()
        .iter()
        .map(|e| GraphEdgeView {
            from_node: e.from.0.0,
            from_port: e.from.1,
            to_node: e.to.0.0,
            to_port: e.to.1,
            delayed: e.delayed,
            out_domain: source_domain(graph, registry, e.from.0, e.from.1),
        })
        .collect();
    // A driven param is an edge — it just lands on a name instead of a port index. It is
    // drawn like one, cut like one, and pulled off its socket like one, because to everything
    // downstream of here it IS one.
    for (node, sources) in graph.all_param_sources() {
        let Some(manifest) = graph
            .node(*node)
            .and_then(|n| registry.resolve(n.type_id()))
            .map(|op| op.manifest())
        else {
            continue;
        };
        for (k, (name, (src, src_port))) in sources.iter().enumerate() {
            if declared_param(manifest, name).is_none() {
                continue; // dead data: the node type no longer has that param
            }
            edges.push(GraphEdgeView {
                from_node: src.0,
                from_port: *src_port,
                to_node: node.0,
                to_port: (manifest.inputs.len() + k) as u16,
                delayed: false,
                out_domain: source_domain(graph, registry, *src, *src_port),
            });
        }
    }

    // The backdrops are NOT here: they live on the `MotionDoc`, not in the graph
    // the cook sees. The shell bridge fills them in after this call.
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes,
        edges,
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

/// The param's own `&'static` name, if the node type still declares it. A socket for a param
/// the manifest does not have would be a socket wired to nothing — and the name has to come
/// from the MANIFEST anyway, because a `PortView` names a compile-time port and the graph
/// only knows a `String`.
pub(crate) fn declared_param(
    manifest: &'static ph2d_nodegraph::node::NodeManifest,
    name: &str,
) -> Option<&'static str> {
    manifest
        .params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.name)
}

/// One input socket per driven param, in the map's sorted order (doc 58).
///
/// The type is the one a **value** node emits — a scalar instance stream — because that is
/// what a param can read (`cook::driven_value`). So the editor's own compatibility rule
/// filters the drop for free: only a value wire can reach a parameter.
fn driven_param_ports(
    graph: &Graph,
    manifest: &'static ph2d_nodegraph::node::NodeManifest,
    node: ph2d_nodegraph::graph::NodeId,
) -> Vec<PortView> {
    let Some(sources) = graph.param_sources(node) else {
        return Vec::new();
    };
    sources
        .keys()
        .filter_map(|name| declared_param(manifest, name))
        .map(|name| PortView {
            name,
            domain: Domain::Instances,
            dim: Dim::Scalar,
            clock: manifest.clock,
        })
        .collect()
}

fn port_views(specs: &[ph2d_nodegraph::node::PortSpec]) -> Vec<PortView> {
    specs
        .iter()
        .map(|s| PortView {
            name: s.name,
            domain: s.ty.domain,
            dim: s.ty.dim,
            clock: s.ty.clock,
        })
        .collect()
}

/// The [`Domain`] of node `node`'s output port `port` (the wire's color), or
/// `Instances` as a neutral fallback if it can't be resolved.
fn source_domain(
    graph: &Graph,
    registry: &NodeRegistry,
    node: ph2d_nodegraph::graph::NodeId,
    port: u16,
) -> Domain {
    use ph2d_nodegraph::cook::OpResolver;
    graph
        .node(node)
        .and_then(|inst| registry.resolve(NodeTypeId::of(&inst.type_name)))
        .and_then(|op| op.manifest().outputs.get(port as usize))
        .map(|spec| spec.ty.domain)
        .unwrap_or(Domain::Instances)
}
