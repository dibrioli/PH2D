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

/// A HEALTHY `grid → integrate → output` (the self-loop plumbed by `reconcile`),
/// with a `force.wind` SPLICED between integrate and output. The force is downstream
/// of the integrator, so its `accel` is never consumed — the Reorder case.
fn reorder_setup() -> (MotionState, NodeId, NodeId, NodeId, NodeId) {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let grid = add(&mut m, "motion.grid");
    let integ = add(&mut m, "motion.integrate");
    let force = add(&mut m, "force.wind");
    let out = add(&mut m, "motion.output");
    wire(&mut m, grid, 0, integ, 0); // grid -> integrate.rest
    wire(&mut m, integ, 0, force, 0); // integrate -> force (force spliced downstream)
    wire(&mut m, force, 0, out, 0); // force -> output
    let before = m.doc.graph.clone();
    reconcile(&mut m, &before); // plumb integrate.forces self-loop
    (m, grid, integ, force, out)
}

/// Fire the heal pass with a constructive gesture that does not itself complete the
/// setup (adding an unrelated node) — the trigger, not the cure.
fn nudge(m: &mut MotionState) {
    let _ = drain_intents();
    push_intent(GraphIntent::AddNode {
        type_name: "motion.tint",
        x: 0.0,
        y: 0.0,
    });
    run(m);
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
    m.pump.pump(
        &m.doc.graph,
        &m.registry,
        &[out],
        tick,
        playhead,
        [0.0; 4],
        [1.0; 2],
    );
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

    let integ = *integrate_ids(&m)
        .first()
        .expect("an integrator was inserted");
    assert!(has_edge(&m, grid, 0, integ, 0), "grid feeds integrate.rest");
    assert!(
        has_edge(&m, force, 0, integ, 1),
        "the force feeds integrate.forces"
    );
    assert!(has_edge(&m, integ, 0, out, 0), "integrate feeds the output");
    assert!(
        !has_edge(&m, force, 0, out, 0),
        "the inert direct edge is gone"
    );
    assert!(
        !has_edge(&m, grid, 0, force, 0),
        "the force is no longer fed by grid"
    );

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
    assert!(
        moved,
        "the healed graph MOVES the points (the wind now integrates)"
    );
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
    assert!(
        integrate_ids(&m).is_empty(),
        "undo removes the inserted integrator"
    );
    assert!(
        has_edge(&m, force, 0, out, 0),
        "and restores the direct force -> output"
    );
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
    push_intent(GraphIntent::DeleteSelection {
        nodes: vec![junk.0],
    });
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

    assert_eq!(
        integrate_ids(&m).len(),
        1,
        "one integrator for the whole chain"
    );
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

/// **A force SPLICED downstream of a working integrator is REORDERED onto it — one
/// integrator, not two — and the points move.** The fixture is a healthy
/// `grid → integrate → output` with a `force.wind` dropped between integrate and
/// output; its `accel` is inert (the integrator is upstream). The heal reuses THAT
/// integrator: the force feeds `integrate.forces`, integrate feeds the output, and
/// cooking across ticks moves the points. FALSIFIED by inserting a SECOND
/// integrator (two would exist) or by leaving the force inert.
#[test]
fn a_force_downstream_of_an_integrator_is_reordered_onto_it_and_the_points_move() {
    let (mut m, grid, integ, force, out) = reorder_setup();
    assert!(
        has_edge(&m, integ, 0, force, 0),
        "the force starts downstream of the integrator"
    );

    nudge(&mut m);

    assert_eq!(
        integrate_ids(&m).len(),
        1,
        "the existing integrator is REUSED, not doubled"
    );
    assert!(
        has_edge(&m, grid, 0, integ, 0),
        "grid still feeds integrate.rest"
    );
    assert!(
        has_edge(&m, force, 0, integ, 1),
        "the force now feeds integrate.forces"
    );
    assert!(
        has_edge(&m, integ, 0, out, 0),
        "integrate now feeds the output"
    );
    assert!(
        !has_edge(&m, integ, 0, force, 0),
        "the old integrate -> force edge is gone"
    );

    let p0 = positions(&mut m, out, 0, 0.0);
    assert!(!p0.is_empty(), "the grid produced points");
    let mut p = p0.clone();
    for t in 1..=12u64 {
        p = positions(&mut m, out, t, t as f64 / 60.0);
    }
    let moved = p0
        .iter()
        .zip(&p)
        .any(|(a, b)| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() > 1e-4);
    assert!(
        moved,
        "the reordered graph MOVES the points (the wind now integrates)"
    );
}

/// **The reorder is its own undo step.** One motion undo returns the pre-heal
/// document — the inert `integrate → force → output` — proving the reuse is a single,
/// separately-undoable rewire. FALSIFIED by a reorder that pushes no undo.
#[test]
fn the_reorder_is_one_undo_step() {
    let (mut m, _grid, integ, force, out) = reorder_setup();
    nudge(&mut m);
    assert!(has_edge(&m, force, 0, integ, 1), "reordered");

    let back = m.history.undo(&m.doc).expect("the reorder is an undo step");
    m.doc = back;
    assert!(
        has_edge(&m, integ, 0, force, 0),
        "undo restores integrate -> force"
    );
    assert!(
        has_edge(&m, force, 0, out, 0),
        "and the force -> output edge"
    );
    assert!(!has_edge(&m, force, 0, integ, 1), "the reorder is undone");
}

// ── W3: the ⚠ badge — the inert set the panel paints, and the click the shell honours ──

/// **The badge set is the completed-but-dead setups, and NOT the mid-builds** (ADR-0155).
/// An inert force whose chain reaches the output is in `inert_reaching_output` (it grows a
/// ⚠); the SAME kind of force wired only toward a `motion.tint` (no output yet) is NOT — it
/// is unfinished, not wrong. This is the `reaches_output` filter the badge shares with the
/// auto-heal. FALSIFIED by dropping the filter (the mid-build force would be flagged) or by
/// a detector that ignores `accel` (nothing flagged at all).
#[test]
fn the_badge_set_is_the_completed_inert_setups_not_the_mid_builds() {
    // Completed wrong setup: grid -> force -> output, no integrator.
    let (mut m, _grid, force, out) = wind_setup();
    wire(&mut m, force, 0, out, 0);
    assert!(
        inert_reaching_output(&m).contains(&force.0),
        "an inert force that reaches the output gets a badge"
    );

    // Mid-build: grid -> force -> tint (a consumer, but not the render output).
    let mut b = MotionState::new();
    b.doc = MotionDoc::new();
    let g = add(&mut b, "motion.grid");
    let f = add(&mut b, "force.wind");
    let tint = add(&mut b, "motion.tint");
    wire(&mut b, g, 0, f, 0);
    wire(&mut b, f, 0, tint, 0);
    assert!(
        inert_reaching_output(&b).is_empty(),
        "a force reaching no output is unfinished, not badged"
    );
}

/// **Clicking a FIXABLE badge heals the node — with no constructive gesture in the batch —
/// and the points then move** (ADR-0155). This is the path a destructive gesture leaves
/// open: the artist deleted the integrator, `heal_setup` (rightly) did not fight them, and
/// the badge is how they ask for it back. Here the inert setup is built directly (NO gesture
/// runs the batch heal) and `heal_one` is called: it inserts the integrator, wires the
/// canonical restructure, cooking moves the points, and it is one undo step. FALSIFIED by
/// `heal_one` skipping `Fix::Insert` (no integrator; the points stay put).
#[test]
fn clicking_a_fixable_badge_heals_the_node_and_the_points_move() {
    let (mut m, _grid, force, out) = wind_setup();
    wire(&mut m, force, 0, out, 0); // inert, and NO gesture ran the batch heal
    assert!(integrate_ids(&m).is_empty());

    heal_one(&mut m, &mut ToastQueue::default(), force);

    let integ = *integrate_ids(&m)
        .first()
        .expect("the click inserted an integrator");
    assert!(
        has_edge(&m, force, 0, integ, 1),
        "the force feeds integrate.forces"
    );
    assert!(has_edge(&m, integ, 0, out, 0), "integrate feeds the output");
    assert!(
        !has_edge(&m, force, 0, out, 0),
        "the inert direct edge is gone"
    );

    let p0 = positions(&mut m, out, 0, 0.0);
    assert!(!p0.is_empty(), "the grid produced points");
    let mut p = p0.clone();
    for t in 1..=12u64 {
        p = positions(&mut m, out, t, t as f64 / 60.0);
    }
    let moved = p0
        .iter()
        .zip(&p)
        .any(|(a, b)| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() > 1e-4);
    assert!(moved, "the click's heal MOVES the points");

    let back = m
        .history
        .undo(&m.doc)
        .expect("the click's heal is an undo step");
    m.doc = back;
    assert!(
        integrate_ids(&m).is_empty(),
        "undo removes the inserted integrator"
    );
}

/// **Clicking an ADVISORY badge changes nothing — it explains, never guesses** (ADR-0155).
/// A `motion.pin_constraint` with no solver downstream is inert (`Fix::Offer`); there is no
/// canonical fix, so `heal_one` leaves the graph byte-for-byte and pushes no undo step (the
/// ADR law: never guess a creative choice — WHICH solver a pin wants is the artist's call).
/// FALSIFIED by `heal_one` applying a fix to the Offer case (a node/edge would appear, or an
/// undo step would exist).
#[test]
fn clicking_an_advisory_badge_changes_nothing() {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let grid = add(&mut m, "motion.grid");
    let pin = add(&mut m, "motion.pin_constraint");
    let out = add(&mut m, "motion.output");
    wire(&mut m, grid, 0, pin, 0);
    wire(&mut m, pin, 0, out, 0);
    // The pin IS badged (inert, reaches output) but has no canonical fix.
    assert!(
        inert_reaching_output(&m).contains(&pin.0),
        "the pin is badged"
    );
    let before = m.doc.graph.clone();

    heal_one(&mut m, &mut ToastQueue::default(), pin);

    assert_eq!(
        m.doc.graph.nodes().len(),
        before.nodes().len(),
        "an advisory click inserts nothing"
    );
    assert_eq!(
        m.doc.graph.edges().len(),
        before.edges().len(),
        "and rewires nothing"
    );
    assert!(
        m.history.undo(&m.doc).is_none(),
        "and pushes no undo step (it only explains)"
    );
}

/// **A source-less deformer badges, and clicking it only explains** (ADR-0155, the
/// required-upstream family / 2a). `motion.bend` wired to the output with NOTHING feeding
/// it reads `P` but has no stream — it is inert and reaches the output, so it badges; and
/// a `MissingSource` is a `Fix::Offer` (WHICH source is the artist's call), so `heal_one`
/// leaves the graph byte-for-byte and pushes no undo step. FALSIFIED by the badge not
/// appearing (the source-less reader is not detected) or by `heal_one` mutating the graph
/// for a `MissingSource` (it would guess a source, which the ADR forbids).
#[test]
fn a_source_less_deformer_badges_and_only_explains() {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let bend = add(&mut m, "motion.bend");
    let out = add(&mut m, "motion.output");
    wire(&mut m, bend, 0, out, 0); // bend reaches the output, but nothing feeds it
    assert!(
        inert_reaching_output(&m).contains(&bend.0),
        "a source-less deformer that reaches the output badges"
    );
    let before = m.doc.graph.clone();

    heal_one(&mut m, &mut ToastQueue::default(), bend);

    assert_eq!(
        m.doc.graph.nodes().len(),
        before.nodes().len(),
        "an advisory click on a source-less node inserts nothing"
    );
    assert_eq!(
        m.doc.graph.edges().len(),
        before.edges().len(),
        "and rewires nothing"
    );
    assert!(
        m.history.undo(&m.doc).is_none(),
        "and pushes no undo step (it only explains — WHICH source is the artist's call)"
    );
}

/// **A duplicator missing a required input badges, and clicking it only explains** (ADR-0155,
/// the required-PORT family / 2b). `motion.duplicator` reaching the output with its `shape`
/// wired but its `points` port empty declares that input required, so it badges; and a
/// `MissingInput` is a `Fix::Offer` (WHAT to wire is the artist's call), so `heal_one` leaves
/// the graph byte-for-byte and pushes no undo. FALSIFIED by the badge not appearing (the
/// declared-required port is not detected) or by `heal_one` mutating the graph for it.
#[test]
fn a_duplicator_missing_an_input_badges_and_only_explains() {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let grid = add(&mut m, "motion.grid");
    let dup = add(&mut m, "motion.duplicator");
    let out = add(&mut m, "motion.output");
    wire(&mut m, grid, 0, dup, 0); // shape (port 0) fed
    wire(&mut m, dup, 0, out, 0); // reaches the output; points (port 1) left empty
    assert!(
        inert_reaching_output(&m).contains(&dup.0),
        "a duplicator missing a required input, reaching the output, badges"
    );
    let before = m.doc.graph.clone();

    heal_one(&mut m, &mut ToastQueue::default(), dup);

    assert_eq!(
        m.doc.graph.nodes().len(),
        before.nodes().len(),
        "an advisory click on a missing-input node inserts nothing"
    );
    assert_eq!(
        m.doc.graph.edges().len(),
        before.edges().len(),
        "and rewires nothing"
    );
    assert!(
        m.history.undo(&m.doc).is_none(),
        "and pushes no undo step (it only explains — WHAT to wire is the artist's call)"
    );
}

// The Node Help toggle gates (the artist's on/off for the whole system) live in a child
// module — shell LOC cap. A child so `use super::*` reaches these fixtures.
#[path = "motion_bridge_heal_toggle_tests.rs"]
mod toggle;

#[path = "motion_bridge_heal_advisory_tests.rs"]
mod advisory;
