//! Gates for the **GPU/M5 ready-to-smoke documents** (`PH2D_GPU_COOK_DEMO`),
//! split out of `motion_state_tests.rs` at the HR-18 cap.
//!
//! These are not smoke tests — they are what makes the smoke MEAN something. The
//! GPU route falls back to the CPU silently and by design, so a demo that
//! quietly stopped being claimed would look identical on screen (the CPU has
//! cooked all of this since M2, just slower) and the reviewer would sign off on
//! a path that never ran. Each one pins the plan the document is supposed to
//! exercise: fully-GPU, hybrid, or a fully-GPU simulation LOOP.

use super::gpu_enabled_from_env;
use super::gpu_panel_demo::build_gpu_panel_demo_document;
use super::*;

/// The **F1.2 hybrid smoke document really is a hybrid** (`PH2D_GPU_COOK_DEMO=2`).
///
/// A demo meant to show the CPU-prefix / GPU-suffix seam is worthless if it
/// secretly plans as fully-GPU (the boundary node accidentally covered) or as
/// all-CPU (nothing covered) — the artist would smoke the wrong path and never
/// know. So this asserts the PLAN, headless: the boundary lands on the un-covered
/// sort, and the GPU suffix carries real compute (the Y wave + the
/// scale), so the route decision returns `Hybrid` and the smoke exercises F1.2.
#[test]
fn the_hybrid_demo_document_plans_as_a_cpu_boundary_with_a_gpu_suffix() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_hybrid_demo_document(&mut doc, &registry).expect("well-typed hybrid demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    // NOT fully-GPU — the sort is uncoverable by structure (a reorder is not a
    // per-element map), so the seam it creates cannot decay with coverage.
    assert!(
        !plan.is_fully_gpu(),
        "the demo must exercise the HYBRID path, not fully-GPU"
    );
    let &(boundary, _) = plan.boundaries.first().expect("a CPU boundary");
    assert_eq!(
        doc.graph.node(boundary).unwrap().type_name,
        "motion.sort",
        "the boundary is the sort, which no per-element kernel can express"
    );
    // The GPU suffix does real work (oscillator(Y) + scale dispatch; output is
    // pass-through), so the route is Hybrid, not a boundary-with-nothing-to-run.
    assert!(
        plan.dispatching_stages(&registry) >= 2,
        "the GPU suffix must carry compute stages, got {}",
        plan.dispatching_stages(&registry)
    );
}

/// The **full-GPU smoke document is 2.000.000 instances, claimed whole**
/// (`PH2D_GPU_COOK_DEMO=1`). Guards the count against a silent edit (a grid
/// resized without noticing the smoke stopped being "millions") and the routing
/// (any un-covered node would split it into a hybrid, changing what the smoke
/// exercises). No cook, no device — just the grid params + the plan.
#[test]
fn the_gpu_demo_document_is_two_million_instances_claimed_fully_on_the_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_demo_document(&mut doc, &registry).expect("well-typed GPU demo");
    let out = *sinks.first().expect("one sink");

    // The grid emits `rows × cols` cells (both are element counts, capped at
    // 16.7M — 2M is well under). Read them off the graph and assert the product.
    let grid = doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.grid")
        .expect("a grid roots the chain");
    let ov = doc
        .graph
        .node_param_overrides(grid.id)
        .expect("grid params");
    let (rows, cols) = (ov["rows"], ov["cols"]);
    assert_eq!(
        rows as u64 * cols as u64,
        2_000_000,
        "the GPU smoke document must be 2.000.000 instances ({rows} × {cols})"
    );

    // Every node is kernel-covered → the plan claims the WHOLE chain (no CPU
    // boundary), with the grid + oscillator + move all dispatching.
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the full-GPU smoke must be claimed whole, not split into a hybrid"
    );
    assert_eq!(
        plan.dispatching_stages(&registry),
        3,
        "grid + oscillator + move dispatch; output is pass-through"
    );
}

/// The **sea** (`PH2D_GPU_COOK_DEMO=4`) must plan as a fully-GPU loop too — and
/// it is the demo that would rot most quietly. `force.buoyancy` was the last
/// force without a kernel, and the cost of that is not one slow node: a single
/// uncovered node inside a `pre` loop leaves a boundary, and a boundary makes
/// the plan refuse the whole simulation. So a regression here does not show up
/// as "the sea got slower" — the sea looks IDENTICAL (the CPU has run buoyancy
/// since M2) and 490k particles quietly stop being a GPU sim.
#[test]
fn the_sea_demo_document_plans_as_a_fully_gpu_loop() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_sea_demo_document(&mut doc, &registry).expect("well-typed sea demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the sea must leave no boundary: {:?}",
        plan.boundaries
    );
    assert!(
        plan.drives_a_loop(),
        "the state must live on the GPU across ticks — otherwise this is not a sim"
    );
    // grid + ramp + wind + buoyancy + integrate; `output` is a pass-through.
    assert_eq!(plan.dispatching_stages(&registry), 5);
    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    let (ig, head, tail) = (
        node("motion.integrate"),
        node("force.wind"),
        node("force.buoyancy"),
    );
    let staged = |n| {
        plan.stages
            .iter()
            .find(|s| s.node == n)
            .unwrap_or_else(|| panic!("{n:?} is staged"))
    };
    assert_eq!(
        staged(head).inputs,
        vec![ph2d_gpu_cook::GpuSource::Prev(ig)],
        "gravity reads last tick's state — it is the loop's head"
    );
    assert_eq!(
        staged(ig).inputs[1],
        ph2d_gpu_cook::GpuSource::Stage(tail),
        "the `forces` port must read the sea's accumulated accel"
    );
}

/// The **emitter fountain** (`PH2D_GPU_COOK_DEMO=5`, ADR-0130) must plan as a
/// fully-GPU loop whose integrator is claimed via the **id-gather** — the whole
/// point of the slice. If a regression made the emitter stop declaring its dense
/// window (or `motion.integrate` go back to refusing an `id` stream), this demo
/// would look identical (the CPU has cooked emitter sims since M2, just slower)
/// and 3.000 particles would quietly stop being a GPU gather. So this pins the
/// PLAN, headless: emitter → integrate is claimed WHOLE, the state loops on the
/// device, and the integrator's `rest` reads the EMITTER (the dense-window
/// source), not a boundary.
#[test]
fn the_emitter_fountain_demo_plans_as_a_fully_gpu_id_gather_loop() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_emitter_demo_document(&mut doc, &registry).expect("well-typed emitter demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the id-gather must leave no boundary — else integrate receded to the CPU: {:?}",
        plan.boundaries
    );
    assert!(
        plan.drives_a_loop(),
        "the per-particle state must live on the GPU across ticks"
    );
    // emitter + wind + curl + integrate + tint dispatch; `output` is a pass-through.
    assert_eq!(plan.dispatching_stages(&registry), 5);
    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    let (em, ig, head, tail) = (
        node("motion.emitter"),
        node("motion.integrate"),
        node("force.wind"),
        node("force.curl"),
    );
    // The Gradient Tint is STAGED, not a boundary. Before it got a kernel it was
    // `applicable`-refused, so colouring the fountain by age would have split
    // this chain and pushed the whole sim back to the CPU — the assertion above
    // (`is_fully_gpu`) would still pass on an UNCOLOURED demo, so the fact has to
    // be named here or the demo could quietly lose its colour and stay green.
    assert!(
        plan.stages.iter().any(|s| s.node == node("motion.tint")),
        "the age gradient must be claimed, not pushed to a CPU boundary"
    );
    let staged = |n| {
        plan.stages
            .iter()
            .find(|s| s.node == n)
            .unwrap_or_else(|| panic!("{n:?} is staged"))
    };
    // The gather: integrate's `rest` is the EMITTER's dense window (id-gathered),
    // and its `forces` reads the loop tail's accel — the loop head reads `prev`.
    assert_eq!(
        staged(ig).inputs[0],
        ph2d_gpu_cook::GpuSource::Stage(em),
        "the integrator gathers the emitter's dense id window"
    );
    assert_eq!(
        staged(ig).inputs[1],
        ph2d_gpu_cook::GpuSource::Stage(tail),
        "the `forces` port reads the loop's accumulated accel"
    );
    assert_eq!(
        staged(head).inputs,
        vec![ph2d_gpu_cook::GpuSource::Prev(ig)],
        "the loop head reads last tick's state"
    );
}

/// The **murmuration** (`PH2D_GPU_COOK_DEMO=7`, ADR-0134) must plan
/// as a fully-GPU LOOP — the whole claim of the neighbourhood sim. A silent CPU
/// fallback (the route degrades by design) would look identical, just at seconds
/// per tick instead of milliseconds, and the reviewer would sign off on a path
/// that never ran the spatial grid at all.
#[test]
fn the_boid_demo_plans_as_a_fully_gpu_neighbour_loop() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_boids_demo_document(&mut doc, &registry).expect("well-typed boids demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the boids sweep + the grid + scale must leave no boundary: {:?}",
        plan.boundaries
    );
    assert!(
        plan.drives_a_loop(),
        "the flock's per-agent state must live on the GPU across ticks"
    );
    // boids + scale dispatch; `output` is a pass-through.
    assert_eq!(plan.dispatching_stages(&registry), 2);

    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    let boids = node("motion.boids");
    // `scale` is STAGED, not a boundary — the grain-shrink runs on the device.
    // Without this the loop assertion above stays green on a demo that lost its
    // scale and pushed the shrink (and, with it, the flock) to the CPU.
    assert!(
        plan.stages.iter().any(|s| s.node == node("motion.scale")),
        "the grain scale must be claimed, not pushed to a CPU boundary"
    );
    // The loop head: the boids stage reads its OWN previous-tick output (the
    // `pre` self-loop), which is what makes the flock state GPU-resident.
    let boids_stage = plan
        .stages
        .iter()
        .find(|s| s.node == boids)
        .expect("boids is staged");
    assert!(
        boids_stage
            .inputs
            .contains(&ph2d_gpu_cook::GpuSource::Prev(boids)),
        "the flock steps from last tick's own state, not a fresh seed each frame"
    );
}

/// The **breathing packing** (`PH2D_GPU_COOK_DEMO=8`, ADR-0134 Fase 5) must plan
/// as a fully-GPU chain — and, unlike every scene before it, one whose kernel is
/// ITERATED. A silent CPU fallback would look identical (the CPU has packed discs
/// since M3, just at `O(N²·iterations)`), so the plan is what has to be pinned.
#[test]
fn the_breathing_packing_demo_plans_as_a_fully_gpu_chain() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_collide_demo_document(&mut doc, &registry).expect("well-typed demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the packing + its LFO must leave no boundary: {:?}",
        plan.boundaries
    );
    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    // The LFO must be STAGED, not a boundary: it is what makes the packing
    // breathe, and without it a `Effect::Pure` relaxation over a static lattice
    // re-cooks the identical picture forever — a scene that looks like a bug.
    assert!(
        plan.stages.iter().any(|s| s.node == node("value.lfo")),
        "the breath must be claimed, not pushed to a CPU boundary"
    );
    // grid + lfo + collide dispatch; `output` is pass-through.
    assert_eq!(plan.dispatching_stages(&registry), 3);
}

/// The packing demo really SWEEPS — `iterations` > 1 is the whole of Fase 5 (the
/// grid is rebuilt between sweeps). Read from the params rather than cooked: the
/// CPU reference at 262 144 discs is the very `O(N²·iterations)` this escapes.
#[test]
fn the_breathing_packing_demo_actually_iterates() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    build_gpu_collide_demo_document(&mut doc, &registry).expect("well-typed demo");
    let collide = doc
        .graph
        .nodes()
        .iter()
        .position(|n| n.type_name == "motion.collide")
        .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
        .expect("the demo has a collider");
    let params = doc
        .graph
        .node_param_overrides(collide)
        .expect("the demo sets the collider's params");
    let iters = params.get("iterations").copied().unwrap_or(0.0);
    assert!(
        iters > 1.0,
        "a single sweep would leave the iterated path (the wave's point) untested: {iters}"
    );
}

/// The SWEEP demo (`=9`) plans on the device end to end — the same claim as `=8`,
/// because it is the same chain with a different LFO shape.
#[test]
fn the_spread_sweep_demo_plans_as_a_fully_gpu_chain() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_sweep_demo_document(&mut doc, &registry).expect("well-typed demo");
    let out = *sinks.first().expect("one sink");
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the sweep + its LFO must leave no boundary: {:?}",
        plan.boundaries
    );
}

/// The SWEEP is what makes `=9` a diagnostic instead of a second breathing blob:
/// a slow LINEAR (triangle) ramp WIDE enough to cross several reach boundaries. If
/// the LFO were the default sine, or too narrow to cross a boundary, the meter
/// could not show a step's presence or absence — the whole point of the scene.
///
/// The reach boundaries are at `spread` = 0.5, 1.0, 1.5, 2.0, …; the range must
/// span at least two of them, and the wave must be Triangle (linear, no jump).
#[test]
fn the_spread_sweep_is_linear_and_crosses_reach_boundaries() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    build_gpu_sweep_demo_document(&mut doc, &registry).expect("well-typed demo");
    let params = |ty: &str| {
        let id = doc
            .graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"));
        doc.graph
            .node_param_overrides(id)
            .unwrap_or_else(|| panic!("{ty} has params"))
            .clone()
    };
    let lfo = params("value.lfo");
    // Triangle = wave 1 (linear ramp, no jump) — a sine's changing speed would
    // smear a step, a saw's reset would pop.
    assert_eq!(
        lfo.get("wave").copied().unwrap_or(0.0),
        1.0,
        "the sweep must be a linear triangle so a cost step reads as a hitch"
    );
    let offset = lfo.get("offset").copied().unwrap_or(0.0);
    let amplitude = lfo.get("amplitude").copied().unwrap_or(0.0);
    let (lo, hi) = (offset - amplitude, offset + amplitude);
    // Count the reach boundaries strictly inside the swept range.
    let crossings = (1..=8).map(|k| 0.5 * k as f32).filter(|b| *b > lo && *b < hi).count();
    assert!(
        crossings >= 2,
        "the sweep {lo:.2}..{hi:.2} must cross ≥2 reach boundaries so the meter can \
         show a staircase's presence or absence; it crosses {crossings}"
    );
}

/// The demo really is a HUGE flock with `spread` on — the two facts that make it
/// the neighbourhood-sim breakthrough rather than a pretty toy. Read from the
/// params (not cooked): a 500 k CPU cook is the very `O(N²)` this demo exists to
/// escape (seconds/tick), so the gate would time out proving the point. `spread`
/// off would silently repack the flock into a fixed box → `O(N²)` on the GPU too,
/// and the loop gate above would stay green on a demo that no longer scales.
///
/// ⚠️ The count is 524 288, NOT a million, and that is the fix — a flock's headline
/// act is to GATHER, which packs cells and raises the cost, and the million spent
/// 124 % of a 60 fps frame once clustered (Enio's stutter report). The floor here
/// keeps it a genuinely large swarm — anything this size is far past the
/// few-hundred-agent toy the grid replaces, and the ceiling stays millions (the
/// `count` is the only thing between this and them). See the measured table in
/// `build_gpu_boids_demo_document`.
#[test]
fn the_boid_demo_is_a_large_spread_flock_sized_for_headroom() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    build_gpu_boids_demo_document(&mut doc, &registry).expect("well-typed boids demo");
    let boids = doc
        .graph
        .nodes()
        .iter()
        .position(|n| n.type_name == "motion.boids")
        .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
        .expect("the demo has a boids node");
    let params = doc
        .graph
        .node_param_overrides(boids)
        .expect("the demo sets the flock's params");
    let count = params.get("count").copied().unwrap_or(0.0);
    // A genuinely large swarm (≥256 k = ~1000× the classic toy), but sized so the
    // SETTLED flock fits a 60 fps frame (measured to equilibrium in
    // `gpu_boids_scale.rs::where_does_the_flock_settle` — the demo's comment
    // carries the table). The first resize (1 M → 524 k) was read off a curve
    // still climbing; the equilibrium is the honest number.
    assert!(
        (262_144.0..=524_288.0).contains(&count),
        "the flock must be large but its EQUILIBRIUM must fit a 60 fps frame; \
         count = {count}"
    );
    // The equilibrium is set by seek-vs-separation, not by the count: seek 0.35
    // collapsed into a 28,5 ms ball at 262 k (and an ORBITING target was measured
    // and refuted — the flock rides it as a dense comet). Pin the attractor weak
    // and the spacing open so nobody cranks the collapse back in.
    let seek = params.get("seek").copied().unwrap_or(0.0);
    let separation = params.get("separation").copied().unwrap_or(0.0);
    assert!(
        seek <= 0.05 && separation >= 2.4,
        "the settled density is seek-vs-separation: seek {seek} / separation \
         {separation} would collapse the flock into a dense ball at ANY count \
         (measured: seek 0.35 plateaus at 28,5 ms for 262 k agents)"
    );
    assert!(
        params.get("spread").copied().unwrap_or(0.0) > 0.5,
        "spread MUST be on — without it the flock packs into a box and the grid \
         cannot help (O(N²)), which is the exact thing this demo disproves"
    );
}

/// The Fase 3 **simulation** demo (`PH2D_GPU_COOK_DEMO=3`) must plan as a
/// fully-GPU chain that DRIVES A LOOP — the whole claim of ADR-0127. Without
/// this, the smoke could be quietly cooking on the CPU (the route falls back
/// silently, by design) and it would look exactly the same, just slower: the
/// forces would still swirl, because the CPU has run them since M2.
#[test]
fn the_simulation_demo_document_plans_as_a_fully_gpu_loop() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_sim_demo_document(&mut doc, &registry).expect("well-typed sim demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "every node in the loop is kernel-covered: {:?}",
        plan.boundaries
    );
    assert!(
        plan.drives_a_loop(),
        "the state must live on the GPU across ticks — otherwise this is not a sim"
    );
    // grid + the ramp + the four forces + integrate dispatch; `output` is a
    // pass-through.
    assert_eq!(plan.dispatching_stages(&registry), 7);
    // The loop closes GPU-side: the chain HEAD reads the integrator's PREVIOUS
    // output (that is where the `pre` edge is), and the integrator's `forces`
    // port reads the chain's tail — this tick's accumulated `accel`.
    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    let (ig, head, tail) = (
        node("motion.integrate"),
        node("force.vortex"),
        node("force.curl"),
    );
    let staged = |n| {
        plan.stages
            .iter()
            .find(|s| s.node == n)
            .unwrap_or_else(|| panic!("{n:?} is staged"))
    };
    assert_eq!(
        staged(head).inputs,
        vec![ph2d_gpu_cook::GpuSource::Prev(ig)],
        "the force chain's head must read last tick's state"
    );
    assert_eq!(
        staged(ig).inputs[1],
        ph2d_gpu_cook::GpuSource::Stage(tail),
        "the `forces` port must read the chain's accumulated accel"
    );
}

/// The fountain's ALIVE COUNT is a product fact, and it was wrong twice without
/// anything going red: the demo asked for 3.000 while `rate × life` really
/// bounded it at 4.200, and the emitter's `max` UI hint still advertised the old
/// 4096 ceiling. A count is the one thing an artist reads off this demo at a
/// glance, so it gets stated here rather than inferred from three params.
#[test]
fn the_emitter_fountain_demo_is_actually_dense() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_emitter_demo_document(&mut doc, &registry).expect("well-typed emitter demo");
    let out = *sinks.first().expect("one sink");

    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let mut lowered = Vec::new();
    // Well past `life`, so the window is full rather than still filling.
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        &doc.graph,
        &registry,
        out,
        10.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &mut lowered,
    )
    .expect("cpu cook");
    assert_eq!(
        lowered.len(),
        1_200_000,
        "the fountain must run the window it advertises"
    );
}

/// **The panel demo is fully GPU** — the scene exists to prove the panel stays
/// readable on the device, so a scene that quietly fell back to the CPU pump
/// would smoke-test nothing at all while looking perfect.
///
/// It also pins the counts the artist is asked to read on screen: `262144` on the
/// instance nodes and on `value.math`'s output. That last one is the count law —
/// a length-262144 field times a length-1 one — and under the engine's default
/// law (*"as wide as port 0"*) it would be `1`.
#[test]
fn the_panel_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_panel_demo_document(&mut doc, &registry).expect("well-typed panel demo");
    let out = *sinks.first().expect("one sink");
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the panel scene must be claimed whole — a CPU boundary here and the \
         smoke would be reading the pump's memo, not the tap: {:?}",
        plan.boundaries
    );

    // The numbers the smoke asks the artist to read.
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let by_type = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of(ty))
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("{ty} is in the scene"))
    };
    for ty in ["motion.grid", "motion.oscillator", "value.math"] {
        let n = by_type(ty);
        let outs = cook
            .cook(&doc.graph, &registry, n, 0.25)
            .unwrap_or_else(|e| panic!("{ty} cooks: {e:?}"));
        assert_eq!(
            outs[0].as_stream().count(),
            262_144,
            "{ty} must carry 262144 — it is the number the smoke reads off the \
             card, and `value.math` carrying 1 would be the count law reading \
             port 0 instead of the widest input"
        );
    }
}

/// **The GPU cook path is ON by default** (GPU/M5, 2026-07-18).
///
/// It was opt-in because a GPU-resident cook left the graph panel blank — no
/// readout, no stamp, no wire march, no probe. `GpuCook::tap` answers all four
/// for a measured +0,075 ms, so the reason expired and the default flipped.
///
/// The gate is on the POLICY rather than on `MotionState::new()` because a test
/// that set `PH2D_GPU_COOK` would mutate the process environment that every other
/// test in the binary shares.
///
/// ⚠️ The OFF escape is pinned too, and it is not decoration: the CPU stays the
/// canonical path (ADR-0126 — the replay-hash never runs on a GPU), so bisecting
/// a suspected device bug against it has to remain one env var away.
#[test]
fn the_gpu_path_is_on_unless_explicitly_switched_off() {
    assert!(
        gpu_enabled_from_env(None),
        "absent means ON — this is the default flip"
    );
    assert!(!gpu_enabled_from_env(Some("0")), "`0` forces the CPU pump");
    // Anything else is ON, including the `1` every existing handoff and smoke
    // command in this repo already passes — those must keep working verbatim.
    for on in ["1", "true", "yes", ""] {
        assert!(gpu_enabled_from_env(Some(on)), "`{on}` must not disable it");
    }
}
