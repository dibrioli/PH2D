//! **Boids on the GPU** (ADR-0134 Phase 3b) — the O(N²) flock, answered by the
//! spatial grid, reconciled against the CPU all-pairs.
//!
//! A sim is `x_{n+1} = f(x_n)`, so ε feeds back and a long trajectory drifts
//! (ADR-0127 D4); the gate therefore asserts the SEED (tick 0, where the integer
//! `hash3` makes it **bit-exact**) and ONE step from it (tick 1, where only the
//! float sum ORDER over an identical neighbour set differs → ε). This is also the
//! first exercise of the grid over the `pre` STATE port and the tick-0 empty-grid
//! path — the neighbour gate used a per-element input port, always present.
//!
//! `#[ignore]`: needs an adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_boids --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const FIXED_DT: f64 = 1.0 / 60.0;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_boids::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

/// `boids ──> output`, with the `out ──pre──> state` self-loop the editor auto-wires.
fn boids_graph(count: f32) -> (Graph, NodeId) {
    boids_graph_spread(count, false)
}

/// As [`boids_graph`], with the √N `spread` mode explicit — its one `sqrt` is the
/// only place the two seeds diverge, so `spread` on is an ε seed (not bit-exact).
fn boids_graph_spread(count: f32, spread: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", count);
    g.set_param(boids, "spread", if spread { 1.0 } else { 0.0 });
    g.set_param(boids, "seed", 1.0);
    // Non-round, non-default weights so a swapped rule can't hide behind a tidy
    // number ([[feedback_test_with_product_numbers_not_convenient_ones]]).
    g.set_param(boids, "radius", 2.3);
    g.set_param(boids, "separation", 1.4);
    g.set_param(boids, "alignment", 0.9);
    g.set_param(boids, "cohesion", 0.7);
    g.set_param(boids, "seek", 1.1);
    g.set_param(boids, "max_speed", 4.0);
    let out = g.add_node("motion.output");
    // The self-loop (delayed) + the render edge.
    g.connect(Edge {
        from: (boids, 0),
        to: (boids, 2),
        delayed: true,
    })
    .unwrap();
    g.connect(Edge {
        from: (boids, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    (g, out)
}

/// Cook `0..=ticks` on the canonical CPU path; return each tick's lowering.
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

/// Cook `0..=ticks` on the GPU; return the last tick's lowering. Proves the plan
/// claims the loop and dispatches — a silent CPU fallback would compare CPU to CPU.
fn gpu_ticks(
    gpu: &GpuContext,
    g: &Graph,
    reg: &NodeRegistry,
    out: NodeId,
    ticks: u64,
) -> Vec<RenderInstance> {
    let plan = plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    assert!(plan.drives_a_loop(), "the flock state must live on the GPU");
    assert_eq!(
        plan.dispatching_stages(reg),
        1,
        "boids dispatches; output is pass-through"
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

fn parity(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance], eps: f32) {
    assert_eq!(cpu.len(), gpu.len(), "{label}: instance count");
    let mut max_pos = 0.0f32;
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            let d = (c.world_pos[k] - g.world_pos[k]).abs();
            max_pos = max_pos.max(d);
            assert!(
                d <= eps,
                "{label}: instance {i} world_pos[{k}]: cpu {} vs gpu {} (|diff| {d} > {eps})",
                c.world_pos[k],
                g.world_pos[k]
            );
        }
    }
    eprintln!(
        "boids {label}: {} agents, max |Δpos| = {max_pos:e}",
        cpu.len()
    );
}

#[test]
#[ignore = "needs a GPU adapter"]
fn the_boids_seed_matches_the_cpu_bit_for_bit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, out) = boids_graph(400.0);
    let cpu = cpu_ticks(&g, &reg, out, 0);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 0);
    // The seed is the integer `hash3` on both sides → bit-exact.
    parity("seed", &cpu[0], &gpu_out, 0.0);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn one_boids_step_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, out) = boids_graph(400.0);
    // Tick 1 is ONE step from the seed (which already carries a muzzle velocity),
    // so the three urges + seek all fire. The neighbour SET is identical (grid =
    // all-pairs within radius); only the float SUM order differs ⇒ ε.
    let cpu = cpu_ticks(&g, &reg, out, 1);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1);
    parity("one step", &cpu[1], &gpu_out, 2e-3);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn the_spread_seed_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    // √N spread at a count whose √(count/64) is IRRATIONAL (300/64 = 4.6875 →
    // √ ≈ 2.165), so the CPU/GPU `sqrt` genuinely differs — this exercises the ε,
    // where 400 (=√6.25=2.5 exact) would have hidden it at 0.
    let (g, out) = boids_graph_spread(300.0, true);
    let cpu = cpu_ticks(&g, &reg, out, 0);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 0);
    // Half-extent ≈ 3·√(300/64) ≈ 6.5 world units → a 1-ULP sqrt is ~1e-6.
    parity("spread seed", &cpu[0], &gpu_out, 1e-4);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn one_spread_step_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, out) = boids_graph_spread(400.0, true);
    let cpu = cpu_ticks(&g, &reg, out, 1);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1);
    parity("spread step", &cpu[1], &gpu_out, 2e-3);
}
