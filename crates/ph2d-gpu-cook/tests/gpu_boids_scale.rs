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
    // For the orbiting-target measurements (`boids_graph_orbit`). ⚠️ `add_node`
    // records a type NAME without checking it; an unregistered type surfaces
    // only as a plan-time refusal, which reads as "the kernel receded" — it was
    // a missing registration, diagnosed by planning a bare LFO as the sink.
    ph2d_node_value_lfo::register(&mut reg).unwrap();
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

/// A boids graph with the FULL tuning exposed, plus the `pre` self-loop — the
/// exact shape of `PH2D_GPU_COOK_DEMO=7`, so the measurement below can sweep the
/// forces that decide how tight the flock packs.
#[allow(clippy::too_many_arguments)]
fn boids_graph_tuned(
    count: f32,
    radius: f32,
    separation: f32,
    alignment: f32,
    cohesion: f32,
    seek: f32,
    max_speed: f32,
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", count);
    g.set_param(boids, "spread", 1.0);
    g.set_param(boids, "seed", 7.0);
    g.set_param(boids, "radius", radius);
    g.set_param(boids, "separation", separation);
    g.set_param(boids, "alignment", alignment);
    g.set_param(boids, "cohesion", cohesion);
    g.set_param(boids, "seek", seek);
    g.set_param(boids, "max_speed", max_speed);
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

/// **Does the flock stutter as it GATHERS?** (Enio's `=7` report, 2026-07-19) — the
/// boids grid is `O(N)` only while density is bounded (Fase 4). `time_step_ms`
/// above times the FIRST few ticks, when the seed cloud is still spread; this runs
/// the sim FORWARD and reports ms/tick in windows, so the climb (or its absence) is
/// visible. Then it sweeps the two forces that set the packing — a stronger
/// `separation` holds the spacing near `radius` (⇒ ~O(1) per cell), a weaker `seek`
/// stops pulling everyone into one clump.
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn does_the_flock_stutter_as_it_gathers() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping boids-gather measurement");
        return;
    };
    let reg = registry();
    // 262 144, not a million: the same density LAW, fast enough to run 600 ticks
    // per candidate. The absolute budget is confirmed at 1 M on the winner below.
    const N: f32 = 262_144.0;
    const TICKS: u64 = 600;
    const WIN: u64 = 120;

    // (label, radius, separation, alignment, cohesion, seek, max_speed)
    let cands = [
        ("shipped  ", 2.0, 1.6, 1.4, 0.6, 0.35, 5.0),
        ("sep 2.4  ", 2.0, 2.4, 1.4, 0.5, 0.20, 5.0),
        ("sep 3.0  ", 2.0, 3.0, 1.2, 0.4, 0.12, 5.0),
    ];

    eprintln!("\nboids ms/tick as the flock evolves ({N} agents, windows of {WIN} ticks):");
    eprint!("  {:<10}", "tuning");
    for w in 0..(TICKS / WIN) {
        eprint!("  t{:>3}-{:<3}", w * WIN, (w + 1) * WIN);
    }
    eprintln!("   {:>8}", "peak");

    for (label, r, sep, al, co, sk, ms) in cands {
        let (g, out) = boids_graph_tuned(N, r, sep, al, co, sk, ms);
        let plan = plan(&g, &reg, &reg, out);
        assert!(plan.is_fully_gpu());
        let mut gc = GpuCook::new();
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        cook(&mut gc, 0); // warm pipelines + seed
        eprint!("  {label:<10}");
        let mut peak = 0.0f64;
        for w in 0..(TICKS / WIN) {
            let start = Instant::now();
            for t in (w * WIN + 1)..=((w + 1) * WIN) {
                cook(&mut gc, t);
            }
            let ms = start.elapsed().as_secs_f64() * 1e3 / WIN as f64;
            peak = peak.max(ms);
            eprint!("  {ms:>8.2}");
        }
        eprintln!("   {peak:>8.2}");
    }
    eprintln!(
        "\n(a rising row = the flock packing tighter and dragging the grid toward\n\
         O(N²); a flat row = the packing held open. The peak is what the frame\n\
         budget must absorb.)"
    );
}

/// **What count leaves headroom for the flock to GATHER?** — the `=7` fix, the same
/// question the `=8` sizing answered. The million was sized for the SPREAD cost;
/// this measures the CLUSTERED peak (600 ticks in) per count, so the demo can be
/// sized to hold 60 fps while doing its headline move.
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn what_boid_count_leaves_headroom() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping boids headroom measurement");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 600;
    eprintln!("\nboids clustered peak vs count (shipped forces, 600 ticks in):");
    eprintln!(
        "  {:>10}  {:>10}  {:>10}  {:>10}",
        "agents", "start ms", "peak ms", "% of 16.7"
    );
    for &n in &[262_144u32, 524_288, 786_432, 1_048_576] {
        let (g, out) = boids_graph_tuned(n as f32, 2.0, 1.6, 1.4, 0.6, 0.35, 5.0);
        let plan = plan(&g, &reg, &reg, out);
        assert!(plan.is_fully_gpu());
        let mut gc = GpuCook::new();
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        cook(&mut gc, 0);
        // start cost (first 30 ticks) vs clustered cost (30 ticks at the end)
        let time = |gc: &mut GpuCook, a: u64, b: u64| {
            let start = Instant::now();
            for t in a..b {
                cook(gc, t);
            }
            start.elapsed().as_secs_f64() * 1e3 / (b - a) as f64
        };
        let start_ms = time(&mut gc, 1, 31);
        for t in 31..(TICKS - 30) {
            cook(&mut gc, t);
        }
        let peak_ms = time(&mut gc, TICKS - 30, TICKS);
        eprintln!(
            "  {n:>10}  {start_ms:>10.2}  {peak_ms:>10.2}  {:>9.0}%",
            peak_ms / 16.67 * 100.0
        );
    }
    eprintln!(
        "\n(the demo must hold 60 fps at its PEAK, not at rest; the % is the cook\n\
         alone, before the render draws a quad per agent.)"
    );
}

/// **Where does the gather SETTLE?** — the 600-tick "peak" above was read off a
/// curve that was still CLIMBING (2.65 → 4.16 with no plateau), i.e. the fixture
/// did not contain the phenomenon: the fully collapsed flock. Enio's second report
/// ("fine until half gathered, then a severe drop") is the part of the curve the
/// window never reached. This runs to equilibrium (4800 ticks = 80 s of sim) and
/// sweeps the force that drives the collapse — `seek` is a GLOBAL attractor, so
/// with it on, the steady state is one dense ball around the target no matter the
/// count; separation only sets how dense.
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn where_does_the_flock_settle() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping settle measurement");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 9600;
    const WIN: u64 = 960;

    // (label, count, radius, separation, alignment, cohesion, seek, max_speed)
    let cands = [
        ("1M seek0 r2.0", 1_048_576.0, 2.0, 2.4, 1.4, 0.6, 0.0, 5.0),
        ("1M seek0 r1.5", 1_048_576.0, 1.5, 2.4, 1.4, 0.6, 0.0, 5.0),
        (
            "1M sk.005 r1.5",
            1_048_576.0,
            1.5,
            4.0,
            1.4,
            0.3,
            0.005,
            5.0,
        ),
    ];
    eprintln!("\nboids ms/tick to equilibrium (windows of {WIN} ticks):");
    for (label, n, r, sep, al, co, sk, ms) in cands {
        let (g, out) = boids_graph_tuned(n, r, sep, al, co, sk, ms);
        let plan = plan(&g, &reg, &reg, out);
        assert!(plan.is_fully_gpu());
        let mut gc = GpuCook::new();
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        cook(&mut gc, 0);
        eprint!("  {label}");
        for w in 0..(TICKS / WIN) {
            let start = Instant::now();
            for t in (w * WIN + 1)..=((w + 1) * WIN) {
                cook(&mut gc, t);
            }
            eprint!(
                "  {:>7.2}",
                start.elapsed().as_secs_f64() * 1e3 / WIN as f64
            );
        }
        eprintln!();
    }
    eprintln!("(a row that keeps climbing = collapse into one ball; a plateau = bounded density.)");
}

/// Boids with the target ORBITING (two phase-shifted LFOs into `target_x/y`) —
/// the classic fix for the static-attractor collapse: a flock that must keep
/// chasing never parks into the dense ball. All on the device (the LFO lowers).
fn boids_graph_orbit(count: f32, seek: f32, orbit_r: f32, period: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", count);
    g.set_param(boids, "spread", 1.0);
    g.set_param(boids, "seed", 7.0);
    g.set_param(boids, "radius", 2.0);
    g.set_param(boids, "separation", 1.6);
    g.set_param(boids, "alignment", 1.4);
    g.set_param(boids, "cohesion", 0.6);
    g.set_param(boids, "seek", seek);
    g.set_param(boids, "max_speed", 5.0);
    let lx = g.add_node("value.lfo");
    g.set_param(lx, "period", period);
    g.set_param(lx, "amplitude", orbit_r);
    let ly = g.add_node("value.lfo");
    g.set_param(ly, "period", period);
    g.set_param(ly, "amplitude", orbit_r);
    g.set_param(ly, "phase", 0.25);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(lx, boids, 0), (ly, boids, 1)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
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

/// The orbit hypothesis, measured to equilibrium: does a MOVING target bound the
/// density that a static one collapses (28.5 ms plateau at 262 k, above)?
#[test]
#[ignore = "measurement, needs a GPU adapter"]
fn does_an_orbiting_target_bound_the_gather() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping orbit measurement");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 4800;
    const WIN: u64 = 480;
    // (label, count, seek, orbit radius, period seconds)
    let cands = [
        ("262k r60 p20 ", 262_144.0, 0.35, 60.0, 20.0),
        ("262k r120 p30", 262_144.0, 0.35, 120.0, 30.0),
        ("524k r120 p30", 524_288.0, 0.35, 120.0, 30.0),
    ];
    eprintln!("\nboids ms/tick to equilibrium with an ORBITING target:");
    for (label, n, sk, orb, per) in cands {
        let (g, out) = boids_graph_orbit(n, sk, orb, per);
        let plan = plan(&g, &reg, &reg, out);
        assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
        let mut gc = GpuCook::new();
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        cook(&mut gc, 0);
        eprint!("  {label}");
        for w in 0..(TICKS / WIN) {
            let start = Instant::now();
            for t in (w * WIN + 1)..=((w + 1) * WIN) {
                cook(&mut gc, t);
            }
            eprint!(
                "  {:>7.2}",
                start.elapsed().as_secs_f64() * 1e3 / WIN as f64
            );
        }
        eprintln!();
    }
}
