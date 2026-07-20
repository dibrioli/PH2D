//! Gates for the **neighbourhood** GPU/M5 demos (`PH2D_GPU_COOK_DEMO=7/8/9`,
//! ADR-0134), split out of `motion_state_gpu_tests.rs` at the HR-18 cap — the
//! same seam their scene builders take in `motion_state_gpu_neighbour_demos.rs`.
//! Each pins the PLAN (fully-GPU, drives a loop, dispatch count), headless, so a
//! silent CPU fallback cannot pass for the real spatial-grid path.

use super::*;

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
    let crossings = (1..=8)
        .map(|k| 0.5 * k as f32)
        .filter(|b| *b > lo && *b < hi)
        .count();
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
    // A genuinely large swarm, sized so the SETTLED flock fits a 60 fps frame
    // (measured to equilibrium in `gpu_boids_scale.rs::where_does_the_flock_settle`
    // — the demo's comment carries the three-round table).
    assert!(
        (262_144.0..=1_048_576.0).contains(&count),
        "the flock must be large but its EQUILIBRIUM must fit a 60 fps frame; \
         count = {count}"
    );
    // The settled density is set by the ATTRACTOR, and the law is superlinear in
    // the count: at 262 k a seek of 0.02 settles at 5–6 ms, but at a million the
    // same pull compresses the core with the whole swarm's weight — seek 0.02
    // measured 74–80 ms settled, and even 0.005 measured 26–30. Above the
    // half-million the only measured tuning that holds a frame is NO attractor
    // (density can then only fall: 9,9 → 5,4 ms over 160 s).
    let seek = params.get("seek").copied().unwrap_or(0.0);
    let separation = params.get("separation").copied().unwrap_or(0.0);
    if count > 524_288.0 {
        assert!(
            seek == 0.0,
            "no attractor fits a million: seek {seek} was measured to settle \
             over a frame (0.005 -> 26-30 ms; 0.02 -> 74-80 ms)"
        );
    } else {
        assert!(
            seek <= 0.05,
            "seek {seek} collapses the flock into a dense ball at any count \
             (measured: 0.35 plateaus at 28,5 ms for 262 k agents)"
        );
    }
    assert!(
        separation >= 2.4,
        "separation {separation} lets the settled spacing close up; ≥2.4 is the \
         measured floor for a bounded murmuration"
    );
    assert!(
        params.get("spread").copied().unwrap_or(0.0) > 0.5,
        "spread MUST be on — without it the flock packs into a box and the grid \
         cannot help (O(N²)), which is the exact thing this demo disproves"
    );
}
