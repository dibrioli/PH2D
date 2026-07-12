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
    /// Add a group backdrop covering the given graph-space rect (the toolbar's
    /// Backdrop chip). The panel computes the rect — wrapping the selection when
    /// there is one, a default block at the view centre otherwise — and the shell
    /// mints the id (the document owns id allocation). One undo step.
    ///
    /// Backdrops are **UI-only**: applying one never re-cooks (`mark_dirty` is
    /// not called), because nothing about the cook can depend on decoration.
    AddBackdrop { x: f32, y: f32, w: f32, h: f32 },
    /// Move a backdrop by an incremental graph-space delta (header drag). The
    /// nodes it frames move in the SAME frame via a companion
    /// [`Self::MoveNodes`], both inside one [`Self::BeginDrag`]/[`Self::EndDrag`]
    /// bracket — so carrying a group is a single undo step, and the panel needs no
    /// second kind of node-move.
    MoveBackdrop { id: u32, dx: f32, dy: f32 },
    /// Grow/shrink a backdrop from one of its bottom corners (`left` = the
    /// bottom-LEFT gripper), by the pointer's incremental graph-space delta. The
    /// corner the artist did NOT grab stays put: dragging the left one moves the
    /// region's `x` and shrinks its `w`, holding the right edge — which is what a
    /// resize from that side means. The shell owns the minimum-size clamp, and
    /// anchors it to that same fixed edge. Bracketed like a move.
    ResizeBackdrop {
        id: u32,
        left: bool,
        dx: f32,
        dy: f32,
    },
    /// Delete a backdrop (Delete with one selected). The nodes it framed stay —
    /// a backdrop owns nothing, it only draws around things. One undo step.
    DeleteBackdrop { id: u32 },
    /// Rename a backdrop (the params panel's Title row).
    SetBackdropTitle { id: u32, title: String },
    /// Re-tint a backdrop (the params panel's Colour row), 0-based into
    /// `graph-backdrop-1..8`.
    SetBackdropColor { id: u32, color: u8 },
    /// Duplicate the selected nodes (Ctrl+D) — with their params, their text
    /// params and the wires **between them**, offset so the copies are visible.
    /// Wires coming from OUTSIDE the selection are not copied: a duplicate is a
    /// new thing to place, not a second consumer silently spliced into the
    /// upstream (Blender / Nuke both copy internal links only).
    ///
    /// The shell mints the ids and hands the copies back as the new selection
    /// (via [`request_graph_selection`]) — so Ctrl+D then drag moves the COPIES,
    /// which is the whole point of the gesture. One undo step.
    DuplicateSelection { nodes: Vec<u32> },
    /// **Smart-connect**: add a node of `to_type` (the add-menu pick that followed
    /// a wire dropped in empty space) and wire the dragged output into its FIRST
    /// compatible input — one undo step with the add. The panel names the target by
    /// TYPE, not by id: the shell is the one minting ids, and it resolves the type
    /// to the node it just created.
    SmartConnect {
        from_node: u32,
        from_port: u16,
        to_type: &'static str,
        x: f32,
        y: f32,
    },
    /// Point the probe at a node (or clear it). The shell samples that node's
    /// output every tick and publishes the readout + the ring of recent samples
    /// back on the snapshot. UI-only: it never edits the document, so no undo step.
    SetProbe { node: Option<u32> },
    /// Cut every wire the knife stroke crossed, identified by target input
    /// `(to_node, to_port)`. **One undo step for the whole stroke** — a knife that
    /// cut five wires and needed five Ctrl+Z would be a trap.
    CutWires { targets: Vec<(u32, u16)> },
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

/// The rows the add-menu shows. Plain `A` / R-click → the whole catalog. Opened by
/// a wire dropped in empty space (**smart-connect**) → only the node types with an
/// input that wire can feed (domain + dim + clock, the same local rule the drag's
/// compatibility ghost uses; the shell still has the last word on cycles and the
/// membrane). Filtering — rather than listing all 71 and refusing 60 of them after
/// the click — is the whole point: the menu answers "what can I plug in here?".
pub(crate) fn menu_catalog(snap: &GraphViewSnapshot, from: Option<(u32, u16)>) -> Vec<NodeChoice> {
    let all = current_catalog();
    let Some((from_node, from_port)) = from else {
        return all;
    };
    let Some(out) = snap
        .nodes
        .iter()
        .find(|n| n.id == from_node)
        .and_then(|n| n.outputs.get(from_port as usize))
    else {
        return all;
    };
    all.into_iter()
        .filter(|c| c.inputs.iter().any(|i| accepts(&i.ty, out)))
        .collect()
}

/// Whether an input port could take the dragged output (the panel-side rule:
/// domain + dim + clock — `connects_directly` minus the membrane).
fn accepts(input: &ph2d_nodegraph::port::PortType, out: &PortView) -> bool {
    input.domain == out.domain && input.dim == out.dim && input.clock == out.clock
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

    // The backdrops are NOT here: they live on the `MotionDoc`, not in the graph
    // the cook sees. The shell bridge fills them in after this call.
    GraphViewSnapshot {
        nodes,
        edges,
        backdrops: Vec::new(),
        probe: None,
    }
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
