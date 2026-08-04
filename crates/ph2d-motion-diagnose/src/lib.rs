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
//! [`diagnose`] is that analysis, and it does NOT keep a parallel hand-written
//! table of which node touches which column. It **derives** the produces/consumes
//! roles from the node's OWN declaration — the same one the cook trusts:
//!
//! - **The GPU `ColumnBinding` set** (`reg.gpu_kernel(ty).bindings`) is what the
//!   device sequencer reads to bind buffers, so it cannot drift from the cook. A
//!   binding that WRITES a column produces it; a binding that READS but does not
//!   write it consumes it. This is the gold-standard "derive, don't declare"
//!   (Blender geometry-node field dependency; Houdini's cook read/create/modify)
//!   and it covers every GPU-resident node — the whole `falloff` family of
//!   fields/forces/deformers — for free, with zero annotation.
//! - **The `Coupling` side-channel** (`reg.couplings(ty)`) covers the handful of
//!   CPU-only nodes that read a column only at eval runtime (no GPU kernel to
//!   derive from) — the `fx.*`/`motion.step`/… falloff consumers.
//!
//! The **re-producer rule** is the load-bearing subtlety: a node that reads AND
//! writes a column (a force accumulating `accel`, a `field.combine` composing
//! `falloff`) is a producer, NOT a consumer. That is exactly what keeps two
//! forces with no integrator inert — the second force reads `accel` but re-writes
//! it, so it does not "save" the first.
//!
//! Only [`TRANSIENT_COLUMNS`] are subject to the analysis: the columns NOT lowered
//! to instances, whose producers are inert without a graph-internal consumer.
//! `P`/`size`/`rot`/`tint` are consumed by the output, so wiring them to it is
//! never inert.
//!
//! For every producer of a transient column, [`diagnose`] walks forward
//! (non-`delayed`) edges: if some reachable node consumes the column, the producer
//! is healthy; if not, it is **inert**, and the [`Fix`] says whether the cure is to
//! insert the canonical plumbing ([`Fix::Insert`], the AUTO-HEAL case), to reorder
//! against an off-path consumer ([`Fix::Reorder`]), or to merely offer a choice
//! with no canonical answer ([`Fix::Offer`], the `falloff`/`inv_mass` advisory).

use ph2d_node_registry::{Coupling, NodeRegistry};
use ph2d_nodegraph::column::ColumnAccess;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use std::collections::BTreeSet;

/// The columns that are **not lowered to instances** — the transient-channel
/// convention (`ph2d_nodegraph::column`). A producer of one of these is inert
/// unless a graph-internal node consumes it; a producer of a LOWERED column
/// (`P`/`size`/`rot`/`tint`, read by the output) is never subject to this
/// analysis, which is why the set is a filter and not "every produced column".
///
/// `accel`/`inv_mass` are the columns some node **`Consume`-drops** (a gate proves
/// this set covers every such column, so a new transient column cannot be added
/// without landing here); `falloff` is the modulation weight — pure-read by
/// forces/deformers, never dropped, never lowered, so it is named directly.
const TRANSIENT_COLUMNS: &[&str] = &["accel", "falloff", "inv_mass"];

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
/// (ADR-0155). Pure: reads only the graph structure and the registry's derived
/// roles (GPU bindings + [`Coupling`] side-channel). A node that produces no
/// transient column is neutral and never diagnosed; a producer of a transient
/// column with a consumer reachable downstream is healthy and reported nowhere.
#[must_use]
pub fn diagnose(graph: &Graph, reg: &NodeRegistry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for inst in graph.nodes() {
        let ty = NodeTypeId::of(&inst.type_name);
        for &col in TRANSIENT_COLUMNS {
            if !produces(reg, ty, col) {
                continue;
            }
            if consumer_reachable(graph, reg, inst.id, col) {
                continue; // healthy: something downstream reads it
            }
            let fix = if consumer_exists_anywhere(graph, reg, col) {
                // A consumer exists but off this producer's path: the answer is to
                // reorder, not to insert a *second* one.
                Fix::Reorder
            } else if let Some(consumer) =
                canonical_consumer(col, particle_upstream(graph, inst.id))
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
/// column?" — the auto-heal (W2) asks the same function.
#[must_use]
pub fn canonical_consumer(col: &str, particle: bool) -> Option<&'static str> {
    match (col, particle) {
        ("accel", true) => Some("sim.step"),
        ("accel", false) => Some("motion.integrate"),
        _ => None,
    }
}

/// Does the node type `ty` **produce** `col` — write it to its output? United from
/// the two static, registry-queryable sources: a GPU `ColumnBinding` that writes
/// it, or a `Coupling::Produces`.
fn produces(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    coupling_produces(reg, ty, col) || gpu_binding(reg, ty, col, is_producer)
}

/// Does the node type `ty` **consume** `col` — read it without re-producing it (a
/// pure read, including `Consume`)? A re-producer (read + write, like a force
/// accumulating `accel` or a field composing `falloff`) is NOT a consumer — that
/// is what keeps two forces with no integrator inert.
fn consumes(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    coupling_consumes(reg, ty, col) || gpu_binding(reg, ty, col, |a| a.reads() && !is_producer(a))
}

/// Any access that can WRITE the column to the node's output. `ReadWriteExisting`
/// writes only when the input carries the column, but it is still a producer of it
/// (never a pure-read consumer) — the distinction between conditional and
/// unconditional writing does not matter for the transient columns, none of which
/// is touched by a `ReadWriteExisting` binding.
fn is_producer(a: ColumnAccess) -> bool {
    matches!(
        a,
        ColumnAccess::Write | ColumnAccess::ReadWrite | ColumnAccess::ReadWriteExisting
    )
}

/// Does `ty` declare a GPU `ColumnBinding` for `col` whose access satisfies `pred`?
/// This is the derived, drift-proof half of a role: the binding set is the same one
/// the device sequencer reads, so it can never disagree with the cook.
fn gpu_binding(
    reg: &NodeRegistry,
    ty: NodeTypeId,
    col: &str,
    pred: impl Fn(ColumnAccess) -> bool,
) -> bool {
    reg.gpu_kernel(ty)
        .is_some_and(|k| k.bindings.iter().any(|b| b.column == col && pred(b.access)))
}

/// The declared half of "produces": a `Coupling::Produces(col)` (the CPU-only
/// stragglers with no GPU kernel).
fn coupling_produces(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    reg.couplings(ty).is_some_and(|cs| {
        cs.iter()
            .any(|c| matches!(c, Coupling::Produces(x) if *x == col))
    })
}

/// The declared half of "consumes": a `Coupling::Consumes(col)`.
fn coupling_consumes(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    reg.couplings(ty).is_some_and(|cs| {
        cs.iter()
            .any(|c| matches!(c, Coupling::Consumes(x) if *x == col))
    })
}

/// Is a node that consumes `col` reachable from `from` via forward (non-`delayed`)
/// edges? `from` itself is excluded — a node does not heal its own output.
fn consumer_reachable(graph: &Graph, reg: &NodeRegistry, from: NodeId, col: &str) -> bool {
    let mut seen = BTreeSet::new();
    seen.insert(from);
    let mut stack = vec![from];
    while let Some(n) = stack.pop() {
        for e in graph.edges() {
            if e.from.0 == n && !e.delayed && seen.insert(e.to.0) {
                if node_consumes(graph, reg, e.to.0, col) {
                    return true;
                }
                stack.push(e.to.0);
            }
        }
    }
    false
}

/// Does any node in the whole graph consume `col`? (Distinguishes "no consumer at
/// all → insert" from "a consumer exists but off my path → reorder".)
fn consumer_exists_anywhere(graph: &Graph, reg: &NodeRegistry, col: &str) -> bool {
    graph
        .nodes()
        .iter()
        .any(|inst| consumes(reg, NodeTypeId::of(&inst.type_name), col))
}

/// Does the node with this id consume `col`? (Resolves id → type_name → role.)
fn node_consumes(graph: &Graph, reg: &NodeRegistry, node: NodeId, col: &str) -> bool {
    graph
        .nodes()
        .iter()
        .find(|n| n.id == node)
        .is_some_and(|n| consumes(reg, NodeTypeId::of(&n.type_name), col))
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::gpu::KernelResolver;

    /// **The transient set covers every column any node `Consume`-drops.** `accel`
    /// and `inv_mass` are derived facts — some node drops each — so this gate makes
    /// it impossible to introduce a new transient column (a new `Consume` binding)
    /// without landing it in [`TRANSIENT_COLUMNS`], where the diagnoser can reason
    /// about a producer of it. `falloff`, the modulation weight that is never
    /// dropped, is the one named directly, so the gate pins it too. FALSIFIED by
    /// removing "accel"/"inv_mass" (a `Consume` column escapes the analysis) or
    /// "falloff" (the modulation column stops being analysed).
    #[test]
    fn the_transient_set_covers_every_consumed_column() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("register all nodes");
        for m in reg.manifests() {
            let Some(k) = reg.gpu_kernel(m.id) else {
                continue;
            };
            for b in k.bindings {
                assert!(
                    !b.access.consumes() || TRANSIENT_COLUMNS.contains(&b.column),
                    "a `Consume` binding drops `{}`, which is not in TRANSIENT_COLUMNS — \
                     the diagnoser would never analyse a producer of it",
                    b.column
                );
            }
        }
        assert!(
            TRANSIENT_COLUMNS.contains(&"falloff"),
            "falloff is the modulation weight the diagnoser must reason about"
        );
    }
}
