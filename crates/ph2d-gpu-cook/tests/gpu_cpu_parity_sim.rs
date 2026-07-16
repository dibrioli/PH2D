//! GPU-vs-CPU parity for the **simulation loop** (GPU/M5 Fase 3, ADR-0123) —
//! THE audit of the slice ("o gate de paridade É o audit").
//!
//! ## Why this gate is shaped differently from Fase 2's
//!
//! A stateless kernel is `f(params, playhead)`: every frame is independent, so
//! its ε is bounded per frame and the Fase 2 gates compare no trajectory at all.
//! A sim is `x_{n+1} = f(x_n)` — ε feeds back, and after N ticks the GPU and the
//! CPU are legitimately **different animations** (ADR-0123 D4; the CPU stays the
//! canonical path, ADR-0122). So this compares a **seeded state plus one step**,
//! against the same ε budget Fase 2 derived. It never runs a long trajectory and
//! loosens the tolerance until it passes — that oracle would model the filter,
//! not the truth ([[reference_topic_oracle_discipline]]).
//!
//! ## The false green this had to be built against
//!
//! One step of a fresh seed moves each element by `a·dt²` — at `dt = 1/60` and a
//! plausible strength, ~1e-3, which is the ε budget itself. The comparison would
//! then be "two seeds agree", and it would stay green **with the integrator
//! deleted**. So every gate here first asserts, on the CPU side alone, that the
//! step MOVED the field by orders of magnitude more than ε
//! ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]] — and the
//! fixture-shaped half of it: a gate only proves what its fixture contains).
//!
//! `#[ignore]`: needs a real adapter. Run on a dev machine / GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_sim --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_integrate::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    ph2d_node_force_drag::register(&mut reg).unwrap();
    ph2d_node_force_attractor::register(&mut reg).unwrap();
    ph2d_node_force_vortex::register(&mut reg).unwrap();
    ph2d_node_force_curl::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
/// The Fase 2 budget, unchanged — a sim gets no tolerance discount.
const EPS_POS: f32 = 2e-3;
/// A step must move the field by far more than ε or the comparison is vacuous.
const MUST_MOVE: f32 = 50.0 * EPS_POS;
/// Deliberately NOT 1/60: `dt` is squared into the displacement, so the
/// product's timestep would put one step's motion down at the ε floor and the
/// gate could not see the integrator at all. `MAX_DT` (0.1) is the clamp — stay
/// off the boundary.
const FIXED_DT: f64 = 0.05;

fn edge(g: &mut Graph, from: NodeId, to: NodeId, port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed,
    })
    .expect("well-formed edge");
}

/// `grid → integrate → output`, with `forces` the chain built by `forces`
/// (source→sink, wired into the `pre` loop). 80×80 = 6400 instances.
fn sim_chain(reg: &NodeRegistry, forces: &[(&str, &[(&str, f32)])]) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 80.0);
    g.set_param(grid, "cols", 80.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, ig, 0, false);
    edge(&mut g, ig, out, 0, false);

    match forces {
        // No chain: the bare self-loop the editor auto-wires.
        [] => edge(&mut g, ig, ig, 1, true),
        _ => {
            let nodes: Vec<NodeId> = forces
                .iter()
                .map(|(ty, params)| {
                    let n = g.add_node(*ty);
                    for (k, v) in *params {
                        g.set_param(n, *k, *v);
                    }
                    n
                })
                .collect();
            // integrate --pre--> f0 → f1 → … --fwd--> integrate.forces
            edge(&mut g, ig, nodes[0], 0, true);
            for w in nodes.windows(2) {
                edge(&mut g, w[0], w[1], 0, false);
            }
            edge(&mut g, *nodes.last().expect("non-empty"), ig, 1, false);
        }
    }
    g.validate(reg).expect("well-typed");
    (g, ig, out)
}

fn assert_close(what: &str, i: usize, a: f32, b: f32, eps: f32) {
    assert!(
        (a - b).abs() <= eps,
        "instance {i} field {what}: cpu {a} vs gpu {b} (|diff| {} > eps {eps})",
        (a - b).abs()
    );
}

fn assert_parity(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance]) {
    assert_eq!(cpu.len(), gpu.len(), "instance count");
    let mut max_pos = 0.0f32;
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            assert_close("world_pos", i, c.world_pos[k], g.world_pos[k], EPS_POS);
            max_pos = max_pos.max((c.world_pos[k] - g.world_pos[k]).abs());
            assert_close("size", i, c.size[k], g.size[k], 1e-5);
        }
        for k in 0..4 {
            assert_close("tint", i, c.tint[k], g.tint[k], 1e-6);
            assert_close("basis", i, c.basis[k], g.basis[k], 1e-4);
        }
    }
    eprintln!("{label}: {} instances, max |Δpos| = {max_pos:e}", cpu.len());
}

/// The largest per-element move between two lowerings of the same field.
fn max_move(a: &[RenderInstance], b: &[RenderInstance]) -> f32 {
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| (0..2).map(move |k| (x.world_pos[k] - y.world_pos[k]).abs()))
        .fold(0.0f32, f32::max)
}

/// Cook `ticks` ticks on the canonical CPU path, returning each tick's lowering.
/// `advance_tick` is what publishes the `pre` feedback, so it is the CPU's own
/// definition of "a tick happened" — the GPU mirrors it by holding the `Arc`s.
fn cpu_ticks(g: &Graph, reg: &NodeRegistry, out: NodeId, ticks: u64) -> Vec<Vec<RenderInstance>> {
    let mut cook = Cook::new();
    let mut frames = Vec::new();
    for t in 0..=ticks {
        let playhead = t as f64 * FIXED_DT;
        let mut lowered = Vec::new();
        ph2d_eval_motion::evaluate_motion_into(
            &mut cook,
            g,
            reg,
            out,
            playhead,
            DEFAULT_UV,
            DEFAULT_SIZE,
            &mut lowered,
        )
        .expect("cpu cook");
        cook.advance_tick(g, reg, playhead).expect("cpu tick");
        frames.push(lowered);
    }
    frames
}

/// Cook `ticks` ticks on the GPU, returning the LAST tick's lowering. Asserts
/// the plan claims the loop and dispatches — a silent CPU fallback would make
/// every comparison below compare the CPU to itself.
fn gpu_ticks(
    gpu: &GpuContext,
    g: &Graph,
    reg: &NodeRegistry,
    out: NodeId,
    ticks: u64,
    want_stages: usize,
) -> Vec<RenderInstance> {
    let plan = plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    assert!(plan.drives_a_loop(), "the state must live on the GPU");
    assert_eq!(
        plan.dispatching_stages(reg),
        want_stages,
        "every sim node must actually dispatch"
    );
    let mut gc = GpuCook::new();
    for t in 0..=ticks {
        gc.cook(
            gpu,
            g,
            reg,
            reg,
            &plan,
            &[],
            CookClock {
                playhead: t as f64 * FIXED_DT,
                tick: Some(t),
            },
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    }
    read_instances(gpu, gc.instances().expect("cooked"))
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_seed_matches_the_cpu() {
    // Tick 0: the `pre` reads Empty on both sides, so the integrator SEEDS —
    // `sim_d = 0`, `vel` from `rest`, `P = rest.P`. The branch a zeroed step
    // would silently replace (`HAS_forces_sim_d`), on the real product path.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(&reg, &[]);
    let cpu = cpu_ticks(&g, &reg, out, 0);
    // grid + integrate dispatch; `output` is a pass-through.
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 0, 2);
    assert_parity("seed", &cpu[0], &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_seed_takes_its_velocity_from_rest_not_from_the_state() {
    // `motion.integrate` reads `vel` off BOTH its ports — the seed off `rest`
    // (an emitter's muzzle velocity is what launches a fresh particle) and the
    // step off `forces`. Getting that backwards is the one mistake a per-port
    // reader exists to prevent, and it is INVISIBLE to every other gate here:
    // the grid emits no `vel`, so both readers answer their identity and the two
    // wires are indistinguishable ([[feedback_a_gate_only_proves_what_its_fixture_contains]]
    // — swapping them survived the whole suite).
    //
    // No shipping generator kernel emits `vel` yet, so the fixture is the shape
    // that will: a source with a muzzle velocity. Without forces, tick 1's
    // displacement IS the seed velocity — read it off the wrong port and the
    // field does not move at all.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let mut reg = registry();
    velgen::register(&mut reg);
    let mut g = Graph::new();
    let src = g.add_node("test.velgen");
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, src, ig, 0, false);
    edge(&mut g, ig, ig, 1, true);
    edge(&mut g, ig, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let cpu = cpu_ticks(&g, &reg, out, 1);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1, 2);
    let launched = max_move(&cpu[0], &cpu[1]);
    assert!(
        launched > MUST_MOVE,
        "the muzzle velocity must launch the element ({launched}) — otherwise \
         the seed is reading a zero and this proves nothing"
    );
    eprintln!("the seed velocity launched the field by {launched}");
    assert_parity("seed velocity", &cpu[1], &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn one_step_of_wind_and_drag_matches_the_cpu() {
    // The full loop: `integrate --pre--> wind → drag --fwd--> integrate.forces`.
    //
    // TWO ticks before the comparison, not one, and the reason is `drag`: it
    // reads `vel`, and a fresh seed has none — a one-tick fixture would leave
    // `−k·v = 0` and gate the drag kernel against nothing at all. So the state
    // is seeded through tick 1 (where a velocity comes to exist) and tick 2 is
    // the step under test. Two steps of ε, at the Fase 2 budget, unloosened.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(
        &reg,
        &[
            (
                "force.wind",
                &[
                    ("angle", 37.5),
                    ("strength", 42.0),
                    ("gust", 0.65),
                    ("gust_freq", 1.7),
                    ("seed", 3.25),
                ],
            ),
            ("force.drag", &[("coefficient", 2.75)]),
        ],
    );
    let cpu = cpu_ticks(&g, &reg, out, 2);
    // grid + wind + drag + integrate.
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 2, 4);

    let moved = max_move(&cpu[0], &cpu[2]);
    assert!(
        moved > MUST_MOVE,
        "the fixture must actually integrate — moved {moved} ≤ {MUST_MOVE}, so \
         this comparison would pass with the integrator dead"
    );
    eprintln!("wind+drag moved the field by {moved}");
    assert_parity("wind+drag one step", &cpu[2], &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn one_step_of_attractor_and_vortex_matches_the_cpu() {
    // The classic stable-orbit pair, on the field's own scale (the 80×80 grid
    // spans roughly ±14, so a radius of 9.5 leaves elements on BOTH sides of the
    // cutoff — the `d > radius` early-out and the dead zone are branches, and a
    // fixture entirely inside the radius would never take them).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(
        &reg,
        &[
            (
                "force.attractor",
                &[
                    ("target_x", 1.25),
                    ("target_y", -0.75),
                    ("strength", 55.0),
                    ("radius", 9.5),
                    ("curve", 1.0),
                    ("repel", 0.0),
                ],
            ),
            (
                "force.vortex",
                &[
                    ("center_x", 1.25),
                    ("center_y", -0.75),
                    ("strength", 40.0),
                    ("radius", 9.5),
                    ("clockwise", 1.0),
                ],
            ),
        ],
    );
    let cpu = cpu_ticks(&g, &reg, out, 2);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 2, 4);
    let moved = max_move(&cpu[0], &cpu[2]);
    assert!(
        moved > MUST_MOVE,
        "the fixture must actually integrate: {moved}"
    );
    eprintln!("attractor+vortex moved the field by {moved}");
    assert_parity("attractor+vortex one step", &cpu[2], &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn one_step_of_curl_matches_the_cpu() {
    // The heaviest kernel: 4 octaves × 4 psi samples × 4 lattice hashes. If the
    // integer hash diverged by ONE cell the noise would jump to an unrelated
    // pseudo-random value — O(amplitude), not ε — so this either matches
    // closely or fails loudly. `octaves` is deliberately x.5-free: 3.0, not a
    // half-way value whose rounding convention differs between the two sides.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(
        &reg,
        &[(
            "force.curl",
            &[
                ("strength", 48.0),
                ("scale", 0.23),
                ("speed", 0.85),
                ("octaves", 3.0),
                ("seed", 1.75),
            ],
        )],
    );
    let cpu = cpu_ticks(&g, &reg, out, 2);
    // grid + curl + integrate.
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 2, 3);
    let moved = max_move(&cpu[0], &cpu[2]);
    assert!(
        moved > MUST_MOVE,
        "the fixture must actually integrate: {moved}"
    );
    eprintln!("curl moved the field by {moved}");
    assert_parity("curl one step", &cpu[2], &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_state_is_double_buffered_and_a_steady_sim_stops_allocating() {
    // D1's claim is that holding last tick's `Arc`s IS the state — which only
    // holds up if the pool keeps RECYCLING around them. A `reclaim` that forgot
    // a still-referenced buffer would free it the moment the sim released it,
    // and a 2M-element sim would allocate its whole state every frame: the exact
    // thing the pool exists to prevent, and invisible to every parity gate.
    //
    // The scrub ring is squeezed to one checkpoint first, deliberately. It pins
    // VRAM BY DESIGN (that is what a window costs), so leaving it at its default
    // budget would measure the ring's growth and say nothing about the
    // ping-pong. Two properties, two knobs, one at a time
    // ([[feedback_layered_defenses_need_per_layer_gates]]).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(&reg, &[("force.drag", &[("coefficient", 1.5)])]);
    let plan = plan(&g, &reg, &reg, out);
    assert!(plan.drives_a_loop());
    let mut gc = GpuCook::new();
    gc.set_ring_budget(0);
    let cook = |gc: &mut GpuCook, t: u64| {
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock {
                playhead: t as f64 * FIXED_DT,
                tick: Some(t),
            },
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    };
    // Warm up: the first ticks legitimately allocate (pool empty, pipelines
    // cold), and the stride's checkpoints churn until the budget evicts them.
    for t in 0..24 {
        cook(&mut gc, t);
    }
    let warm = gc.pool_allocations();
    for t in 24..64 {
        cook(&mut gc, t);
    }
    assert_eq!(
        gc.pool_allocations(),
        warm,
        "a steady sim must allocate NOTHING after warm-up (the ping-pong is a refcount)"
    );
    // And the state really is held: exactly the sim's columns stay referenced.
    assert!(
        gc.pool_retained() > 0,
        "last tick's columns must still be referenced — else `prev` is empty and \
         the sim seeds every tick"
    );
}

/// A generator WITH a kernel that stamps a **muzzle velocity** — the shape a GPU
/// emitter will have, so the integrator's seed-from-`rest` wire is reachable
/// today rather than the day it ships. (Deliberately no `id`: identity is the
/// gather D3 refuses; this fixture is about `vel` alone.)
mod velgen {
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    pub const N: usize = 2048;
    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.velgen"),
        name: "test.velgen",
        inputs: &[],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };

    /// Non-round, per-element, and large enough that one step's displacement is
    /// orders above ε ([[feedback_test_with_product_numbers_not_convenient_ones]]).
    fn muzzle(i: usize) -> [f32; 2] {
        let k = i as f32;
        [11.375 + k * 0.0125, -7.625 + k * 0.0075]
    }
    fn pos(i: usize) -> [f32; 2] {
        let k = i as f32;
        [k * 0.017 - 3.5, k * 0.011 - 2.25]
    }

    struct Op;
    impl NodeOp for Op {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(N)
                    .with("P", Column::Vec2((0..N).map(pos).collect()))
                    .with("vel", Column::Vec2((0..N).map(muzzle).collect())),
            );
        }
    }

    const KERNEL: GpuKernel = GpuKernel {
        // The same arithmetic as `pos`/`muzzle`, in the same order.
        wgsl: "\
            let k = f32(i);\n\
            write_P(i, vec2<f32>(k * 0.017 - 3.5, k * 0.011 - 2.25));\n\
            write_vel(i, vec2<f32>(11.375 + k * 0.0125, -7.625 + k * 0.0075));\n",
        wgsl_lib: "",
        bindings: &[
            ColumnBinding {
                column: "P",
                dim: Dim::Vec2,
                access: ColumnAccess::Write,
                identity: [0.0; 4],
                port: 0,
            },
            ColumnBinding {
                column: "vel",
                dim: Dim::Vec2,
                access: ColumnAccess::Write,
                identity: [0.0; 4],
                port: 0,
            },
        ],
        params: &[],
        source_count: Some(|_| N),
        applicable: None,
    };

    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(Op)).unwrap();
        reg.register_gpu_kernel(MAN.id, KERNEL);
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn scrubbing_back_reproduces_the_past_state_instead_of_the_marching_future() {
    // D5 — the whole reason the ring exists, and the failure it prevents is a
    // QUIET one: cook tick 4 holding tick 20's state and the integrator's own
    // guard clamps `dt` to zero (the playhead went backwards), so nothing
    // explodes, nothing NaNs — the field just sits at the future's pose while
    // the ruler says 4. That is the naive-scrub bug the CPU ring was built
    // against, on the other side of the fence.
    //
    // The oracle is the CPU's tick 4, cooked forward from scratch: the same
    // question this engine has to answer after having marched past it.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(
        &reg,
        &[(
            "force.wind",
            &[
                ("angle", 37.5),
                ("strength", 42.0),
                ("gust", 0.65),
                ("gust_freq", 1.7),
                ("seed", 3.25),
            ],
        )],
    );
    const BACK_TO: u64 = 4;
    const MARCHED: u64 = 20;

    let cpu = cpu_ticks(&g, &reg, out, MARCHED);
    // The state at 4 and at 20 must be far apart, or "reproduced the past" and
    // "showed the future" are the same picture and this proves nothing.
    let travelled = max_move(&cpu[BACK_TO as usize], &cpu[MARCHED as usize]);
    assert!(
        travelled > MUST_MOVE,
        "the sim must have gone somewhere between the two ticks ({travelled})"
    );

    let plan = plan(&g, &reg, &reg, out);
    assert!(plan.drives_a_loop());
    let mut gc = GpuCook::new();
    let mut march = |gc: &mut GpuCook, target: u64| {
        for t in gc.rewind_for(target)..=target {
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[],
                CookClock {
                    playhead: t as f64 * FIXED_DT,
                    tick: Some(t),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
        }
        read_instances(&gpu, gc.instances().expect("cooked"))
    };

    // March forward past the target, then scrub back to it.
    let _ = march(&mut gc, MARCHED);
    let (checkpoints, bytes) = gc.ring_stats();
    assert!(
        checkpoints > 1,
        "the ring must hold a window: {checkpoints}"
    );
    eprintln!("ring: {checkpoints} checkpoints pinning {bytes} B");

    let scrubbed = march(&mut gc, BACK_TO);
    assert_eq!(
        gc.last_cooked_tick(),
        Some(BACK_TO),
        "the sim's clock must stand where the playhead does"
    );
    assert_parity("scrub back", &cpu[BACK_TO as usize], &scrubbed);

    // And forward again from there — the restore left a state the march can
    // continue from, not a dead end.
    let resumed = march(&mut gc, MARCHED);
    assert_parity("re-marched", &cpu[MARCHED as usize], &resumed);
}
