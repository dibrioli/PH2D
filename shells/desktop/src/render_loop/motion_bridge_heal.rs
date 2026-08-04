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
    /// The node feeding the head's `in0` (the base positions) + its output port.
    source: (NodeId, u16),
    /// The last force in the chain — feeds `integrate.forces`.
    last: NodeId,
    /// The node the last force feeds, and the input port (the sink side).
    consumer: (NodeId, u16),
    /// `motion.integrate` (or `sim.step` in a particle chain).
    integ: &'static str,
    /// Where to drop the integrator.
    pos: Pos,
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
        let Fix::Insert(integ) = d.fix else { continue };
        if !reaches_output(&motion.doc.graph, d.node) {
            continue; // a dangling mid-build force (no sink) is not a completed setup
        }
        if let Some(plan) = plan_heal(&motion.doc.graph, &motion.registry, d.node, integ)
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
        "Wired the force through Integrate so it moves the points (undo to remove)"
    } else {
        "Wired the forces through Integrate so they move the points (undo to remove)"
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
    integ: &'static str,
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
        integ,
        pos,
    })
}

/// Apply one plan to the graph, returning the inserted integrator (or `None` if a
/// connect unexpectedly fails). Frees the head from its source and the last force
/// from its consumer, then wires `source → integ.rest`, `last → integ.forces`,
/// `integ → consumer`.
fn apply_heal(g: &mut Graph, plan: &HealPlan) -> Option<NodeId> {
    g.disconnect(plan.head, 0); // free source → head (becomes the pre-loop)
    g.disconnect(plan.consumer.0, plan.consumer.1); // free last → consumer
    let integ = g.add_node(plan.integ);
    g.set_pos(integ, plan.pos);
    g.connect(Edge {
        from: plan.source,
        to: (integ, 0), // rest ← the base positions
        delayed: false,
    })
    .ok()?;
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
mod tests {
    //! The gates drive the REAL intent funnel (`apply_graph_intents`), so the heal
    //! fires exactly where it will in the app. The star gate then COOKS the healed
    //! graph and watches the points move — a heal that wires an integrator that does
    //! not actually integrate would pass a structural gate and fail this one.
    use super::super::apply_graph_intents;
    use super::*;
    use crate::motion_state::MotionState;
    use ph2d_motion_doc::MotionDoc;
    use ph2d_panel_motion_graph::{drain_intents, push_intent};

    fn run(m: &mut MotionState) {
        apply_graph_intents(
            m,
            &mut ph2d_core::Playhead::default(),
            &mut ToastQueue::default(),
            &mut ph2d_editor::screens::layout::CenterSplit::None,
        );
    }

    fn add(m: &mut MotionState, ty: &str) -> NodeId {
        m.doc.graph.add_node(ty)
    }
    fn wire(m: &mut MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) {
        m.doc
            .graph
            .connect(Edge {
                from: (from, fp),
                to: (to, tp),
                delayed: false,
            })
            .expect("wire");
    }

    /// `grid → force.wind` plus a detached `motion.output`. Fresh history.
    fn wind_setup() -> (MotionState, NodeId, NodeId, NodeId) {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let grid = add(&mut m, "motion.grid");
        let force = add(&mut m, "force.wind");
        let out = add(&mut m, "motion.output");
        wire(&mut m, grid, 0, force, 0);
        (m, grid, force, out)
    }

    /// Push a constructive `Connect` and apply the batch.
    fn connect(m: &mut MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) {
        let _ = drain_intents();
        push_intent(GraphIntent::Connect {
            from_node: from.0,
            from_port: fp,
            to_node: to.0,
            to_port: tp,
        });
        run(m);
    }

    fn integrate_ids(m: &MotionState) -> Vec<NodeId> {
        m.doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.integrate")
            .map(|n| n.id)
            .collect()
    }
    fn has_edge(m: &MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) -> bool {
        m.doc
            .graph
            .edges()
            .iter()
            .any(|e| e.from == (from, fp) && e.to == (to, tp) && !e.delayed)
    }

    /// Cook the current graph at `tick`/`playhead` and return the output positions.
    fn positions(m: &mut MotionState, out: NodeId, tick: u64, playhead: f64) -> Vec<[f32; 2]> {
        m.pump.mark_dirty();
        m.pump
            .pump(&m.doc.graph, &m.registry, &[out], tick, playhead, [0.0; 4], [1.0; 2]);
        m.pump.instances.iter().map(|i| i.world_pos).collect()
    }

    /// **The star: connecting a force to the output heals the setup AND the points
    /// then move.** The healed graph is the canonical restructure (rest ← grid,
    /// forces ← force, integrate → output; grid no longer feeds the force), and cooking
    /// it across ticks moves the points — the force now does something. FALSIFIED by
    /// the heal not firing, or by a wiring that validates but does not integrate.
    #[test]
    fn a_force_wired_to_output_is_healed_and_the_points_move() {
        let (mut m, grid, force, out) = wind_setup();
        connect(&mut m, force, 0, out, 0); // the wrong setup: force -> output, no integrator

        let integ = *integrate_ids(&m).first().expect("an integrator was inserted");
        assert!(has_edge(&m, grid, 0, integ, 0), "grid feeds integrate.rest");
        assert!(has_edge(&m, force, 0, integ, 1), "the force feeds integrate.forces");
        assert!(has_edge(&m, integ, 0, out, 0), "integrate feeds the output");
        assert!(!has_edge(&m, force, 0, out, 0), "the inert direct edge is gone");
        assert!(!has_edge(&m, grid, 0, force, 0), "the force is no longer fed by grid");

        let p0 = positions(&mut m, out, 0, 0.0);
        assert!(!p0.is_empty(), "the grid produced points");
        let mut p = p0.clone();
        for t in 1..=12u64 {
            p = positions(&mut m, out, t, t as f64 / 60.0);
        }
        assert_eq!(p.len(), p0.len(), "the count is stable");
        let moved = p0
            .iter()
            .zip(&p)
            .any(|(a, b)| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() > 1e-4);
        assert!(moved, "the healed graph MOVES the points (the wind now integrates)");
    }

    /// **The heal is its own undo step.** After the heal, one motion undo returns the
    /// pre-heal document — the inert `force → output` with no integrator — proving the
    /// fix is a single, separately-undoable step. FALSIFIED by a heal that pushes no
    /// undo (nothing to revert) or folds into the connect.
    #[test]
    fn the_heal_is_one_undo_step() {
        let (mut m, _grid, force, out) = wind_setup();
        connect(&mut m, force, 0, out, 0);
        assert_eq!(integrate_ids(&m).len(), 1);

        let back = m.history.undo(&m.doc).expect("the heal is an undo step");
        m.doc = back;
        assert!(integrate_ids(&m).is_empty(), "undo removes the inserted integrator");
        assert!(has_edge(&m, force, 0, out, 0), "and restores the direct force -> output");
    }

    /// **A constructive batch heals a pre-existing inert setup**, and its twin below
    /// proves a destructive batch does not. Same inert graph, opposite gesture — the
    /// pair is the mutation proof of the constructive/destructive gate. FALSIFIED by a
    /// heal that ignores the batch classification.
    #[test]
    fn a_constructive_batch_heals_a_preexisting_inert_setup() {
        let (mut m, _grid, force, out) = wind_setup();
        wire(&mut m, force, 0, out, 0); // inert setup built directly, no gesture yet
        assert!(integrate_ids(&m).is_empty());
        // Any constructive gesture (add an unrelated node) triggers the heal.
        let _ = drain_intents();
        push_intent(GraphIntent::AddNode {
            type_name: "motion.tint",
            x: 0.0,
            y: 0.0,
        });
        run(&mut m);
        assert_eq!(integrate_ids(&m).len(), 1, "a constructive batch heals it");
    }

    /// The twin: **a batch that also REMOVES something never heals**, even when it
    /// also completes a wrong setup. This is the only case the `!is_destructive`
    /// guard defends — a pure delete is already blocked by "no constructive intent",
    /// so the proof needs a MIXED batch (a Connect that completes the setup AND a
    /// Delete). Deleting the integrator to rewire is never fought. FALSIFIED by
    /// dropping the `!is_destructive` guard (the mixed batch would heal).
    #[test]
    fn a_batch_that_also_removes_does_not_heal() {
        let (mut m, _grid, force, out) = wind_setup();
        let junk = add(&mut m, "motion.tint");
        let _ = drain_intents();
        // One batch: complete the wrong setup AND remove a node. Mixed → no heal.
        push_intent(GraphIntent::Connect {
            from_node: force.0,
            from_port: 0,
            to_node: out.0,
            to_port: 0,
        });
        push_intent(GraphIntent::DeleteSelection { nodes: vec![junk.0] });
        run(&mut m);
        assert!(has_edge(&m, force, 0, out, 0), "the connect applied");
        assert!(
            integrate_ids(&m).is_empty(),
            "a batch that also removes does not heal, even a completed setup"
        );
    }

    /// **Two forces in one chain get ONE integrator, past both.** The dedupe by chain
    /// head + the walk to the last force keep the rule "one integrator applies".
    /// FALSIFIED by inserting an integrator per inert force (two, one mid-chain).
    #[test]
    fn two_forces_in_a_chain_get_one_integrator() {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let grid = add(&mut m, "motion.grid");
        let fa = add(&mut m, "force.wind");
        let fb = add(&mut m, "force.attractor");
        let out = add(&mut m, "motion.output");
        wire(&mut m, grid, 0, fa, 0);
        wire(&mut m, fa, 0, fb, 0);
        connect(&mut m, fb, 0, out, 0);

        assert_eq!(integrate_ids(&m).len(), 1, "one integrator for the whole chain");
        let integ = integrate_ids(&m)[0];
        assert!(has_edge(&m, fa, 0, fb, 0), "the force chain is intact");
        assert!(has_edge(&m, fb, 0, integ, 1), "the LAST force feeds forces");
        assert!(has_edge(&m, grid, 0, integ, 0), "grid feeds rest");
        assert!(has_edge(&m, integ, 0, out, 0), "integrate feeds the output");
    }

    /// **A force whose chain reaches no output is NOT healed** — it is a mid-build,
    /// not a completed wrong setup. The force here DOES feed a consumer (`motion.tint`),
    /// so `plan_heal` would happily restructure it; only the `reaches_output` guard
    /// holds it back. FALSIFIED by dropping that guard (the incomplete chain would be
    /// restructured while the artist is still wiring toward the output).
    #[test]
    fn a_force_that_reaches_no_output_is_not_healed() {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let grid = add(&mut m, "motion.grid");
        let force = add(&mut m, "force.wind");
        let tint = add(&mut m, "motion.tint"); // a consumer, but NOT the render output
        wire(&mut m, grid, 0, force, 0);
        wire(&mut m, force, 0, tint, 0);
        let _ = drain_intents();
        push_intent(GraphIntent::AddNode {
            type_name: "motion.scale",
            x: 0.0,
            y: 0.0,
        });
        run(&mut m);
        assert!(
            integrate_ids(&m).is_empty(),
            "an incomplete chain (no output) is left alone"
        );
    }
}
