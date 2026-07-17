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
/// Rotation oscillator, and the GPU suffix carries real compute (the Y wave + the
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
    // NOT fully-GPU — the first oscillator (Rotation) has no kernel.
    assert!(
        !plan.is_fully_gpu(),
        "the demo must exercise the HYBRID path, not fully-GPU"
    );
    let &(boundary, _) = plan.boundaries.first().expect("a CPU boundary");
    assert_eq!(
        doc.graph.node(boundary).unwrap().type_name,
        "motion.oscillator",
        "the boundary is the Rotation oscillator the kernel does not cover"
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

/// The Fase 3 **simulation** demo (`PH2D_GPU_COOK_DEMO=3`) must plan as a
/// fully-GPU chain that DRIVES A LOOP — the whole claim of ADR-0123. Without
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
