//! Graph view snapshot (Motion Nodes M1.E10) — the state-publish channel from
//! the shell bridge to this panel.
//!
//! The bridge builds a [`GraphViewSnapshot`] each frame from the shell-owned
//! `MotionDoc` + `NodeRegistry` (via [`snapshot_from`]) and hands it over the
//! [`set_current_motion_graph`] thread-local; `paint` reads it back with
//! [`current_snapshot`]. Neither side downcasts the other (mold:
//! `ph2d_panel_vector::set_current_vector_style`). The panel returns edits the
//! other way as `GraphIntent`s (M1.E10 phase 2; not yet wired).

use ph2d_node_registry::{NodeRegistry, NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::port::{Clock, Dim, Domain};
use std::cell::RefCell;

/// One socket on a node card. Color ← [`Domain`], shape ← [`Dim`] (plan §2.4).
/// `clock` completes the [`ph2d_nodegraph::port::PortType`] axes so the editor's
/// live wire-compatibility preview matches `connects_directly` (domain + dim +
/// clock) without touching the graph; the membrane check stays server-side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortView {
    pub name: &'static str,
    pub domain: Domain,
    pub dim: Dim,
    pub clock: Clock,
}

/// One node card in the view.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeView {
    /// `NodeId.0` — the opaque handle carried through the `GraphSurface` channel.
    pub id: u32,
    /// English display label (registry UI metadata, else the type name).
    pub display_name: String,
    /// Header tint category + body silhouette (registry UI metadata).
    pub category: NodeUiCategory,
    pub silhouette: NodeSilhouette,
    /// Top-left in graph space.
    pub x: f32,
    pub y: f32,
    pub inputs: Vec<PortView>,
    pub outputs: Vec<PortView>,
}

/// One wire in the view. Its color is the source port's [`Domain`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEdgeView {
    pub from_node: u32,
    pub from_port: u16,
    pub to_node: u32,
    pub to_port: u16,
    pub delayed: bool,
    pub out_domain: Domain,
}

/// The whole graph, resolved to primitives the panel can paint without touching
/// the registry or the graph directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphViewSnapshot {
    pub nodes: Vec<GraphNodeView>,
    pub edges: Vec<GraphEdgeView>,
}

/// An edit the panel asks the shell to apply to the document (M1.E10, reverse of
/// [`GraphViewSnapshot`]). Ephemeral view state (pan / zoom / selection / drag)
/// stays in `Panel::State` and is NEVER an intent; only doc mutations are.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphIntent {
    /// Open a one-undo-step bracket (the shell snapshots the doc). Pushed on the
    /// first movement of a node drag.
    BeginDrag,
    /// Move nodes by an INCREMENTAL graph-space delta — applied live each frame
    /// so the node tracks the cursor (no end-jump); the whole drag is one undo
    /// step (bracketed by [`Self::BeginDrag`]/[`Self::EndDrag`]).
    MoveNodes { nodes: Vec<u32>, dx: f32, dy: f32 },
    /// Close the bracket (the shell commits the undo step). Pushed on release.
    EndDrag,
    /// Connect an output port to an input port (drag socket→socket). The shell is
    /// the authority: it runs `Graph::connect` (cycle / occupied-input structure)
    /// then `Graph::validate` (typing / membrane) on a trial clone and only keeps
    /// it when the new edge is legal — else it raises a refusal toast. One undo
    /// step; re-cooks (`mark_dirty`).
    Connect {
        from_node: u32,
        from_port: u16,
        to_node: u32,
        to_port: u16,
    },
    /// Remove the edge feeding an input port (alt-click a wire). Identified by its
    /// unique target `(to_node, to_port)`. One undo step; re-cooks.
    Disconnect { to_node: u32, to_port: u16 },
    /// Add a node of the given canonical type at a graph-space position (add-node
    /// menu). `type_name` is a `&'static` canonical name from the published
    /// [`NodeChoice`] catalog. One undo step; re-cooks.
    AddNode {
        type_name: &'static str,
        x: f32,
        y: f32,
    },
    /// Delete the selected nodes and their orphaned incident edges (Delete key).
    /// A no-op (no undo step) when `nodes` is empty or none still exist — safe
    /// against the double key-dispatch (M0 focus gate + shell cursor push). One
    /// undo step; re-cooks.
    DeleteSelection { nodes: Vec<u32> },
    /// Set the viewport⟂graph split fraction (divider drag, E9). Absolute `t` in
    /// the center band; the shell clamps it to the split's legal range. UI-only
    /// (never touches the cook / undo).
    SetSplit { t: f32 },
    /// Flip the split orientation (SplitH/SplitV chips, E9): `true` = vertical
    /// (scene left, graph right), `false` = horizontal (scene top, graph bottom).
    /// UI-only.
    SetSplitVertical { vertical: bool },
    /// Toggle transport play/pause (Space) so time-driven behaviours animate.
    /// The shell owns the transport; UI-only w.r.t. the doc (no undo step).
    TogglePlay,
    /// Set `node` as the render output — the node whose cooked stream is lowered
    /// to instances (the cook's sink). Lets any node become the visible output
    /// (O key). Re-cooks; no undo step (an output choice, not a doc edit).
    SetSink { node: u32 },
}

/// One addable node type in the add-node menu (M1.E7). Copy — the canonical
/// `type_name` (fed straight back as [`GraphIntent::AddNode`]) and the English
/// `display` label are both `&'static` (the registry's manifest / UI metadata);
/// `category` tints the row's dot so the palette teaches the library map.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeChoice {
    pub type_name: &'static str,
    pub display: &'static str,
    pub category: NodeUiCategory,
}

thread_local! {
    static CURRENT: RefCell<Option<GraphViewSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<GraphIntent>> = const { RefCell::new(Vec::new()) };
    static CATALOG: RefCell<Vec<NodeChoice>> = const { RefCell::new(Vec::new()) };
    static SELECTION: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

/// Publish the current node selection (panel `paint` → shell bridge, M1.P1). The
/// bridge reads it to build the selected node's params snapshot for the params
/// panel. Written every frame from the ephemeral `state.selected`.
pub fn set_graph_selection(selection: Vec<u32>) {
    SELECTION.with(|c| *c.borrow_mut() = selection);
}

/// Read the published node selection (shell bridge). Empty when nothing is
/// selected or the Motion tool is inactive.
pub fn current_graph_selection() -> Vec<u32> {
    SELECTION.with(|c| c.borrow().clone())
}

/// Publish the addable-node catalog (shell bridge → panel). Set once on tool
/// activation (the registry is fixed at boot) and cleared to empty on deactivate.
pub fn set_current_node_catalog(catalog: Vec<NodeChoice>) {
    CATALOG.with(|c| *c.borrow_mut() = catalog);
}

/// Read the published catalog (panel, only while the add-node menu is open, so no
/// per-frame clone in the common path).
pub(crate) fn current_catalog() -> Vec<NodeChoice> {
    CATALOG.with(|c| c.borrow().clone())
}

/// Queue an edit for the shell bridge to apply (panel → shell).
pub(crate) fn push_intent(intent: GraphIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drain the queued edits (shell bridge, each frame). Capacity-retaining.
pub fn drain_intents() -> Vec<GraphIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Publish the current graph snapshot (shell bridge → panel). `None` while the
/// Motion tool is inactive.
pub fn set_current_motion_graph(snapshot: Option<GraphViewSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the published snapshot (panel `paint`). Empty when nothing is published.
pub(crate) fn current_snapshot() -> GraphViewSnapshot {
    CURRENT.with(|c| c.borrow().clone().unwrap_or_default())
}

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
                display_name: ui
                    .map(|u| u.display_name.to_string())
                    .unwrap_or_else(|| inst.type_name.clone()),
                category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
                silhouette: ui.map(|u| u.silhouette).unwrap_or(NodeSilhouette::Rect),
                x: pos.x,
                y: pos.y,
                inputs: manifest.map(|m| port_views(m.inputs)).unwrap_or_default(),
                outputs: manifest.map(|m| port_views(m.outputs)).unwrap_or_default(),
            }
        })
        .collect();

    let edges = graph
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

    GraphViewSnapshot { nodes, edges }
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
