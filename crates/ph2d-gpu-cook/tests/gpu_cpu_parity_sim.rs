//! GPU-vs-CPU parity for the **simulation loop** (GPU/M5 Fase 3, ADR-0127) —
//! THE audit of the slice ("o gate de paridade É o audit").
//!
//! ## Why this gate is shaped differently from Fase 2's
//!
//! A stateless kernel is `f(params, playhead)`: every frame is independent, so
//! its ε is bounded per frame and the Fase 2 gates compare no trajectory at all.
//! A sim is `x_{n+1} = f(x_n)` — ε feeds back, and after N ticks the GPU and the
//! CPU are legitimately **different animations** (ADR-0127 D4; the CPU stays the
//! canonical path, ADR-0126). So this compares a **seeded state plus one step**,
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
    ph2d_node_force_buoyancy::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_spring::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    // The uncoverable subject for the refusal control (a global permutation).
    ph2d_node_motion_sort::register(&mut reg).unwrap();
    // GPU/M5 (ADR-0135) — the sim-zone family: the state loop, its own
    // integrator (reads a per-element clock column) and its static collider.
    ph2d_node_sim_zone::register(&mut reg).unwrap();
    ph2d_node_sim_step::register(&mut reg).unwrap();
    ph2d_node_sim_collide::register(&mut reg).unwrap();
    // Render transforms the demo=10 scene uses (lift + shrink).
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_scale::register(&mut reg).unwrap();
    reg
}

/// `grid → oscillator [→ spring] → output` — the spring on its own `pre`
/// self-loop (`out --pre--> state`, the sequential-node convention).
///
/// The oscillator is not scenery: `motion.spring` chases a target and its own
/// doc opens with the warning that it **only acts on targets that CHANGE**. A
/// spring behind a static grid is a pass-through, and every assertion below
/// would hold with the node deleted.
///
/// ⚠️ **The oscillator drives the SAME `channel` the spring springs on**, which
/// is the whole reason this takes a channel at all. A spring on Rotation behind
/// an oscillator on Y has a target of `rot = 0` forever: it settles instantly,
/// is a pass-through, and both the lag oracle and the parity comparison would be
/// vacuous in the exact way this helper's doc warns about — the moving field
/// would just be a *different* field than the one under test.
///
/// The Size channel takes an `offset` because its identity is unit scale and its
/// amplitude would otherwise sweep the sprite through zero and negative every
/// cycle. That is a fixture concern, not a physics one: a negative size is a
/// legal number for the solver and a meaningless one to compare renders of.
fn spring_chain(
    reg: &NodeRegistry,
    with_spring: bool,
    tension: f32,
    channel: f32,
) -> (Graph, NodeId, Option<NodeId>) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 80.0);
    g.set_param(grid, "cols", 80.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", channel);
    // Rotation is in DEGREES, so 3.0 would be a wobble the basis barely records;
    // Size is centred on 4 so the sweep stays positive (see the note above).
    let (amp, offset) = match channel as i32 {
        2 => (30.0, 0.0),
        3 => (3.0, 4.0),
        _ => (3.0, 0.0),
    };
    g.set_param(osc, "amplitude", amp);
    g.set_param(osc, "offset", offset);
    g.set_param(osc, "frequency", 2.0);
    let out = g.add_node("motion.output");
    edge(&mut g, grid, osc, 0, false);
    let mut spring = None;
    if with_spring {
        let sp = g.add_node("motion.spring");
        spring = Some(sp);
        g.set_param(sp, "channel", channel);
        g.set_param(sp, "tension", tension);
        g.set_param(sp, "friction", 1.5);
        edge(&mut g, osc, sp, 0, false);
        // The feedback port: the node's own previous output.
        edge(&mut g, sp, sp, 1, true);
        edge(&mut g, sp, out, 0, false);
    } else {
        edge(&mut g, osc, out, 0, false);
    }
    g.validate(reg).expect("well-typed");
    (g, out, spring)
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

/// A fixed-population **simulation zone** (ADR-0135): a grid seeds the
/// population once through `zone.init`, and the interior — `wind → sim.step →
/// sim.collide` on the zone's `pre` state loop — falls it under gravity and
/// bounces it off a static collider. No birth/death, so the whole loop is
/// coverable and runs 100% on the device.
///
/// ```text
///   grid ──> zone.init                 zone.out ──> output
///            zone.out ⊙──pre──> wind → sim.step → sim.collide ──> zone.state
/// ```
///
/// The `pre` edge is `zone.out --pre--> wind` and the forward tail is
/// `sim.collide → zone.state`, exactly the topology the editor's plumbing wires
/// (and `sim.zone`'s own tests use). With `sea`, a `force.buoyancy` is spliced
/// between the wind and the step — the extra force accumulating into the same
/// `accel` the step consumes, which is the demo's snow-into-the-sea physics.
/// 40×40 = 1600 instances.
fn zone_chain(
    reg: &NodeRegistry,
    collide: &[(&str, f32)],
    sea: Option<&[(&str, f32)]>,
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 40.0);
    g.set_param(grid, "cols", 40.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    // Gravity: 270 deg = straight down (y-up), strong enough that the fall clears
    // the ε floor in a few ticks and reaches the collider.
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", 8.0);
    let step = g.add_node("sim.step");
    let ground = g.add_node("sim.collide");
    for (k, v) in collide {
        g.set_param(ground, *k, *v);
    }
    let out = g.add_node("motion.output");
    edge(&mut g, grid, zone, 0, false); // grid → zone.init
    edge(&mut g, zone, out, 0, false); // zone.out → output (render)
    edge(&mut g, zone, wind, 0, true); // zone.out --pre--> wind (the state entry)
    // The interior: wind [→ buoyancy] → sim.step → sim.collide → zone.state.
    let last_force = match sea {
        Some(params) => {
            let sea = g.add_node("force.buoyancy");
            for (k, v) in params {
                g.set_param(sea, *k, *v);
            }
            edge(&mut g, wind, sea, 0, false);
            sea
        }
        None => wind,
    };
    edge(&mut g, last_force, step, 0, false);
    edge(&mut g, step, ground, 0, false);
    edge(&mut g, ground, zone, 1, false); // interior tail → zone.state
    g.validate(reg).expect("well-typed");
    (g, out)
}

fn assert_close(what: &str, i: usize, a: f32, b: f32, eps: f32) {
    assert!(
        (a - b).abs() <= eps,
        "instance {i} field {what}: cpu {a} vs gpu {b} (|diff| {} > eps {eps})",
        (a - b).abs()
    );
}

/// The size budget for a chain where **nothing drives `size`** — which was every
/// gate in this file until the spring gained its channel variants.
///
/// It is not a calibrated tolerance, it is a bit-identity check wearing one:
/// measured across all 25 undriven gates, `max |dsize|` is `0e0` on every single
/// one, because `size` is the `DEFAULT_SIZE` uniform copied down both paths. Kept
/// tight on purpose — a driven field's budget must not be spent where the answer
/// is a constant ([[feedback_a_ratio_between_two_sick_channels_is_green_by_construction]]).
const EPS_SIZE_UNDRIVEN: f32 = 1e-5;

fn assert_parity(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance]) {
    assert_parity_sized(label, cpu, gpu, EPS_SIZE_UNDRIVEN);
}

/// As [`assert_parity`], with an explicit budget for `size`.
///
/// A chain that actually DRIVES size wants [`EPS_POS`], the same budget this file
/// gives a driven position, and for the same reason: both are a world-space
/// length coming out of the same solver. Measured on the spring, whose Size and
/// Y channels run identical arithmetic — position at tension 60 diverges
/// `2.17e-4` and passes under `2e-3`; size diverges `3.58e-5`, six times less.
/// Holding size to `1e-5` there would be pricing one channel 200× tighter than
/// its twin for no reason other than that the old fixture never moved it.
fn assert_parity_sized(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance], eps_size: f32) {
    assert_eq!(cpu.len(), gpu.len(), "instance count");
    let mut max_pos = 0.0f32;
    let mut max_size = 0.0f32;
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            assert_close("world_pos", i, c.world_pos[k], g.world_pos[k], EPS_POS);
            assert_close("size", i, c.size[k], g.size[k], eps_size);
            max_pos = max_pos.max((c.world_pos[k] - g.world_pos[k]).abs());
            max_size = max_size.max((c.size[k] - g.size[k]).abs());
        }
        for k in 0..4 {
            assert_close("tint", i, c.tint[k], g.tint[k], 1e-6);
            assert_close("basis", i, c.basis[k], g.basis[k], 1e-4);
        }
    }
    eprintln!(
        "{label}: {} instances, max |dpos| = {max_pos:e}, max |dsize| = {max_size:e}",
        cpu.len()
    );
}

/// The largest per-element move between two lowerings of the same field.
fn max_move(a: &[RenderInstance], b: &[RenderInstance]) -> f32 {
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| (0..2).map(move |k| (x.world_pos[k] - y.world_pos[k]).abs()))
        .fold(0.0f32, f32::max)
}

/// The largest move on the RENDER field a transform `channel` lands in — the lag
/// oracle for a spring that is not on X/Y.
///
/// [`max_move`] reads `world_pos`, which is the right question for two thirds of
/// the channels and blind for the rest: a spring on Rotation could be compiled to
/// a pass-through and `world_pos` would not budge either way, so the "it must
/// LAG" guard that makes the parity comparison mean anything would pass on a
/// field nothing is driving ([[reference_topic_oracle_discipline]] — ask the
/// question of the field the node actually writes).
fn max_channel_move(a: &[RenderInstance], b: &[RenderInstance], channel: f32) -> f32 {
    let pick = |r: &RenderInstance| -> Vec<f32> {
        match channel as i32 {
            2 => r.basis.to_vec(),
            3 => r.size.to_vec(),
            _ => r.world_pos.to_vec(),
        }
    };
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| {
            pick(x)
                .into_iter()
                .zip(pick(y))
                .map(|(u, v)| (u - v).abs())
                .collect::<Vec<_>>()
        })
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

/// DIAGNOSTIC (`PH2D_GPU_COOK_DEMO=10` FPS report): build the demo document at
/// its shipped scale and MEASURE the GPU cook, the CPU cook, and whether the
/// field stays bounded. Run:
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_sim --release -- \
///     --ignored --nocapture --test-threads=1 the_zone_demo_scale_cook_cost
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_zone_demo_scale_cook_cost() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // The demo, verbatim: grid 256x1024 → move → zone, interior wind → buoyancy →
    // sim.step → sim.collide, render → scale → output.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 256.0);
    g.set_param(grid, "cols", 1024.0);
    g.set_param(grid, "gap_x", 0.03);
    g.set_param(grid, "gap_y", 0.05);
    let lift = g.add_node("motion.move");
    g.set_param(lift, "dy", 9.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", 4.0);
    g.set_param(wind, "gust", 0.35);
    let sea = g.add_node("force.buoyancy");
    for (k, v) in [
        ("level", -0.5),
        ("density", 14.0),
        ("depth", 0.3),
        ("drag", 5.0),
        ("wave_amplitude", 0.14),
        ("wave_length", 2.4),
        ("wave_speed", 0.5),
    ] {
        g.set_param(sea, k, v);
    }
    let step = g.add_node("sim.step");
    let bed = g.add_node("sim.collide");
    g.set_param(bed, "shape", 0.0);
    g.set_param(bed, "height", -1.1);
    g.set_param(bed, "restitution", 0.25);
    g.set_param(bed, "friction", 0.35);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.06);
    let out = g.add_node("motion.output");
    edge(&mut g, grid, lift, 0, false);
    edge(&mut g, lift, zone, 0, false);
    edge(&mut g, zone, scale, 0, false);
    edge(&mut g, scale, out, 0, false);
    edge(&mut g, zone, wind, 0, true);
    edge(&mut g, wind, sea, 0, false);
    edge(&mut g, sea, step, 0, false);
    edge(&mut g, step, bed, 0, false);
    edge(&mut g, bed, zone, 1, false);
    g.validate(&reg).expect("well-typed");

    let plan = plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu(), "NOT fully GPU: {:?}", plan.boundaries);
    eprintln!(
        "\n=== demo=10 scale (262144 elements) ===\nfully_gpu={} dispatching={}",
        plan.is_fully_gpu(),
        plan.dispatching_stages(&reg),
    );

    // GPU: warm up 30 ticks (past the fall), then time 60 steady-state ticks.
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
    for t in 0..=200 {
        cook(&mut gc, t);
    }
    let n0 = std::time::Instant::now();
    for t in 201..=260 {
        cook(&mut gc, t);
    }
    let gpu_ms = n0.elapsed().as_secs_f64() * 1000.0 / 60.0;

    // Bounded? A settled snow field should sit within a few units of the sea.
    let frame = read_instances(&gpu, gc.instances().expect("cooked"));
    let (mut maxabs, mut nonfinite) = (0.0f32, 0usize);
    for r in &frame {
        for k in 0..2 {
            if !r.world_pos[k].is_finite() {
                nonfinite += 1;
            } else {
                maxabs = maxabs.max(r.world_pos[k].abs());
            }
        }
    }

    // CPU, one steady-state tick, for the ratio.
    let cpu = cpu_ticks(&g, &reg, out, 205);
    let c0 = std::time::Instant::now();
    let _ = cpu_ticks(&g, &reg, out, 205);
    let cpu_ms = c0.elapsed().as_secs_f64() * 1000.0 / 206.0;
    let _ = cpu;

    eprintln!(
        "GPU cook {gpu_ms:.3} ms/tick · CPU cook ~{cpu_ms:.3} ms/tick · \
         max|pos| {maxabs:.2} · non-finite {nonfinite} of {} lanes",
        frame.len() * 2
    );
    assert_eq!(nonfinite, 0, "the sim exploded to NaN/inf");

    // Replicate the SHELL's exact forward-play loop: each frame cooks the range
    // `rewind_for(target)..=target`. If that is 1 tick/frame the demo is cheap; if
    // it re-cooks the whole history every frame, THAT is the FPS drop (and my
    // direct-cook timing above would never have shown it).
    let mut sc = GpuCook::new();
    let mut total = 0u64;
    let mut worst = 0u64;
    let f0 = std::time::Instant::now();
    for target in 0..=60u64 {
        let lo = sc.rewind_for(target);
        let n = target - lo + 1;
        total += n;
        worst = worst.max(n);
        for t in lo..=target {
            sc.cook(
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
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    let shell_ms = f0.elapsed().as_secs_f64() * 1000.0 / 61.0;
    eprintln!(
        "SHELL loop: {total} cooks over 61 frames ({:.1}/frame, worst {worst}) · \
         {shell_ms:.3} ms/frame",
        total as f64 / 61.0
    );
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
fn one_step_of_buoyancy_matches_the_cpu() {
    // `force.buoyancy` alone — it carries a drag term of its own, so two ticks
    // for the same reason `force.drag` needs them: a fresh seed has no velocity
    // and `−k·v` would be gated against nothing.
    //
    // The fixture is built so that **all three submersion regimes are present**.
    // The 80×80 grid spans y ≈ ±9.9 and the sea sits at 0, so half the field is
    // dry, half is under, and the `depth = 0.3` band between them is thin enough
    // that the partial fraction is a real value and not a saturated 1. A fixture
    // wholly underwater would take neither side of `clamp(sub, 0, 1)` and would
    // stay green with the clamp deleted ([[reference_topic_fixture_discipline]]).
    // The assertion below pins that: the dry ones must not move AT ALL and the
    // wet ones must move a lot — that is the surface, observed through the
    // product's own lowering rather than asserted about an internal.
    //
    // `density = 40`, not the default 12, and the reason is arithmetic, not
    // taste: at `dt = 0.05` the default displaces the field 0.0859 in two ticks,
    // *under* the `MUST_MOVE` floor of 0.1. The honest move is a fixture that
    // pushes harder, never a floor lowered until the gate agrees
    // ([[feedback_frozen_bar_check_the_arithmetic_before_gaming_it]]).
    //
    // The wave is deliberately steep (`amplitude 0.6` over `wave_length 2.5`
    // tilts the normal by up to ~56°): the surface slope is the whole reason the
    // parabolic sine is ported, and a flat sea would leave `slope = 0`,
    // `inv_len = 1` and never read the cosine at all.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(
        &reg,
        &[(
            "force.buoyancy",
            &[
                ("level", 0.0),
                ("density", 40.0),
                ("depth", 0.3),
                ("drag", 2.75),
                ("wave_amplitude", 0.6),
                ("wave_length", 2.5),
                ("wave_speed", 0.4),
            ],
        )],
    );
    let cpu = cpu_ticks(&g, &reg, out, 2);
    // grid + buoyancy + integrate.
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 2, 3);

    let moves: Vec<f32> = cpu[0]
        .iter()
        .zip(&cpu[2])
        .map(|(x, y)| {
            (0..2)
                .map(|k| (x.world_pos[k] - y.world_pos[k]).abs())
                .fold(0.0f32, f32::max)
        })
        .collect();
    let dry = moves.iter().filter(|m| **m < 1e-6).count();
    let wet = moves.iter().filter(|m| **m > MUST_MOVE).count();
    assert!(
        dry > 0 && wet > 0,
        "the fixture must straddle the waterline — {dry} dry, {wet} afloat; with \
         everything on one side of it this compares two seeds and would pass \
         with the submersion clamp dead"
    );
    eprintln!(
        "buoyancy: {dry} dry, {wet} afloat, max move {}",
        max_move(&cpu[0], &cpu[2])
    );
    assert_parity("buoyancy one step", &cpu[2], &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn one_step_of_spring_matches_the_cpu() {
    // The second sequential node, and the one this suite's `MUST_MOVE` idiom
    // does NOT protect. Everywhere else the mover under test is the only thing
    // moving the field, so "did it move?" and "did the kernel fire?" are the
    // same question. Here they come apart: the **oscillator** moves the field,
    // and a spring compiled to a pass-through would sail through `MUST_MOVE`
    // and through parity, because the CPU pass-through and the GPU one agree
    // perfectly ([[reference_topic_oracle_discipline]] — the oracle has to model
    // the appearance, and the appearance of a spring is LAG).
    //
    // So the oracle is the same chain WITHOUT the spring: the raw target. The
    // spring's whole job is to not be there yet, and that gap is what is
    // asserted before any comparison is believed.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (gt, out_t, _) = spring_chain(&reg, false, 0.0, 1.0);
    let cpu_target = cpu_ticks(&gt, &reg, out_t, 2);

    // BOTH sub-step regimes. `steps = ceil(dt/sqrt(STABLE/tension))`, so at the
    // default tension of 8 the adaptive loop runs exactly ONCE — the whole
    // reason it is a loop goes untested, and a kernel that hardcoded one step
    // would be green. 60 (the UI's max) is the first fixture in reach that
    // takes two ([[reference_topic_fixture_discipline]]).
    for (tension, want_steps) in [(8.0f32, 1), (60.0f32, 2)] {
        let (gs, out_s, _) = spring_chain(&reg, true, tension, 1.0);
        let cpu_spring = cpu_ticks(&gs, &reg, out_s, 2);

        let lag = max_move(&cpu_target[2], &cpu_spring[2]);
        assert!(
            lag > MUST_MOVE,
            "tension {tension}: the spring must LAG its target — gap {lag} ≤ \
             {MUST_MOVE}, so this compares the oscillator to itself and would \
             pass with the spring compiled to a pass-through"
        );
        // grid + oscillator + spring.
        let gpu_out = gpu_ticks(&gpu, &gs, &reg, out_s, 2, 3);
        eprintln!("spring tension {tension} ({want_steps} sub-step/s): lags by {lag}");
        assert_parity(
            &format!("spring one step, tension {tension}"),
            &cpu_spring[2],
            &gpu_out,
        );
    }
}

/// **A spring on Rotation and on Size now runs on the DEVICE, inside the loop.**
///
/// This gate used to assert the opposite (`a_spring_on_rotation_recedes_to_the_cpu`)
/// and that assertion was right while a static binding set could not switch its
/// output column. `GpuKernel::variant_by_param` switches it, so the fixture
/// FLIPPED rather than being loosened — and flipping it is the point, because the
/// old behaviour was expensive in a way local to nothing: inside a `pre` loop a
/// single boundary makes `plan` refuse the WHOLE simulation (the two-sims rule),
/// so a spring on Rotation dragged every force in the graph back to the CPU with
/// it.
///
/// The lag guard is [`max_channel_move`] and not [`max_move`]: on these channels
/// `world_pos` never moves, so the guard that keeps parity from comparing a
/// pass-through to itself has to read the field the spring actually writes.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_spring_on_rotation_and_size_matches_the_cpu_inside_the_loop() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (channel, label) in [(2.0f32, "Rotation"), (3.0, "Size")] {
        let (gt, out_t, _) = spring_chain(&reg, false, 0.0, channel);
        let cpu_target = cpu_ticks(&gt, &reg, out_t, 2);

        let (gs, out_s, _) = spring_chain(&reg, true, 8.0, channel);
        let plan = plan(&gs, &reg, &reg, out_s);
        assert!(
            plan.is_fully_gpu(),
            "a spring on {label} must be claimed whole — inside a `pre` loop one \
             boundary refuses the entire simulation: {:?}",
            plan.boundaries
        );

        let cpu_spring = cpu_ticks(&gs, &reg, out_s, 2);
        let lag = max_channel_move(&cpu_target[2], &cpu_spring[2], channel);
        assert!(
            lag > MUST_MOVE,
            "{label}: the spring must LAG its target — gap {lag} <= {MUST_MOVE}, \
             so this compares the oscillator to itself and would pass with the \
             spring compiled to a pass-through"
        );
        let gpu_out = gpu_ticks(&gpu, &gs, &reg, out_s, 2, 3);
        eprintln!("spring on {label}: lags by {lag}");
        // The Size channel is the first thing in this file to DRIVE size, so it
        // takes the driven budget — see [`assert_parity_sized`] for the numbers.
        assert_parity_sized(
            &format!("spring on {label}"),
            &cpu_spring[2],
            &gpu_out,
            EPS_POS,
        );
    }
}

/// **A spring on Size with no `size` upstream settles at UNIT scale, not zero.**
///
/// The identity layer, which the gate above cannot reach: there the oscillator
/// materialises `size`, so the binding's `identity` is never read and a kernel
/// declaring `[0,0]` would be byte-perfect against the CPU
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// Behind a bare grid there is no `size` column, so `channel_get` returns the
/// channel's identity — and `size`'s identity is **unit scale**, never the
/// blanket zero the other three channels use. Get it wrong and the spring settles
/// the sprite at scale 0: invisible, on a chain where nothing looks broken.
///
/// The spring is a pass-through here (a constant target seeds and never steps),
/// and that is fine: this gate is about the number the target is READ as, which
/// is exactly what a pass-through publishes.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_spring_on_size_reads_unit_scale_from_an_absent_column() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 20.0);
    g.set_param(grid, "cols", 20.0);
    let sp = g.add_node("motion.spring");
    g.set_param(sp, "channel", 3.0); // Size
    let out = g.add_node("motion.output");
    edge(&mut g, grid, sp, 0, false);
    edge(&mut g, sp, sp, 1, true);
    edge(&mut g, sp, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let cpu = cpu_ticks(&g, &reg, out, 2);
    let last = &cpu[2];
    // The fixture's own premise: the CPU must publish a NON-ZERO size, or a
    // kernel with a zero identity would agree with it and this gate would be
    // measuring nothing.
    assert!(
        last.iter().all(|r| r.size[0] > 0.0 && r.size[1] > 0.0),
        "fixture check: the CPU must settle at unit scale, not zero"
    );
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 2, 2);
    assert_parity("spring on Size, absent column", last, &gpu_out);
}

/// **The sim-zone family on the device** (ADR-0135): a fixed-population
/// `sim.zone` — `grid → zone`, interior `zone --pre--> wind → sim.step →
/// sim.collide → zone.state` — cooked tick for tick on both paths and reconciled.
///
/// This is the FIRST time a sim runs on the GPU through a `sim.zone` rather than a
/// bare `motion.integrate` self-loop, and it exercises three new things at once:
///
/// - **the zone select** — on tick 0 the zone forwards its INIT port (the grid);
///   from tick 1 its STATE port (the interior). A select stuck on `init` would
///   leave the GPU frozen at the lattice while the CPU falls (the `MUST_MOVE`
///   check below fails); a select stuck on `state` would read the empty interior
///   on tick 0 and the population would be zero forever (the count assert in
///   `gpu_ticks` fires). Both mutations die here.
/// - **`sim.step`** — the per-element integrator that reads its own clock column
///   `sim_t` (so a fresh element STARTS at `dt = 0` rather than leaping) — proven
///   by the fall matching over the whole run.
/// - **`sim.collide`** — the static-shape response, on all three shapes. Each is
///   compared against a **free-fall baseline** (a collider placed where it never
///   contacts): if a shape's branch were dead in the fixture, its frame would
///   equal free-fall and the `collider did nothing` assert would fire. That is
///   what makes each shape's parity non-vacuous, not a snapshot of where the
///   elements happen to be.
///
/// Parity is ε (FMA on the device, not a sum-order divergence — these are
/// per-element kernels), reconciled by the same [`EPS_POS`] the Fase 2 kernels
/// carry.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_sim_zone_falls_and_collides_on_the_device_matching_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 40;

    // The free-fall baseline: a floor so far below that no element ever contacts
    // it, so `sim.collide` is a no-op. Every case below must differ from THIS.
    let (gf, of) = zone_chain(&reg, &[("shape", 0.0), ("height", -1.0e6)], None);
    let freefall = cpu_ticks(&gf, &reg, of, TICKS);
    // The fall itself must be real, or every comparison is vacuous.
    assert!(
        max_move(&freefall[0], &freefall[TICKS as usize]) > MUST_MOVE,
        "fixture check: gravity must move the field"
    );

    // The strobe's shallow sea (doc 52), so the demo's exact physics is under test:
    // a flake falls, punches the surface, taps the bed, and floats on the swell.
    let sea: &[(&str, f32)] = &[
        ("level", -0.5),
        ("density", 14.0),
        ("depth", 0.3),
        ("drag", 5.0),
        ("wave_amplitude", 0.14),
        ("wave_length", 2.4),
        ("wave_speed", 0.5),
    ];

    // Non-round collider params so a swapped term cannot hide behind a tidy number.
    // `want` is grid + wind [+ buoyancy] + sim.step + sim.collide — zone/output pass.
    for (label, params, with_sea, want) in [
        // Floor at y = -3: the fall is caught and the pile bounces and settles.
        (
            "floor",
            &[
                ("shape", 0.0),
                ("height", -3.0),
                ("restitution", 0.35),
                ("friction", 0.15),
            ][..],
            None,
            4,
        ),
        // Disc obstacle (a solid dome elements are pushed OUT of).
        (
            "disc",
            &[
                ("shape", 1.0),
                ("center_x", 0.0),
                ("center_y", -4.0),
                ("radius", 3.5),
                ("restitution", 0.4),
                ("friction", 0.2),
            ][..],
            None,
            4,
        ),
        // Bowl (elements are pushed IN — a basin that catches the fall).
        (
            "bowl",
            &[
                ("shape", 2.0),
                ("center_x", 0.0),
                ("center_y", 2.0),
                ("radius", 6.0),
                ("restitution", 0.25),
                ("friction", 0.3),
            ][..],
            None,
            4,
        ),
        // The demo's physics: gravity + the shallow SEA + a floor bed. `buoyancy`
        // accumulates into the same `accel` the step consumes — the composition
        // that the demo (`=10`) shows and that no `integrate` gate exercises.
        (
            "sea+bed",
            &[
                ("shape", 0.0),
                ("height", -1.1),
                ("restitution", 0.25),
                ("friction", 0.35),
            ][..],
            Some(sea),
            5,
        ),
    ] {
        let (g, out) = zone_chain(&reg, params, with_sea);
        let cpu = cpu_ticks(&g, &reg, out, TICKS);
        let gpu_out = gpu_ticks(&gpu, &g, &reg, out, TICKS, want);
        assert_parity(
            &format!("sim.zone / {label}"),
            &cpu[TICKS as usize],
            &gpu_out,
        );
        assert!(
            max_move(&cpu[TICKS as usize], &freefall[TICKS as usize]) > MUST_MOVE,
            "case `{label}` did nothing: its frame equals free-fall, so a branch is \
             dead in the fixture and the parity above proves nothing"
        );
    }
}

/// **The control for the gate above**: the boundary mechanism still WORKS.
///
/// With every channel of the spring claimed, nothing in this suite would notice
/// `plan` losing the ability to refuse at all — every `is_fully_gpu` assertion
/// would simply keep passing.
///
/// So the refusal keeps a subject: `motion.sort` REORDERS the stream, and the
/// kernel contract is per-element (`read_<col>(i)` / `write_<col>(i)`, element
/// `i` writing row `i`). A global permutation is not expressible in it at all —
/// uncoverable by STRUCTURE rather than by backlog, which is what a seam fixture
/// has to rest on ([[feedback_a_seam_fixture_must_rest_on_something_uncoverable]]).
///
/// The graph is re-validated after the splice: an unregistered or ill-typed node
/// also produces a refusal, and that refusal would be green here for a reason
/// that has nothing to do with coverage.
#[test]
fn a_pre_loop_still_refuses_when_a_node_in_it_cannot_be_covered() {
    let reg = registry();
    let (mut g, out, sp) = spring_chain(&reg, true, 8.0, 1.0);
    let sp = sp.expect("the chain has a spring");
    // The control, first: this chain is claimed whole before the splice.
    assert!(plan(&g, &reg, &reg, out).is_fully_gpu());

    let sort = g.add_node("motion.sort");
    g.disconnect(out, 0)
        .expect("the chain wired spring -> output");
    edge(&mut g, sp, sort, 0, false);
    edge(&mut g, sort, out, 0, false);
    g.validate(&reg).expect("the spliced chain is well-typed");

    let refused = plan(&g, &reg, &reg, out);
    assert!(
        !refused.is_fully_gpu(),
        "an uncoverable node inside the loop must leave a boundary"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_sea_of_no_wavelength_matches_the_cpu() {
    // `eval` reads `wave_length` through `.max(1e-3)`, so the kernel must too —
    // and this is the layer the gate above does NOT reach. Removing both param
    // clamps from the kernel **survived** the whole suite, because that fixture's
    // `wave_length = 2.5` never goes near the floor: a clamp is only observable
    // where its domain is empty ([[feedback_a_threshold_must_live_where_the_domain_is_empty]],
    // [[feedback_layered_defenses_need_per_layer_gates]]).
    //
    // At `wave_length = 0` an unclamped kernel computes `phase = x/0 = inf`,
    // `frac(inf) = NaN`, and NaNs the field — which `motion.integrate`'s
    // finiteness guard then freezes, so the tell is a GPU field that sits still
    // while the CPU's moves.
    //
    // **The amplitude is 1e-4 and that is the whole design of this fixture.** The
    // clamp lands the pair at `wave_length = 1e-3` — a wave far finer than the
    // grid — and at the gate above's `amplitude = 0.6` that sea is genuinely
    // CHAOTIC: `slope = amp·2π/λ ≈ 3770·cos`, so the normal lies almost flat and
    // its direction flips with the *sign* of the cosine. Bounded magnitude
    // (`|a| ≤ density`, the normal is a unit vector) with a flipping sign is a
    // 2·density divergence, and one ulp of phase decides it: measured, the two
    // paths split by 0.2022 ≈ 2·40·dt². That is ADR-0127 D4's own point, not a
    // porting bug, and gating it would gate chaos — the fix is a fixture that is
    // well-conditioned, never an ε loosened until the chaos fits under it.
    // At `1e-4` the same clamped sea has `slope ≈ 0.63`, the normal is honest,
    // and the clamp is just as observable: what NaNs an unclamped kernel is the
    // `phase`, which the amplitude never touches. (Not `0.0` — `0·NaN` is NaN by
    // IEEE and would work, but only until a driver's fast-math folds it to 0.)
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, out) = sim_chain(
        &reg,
        &[(
            "force.buoyancy",
            &[
                ("level", 0.0),
                ("density", 40.0),
                ("depth", 0.3),
                ("drag", 2.75),
                // Keeps the clamped `λ = 1e-3` sea out of the sign-flip regime.
                ("wave_amplitude", 1e-4),
                // Below the floor: the node under test is the CLAMPED sea.
                ("wave_length", 0.0),
                ("wave_speed", 0.4),
            ],
        )],
    );
    let cpu = cpu_ticks(&g, &reg, out, 2);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 2, 3);
    let moved = max_move(&cpu[0], &cpu[2]);
    assert!(
        moved > MUST_MOVE,
        "the clamped sea must still float things — moved {moved} ≤ {MUST_MOVE}; a \
         frozen CPU field would match a NaN-frozen GPU one and prove nothing"
    );
    assert_parity("buoyancy, clamped wavelength", &cpu[2], &gpu_out);
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
    use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, SourceWindow};
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
        count_law: Some(|_| SourceWindow::of_count(N)),
        variant_by_param: None,
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
    let march = |gc: &mut GpuCook, target: u64| {
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

// ── ADR-0130: the emitter sim — the id-gather, end to end ────────────────────
//
// THE audit of slice 3. `emitter → [force] → integrate → output` on the GPU, its
// per-particle state paired by the ARITHMETIC gather (`current_id − prev_first`),
// against the CPU's `BTreeMap<id,row>`. Every gate here uses a CAPPED emitter so
// the alive window SLIDES every tick (births outrun the cap → `first` advances,
// the oldest evict): a static count would pair positionally and stay green with
// the gather dead ([[feedback_test_with_product_numbers_not_convenient_ones]]).

/// `emitter → [forces] → integrate → output`, the forces wired into the `pre`
/// loop exactly like [`sim_chain`]. A CAPPED emitter (rate 400, dt 0.05 → 20
/// births/tick; `max` particles) whose window slides from tick 2 on.
fn emitter_sim(
    reg: &NodeRegistry,
    max: f32,
    forces: &[(&str, &[(&str, f32)])],
) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 400.0);
    // Huge life → no deaths; the cap alone slides the window (`first =
    // newest+1−max` advances as `newest` does). `max` 40 at 20 births/tick means
    // the cap binds at tick 2 and every later tick is ~half survivors, half
    // newborns — both the gather AND the per-element seed exercised at once.
    g.set_param(em, "life", 100.0);
    g.set_param(em, "max", max);
    g.set_param(em, "speed", 4.0);
    g.set_param(em, "angle", 90.0);
    // A wide cone so the muzzle velocity really DIFFERS per id: a positional
    // mispair then gives a survivor a stranger's velocity, which parity sees.
    g.set_param(em, "spread", 120.0);
    g.set_param(em, "x", 0.5);
    g.set_param(em, "y", -0.25);
    g.set_param(em, "seed", 7.0);
    g.set_param(em, "size", 0.15);
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, em, ig, 0, false);
    edge(&mut g, ig, out, 0, false);
    match forces {
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

/// Cook the emitter sim tick-by-tick on the GPU, returning EACH tick's lowering
/// (the count changes per tick, so a single last-tick read would hide a
/// mispair on an earlier one). Asserts the plan claims the loop and dispatches.
fn emitter_gpu_ticks(
    gpu: &GpuContext,
    g: &Graph,
    reg: &NodeRegistry,
    out: NodeId,
    ticks: u64,
    want_stages: usize,
) -> Vec<Vec<RenderInstance>> {
    let plan = plan(g, reg, reg, out);
    assert!(
        plan.is_fully_gpu(),
        "the dense emitter sim must run whole on the GPU: {:?}",
        plan.boundaries
    );
    assert!(plan.drives_a_loop(), "the state must live on the GPU");
    assert_eq!(
        plan.dispatching_stages(reg),
        want_stages,
        "every sim node must actually dispatch"
    );
    let mut gc = GpuCook::new();
    let mut frames = Vec::new();
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
        frames.push(read_instances(gpu, gc.instances().expect("cooked")));
    }
    frames
}

/// The largest distance any instance sits from the emitter origin — how far the
/// integration has actually carried the field (each particle is BORN at the
/// origin, so this is `max |sim_d|`).
fn max_from_origin(frame: &[RenderInstance], origin: [f32; 2]) -> f32 {
    frame
        .iter()
        .flat_map(|r| (0..2).map(move |k| (r.world_pos[k] - origin[k]).abs()))
        .fold(0.0f32, f32::max)
}

const EM_ORIGIN: [f32; 2] = [0.5, -0.25];

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_emitter_ballistic_sim_matches_the_cpu() {
    // The pure gather: `emitter → integrate` on the bare self-loop. No force, so
    // each particle drifts by its own muzzle velocity (from `hash(seed, id)`) —
    // which means WHICH prior row each survivor pairs to is exactly what its
    // displacement is. A positional mispair (element `i` ← prev row `i`, a
    // different-aged particle) fails parity the first tick the window slides.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const MAX: f32 = 40.0;
    const TICKS: u64 = 4; // cap binds at tick 2, the window slides at 3 and 4.
    let (g, _, out) = emitter_sim(&reg, MAX, &[]);
    let cpu = cpu_ticks(&g, &reg, out, TICKS);
    // emitter + integrate dispatch; output is a pass-through.
    let gpu = emitter_gpu_ticks(&gpu, &g, &reg, out, TICKS, 2);

    // The window really slid AND newborns coexist with survivors — otherwise the
    // gather is either untouched (static count) or trivial (all fresh).
    let counts: Vec<usize> = cpu.iter().map(Vec::len).collect();
    assert_eq!(counts.last().copied(), Some(MAX as usize), "the cap binds");
    assert!(
        counts[1] < counts[3],
        "the window grew off zero: {counts:?}"
    );
    // The field integrated: the oldest survivors are far from the origin.
    let moved = max_from_origin(&cpu[TICKS as usize], EM_ORIGIN);
    assert!(
        moved > MUST_MOVE,
        "the muzzle velocities must launch the field ({moved}) — else the gather \
         pairs zeros and this proves nothing"
    );
    eprintln!("emitter sim: counts {counts:?}, drifted {moved}");

    for (t, (c, gp)) in cpu.iter().zip(&gpu).enumerate() {
        assert_parity(&format!("emitter ballistic tick {t}"), c, gp);
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_emitter_sim_under_drag_matches_the_cpu() {
    // The accel gather: `emitter → drag → integrate` in the `pre` loop. `drag`
    // writes `−k·vel` per state element, and the integrator gathers that `accel`
    // by the SAME row as `vel`/`sim_d` — so a mispair here also gives a survivor
    // the wrong deceleration. Drag reads `vel`, which the muzzle seeds, so the
    // force has something to act on from tick 1.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const MAX: f32 = 40.0;
    const TICKS: u64 = 4;
    let (g, _, out) = emitter_sim(&reg, MAX, &[("force.drag", &[("coefficient", 2.75)])]);
    let cpu = cpu_ticks(&g, &reg, out, TICKS);
    // emitter + drag + integrate.
    let gpu = emitter_gpu_ticks(&gpu, &g, &reg, out, TICKS, 3);

    let moved = max_from_origin(&cpu[TICKS as usize], EM_ORIGIN);
    assert!(moved > MUST_MOVE, "the sim must integrate: {moved}");
    eprintln!("emitter+drag sim: drifted {moved}");
    for (t, (c, gp)) in cpu.iter().zip(&gpu).enumerate() {
        assert_parity(&format!("emitter+drag tick {t}"), c, gp);
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn scrubbing_the_emitter_sim_reproduces_the_past_window() {
    // D5 for the emitter: `first` is `pure(t)`, so a re-simulated tick reproduces
    // exactly the ids of the original march, and the restored `prev` is a genuine
    // past whose `id[0]` is that tick's `prev_first`. The `id` column travels in
    // the checkpoint (it rides `rest→out→pre→forces`), so the gather on restore
    // is the gather going forward. The quiet failure this prevents: showing the
    // FUTURE's particles at a past tick (the integrator's `dt≤0` clamp hides it).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const MAX: f32 = 40.0;
    const BACK_TO: u64 = 4;
    const MARCHED: u64 = 20;
    let (g, _, out) = emitter_sim(&reg, MAX, &[("force.drag", &[("coefficient", 1.5)])]);

    let cpu = cpu_ticks(&g, &reg, out, MARCHED);
    // The past and the marched-to state must be DIFFERENT particles, or
    // "reproduced the past" and "showed the future" are the same picture. A
    // capped emitter reaches a steady max-spread, so the tell is not distance
    // from origin (both ~0.18) but the per-slot diff: tick 4's window is ids
    // [41,81), tick 20's is [361,401) — wholly disjoint, so element-for-element
    // they sit far apart (different ids → different muzzle velocities).
    assert_eq!(
        cpu[BACK_TO as usize].len(),
        cpu[MARCHED as usize].len(),
        "both ticks are capped to the same count"
    );
    let travelled = max_move(&cpu[BACK_TO as usize], &cpu[MARCHED as usize]);
    assert!(
        travelled > MUST_MOVE,
        "the alive window must have churned between the two ticks ({travelled})"
    );

    let plan = plan(&g, &reg, &reg, out);
    assert!(plan.drives_a_loop());
    let mut gc = GpuCook::new();
    let march = |gc: &mut GpuCook, target: u64| {
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

    let _ = march(&mut gc, MARCHED);
    let (checkpoints, _) = gc.ring_stats();
    assert!(
        checkpoints > 1,
        "the ring must hold a window: {checkpoints}"
    );
    let scrubbed = march(&mut gc, BACK_TO);
    assert_eq!(gc.last_cooked_tick(), Some(BACK_TO));
    assert_parity("emitter scrub back", &cpu[BACK_TO as usize], &scrubbed);
    // And forward again — the restore left a continuable state, not a dead end.
    let resumed = march(&mut gc, MARCHED);
    assert_parity("emitter re-marched", &cpu[MARCHED as usize], &resumed);
}

fn emitter_of(g: &Graph) -> NodeId {
    g.nodes()
        .iter()
        .find(|n| n.type_name == "motion.emitter")
        .expect("the fixture has an emitter")
        .id
}

/// Cook the emitter sim tick-by-tick, optionally editing one emitter param
/// **live** mid-march — with no invalidation of any kind, which is the claim.
fn emitter_gpu_ticks_with_live_edit(
    gpu: &GpuContext,
    g: &mut Graph,
    reg: &NodeRegistry,
    out: NodeId,
    ticks: u64,
    edit: Option<(u64, &str, f32)>,
) -> Vec<Vec<RenderInstance>> {
    let em = emitter_of(g);
    let plan = plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu() && plan.drives_a_loop());
    let mut gc = GpuCook::new();
    let mut frames = Vec::new();
    for t in 0..=ticks {
        if let Some((at, param, v)) = edit
            && t == at
        {
            g.set_param(em, param, v);
        }
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
        frames.push(read_instances(gpu, gc.instances().expect("cooked")));
    }
    frames
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn shrinking_the_life_of_a_live_emitter_leaves_the_survivors_untouched() {
    // THE gate behind narrowing `edit_renumbers_emitter` to `rate` alone (Enio:
    // *"o que impede que as atualizações sejam feitas em tempo real, sem
    // travamentos e sem reiniciar? Godot faz assim"*).
    //
    // `life` does not appear in `birth(k) = k/rate`, so it cannot re-number: it
    // only moves the alive window's LEFT edge. Every particle the shorter life
    // keeps is the SAME particle, pairs to its own row, and must therefore fly
    // an IDENTICAL trajectory to a run that never changed — bit-identical, not
    // within an ε, because nothing about its physics depends on how many older
    // particles were trimmed away beside it.
    //
    // The oracle is exact and needs no ids in the instance stream: ids ascend
    // oldest-first, so a shorter life makes the edited frame a literal SUFFIX of
    // the unedited one.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // rate 400 at `FIXED_DT` = 0.05 → 20 births/tick. `life` binds (not the cap),
    // which is the point: `max` is left wide so the LEFT edge is life's alone to
    // move. **A life of one tick would make this gate vacuous** — every element
    // would be a newborn, nothing would pair, and "the survivors match" would be a
    // claim about the empty set. So life spans FOUR ticks and the shorter one TWO:
    // roughly half the edited window is a survivor with a row to inherit.
    const TICKS: u64 = 9;
    const EDIT_AT: u64 = 6;
    const LIFE: f32 = 0.20;
    const SHORTER: f32 = 0.10;

    let (mut steady, _, out) = emitter_sim(&reg, 1000.0, &[]);
    steady.set_param(emitter_of(&steady), "life", LIFE);
    let mut edited = steady.clone();

    let unedited = emitter_gpu_ticks_with_live_edit(&gpu, &mut steady, &reg, out, TICKS, None);
    let live = emitter_gpu_ticks_with_live_edit(
        &gpu,
        &mut edited,
        &reg,
        out,
        TICKS,
        Some((EDIT_AT, "life", SHORTER)),
    );

    // The fixture must contain the phenomenon: the edit has to actually KILL
    // particles, and survivors must remain — otherwise "the suffix matches" is a
    // statement about an empty set or about two identical runs.
    let (before, after) = (unedited[TICKS as usize].len(), live[TICKS as usize].len());
    assert!(
        after < before && after > 0,
        "the shorter life must trim the window and leave survivors: {before} → {after}"
    );
    // …and the survivors must have MOVED, or a mispair would be invisible.
    let moved = max_from_origin(&live[TICKS as usize], EM_ORIGIN);
    assert!(moved > MUST_MOVE, "the field must have integrated: {moved}");
    eprintln!("live life edit: {before} → {after} alive, drifted {moved}");

    for t in EDIT_AT..=TICKS {
        let (full, cut) = (&unedited[t as usize], &live[t as usize]);
        let tail = &full[full.len() - cut.len()..];
        // EXACTLY zero, not within ε: `max_move` is the same ruler the parity
        // gates use, and here the honest bar is bit-identity.
        let drift = max_move(cut, tail);
        assert_eq!(
            drift, 0.0,
            "tick {t}: shrinking `life` must leave the survivors BIT-identical — \
             a live edit is not allowed to disturb a particle it did not kill"
        );
    }
}

/// Perf probe, not a gate — the sim number the Fase 3 handoff asked for (§9.3:
/// *"só tenho o número stateless (2M @ 4,02 ms)"*), pointed at the decision it
/// unblocks: **`motion.emitter`'s `MAX_ALIVE` ceiling.**
///
/// The ceiling is 4096 and it is applied on BOTH paths (`eval` and the GPU
/// `source_count`) — deliberately, because ADR-0130's parity is "by construction"
/// from one shared `n`, so the two can never be given different caps. Raising it
/// therefore costs whatever the CPU costs, and that is what this measures: the
/// SAME particle sim, tick by tick, down both paths at several window sizes.
///
/// Run it before touching the ceiling (`MAX_ALIVE` has to be raised locally to
/// sample past 4096 — the probe cannot exceed the cap it is measuring):
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_sim --release -- --ignored --nocapture emitter_sim_ceiling_probe
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn emitter_sim_ceiling_probe() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 30;
    // Gravity + drag: the DEMO=5 shape (a force loop, not a bare integrate), so
    // the number describes a sim someone would actually author.
    let forces: &[(&str, &[(&str, f32)])] = &[
        (
            "force.wind",
            &[("angle", 270.0), ("strength", 26.0), ("gust", 0.0)],
        ),
        ("force.drag", &[("coefficient", 0.12)]),
    ];
    eprintln!("  window │      GPU ms/tick │      CPU ms/tick │ CPU/GPU");
    for &want in &[4096.0f32, 65536.0, 262144.0, 1048576.0, 4194304.0] {
        // `emitter_sim`'s rate is fixed at 400, so in TICKS·FIXED_DT seconds only
        // ~601 particles are ever BORN and every row would report the same window
        // — a perf table that prints one number four times is worse than none.
        // The rate has to be chosen so the window is FULL at the sample instant.
        let (mut g, _, out) = emitter_sim(&reg, want, forces);
        let em = emitter_of(&g);
        g.set_param(em, "rate", want / (TICKS as f32 * FIXED_DT as f32));
        let plan = plan(&g, &reg, &reg, out);
        if !plan.is_fully_gpu() {
            eprintln!("  {want:>6} │ NOT FULLY GPU — {:?}", plan.boundaries);
            continue;
        }
        let mut gc = GpuCook::new();
        let march = |gc: &mut GpuCook| {
            for t in 0..=TICKS {
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        march(&mut gc); // warm: pipelines + pool
        let mut gc2 = GpuCook::new();
        let t0 = std::time::Instant::now();
        march(&mut gc2);
        let gpu_ms = t0.elapsed().as_secs_f64() * 1000.0 / (TICKS + 1) as f64;

        let t1 = std::time::Instant::now();
        let cpu = cpu_ticks(&g, &reg, out, TICKS);
        let cpu_ms = t1.elapsed().as_secs_f64() * 1000.0 / (TICKS + 1) as f64;
        let n = cpu.last().map(Vec::len).unwrap_or(0);
        // The row is only about `want` if the window actually GOT there; the cap
        // (or too short a march) silently truncating it is the whole subject.
        let note = if (n as f32) < want * 0.99 {
            " ← CAPPED (MAX_ALIVE), not the requested window"
        } else {
            ""
        };
        eprintln!(
            "  {n:>6} │ {gpu_ms:>16.3} │ {cpu_ms:>16.3} │ {:>6.1}×{note}",
            cpu_ms / gpu_ms
        );
    }
    eprintln!("  (the 60 fps budget is 16.7 ms/frame; the sim is one part of it)");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_emitter_sim_is_exact_past_the_old_id_cliff() {
    // The gate the whole window rework exists for. Ids are `f32`, so a spawn
    // index past 2²⁴ used to collapse onto its neighbour and BOTH pairings — the
    // CPU's `BTreeMap<id,row>` and this engine's `id − prev_first` — silently
    // handed two particles one prior state. Measured before the fix: rate
    // 4.000.000 at t = 5 s gave 4096 particles and 2049 distinct ids.
    //
    // Now the count law runs once, in `f64`, and the kernel is TOLD its window
    // (`SourceWindow`) instead of re-deriving `floor(t·rate)` in `f32`. So this
    // marches a real sim right through the old cliff and demands the two paths
    // still agree element for element at the far side.
    //
    // The fixture is pinned by three constraints that fight each other, which is
    // why the numbers look odd:
    //   • particles must SURVIVE a tick (`life > FIXED_DT`), or every element is
    //     a newborn at the origin and the gather is never exercised at all;
    //   • the window must fit `MAX_ALIVE`;
    //   • and the march must actually reach 2²⁴ spawns.
    // Together those cap the rate and force a long march — so this keeps only
    // the FINAL frame rather than every frame (2100 × 16384 instances would be
    // gigabytes).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const RATE: f32 = 163_840.0;
    const LIFE: f32 = 0.1; // 2× FIXED_DT → half the window survives each tick
    const TICKS: u64 = 2100; // t = 105 s > 2²⁴ / RATE = 102,4 s
    let (mut g, _, out) = emitter_sim(&reg, 16_384.0, &[]);
    let em = emitter_of(&g);
    g.set_param(em, "rate", RATE);
    g.set_param(em, "life", LIFE);

    let spawns = TICKS as f64 * FIXED_DT * f64::from(RATE);
    assert!(
        spawns > 16_777_216.0,
        "the march must pass 2²⁴ spawns, got {spawns:.3e}"
    );

    // CPU, final frame only.
    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    for t in 0..=TICKS {
        let playhead = t as f64 * FIXED_DT;
        cpu.clear();
        ph2d_eval_motion::evaluate_motion_into(
            &mut cook,
            &g,
            &reg,
            out,
            playhead,
            DEFAULT_UV,
            DEFAULT_SIZE,
            &mut cpu,
        )
        .expect("cpu cook");
        cook.advance_tick(&g, &reg, playhead).expect("cpu tick");
    }

    // GPU, same march, final frame only.
    let plan = plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu() && plan.drives_a_loop());
    let mut gc = GpuCook::new();
    for t in 0..=TICKS {
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
    let gpu_out = read_instances(&gpu, gc.instances().expect("cooked"));

    // The fixture must contain the phenomenon: a full window, of particles that
    // have MOVED (a cloud of origins would compare equal and prove nothing).
    assert_eq!(cpu.len(), 16_384, "the window must be full");
    let moved = max_from_origin(&cpu, EM_ORIGIN);
    assert!(moved > MUST_MOVE, "the field must have integrated: {moved}");

    assert_parity("emitter past the 2^24 cliff", &cpu, &gpu_out);
    eprintln!(
        "past the cliff: {} particles after {spawns:.3e} spawns, drifted {moved}",
        cpu.len()
    );
}

/// **What would a GPU→CPU tap at the seam cost?** — the number that decides
/// between grinding 52 kernels and one architectural change.
///
/// The 2026-07-18 measurement (`crates/ph2d-gpu-cook/tests/boundary_arity.rs`)
/// found that the seam only ever hands the GPU the **suffix**, so ONE uncovered
/// node in a particle graph's stream path forfeits the whole sim to the CPU
/// (227 ms/tick at 4.19 M). The obvious alternative is the mirror seam: keep the
/// sim GPU-resident, **read back once** at the boundary, and let the CPU run the
/// uncovered tail. That would unblock EVERY graph at once instead of one class
/// per kernel — if the readback is affordable.
///
/// So: measure it. This is the §0.0 rule applied to an architecture (*"meça antes
/// de limitar"*) — the answer is a bandwidth fact about this machine, not a
/// preference. `read_instances` is the existing readback path (copy to a staging
/// buffer + map + wait), which is exactly the shape a tap would use.
///
/// ## MEASURED (RTX, 2026-07-18) — the tap is REFUTED
///
/// `RenderInstance` is **184 B**, so the frame is far heavier than the sim:
///
/// | window | cook ms/tick | readback ms | MB | GB/s |
/// |---:|---:|---:|---:|---:|
/// | 65 536 | 0,129 | 0,977 | 11,5 | 12,3 |
/// | 262 144 | 0,275 | 10,538 | 46,0 | 4,6 |
/// | 1 048 576 | 1,024 | 66,796 | 184,0 | 2,9 |
/// | **4 194 304** | **3,832** | **268,620** | **736,0** | **2,9** |
///
/// At 4,19 M the readback is **70× the cook** — and **268 ms is worse than the
/// CPU's own 227,8 ms/tick** for the same sim. A GPU-sim + CPU-tail graph would
/// be SLOWER than never touching the GPU. The tap does not merely fail to pay;
/// it is negative.
///
/// ⚠️ **The verdict survives a better implementation.** This `read_instances` is
/// a *synchronous* copy+map+wait (2,9 GB/s effective), so these numbers are an
/// upper bound on cost; a pipelined staging ring would do much better. But the
/// FLOOR is PCIe bandwidth, and at a generous 25 GB/s the 736 MB still costs
/// **~29 ms/tick** — above the whole 16,7 ms frame budget, to move data for a sim
/// that cost 3,8 ms. The ratio is absurd at every window above ~256 k, and no
/// amount of engineering moves a number that is bandwidth-bound by 184 B/element.
///
/// ⇒ **The zero-readback design is not a preference; it is the only viable one**,
/// and **coverage (more kernels) is the only path to reach**. This probe exists so
/// the next agent does not re-propose the mirror seam from first principles: it is
/// an attractive idea, and it is dead.
///
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_sim --release -- --ignored --nocapture readback_tap_cost_probe
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn readback_tap_cost_probe() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 30;
    let forces: &[(&str, &[(&str, f32)])] = &[
        (
            "force.wind",
            &[("angle", 270.0), ("strength", 26.0), ("gust", 0.0)],
        ),
        ("force.drag", &[("coefficient", 0.12)]),
    ];
    let stride = std::mem::size_of::<RenderInstance>();
    eprintln!("  RenderInstance = {stride} B");
    eprintln!("  window │ cook ms/tick │ readback ms │    MB │   GB/s │ tap total");
    for &want in &[65536.0f32, 262144.0, 1048576.0, 4194304.0] {
        let (mut g, _, out) = emitter_sim(&reg, want, forces);
        let em = emitter_of(&g);
        g.set_param(em, "rate", want / (TICKS as f32 * FIXED_DT as f32));
        let plan = plan(&g, &reg, &reg, out);
        if !plan.is_fully_gpu() {
            eprintln!("  {want:>6} │ NOT FULLY GPU — {:?}", plan.boundaries);
            continue;
        }
        let mut gc = GpuCook::new();
        let march = |gc: &mut GpuCook| {
            for t in 0..=TICKS {
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        };
        march(&mut gc); // warm
        let mut gc2 = GpuCook::new();
        let t0 = std::time::Instant::now();
        march(&mut gc2);
        let cook_ms = t0.elapsed().as_secs_f64() * 1000.0 / (TICKS + 1) as f64;

        // The tap: one readback of the cooked frame. Averaged over a few pulls so
        // a single map-wait hiccup does not become the headline number.
        let inst = gc2.instances().expect("cooked");
        let _ = read_instances(&gpu, inst); // warm the staging path
        const PULLS: u32 = 5;
        let t1 = std::time::Instant::now();
        let mut n = 0usize;
        for _ in 0..PULLS {
            n = read_instances(&gpu, inst).len();
        }
        let rb_ms = t1.elapsed().as_secs_f64() * 1000.0 / f64::from(PULLS);
        let mb = (n * stride) as f64 / (1024.0 * 1024.0);
        let gbs = (n * stride) as f64 / (rb_ms / 1000.0) / 1e9;
        eprintln!(
            "  {n:>6} │ {cook_ms:>12.3} │ {rb_ms:>11.3} │ {mb:>5.1} │ {gbs:>6.1} │ {:>9.3}",
            cook_ms + rb_ms
        );
    }
    eprintln!("  (60 fps budget = 16.7 ms/frame. 'tap total' is what a GPU-sim +");
    eprintln!("   CPU-tail graph would pay per tick BEFORE the CPU tail runs.)");
}

/// **Is a BOUNDED readback the same animal as a full one?** — the measurement the
/// "readback is negative" conclusion never took.
///
/// `readback_tap_cost_probe` pulls the ENTIRE instance buffer and reports 268 ms
/// at 4,19 M, which is worse than cooking the whole thing on the CPU. That number
/// is true and it is the reason nothing streams results back on the hot path.
///
/// But the graph panel does not want the whole buffer. Its four consumers of the
/// CPU memo want: a COUNT (already published by `CookShape`, no readback), a
/// 48-point postage stamp, a change digest, and a one-node probe. Forty-eight
/// elements is 1,5 KB — four orders of magnitude off the number the conclusion
/// was measured at, and *"readback is negative"* is a claim about a SIZE
/// ([[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]).
///
/// The suspicion this has to settle is that the cost is not bytes at all but the
/// **map + poll stall**, which a small pull pays in full. So the probe measures
/// both against the same cooked frame: `full` (every element) and `head` (the
/// first 48), at four window sizes. If `head` is flat across the sweep, the cost
/// is the stall and the size is irrelevant; if it tracks `full`, it is bandwidth.
///
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_sim --release -- --ignored --nocapture bounded_readback_cost_probe
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn bounded_readback_cost_probe() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const TICKS: u64 = 30;
    const HEAD: usize = 48; // PREVIEW_POINTS, the panel's postage stamp
    let forces: &[(&str, &[(&str, f32)])] = &[
        (
            "force.wind",
            &[("angle", 270.0), ("strength", 26.0), ("gust", 0.0)],
        ),
        ("force.drag", &[("coefficient", 0.12)]),
    ];
    let stride = std::mem::size_of::<RenderInstance>();
    eprintln!(
        "  RenderInstance = {stride} B · head = {HEAD} elements = {} B",
        HEAD * stride
    );
    eprintln!("  window │  full ms │  head ms │ full MB │   ratio │ cook ms │ cook+tap │ tap cost");
    for &want in &[65536.0f32, 262144.0, 1048576.0, 4194304.0] {
        let (mut g, _, out) = emitter_sim(&reg, want, forces);
        let em = emitter_of(&g);
        g.set_param(em, "rate", want / (TICKS as f32 * FIXED_DT as f32));
        let plan = plan(&g, &reg, &reg, out);
        if !plan.is_fully_gpu() {
            eprintln!("  {want:>6} │ NOT FULLY GPU — {:?}", plan.boundaries);
            continue;
        }
        let mut gc = GpuCook::new();
        for t in 0..=TICKS {
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
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());

        let inst = gc.instances().expect("cooked");
        let n = inst.len() as usize;
        let head = HEAD.min(n);
        const PULLS: u32 = 5;

        let _ = read_instances(&gpu, inst); // warm the staging path
        let t0 = std::time::Instant::now();
        for _ in 0..PULLS {
            let _ = read_instances(&gpu, inst);
        }
        let full_ms = t0.elapsed().as_secs_f64() * 1000.0 / PULLS as f64;

        let _ = read_head(&gpu, inst, head); // warm
        let t1 = std::time::Instant::now();
        for _ in 0..PULLS {
            let v = read_head(&gpu, inst, head);
            assert_eq!(v.len(), head);
        }
        let head_ms = t1.elapsed().as_secs_f64() * 1000.0 / PULLS as f64;

        // ⚠️ The two numbers above are taken on an already-DRAINED device, and
        // that is not where a panel would tap. In a live frame the cook has just
        // been submitted and is still in flight, so the tap's `poll(wait)` drains
        // IT too — the stall is not the tap's 0,023 ms, it is however much cook
        // is left. Measured here by submitting a fresh tick and tapping without
        // polling first, which is exactly the shape of the frame.
        let t2 = std::time::Instant::now();
        for k in 0..PULLS {
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[],
                CookClock {
                    playhead: (TICKS + 1 + k as u64) as f64 * FIXED_DT,
                    tick: Some(TICKS + 1 + k as u64),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            let inst = gc.instances().expect("cooked");
            let v = read_head(&gpu, inst, head);
            assert_eq!(v.len(), head);
        }
        let inflight_ms = t2.elapsed().as_secs_f64() * 1000.0 / PULLS as f64;

        // The control: the same cooks with NO tap, so the tap's share is the
        // difference and not the whole frame ([[feedback_absence_gate_needs_a_presence_sibling]]).
        let t3 = std::time::Instant::now();
        for k in 0..PULLS {
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[],
                CookClock {
                    playhead: (TICKS + 100 + k as u64) as f64 * FIXED_DT,
                    tick: Some(TICKS + 100 + k as u64),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let cook_ms = t3.elapsed().as_secs_f64() * 1000.0 / PULLS as f64;

        let mb = (n * stride) as f64 / 1e6;
        eprintln!(
            "  {n:>7} │ {full_ms:8.3} │ {head_ms:8.3} │ {mb:7.2} │ {:7.1}x │ {cook_ms:8.3} │ {inflight_ms:8.3} │ {:+8.3}",
            full_ms / head_ms.max(1e-9),
            inflight_ms - cook_ms
        );
    }
}

/// Pull the first `n` instances — the bounded tap the probe above measures.
/// Deliberately a plain prefix copy and not a strided gather: this isolates the
/// map+poll STALL from the byte cost, which is the question.
fn read_head(
    gpu: &ph2d_gpu::GpuContext,
    instances: &ph2d_gpu_cook::GpuInstances,
    n: usize,
) -> Vec<RenderInstance> {
    if n == 0 {
        return Vec::new();
    }
    let bytes = (n * std::mem::size_of::<RenderInstance>()) as u64;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("bounded readback probe"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(instances.buffer(), 0, &staging, 0, bytes);
    gpu.queue.submit(Some(encoder.finish()));
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("callback").expect("map");
    let out: Vec<RenderInstance> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    staging.unmap();
    out
}
