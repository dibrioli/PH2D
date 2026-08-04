//! **Setup auto-heal** (ADR-0155 W2) — sibling of `motion_bridge_adapt` (shell LOC
//! cap). `super` is `render_loop::motion_bridge`.
//!
//! The Adapter heals a *refused* wire; this heals a *successful-but-inert* graph.
//! A `force.*` accumulates `accel`; only an integrator consumes it. A force wired
//! toward the sink with no integrator writes `accel`, nothing reads it, and the
//! scene stays static — with no error (`ph2d_motion_diagnose` is the analysis that
//! finds it). After a CONSTRUCTIVE gesture, [`heal_setup`] wires every inert force
//! chain that reaches a sink through the integrator it forgot, as its own undo step
//! (the artist can undo just the fix), with a toast + selection so it is never
//! silent.
//!
//! ⚠️ **The cure is a RESTRUCTURE, not a splice** (doc 87 §3.4). `motion.integrate`
//! reads the base positions on `rest` (port 0) and the force-chain output on
//! `forces` (port 1, the feedback port `reconcile` owns). A force cannot live in the
//! horizontal `grid → force → output` chain: the healed graph is
//! `source → integrate.rest`, `last_force → integrate.forces`, `integrate → consumer`,
//! and `reconcile` plumbs `integrate.out ⟿pre⟿ chain_head.in0`. Healing therefore
//! REROUTES the artist's `source → force` edge onto the integrator — the only wiring
//! that actually moves the points.

use super::{MotionState, reconcile};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_motion_diagnose::{diagnose, Fix};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_panel_motion_graph::GraphIntent;
use std::collections::BTreeSet;

/// A gesture that can COMPLETE a wrong setup (add a node / a wire). The heal fires
/// only after a batch that has one of these and no [`is_destructive`] intent — so
/// deleting the integrator to rewire never triggers a re-insert.
pub(super) fn is_constructive(i: &GraphIntent) -> bool {
    matches!(
        i,
        GraphIntent::Connect { .. }
            | GraphIntent::AddNode { .. }
            | GraphIntent::SpliceNode { .. }
            | GraphIntent::SpliceReroute { .. }
            | GraphIntent::SmartConnect { .. }
            | GraphIntent::Paste
            | GraphIntent::DuplicateSelection { .. }
    )
}

/// A gesture that REMOVES nodes/edges — never a reason to heal (the artist is taking
/// the graph apart, possibly to rewire it by hand).
pub(super) fn is_destructive(i: &GraphIntent) -> bool {
    matches!(
        i,
        GraphIntent::Disconnect { .. }
            | GraphIntent::DeleteSelection { .. }
            | GraphIntent::CutWires { .. }
            | GraphIntent::MoveWireEnd { .. }
    )
}

/// The whole heal for one force chain: the ids to disconnect + the integrator to
/// splice, all resolved on the ORIGINAL graph so applying one chain's plan never
/// breaks the planning of another.
struct HealPlan {
    /// The chain's head force — its `in0` (freed from `source`) becomes the pre-loop.
    head: NodeId,
    /// The node feeding the head's `in0` + its output port. For [`HealKind::Insert`]
    /// it is the base positions (`grid`), fed into the NEW `integrate.rest`; for
    /// [`HealKind::Reuse`] it IS the existing integrator (its output fed the head).
    source: (NodeId, u16),
    /// The last force in the chain — feeds `integrate.forces`.
    last: NodeId,
    /// The node the last force feeds, and the input port (the sink side).
    consumer: (NodeId, u16),
    /// Create the integrator, or reuse the one already feeding the chain head.
    kind: HealKind,
    /// Where to drop a NEWLY-inserted integrator (ignored when reusing one).
    pos: Pos,
}

/// Whether the inert chain needs a NEW integrator or already has one feeding its head.
enum HealKind {
    /// No integrator on the chain: create one of this type (`motion.integrate`, or
    /// `sim.step` in a particle chain), and `source` (the base positions) feeds its
    /// `rest`. The Enio's canonical case — a force in the horizontal chain.
    Insert(&'static str),
    /// An integrator already feeds the chain head (`grid → integrate → force → …` — a
    /// force spliced DOWNSTREAM of a working integrator): reuse it (`source.0`), whose
    /// `rest` is already wired, and only the force-chain and sink sides move.
    Reuse,
}

/// **Heal the setup after a constructive gesture.** For every force chain that is
/// inert (ADR-0155), reaches a sink, and has a clean linear shape, wire it through
/// the integrator it forgot — one undo step for the whole heal. Returns how many
/// integrators were inserted (0 = nothing to do). The caller gates this on a
/// constructive-only batch.
pub(super) fn heal_setup(motion: &mut MotionState, toasts: &mut ToastQueue) -> usize {
    // Plan every inert insert-fixable producer that reaches a sink, on the original
    // graph, deduped by chain head (many inert forces in one chain heal once).
    let mut plans: Vec<HealPlan> = Vec::new();
    let mut heads: BTreeSet<u32> = BTreeSet::new();
    for d in diagnose(&motion.doc.graph, &motion.registry) {
        // Insert (no integrator anywhere) and Reorder (a force spliced downstream of
        // one) are the two force-in-the-wrong-place cases with a canonical cure. Offer
        // (a pin needing SOME solver) is a W3 badge, never auto-applied.
        if !matches!(d.fix, Fix::Insert(_) | Fix::Reorder) {
            continue;
        }
        if !reaches_output(&motion.doc.graph, d.node) {
            continue; // a dangling mid-build force (no sink) is not a completed setup
        }
        if let Some(plan) = plan_heal(&motion.doc.graph, &motion.registry, d.node, d.fix)
            && heads.insert(plan.head.0)
        {
            plans.push(plan);
        }
    }
    if plans.is_empty() {
        return 0;
    }

    // Apply every plan to one trial clone; commit only if it validates.
    let mut trial = motion.doc.graph.clone();
    let mut inserted: Vec<NodeId> = Vec::new();
    for plan in &plans {
        if let Some(node) = apply_heal(&mut trial, plan) {
            inserted.push(node);
        }
    }
    if inserted.is_empty() || trial.validate(&motion.registry).is_err() {
        return 0; // never ship a heal the cook would then reject
    }

    let pre = motion.doc.clone();
    motion.doc.graph = trial;
    // `reconcile` plumbs `integrate.out ⟿pre⟿ chain_head.in0` (the head's in0 is now
    // free), so the force reads last tick's integrated state — the loop the user
    // never draws.
    reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
    ph2d_panel_motion_graph::request_graph_selection(inserted.iter().map(|n| n.0).collect());
    toasts.push(Toast::info(if inserted.len() == 1 {
        "Wired the force through Integrate so it moves the points (undo to revert)"
    } else {
        "Wired the forces through Integrate so they move the points (undo to revert)"
    }));
    inserted.len()
}

/// Plan the restructure for the chain containing `force`. `None` when the chain is
/// dangling (head has no source, or the last force has no single consumer) — those
/// are other diagnostics, not an insert.
fn plan_heal(
    g: &Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    force: NodeId,
    fix: Fix,
) -> Option<HealPlan> {
    // Walk back through accel-producers to the head; its in0 feeder is the source.
    let mut head = force;
    let source = loop {
        let feed = feeder(g, head, 0)?; // no feeder → dangling head → cannot heal
        if produces_accel(g, reg, feed.0) {
            head = feed.0;
        } else {
            break feed;
        }
    };
    // Insert makes a NEW integrator (`source` is the base positions); Reorder reuses
    // the one that must feed the head DIRECTLY. If a non-integrator (a transform) sits
    // between an existing integrator and the force, reusing would double-integrate (its
    // `rest` would be a moving, transformed base) — leave it for a W3 badge.
    let kind = match fix {
        Fix::Insert(integ) => HealKind::Insert(integ),
        Fix::Reorder if consumes_accel(g, reg, source.0) => HealKind::Reuse,
        _ => return None,
    };
    // Walk forward through accel-producers to the last force; its single forward edge
    // is the consumer (the sink side).
    let mut last = force;
    let consumer = loop {
        let outs = forward_edges(g, last);
        let [only] = outs.as_slice() else {
            // Zero (dangling) or a branch we do not auto-restructure in W1/W2.
            return None;
        };
        if produces_accel(g, reg, only.to.0) {
            last = only.to.0;
        } else {
            break (only.to.0, only.to.1);
        }
    };

    let pos = match (g.pos(last), g.pos(consumer.0)) {
        (Some(a), Some(b)) => Pos {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        },
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => Pos { x: 0.0, y: 0.0 },
    };
    Some(HealPlan {
        head,
        source,
        last,
        consumer,
        kind,
        pos,
    })
}

/// Apply one plan to the graph, returning the integrator now driving the chain (or
/// `None` if a connect unexpectedly fails). Frees the head from its source and the last
/// force from its consumer, then wires `last → integ.forces`, `integ → consumer`. The
/// integrator is either NEW (fed the base positions on `rest`) or the one that already
/// fed the head (its `rest` is already wired — just its self-loop is cleared so the
/// force chain can take `forces`).
fn apply_heal(g: &mut Graph, plan: &HealPlan) -> Option<NodeId> {
    g.disconnect(plan.head, 0); // free source → head (becomes the pre-loop)
    g.disconnect(plan.consumer.0, plan.consumer.1); // free last → consumer
    let integ = match plan.kind {
        HealKind::Insert(ty) => {
            let n = g.add_node(ty);
            g.set_pos(n, plan.pos);
            g.connect(Edge {
                from: plan.source,
                to: (n, 0), // rest ← the base positions (grid)
                delayed: false,
            })
            .ok()?;
            n
        }
        HealKind::Reuse => {
            // The integrator already feeds the head (`source.0`); its `rest` is wired.
            // Clear the engine's self-loop on `forces` so the force chain can take it —
            // `reconcile` re-plumbs the pre-loop to the freed head afterward.
            let n = plan.source.0;
            g.disconnect(n, 1);
            n
        }
    };
    g.connect(Edge {
        from: (plan.last, 0),
        to: (integ, 1), // forces ← the force-chain output
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (integ, 0),
        to: plan.consumer, // integrate → the sink side
        delayed: false,
    })
    .ok()?;
    Some(integ)
}

/// Does `node`'s output reach a `motion.output` (the render sink) via forward
/// (non-`delayed`) edges? A completed wrong setup reaches the output; a dangling
/// mid-build force does not — so healing waits for the artist to finish wiring.
fn reaches_output(g: &Graph, node: NodeId) -> bool {
    let mut seen = BTreeSet::new();
    seen.insert(node);
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if g.node(n).is_some_and(|i| i.type_name == "motion.output") {
            return true;
        }
        for e in g.edges() {
            if e.from.0 == n && !e.delayed && seen.insert(e.to.0) {
                stack.push(e.to.0);
            }
        }
    }
    false
}

/// Does the node `n` produce `accel` (is it a force)?
fn produces_accel(g: &Graph, reg: &ph2d_node_registry::NodeRegistry, n: NodeId) -> bool {
    g.node(n)
        .and_then(|inst| reg.couplings(NodeTypeId::of(&inst.type_name)))
        .is_some_and(|cs| {
            cs.iter()
                .any(|c| matches!(c, ph2d_node_registry::Coupling::Produces(x) if *x == "accel"))
        })
}

/// Does the node `n` consume `accel` (is it an integrator)? Distinguishes a chain head
/// fed DIRECTLY by an integrator (the Reorder/reuse case) from one fed by a plain
/// source (the Insert case).
fn consumes_accel(g: &Graph, reg: &ph2d_node_registry::NodeRegistry, n: NodeId) -> bool {
    g.node(n)
        .and_then(|inst| reg.couplings(NodeTypeId::of(&inst.type_name)))
        .is_some_and(|cs| {
            cs.iter()
                .any(|c| matches!(c, ph2d_node_registry::Coupling::Consumes(x) if *x == "accel"))
        })
}

/// The node feeding `(node, port)` through a forward (non-`pre`) edge.
fn feeder(g: &Graph, node: NodeId, port: u16) -> Option<(NodeId, u16)> {
    g.edges()
        .iter()
        .find(|e| !e.delayed && e.to == (node, port))
        .map(|e| e.from)
}

/// The forward (non-`delayed`) edges leaving `node`.
fn forward_edges(g: &Graph, node: NodeId) -> Vec<Edge> {
    g.edges()
        .iter()
        .filter(|e| !e.delayed && e.from.0 == node)
        .copied()
        .collect()
}

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_heal_tests.rs"]
mod tests;
