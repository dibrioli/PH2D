//! Gates for the **Lloyd/JFA voronoi** scene (`PH2D_GPU_COOK_DEMO=11`,
//! ADR-0139) — its own file for the same reason the scene is: it answers a
//! question none of the others do (a multi-pass ALGORITHM on the device).
//!
//! The plan gate is the one that matters. The route degrades by design, so a
//! silent CPU fallback renders the SAME picture — just at minutes per cook
//! instead of milliseconds, and at this count the app would simply appear
//! hung. Pinning the plan headless is what makes "it looked right" mean
//! "it ran on the device".

use super::*;

/// The honeycomb must plan **fully GPU** — the algorithm, the falloff, the
/// projection, the ramp and the scale, with no boundary. It must NOT drive a
/// loop: the node is `Effect::Pure` (its animation arrives through `relax`),
/// so a `pre` edge appearing here would mean someone gave a stateless
/// algorithm state.
#[test]
fn the_voronoi_demo_plans_as_a_fully_gpu_algorithm() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_voronoi_demo_document(&mut doc, &registry).expect("well-typed voronoi demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the algorithm + falloff + projection + ramp + scale must leave no boundary: {:?}",
        plan.boundaries
    );
    assert!(
        !plan.drives_a_loop(),
        "the voronoi is Pure — a `pre` edge here means someone gave it state"
    );

    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    // Each of these is a distinct claim, and each has its own way of silently
    // becoming a CPU boundary — the voronoi through its algorithm channel, the
    // attribute through its text param, the ramp through the projected `t`.
    for ty in [
        "motion.voronoi",
        "motion.falloff",
        "value.attribute",
        "motion.color_ramp",
        "motion.scale",
    ] {
        assert!(
            plan.stages.iter().any(|s| s.node == node(ty)),
            "{ty} must be claimed, not pushed to a CPU boundary"
        );
    }
    // The LFO drives `relax` through the node's port 0 — a STREAM edge the plan
    // stages, not a driven param (which would refuse the node outright).
    let voronoi_stage = plan
        .stages
        .iter()
        .find(|s| s.node == node("motion.voronoi"))
        .expect("the voronoi is staged");
    assert!(
        matches!(
            voronoi_stage.inputs.first(),
            Some(ph2d_gpu_cook::GpuSource::Stage(n)) if *n == node("value.lfo")
        ),
        "relax must be fed by the staged LFO: {:?}",
        voronoi_stage.inputs
    );
}

/// The scene emits the count it claims to — the demonstration IS the number,
/// and a param the node silently clamped would make the whole scene a lie.
#[test]
fn the_voronoi_demo_emits_its_full_count() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_voronoi_demo_document(&mut doc, &registry).expect("well-typed voronoi demo");
    let out = *sinks.first().expect("one sink");

    let mut cook = ph2d_nodegraph::cook::Cook::new();
    // The CANONICAL path, at `relax = 0` (iterations still run; this is the
    // cheap end of the CPU's cost, and the count is what is being asserted).
    doc.graph.set_param(
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == "motion.voronoi")
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .expect("the demo has a voronoi"),
        "iterations",
        0.0,
    );
    let outs = cook
        .cook(&doc.graph, &registry, out, 0.0)
        .expect("the demo cooks on the canonical path");
    assert_eq!(
        outs[0].as_stream().count(),
        super::gpu_voronoi_demo::DEMO_POINTS as usize,
        "the scene must emit every point it asks for"
    );
}
