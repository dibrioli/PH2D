//! GPU-vs-CPU parity for the **count-changing family** (ADR-0136): compaction
//! (`motion.cull`, `sim.lifetime`), birth (`sim.spawn`), concatenation
//! (`motion.combine`), projection (`value.attribute`) and the ramp's `t` path.
//!
//! Same discipline as `gpu_cpu_parity.rs`: the CPU `eval` is canonical, the
//! device reconciles within ε — except that a compaction's COUNT is not an ε,
//! it is a set: the survivors must be the same elements in the same order
//! (order-preserving is the design, `sim.lifetime`'s own rule).
//!
//!   cargo test -p ph2d-gpu-cook --test gpu_stream_ops --release -- --ignored --nocapture

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
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_value_attribute::register(&mut reg).unwrap();
    // The family under test.
    ph2d_node_motion_cull::register(&mut reg).unwrap();
    ph2d_node_sim_lifetime::register(&mut reg).unwrap();
    ph2d_node_sim_spawn::register(&mut reg).unwrap();
    ph2d_node_motion_combine::register(&mut reg).unwrap();
    // The birth-zone loop.
    ph2d_node_sim_zone::register(&mut reg).unwrap();
    ph2d_node_sim_step::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const FIXED_DT: f64 = 1.0 / 60.0;

fn connect(g: &mut Graph, from: NodeId, to: NodeId, port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed,
    })
    .expect("well-formed edge");
}

fn grid(g: &mut Graph, rows: f32, cols: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", rows);
    g.set_param(n, "cols", cols);
    g.set_param(n, "gap_x", 0.37);
    g.set_param(n, "gap_y", 0.23);
    n
}

/// The family's comparator. Counts are EXACT (a compaction that filtered a
/// different set is not an ε); floats within the parity suite's budgets. `tint` is exact
/// (`1e-5`) for the callers whose colour is a solid or a direct field; a caller whose colour
/// comes from `motion.color_ramp` passes the LUT's ε via [`assert_parity_tint`] (doc 85: the
/// ramp bakes to a 256-sample LUT on the device, so the tint carries ~one sample-step).
fn assert_parity(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance]) {
    assert_parity_tint(label, cpu, gpu, 1e-5);
}

fn assert_parity_tint(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance], tint_tol: f32) {
    assert_eq!(cpu.len(), gpu.len(), "{label}: instance count");
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            let d = (c.world_pos[k] - g.world_pos[k]).abs();
            assert!(d <= 2e-3, "{label} instance {i} world_pos[{k}]: |Δ| {d}");
            let d = (c.size[k] - g.size[k]).abs();
            assert!(d <= 2e-3, "{label} instance {i} size[{k}]: |Δ| {d}");
        }
        for k in 0..4 {
            let d = (c.tint[k] - g.tint[k]).abs();
            assert!(d <= tint_tol, "{label} instance {i} tint[{k}]: |Δ| {d}");
        }
        let d = (c.opacity - g.opacity).abs();
        assert!(d <= 1e-5, "{label} instance {i} opacity: |Δ| {d}");
    }
    eprintln!("{label}: {} instances OK", cpu.len());
}

/// One CPU frame at `playhead` (the canonical lowering), advancing the tick so
/// a sequential graph marches.
fn cpu_frame(
    cook: &mut Cook,
    g: &Graph,
    reg: &NodeRegistry,
    out: NodeId,
    playhead: f64,
) -> Vec<RenderInstance> {
    let mut lowered = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        cook,
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
    lowered
}

/// Cook the graph on BOTH paths across `ticks` and compare the FINAL frame —
/// asserting the plan claimed everything and that the populations agree at
/// EVERY tick (the trajectory, not the endpoint: a wrong compaction can
/// re-converge — the W1.5 lesson, applied to counts).
fn parity_over_ticks(
    label: &str,
    g: &Graph,
    reg: &NodeRegistry,
    out: NodeId,
    ticks: u64,
    tint_tol: f32,
) {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let p = plan(g, reg, reg, out);
    assert!(p.is_fully_gpu(), "{label}: boundaries {:?}", p.boundaries);

    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    let mut cpu_last = Vec::new();
    for t in 0..=ticks {
        let playhead = t as f64 * FIXED_DT;
        cpu_last = cpu_frame(&mut cook, g, reg, out, playhead);
        let gpu_count = gc
            .cook(
                &gpu,
                g,
                reg,
                reg,
                &p,
                &[],
                CookClock {
                    playhead,
                    tick: p.drives_a_loop().then_some(t),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
        assert_eq!(
            cpu_last.len(),
            gpu_count as usize,
            "{label}: population diverged at tick {t}"
        );
    }
    let gpu_out = read_instances(&gpu, gc.instances().expect("cooked"));
    assert_parity_tint(label, &cpu_last, &gpu_out, tint_tol);
}

/// `motion.cull`, all four shapes: fraction / fraction inverted / falloff
/// threshold / a value-driven amount (a length-1 LFO field, the broadcast row-0
/// read). Each case must actually CULL — a fixture that keeps everything would
/// pass with the predicate dead.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_cull_survivors_match_the_cpu_in_every_mode() {
    let reg = registry();
    for (label, mode, amount, invert, wire_lfo) in [
        ("fraction", 0.0, 0.437, 0.0, false),
        ("fraction-inverted", 0.0, 0.437, 1.0, false),
        ("falloff", 1.0, 0.55, 0.0, false),
        ("lfo-amount", 0.0, 1.0, 0.0, true),
    ] {
        let mut g = Graph::new();
        let gr = grid(&mut g, 48.0, 48.0);
        let fall = g.add_node("motion.falloff");
        g.set_param(fall, "radius", 6.0);
        let cull = g.add_node("motion.cull");
        g.set_param(cull, "mode", mode);
        g.set_param(cull, "amount", amount);
        g.set_param(cull, "invert", invert);
        let out = g.add_node("motion.output");
        connect(&mut g, gr, fall, 0, false);
        connect(&mut g, fall, cull, 0, false);
        connect(&mut g, cull, out, 0, false);
        if wire_lfo {
            // Unconnected, an LFO is ONE global oscillation — a length-1 value
            // field: the broadcast shape, whose row 0 is the CPU's `first()`.
            let lfo = g.add_node("value.lfo");
            g.set_param(lfo, "period", 3.1);
            connect(&mut g, lfo, cull, 1, false);
        }
        g.validate(&reg).expect("well-typed");

        let mut cook = Cook::new();
        let cpu = cpu_frame(&mut cook, &g, &reg, out, 0.4);
        assert!(
            !cpu.is_empty() && cpu.len() < 48 * 48,
            "{label}: the fixture must cull SOMETHING and keep something \
             (kept {} of {})",
            cpu.len(),
            48 * 48
        );
        parity_over_ticks(label, &g, &reg, out, 0, 1e-5);
    }
}

/// `sim.lifetime` on an aged population: the emitter's window carries a spread
/// of ages, the hashed per-id variance kills a strict subset, and the
/// survivors' `life` drives the ramp so the number is VISIBLE in the compared
/// tint (a `life` written wrong but a correct kill-set would fail here).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_lifetime_reaps_the_same_flakes_and_writes_the_same_life() {
    let reg = registry();
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 900.0);
    g.set_param(em, "life", 60.0); // the emitter's own window stays wide open
    let lt = g.add_node("sim.lifetime");
    g.set_param(lt, "life", 1.1); // …and the reaper's hashed spans do the killing
    g.set_param(lt, "variance", 0.8);
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, "attr", "life");
    let ramp = g.add_node("motion.color_ramp");
    let out = g.add_node("motion.output");
    connect(&mut g, em, lt, 0, false);
    connect(&mut g, lt, attr, 0, false);
    connect(&mut g, lt, ramp, 0, false);
    connect(&mut g, attr, ramp, 1, false);
    connect(&mut g, ramp, out, 0, false);
    g.validate(&reg).expect("well-typed");

    // At t = 1.6 the oldest emitted flakes are past every hashed span and the
    // newest are inside all of them: the kill must be a PROPER subset.
    let mut cook = Cook::new();
    let cpu = cpu_frame(&mut cook, &g, &reg, out, 1.6);
    let alive_upstream = (900.0f64 * 1.6).floor() as usize;
    assert!(
        !cpu.is_empty() && cpu.len() < alive_upstream,
        "fixture: the reaper must kill some and spare some ({} of ~{alive_upstream})",
        cpu.len()
    );
    // 96 ticks = the 1.6 s the fixture check above proved non-trivial. At a
    // bare `ticks: 0` the emitter is EMPTY and the parity is vacuous — the
    // rows-scatter mutation survived exactly that way (a fixture only proves
    // what it contains).
    parity_over_ticks("lifetime+life→ramp", &g, &reg, out, 96, 6e-3);
}

/// `sim.spawn` marched across ticks, both slot modes. Sequential on BOTH sides
/// (`dt` is the difference of consecutive playheads — the same expression the
/// count law runs on the GPU), so this also gates `CountLawCtx::dt` end to end.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_spawn_births_the_same_ids_from_the_same_template_rows() {
    let reg = registry();
    for (label, scatter) in [("round-robin", 0.0), ("scatter", 1.0)] {
        let mut g = Graph::new();
        let tpl = grid(&mut g, 1.0, 14.0);
        let sp = g.add_node("sim.spawn");
        g.set_param(sp, "rate", 37.0); // fractional per tick: the floor-difference law
        g.set_param(sp, "scatter", scatter);
        let out = g.add_node("motion.output");
        connect(&mut g, tpl, sp, 0, false);
        connect(&mut g, sp, out, 0, false);
        g.validate(&reg).expect("well-typed");

        let Some(gpu) = try_headless_gpu() else {
            eprintln!("no GPU adapter — skipping");
            return;
        };
        let p = plan(&g, &reg, &reg, out);
        assert!(p.is_fully_gpu(), "{label}: boundaries {:?}", p.boundaries);
        let mut cook = Cook::new();
        let mut gc = GpuCook::new();
        let mut born_any = false;
        for t in 0..=40u64 {
            let playhead = t as f64 * FIXED_DT;
            let cpu = cpu_frame(&mut cook, &g, &reg, out, playhead);
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &p,
                &[],
                CookClock::at(playhead),
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            let gpu_out = read_instances(&gpu, gc.instances().expect("cooked"));
            assert_parity(&format!("spawn {label} tick {t}"), &cpu, &gpu_out);
            born_any |= !cpu.is_empty();
        }
        assert!(born_any, "{label}: the fixture never birthed anything");
    }
}

/// `motion.combine`: two streams with ASYMMETRIC column sets — the emitter
/// carries tint/size the grid does not — so the zero-fill convention is in the
/// compared bytes, not just the count.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_combine_lays_the_streams_end_to_end_with_zero_fill() {
    let reg = registry();
    let mut g = Graph::new();
    let a = grid(&mut g, 9.0, 9.0);
    let b = g.add_node("motion.emitter");
    g.set_param(b, "rate", 120.0);
    g.set_param(b, "life", 3.0);
    let comb = g.add_node("motion.combine");
    let out = g.add_node("motion.output");
    connect(&mut g, a, comb, 0, false);
    connect(&mut g, b, comb, 1, false);
    connect(&mut g, comb, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let mut cook = Cook::new();
    let cpu = cpu_frame(&mut cook, &g, &reg, out, 0.8);
    assert!(
        cpu.len() > 9 * 9,
        "fixture: the emitter must contribute rows beyond the grid's"
    );
    // THREE ticks, not one: a fresh wgpu buffer is zero-initialised, so on a
    // first cook a missing zero-fill still reads zeros and passes. Only after
    // the pool RECYCLES buffers (cook 2+, the emitter growing between them)
    // does a skipped `clear_buffer` surface as stale bytes.
    parity_over_ticks("combine", &g, &reg, out, 3, 1e-5);
}

/// `value.attribute`'s whole ladder, each arm visible in the ramp's tint:
/// a scalar column (`age`), a vec2 magnitude (`P`, mode Length), and a mistyped
/// name (zeros — uniform stop-0 colour, asserted so the arm provably ran).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_attribute_projects_scalar_length_and_missing_alike() {
    let reg = registry();
    for (label, name, mode) in [
        ("scalar-age", "age", 0.0),
        ("length-P", "P", 1.0),
        ("missing", "no_such_column", 0.0),
    ] {
        let mut g = Graph::new();
        // length-P needs |P| INSIDE the ramp's [0, 1] clamp to vary the colour —
        // an emitter's spread saturates it; a tight grid does not.
        let src = if name == "P" {
            let n = g.add_node("motion.grid");
            g.set_param(n, "rows", 8.0);
            g.set_param(n, "cols", 8.0);
            g.set_param(n, "gap_x", 0.09);
            g.set_param(n, "gap_y", 0.07);
            n
        } else {
            let em = g.add_node("motion.emitter");
            g.set_param(em, "rate", 700.0);
            g.set_param(em, "life", 2.0);
            em
        };
        let attr = g.add_node("value.attribute");
        g.set_text_param(attr, "attr", name);
        g.set_param(attr, "mode", mode);
        let ramp = g.add_node("motion.color_ramp");
        let out = g.add_node("motion.output");
        connect(&mut g, src, attr, 0, false);
        connect(&mut g, src, ramp, 0, false);
        connect(&mut g, attr, ramp, 1, false);
        connect(&mut g, ramp, out, 0, false);
        g.validate(&reg).expect("well-typed");

        let mut cook = Cook::new();
        let cpu = cpu_frame(&mut cook, &g, &reg, out, 1.1);
        assert!(!cpu.is_empty(), "{label}: fixture must emit");
        if label == "missing" {
            assert!(
                cpu.iter().all(|c| c.tint == cpu[0].tint),
                "a missing attribute is UNIFORM zeros — stop-0 for every element"
            );
        } else {
            assert!(
                cpu.iter().any(|c| c.tint != cpu[0].tint),
                "{label}: the projected field must vary, or the arm is dead"
            );
        }
        // 66 ticks = the 1.1 s of the fixture check — an emitter-fed arm at
        // tick 0 is empty and proves nothing.
        parity_over_ticks(label, &g, &reg, out, 66, 6e-3);
    }
}

/// The ramp's `t` port under a length-1 field (an unconnected LFO): the
/// broadcast row-0 read, against the CPU's fixed `0/1/n` ladder — every element
/// wears the SAME colour, and it is the field's, not the positional key's.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_ramp_t_broadcasts_a_length_one_field() {
    let reg = registry();
    let mut g = Graph::new();
    let gr = grid(&mut g, 12.0, 12.0);
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 5.9);
    let ramp = g.add_node("motion.color_ramp");
    let out = g.add_node("motion.output");
    connect(&mut g, gr, ramp, 0, false);
    connect(&mut g, lfo, ramp, 1, false);
    connect(&mut g, ramp, out, 0, false);
    g.validate(&reg).expect("well-typed");

    let mut cook = Cook::new();
    let cpu = cpu_frame(&mut cook, &g, &reg, out, 0.9);
    assert!(
        cpu.iter().all(|c| c.tint == cpu[0].tint),
        "a broadcast t colours the whole set alike"
    );
    // …and NOT like the positional key (each element its own colour) — the
    // shape the pre-fix CPU painted for every element but 0.
    let mut positional = Graph::new();
    let gr2 = grid(&mut positional, 12.0, 12.0);
    let ramp2 = positional.add_node("motion.color_ramp");
    let out2 = positional.add_node("motion.output");
    connect(&mut positional, gr2, ramp2, 0, false);
    connect(&mut positional, ramp2, out2, 0, false);
    positional.validate(&reg).expect("well-typed");
    let mut cook2 = Cook::new();
    let pos = cpu_frame(&mut cook2, &positional, &reg, out2, 0.9);
    assert!(
        pos.iter().any(|c| c.tint != pos[0].tint),
        "control: the positional key varies"
    );
    parity_over_ticks("ramp-broadcast-t", &g, &reg, out, 54, 6e-3);
}

/// **The price of the compaction seam, measured** (ADR-0136 §2 promises this
/// number): the same zone loop with and without a compaction inside it, GPU
/// ms/tick each, at a population around 65k. The delta is the submit-split +
/// 8-byte readback sync — constant in N by design; this prints it next to the
/// baseline so the constant is a number, not a claim.
///
///   cargo test -p ph2d-gpu-cook --test gpu_stream_ops --release -- --ignored --nocapture the_compaction_seam
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn the_compaction_seam_cost_probe() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // (with a lifetime compaction?, ms/tick) — the loop seeded with a 256x256
    // grid (65 536 elements) and no births, so the population is stable and
    // the two runs march the same field.
    let mut report = Vec::new();
    for with_compaction in [false, true] {
        let mut g = Graph::new();
        let seed = grid(&mut g, 256.0, 256.0);
        let zone = g.add_node("sim.zone");
        let wind = g.add_node("force.wind");
        g.set_param(wind, "strength", 0.4);
        let step = g.add_node("sim.step");
        let out = g.add_node("motion.output");
        connect(&mut g, seed, zone, 0, false); // init
        connect(&mut g, zone, wind, 0, true); // state entry
        connect(&mut g, wind, step, 0, false);
        let tail = if with_compaction {
            let lt = g.add_node("sim.lifetime");
            g.set_param(lt, "life", 1.0e6); // nobody dies: same field, plus the seam
            connect(&mut g, step, lt, 0, false);
            lt
        } else {
            step
        };
        connect(&mut g, tail, zone, 1, false);
        connect(&mut g, zone, out, 0, false);
        g.validate(&reg).expect("well-typed");

        let p = plan(&g, &reg, &reg, out);
        assert!(p.is_fully_gpu() && p.drives_a_loop());
        let mut gc = GpuCook::new();
        let mut best = f64::MAX;
        for t in 0..=120u64 {
            let t0 = std::time::Instant::now();
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &p,
                &[],
                CookClock {
                    playhead: t as f64 * FIXED_DT,
                    tick: Some(t),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            if t > 10 {
                best = best.min(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        report.push((with_compaction, best));
    }
    for (comp, ms) in &report {
        eprintln!(
            "  zone 65 536 el {}: {ms:.3} ms/tick",
            if *comp {
                "com compaction (1 seam)"
            } else {
                "sem compaction     "
            }
        );
    }
    eprintln!("  seam cost: {:.3} ms", report[1].1 - report[0].1);
}

/// **A loop wrap anchors on thinned coverage — on the DEVICE, under a squeezed
/// budget** (ADR-0137). The starvation composed three rules (forward-only
/// recording, oldest-first eviction, seed-anchor wraps) into "every lap re-sims
/// the whole history"; this drives the product sequence — march, wrap, march,
/// wrap — with a budget small enough to force eviction, and asserts the SECOND
/// wrap's replay is bounded by the ring's RESOLUTION (`span / entries`, plus a
/// stride of slack), never by the loop's POSITION. That is the ADR's exact
/// promise for the over-budget regime: history degrades in resolution, not by
/// amputation — the first cut of this gate demanded one-stride anchoring and
/// the thinning refuted it honestly (uniform spread beats dense-at-the-loop
/// when the budget cannot hold both). The CPU twin is `ph2d-eval-motion`'s
/// O(1) gate; this one proves the GPU ring's copy of the policy against the
/// real cook.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_gpu_loop_wrap_replays_at_most_a_stride_under_a_squeezed_budget() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // The zone loop from the seam probe: 64x64 seed, no births (stable field).
    let mut g = Graph::new();
    let seed = grid(&mut g, 64.0, 64.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "strength", 0.4);
    let step = g.add_node("sim.step");
    let out = g.add_node("motion.output");
    connect(&mut g, seed, zone, 0, false);
    connect(&mut g, zone, wind, 0, true);
    connect(&mut g, wind, step, 0, false);
    connect(&mut g, step, zone, 1, false);
    connect(&mut g, zone, out, 0, false);
    g.validate(&reg).expect("well-typed");
    let p = plan(&g, &reg, &reg, out);
    assert!(p.is_fully_gpu() && p.drives_a_loop());

    let mut gc = GpuCook::new();
    // A 64x64 zone state is a few hundred KB; budget ~6 checkpoints so the
    // 0..=200 march MUST evict — the exact regime the old ring starved in.
    gc.set_ring_budget(2 * 1024 * 1024);
    let march = |gc: &mut GpuCook, from: u64, to: u64| {
        for t in from..=to {
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &p,
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
    };
    march(&mut gc, 0, 200);
    // Lap 1's wrap to tick 60: replay whatever it costs, but RECORD as it goes.
    let anchor1 = gc.rewind_for(60);
    march(&mut gc, anchor1, 200);
    // Lap 2's wrap: bounded by the ring's RESOLUTION over the marched span.
    let (entries, bytes) = gc.ring_stats();
    let anchor2 = gc.rewind_for(60);
    let gap_bound = 200 / entries.max(1) as u64 + 8;
    eprintln!(
        "loop wrap anchors: lap 1 at {anchor1}, lap 2 at {anchor2} \
         ({entries} entries, {bytes} B, resolution bound {gap_bound})"
    );
    assert!(
        anchor2 >= 40,
        "the wrap must anchor on kept coverage, never back at the seed \
         (anchored at {anchor2} — the pre-ADR-0137 ring answered 0, forever)"
    );
    assert!(
        60 - anchor2 <= gap_bound,
        "the replay is bounded by resolution, not by the loop's position \
         (anchored at {anchor2}, bound {gap_bound})"
    );
    march(&mut gc, anchor2, 60); // leave the sim consistent at the target
}

/// **The birth zone, whole, on the device** — the miniature of the snow's
/// structure: spawn feeds combine feeds the state loop, lifetime reaps inside
/// it, and the population breathes (grows while young, sheds as flakes age
/// out). Marched 90 ticks with the count compared at EVERY tick and the final
/// frame within ε — this is the gate the whole slice exists for.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_birth_zone_loop_lives_and_dies_on_the_device_matching_the_cpu() {
    let reg = registry();
    let mut g = Graph::new();
    let tpl = grid(&mut g, 1.0, 12.0);
    let sp = g.add_node("sim.spawn");
    g.set_param(sp, "rate", 45.0);
    let comb = g.add_node("motion.combine");
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "strength", 0.8);
    let step = g.add_node("sim.step");
    let lt = g.add_node("sim.lifetime");
    g.set_param(lt, "life", 0.9);
    g.set_param(lt, "variance", 0.6);
    let out = g.add_node("motion.output");
    connect(&mut g, tpl, sp, 0, false);
    connect(&mut g, zone, comb, 0, true); // zone.out --pre--> combine.in0 (the state)
    connect(&mut g, sp, comb, 1, false); // this tick's newborns
    connect(&mut g, comb, wind, 0, false);
    connect(&mut g, wind, step, 0, false);
    connect(&mut g, step, lt, 0, false);
    connect(&mut g, lt, zone, 1, false); // survivors → zone.state
    connect(&mut g, zone, out, 0, false);
    g.validate(&reg).expect("well-typed");

    // The population must BREATHE in the window: births outpace deaths early,
    // deaths bite after ~0.9 s — otherwise half the family idles in the fixture.
    let mut probe = Cook::new();
    let mut counts = Vec::new();
    for t in 0..=90u64 {
        counts.push(cpu_frame(&mut probe, &g, &reg, out, t as f64 * FIXED_DT).len());
    }
    assert!(
        counts.windows(2).any(|w| w[1] > w[0]) && counts.windows(2).any(|w| w[1] < w[0]),
        "fixture: the population must both grow and shrink: {counts:?}"
    );

    parity_over_ticks("birth-zone", &g, &reg, out, 90, 1e-5);
}
