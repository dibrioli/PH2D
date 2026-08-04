#![forbid(unsafe_code)]
//! **The setup DIAGNOSER for the Motion graph** (ADR-0155).
//!
//! The Motion graph has a class of error that produces no error. The canonical
//! case the Enio named: a `force.*` node is `Pure` and accumulates into the
//! transient column `accel`; the only nodes that consume `accel` are the
//! integrators (`motion.integrate`, `sim.step`). A force wired toward the sink
//! with **no integrator on the path** writes `accel`, nothing reads it, and the
//! scene stays static — with no error and no warning. `Graph::validate` checks
//! only port types and membranes; it has no reachability analysis.
//!
//! [`diagnose`] is that analysis. It is a **pure** function of the graph structure
//! and the registry's [`Coupling`] side-channel — the same single door the auto-heal
//! (W2) and the badges (W3) will read, so they cannot diverge from what the gates
//! test. For every node that `Produces(col)`, it walks forward (non-`delayed`)
//! edges: if some reachable node `Consumes(col)`, the producer is healthy; if not,
//! the producer is **inert**, and the [`Fix`] says whether the cure is to insert the
//! canonical plumbing ([`Fix::Insert`], the AUTO-HEAL case), to reorder against an
//! integrator that exists but sits off the path ([`Fix::Reorder`]), or to merely
//! offer a choice with no canonical answer ([`Fix::Offer`]).

use ph2d_node_registry::{Coupling, NodeRegistry};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use std::collections::BTreeSet;

/// One diagnosed defect: a node whose placement makes its output inert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The offending producer.
    pub node: NodeId,
    /// What is wrong.
    pub deficit: Deficit,
    /// How to fix it (and how aggressively the editor may act — see [`Fix`]).
    pub fix: Fix,
}

/// The kind of defect found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deficit {
    /// This node writes the named transient column, and no node reachable
    /// downstream (via forward, non-`delayed` edges) consumes it — so it does
    /// nothing.
    InertProducer(&'static str),
}

/// The suggested cure, carrying how confidently the editor may apply it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Insert this canonical consumer node type to make the producer live
    /// (`accel` → `motion.integrate`, or `sim.step` in a particle chain). The
    /// **AUTO-HEAL** candidate — unambiguous plumbing the artist forgot.
    Insert(&'static str),
    /// A consumer of this column exists in the graph, but not on the producer's
    /// forward path — the cure is to REORDER (put the producer upstream of it),
    /// never to insert a second one (one integrator applies). An **OFFER**.
    Reorder,
    /// The missing consumer is a creative choice with no canonical inserter
    /// (`inv_mass` needs *a* solver; `falloff` needs *a* force/deformer) — surface
    /// it, never guess. An **OFFER/AVISO**.
    Offer,
}

/// Walk the graph and report every node whose output is semantically inert
/// (ADR-0155). Pure: reads only the graph structure and the registry's
/// [`Coupling`] side-channel. A node with no couplings is neutral and never
/// diagnosed; a `Produces(col)` with a `Consumes(col)` reachable downstream is
/// healthy and reported nowhere.
#[must_use]
pub fn diagnose(graph: &Graph, reg: &NodeRegistry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for inst in graph.nodes() {
        let Some(couplings) = reg.couplings(NodeTypeId::of(&inst.type_name)) else {
            continue;
        };
        for c in couplings {
            let Coupling::Produces(col) = *c else { continue };
            if consumer_reachable(graph, reg, inst.id, col) {
                continue; // healthy: something downstream reads it
            }
            let fix = if consumer_exists_anywhere(graph, reg, col) {
                // An integrator/solver exists but off this producer's path: the
                // answer is to reorder, not to insert a *second* one.
                Fix::Reorder
            } else if let Some(consumer) = canonical_consumer(col, particle_upstream(graph, inst.id))
            {
                Fix::Insert(consumer)
            } else {
                Fix::Offer
            };
            out.push(Diagnostic {
                node: inst.id,
                deficit: Deficit::InertProducer(col),
                fix,
            });
        }
    }
    out
}

/// The canonical PLUMBING node that consumes `col`, when the cure is an
/// unambiguous insert. Only `accel` has one (the integrator); `inv_mass`/`falloff`
/// are creative choices with more than one reasonable consumer, so they return
/// `None` and become an [`Fix::Offer`]. The single door for "what heals this
/// column?" — the auto-heal (W2) will ask the same function.
#[must_use]
pub fn canonical_consumer(col: &str, particle: bool) -> Option<&'static str> {
    match (col, particle) {
        ("accel", true) => Some("sim.step"),
        ("accel", false) => Some("motion.integrate"),
        _ => None,
    }
}

/// Is a node that `Consumes(col)` reachable from `from` via forward
/// (non-`delayed`) edges? `from` itself is excluded — a node does not heal its own
/// output.
fn consumer_reachable(graph: &Graph, reg: &NodeRegistry, from: NodeId, col: &str) -> bool {
    let mut seen = BTreeSet::new();
    seen.insert(from);
    let mut stack = vec![from];
    while let Some(n) = stack.pop() {
        for e in graph.edges() {
            if e.from.0 == n && !e.delayed && seen.insert(e.to.0) {
                if consumes(graph, reg, e.to.0, col) {
                    return true;
                }
                stack.push(e.to.0);
            }
        }
    }
    false
}

/// Does any node in the whole graph `Consumes(col)`? (Distinguishes "no consumer
/// at all → insert" from "a consumer exists but off my path → reorder".)
fn consumer_exists_anywhere(graph: &Graph, reg: &NodeRegistry, col: &str) -> bool {
    graph.nodes().iter().any(|inst| {
        reg.couplings(NodeTypeId::of(&inst.type_name))
            .is_some_and(|cs| cs.iter().any(|c| matches!(c, Coupling::Consumes(x) if *x == col)))
    })
}

/// Does the node with this id `Consumes(col)`?
fn consumes(graph: &Graph, reg: &NodeRegistry, node: NodeId, col: &str) -> bool {
    graph
        .nodes()
        .iter()
        .find(|n| n.id == node)
        .and_then(|n| reg.couplings(NodeTypeId::of(&n.type_name)))
        .is_some_and(|cs| cs.iter().any(|c| matches!(c, Coupling::Consumes(x) if *x == col)))
}

/// Is a `sim.spawn` upstream of `node` (feeding it via forward edges)? A
/// `sim.spawn` chain wants `sim.step`; everything else wants `motion.integrate`.
fn particle_upstream(graph: &Graph, node: NodeId) -> bool {
    let spawn = NodeTypeId::of("sim.spawn");
    let mut seen = BTreeSet::new();
    seen.insert(node);
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if graph
            .nodes()
            .iter()
            .any(|i| i.id == n && NodeTypeId::of(&i.type_name) == spawn)
        {
            return true;
        }
        for e in graph.edges() {
            if e.to.0 == n && !e.delayed && seen.insert(e.from.0) {
                stack.push(e.from.0);
            }
        }
    }
    false
}
