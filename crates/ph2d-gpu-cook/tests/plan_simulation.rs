//! Plan-level gates for the **simulation loop** (GPU/M5 Fase 3, ADR-0127) — no
//! GPU device needed, so they run on every CI lane.
//!
//! A wrong plan here is worse than a slow one. The three it must never get
//! wrong, each gated on its own (a layered defence whose layers share one test
//! passes with a layer deleted — [[feedback_layered_defenses_need_per_layer_gates]]):
//!
//! 1. **The `pre` loop must be claimed WHOLE.** A CPU boundary inside it makes
//!    the pump cook the loop's other half with its own `prev` — two simulations
//!    of one state, and the GPU integrating the CPU's divergent `accel`.
//! 2. **An `id` column is an arithmetic gather ONLY on a dense window**
//!    (ADR-0130): a `motion.emitter`'s contiguous id range CLAIMS the
//!    integrator (`current_id − prev_first` is a row offset), but a
//!    present-but-not-dense id (a `sort`/`cull` broke the window) still recedes
//!    rather than mispair by position.
//! 3. **An unprovable shape is a refusal**, not an assumption: if a CPU
//!    boundary feeds `rest`, the plan cannot know whether `id` is there.
//!
//! (2) and (3) are separate layers of the same defence, each gated on its own
//! fixture: the dense emitter (claim) and its reorder sibling (recede) for (2);
//! a derivable non-dense id (`test.idgen`, recede) between them; and an
//! unprovable boundary for (3). Delete any one check and one test goes red.

use ph2d_gpu_cook::plan::output_dense_window;
use ph2d_gpu_cook::{GpuSource, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_integrate::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    ph2d_node_force_drag::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    reg
}

fn edge(g: &mut Graph, from: NodeId, to: NodeId, port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed,
    })
    .expect("well-formed edge");
}

/// `grid → integrate → output` with the bare self-loop the editor auto-wires
/// when no force chain is present (`out --pre--> forces`).
fn self_loop(reg: &NodeRegistry) -> (Graph, [NodeId; 3]) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, ig, 0, false);
    edge(&mut g, ig, ig, 1, true); // the self-loop
    edge(&mut g, ig, out, 0, false);
    g.validate(reg).expect("well-typed");
    (g, [grid, ig, out])
}

/// The same, with a force chain in the loop:
/// `integrate.out --pre--> wind → drag --fwd--> integrate.forces`.
fn force_loop(reg: &NodeRegistry) -> (Graph, [NodeId; 5]) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let ig = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    let drag = g.add_node("force.drag");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, ig, 0, false);
    edge(&mut g, ig, wind, 0, true); // the `pre` that closes the loop
    edge(&mut g, wind, drag, 0, false);
    edge(&mut g, drag, ig, 1, false);
    edge(&mut g, ig, out, 0, false);
    g.validate(reg).expect("well-typed");
    (g, [grid, ig, wind, drag, out])
}

#[test]
fn the_bare_self_loop_is_claimed_whole() {
    // The DEFAULT integrator shape: `out --pre--> forces` straight back into
    // itself. The `pre` source is the node being decided, which is not yet in
    // the walk's claimed set — a rule that only accepted an already-claimed
    // ancestor would refuse the most common document there is.
    let reg = registry();
    let (g, [grid, ig, out]) = self_loop(&reg);
    let p = plan(&g, &reg, &reg, out);
    assert!(
        p.is_fully_gpu(),
        "no node here lacks a kernel: {:?}",
        p.boundaries
    );
    assert_eq!(
        p.stages.iter().map(|s| s.node).collect::<Vec<_>>(),
        vec![grid, ig, out],
        "topological order, sink last"
    );
    assert!(p.drives_a_loop(), "the state lives on the GPU across ticks");
    // grid + integrate dispatch; `output` is a pass-through.
    assert_eq!(p.dispatching_stages(&reg), 2);
    // The integrator's two ports resolve to the live chain and to ITSELF.
    let st = p.stages.iter().find(|s| s.node == ig).expect("staged");
    assert_eq!(
        st.inputs,
        vec![GpuSource::Stage(grid), GpuSource::Prev(ig)],
        "rest from the chain, forces from last tick"
    );
}

#[test]
fn a_force_chain_inside_the_loop_is_claimed_and_reads_last_ticks_state() {
    let reg = registry();
    let (g, [grid, ig, wind, drag, out]) = force_loop(&reg);
    let p = plan(&g, &reg, &reg, out);
    assert!(p.is_fully_gpu(), "boundaries: {:?}", p.boundaries);
    // Topological: the forces read `prev` (last tick), so they may be encoded
    // before the integrator that produces this tick's state.
    assert_eq!(
        p.stages.iter().map(|s| s.node).collect::<Vec<_>>(),
        vec![grid, wind, drag, ig, out]
    );
    assert!(p.drives_a_loop());
    // grid + wind + drag + integrate.
    assert_eq!(p.dispatching_stages(&reg), 4);
    let head = p.stages.iter().find(|s| s.node == wind).expect("staged");
    assert_eq!(
        head.inputs,
        vec![GpuSource::Prev(ig)],
        "the loop head reads the integrator's PREVIOUS output"
    );
    let sim = p.stages.iter().find(|s| s.node == ig).expect("staged");
    assert_eq!(
        sim.inputs,
        vec![GpuSource::Stage(grid), GpuSource::Stage(drag)]
    );
}

#[test]
fn a_cpu_node_inside_the_loop_refuses_the_whole_claim() {
    // THE trap this slice had to name. `test.nokernel` sits in the force chain,
    // so the plan would leave a boundary there — and the caller would cook it on
    // the pump, which to do so re-cooks the integrator with the CPU's OWN
    // `prev`. Two simulations of one state: the GPU would integrate an `accel`
    // computed from a trajectory that is not its own, every tick, forever.
    //
    // Claiming "just the rest chain" is not a fallback either: the refusal is
    // whole-plan, because the CPU already runs this document correctly.
    let mut reg = registry();
    nokernel::register(&mut reg);
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let ig = g.add_node("motion.integrate");
    let alien = g.add_node("test.nokernel");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, ig, 0, false);
    edge(&mut g, ig, alien, 0, true);
    edge(&mut g, alien, ig, 1, false);
    edge(&mut g, ig, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(!p.is_fully_gpu());
    assert_eq!(
        p.dispatching_stages(&reg),
        0,
        "nothing is claimed → the caller's route recuses to the CPU whole"
    );
    assert!(!p.drives_a_loop(), "no state may live on the GPU here");
    assert!(
        !p.stages.iter().any(|s| s.node == ig),
        "the integrator must NOT be staged: the pump owns this sim"
    );
}

#[test]
fn a_partly_covered_sim_zone_keeps_its_render_suffix_on_the_gpu() {
    // The RETREAT (ADR-0135). A `sim.zone` whose interior has an uncovered
    // node cannot be claimed WHOLE — the loop recedes to the pump (else two
    // simulations of one state, the refusal above). But the render chain
    // DOWNSTREAM of the zone is ordinary per-element work and stays on the GPU:
    // the plan forbids the zone (making it a boundary) and re-plans, rather than
    // refusing the whole plan to `(sink, 0)`. Refusing whole would drop the
    // suffix for nothing — regressing the boot snow from hybrid to full-CPU,
    // measured in `motion_gpu_coverage`.
    let mut reg = registry();
    ph2d_node_sim_zone::register(&mut reg).unwrap();
    ph2d_node_sim_step::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    nokernel::register(&mut reg);

    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    let step = g.add_node("sim.step");
    let alien = g.add_node("test.nokernel"); // the uncoverable node inside the loop
    let mv = g.add_node("motion.move"); // the render suffix (dispatches)
    let out = g.add_node("motion.output");
    edge(&mut g, grid, zone, 0, false); // grid → zone.init
    edge(&mut g, zone, wind, 0, true); // zone.out --pre--> wind (state entry)
    edge(&mut g, wind, step, 0, false);
    edge(&mut g, step, alien, 0, false);
    edge(&mut g, alien, zone, 1, false); // interior tail (through the alien) → zone.state
    edge(&mut g, zone, mv, 0, false); // zone.out → move → output (the suffix)
    edge(&mut g, mv, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(!p.is_fully_gpu(), "the alien in the loop leaves a boundary");
    assert_eq!(
        p.dispatching_stages(&reg),
        1,
        "the render suffix (move) must still dispatch — the retreat keeps it, and \
         a refuse-to-(sink,0) would drop it to 0"
    );
    assert!(
        !p.stages.iter().any(|s| s.node == zone || s.node == step),
        "the sim recedes to the pump: neither the zone nor its integrator is staged"
    );
    assert!(
        p.boundaries.iter().any(|(n, _)| *n == zone),
        "the boundary lands on the zone the retreat forbade, not on the sink"
    );
    assert!(!p.drives_a_loop(), "no sim state lives on the GPU");
}

#[test]
fn a_fully_covered_sim_zone_loop_is_claimed_whole() {
    // The CONTROL for the retreat: with every interior node covered, the zone IS
    // claimed and the whole loop runs on the GPU. Without this, the gate above
    // would pass with the plan forbidding the zone UNCONDITIONALLY — the retreat
    // would have quietly become "never claim a sim.zone", which is the opposite
    // of the feature ([[feedback_a_negative_search_needs_a_positive_control]]).
    let mut reg = registry();
    ph2d_node_sim_zone::register(&mut reg).unwrap();
    ph2d_node_sim_step::register(&mut reg).unwrap();
    ph2d_node_sim_collide::register(&mut reg).unwrap();

    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    let step = g.add_node("sim.step");
    let ground = g.add_node("sim.collide");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, zone, 0, false);
    edge(&mut g, zone, out, 0, false);
    edge(&mut g, zone, wind, 0, true); // the state entry
    edge(&mut g, wind, step, 0, false);
    edge(&mut g, step, ground, 0, false);
    edge(&mut g, ground, zone, 1, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(p.is_fully_gpu(), "boundaries: {:?}", p.boundaries);
    assert!(p.drives_a_loop(), "the zone's state loop lives on the GPU");
    // grid + wind + sim.step + sim.collide dispatch; zone + output are passes.
    assert_eq!(p.dispatching_stages(&reg), 4);
    assert!(
        p.stages.iter().any(|s| s.node == zone),
        "the zone IS staged (as a conditional passthrough)"
    );
}

#[test]
fn an_unprovable_shape_refuses_a_kernel_that_names_a_column() {
    // Layer 3, ISOLATED. `test.refuser` names `id` on its input but has no `pre`
    // loop, so the whole-claim refusal of the previous test cannot reach it: the
    // ONLY thing that decides is "can the plan derive what crosses this input?".
    //
    // The first draft of this gate used `emitter → integrate` and stayed GREEN
    // with the layer deleted — the loop refusal was holding it up. Two layers
    // pointing the same way in the product is exactly why a shared fixture
    // proves neither ([[feedback_layered_defenses_need_per_layer_gates]]).
    let mut reg = registry();
    nokernel::register(&mut reg);
    refuser::register(&mut reg);
    let mut g = Graph::new();
    let alien = g.add_node("test.nokernel");
    let rf = g.add_node("test.refuser");
    let out = g.add_node("motion.output");
    edge(&mut g, alien, rf, 0, false);
    edge(&mut g, rf, out, 0, false);
    g.validate(&reg).expect("well-typed");
    let p = plan(&g, &reg, &reg, out);
    assert!(
        !p.stages.iter().any(|s| s.node == rf),
        "a CPU boundary feeds it: the column could be there, so recede"
    );

    // …and a DERIVABLE input without the column is claimed — otherwise this
    // would pass with the node refused for any reason at all.
    let mut reg2 = registry();
    idgen::register_without_id(&mut reg2);
    refuser::register(&mut reg2);
    let mut g2 = Graph::new();
    let src = g2.add_node("test.idgen");
    let rf2 = g2.add_node("test.refuser");
    let out2 = g2.add_node("motion.output");
    edge(&mut g2, src, rf2, 0, false);
    edge(&mut g2, rf2, out2, 0, false);
    let p2 = plan(&g2, &reg2, &reg2, out2);
    assert!(
        p2.stages.iter().any(|s| s.node == rf2),
        "a provable shape without the column runs on the GPU"
    );
}

#[test]
fn the_emitters_dense_window_claims_the_integrator_gather() {
    // ADR-0130 slice 3 — the flip. The emitter emits a DENSE id window and has a
    // kernel, so `emitter → integrate` is claimed WHOLE: the id-gather is
    // arithmetic (`current_id − prev_first`), not the hash/sort ADR-0127 D3
    // imagined, so it runs on the GPU. This is the presence half of the sibling
    // below ([[feedback_absence_gate_needs_a_presence_sibling]]).
    let reg = registry();
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, em, ig, 0, false);
    edge(&mut g, ig, ig, 1, true);
    edge(&mut g, ig, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(
        p.is_fully_gpu(),
        "the dense emitter sim runs whole on the GPU: {:?}",
        p.boundaries
    );
    assert!(
        p.stages.iter().any(|s| s.node == ig),
        "the integrator gathers the dense id window by arithmetic"
    );
    assert!(p.drives_a_loop(), "the state lives on the GPU across ticks");
    // emitter + integrate dispatch; output is a pass-through.
    assert_eq!(p.dispatching_stages(&reg), 2);
    // rest from the emitter, forces from last tick (the self-loop).
    let st = p.stages.iter().find(|s| s.node == ig).expect("staged");
    assert_eq!(st.inputs, vec![GpuSource::Stage(em), GpuSource::Prev(ig)]);
}

#[test]
fn a_reorder_between_emitter_and_integrate_recedes() {
    // The absence sibling of the claim above. `test.reorder` HAS a kernel and
    // passes `id` through, but does NOT declare `register_dense_window` — it
    // stands for a future GPU `sort`/`cull` whose reordering makes `current_id −
    // prev_first` mispair. So the emitter's dense window is CLEARED before the
    // integrator, the id is present-but-not-dense, and the gather recedes to the
    // CPU (the safe default — never a silent mispair). The mutation that drops
    // the `!dense` check in `eligible` turns this GREEN with the mispair back.
    let mut reg = registry();
    reorder::register(&mut reg);
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    let ro = g.add_node("test.reorder");
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, em, ro, 0, false);
    edge(&mut g, ro, ig, 0, false);
    edge(&mut g, ig, ig, 1, true);
    edge(&mut g, ig, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(!p.is_fully_gpu(), "a broken window must recede");
    assert!(
        !p.stages.iter().any(|s| s.node == ig),
        "the integrator must NOT gather a reordered id stream"
    );
    assert_eq!(p.dispatching_stages(&reg), 0, "only the pass-through sink");
}

#[test]
fn a_derivable_rest_shape_carrying_id_refuses_the_integrator() {
    // Layer 2 — the shape IS derivable and contains `id`. No registered
    // generator kernel emits one today, so this fixture is the future the
    // refusal exists for (a GPU emitter): without it, that day the integrator
    // would silently pair particles BY POSITION as the set churns — each
    // survivor inheriting a stranger's velocity.
    let mut reg = registry();
    idgen::register(&mut reg);
    let mut g = Graph::new();
    let src = g.add_node("test.idgen");
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, src, ig, 0, false);
    edge(&mut g, ig, ig, 1, true);
    edge(&mut g, ig, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(
        !p.stages.iter().any(|s| s.node == ig),
        "a rest stream with identity must keep the integrator on the CPU"
    );
    assert_eq!(p.dispatching_stages(&reg), 0);

    // …and the SAME generator without the `id` column is claimed — otherwise
    // this test would pass with the whole node refused for any reason at all.
    let mut reg2 = registry();
    idgen::register_without_id(&mut reg2);
    let mut g2 = Graph::new();
    let src2 = g2.add_node("test.idgen");
    let ig2 = g2.add_node("motion.integrate");
    let out2 = g2.add_node("motion.output");
    edge(&mut g2, src2, ig2, 0, false);
    edge(&mut g2, ig2, ig2, 1, true);
    edge(&mut g2, ig2, out2, 0, false);
    let p2 = plan(&g2, &reg2, &reg2, out2);
    assert!(
        p2.stages.iter().any(|s| s.node == ig2),
        "the id column is what refuses — not the fixture"
    );
}

// ── ADR-0130: the dense id window is a PROVABLE plan property ───────────────
//
// The `id`-gather is arithmetic (`current_id − prev_first`) only when the id
// column is a dense ascending window — a property of the BARE emitter, not of
// the stream (a structural node breaks it). The plan carries a `dense_window`
// bit (`output_dense_window`), SET by the emitter, PRESERVED only by a node that
// declares `register_dense_window`; anything else clears it (default-false, never
// an allowlist of the breakers). These gate the propagation and the
// safety-critical clearing on the plan alone — no device.
//
// (This slice proves the property. The refusal still recedes on any id stream;
// the conditional claim lands WITH the arithmetic gather, ADR-0130 slice 3.)

#[test]
fn the_emitter_emits_a_dense_window() {
    let reg = registry();
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    let out = g.add_node("motion.output");
    edge(&mut g, em, out, 0, false);
    g.validate(&reg).expect("well-typed");
    assert_eq!(output_dense_window(&g, &reg, &reg, em), Some(true));
}

#[test]
fn a_source_that_does_not_declare_it_is_not_dense() {
    // `test.idgen` stamps an `id` column but is NOT a declared keeper — a future
    // id-emitting generator that is not the contiguous emitter. Emitting `id` is
    // not enough; the dense window must be DECLARED.
    let mut reg = registry();
    idgen::register(&mut reg);
    let mut g = Graph::new();
    let src = g.add_node("test.idgen");
    let out = g.add_node("motion.output");
    edge(&mut g, src, out, 0, false);
    g.validate(&reg).expect("well-typed");
    assert_eq!(output_dense_window(&g, &reg, &reg, src), Some(false));
}

#[test]
fn a_per_element_keeper_preserves_the_window() {
    // `force.wind` is a per-element endomorphism that declares it keeps the
    // window, so `emitter → wind` stays dense.
    let reg = registry();
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    let wind = g.add_node("force.wind");
    let out = g.add_node("motion.output");
    edge(&mut g, em, wind, 0, false);
    edge(&mut g, wind, out, 0, false);
    g.validate(&reg).expect("well-typed");
    assert_eq!(output_dense_window(&g, &reg, &reg, wind), Some(true));
}

#[test]
fn a_non_keeper_stage_clears_the_window() {
    // THE safety-critical clearing (ADR-0130 D2). `test.refuser` HAS a kernel —
    // so the plan can walk its output shape — but does not declare
    // `register_dense_window`: it stands for the future GPU `sort`/`cull` whose
    // reordering/filtering makes `id − prev_first` mispair. The window must clear
    // even though the emitter fed a dense one, or the arithmetic gather would
    // silently read the wrong prior state. The mutation that drops the
    // `keeps_dense_window` check turns the second assertion Some(true) → red.
    let mut reg = registry();
    refuser::register(&mut reg);
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    let rf = g.add_node("test.refuser");
    let out = g.add_node("motion.output");
    edge(&mut g, em, rf, 0, false);
    edge(&mut g, rf, out, 0, false);
    g.validate(&reg).expect("well-typed");
    // Control: the base the non-keeper sits on IS dense …
    assert_eq!(output_dense_window(&g, &reg, &reg, em), Some(true));
    // … and the non-keeper clears it.
    assert_eq!(output_dense_window(&g, &reg, &reg, rf), Some(false));
}

#[test]
fn a_bare_grid_carries_no_dense_window() {
    // A source that is not the emitter: its shape is provable (Some), it simply
    // is not a dense id window (false) — the default for anything undeclared.
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, out, 0, false);
    g.validate(&reg).expect("well-typed");
    assert_eq!(output_dense_window(&g, &reg, &reg, grid), Some(false));
}

// ── fixtures ──────────────────────────────────────────────────────────────

/// A node with an op but no kernel: the CPU can cook it, the GPU cannot claim it.
/// ADR-0136 §5 — **a STATIC boundary does not evict the loop.** The retreat
/// exists to prevent two simulations of one state; a boundary whose chain is
/// all-`Pure` with no delayed edges holds no state and reads no clock (the
/// `Effect` contract), so the pump re-evaluating it is not a second simulation
/// of anything. This is the `motion.distribute_poisson` template shape — an
/// inherently sequential algorithm that will never have a kernel — feeding the
/// snow's GPU loop.
#[test]
fn a_static_boundary_template_keeps_the_loop_claimed() {
    let mut reg = registry();
    ph2d_node_sim_zone::register(&mut reg).unwrap();
    ph2d_node_sim_step::register(&mut reg).unwrap();
    ph2d_node_sim_spawn::register(&mut reg).unwrap();
    ph2d_node_motion_combine::register(&mut reg).unwrap();
    nokernel::register(&mut reg); // Pure, kernel-less: the poisson stand-in

    let (g, zone, alien) = birth_zone(&mut reg, "test.nokernel");
    let out = sink_of(&g);
    let p = plan(&g, &reg, &reg, out);
    assert!(
        p.drives_a_loop(),
        "the loop must STAY claimed over a static boundary (ADR-0136 §5)"
    );
    assert!(
        p.stages.iter().any(|s| s.node == zone),
        "the zone is staged — the sim lives on the device"
    );
    assert!(
        p.boundaries.iter().any(|(n, _)| *n == alien),
        "the static template is the boundary the pump keeps cooking"
    );
}

/// The CONTROL for the gate above: a TEMPORAL kernel-less boundary still
/// retreats — without it, the static allowance could quietly become "any
/// boundary keeps the loop", which is the two-sims divergence coming back
/// ([[feedback_a_negative_search_needs_a_positive_control]]).
#[test]
fn a_temporal_boundary_still_retreats_the_loop() {
    let mut reg = registry();
    ph2d_node_sim_zone::register(&mut reg).unwrap();
    ph2d_node_sim_step::register(&mut reg).unwrap();
    ph2d_node_sim_spawn::register(&mut reg).unwrap();
    ph2d_node_motion_combine::register(&mut reg).unwrap();
    temporal_nokernel::register(&mut reg);

    let (g, zone, _alien) = birth_zone(&mut reg, "test.temporal_nokernel");
    let out = sink_of(&g);
    let p = plan(&g, &reg, &reg, out);
    assert!(
        !p.drives_a_loop(),
        "a clock-reading boundary must still evict the loop (two sims of one state)"
    );
    assert!(
        !p.stages.iter().any(|s| s.node == zone),
        "the sim recedes to the pump"
    );
}

/// The **whole count-changing family in one chain, claimed whole** (ADR-0136):
/// template → spawn (SourceRows) → combine (Concat) → lifetime (Compact +
/// epilogue) → cull (Compact) → output. No boundary, every structural node
/// staged — the plan-level fact the snow depends on before any device runs.
#[test]
fn the_count_changing_family_is_claimed_whole() {
    let mut reg = registry();
    ph2d_node_sim_spawn::register(&mut reg).unwrap();
    ph2d_node_motion_combine::register(&mut reg).unwrap();
    ph2d_node_sim_lifetime::register(&mut reg).unwrap();
    ph2d_node_motion_cull::register(&mut reg).unwrap();

    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let spawn = g.add_node("sim.spawn");
    let comb = g.add_node("motion.combine");
    let life = g.add_node("sim.lifetime");
    let cull = g.add_node("motion.cull");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, spawn, 0, false);
    edge(&mut g, spawn, comb, 0, false);
    edge(&mut g, comb, life, 0, false);
    edge(&mut g, life, cull, 0, false);
    edge(&mut g, cull, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let p = plan(&g, &reg, &reg, out);
    assert!(p.is_fully_gpu(), "boundaries: {:?}", p.boundaries);
    for (label, n) in [
        ("spawn", spawn),
        ("combine", comb),
        ("lifetime", life),
        ("cull", cull),
    ] {
        assert!(
            p.stages.iter().any(|s| s.node == n),
            "{label} must be staged"
        );
    }
}

/// The birth-zone fixture the two boundary gates share: a fully covered zone
/// loop whose spawn TEMPLATE is fed by `template_ty` — the node under test.
/// Returns `(graph, zone, template-chain node)`; the sink is [`sink_of`].
fn birth_zone(reg: &mut NodeRegistry, template_ty: &str) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let alien = g.add_node(template_ty);
    let spawn = g.add_node("sim.spawn");
    let comb = g.add_node("motion.combine");
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    let step = g.add_node("sim.step");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, alien, 0, false); // grid → template node (the boundary)
    edge(&mut g, alien, spawn, 0, false); // template → spawn
    edge(&mut g, zone, comb, 0, true); // zone.out --pre--> combine.in0 (state)
    edge(&mut g, spawn, comb, 1, false); // newborns → combine.in1
    edge(&mut g, comb, wind, 0, false);
    edge(&mut g, wind, step, 0, false);
    edge(&mut g, step, zone, 1, false); // interior tail → zone.state
    edge(&mut g, zone, out, 0, false);
    g.validate(reg).expect("well-typed");
    (g, zone, alien)
}

/// The output node of a [`birth_zone`] graph (added last).
fn sink_of(g: &Graph) -> NodeId {
    g.nodes().last().expect("non-empty").id
}

/// `test.temporal_nokernel` — Pure's opposite for the boundary gates: reads the
/// clock (declares `Temporal`), registers no kernel.
mod temporal_nokernel {
    use ph2d_nodegraph::attr::Stream;
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.temporal_nokernel"),
        name: "test.temporal_nokernel",
        inputs: &[PortSpec { name: "in", ty: T }],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Temporal,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Op;
    impl NodeOp for Op {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(ctx.input(0).count()));
        }
    }
    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
    }
}

mod nokernel {
    use ph2d_nodegraph::attr::Stream;
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.nokernel"),
        name: "test.nokernel",
        inputs: &[PortSpec { name: "in", ty: T }],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Op;
    impl NodeOp for Op {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(ctx.input(0).count()));
        }
    }
    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
    }
}

/// A single-input node whose kernel NAMES a column it cannot answer for, with
/// no `pre` loop — the engine rule `motion.integrate` is one instance of,
/// isolated so the "unprovable ⇒ recede" layer can be gated on its own.
mod refuser {
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.refuser"),
        name: "test.refuser",
        inputs: &[PortSpec { name: "in", ty: T }],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Op;
    impl NodeOp for Op {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(ctx.input(0).clone());
        }
    }
    const KERNEL: GpuKernel = GpuKernel {
        wgsl: "write_P(i, read_P(i));\n",
        wgsl_lib: "",
        bindings: &[
            ColumnBinding {
                column: "P",
                dim: Dim::Vec2,
                access: ColumnAccess::ReadWrite,
                identity: [0.0; 4],
                port: 0,
            },
            ColumnBinding {
                column: "id",
                dim: Dim::Scalar,
                access: ColumnAccess::RefuseIfPresent,
                identity: [0.0; 4],
                port: 0,
            },
        ],
        params: &[],
        count_law: None,
        variant_by_param: None,
        applicable: None,
    };
    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
        reg.register_gpu_kernel(MAN.id, KERNEL);
    }
}

/// A generator WITH a kernel that stamps an `id` column — the GPU emitter this
/// engine does not have yet, so that the `id` refusal is reachable and can be
/// mutation-tested today rather than the day it ships.
mod idgen {
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, SourceWindow};
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    pub const N: usize = 8;
    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.idgen"),
        name: "test.idgen",
        inputs: &[],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Op;
    impl NodeOp for Op {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(N)
                    .with("P", Column::Vec2(vec![[0.0, 0.0]; N]))
                    .with("id", Column::Scalar((0..N).map(|i| i as f32).collect())),
            );
        }
    }

    const P_BINDING: ColumnBinding = ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    };
    const ID_BINDING: ColumnBinding = ColumnBinding {
        column: "id",
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    };
    const WITH_ID: GpuKernel = GpuKernel {
        wgsl: "write_P(i, vec2<f32>(0.0, 0.0));\nwrite_id(i, f32(i));\n",
        wgsl_lib: "",
        bindings: &[P_BINDING, ID_BINDING],
        params: &[],
        count_law: Some(|_| SourceWindow::of_count(N)),
        variant_by_param: None,
        applicable: None,
    };
    const WITHOUT_ID: GpuKernel = GpuKernel {
        wgsl: "write_P(i, vec2<f32>(0.0, 0.0));\n",
        wgsl_lib: "",
        bindings: &[P_BINDING],
        params: &[],
        count_law: Some(|_| SourceWindow::of_count(N)),
        variant_by_param: None,
        applicable: None,
    };

    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
        reg.register_gpu_kernel(MAN.id, WITH_ID);
    }
    pub fn register_without_id(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
        reg.register_gpu_kernel(MAN.id, WITHOUT_ID);
    }
}

/// A per-element transformer WITH a kernel that passes `id` through but does NOT
/// declare `register_dense_window` — the stand-in for a future GPU `sort`/`cull`
/// that CLEARS the dense window (ADR-0130 D2). Its output shape is provable and
/// carries `id`, so the integrator downstream sees a present-but-not-dense id
/// and recedes.
mod reorder {
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.reorder"),
        name: "test.reorder",
        inputs: &[PortSpec { name: "in", ty: T }],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Op;
    impl NodeOp for Op {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(ctx.input(0).clone());
        }
    }
    // Touches only P; `id` (and everything else) rides through the base — so the
    // output shape carries `id`. Crucially it never calls `register_dense_window`.
    const KERNEL: GpuKernel = GpuKernel {
        wgsl: "write_P(i, read_P(i));\n",
        wgsl_lib: "",
        bindings: &[ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        }],
        params: &[],
        count_law: None,
        variant_by_param: None,
        applicable: None,
    };
    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
        reg.register_gpu_kernel(MAN.id, KERNEL);
    }
}
