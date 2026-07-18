//! Gates for the **GPU/M5 ready-to-smoke documents** (`PH2D_GPU_COOK_DEMO`),
//! split out of `motion_state_tests.rs` at the HR-18 cap.
//!
//! These are not smoke tests — they are what makes the smoke MEAN something. The
//! GPU route falls back to the CPU silently and by design, so a demo that
//! quietly stopped being claimed would look identical on screen (the CPU has
//! cooked all of this since M2, just slower) and the reviewer would sign off on
//! a path that never ran. Each one pins the plan the document is supposed to
//! exercise: fully-GPU, hybrid, or a fully-GPU simulation LOOP.

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
