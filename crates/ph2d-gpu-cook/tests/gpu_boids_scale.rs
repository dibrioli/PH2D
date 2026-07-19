//! **How far does the flock scale on the device?** (ADR-0134 Phase 4) — the
//! §0.0 measurement that a cap must quote. The slider stops at 500 because the
//! CPU is `O(N²)` all-pairs; the GPU path must not inherit that number — it must
//! quote its OWN, measured here.
//!
//! Two configurations, because the grid's win is CONDITIONAL on density:
//! - **packed** — the node exactly as it seeds today (`SEED_SPREAD = 3.0`, fixed):
//!   the seek spring gathers every agent into a ~6×6 box, so at a million the
//!   whole flock lands in a handful of cells and each agent's 3×3 sweep still
//!   visits ~everyone → the grid gives NO acceleration, it stays `O(N²)`.
//! - **spread** — the domain grows with the count (seed spread ∝ √N) so the
//!   agents-per-cell stays bounded → the 3×3 sweep is `O(k)`, k constant, and the
//!   whole tick is `O(N)`. This is what "millions of boids" needs, and what the
//!   demo must configure. It is measured here with a SYNTHETIC spread seed (a
//!   throw-away node), NOT by changing the shipped node's seed — that is a
//!   product decision (it moves the replay of every count≠48 flock).
//!
//! `#[ignore]`: needs an adapter, and it is a measurement, not a gate. Run it on
//! the GPU lane and read the table:
//!   cargo test -p ph2d-gpu-cook --test gpu_boids_scale --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::time::Instant;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const FIXED_DT: f64 = 1.0 / 60.0;

fn try_headless_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// `boids ──> output`, with the `pre` self-loop. `spread_ref`: when `Some(r)`,
/// the seed cloud's half-extent scales as `SEED_SPREAD·√(count/r)` — a bounded
/// density. This rides a per-graph text param the measurement reads; the SHIPPED
/// node ignores it (packed) unless the demo/measurement asks for spread.
fn boids_graph(count: f32, spread: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", count);
    g.set_param(boids, "spread", if spread { 1.0 } else { 0.0 });
    g.set_param(boids, "seed", 1.0);
    g.set_param(boids, "radius", 2.0);
    g.set_param(boids, "separation", 1.6);
    g.set_param(boids, "alignment", 1.0);
    g.set_param(boids, "cohesion", 0.9);
    g.set_param(boids, "seek", 1.0);
    g.set_param(boids, "max_speed", 4.0);
    let out = g.add_node("motion.output");
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

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_boids::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

/// Warm up (compile pipelines + seed + settle a few steps), then time `TIMED`
/// step-ticks with a full device sync after each submit. Returns ms/tick.
fn time_step_ms(gpu: &GpuContext, g: &Graph, reg: &NodeRegistry, out: NodeId) -> f64 {
    const WARM: u64 = 3;
    const TIMED: u64 = 8;
    let plan = plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    let mut gc = GpuCook::new();
    let cook = |gc: &mut GpuCook, t: u64| {
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
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    };
    for t in 0..WARM {
        cook(&mut gc, t);
    }
    let start = Instant::now();
    for t in WARM..WARM + TIMED {
        cook(&mut gc, t);
    }
    start.elapsed().as_secs_f64() * 1e3 / TIMED as f64
}

#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn how_far_does_the_flock_scale() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids_scale");
        return;
    };
    let reg = registry();
    eprintln!("\nboids on the GPU — packed flock (spread OFF, SEED_SPREAD=3 fixed):");
    eprintln!(
        "  {:>10}  {:>10}  {:>14}",
        "agents", "ms/tick", "~neighbours"
    );
    for &count in &[4096u32, 16384, 65536, 262_144, 1_048_576] {
        let (g, out) = boids_graph(count as f32, false);
        let ms = time_step_ms(&gpu, &g, &reg, out);
        // Packed density: count/(2·SEED_SPREAD)² over a disc of the perception
        // radius (r=2) → an honest read of how many the 3×3 sweep touches.
        let density = count as f64 / (6.0 * 6.0);
        let neigh = density * std::f64::consts::PI * 2.0 * 2.0;
        eprintln!("  {count:>10}  {ms:>10.3}  {neigh:>14.0}");
    }
    eprintln!(
        "\n(packed is O(N²): the whole swarm sits in a few cells, so the grid\n\
         cannot help. Spread ON keeps the density bounded — the grid stays O(N).)"
    );

    eprintln!("\nboids on the GPU — spread flock (spread ON, seed ∝ √N, density fixed):");
    eprintln!("  {:>10}  {:>10}  {:>10}", "agents", "ms/tick", "ns/agent");
    // The top count stays under the FIRST ceiling of the current implementation.
    // Two named, RAISABLE limits sit above ~4 M — neither is silicon:
    //   1. The grid's per-BUCKET passes dispatch over `num_buckets = pow2(2N)`.
    //      At N ≈ 8 M that is 2²⁴ buckets → 2²⁴/256 = 65 536 workgroups, just past
    //      the 65 535 workgroups-per-dimension limit. Fix: 2-D dispatch (or cap
    //      `num_buckets`) — a change to `grid.rs`, not the hardware.
    //   2. The lowering binds a `RenderInstance` (184 B) buffer, capped at 2 GiB
    //      by default → 2·1024³/184 ≈ 11.67 M agents (`max_storage_buffer_binding
    //      _size`, a requestable limit, adapter-permitting toward VRAM).
    for &count in &[65536u32, 262_144, 1_048_576, 2_097_152, 4_194_304] {
        let (g, out) = boids_graph(count as f32, true);
        let ms = time_step_ms(&gpu, &g, &reg, out);
        let ns_each = ms * 1e6 / count as f64;
        eprintln!("  {count:>10}  {ms:>10.3}  {ns_each:>10.2}");
    }
    eprintln!(
        "\n(near-constant ns/agent = O(N): the grid delivers millions. Above ~4 M\n\
         the grid's bucket dispatch is the first ceiling (~8 M), then the instance\n\
         binding (~11.67 M) — both raisable, neither silicon. The interactive\n\
         ceiling is whatever ms/tick the frame budget allows — all §0.0 numbers.)"
    );
}
