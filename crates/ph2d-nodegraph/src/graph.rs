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

/// A typed edge from one node's output port to another's input port.
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
        self.nodes.push(NodeInstance { id, type_name: type_name.into() });
        id
    }

    /// Insert a node with an explicit id (used by the textual-format parser to
    /// preserve stable ids on load). Bumps `next_id` past it.
    pub fn insert_raw(&mut self, id: NodeId, type_name: impl Into<String>) {
        self.nodes.push(NodeInstance { id, type_name: type_name.into() });
        self.next_id = self.next_id.max(id.0 + 1);
    }

    /// Connect two ports. Rejects an edge that would create a cycle unless it
    /// is `delayed` (a `pre` edge). Acyclicity is thus an invariant of the
    /// stored graph; nothing else in the engine performs cycle detection.
    pub fn connect(&mut self, edge: Edge) -> Result<(), EdgeError> {
        if !self.contains(edge.from.0) || !self.contains(edge.to.0) {
            return Err(EdgeError::UnknownNode);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(from: NodeId, to: NodeId, delayed: bool) -> Edge {
        Edge { from: (from, 0), to: (to, 0), delayed }
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
        assert_eq!(g.connect(edge(a, NodeId(999), false)), Err(EdgeError::UnknownNode));
    }

    #[test]
    fn self_edge_is_a_cycle() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        assert_eq!(g.connect(edge(a, a, false)), Err(EdgeError::WouldCycle));
    }

    #[test]
    fn input_edge_resolves_source() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.connect(Edge { from: (a, 0), to: (b, 1), delayed: false }).unwrap();
        assert_eq!(g.input_edge(b, 1), Some((a, 0, false)));
        assert_eq!(g.input_edge(b, 0), None);
    }
}
