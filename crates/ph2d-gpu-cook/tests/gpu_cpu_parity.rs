//! GPU-vs-CPU parity (ε) — THE audit of Fase 1 (briefing §8: "o gate de
//! paridade É o audit"; pattern of [[project_painter_w4_spatial_gpu_bloom_sh]]).
//!
//! Cooks the SAME graph through the canonical CPU path (`evaluate_motion`,
//! the exact production lowering) and through the GPU sequencer, and compares
//! every `RenderInstance` field within ε. Tolerance, not bit-equality:
//! ADR-0126 — GPU floats are not bit-reproducible cross-vendor, and WGSL may
//! contract `a*b + c` into an FMA the CPU didn't use. The ε budget is derived,
//! not hand-waved: positions carry the oscillator's phase arithmetic (one
//! possible FMA rounding on a phase of magnitude ≤ ~256 → ~3e-5 in the frac,
//! amplified by amplitude) → 2e-3 absolute; the basis is sin/cos of the same
//! radians (ULP-level) → 1e-4; pass-through defaults are identical bytes but
//! share the float comparator for uniformity.
//!
//! `#[ignore]`: needs a real adapter. Run on a dev machine / GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
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
    // GPU/M5 Fase 2 nodes.
    ph2d_node_motion_transform::register(&mut reg).unwrap();
    ph2d_node_motion_rotate::register(&mut reg).unwrap();
    ph2d_node_motion_scale::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_motion_cull::register(&mut reg).unwrap();
    ph2d_node_motion_tint::register(&mut reg).unwrap();
    ph2d_node_motion_wiggle::register(&mut reg).unwrap();
    ph2d_node_motion_noise::register(&mut reg).unwrap();
    // GPU/M5 Fase 3 — colour.
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
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
        &mut cook,
        &g,
        &reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
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
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(PLAYHEAD),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    assert_eq!(n, 160 * 160);
    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));

    assert_parity(&cpu, &gpu_out);

    // Same-device reproducibility: a second cook of the same frame must be
    // byte-identical to the first (catches a racing pass / a stale binding;
    // cross-VENDOR bit-equality is deliberately NOT asserted — ADR-0126).
    let n2 = gc
        .cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(PLAYHEAD),
            DEFAULT_UV,
            DEFAULT_SIZE,
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
        &mut cook,
        &g,
        &reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cpu cook");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(
        plan.boundaries,
        vec![(osc, 0)],
        "boundary at the uncovered node"
    );
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
            &[(osc, &boundary)],
            CookClock::at(PLAYHEAD),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    assert_eq!(n as usize, cpu.len());
    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));
    assert_parity(&cpu, &gpu_out);
}

// ── Fase 2: one parity test per ported deformer ───────────────────────────
//
// Each is the SMALLEST fully-GPU chain that exercises the node — `grid →
// <deformer> → output` — cooked on both paths and compared within ε. Params
// are non-default and non-round ([[feedback_test_with_product_numbers_not_convenient_ones]])
// so a unit slip (deg↔rad, a swapped operand) can't hide behind a tidy number,
// and `assert_gpu_parity` PROVES the plan dispatches before it compares — a
// silent CPU fallback would compare CPU to CPU and stay green with the kernel
// dead ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]).
//
// The grid emits no `falloff`, so these run with the column ABSENT — the common
// case, and the one the kernel's `read_falloff` identity (1.0 = full effect)
// must match ([[feedback_a_gate_only_proves_what_its_fixture_contains]]); the
// naga gate covers the present/absent WGSL variants exhaustively.

/// A `rows²`-instance grid (≥ `PAR_THRESHOLD` at 160), the source every Fase 2
/// parity chain roots on. Emits `P`/`Index`/`Count` (no `size`/`rot`/`tint`/
/// `falloff`), so the deformers run with their target columns ABSENT.
fn grid_node(g: &mut Graph, rows: f32) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", rows);
    g.set_param(grid, "cols", rows);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    grid
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

/// Build `grid → <ty> → output` at `rows²` instances.
/// Returns the deformer node (for param overrides) and the output sink.
fn deformer_chain(g: &mut Graph, rows: f32, ty: &str) -> (NodeId, NodeId) {
    let grid = grid_node(g, rows);
    let node = g.add_node(ty);
    let out = g.add_node("motion.output");
    connect(g, grid, node);
    connect(g, node, out);
    (node, out)
}

/// Cook `g→out` on the canonical CPU path and the GPU sequencer and assert
/// ε-parity, after asserting the plan claims the whole chain and dispatches
/// exactly `expected_stages` compute passes (grid + the deformer; `output` is
/// pass-through and dispatches nothing).
fn assert_gpu_parity(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    g: &Graph,
    out: NodeId,
    expected_stages: usize,
) {
    g.validate(reg).expect("well-typed");
    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        g,
        reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cpu cook");

    let plan = ph2d_gpu_cook::plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu(), "the chain must be claimed whole");
    assert_eq!(
        plan.dispatching_stages(reg),
        expected_stages,
        "the optimization must actually dispatch"
    );
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let n = gc
        .cook(
            gpu,
            g,
            reg,
            reg,
            &plan,
            &[],
            CookClock::at(PLAYHEAD),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    assert_eq!(n as usize, cpu.len());
    let gpu_out = ph2d_gpu_cook::read_instances(gpu, gc.instances().expect("cooked"));
    assert_parity(&cpu, &gpu_out);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn transform_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (node, out) = deformer_chain(&mut g, 160.0, "motion.transform");
    // Spread the layout about the origin (scale ≠ 1) and offset it — non-round.
    g.set_param(node, "scale", 1.37);
    g.set_param(node, "offset_x", 2.9);
    g.set_param(node, "offset_y", -1.4);
    assert_gpu_parity(&gpu, &reg, &g, out, 2);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn rotate_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (node, out) = deformer_chain(&mut g, 160.0, "motion.rotate");
    // Degrees → the lowering turns `rot` into the sin/cos basis; a non-round,
    // non-90° angle catches a deg↔rad slip or a swapped basis lane.
    g.set_param(node, "angle", 27.3);
    assert_gpu_parity(&gpu, &reg, &g, out, 2);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn scale_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (node, out) = deformer_chain(&mut g, 160.0, "motion.scale");
    // Grows each sprite's `size` (materialized from SIZE_IDENTITY, grid emits none).
    g.set_param(node, "amount", 1.85);
    assert_gpu_parity(&gpu, &reg, &g, out, 2);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn wiggle_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (node, out) = deformer_chain(&mut g, 160.0, "motion.wiggle");
    // Y channel, non-round amplitude/frequency/seed. The integer-hash noise must
    // land bit-exact per lattice cell or the offset diverges by O(amplitude) —
    // far outside ε — so this gate is the real proof the u32 mix ported right.
    g.set_param(node, "channel", 1.0);
    g.set_param(node, "amplitude", 1.7);
    g.set_param(node, "frequency", 0.9);
    g.set_param(node, "seed", 3.0);
    assert_gpu_parity(&gpu, &reg, &g, out, 2);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn tint_solid_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    // A focus field upstream so the tint lerps between the existing white and the
    // target ACROSS the grid — exercising `mixed_tint` at intermediate falloff,
    // not just the f=1 endpoint (radius 27.3 spans the grid → a smooth ramp).
    let foc = g.add_node("motion.falloff");
    g.set_param(foc, "radius", 27.3);
    g.set_param(foc, "center_x", 2.9);
    g.set_param(foc, "center_y", -1.7);
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.31);
    g.set_param(tint, "g", 0.72);
    g.set_param(tint, "b", 0.16);
    g.set_param(tint, "a", 0.85);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, foc);
    connect(&mut g, foc, tint);
    connect(&mut g, tint, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 3); // grid + falloff + tint
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn tint_gradient_kernel_matches_the_cpu_within_epsilon() {
    // Gradient used to fall back to the CPU: the ramp keys off `Index/(Count−1)`
    // and `ColumnBinding.identity` is a CONSTANT, so an absent column could not
    // mean `f32(i)`. The `HAS_<col>` const closes it. Here both columns ARE
    // present (the grid emits them) — the sibling below is the absent case.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    // Same intermediate-falloff idea as the Solid gate: the gradient target must
    // still be lerped INTO the existing tint, not written over it. A gate at
    // f = 1 everywhere would stay green with `mixed_tint` deleted.
    let foc = g.add_node("motion.falloff");
    g.set_param(foc, "radius", 27.3);
    g.set_param(foc, "center_x", 2.9);
    g.set_param(foc, "center_y", -1.7);
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 1.0); // Gradient
    // Start and End differ on every channel and none is round, so a swapped
    // endpoint or a dropped channel cannot hide behind a tidy number.
    g.set_param(tint, "r", 0.31);
    g.set_param(tint, "g", 0.72);
    g.set_param(tint, "b", 0.16);
    g.set_param(tint, "a", 0.85);
    g.set_param(tint, "r2", 0.94);
    g.set_param(tint, "g2", 0.23);
    g.set_param(tint, "b2", 0.58);
    g.set_param(tint, "a2", 0.41);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, foc);
    connect(&mut g, foc, tint);
    connect(&mut g, tint, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 3); // grid + falloff + tint
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_gradient_tint_keys_positionally_when_index_is_absent() {
    // THE gate for the `HAS_<col>` branch, and it needed a fixture that can
    // CONTRADICT it: every GPU-rootable generator (grid, emitter) emits
    // `Index`/`Count`, and every transform node carries the base through, so a
    // normal chain has `HAS_Index` true ALWAYS and the positional fallback would
    // be untested code behind a green suite
    // ([[feedback_a_green_gate_may_be_green_by_accident]]).
    //
    // The oracle here is not the CPU — it is the fallback's own DEFINITION.
    // "Absent Index keys positionally" means precisely: a stream WITHOUT the
    // columns must colour exactly as the same stream WITH `Index = 0..n−1` and
    // `Count = n`. So the gate cooks the real boundary twice, once stripped, and
    // demands the two agree BYTE for byte — no ε, no hand-computed expectation.
    // The `Index`-present run is in turn pinned to the CPU by the sibling above,
    // which is what makes this a parity claim and not a self-consistency one.
    //
    // `motion.cull` is here only to BE a CPU boundary (it has no kernel), which
    // is what lets the test hand the GPU suffix a stream of its own choosing.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let cull = g.add_node("motion.cull");
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 1.0); // Gradient
    g.set_param(tint, "r", 0.31);
    g.set_param(tint, "g", 0.72);
    g.set_param(tint, "b", 0.16);
    g.set_param(tint, "a", 0.85);
    g.set_param(tint, "r2", 0.94);
    g.set_param(tint, "g2", 0.23);
    g.set_param(tint, "b2", 0.58);
    g.set_param(tint, "a2", 0.41);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, cull);
    connect(&mut g, cull, tint);
    connect(&mut g, tint, out);

    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        &g,
        &reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cpu cook");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(plan.boundaries, vec![(cull, 0)], "boundary at the cull");

    let mut boundary_cook = Cook::new();
    let outputs = boundary_cook
        .cook(&g, &reg, cull, PLAYHEAD)
        .expect("boundary cpu cook");
    let boundary = outputs[0].as_stream().clone();
    let n_el = boundary.count();
    assert!(
        n_el > 1,
        "a single element would key at t = 0 and prove nothing"
    );
    // The equality below is only meaningful if the columns being stripped really
    // ARE the positional ones — otherwise "stripped == present" would be a
    // coincidence about this stream, not a statement about the fallback.
    assert!(
        matches!(boundary.get("Index"), Some(Column::Scalar(v))
            if v.iter().enumerate().all(|(i, x)| *x == i as f32)),
        "Index must be 0..n−1 for the positional claim to mean anything"
    );
    assert!(
        matches!(boundary.get("Count"), Some(Column::Scalar(v))
            if v.iter().all(|x| *x == n_el as f32)),
        "Count must be n for the positional claim to mean anything"
    );
    let mut stripped = Stream::new(n_el);
    for (name, col) in boundary.columns() {
        if name != "Index" && name != "Count" {
            stripped.set(name.clone(), col.clone());
        }
    }

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let mut run = |s: &Stream| {
        let n = gc
            .cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[(cull, s)],
                CookClock::at(PLAYHEAD),
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
        assert_eq!(n as usize, n_el);
        ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"))
    };
    let with_index = run(&boundary);
    let positional = run(&stripped);

    // The ramp must actually SWEEP — two flat results agree flatly and would keep
    // this green with the whole Gradient branch deleted.
    let reds: Vec<f32> = positional.iter().map(|r| r.tint[0]).collect();
    let spread = reds.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - reds.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(spread > 0.1, "the positional ramp must sweep: {spread}");

    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&with_index),
        bytemuck::cast_slice::<_, u8>(&positional),
        "an absent Index/Count must key EXACTLY as Index = 0..n−1, Count = n"
    );
    // …and the `Index`-present run is the one the CPU pins, which is what makes
    // the equality above a parity claim rather than self-consistency.
    assert_parity(&cpu, &with_index);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn falloff_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let foc = g.add_node("motion.falloff");
    g.set_param(foc, "shape", 1.0); // Rect — the Chebyshev max/abs branch
    g.set_param(foc, "curve", 3.0); // Smoother — the quintic
    g.set_param(foc, "center_x", 2.9);
    g.set_param(foc, "center_y", -1.7);
    g.set_param(foc, "radius", 27.3);
    // The falloff column is not a RenderInstance field; a `move` reads it and
    // scales its offset by it, so the field value shows up in world_pos.
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 3.1);
    g.set_param(mv, "dy", -1.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, foc);
    connect(&mut g, foc, mv);
    connect(&mut g, mv, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 3); // grid + falloff + move
}

/// Perf probe, not a gate (mirrors Fase 0's `cook_500k_timing`): the F1.1
/// chain at 1415×1415 ≈ **2M instances** — the roadmap's "millions" target —
/// steady-state cook + full GPU wait per frame. Compare against the CPU
/// baseline (Fase 0: 4,93 ms @ 32 threads for 500k).
///
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture gpu_cook_millions_timing
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn gpu_cook_millions_timing() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, [_, _, _, out]) = chain(&reg, 1415.0);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu());
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    // Warm-up: compiles the pipelines + allocates the pool.
    let n = gc
        .cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(0.0),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
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
            &[],
            CookClock::at(f64::from(f) / 60.0),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .unwrap();
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    let per = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);
    eprintln!("gpu cook of {n} instances: {per:.3} ms/frame (encode+submit+wait)");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn two_nodes_of_one_type_at_different_column_presence_get_different_pipelines() {
    // A Fase 1 latent, found by Fase 3's fixtures and gated at the level it
    // actually broke: `grid → scale → scale → output`. The grid emits no `size`,
    // so scale #1 runs with the column ABSENT (materializing it from
    // `SIZE_IDENTITY`) and scale #2 with it PRESENT — two different modules, one
    // node TYPE, and the pipeline cache is keyed by type + presence signature.
    //
    // While that signature could not tell a `ReadWrite` column's presence apart,
    // both stages hit ONE cached pipeline and wgpu rejected the second bind
    // group against the first's layout: not a wrong number, a crash. Nothing in
    // Fase 2 caught it because a single-deformer chain never changes a presence.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let s1 = g.add_node("motion.scale");
    g.set_param(s1, "amount", 1.75);
    let s2 = g.add_node("motion.scale");
    g.set_param(s2, "amount", 0.625);
    let out = g.add_node("motion.output");
    for (a, b) in [(grid, s1), (s1, s2), (s2, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(&reg).expect("well-typed");

    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        &g,
        &reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cpu cook");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu());
    assert_eq!(plan.dispatching_stages(&reg), 3, "grid + both scales");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.cook(
        &gpu,
        &g,
        &reg,
        &reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
    )
    .expect("gpu cook");
    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));
    assert_parity(&cpu, &gpu_out);
    // Both stages composed. The value is the CPU's — `scale` materializes an
    // absent `size` from SIZE_IDENTITY (`[1, 1]`), NOT from the lowering's
    // `default_size`, so the product is 1.75 × 0.625 off unit scale and the
    // sprite's default never enters ([[feedback_test_with_product_numbers_not_convenient_ones]]:
    // this asserted `DEFAULT_SIZE × …` on the first draft — plausible, and wrong).
    assert!(
        (gpu_out[0].size[0] - 1.75 * 0.625).abs() < 1e-5,
        "size {} — both stages must have run",
        gpu_out[0].size[0]
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn color_ramp_kernel_matches_the_cpu_within_epsilon() {
    // `grid → color_ramp → output`, every preset × both interpolations. The ramp
    // is keyed on the POSITIONAL index (`t` unconnected — the only shape the
    // kernel claims), which is precisely the identity a constant
    // `ColumnBinding.identity` could not express and the generated `HAS_<col>`
    // now can.
    //
    // Every preset, because they are different STOP TABLES (7 / 5 / 3 / 2 / 2)
    // and the bracket search is what a fixture of one table would never
    // exercise. `interp` is rounded, so it goes through the half-away helper —
    // WGSL's `round` is half-even and would pick the other branch at x.5.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for preset in 0..5 {
        for interp in 0..2 {
            let mut g = Graph::new();
            let grid = grid_node(&mut g, 160.0);
            let cr = g.add_node("motion.color_ramp");
            g.set_param(cr, "preset", preset as f32);
            g.set_param(cr, "interp", interp as f32);
            // Custom's two stops: non-round, and NOT the defaults.
            g.set_param(cr, "a_r", 0.8125);
            g.set_param(cr, "a_g", 0.1875);
            g.set_param(cr, "a_b", 0.4375);
            g.set_param(cr, "b_r", 0.0625);
            g.set_param(cr, "b_g", 0.9375);
            g.set_param(cr, "b_b", 0.5625);
            let out = g.add_node("motion.output");
            for (a, b) in [(grid, cr), (cr, out)] {
                g.connect(Edge {
                    from: (a, 0),
                    to: (b, 0),
                    delayed: false,
                })
                .unwrap();
            }
            g.validate(&reg).expect("well-typed");

            let mut cook = Cook::new();
            let mut cpu = Vec::new();
            ph2d_eval_motion::evaluate_motion_into(
                &mut cook,
                &g,
                &reg,
                out,
                PLAYHEAD,
                DEFAULT_UV,
                DEFAULT_SIZE,
                &mut cpu,
            )
            .expect("cpu cook");

            let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
            assert!(
                plan.is_fully_gpu(),
                "preset {preset}: {:?}",
                plan.boundaries
            );
            assert_eq!(plan.dispatching_stages(&reg), 2, "grid + the ramp");
            let mut gc = ph2d_gpu_cook::GpuCook::new();
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[],
                CookClock::at(PLAYHEAD),
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));

            // The ramp must actually COLOUR: a fixture where every instance came
            // out the same would compare two flat fields and pass with the
            // bracket search dead.
            let spread = cpu
                .iter()
                .flat_map(|c| (0..3).map(move |k| c.tint[k]))
                .fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
            assert!(
                spread.1 - spread.0 > 0.5,
                "preset {preset} must span colours, got {spread:?}"
            );
            eprintln!("color_ramp preset {preset} interp {interp}: spread {spread:?}");
            assert_parity(&cpu, &gpu_out);
        }
    }
}

#[test]
fn a_connected_t_keeps_the_color_ramp_on_the_cpu() {
    // The refusal (no device needed). The kernel only claims the positional key;
    // a `t` field is a length this plan cannot prove, and answering with the
    // index instead of the field would be a silently different colour.
    let mut reg = registry();
    ph2d_node_value_instance_field::register(&mut reg).unwrap();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 8.0);
    let field = g.add_node("value.instance_field");
    let cr = g.add_node("motion.color_ramp");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (field, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (grid, 0),
        to: (cr, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (field, 0),
        to: (cr, 1),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (cr, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        !plan.stages.iter().any(|s| s.node == cr),
        "a connected `t` must keep the ramp on the CPU"
    );

    // …and the SAME graph without the `t` wire is claimed — otherwise this would
    // pass with the ramp refused for any reason at all.
    let mut g2 = Graph::new();
    let grid2 = grid_node(&mut g2, 8.0);
    let cr2 = g2.add_node("motion.color_ramp");
    let out2 = g2.add_node("motion.output");
    for (a, b) in [(grid2, cr2), (cr2, out2)] {
        g2.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    let plan2 = ph2d_gpu_cook::plan(&g2, &reg, &reg, out2);
    assert!(
        plan2.stages.iter().any(|s| s.node == cr2),
        "the `t` wire is what refuses — not the fixture"
    );
}

/// A bare `emitter → output`: a stateless GENERATOR with no `pre` loop, so it
/// plans fully-GPU **without** the ADR-0130 gather (that is the sim, `emitter →
/// integrate`, gated once integrate stops refusing an id stream).
fn emitter_graph(reg: &NodeRegistry, life: f32, max: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 40.0);
    g.set_param(em, "life", life);
    g.set_param(em, "max", max);
    g.set_param(em, "speed", 4.0);
    g.set_param(em, "angle", 90.0);
    g.set_param(em, "spread", 30.0);
    g.set_param(em, "x", 0.5);
    g.set_param(em, "y", -0.25);
    g.set_param(em, "seed", 7.0);
    g.set_param(em, "size", 0.15);
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (em, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(reg).expect("well-typed");
    (g, out)
}

/// The **count law** (ADR-0130 fatia do emitter): `source_count`'s `n(t)` and
/// that the generator dispatches on the GPU. Every particle sits at the origin
/// until a force moves it, so what render parity sees here is the origin + the
/// size; the per-particle `vel` (from the hash) and the `id` gather are the
/// SIM's to prove (`emitter → integrate`, the next slice). This gate's job is
/// the one thing that is visible at emission — HOW MANY, and WHERE they start.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_emitter_generator_matches_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();

    let cook_cpu = |g: &Graph, out: NodeId, t: f64| -> Vec<RenderInstance> {
        let mut cook = Cook::new();
        let mut cpu = Vec::new();
        ph2d_eval_motion::evaluate_motion_into(
            &mut cook,
            g,
            &reg,
            out,
            t,
            DEFAULT_UV,
            DEFAULT_SIZE,
            &mut cpu,
        )
        .expect("cpu cook");
        cpu
    };
    let cook_gpu = |g: &Graph, out: NodeId, t: f64| -> Vec<RenderInstance> {
        let plan = ph2d_gpu_cook::plan(g, &reg, &reg, out);
        assert!(
            plan.is_fully_gpu(),
            "emitter → output must be claimed whole: {:?}",
            plan.boundaries
        );
        assert_eq!(
            plan.dispatching_stages(&reg),
            1,
            "the emitter is the only dispatch (output is a pass-through)"
        );
        let mut gc = ph2d_gpu_cook::GpuCook::new();
        gc.cook(
            &gpu,
            g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(t),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
        ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"))
    };

    // The window SLIDES: births outrun the still-empty tail, so the count grows,
    // then stabilizes once deaths begin and `first` starts advancing. A constant
    // count would not exercise `source_count` at all.
    let (g, out) = emitter_graph(&reg, 3.0, 512.0);
    let mut counts = Vec::new();
    for &t in &[0.37f64, 2.0, 4.0, 10.0] {
        let cpu = cook_cpu(&g, out, t);
        let gpu = cook_gpu(&g, out, t);
        assert_eq!(
            cpu.len(),
            gpu.len(),
            "t={t}: alive count cpu {} vs gpu {}",
            cpu.len(),
            gpu.len()
        );
        assert_parity(&cpu, &gpu);
        counts.push(cpu.len());
    }
    assert!(
        counts[0] < counts[2] && counts[0] > 0,
        "the window must grow off zero — counts {counts:?}"
    );

    // The CAP: `rate·life ≫ max`, so `n = max` and `first = newest+1−max` — the
    // path where `first` advances EVERY tick. `emit` keeps the NEWEST; the GPU
    // must too, because the kernel derives `first` from the (capped) `count`.
    let (gc, outc) = emitter_graph(&reg, 100.0, 256.0);
    let cpu = cook_cpu(&gc, outc, 10.0);
    let gpu = cook_gpu(&gc, outc, 10.0);
    assert_eq!(cpu.len(), 256, "the cap holds n at max");
    assert_parity(&cpu, &gpu);
    eprintln!("emitter: window {counts:?}, capped {}", cpu.len());

    // The HARD ceiling used to be asserted here, by asking for a billion
    // particles and checking both paths landed on the same `n`. It moved to the
    // emitter's own suite for two reasons, and both are improvements:
    //
    //   • the ceiling is now 4M (a GPU memory budget, no longer the CPU's frame
    //     time), so binding it end-to-end would allocate ~176 MB per path to
    //     learn a number the count law already knows;
    //   • and there is now only ONE count law — `emit` and the GPU
    //     `source_window` call the same `window()` — so "both paths agree on n"
    //     stopped being a claim two implementations could falsify and became a
    //     property of having one. Gating it where it can still be contradicted
    //     (the law) beats gating it where it cannot (the seam).
    //
    // What stays here is the seam this file is FOR: the param `max` above, and
    // the sliding window, cooked down both paths and compared instance by
    // instance.
}

/// **The panel can still read the wire mass off a GPU frame.**
///
/// The graph panel's taper is `f(count)` and its usual source is the CPU memo,
/// which a GPU-resident cook never fills — so without this every wire flattens to
/// the same thread exactly when the counts got interesting. The count is not a
/// result, though: it is what the host SIZED the dispatch with, so publishing it
/// is bookkeeping, not a readback (and the readback is measured-negative —
/// `readback_tap_cost_probe`).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_sequencer_publishes_each_staged_nodes_element_count() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    const ROWS: f32 = 40.0;
    let (g, [grid, osc, mv, out]) = chain(&reg, ROWS);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu(), "the F1.1 chain is claimed whole");

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    // Before any cook there is nothing to report — and reporting a confident 0
    // would be worse than reporting nothing (a 0-wide wire is a claim).
    assert_eq!(gc.node_count(grid), None, "no cook yet, no count");

    let n = gc
        .cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(PLAYHEAD),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");

    let want = (ROWS * ROWS) as u32;
    assert_eq!(n, want, "the chain carries rows² instances");
    for (node, label) in [
        (grid, "grid"),
        (osc, "oscillator"),
        (mv, "move"),
        (out, "output"),
    ] {
        assert_eq!(
            gc.node_count(node),
            Some(want),
            "{label} is staged, so its element count is host-known"
        );
    }
    // A node the plan never staged has no entry — the panel then falls back to
    // the CPU memo, which for a hybrid prefix genuinely holds one.
    assert_eq!(
        gc.node_count(ph2d_nodegraph::graph::NodeId(9999)),
        None,
        "an unstaged node reports nothing, never a stale or invented count"
    );
}

/// `motion.noise` — the Perlin 2002 **gradient**-noise field, ported per element.
///
/// The two things this gate is really for:
///
/// 1. **It is gradient noise, not the value noise `force.curl` already ships.**
///    The lattice hash, the eight `(±1,±2)` gradients selected by three hash bits,
///    the quintic fade and the `1/1.5` normalisation all have to land bit-for-bit
///    per cell or the field is a DIFFERENT field — a divergence of O(amplitude),
///    orders outside ε, not a rounding wobble.
/// 2. **The discrete params pick branches.** `type` selects the per-octave
///    rectification and `seed`/`octaves` pick hashes and a loop count, so
///    half-even vs half-away rounding is not an ε either. Hence non-round values
///    below ([[feedback_test_with_product_numbers_not_convenient_ones]]).
///
/// `type` is also the param that forced `codegen::wgsl_field`: it is a WGSL
/// reserved word, and before the sanitizer this kernel could not compile at all.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn noise_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // Every fractal-sum flavour, because each is a different branch inside the
    // octave loop and a fixture with only the default would prove one third of
    // the kernel ([[reference_topic_fixture_discipline]]).
    for (ty, label) in [(0.0, "fbm"), (1.0, "turbulence"), (2.0, "ridged")] {
        let mut g = Graph::new();
        let (node, out) = deformer_chain(&mut g, 160.0, "motion.noise");
        g.set_param(node, "channel", 1.0);
        g.set_param(node, "amplitude", 1.7);
        g.set_param(node, "scale", 0.37);
        g.set_param(node, "octaves", 3.0);
        g.set_param(node, "roughness", 0.62);
        g.set_param(node, "type", ty);
        g.set_param(node, "speed", 0.43);
        g.set_param(node, "seed", 5.0);
        eprintln!("  noise type = {label}");
        assert_gpu_parity(&gpu, &reg, &g, out, 2);
    }
}

/// The Rotation/Size channels have no kernel path (they write a different
/// column), so the plan must RECEDE rather than draw a wrong answer — the
/// `motion.oscillator` precedent, re-asserted per node because `applicable` is
/// per node and a copy-paste that dropped it would be invisible.
#[test]
fn the_noise_recuses_on_the_channels_its_kernel_cannot_write() {
    let reg = registry();
    for channel in [2.0, 3.0] {
        let mut g = Graph::new();
        let (node, out) = deformer_chain(&mut g, 8.0, "motion.noise");
        g.set_param(node, "channel", channel);
        let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
        assert_eq!(
            plan.boundaries,
            vec![(node, 0)],
            "channel {channel} writes rot/size — the boundary is AT the noise"
        );
    }
}
