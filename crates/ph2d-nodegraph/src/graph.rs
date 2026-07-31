//! The graph: node instances + typed edges, **acyclic by construction**.
//!
//! Feedback is expressed only via a `pre` (1-tick delay) edge, never a plain
//! back-edge — so there is no runtime cycle detection anywhere (ADR-0032 §4,
//! the Lustre `pre` operator). A plain `connect` that would close a cycle is
//! rejected at edit time, keeping acyclicity an invariant of the stored graph.
//!
//! Instance ids are stable and monotonic; the textual format ([`crate::format`])
//! keys on them. Node **layout** (editor position) is stored in a separate map
//! so it never affects semantics or a semantic diff (ADR-0032 §6).
//!
//! Two layers of invariant:
//! - [`Graph::connect`] enforces the **structural** invariants that need no
//!   external information: acyclicity (forward edges) and one edge per input
//!   port.
//! - [`Graph::validate`] enforces the **semantic** invariants that need the
//!   node manifests (a resolver): port indices exist, port types are
//!   compatible ([`crate::port::PortType::connects_directly`]), and the
//!   membrane holds ([`crate::effect::Effect::can_feed`]). This is where the
//!   ADR-0030 membrane is actually *proven* — call it after editing/loading.

use crate::cook::OpResolver;
use crate::node::NodeTypeId;
use std::collections::{BTreeMap, BTreeSet};

/// A canonical node type name must be non-empty and whitespace-free, so it
/// round-trips through the whitespace-delimited textual format unambiguously.
fn is_valid_type_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(char::is_whitespace)
}

/// Stable instance id within a graph. Assigned monotonically, never reused,
/// survives serialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeInstance {
    pub id: NodeId,
    /// Canonical node-type name, e.g. `"motion.clone"`. Human-readable in the
    /// textual format; the type id is its hash.
    pub type_name: String,
}

impl NodeInstance {
    pub fn type_id(&self) -> NodeTypeId {
        NodeTypeId::of(&self.type_name)
    }
}

/// An edge from one node's output port to another's input port. Structurally
/// constrained by [`Graph::connect`]; its *typing* and membrane-conformance are
/// checked by [`Graph::validate`] (which needs the node manifests).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    /// `(node, output port index)`.
    pub from: (NodeId, u16),
    /// `(node, input port index)`.
    pub to: (NodeId, u16),
    /// A `pre` edge carries last-tick's value, turning an apparent cycle into
    /// a well-founded recurrence over the clock. Delayed edges are exempt from
    /// the acyclicity check.
    pub delayed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EdgeError {
    /// A non-delayed edge would close a cycle. Use a `pre` (delayed) edge for
    /// feedback instead.
    WouldCycle,
    /// One of the endpoints is not a node in this graph.
    UnknownNode,
    /// The target input port already has an incoming edge. An input port takes
    /// at most one edge (the cook reads the first; ambiguity is rejected here).
    InputAlreadyConnected,
}

/// A semantic invariant violation found by [`Graph::validate`]. (Structural
/// problems are rejected earlier, by [`Graph::connect`].)
#[derive(Debug, PartialEq, Eq)]
pub enum Violation {
    /// A node's type is not known to the resolver.
    UnknownType { node: NodeId },
    /// An edge references an output port index the source type does not have.
    BadOutputPort {
        node: NodeId,
        port: u16,
        n_outputs: usize,
    },
    /// An edge references an input port index the target type does not have.
    BadInputPort {
        node: NodeId,
        port: u16,
        n_inputs: usize,
    },
    /// The connected port types are not directly compatible (domain/dim/clock).
    /// A clock/domain crossing must be a dedicated node, not a plain edge.
    TypeMismatch {
        from: (NodeId, u16),
        to: (NodeId, u16),
    },
    /// A `Stateful` (push) node feeds a presentation (pull) node directly —
    /// the membrane crossing must go through an export, not a plain edge.
    Membrane {
        from: (NodeId, u16),
        to: (NodeId, u16),
    },
    /// A per-instance param override names a parameter the node's type does not
    /// declare (a typo, or a stale name after a manifest change). Surfaced here
    /// rather than silently ignored at cook time — the override would otherwise
    /// read as the manifest default with no signal that the authored value was
    /// dropped.
    UnknownParam { node: NodeId, param: String },
}

/// Editor-space position of a node. Pure layout — never affects cook.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

// `Clone`/`PartialEq` back the Motion Nodes snapshot history (`ph2d-motion-doc`
// `MotionHistory`, molde `ph2d-vec-edit::History`): undo/redo snapshots the whole
// document (graph included) and `commit_if_changed` compares pre != current.
// `Debug` for test/diagnostic ergonomics. All fields already derive the three;
// `Eq` is intentionally NOT derived (`Pos`/`node_params` carry `f32`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    nodes: Vec<NodeInstance>,
    edges: Vec<Edge>,
    layout: BTreeMap<NodeId, Pos>,
    /// Per-instance parameter overrides, keyed by node then param name. Unlike
    /// [`layout`](Self::layout) these are **semantic** — they change the cooked
    /// result — so they live in the semantic section of the textual format.
    /// A node with no entry cooks at its manifest defaults. `BTreeMap` for
    /// deterministic iteration (stable diff / serialization, ADR-0032 §6).
    node_params: BTreeMap<NodeId, BTreeMap<String, f32>>,
    /// Per-node **text** params (e.g. an expression node's formula) — the additive
    /// string channel that keeps the FROZEN `NodeManifest` (f32 `ParamSpec` only,
    /// ADR-0039) intact: a node reads its own text via
    /// [`crate::cook::EvalCtx::text_param`], absent → the node's own default. This
    /// is the isolation-preserving realisation of M4.N1 (ParamSpec-tipado) — no
    /// contract bump, no fan-out breakage (docs/Motion Nodes/32). Persisted by both
    /// `Clone`/`PartialEq` (undo) and the textual format (the `x` record, which bumps
    /// the header to `v2` — [`crate::format`]).
    node_text_params: BTreeMap<NodeId, BTreeMap<String, String>>,
    /// **Driven parameters** — a param fed by a node's output instead of a constant
    /// ([`crate::param_source`], doc 58). The same additive move as `node_text_params`,
    /// for the same reason: `NodeManifest.inputs` is `&'static` (ADR-0039), so a node
    /// cannot grow a port, and a *driven* param is not a port — it is an **edge the
    /// manifest does not know about**, which is document state, which lives here.
    ///
    /// It is a real dependency: [`Graph::would_cycle`] walks it, the cook cooks it, and
    /// [`Graph::remove_node`] cleans it up on both sides.
    param_sources: crate::param_source::All,
    /// **The artist's name for a node** (doc 61) — what the card says instead of its type's
    /// display name. Absent (or empty) → the card falls back to the type, which is what it
    /// always did, so a graph nobody renamed serializes byte for byte as before.
    ///
    /// It is here, in a parallel map, rather than as a field on [`NodeInstance`], for the
    /// same reason `node_text_params` is: this is a **foundational** crate that several lines
    /// extend at once, and an append-only map is a merge that never conflicts while a new
    /// struct field touches every construction site in the repo
    /// ([[feedback_foundational_editable_design_for_isolation]]).
    ///
    /// A label is **not** semantic — no cook reads it — but it is *authored*, so it lives
    /// above `[layout]` in the textual format (the `t` record, header `v4`) and it rides
    /// `Clone`/`PartialEq`, which is what puts a rename in the undo queue for free.
    node_labels: BTreeMap<NodeId, String>,
    /// **Nodes switched OFF** (bypass/mute — the Blender H, the Nuke disable). A bypassed node
    /// does not run its op: the cook ([`crate::cook`]) passes its primary input (port 0) straight
    /// to its primary output (port 0), every other output going `Empty`. It is **semantic** — it
    /// changes the cooked result — so it enters the node's cook fingerprint and sits in the
    /// semantic section of the textual format (the `y` record, header `v5`). Absent → the node
    /// runs normally, so a graph nobody muted cooks and serializes byte for byte as before.
    ///
    /// Same append-only, foundational-safe move as `node_text_params`/`node_labels`: a parallel
    /// **set**, not a field on [`NodeInstance`], so a new line extending the graph never conflicts
    /// on a construction site ([[feedback_foundational_editable_design_for_isolation]]). It rides
    /// `Clone`/`PartialEq`, which is what puts a bypass toggle in the undo queue for free.
    node_bypassed: BTreeSet<NodeId>,
    next_id: u32,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node of the given canonical type name; returns its fresh id.
    /// Panics on an invalid name (empty or containing whitespace) — a
    /// programmer error that would otherwise corrupt the whitespace-delimited
    /// textual format ([`crate::format`]).
    pub fn add_node(&mut self, type_name: impl Into<String>) -> NodeId {
        let type_name = type_name.into();
        assert!(
            is_valid_type_name(&type_name),
            "node type name must be non-empty and whitespace-free: {type_name:?}"
        );
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push(NodeInstance { id, type_name });
        id
    }

    /// Insert a node with an explicit id (used by the textual-format parser to
    /// preserve stable ids on load). Bumps `next_id` past it. Saturating add
    /// guards against a corrupt/adversarial file using `id == u32::MAX`.
    pub fn insert_raw(&mut self, id: NodeId, type_name: impl Into<String>) {
        let type_name = type_name.into();
        assert!(
            is_valid_type_name(&type_name),
            "node type name must be non-empty and whitespace-free: {type_name:?}"
        );
        self.nodes.push(NodeInstance { id, type_name });
        self.next_id = self.next_id.max(id.0.saturating_add(1));
    }

    /// Connect two ports, enforcing the **structural** invariants:
    /// - both endpoints exist;
    /// - the target input port is not already connected (one edge per input);
    /// - a non-`delayed` edge does not close a cycle (feedback must be `pre`).
    ///
    /// Acyclicity is thus an invariant of the stored graph; nothing else in the
    /// engine performs cycle detection. **Semantic** typing and the membrane
    /// are checked separately by [`Graph::validate`] (they need the manifests).
    pub fn connect(&mut self, edge: Edge) -> Result<(), EdgeError> {
        if !self.contains(edge.from.0) || !self.contains(edge.to.0) {
            return Err(EdgeError::UnknownNode);
        }
        if self.edges.iter().any(|e| e.to == edge.to) {
            return Err(EdgeError::InputAlreadyConnected);
        }
        if !edge.delayed && self.would_cycle(edge.from.0, edge.to.0) {
            return Err(EdgeError::WouldCycle);
        }
        self.edges.push(edge);
        Ok(())
    }

    /// Remove the edge feeding input port `(to, port)`, returning it if present.
    /// An input port takes at most one edge ([`Graph::connect`]), so this is the
    /// unambiguous inverse of a `connect` into that port — the editor's alt-click
    /// disconnect resolves a wire to its unique target input. Removing an edge
    /// only relaxes the invariants (acyclicity, one-edge-per-input), so no
    /// re-validation is needed.
    pub fn disconnect(&mut self, to: NodeId, port: u16) -> Option<Edge> {
        self.edges
            .iter()
            .position(|e| e.to == (to, port))
            .map(|i| self.edges.remove(i))
    }

    /// Remove node `id` and everything that references it: its instance, every
    /// incident edge (as source **or** target), its layout position, and its
    /// per-instance param overrides. Returns `true` iff the node existed.
    ///
    /// A no-op (returns `false`, touches nothing) when `id` is not a node, so a
    /// stale selection id from the editor deletes cleanly. Only removes, so the
    /// structural invariants are preserved (the remaining edges were already
    /// acyclic and one-per-input).
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        let before = self.nodes.len();
        self.nodes.retain(|n| n.id != id);
        if self.nodes.len() == before {
            return false;
        }
        self.edges.retain(|e| e.from.0 != id && e.to.0 != id);
        self.layout.remove(&id);
        self.node_params.remove(&id);
        self.node_text_params.remove(&id);
        self.node_labels.remove(&id);
        self.node_bypassed.remove(&id);
        // Both sides: the params IT drove, and the params driven BY it. A source left
        // pointing at a deleted node would cook as `Empty` forever — a socket wired to a
        // ghost.
        self.param_sources.remove(&id);
        for sources in self.param_sources.values_mut() {
            sources.retain(|_, (src, _)| *src != id);
        }
        self.param_sources.retain(|_, s| !s.is_empty());
        true
    }

    pub fn set_pos(&mut self, id: NodeId, pos: Pos) {
        self.layout.insert(id, pos);
    }

    pub fn pos(&self, id: NodeId) -> Option<Pos> {
        self.layout.get(&id).copied()
    }

    /// Set a per-instance parameter override for `id`, replacing any previous
    /// value for `name`. Overrides the node type's manifest default at cook
    /// time (read via [`crate::cook::EvalCtx::param`]). A `name` the node's type
    /// does not declare is **not** rejected here (the graph holds no manifests)
    /// — [`Graph::validate`] surfaces it as [`Violation::UnknownParam`].
    ///
    /// Panics in debug on a non-finite `value`: an override is authored data and
    /// `NaN`/`±∞` are never a legitimate parameter (the textual parser rejects
    /// them at load, so this guards the in-code path).
    ///
    /// Lenient on `id` (does not check the node exists), mirroring
    /// [`Graph::set_pos`]; an override on a non-existent node is dead data the
    /// cook never reads. The textual parser is stricter — it rejects a `p`
    /// record whose id has no node — so a corrupt file cannot smuggle one in.
    pub fn set_param(&mut self, id: NodeId, name: impl Into<String>, value: f32) {
        debug_assert!(value.is_finite(), "param override must be finite: {value}");
        self.node_params
            .entry(id)
            .or_default()
            .insert(name.into(), value);
    }

    /// The per-instance param overrides for `id` (none if untouched). The cook
    /// resolves a param as this map's value if present, else the manifest
    /// default.
    pub fn node_param_overrides(&self, id: NodeId) -> Option<&BTreeMap<String, f32>> {
        self.node_params.get(&id)
    }

    /// All per-instance param overrides, keyed by node id then param name
    /// (deterministic order). Used by the textual format; cook reads a single
    /// node's via [`Graph::node_param_overrides`].
    pub fn node_params(&self) -> &BTreeMap<NodeId, BTreeMap<String, f32>> {
        &self.node_params
    }

    /// Set a per-node **text** param (e.g. an expression node's formula). The
    /// additive string channel that keeps the frozen `NodeManifest` intact — read
    /// at cook time via [`crate::cook::EvalCtx::text_param`]. Not validated against
    /// the manifest (text params are free-form; a node reads whatever key it
    /// wants). See the `node_text_params` field doc.
    pub fn set_text_param(
        &mut self,
        id: NodeId,
        name: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.node_text_params
            .entry(id)
            .or_default()
            .insert(name.into(), value.into());
    }

    /// The per-node text-param overrides for `id` (none if untouched). The cook
    /// threads this into [`crate::cook::EvalCtx`].
    pub fn node_text_param_overrides(&self, id: NodeId) -> Option<&BTreeMap<String, String>> {
        self.node_text_params.get(&id)
    }

    /// All per-node text-param overrides, keyed by node id then param name
    /// (deterministic `BTreeMap` order). Used by the textual format ([`crate::format`],
    /// the `x` record).
    pub fn node_text_params(&self) -> &BTreeMap<NodeId, BTreeMap<String, String>> {
        &self.node_text_params
    }

    // ── Bypass: nodes switched off (H) ──────────────────────────────────────

    /// Switch a node off (bypass/mute) or back on. A bypassed node's op never runs; the cook
    /// passes port 0 straight through. Idempotent; lenient on `id` like [`Graph::set_pos`] (a
    /// bypass on a non-existent node is dead data the cook never reads — the textual parser is
    /// stricter and rejects a `y` record whose id has no node).
    pub fn set_bypassed(&mut self, id: NodeId, on: bool) {
        if on {
            self.node_bypassed.insert(id);
        } else {
            self.node_bypassed.remove(&id);
        }
    }

    /// Toggle a node's bypass, returning the NEW state (`true` = now switched off).
    pub fn toggle_bypass(&mut self, id: NodeId) -> bool {
        if self.node_bypassed.remove(&id) {
            false
        } else {
            self.node_bypassed.insert(id);
            true
        }
    }

    /// Is this node switched off? Read by the cook (passthrough) and the snapshot (dimmed card).
    pub fn node_bypassed(&self, id: NodeId) -> bool {
        self.node_bypassed.contains(&id)
    }

    /// Every bypassed node id, in deterministic order. Used by the textual format (the `y`
    /// record); the cook reads a single node's state via [`Graph::node_bypassed`].
    pub fn bypassed_nodes(&self) -> &BTreeSet<NodeId> {
        &self.node_bypassed
    }

    // ── Labels: the artist's name for a node (doc 61) ───────────────────────

    /// Name a node. An **empty** name is not a name: it removes the label, so clearing the
    /// rename box means *"call it what it is"* rather than leaving a card with a blank
    /// title. This is also what keeps `to_text` byte-identical for a graph that was renamed
    /// and un-renamed.
    ///
    /// Whitespace is trimmed, and interior newlines are refused (the textual format is
    /// line-oriented: a label with a newline in it would be a second, unparsable record).
    pub fn set_label(&mut self, id: NodeId, label: impl Into<String>) {
        let label = label.into();
        let label = label.trim();
        if label.is_empty() || label.contains(['\n', '\r']) {
            self.node_labels.remove(&id);
        } else {
            self.node_labels.insert(id, label.to_string());
        }
    }

    /// The artist's name for this node, if it has one. `None` → the card shows its type's
    /// display name (the one derivation both the paint and the rename box read, so what you
    /// see is what the box seeds with).
    pub fn label(&self, id: NodeId) -> Option<&str> {
        self.node_labels.get(&id).map(String::as_str)
    }

    /// Every label, in deterministic id order. Used by the textual format (the `t` record).
    pub fn node_labels(&self) -> &BTreeMap<NodeId, String> {
        &self.node_labels
    }

    // ── Driven params (doc 58) ──────────────────────────────────────────────
    //
    // The verbs are deliberately shaped like the edge verbs (`connect` / `disconnect`),
    // because that is what they are: an edge onto a parameter. There is no separate
    // "promote" — the wire IS the promotion, and pulling it off takes the socket away
    // (doc 58; the derived-interface rule of doc 57 §3, applied a second time).

    /// **Drive parameter `param` of `node` from `src`.** Replaces any previous source.
    ///
    /// Enforces the same structural invariants a `connect` does — both endpoints exist,
    /// and it does not close a cycle — because it IS a dependency: the cook will recurse
    /// through it. (A param has no *"already connected"* case: setting a second source
    /// replaces the first, exactly as re-plugging an input socket would.)
    ///
    /// The param NAME is not checked here (the graph holds no manifests, the same reason
    /// `set_param` cannot check it); `Graph::validate` surfaces an unknown one.
    pub fn drive_param(
        &mut self,
        node: NodeId,
        param: impl Into<String>,
        src: crate::param_source::Source,
    ) -> Result<(), EdgeError> {
        if !self.contains(node) || !self.contains(src.0) {
            return Err(EdgeError::UnknownNode);
        }
        if src.0 == node || self.would_cycle(src.0, node) {
            return Err(EdgeError::WouldCycle);
        }
        self.param_sources
            .entry(node)
            .or_default()
            .insert(param.into(), src);
        Ok(())
    }

    /// Stop driving `param` — the node goes back to its override, else its manifest
    /// default. Returns the source that was there. The socket vanishes with it.
    pub fn undrive_param(
        &mut self,
        node: NodeId,
        param: &str,
    ) -> Option<crate::param_source::Source> {
        let sources = self.param_sources.get_mut(&node)?;
        let was = sources.remove(param);
        if sources.is_empty() {
            self.param_sources.remove(&node);
        }
        was
    }

    /// What drives `node`'s params — sorted by param name, which is what makes socket `k`
    /// mean the same parameter every frame.
    pub fn param_sources(&self, node: NodeId) -> Option<&crate::param_source::Sources> {
        self.param_sources.get(&node)
    }

    /// Every driven param in the document (deterministic order). Used by the textual
    /// format ([`crate::format`], the `d` record) and by the view's fold.
    pub fn all_param_sources(&self) -> &crate::param_source::All {
        &self.param_sources
    }

    fn contains(&self, id: NodeId) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    /// Would adding a non-delayed edge `from -> to` close a cycle? True iff
    /// `from` is already reachable from `to` along non-delayed edges.
    fn would_cycle(&self, from: NodeId, to: NodeId) -> bool {
        if from == to {
            return true;
        }
        let mut stack = vec![to];
        let mut seen: Vec<NodeId> = Vec::new();
        while let Some(cur) = stack.pop() {
            if cur == from {
                return true;
            }
            if seen.contains(&cur) {
                continue;
            }
            seen.push(cur);
            for e in &self.edges {
                if !e.delayed && e.from.0 == cur {
                    stack.push(e.to.0);
                }
            }
            // A driven param is a dependency like any other, so it can close a cycle like
            // any other — and a cycle the check does not see is not a refused connect, it
            // is the cook recursing until the stack runs out.
            for (node, sources) in &self.param_sources {
                if sources.values().any(|(src, _)| *src == cur) {
                    stack.push(*node);
                }
            }
        }
        false
    }

    pub fn node(&self, id: NodeId) -> Option<&NodeInstance> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// The edge feeding `(node, port)`, if any: `(source node, source port,
    /// delayed)`. Assumes at most one edge per input port.
    pub fn input_edge(&self, node: NodeId, port: usize) -> Option<(NodeId, usize, bool)> {
        self.edges
            .iter()
            .find(|e| e.to == (node, port as u16))
            .map(|e| (e.from.0, e.from.1 as usize, e.delayed))
    }

    pub fn nodes(&self) -> &[NodeInstance] {
        &self.nodes
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn layout(&self) -> &BTreeMap<NodeId, Pos> {
        &self.layout
    }

    /// Check the **semantic** invariants of every edge against the node
    /// manifests resolved through `ops`: that referenced ports exist, that the
    /// connected port types are directly compatible, and that the membrane
    /// holds (no `Stateful` → presentation by a plain edge). Returns every
    /// violation found (not just the first), so an editor can surface them all.
    ///
    /// This is where the ADR-0030 membrane and the algebraic port types are
    /// actually enforced; `connect` only guarantees structure (acyclicity,
    /// one-edge-per-input). Run after editing or loading a graph.
    pub fn validate(&self, ops: &dyn OpResolver) -> Result<(), Vec<Violation>> {
        let mut violations = Vec::new();
        for e in &self.edges {
            let from_man = self
                .node(e.from.0)
                .and_then(|n| ops.resolve(n.type_id()))
                .map(|o| o.manifest());
            let to_man = self
                .node(e.to.0)
                .and_then(|n| ops.resolve(n.type_id()))
                .map(|o| o.manifest());

            let (Some(fm), Some(tm)) = (from_man, to_man) else {
                if from_man.is_none() {
                    violations.push(Violation::UnknownType { node: e.from.0 });
                }
                if to_man.is_none() {
                    violations.push(Violation::UnknownType { node: e.to.0 });
                }
                continue;
            };

            let out = fm.outputs.get(e.from.1 as usize);
            let inp = tm.inputs.get(e.to.1 as usize);
            if out.is_none() {
                violations.push(Violation::BadOutputPort {
                    node: e.from.0,
                    port: e.from.1,
                    n_outputs: fm.outputs.len(),
                });
            }
            if inp.is_none() {
                violations.push(Violation::BadInputPort {
                    node: e.to.0,
                    port: e.to.1,
                    n_inputs: tm.inputs.len(),
                });
            }
            if let (Some(o), Some(i)) = (out, inp) {
                if !o.ty.connects_directly(i.ty) {
                    violations.push(Violation::TypeMismatch {
                        from: e.from,
                        to: e.to,
                    });
                }
                if !fm.effect.can_feed(tm.effect) {
                    violations.push(Violation::Membrane {
                        from: e.from,
                        to: e.to,
                    });
                }
            }
        }

        // Per-instance param overrides must name a declared param of the node's
        // type — otherwise the authored value is silently dropped at cook time
        // (the cook would read the manifest default). Checked against the
        // resolved manifest; a node whose type does not resolve is left to the
        // edge loop above (if connected) or is a harmless isolated node.
        for (&node, overrides) in &self.node_params {
            let Some(manifest) = self
                .node(node)
                .and_then(|n| ops.resolve(n.type_id()))
                .map(|o| o.manifest())
            else {
                continue;
            };
            for name in overrides.keys() {
                if manifest.param_default(name).is_none() {
                    violations.push(Violation::UnknownParam {
                        node,
                        param: name.clone(),
                    });
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
