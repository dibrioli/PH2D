//! GPU-vs-CPU parity for **`motion.voronoi` via jump flooding** (ADR-0139).
//!
//! The CPU `eval` (linear exact nearest) is canonical; the device runs JFA +
//! integer centroids. The two paths honestly differ in two measured ways
//! (`voronoi.rs` module docs): rare boundary texels (JFA's known error class)
//! and seed collisions (two points in one texel hide one for a round). So the
//! gate structure follows ADR-0127 D4 — a sequential system gates **one
//! step** tightly; the full trajectory gets a measured band:
//!
//! - the assignment oracle compares the flooded grid against the exact
//!   nearest, texel by texel, on a collision-free fixture — divergences must
//!   be near-ties, and few;
//! - `relax = 0` / `iterations = 0` reproduce the raw hashed seed **bit
//!   exactly** (the hash port is an integer avalanche — there is no ε to hide
//!   behind);
//! - one Lloyd step from a collision-free seed matches within a tight
//!   measured ε;
//! - the full 8-iteration relaxation stays within a measured band, collisions
//!   and all — the product's shape, not a tuned-until-green ε.
//!
//!   cargo test -p ph2d-gpu-cook --test gpu_voronoi --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances, voronoi};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::gpu::GpuAlgorithm;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;
use std::time::Instant;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_voronoi::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const W: f32 = 5.0;
const H: f32 = 5.0;

/// The node's own caps and sampling law, read from ITS spec — hardcoding them
/// here would be a second copy that drifts.
fn node_law() -> (usize, usize, usize) {
    let GpuAlgorithm::LloydVoronoi {
        samples_per_point,
        min_res,
        max_res,
        ..
    } = ph2d_node_motion_voronoi::GPU_ALGORITHM;
    (samples_per_point, min_res, max_res)
}

fn res_for(count: usize) -> usize {
    let (spp, lo, hi) = node_law();
    GpuAlgorithm::lloyd_resolution(count, spp, lo, hi)
}

fn voronoi_graph(count: f32, iterations: f32, seed: f32) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let v = g.add_node("motion.voronoi");
    g.set_param(v, "count", count);
    g.set_param(v, "width", W);
    g.set_param(v, "height", H);
    g.set_param(v, "seed", seed);
    g.set_param(v, "iterations", iterations);
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (v, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    (g, v, out)
}

/// The canonical CPU frame (the node is `Pure`; playhead is irrelevant).
fn cpu_frame(g: &Graph, reg: &NodeRegistry, out: NodeId) -> Vec<RenderInstance> {
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

fn gpu_frame(gpu: &GpuContext, g: &Graph, reg: &NodeRegistry, out: NodeId) -> Vec<RenderInstance> {
    let p = plan(g, reg, reg, out);
    assert!(p.is_fully_gpu(), "boundaries: {:?}", p.boundaries);
    let mut gc = GpuCook::new();
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &p,
        &[],
        CookClock::at(0.0),
        DEFAULT_UV,
        DEFAULT_SIZE,
    )
    .expect("gpu cook");
    read_instances(gpu, gc.instances().expect("cooked"))
}

/// Per-point |Δ| stats between the two paths' positions.
fn position_deltas(cpu: &[RenderInstance], gpu: &[RenderInstance]) -> (f32, f32, f32) {
    assert_eq!(cpu.len(), gpu.len(), "instance count");
    let mut ds: Vec<f32> = cpu
        .iter()
        .zip(gpu)
        .map(|(c, g)| {
            let dx = c.world_pos[0] - g.world_pos[0];
            let dy = c.world_pos[1] - g.world_pos[1];
            (dx * dx + dy * dy).sqrt()
        })
        .collect();
    ds.sort_by(f32::total_cmp);
    let mean = ds.iter().sum::<f32>() / ds.len() as f32;
    let p95 = ds[((ds.len() * 95) / 100).min(ds.len() - 1)];
    let max = *ds.last().unwrap();
    (mean, p95, max)
}

/// The exact nearest-point owner of each texel centre — the CPU node's
/// assignment rule, restated (strict `<` keeps the FIRST, so the lower id
/// wins an exact tie).
fn cpu_assignment(points: &[[f32; 2]], w: f32, h: f32, res: usize) -> Vec<u32> {
    (0..res * res)
        .map(|s| {
            let (gy, gx) = (s / res, s % res);
            let tc = [
                ((gx as f32 + 0.5) / res as f32 - 0.5) * w,
                ((gy as f32 + 0.5) / res as f32 - 0.5) * h,
            ];
            let mut best = 0u32;
            let mut best_d = f32::MAX;
            for (j, p) in points.iter().enumerate() {
                let (dx, dy) = (tc[0] - p[0], tc[1] - p[1]);
                let d = dx * dx + dy * dy;
                if d < best_d {
                    best_d = d;
                    best = j as u32;
                }
            }
            best
        })
        .collect()
}

/// The texel each point falls in — the collision precondition for the
/// collision-free fixtures (a fixture must contain ONLY the phenomenon under
/// test; a seed collision is a different, documented mechanism).
fn cells_of(points: &[[f32; 2]], w: f32, h: f32, res: usize) -> Vec<usize> {
    points
        .iter()
        .map(|p| {
            let gx = (((p[0] / w + 0.5) * res as f32).floor() as isize).clamp(0, res as isize - 1);
            let gy = (((p[1] / h + 0.5) * res as f32).floor() as isize).clamp(0, res as isize - 1);
            gy as usize * res + gx as usize
        })
        .collect()
}

fn has_collision(points: &[[f32; 2]], w: f32, h: f32, res: usize) -> bool {
    let mut cells = cells_of(points, w, h, res);
    cells.sort_unstable();
    cells.windows(2).any(|c| c[0] == c[1])
}

/// A seed whose raw cloud (from the canonical CPU at `iterations = 0`) puts
/// every point in its own texel — plus that cloud. Panics if 200 seeds all
/// collide, which at these densities cannot happen by chance.
fn collision_free_cloud(reg: &NodeRegistry, count: usize) -> (f32, Vec<[f32; 2]>) {
    let res = res_for(count);
    for seed in 1..=200 {
        let (g, _, out) = voronoi_graph(count as f32, 0.0, seed as f32);
        let pts: Vec<[f32; 2]> = cpu_frame(&g, reg, out)
            .iter()
            .map(|i| i.world_pos)
            .collect();
        if !has_collision(&pts, W, H, res) {
            return (seed as f32, pts);
        }
    }
    panic!("no collision-free seed in 200 tries at count {count}");
}

/// The node's `MAX_RES` and the engine's integer-centroid ceiling are two
/// copies of one number on opposite sides of a dependency edge (the node
/// cannot import the engine's const) — pinned together here so neither
/// drifts past the other; and the point cap must keep the sampling law
/// intact (`count·samples ≤ res²`), which is the cap's whole justification.
#[test]
fn the_node_cap_respects_the_integer_centroid_ceiling() {
    let (spp, _, max_res) = node_law();
    assert!(
        max_res <= voronoi::INT_CENTROID_RES_CEILING,
        "node max_res {max_res} exceeds the u32-centroid bound"
    );
    let GpuAlgorithm::LloydVoronoi { max_points, .. } = ph2d_node_motion_voronoi::GPU_ALGORITHM;
    assert!(
        max_points * spp <= max_res * max_res,
        "the sampling law degrades before the cap ({max_points}·{spp} > {max_res}²)"
    );
}

/// The plan claims the whole chain (CPU-only — the plan is pure), including
/// an LFO wired into relax: nothing recedes, no boundary.
#[test]
fn the_plan_claims_the_voronoi_chain() {
    let reg = registry();
    let (mut g, v, out) = voronoi_graph(96.0, 8.0, 1.0);
    let lfo = g.add_node("value.lfo");
    g.connect(Edge {
        from: (lfo, 0),
        to: (v, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(&reg).expect("well-typed");
    let p = plan(&g, &reg, &reg, out);
    assert!(p.is_fully_gpu(), "boundaries: {:?}", p.boundaries);
    assert!(
        p.stages.iter().any(|s| s.node == v),
        "the voronoi must be a claimed stage"
    );
}

/// Two points mirrored about x = 0, an odd `res` so the middle column's texel
/// centres sit at exactly x = 0: both distances are bit-equal, and the tie
/// must go to the LOWER id — the CPU nearest's keep-first on strict `<`.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn an_exact_tie_prefers_the_lower_id() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let points = vec![[-1.0f32, 0.0], [1.0, 0.0]];
    let res = 9;
    let owners = voronoi::jfa_assignment(&gpu, &points, 4.0, 4.0, res);
    for gy in 0..res {
        let mid = owners[gy * res + res / 2];
        assert_eq!(mid, 0, "middle column row {gy}: tie must prefer id 0");
        assert_eq!(owners[gy * res], 0, "left edge row {gy}");
        assert_eq!(owners[gy * res + res - 1], 1, "right edge row {gy}");
    }
}

/// The flooded grid against the exact nearest, texel by texel, on a
/// collision-free cloud. JFA's known error class is a texel whose two nearest
/// owners are nearly equidistant — so every divergence must BE a near-tie
/// (distance ratio ≈ 1), and there must be few of them.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_jfa_assignment_agrees_with_the_linear_nearest() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // 40 and 96 — at 300 the birthday arithmetic makes a collision-free seed
    // a 1-in-10⁴ draw (expected ≈ 9 collisions in 4 900 texels), and the
    // collision mechanism is a DIFFERENT, documented divergence this oracle
    // must not contain. Density does not change JFA's error class.
    for count in [40usize, 96] {
        let res = res_for(count);
        let (seed, pts) = collision_free_cloud(&reg, count);
        let gpu_own = voronoi::jfa_assignment(&gpu, &pts, W, H, res);
        let cpu_own = cpu_assignment(&pts, W, H, res);
        let mut divergent = 0usize;
        let mut worst_ratio = 1.0f32;
        for s in 0..res * res {
            if gpu_own[s] == cpu_own[s] {
                continue;
            }
            divergent += 1;
            let (gy, gx) = (s / res, s % res);
            let tc = [
                ((gx as f32 + 0.5) / res as f32 - 0.5) * W,
                ((gy as f32 + 0.5) / res as f32 - 0.5) * H,
            ];
            let d = |p: [f32; 2]| {
                let (dx, dy) = (tc[0] - p[0], tc[1] - p[1]);
                (dx * dx + dy * dy).sqrt()
            };
            assert!(
                (gpu_own[s] as usize) < pts.len(),
                "count {count}: unowned texel {s} after the flood"
            );
            let ratio = d(pts[gpu_own[s] as usize]) / d(pts[cpu_own[s] as usize]).max(1e-9);
            worst_ratio = worst_ratio.max(ratio);
            assert!(
                ratio <= 1.05,
                "count {count} texel {s}: divergent owner is not a near-tie (ratio {ratio})"
            );
        }
        let frac = divergent as f32 / (res * res) as f32;
        eprintln!(
            "count {count} res {res} seed {seed}: {divergent}/{} divergent ({frac:.5}), worst ratio {worst_ratio:.4}",
            res * res
        );
        assert!(
            frac <= 0.002,
            "count {count}: {frac} of texels diverge — beyond JFA's error class"
        );
    }
}

/// `iterations = 0` (so relaxed == raw regardless of relax) must reproduce
/// the CPU's hashed cloud **bit-exactly**: the hash is an integer avalanche
/// ported instruction for instruction, and `raw + (raw − raw)·t` is exact.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn iteration_zero_reproduces_the_raw_seed_bit_exact() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (count, seed) in [(24.0, 1.0), (96.0, 7.0), (600.0, 3.0)] {
        let (g, _, out) = voronoi_graph(count, 0.0, seed);
        g.validate(&reg).expect("well-typed");
        let cpu = cpu_frame(&g, &reg, out);
        let dev = gpu_frame(&gpu, &g, &reg, out);
        assert_eq!(cpu.len(), dev.len());
        for (i, (c, d)) in cpu.iter().zip(&dev).enumerate() {
            assert_eq!(
                c.world_pos, d.world_pos,
                "count {count} seed {seed} point {i}: the raw seed must be bit-exact"
            );
        }
        eprintln!("count {count} seed {seed}: {} points bit-exact", cpu.len());
    }
}

/// One Lloyd step from a collision-free seed (ADR-0127 D4: a sequential
/// system gates ONE step) — tight, measured ε. What remains inside it: the
/// CPU sums sample positions in `f32` sequentially while the device averages
/// exact integer indices, and JFA's rare boundary texels shift a centroid by
/// `O(texel/cell)`.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn one_lloyd_step_matches_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for count in [24usize, 96] {
        let (seed, _) = collision_free_cloud(&reg, count);
        let (g, _, out) = voronoi_graph(count as f32, 1.0, seed);
        g.validate(&reg).expect("well-typed");
        let cpu = cpu_frame(&g, &reg, out);
        let dev = gpu_frame(&gpu, &g, &reg, out);
        let (mean, p95, max) = position_deltas(&cpu, &dev);
        eprintln!("count {count} seed {seed}: one step Δ mean {mean:.6} p95 {p95:.6} max {max:.6}");
        // Measured on the RTX: mean 0.000000, max 0.000001 — the JFA is
        // texel-exact here and the centroid arithmetic difference is sub-µm.
        assert!(mean <= 1e-5, "count {count}: one-step mean Δ {mean}");
        assert!(max <= 1e-4, "count {count}: one-step max Δ {max}");
    }
}

/// The product's answer: the full default relaxation (8 iterations), seed
/// collisions and all, across the count range. The band is measured, not
/// tuned — it is where the two documented mechanisms land, printed so the
/// numbers stay inspectable.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_full_relaxation_stays_within_the_measured_band() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for count in [96.0, 300.0, 600.0] {
        let (g, _, out) = voronoi_graph(count, 8.0, 1.0);
        g.validate(&reg).expect("well-typed");
        let cpu = cpu_frame(&g, &reg, out);
        let dev = gpu_frame(&gpu, &g, &reg, out);
        let (mean, p95, max) = position_deltas(&cpu, &dev);
        eprintln!("count {count}: full relax Δ mean {mean:.5} p95 {p95:.5} max {max:.5}");
        // Measured on the RTX (2026-07-21), domain 5×5:
        //   count 96:  mean 0.01177  p95 0.07777  max 0.27532
        //   count 300: mean 0.02257  p95 0.08644  max 0.33968
        //   count 600: mean 0.01540  p95 0.08007  max 0.25505
        // — the seed-collision mechanism compounding over 8 iterations (the
        // one-step gate proves the step itself is exact to 1e-6).
        assert!(mean <= 0.04, "count {count}: full-relax mean Δ {mean}");
        assert!(p95 <= 0.15, "count {count}: full-relax p95 Δ {p95}");
        assert!(max <= 0.55, "count {count}: full-relax max Δ {max}");
    }
}

/// An animated relax rides the device: an LFO pinned to a constant (amplitude
/// 0, offset 0.5) wired into the relax port must land BETWEEN the raw seed
/// and the full CVT, and match the CPU's own half-relaxed answer — proving
/// the port's row 0 is read on the device, not defaulted.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn an_animated_relax_rides_row_0_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let build = |offset: f32| {
        let (mut g, v, out) = voronoi_graph(96.0, 8.0, 1.0);
        let lfo = g.add_node("value.lfo");
        g.set_param(lfo, "amplitude", 0.0);
        g.set_param(lfo, "offset", offset);
        g.connect(Edge {
            from: (lfo, 0),
            to: (v, 0),
            delayed: false,
        })
        .unwrap();
        g.validate(&reg).expect("well-typed");
        (g, out)
    };
    let (g_half, out_half) = build(0.5);
    let cpu = cpu_frame(&g_half, &reg, out_half);
    let dev = gpu_frame(&gpu, &g_half, &reg, out_half);
    let (mean, p95, max) = position_deltas(&cpu, &dev);
    eprintln!("relax 0.5: Δ mean {mean:.5} p95 {p95:.5} max {max:.5}");
    assert!(mean <= 0.01 && max <= 0.5, "half-relax parity band");

    // And t actually matters on the device: 0.5 is neither 0 nor 1.
    let (g_raw, out_raw) = build(0.0);
    let raw = gpu_frame(&gpu, &g_raw, &reg, out_raw);
    let moved = dev
        .iter()
        .zip(&raw)
        .filter(|(h, r)| h.world_pos != r.world_pos)
        .count();
    assert!(
        moved > dev.len() / 2,
        "half-relax must move most points off the raw seed ({moved}/{})",
        dev.len()
    );
}

/// A registry whose voronoi spec carries LIFTED caps — last-write-wins on the
/// side channel, so the probe can measure beyond the node's shipped numbers
/// without touching the node (the CPU `eval` is not run here).
fn lifted_registry(max_points: usize, max_res: usize) -> NodeRegistry {
    let mut reg = registry();
    let GpuAlgorithm::LloydVoronoi {
        count_param,
        width_param,
        height_param,
        seed_param,
        iterations_param,
        relax_port,
        samples_per_point,
        min_res,
        max_iterations,
        ..
    } = ph2d_node_motion_voronoi::GPU_ALGORITHM;
    reg.register_gpu_algorithm(
        ph2d_node_motion_voronoi::MANIFEST.id,
        GpuAlgorithm::LloydVoronoi {
            count_param,
            width_param,
            height_param,
            seed_param,
            iterations_param,
            relax_port,
            max_points,
            min_res,
            max_res,
            samples_per_point,
            max_iterations,
        },
    );
    reg
}

/// MEASUREMENT (ADR-0139 §5 — the caps fall to device numbers): ms/frame of
/// the full default relaxation (8 iterations) as count and the res law climb.
/// `1625` is [`voronoi::INT_CENTROID_RES_CEILING`] (the u32-centroid bound);
/// the law un-clamps at `count = res²/16`, so past ~165k the grid saturates
/// and the cost goes flat — the JFA is count-independent by construction.
///
///   cargo test -p ph2d-gpu-cook --test gpu_voronoi --release -- --ignored --nocapture how_far
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn how_far_does_the_lloyd_scale() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    const WARM: usize = 3;
    const TIMED: usize = 8;
    eprintln!("\nLloyd/JFA scale (8 iterations, ms/frame):");
    for (count, cap) in [
        (600usize, 96usize), // the shipped CPU-era caps, for the before/after
        (600, 1625),
        (2_000, 1625),
        (10_000, 1625),
        (20_000, 1625), // the shell's ready-to-smoke scene (PH2D_GPU_COOK_DEMO=11)
        (50_000, 1625),
        (165_000, 1625), // the shipped cap — the sampling law's last intact count
        (200_000, 1625),
        (1_000_000, 1625),
    ] {
        let reg = lifted_registry(2_000_000, cap);
        let (spp, lo, _) = node_law();
        let res = GpuAlgorithm::lloyd_resolution(count, spp, lo, cap);
        let (g, _, out) = voronoi_graph(count as f32, 8.0, 1.0);
        let p = plan(&g, &reg, &reg, out);
        assert!(p.is_fully_gpu(), "boundaries: {:?}", p.boundaries);
        let mut gc = GpuCook::new();
        let cook = |gc: &mut GpuCook| {
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &p,
                &[],
                CookClock::at(0.0),
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        for _ in 0..WARM {
            cook(&mut gc);
        }
        let start = Instant::now();
        for _ in 0..TIMED {
            cook(&mut gc);
        }
        let ms = start.elapsed().as_secs_f64() * 1e3 / TIMED as f64;
        eprintln!("  count {count:>9} res {res:>4}: {ms:>8.2} ms/frame");
    }
}
