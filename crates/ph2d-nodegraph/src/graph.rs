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
}

/// Editor-space position of a node. Pure layout — never affects cook.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Default)]
pub struct Graph {
    nodes: Vec<NodeInstance>,
    edges: Vec<Edge>,
    layout: BTreeMap<NodeId, Pos>,
    next_id: u32,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node of the given canonical type name; returns its fresh id.
    pub fn add_node(&mut self, type_name: impl Into<String>) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push(NodeInstance {
            id,
            type_name: type_name.into(),
        });
        id
    }

    /// Insert a node with an explicit id (used by the textual-format parser to
    /// preserve stable ids on load). Bumps `next_id` past it. Saturating add
    /// guards against a corrupt/adversarial file using `id == u32::MAX`.
    pub fn insert_raw(&mut self, id: NodeId, type_name: impl Into<String>) {
        self.nodes.push(NodeInstance {
            id,
            type_name: type_name.into(),
        });
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

    pub fn set_pos(&mut self, id: NodeId, pos: Pos) {
        self.layout.insert(id, pos);
    }

    pub fn pos(&self, id: NodeId) -> Option<Pos> {
        self.layout.get(&id).copied()
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
