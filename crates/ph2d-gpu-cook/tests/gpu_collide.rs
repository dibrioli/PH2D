//! **Push-apart on the GPU** (ADR-0134 Fase 5) — the `O(N²·iterations)` packing,
//! answered by the spatial grid, reconciled against the CPU's averaged Jacobi.
//!
//! This is the first ITERATED kernel: the sequencer runs the pass `iterations`
//! times (`GridSpec::sweeps_param`), rebuilding the grid between sweeps because
//! each sweep moves the very column the grid indexes. A grid built once would, by
//! sweep 2, be answering *"who was near you before you moved?"* — so a gate that
//! only ever ran ONE sweep would prove nothing about the thing being built.
//!
//! Parity is ε, not bit-exact, and the reason is structural: both sides compute the
//! same per-disc AVERAGE over the same contact set, but the CPU sums in index order
//! and the device sums in grid-traversal order. Float addition is not associative,
//! and eight sweeps feed each sweep's difference into the next.
//!
//! `#[ignore]`: needs an adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_collide --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];

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
    ph2d_node_motion_collide::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

/// `grid → collide → output`. The lattice pitch is far tighter than `2·radius`, so
/// essentially every disc starts overlapping several neighbours — the crowded case
/// the solver exists for, and the one where a missed contact shows up immediately.
fn collide_graph(side: f32, iterations: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", side);
    g.set_param(src, "cols", side);
    g.set_param(src, "gap_x", 0.25);
    g.set_param(src, "gap_y", 0.25);
    let col = g.add_node("motion.collide");
    // radius 0.3 ⇒ min_dist 0.6, against a pitch of 0.25: heavy overlap.
    // Non-round weights so a swapped term cannot hide behind a tidy number.
    g.set_param(col, "radius", 0.3);
    g.set_param(col, "iterations", iterations);
    g.set_param(col, "strength", 0.85);
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (src, 0),
        to: (col, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (col, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    (g, out)
}

fn cpu_cook(g: &Graph, reg: &NodeRegistry, out: NodeId) -> Vec<RenderInstance> {
    let mut cook = Cook::new();
    let mut lowered = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        g,
        reg,
        out,
        0.0,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut lowered,
    )
    .expect("cpu cook");
    lowered
}

fn gpu_cook(gpu: &GpuContext, g: &Graph, reg: &NodeRegistry, out: NodeId) -> Vec<RenderInstance> {
    let plan = plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    // grid + collide dispatch; `output` is pass-through. A silent CPU fallback
    // would compare the CPU against itself and pass on a kernel that never ran.
    assert_eq!(plan.dispatching_stages(reg), 2);
    let mut gc = GpuCook::new();
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &[],
        CookClock {
            playhead: 0.0,
            tick: Some(0),
        },
        DEFAULT_UV,
        DEFAULT_SIZE,
    )
    .expect("gpu cook");
    read_instances(gpu, gc.instances().expect("cooked"))
}

fn parity(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance], eps: f32) -> f32 {
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
        "collide {label}: {} discs, max |Δpos| = {max_pos:e}",
        cpu.len()
    );
    max_pos
}

/// ONE sweep: the neighbour SET must match the CPU's all-pairs exactly. With a
/// single sweep there is no accumulation across iterations, so this isolates the
/// grid query itself — if the 3×3-derived reach ever missed a contact, it shows
/// here as a hard divergence rather than as drift.
#[test]
#[ignore = "needs a GPU adapter"]
fn one_push_apart_sweep_matches_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_collide");
        return;
    };
    let reg = registry();
    let (g, out) = collide_graph(16.0, 1.0);
    let cpu = cpu_cook(&g, &reg, out);
    let gpu_out = gpu_cook(&gpu, &g, &reg, out);
    parity("1 sweep", &cpu, &gpu_out, 1e-5);
}

/// The SHIPPED default of 8 sweeps — the iterated path, with the grid rebuilt
/// between each. This is what `GridSpec::sweeps_param` exists for; running it at
/// one sweep would leave the whole feature untested.
#[test]
#[ignore = "needs a GPU adapter"]
fn eight_push_apart_sweeps_match_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_collide");
        return;
    };
    let reg = registry();
    let (g, out) = collide_graph(16.0, 8.0);
    let cpu = cpu_cook(&g, &reg, out);
    let gpu_out = gpu_cook(&gpu, &g, &reg, out);
    parity("8 sweeps", &cpu, &gpu_out, 2e-3);
}

/// **How far does the packing scale?** (§0.0 — the cap a slider would quote.) The
/// CPU is `O(N²·iterations)`, so a million discs there is ~10¹² pair tests per
/// sweep; the grid makes it `O(N·k·iterations)`. Density needs no special care
/// here — the lattice source keeps the pitch constant as it grows, so `k` is
/// bounded by construction (unlike boids, whose seek spring packs the flock).
///
/// `#[ignore]`: a measurement, not a gate.
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn how_far_does_the_packing_scale() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_collide scale");
        return;
    };
    let reg = registry();
    eprintln!("\npush-apart on the GPU — 8 sweeps, grid rebuilt between each:");
    eprintln!("  {:>10}  {:>10}  {:>10}", "discs", "ms/cook", "ns/disc");
    for &side in &[256u32, 512, 1024, 2048] {
        let (g, out) = collide_graph(side as f32, 8.0);
        let plan = plan(&g, &reg, &reg, out);
        assert!(plan.is_fully_gpu());
        let mut gc = GpuCook::new();
        let run = |gc: &mut GpuCook| {
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[],
                CookClock {
                    playhead: 0.0,
                    tick: Some(0),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        for _ in 0..2 {
            run(&mut gc);
        }
        let n = side as u64 * side as u64;
        let start = std::time::Instant::now();
        const REPS: u32 = 4;
        for _ in 0..REPS {
            run(&mut gc);
        }
        let ms = start.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
        eprintln!("  {n:>10}  {ms:>10.3}  {:>10.2}", ms * 1e6 / n as f64);
    }
    eprintln!(
        "\n(the CPU reference is O(N²·iters): a million discs is ~10¹² pair tests\n\
         per sweep, which is why this node had never left a few thousand.)"
    );
}

/// **Zero iterations is the IDENTITY**, on both paths — the CPU clamps to `[0,64]`
/// and `for _ in 0..0` never runs. The device must not "helpfully" sweep once: a
/// stage that dispatches when the reference does not is a product that disagrees
/// with itself at one end of a slider the artist can reach.
#[test]
#[ignore = "needs a GPU adapter"]
fn zero_iterations_is_the_identity_on_the_device_too() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_collide");
        return;
    };
    let reg = registry();
    let (g, out) = collide_graph(16.0, 0.0);
    let cpu = cpu_cook(&g, &reg, out);
    let gpu_out = gpu_cook(&gpu, &g, &reg, out);
    // Bit-exact: nothing was computed, so nothing can have drifted.
    let worst = parity("0 sweeps", &cpu, &gpu_out, 0.0);
    assert_eq!(worst, 0.0);
}
