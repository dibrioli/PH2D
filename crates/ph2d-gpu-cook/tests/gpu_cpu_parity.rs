//! GPU-vs-CPU parity (ε) — THE audit of Fase 1 (briefing §8: "o gate de
//! paridade É o audit"; pattern of [[project_painter_w4_spatial_gpu_bloom_sh]]).
//!
//! Cooks the SAME graph through the canonical CPU path (`evaluate_motion`,
//! the exact production lowering) and through the GPU sequencer, and compares
//! every `RenderInstance` field within ε. Tolerance, not bit-equality:
//! ADR-0122 — GPU floats are not bit-reproducible cross-vendor, and WGSL may
//! contract `a*b + c` into an FMA the CPU didn't use. The ε budget is derived,
//! not hand-waved: positions carry the oscillator's phase arithmetic (one
//! possible FMA rounding on a phase of magnitude ≤ ~256 → ~3e-5 in the frac,
//! amplified by amplitude) → 2e-3 absolute; the basis is sin/cos of the same
//! radians (ULP-level) → 1e-4; pass-through defaults are identical bytes but
//! share the float comparator for uniformity.
//!
//! `#[ignore]`: needs a real adapter. Run on a dev machine / GPU lane:
//!   cd Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
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
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

/// The F1.1 chain at 160×160 = 25.600 instances — above `PAR_THRESHOLD`, the
/// same scale as the Fase 0 `cook_determinism` golden.
fn chain(reg: &NodeRegistry, rows: f32) -> (Graph, [NodeId; 4]) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", rows);
    g.set_param(grid, "cols", rows);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 2.5);
    g.set_param(osc, "frequency", 1.3);
    // Small stagger keeps the phase magnitude ≤ ~256 over 25.6k instances, so
    // the f32 frac keeps ~16 bits and the ε budget above is honest.
    g.set_param(osc, "phase_stagger", 0.01);
    g.set_param(osc, "offset", 0.4);
    g.set_param(osc, "phase", 0.15);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 3.0);
    g.set_param(mv, "dy", -1.5);
    let out = g.add_node("motion.output");
    for (a, b) in [(grid, osc), (osc, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, [grid, osc, mv, out])
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;

fn assert_close(what: &str, i: usize, a: f32, b: f32, eps: f32) {
    assert!(
        (a - b).abs() <= eps,
        "instance {i} field {what}: cpu {a} vs gpu {b} (|diff| {} > eps {eps})",
        (a - b).abs()
    );
}

fn assert_parity(cpu: &[RenderInstance], gpu: &[RenderInstance]) {
    assert_eq!(cpu.len(), gpu.len(), "instance count");
    let mut max_pos = 0.0f32;
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            assert_close("world_pos", i, c.world_pos[k], g.world_pos[k], 2e-3);
            max_pos = max_pos.max((c.world_pos[k] - g.world_pos[k]).abs());
            assert_close("size", i, c.size[k], g.size[k], 1e-5);
            assert_close("anchor", i, c.anchor[k], g.anchor[k], 0.0);
        }
        for k in 0..4 {
            assert_close("atlas_uv", i, c.atlas_uv[k], g.atlas_uv[k], 1e-6);
            assert_close("tint", i, c.tint[k], g.tint[k], 1e-6);
            assert_close("basis", i, c.basis[k], g.basis[k], 1e-4);
            assert_close("uv_xform", i, c.uv_xform[k], g.uv_xform[k], 0.0);
            for corner in 0..4 {
                assert_close(
                    "per_corner_tint",
                    i,
                    c.per_corner_tint[corner][k],
                    g.per_corner_tint[corner][k],
                    0.0,
                );
            }
        }
        assert_close("premultiplied", i, c.premultiplied, g.premultiplied, 0.0);
        assert_close("opacity", i, c.opacity, g.opacity, 0.0);
        assert_eq!(c.flip_uv, g.flip_uv, "instance {i} flip_uv");
        assert_eq!(c.texture_id, g.texture_id, "instance {i} texture_id");
        assert_eq!(c.z_order, g.z_order, "instance {i} z_order");
        assert_eq!(c.sampling, g.sampling, "instance {i} sampling");
        assert_eq!(c.clip_group, g.clip_group, "instance {i} clip_group");
        assert_eq!(c.clip_meta, g.clip_meta, "instance {i} clip_meta");
    }
    eprintln!("parity: {} instances, max |Δpos| = {max_pos:e}", cpu.len());
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_fully_gpu_chain_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, [_, _, _, out]) = chain(&reg, 160.0);

    // CPU (canonical): the production path, unchanged.
    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook, &g, &reg, out, PLAYHEAD, DEFAULT_UV, DEFAULT_SIZE, &mut cpu,
    )
    .expect("cpu cook");
    assert_eq!(cpu.len(), 160 * 160);

    // GPU: the plan must claim the WHOLE chain and actually dispatch — a
    // silent CPU fallback comparing CPU to CPU would stay green with the
    // engine broken ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]).
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu(), "the F1.1 chain must be claimed whole");
    assert_eq!(plan.dispatching_stages(&reg), 3);
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let n = gc
        .cook(
            &gpu, &g, &reg, &reg, &plan, None, PLAYHEAD, DEFAULT_UV, DEFAULT_SIZE,
        )
        .expect("gpu cook");
    assert_eq!(n, 160 * 160);
    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));

    assert_parity(&cpu, &gpu_out);

    // Same-device reproducibility: a second cook of the same frame must be
    // byte-identical to the first (catches a racing pass / a stale binding;
    // cross-VENDOR bit-equality is deliberately NOT asserted — ADR-0122).
    let n2 = gc
        .cook(
            &gpu, &g, &reg, &reg, &plan, None, PLAYHEAD, DEFAULT_UV, DEFAULT_SIZE,
        )
        .expect("gpu cook 2");
    assert_eq!(n2, n);
    let gpu_out2 = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&gpu_out),
        bytemuck::cast_slice::<_, u8>(&gpu_out2),
        "same device, same frame → byte-identical"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_hybrid_boundary_chain_matches_the_cpu_within_epsilon() {
    // Oscillator on Rotation (channel 2): its kernel doesn't cover that, so
    // the plan puts the CPU boundary at the oscillator — the CPU cooks
    // grid→oscillator (the REAL `Cook`, canonical semantics), the stream is
    // uploaded once, and the GPU runs move→output + lowering. This exercises
    // the seam: `upload_stream`, the boundary handoff, and a `rot` column
    // flowing into the lowering's sin/cos.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (mut g, [_, osc, mv, out]) = chain(&reg, 160.0);
    g.set_param(osc, "channel", 2.0); // Rotation — outside the kernel's coverage

    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook, &g, &reg, out, PLAYHEAD, DEFAULT_UV, DEFAULT_SIZE, &mut cpu,
    )
    .expect("cpu cook");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(plan.boundary, Some((osc, 0)), "boundary at the uncovered node");
    assert_eq!(
        plan.stages.iter().map(|s| s.node).collect::<Vec<_>>(),
        vec![mv, out]
    );
    // The boundary stream: cook the oscillator on the SAME canonical CPU path.
    let mut boundary_cook = Cook::new();
    let outputs = boundary_cook
        .cook(&g, &reg, osc, PLAYHEAD)
        .expect("boundary cpu cook");
    let boundary = outputs[0].as_stream().clone();

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let n = gc
        .cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            Some(&boundary),
            PLAYHEAD,
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    assert_eq!(n as usize, cpu.len());
    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));
    assert_parity(&cpu, &gpu_out);
}

/// Perf probe, not a gate (mirrors Fase 0's `cook_500k_timing`): the F1.1
/// chain at 707×707 ≈ 500k instances, steady-state cook + full GPU wait per
/// frame. Compare against the CPU baseline (4,93 ms @ 32 threads).
///
///   cd Worktrees/line-gpu-nodes && cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture gpu_cook_500k_timing
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn gpu_cook_500k_timing() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, [_, _, _, out]) = chain(&reg, 707.0);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu());
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    // Warm-up: compiles the pipelines + allocates the pool.
    let n = gc
        .cook(&gpu, &g, &reg, &reg, &plan, None, 0.0, DEFAULT_UV, DEFAULT_SIZE)
        .unwrap();
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let frames = 100u32;
    let t0 = std::time::Instant::now();
    for f in 0..frames {
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            None,
            f64::from(f) / 60.0,
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .unwrap();
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    let per = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);
    eprintln!("gpu cook of {n} instances: {per:.3} ms/frame (encode+submit+wait)");
}
