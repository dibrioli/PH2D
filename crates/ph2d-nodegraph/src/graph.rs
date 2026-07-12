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
use std::collections::BTreeMap;

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
mod tests {
    use super::*;

    fn edge(from: NodeId, to: NodeId, delayed: bool) -> Edge {
        Edge {
            from: (from, 0),
            to: (to, 0),
            delayed,
        }
    }

    #[test]
    fn plain_back_edge_is_rejected_but_pre_is_allowed() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        assert_eq!(g.connect(edge(a, b, false)), Ok(()));
        // b -> a as a plain edge would close a cycle: rejected.
        assert_eq!(g.connect(edge(b, a, false)), Err(EdgeError::WouldCycle));
        // b -> a as a `pre` (delayed) edge is the legal way to express feedback.
        assert_eq!(g.connect(edge(b, a, true)), Ok(()));
        assert_eq!(g.edges().len(), 2);
    }

    #[test]
    fn unknown_node_is_rejected() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        assert_eq!(
            g.connect(edge(a, NodeId(999), false)),
            Err(EdgeError::UnknownNode)
        );
    }

    #[test]
    fn self_edge_is_a_cycle() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        assert_eq!(g.connect(edge(a, a, false)), Err(EdgeError::WouldCycle));
    }

    #[test]
    #[should_panic(expected = "whitespace-free")]
    fn add_node_rejects_whitespaced_name() {
        // Would corrupt the whitespace-delimited textual format.
        Graph::new().add_node("motion clone");
    }

    #[test]
    fn duplicate_input_edge_is_rejected() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.connect(Edge {
            from: (a, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        // A second edge into the same input port (c, 0) is rejected.
        assert_eq!(
            g.connect(Edge {
                from: (b, 0),
                to: (c, 0),
                delayed: false
            }),
            Err(EdgeError::InputAlreadyConnected)
        );
    }

    #[test]
    fn disconnect_removes_the_edge_into_an_input() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.connect(Edge {
            from: (a, 0),
            to: (b, 1),
            delayed: false,
        })
        .unwrap();
        assert_eq!(g.edges().len(), 1);
        // Wrong port → nothing removed.
        assert_eq!(g.disconnect(b, 0), None);
        assert_eq!(g.edges().len(), 1);
        // Right port → the edge comes back out.
        assert_eq!(
            g.disconnect(b, 1),
            Some(Edge {
                from: (a, 0),
                to: (b, 1),
                delayed: false
            })
        );
        assert!(g.edges().is_empty());
        // The port is free again — a fresh connect is accepted.
        assert_eq!(
            g.connect(Edge {
                from: (a, 0),
                to: (b, 1),
                delayed: false
            }),
            Ok(())
        );
    }

    #[test]
    fn remove_node_drops_node_incident_edges_layout_and_params() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.connect(edge(a, b, false)).unwrap();
        g.connect(edge(b, c, false)).unwrap();
        g.set_pos(b, Pos { x: 1.0, y: 2.0 });
        g.set_param(b, "k", 3.0);

        assert!(g.remove_node(b));
        // Node gone.
        assert!(g.node(b).is_none());
        assert_eq!(g.nodes().len(), 2);
        // Both edges incident on `b` (a→b and b→c) are gone; none reference `b`.
        assert!(g.edges().is_empty());
        // Layout + param overrides for `b` are gone.
        assert!(g.pos(b).is_none());
        assert!(g.node_param_overrides(b).is_none());
        // Untouched neighbours survive.
        assert!(g.node(a).is_some() && g.node(c).is_some());
        // Removing a non-existent node is a no-op.
        assert!(!g.remove_node(b));
        assert!(!g.remove_node(NodeId(999)));
    }

    #[test]
    fn input_edge_resolves_source() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.connect(Edge {
            from: (a, 0),
            to: (b, 1),
            delayed: false,
        })
        .unwrap();
        assert_eq!(g.input_edge(b, 1), Some((a, 0, false)));
        assert_eq!(g.input_edge(b, 0), None);
    }
}
