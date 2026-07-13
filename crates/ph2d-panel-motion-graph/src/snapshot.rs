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

#[path = "snapshot_intent.rs"]
mod intent;
pub use intent::GraphIntent;

#[path = "snapshot_drop.rs"]
mod drop_targets;

#[path = "snapshot_menu.rs"]
mod menu;
pub use drop_targets::{
    ChoiceTarget, HiddenPorts, PortChoice, card_hidden_ports, set_card_hidden_ports,
};
pub(crate) use menu::{menu_matches, menu_rows};
// The raw (unranked) catalog is the gates' business only: production code reads the popup
// through `menu_rows`, which is the point.
#[cfg(test)]
pub(crate) use menu::menu_catalog;

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

/// **What a card in the view actually IS** (doc 57). The panel paints and hits all
/// three the same way — a card is a card — but they answer to different verbs, and
/// the difference is not cosmetic: a double-click ENTERS a subgraph, and a ghost
/// refuses to be dragged or deleted because it does not live at this level.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeViewKind {
    /// An ordinary node of the graph, at the level being viewed.
    Node,
    /// A **collapsed subgraph**: one card standing for every node inside it. Its
    /// `id` is a subgraph id tagged with [`SUBGRAPH_VIEW_TAG`] (the two id spaces
    /// are independent, so an untagged id would be ambiguous), and its ports are
    /// the edges that CROSS its boundary — derived, never declared (doc 57 §3).
    Subgraph,
    /// A **ghost**: a node from OUTSIDE this level that is wired across the
    /// boundary, drawn read-only where the wire enters. Houdini's *indirect input*
    /// ("a node-like item that appears inside subnets and corresponds to the node
    /// wired into the subnet") — with the real node's name on it, because we can
    /// afford to say which node it is.
    ///
    /// It is what keeps the inside of a group from LYING: without it, a member fed
    /// from outside would draw an empty input socket, which is the one thing a
    /// socket must never do.
    Ghost,
}

/// A view id with this bit set names a SUBGRAPH, not a node (doc 57 §4). Node ids
/// are minted from 0 upward by `Graph::next_id` and subgraph ids by their own
/// counter, so the two spaces overlap and a bare `u32` would be ambiguous the
/// moment a document has both a node 3 and a subgraph 3 — which is the common case,
/// not the corner one. The shell mints the tag and the shell decodes it; the panel
/// only ever asks *is this a card?*.
pub const SUBGRAPH_VIEW_TAG: u32 = 0x8000_0000; // LITERAL-COLOR-OK: an id tag bit, not a colour

/// Is this view id a collapsed subgraph card?
pub fn is_subgraph_view(id: u32) -> bool {
    id & SUBGRAPH_VIEW_TAG != 0
}

/// One step of the breadcrumb: the level it walks to (`None` = the root canvas)
/// and what to write on it. Built by the shell from the parent chain, root first —
/// Blender's *"breadcrumbs in the top left corner of the node editor"*, Houdini's
/// clickable path gadget.
#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub level: Option<u32>,
    pub title: String,
}

/// One node card in the view.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeView {
    /// `NodeId.0` — the opaque handle carried through the `GraphSurface` channel.
    /// For a [`NodeViewKind::Subgraph`] card this is the subgraph id | [`SUBGRAPH_VIEW_TAG`].
    pub id: u32,
    /// Node, collapsed subgraph, or boundary ghost (see [`NodeViewKind`]).
    pub kind: NodeViewKind,
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
    /// **The inline readout** (F2): what this node produced on THIS frame's cook — the
    /// number the artist would otherwise have to aim the probe at, on every card at once.
    ///
    /// **`None` means the node was never cooked**, which is not an error but the single
    /// most useful thing a node graph can tell you: *nothing downstream consumes this
    /// card*. A freshly dropped node, a chain the artist forgot to wire into the Output, a
    /// branch orphaned by the knife — all of them are blank, and the blankness is the
    /// diagnosis. (The shell fills this from the cook's MEMO, so it costs a lookup, never a
    /// second evaluation — see `Cook::peek`.)
    pub readout: Option<String>,
    /// **How many instances this node emitted** this frame (`None` = never cooked). The wires
    /// leaving it are drawn thicker the heavier the stream (F3): a scatter that fans 12 points
    /// into 5 000 says so in the WIDTH of its wire, with no number to read.
    pub count: Option<u32>,
    /// **Its output CHANGED since last frame** — data is flowing down its wires right now, and
    /// they march (F3). TouchDesigner's reading: *"when you see the wires between nodes
    /// animating, it means the upstream node is cooking"*. A constant branch is wired, alive,
    /// and simply still — and it draws still.
    pub hot: bool,
    /// This node is a **sink** (`motion.output`). The graph is PULLED from the sinks, so this
    /// is where "does anything consume me?" gets answered from (F3, [`crate::flow::live_set`]).
    pub is_sink: bool,
    /// **The postage stamp** (F3): a bounded SUBSAMPLE of the positions this node emitted, in
    /// world units — drawn as a little scatter on the card. Nuke's reading: the thumbnails
    /// *"show what each node passes onto the next node in the tree"*, which is the question a
    /// wire cannot answer no matter how well it is drawn (*where does the spiral become a
    /// grid?*).
    ///
    /// `None` for a node with no positions (a VALUE node's stamp is its number) and for a node
    /// the cook never pulled. The shell caps the point count, so the cost is bounded by the
    /// number of CARDS, never by the size of the stream — which is why this can be on by
    /// default, where Nuke has to tell you to switch its thumbnails off on a heavy script.
    pub preview: Option<Vec<[f32; 2]>>,
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

/// One backdrop (group region) in the view — the document's `Backdrop`, resolved
/// into the panel's own vocabulary (the panel never sees `ph2d-motion-doc`). Pure
/// decoration: it is drawn behind the wires and cards and never cooks.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphBackdropView {
    /// Stable document id (drives selection + every backdrop intent).
    pub id: u32,
    /// Top-left in graph space.
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Tint index into the `graph-backdrop-1..8` tokens (clamped at paint).
    pub color: u8,
    pub title: String,
}

/// The probe's live reading of one node's output (F2). Published by the shell each
/// cook: `samples` is a ring of the most recent values, oldest first, so the panel
/// can draw a sparkline without keeping any history of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeView {
    /// The probed node.
    pub node: u32,
    /// What the number MEANS — a value stream reads out its scalar, an instance
    /// stream reads out how many instances it carries. English (app UI).
    pub label: String,
    /// The latest reading (also the last element of `samples`).
    pub value: f32,
    /// The recent readings, oldest first (up to `PROBE_SAMPLES`).
    pub samples: Vec<f32>,
}

/// How many ticks of history the probe's sparkline shows (~1s at 60 Hz).
pub const PROBE_SAMPLES: usize = 60;

/// The whole graph, resolved to primitives the panel can paint without touching
/// the registry or the graph directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphViewSnapshot {
    pub nodes: Vec<GraphNodeView>,
    pub edges: Vec<GraphEdgeView>,
    /// The probe's live reading, when one is armed on a node (F2).
    pub probe: Option<ProbeView>,
    /// Group regions, painted behind everything. Filled by the shell bridge from
    /// `MotionDoc::backdrops` (not by [`snapshot_from`], which only sees the
    /// graph — the backdrops live on the document, not in the cook).
    pub backdrops: Vec<GraphBackdropView>,
    /// **The level this view is showing** (doc 57): `None` = the root canvas, else
    /// the subgraph that was entered. The panel does not own it — the SHELL does
    /// (like the probe), so that an undo which deletes the subgraph you are
    /// standing in can put you back on solid ground.
    pub level: Option<u32>,
    /// The path from the root to [`Self::level`], root first — one clickable crumb
    /// per step. Always at least one entry (the root).
    pub breadcrumb: Vec<Crumb>,
    /// The playhead, in seconds — the ONE clock the marching dashes read (F3). The panel has
    /// no clock of its own and must not grow one: a flow animation driven by a paint counter
    /// would keep marching on a paused graph, which is precisely the lie the dashes exist to
    /// not tell.
    pub now: f32,
}

/// One addable node type in the add-node menu (M1.E7). Copy — the canonical
/// `type_name` (fed straight back as [`GraphIntent::AddNode`]) and the English
/// `display` label are both `&'static` (the registry's manifest / UI metadata);
/// `category` tints the row's dot so the palette teaches the library map.
#[derive(Copy, Clone, Debug)]
pub struct NodeChoice {
    pub type_name: &'static str,
    pub display: &'static str,
    pub category: NodeUiCategory,
    /// The node type's INPUT ports (straight off its `NodeManifest`, hence
    /// `&'static`). Carried so the panel can answer "what could this wire feed?"
    /// on its own — the smart-connect popup filters the catalog by it, and the
    /// panel must not have to ask the shell a question the manifest already
    /// answers.
    pub inputs: &'static [ph2d_nodegraph::node::PortSpec],
}

/// Two choices are the same choice when they name the same node TYPE — the
/// canonical name is the identity (`PortSpec` itself carries no `PartialEq`, and
/// comparing port lists would say nothing the type name does not).
impl PartialEq for NodeChoice {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name
    }
}
impl Eq for NodeChoice {}

thread_local! {
    static CURRENT: RefCell<Option<GraphViewSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<GraphIntent>> = const { RefCell::new(Vec::new()) };
    static CATALOG: RefCell<Vec<NodeChoice>> = const { RefCell::new(Vec::new()) };
    static SELECTION: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    static BACKDROP_SELECTION: RefCell<Option<u32>> = const { RefCell::new(None) };
    static SELECTION_REQUEST: RefCell<Option<Vec<u32>>> = const { RefCell::new(None) };
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

/// Hand the panel a NEW selection (shell bridge → panel). The one channel that
/// runs against the usual direction, and it has to: only the shell knows the ids
/// it just minted (Ctrl+D's duplicates), and the copies must end up selected — or
/// the drag that naturally follows would move the ORIGINALS. Drained by the panel
/// on its next `process`.
pub fn request_graph_selection(nodes: Vec<u32>) {
    SELECTION_REQUEST.with(|c| *c.borrow_mut() = Some(nodes));
}

/// Take the pending selection request, if any (panel).
pub(crate) fn take_selection_request() -> Option<Vec<u32>> {
    SELECTION_REQUEST.with(|c| c.borrow_mut().take())
}

/// Look at the pending request WITHOUT consuming it — how a seam gate on the shell
/// side checks what an edit handed back. Deliberately non-destructive: a reader that
/// took it would steal the panel's selection and the bug would surface as "sometimes
/// the copies aren't selected", which is exactly the class of bug this channel exists
/// to prevent.
pub fn pending_graph_selection() -> Option<Vec<u32>> {
    SELECTION_REQUEST.with(|c| c.borrow().clone())
}

/// Publish the selected BACKDROP (panel `paint` → shell bridge). Mutually
/// exclusive with the node selection by construction — selecting a backdrop
/// clears the nodes and vice-versa — so the params panel always has exactly one
/// subject to show properties for.
pub fn set_graph_backdrop_selection(selected: Option<u32>) {
    BACKDROP_SELECTION.with(|c| *c.borrow_mut() = selected);
}

/// Read the selected backdrop (shell bridge). `None` when a node (or nothing) is
/// selected, or the Motion tool is inactive.
pub fn current_graph_backdrop_selection() -> Option<u32> {
    BACKDROP_SELECTION.with(|c| *c.borrow())
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

/// Whether two ports could be wired together — the panel-side rule (domain + dim + clock,
/// `connects_directly` minus the membrane; the shell still has the last word).
pub(crate) fn same_type(a: &PortView, b: &PortView) -> bool {
    a.domain == b.domain && a.dim == b.dim && a.clock == b.clock
}

/// Queue an edit for the shell bridge to apply (panel → shell).
///
/// **Public so the shell's seam tests can drive the REAL intent path** — the same
/// queue the panel writes and `drain_intents` reads. A test that called the shell's
/// apply functions directly would prove the apply and skip the wiring, which is the
/// half that breaks (DIRETIVA_IMPLEMENTACAO §2: a missing end of the seam is a
/// dropped click, not a compile error).
pub fn push_intent(intent: GraphIntent) {
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
                // Every node of the graph is a plain node here. The shell FOLDS this
                // full view down to the level being shown (`motion_bridge_subgraph`),
                // turning the nested ones into cards and the boundary-touching
                // outsiders into ghosts — it is the only side that knows the nesting.
                kind: NodeViewKind::Node,
                display_name: ui
                    .map(|u| u.display_name.to_string())
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
