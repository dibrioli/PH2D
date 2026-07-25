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

/// The **sim-zone snow globe** (`PH2D_GPU_COOK_DEMO=10`, ADR-0135) must plan as
/// a fully-GPU LOOP whose state lives on the device — the whole point of the sim-
/// zone family landing on the GPU. The smoke would look identical on the CPU pump
/// (a modest fixed population is cheap either way), so a regression that made the
/// zone stop being claimed — its `StateSelect` unregistered, `sim.step`/`sim.collide`
/// losing a kernel, or the retreat firing on a fully-covered loop — has to be
/// caught HERE, headless, not by eye.
#[test]
fn the_zone_demo_document_plans_as_a_fully_gpu_loop() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_zone_demo_document(&mut doc, &registry).expect("well-typed zone demo");
    let out = *sinks.first().expect("one sink");

    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the fixed-population zone must leave no boundary: {:?}",
        plan.boundaries
    );
    assert!(
        plan.drives_a_loop(),
        "the zone's state loop must live on the GPU — otherwise this is not a sim"
    );
    // grid + move + wind + buoyancy + sim.step + sim.collide + scale dispatch;
    // sim.zone and output are pass-throughs (the zone is a conditional select).
    assert_eq!(plan.dispatching_stages(&registry), 7);

    let node = |ty: &str| {
        doc.graph
            .nodes()
            .iter()
            .position(|n| n.type_name == ty)
            .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
            .unwrap_or_else(|| panic!("the demo has a {ty}"))
    };
    // The zone IS staged (a conditional passthrough), and the loop head reads its
    // PREVIOUS output — the state entry the editor draws as a portal badge.
    let (zone, head) = (node("sim.zone"), node("force.wind"));
    assert!(
        plan.stages.iter().any(|s| s.node == zone),
        "the sim.zone must be claimed, not left a boundary"
    );
    let staged_head = plan
        .stages
        .iter()
        .find(|s| s.node == head)
        .expect("the interior head is staged");
    assert_eq!(
        staged_head.inputs.first(),
        Some(&ph2d_gpu_cook::GpuSource::Prev(zone)),
        "gravity reads last tick's zone output — the state loop closes on the device"
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

/// **The boot snow claims its loop over the static poisson (ADR-0136).** The
/// artist document the app opens with: birth (`sim.spawn` + `motion.combine`),
/// two deaths (`sim.lifetime`, `motion.falloff` + `motion.cull`), the age chain
/// (`value.attribute` → `motion.color_ramp.t`) and the world (`sim.step` +
/// `sim.collide` + forces) must ALL be staged, the plan must drive the loop on
/// the device, and the ONE boundary left is `motion.distribute_poisson` — a
/// static template (Bridson is sequential by nature; the plan keeps it on the
/// pump because a constant is not a second simulation of anything).
///
/// This is the plan-shape half of the slice's goal; the device half is
/// `gpu_stream_ops::the_birth_zone_loop_lives_and_dies_on_the_device_matching_the_cpu`.
#[test]
fn the_boot_snow_claims_the_loop_and_only_the_poisson_stays_cpu() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    let mut doc = ph2d_motion_doc::MotionDoc::new();
    let sinks = build_default_document(&mut doc, &reg).expect("the boot document builds");
    let g = &doc.graph;

    // The snow's render sink is the one whose plan drives the sim loop.
    let mut looped = 0usize;
    for sink in &sinks {
        let plan = ph2d_gpu_cook::plan(g, &reg, &reg, *sink);
        if !plan.drives_a_loop() {
            continue;
        }
        looped += 1;
        for (n, _) in &plan.boundaries {
            let ty = g.node(*n).expect("boundary exists").type_name.as_str();
            assert_eq!(
                ty, "motion.distribute_poisson",
                "the only CPU node left in the snow is the static template, found `{ty}`"
            );
        }
        assert!(
            !plan.boundaries.is_empty(),
            "the poisson template must be a boundary (it has no kernel by design)"
        );
        // Every count-changing family member is STAGED — the claim is whole.
        for ty in [
            "sim.spawn",
            "motion.combine",
            "sim.lifetime",
            "motion.cull",
            "value.attribute",
            "motion.color_ramp",
            "sim.zone",
        ] {
            assert!(
                plan.stages
                    .iter()
                    .any(|s| g.node(s.node).is_some_and(|i| i.type_name == ty)),
                "`{ty}` must be staged in the snow's plan"
            );
        }
    }
    assert_eq!(looped, 1, "exactly one sink drives the snow's sim loop");
}

/// The **FIELD family smoke really runs on the device** (`PH2D_GPU_COOK_DEMO=17`).
///
/// `field.index_range` writes the `falloff` mask on the GPU and a Solid tint reads
/// it there; the whole `grid → field.index_range → tint → output` chain must be
/// claimed whole, or the artist would smoke the CPU pump's memo (the band looks
/// identical on screen, just cooked slower) and sign off on a path that never ran
/// — the exact failure this file exists to prevent.
#[test]
fn the_field_index_range_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_index_range_demo_document(&mut doc, &registry)
        .expect("well-typed field.index_range demo");
    let out = *sinks.first().expect("one sink");
    // The scene really contains the node under test — a gate that planned an empty
    // or wrong graph fully-GPU would be vacuously green.
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.index_range")),
        "the demo must contain the field.index_range node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.index_range -> tint must be claimed whole — a CPU boundary here and \
         the smoke would be reading the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// The **spatial** field smoke really runs on the device (`PH2D_GPU_COOK_DEMO=18`).
/// `field.box` reads `P` and writes the `falloff` mask on the GPU; the whole
/// `grid -> motion.scale -> field.box -> tint -> output` chain must be claimed
/// whole, or the artist smokes the CPU pump's memo, not the device.
#[test]
fn the_field_box_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_field_box_demo_document(&mut doc, &registry).expect("well-typed field.box demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.box")),
        "the demo must contain the field.box node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.box -> tint must be claimed whole — a CPU boundary here and the \
         smoke would be reading the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// The **composition** smoke really runs on the device (`PH2D_GPU_COOK_DEMO=19`).
/// Two field branches off one grid (a fan-out) unioned by `field.combine`; the
/// whole fan-out — grid cooked once, both branches, the 2-input composer, the
/// tint — must be claimed whole, or the artist smokes the CPU pump, not the device.
#[test]
fn the_field_combine_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_combine_demo_document(&mut doc, &registry)
        .expect("well-typed field.combine demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.combine")),
        "the demo must contain the field.combine node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the field.combine fan-out must be claimed whole — a CPU boundary here and \
         the smoke reads the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// `field.radial_sweep` reads `P` and writes the `falloff` mask on the GPU; the
/// whole `grid -> motion.scale -> field.radial_sweep -> tint -> output` chain must
/// be claimed whole, or the `=20` smoke reads the CPU pump's memo, not the device.
/// The angular sector is HR-5 (no `atan2`), so nothing about it forces a boundary.
#[test]
fn the_field_radial_sweep_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_radial_sweep_demo_document(&mut doc, &registry)
        .expect("well-typed field.radial_sweep demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.radial_sweep")),
        "the demo must contain the field.radial_sweep node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.radial_sweep -> tint must be claimed whole — a CPU boundary here and \
         the smoke reads the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// `field.remap` reads and rewrites the `falloff` mask on the GPU; the whole
/// `grid -> motion.scale -> field.box -> field.remap -> tint -> output` chain must be
/// claimed whole, or the `=21` smoke reads the CPU pump's memo, not the device. The
/// remap is a position-blind transfer function, so nothing about it forces a boundary.
#[test]
fn the_field_remap_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_remap_demo_document(&mut doc, &registry)
        .expect("well-typed field.remap demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.remap")),
        "the demo must contain the field.remap node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.box -> field.remap -> tint must be claimed whole — a CPU boundary here \
         and the smoke reads the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// The **Curve contour demo is a MIXED plan** (`PH2D_GPU_COOK_DEMO=22`): the same
/// `field.remap`, but with `contour = Curve`, whose shape is a text param the uniform
/// layout cannot carry — so its kernel declines (`applicable` false at mode 4) and the
/// remap is a CPU boundary while the box before it and the tint after it stay on the GPU.
/// This pins the CPU↔GPU boundary at the PLAN level (the `motion.oscillator` precedent):
/// were the kernel to stop declining, the smoke would cook a WRONG mask on the device with
/// nothing on screen to say so. A1-gpu bakes the LUT and this demo becomes fully-GPU.
#[test]
fn the_field_curve_demo_falls_back_to_the_cpu_for_the_contour() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_curve_demo_document(&mut doc, &registry)
        .expect("well-typed field.remap curve demo");
    let out = *sinks.first().expect("one sink");
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    // NOT fully-GPU — the Curve contour cannot be a per-element WGSL uniform.
    assert!(
        !plan.is_fully_gpu(),
        "the Curve contour must force a CPU boundary, not plan as fully-GPU"
    );
    // …and the boundary is the remap itself (the node that declines mode 4).
    assert!(
        plan.boundaries.iter().any(|&(n, _)| doc
            .graph
            .node(n)
            .is_some_and(|node| node.type_name == "field.remap")),
        "the CPU boundary must be the field.remap running the Curve contour: {:?}",
        plan.boundaries
    );
}
