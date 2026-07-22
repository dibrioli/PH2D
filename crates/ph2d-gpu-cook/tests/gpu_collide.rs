//! **Push-apart on the GPU** (ADR-0140 Fase 5) — the `O(N²·iterations)` packing,
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

/// **What does the breath COST?** — the reference table for `motion.collide` on
/// the device, and the answer to two reports against `PH2D_GPU_COOK_DEMO=8`
/// (Enio, 2026-07-19): a deep FPS drop on the LFO's positive half, and then again
/// when the LFO's Amplitude is raised.
///
/// The first was a DEFECT and is fixed (the cell cull — see the gate below). The
/// second is **honest work, measured**: what this node costs is contacts-per-disc,
/// and `spread` scales the interaction radius, so contacts grow with its AREA. It
/// is the same law Fase 4 measured on the packed flock — *the grid is `O(N)` only
/// under bounded density* — arriving from the other side: there the crowd got
/// denser, here the reach gets longer, and no spatial structure helps once a
/// neighbourhood contains most of the scene.
///
/// Two optimisations were tried on top of the cull and **both measured NEUTRAL**;
/// they are recorded so nobody spends the day re-deriving them:
/// - **Cheapest-test-first** in the inner loop (distance before the cell dedup and
///   the inverse-mass read). 6.47 → 6.45 ms. The kernel is memory-bound on reading
///   the neighbours' positions, so moving arithmetic behind a filter buys nothing.
/// - **Grid-order thread assignment** (`let me = grid_sorted[i]`, so adjacent
///   threads take spatially adjacent discs). 6.24 ms, and WORSE at scale (1 M:
///   36.9 → 43.6). `motion.grid` already emits its lattice row-major, so index
///   order is ALREADY spatially coherent; the permutation only adds indirection
///   and scatters the writes.
///
/// `#[ignore]`: a measurement, not a gate. The gate is the ratio below.
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn what_does_the_breath_cost() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping breath cost");
        return;
    };
    let mut reg = registry();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    eprintln!("\nmotion.collide on the GPU, 8 sweeps, lattice pitch 0.25, radius 0.3:");
    eprintln!(
        "  {:>8}  {:>8}  {:>8}  {:>7}  {:>10}",
        "discs", "spread", "cells", "reach", "ms/cook"
    );
    // `spread` 0.02 is the FLOOR: reach 1 and essentially no contact, so what it
    // measures is the infrastructure alone (8 grid builds + 8 dispatches + the
    // column traffic). Everything above it is contact work.
    for &(side, spread) in &[
        (360.0f32, 0.02f32),
        (360.0, 0.65),
        (360.0, 1.0),
        (360.0, 1.35),
        (360.0, 2.0),
        (360.0, 3.0),
        (360.0, 4.0),
        (512.0, 1.0),
        (512.0, 1.35),
        (512.0, 2.0),
        (512.0, 3.0),
    ] {
        let ms = breath_ms(&gpu, &reg, side, spread);
        let reach = (2.0 * spread).ceil().max(1.0) as i32;
        let cells = (2 * reach + 1) * (2 * reach + 1);
        let n = (side * side) as u32;
        eprintln!("  {n:>8}  {spread:>8.3}  {cells:>8}  {reach:>7}  {ms:>10.3}");
    }
    eprintln!(
        "\n(the shipped scene is 360² with the LFO at offset 1.0 / amplitude 0.35,\n\
         so it lives in the 0.65-1.35 rows and the artist can double the amplitude\n\
         and still hold 60 fps. The far end of that slider is amplitude 10 ⇒ spread\n\
         11 ⇒ 2025 cells: measured at 314 ms/cook on 262 144 discs. That is not a\n\
         ceiling anyone should hit by accident, which is why the scene ships with\n\
         room instead of at the edge.)"
    );
}

/// Build the scene's topology held still at one `spread`, and return the best of
/// `REPS` cook times (the minimum: noise only ever ADDS time, so the floor is the
/// least contaminated estimate of what the kernel costs).
fn breath_ms(gpu: &GpuContext, reg: &NodeRegistry, side: f32, spread: f32) -> f64 {
    let mut g = Graph::new();
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", side);
    g.set_param(src, "cols", side);
    g.set_param(src, "gap_x", 0.25);
    g.set_param(src, "gap_y", 0.25);
    let col = g.add_node("motion.collide");
    g.set_param(col, "radius", 0.3);
    g.set_param(col, "iterations", 8.0);
    g.set_param(col, "strength", 1.0);
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 0.0);
    g.set_param(lfo, "offset", spread);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(src, col, 0), (lfo, col, 1), (col, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    let plan = plan(&g, reg, reg, out);
    let mut gc = GpuCook::new();
    let run = |gc: &mut GpuCook| {
        gc.cook(
            gpu,
            &g,
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
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    };
    for _ in 0..2 {
        run(&mut gc);
    }
    const REPS: u32 = 6;
    let mut best = f64::INFINITY;
    for _ in 0..REPS {
        let start = std::time::Instant::now();
        run(&mut gc);
        best = best.min(start.elapsed().as_secs_f64() * 1e3);
    }
    best
}

/// **The cost of a packing is a fact of its GEOMETRY, not of where the cell grid
/// happens to fall.** Reported by Enio (2026-07-19) against `PH2D_GPU_COOK_DEMO=8`:
/// a deep FPS drop on the LFO's positive half. It was a step, not a slope — the
/// swept neighbourhood is `(2·reach+1)²` cells with `reach = ceil(min_dist / cell)`,
/// so a `spread` moving through 1.0 stepped 25 cells → 49 and the cook went
/// **7.58 ms → 13.08 ms between two neighbouring values** (1.73×, and 13 ms is 78 %
/// of a 60 fps frame).
///
/// `reach` is right — it is the WORST case, a disc pressed against the far edge of
/// its own cell, and a disc there really can have a contact one cell further out.
/// It is just not what a disc in the MIDDLE of its cell needs, which is nearly all
/// of them. So the kernel now asks each cell whether its nearest point is even in
/// range before touching its discs, and the cells the `ceil` added stop being paid
/// for by discs that cannot use them.
///
/// The oracle is a RATIO across the boundary, measured back to back on one machine:
/// it is what the artist actually feels (a jump), and being a ratio it is immune to
/// machine drift the way a wall-clock bar is not. Removing the cull restores 1.73.
#[test]
#[ignore = "needs a GPU adapter"]
fn crossing_the_reach_boundary_does_not_step_the_cost() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping reach-boundary gate");
        return;
    };
    let mut reg = registry();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    // Either side of `spread = 1`, where `ceil(2·spread)` steps 2 → 3. The two
    // packings are the same to a tenth of a percent; only the sweep changes.
    let under = breath_ms(&gpu, &reg, 512.0, 0.999);
    let over = breath_ms(&gpu, &reg, 512.0, 1.001);
    let ratio = over / under;
    eprintln!(
        "reach boundary: spread 0.999 = {under:.3} ms, 1.001 = {over:.3} ms, ratio {ratio:.3}"
    );
    // The step was 1.73×. A tenth of a percent more radius may cost a little more
    // honest work, but it cannot cost a QUARTER more.
    assert!(
        ratio < 1.25,
        "the cook stepped {ratio:.3}× across the reach boundary ({under:.3} → {over:.3} ms): \
         a disc is paying for cells it cannot reach"
    );
}
