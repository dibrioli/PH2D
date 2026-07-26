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
    // The field.* focus-field family (index-keyed + spatial-box `falloff` masks).
    ph2d_node_field_index_range::register(&mut reg).unwrap();
    ph2d_node_field_box::register(&mut reg).unwrap();
    ph2d_node_field_combine::register(&mut reg).unwrap();
    ph2d_node_field_radial_sweep::register(&mut reg).unwrap();
    ph2d_node_field_remap::register(&mut reg).unwrap();
    ph2d_node_motion_cull::register(&mut reg).unwrap();
    ph2d_node_motion_tint::register(&mut reg).unwrap();
    ph2d_node_motion_wiggle::register(&mut reg).unwrap();
    ph2d_node_motion_noise::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_value_noise::register(&mut reg).unwrap();
    ph2d_node_value_mix::register(&mut reg).unwrap();
    ph2d_node_value_quantize::register(&mut reg).unwrap();
    ph2d_node_value_gain::register(&mut reg).unwrap();
    ph2d_node_value_step::register(&mut reg).unwrap();
    ph2d_node_value_normalize::register(&mut reg).unwrap();
    ph2d_node_value_unary::register(&mut reg).unwrap();
    ph2d_node_value_reduce::register(&mut reg).unwrap();
    ph2d_node_value_smooth::register(&mut reg).unwrap();
    ph2d_node_value_pattern::register(&mut reg).unwrap();
    // The value-domain combiner + router (the widest-input count law).
    ph2d_node_value_math::register(&mut reg).unwrap();
    ph2d_node_value_switch::register(&mut reg).unwrap();
    ph2d_node_motion_luminance::register(&mut reg).unwrap();
    ph2d_node_value_map_range::register(&mut reg).unwrap();
    ph2d_node_motion_orbit::register(&mut reg).unwrap();
    ph2d_node_motion_pin_constraint::register(&mut reg).unwrap();
    ph2d_node_motion_stagger::register(&mut reg).unwrap();
    ph2d_node_motion_spring::register(&mut reg).unwrap();
    ph2d_node_motion_look_at::register(&mut reg).unwrap();
    ph2d_node_motion_sort::register(&mut reg).unwrap();
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    ph2d_node_value_instance_field::register(&mut reg).unwrap();
    ph2d_node_value_attribute::register(&mut reg).unwrap();
    // GPU/M5 Fase 3 — colour.
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    // The test-local kernel-less fixtures the seam gates rest on.
    nokernel_fixture::register(&mut reg);
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
            // **`size` gets the same bound as `world_pos`, because it now carries
            // the same deltas.** It used to be `1e-5`, calibrated when nothing
            // could DRIVE it: every gate left it at the default, so the bound
            // really meant "identical". `motion.oscillator`/`noise`/`wiggle` write
            // a wave delta there now (`variant_by_param`).
            //
            // Measured, from the identical arithmetic: worst **1,66e-4** on size,
            // and **4,39e-4** on position in `the_fully_gpu_chain` — which has
            // always passed under `2e-3`. Holding size to `1e-5` was holding one
            // field 200× tighter than the field the same wave lands on.
            //
            // Relative-to-result was tried and is the WRONG model: a delta can
            // land near a cancellation (`1.0 + (−0.81) = 0.19`) where the result
            // is small but the error is inherited from the delta's magnitude, so
            // a relative bound tightens exactly where the arithmetic is hardest.
            //
            // What this must catch — a variant writing the wrong column, or the
            // wrong delta — diverges by about the amplitude (1,9), three orders
            // of magnitude above the bound.
            assert_close("size", i, c.size[k], g.size[k], 2e-3);
            assert_close("anchor", i, c.anchor[k], g.anchor[k], 0.0);
        }
        for k in 0..4 {
            assert_close("atlas_uv", i, c.atlas_uv[k], g.atlas_uv[k], 1e-6);
            // **`tint` gets `1e-5`, because the falloff mask it now carries runs
            // richer arithmetic.** The Solid tint is `colour × falloff`, so a field's
            // mask reaches this field; `1e-6` was calibrated when only `field.box`
            // drove it (its rotation is one FMA site, and it stayed under `1e-6`).
            // `field.radial_sweep` adds a `sqrt` (the radial distance), a division
            // (the pseudo-angle), a `wrap_sym` fold and the smooth curve — each an
            // FMA site where a GPU fuses `a*b + c` and the CPU rounds twice, so the
            // deltas ACCUMULATE. Measured, from the identical arithmetic: worst
            // **1,37e-6** on the `=20` star scene — an honest FMA delta (the
            // compositor already declares runtime is not bit-identical across
            // backends), 7× under this bound. What this must catch — a kernel writing
            // the WRONG mask (a dropped ramp, a swapped axis) — diverges by about the
            // mask amplitude (~0,1..1), five orders of magnitude above the bound.
            assert_close("tint", i, c.tint[k], g.tint[k], 1e-5);
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
    // A `motion.sort` between the grid and the oscillator: the CPU cooks
    // grid→sort (the REAL `Cook`, canonical semantics), the stream is uploaded
    // once, and the GPU runs oscillator→move→output + lowering. This exercises
    // the seam: `upload_stream`, the boundary handoff, and a full instance
    // stream crossing it.
    //
    // ⚠️ The boundary used to be an oscillator on **Rotation**, which its kernel
    // did not cover — and covering every channel (`variant_by_param`) deleted
    // this fixture's seam outright. `motion.sort` REORDERS the stream, a global
    // permutation, and this engine's kernel contract is strictly per-element
    // (`i` → element `i`), so it is uncoverable by STRUCTURE rather than by
    // backlog ([[feedback_a_seam_fixture_must_rest_on_something_uncoverable]]).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (mut g, [grid, osc, mv, out]) = chain(&reg, 160.0);
    // Splice the sort in: grid → sort → oscillator.
    let srt = g.add_node("motion.sort");
    g.disconnect(osc, 0).expect("the chain wired grid → osc");
    g.connect(Edge {
        from: (grid, 0),
        to: (srt, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (srt, 0),
        to: (osc, 0),
        delayed: false,
    })
    .unwrap();

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
        vec![(srt, 0)],
        "boundary at the uncovered node"
    );
    assert_eq!(
        plan.stages.iter().map(|s| s.node).collect::<Vec<_>>(),
        vec![osc, mv, out]
    );
    // The boundary stream: cook the oscillator on the SAME canonical CPU path.
    let mut boundary_cook = Cook::new();
    let outputs = boundary_cook
        .cook(&g, &reg, srt, PLAYHEAD)
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
            &[(srt, &boundary)],
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
fn field_index_range_kernel_matches_the_cpu_within_epsilon() {
    // The first `field.*` source: a mask keyed by ORDINAL `i/(count−1)`, not by
    // position. A downstream Solid tint lerps white → target BY the mask, so the
    // index band becomes a colour gradient across the 160×160 = 25.6k-instance
    // grid — the only way the scalar `falloff` column reaches a RenderInstance
    // field the parity comparator sees. The grid emits no `falloff`, so the
    // kernel exercises its `read_falloff` identity (1.0) path (the common case).
    // Band bounds are deliberately un-round so a swapped Start/End or a dropped
    // param cannot hide behind a tidy number; a `soft` ramp keeps the mask at
    // INTERMEDIATE values across the grid (an all-1 band would stay green with
    // the whole kernel deleted).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let foc = g.add_node("field.index_range");
    g.set_param(foc, "start", 0.23);
    g.set_param(foc, "end", 0.71);
    g.set_param(foc, "soft", 0.14);
    g.set_param(foc, "curve", 2.0); // Smooth — the polynomial the ε budget covers
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
    assert_gpu_parity(&gpu, &reg, &g, out, 3); // grid + index_range + tint
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn field_box_kernel_matches_the_cpu_within_epsilon() {
    // The spatial field: a mask keyed by POSITION (reads `P`, unlike the ordinal
    // index field). A rectangle with a soft plateau, coloured by a Solid tint, so
    // the box ramp becomes a colour gradient across the 160×160 grid — the box's
    // per-axis width/height AND the soft edge run at INTERMEDIATE values (a box
    // larger than the grid would stay green with the whole mask deleted). Extents
    // and centre are un-round so a swapped axis or a dropped param cannot hide.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 31.4);
    g.set_param(bx, "height", 21.7);
    g.set_param(bx, "soft", 7.3);
    g.set_param(bx, "center_x", 2.9);
    g.set_param(bx, "center_y", -1.7);
    // A non-zero, un-round rotation so the shared parabolic-sine basis (trig.rs,
    // the SAME polynomial on both paths) is exercised in parity, not just the
    // axis-aligned fast path.
    g.set_param(bx, "rotation", 23.0);
    g.set_param(bx, "curve", 2.0); // Smooth — the polynomial the ε budget covers
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 0.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, bx);
    connect(&mut g, bx, tint);
    connect(&mut g, tint, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 3); // grid + box + tint
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn field_radial_sweep_kernel_matches_the_cpu_within_epsilon() {
    // The ANGULAR field: a mask keyed by the point's ANGLE about a centre (reads
    // `P`), the HR-5 pseudo-angle sector. Every param runs at an INTERMEDIATE,
    // un-round value so a swapped/dropped one cannot hide, and the scene exercises
    // ALL the divergence-prone paths at once: `repetitions = 3` drives the
    // `wrap_sym` fold; a `radius` that CROSSES the grid drives the `sqrt` radial
    // ramp at intermediate values (a huge radius would stay full with the whole
    // clip deleted); a non-zero `rotation` + the `start`/`end` → pseudo-bounds run
    // the shared parabolic-sine basis on both paths; and points all around the
    // centre hit all four octant branches of `pseudo_angle`. `end − start = 66 <
    // 360`, so the non-`full` angular branch (the hard one) is the one under test.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let sw = g.add_node("field.radial_sweep");
    g.set_param(sw, "radius", 13.5);
    g.set_param(sw, "start_angle", 12.0);
    g.set_param(sw, "end_angle", 78.0);
    g.set_param(sw, "repetitions", 3.0);
    g.set_param(sw, "soft", 0.35);
    g.set_param(sw, "center_x", 2.9);
    g.set_param(sw, "center_y", -1.7);
    g.set_param(sw, "rotation", 23.0);
    g.set_param(sw, "curve", 2.0); // Smooth — the polynomial the ε budget covers
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 0.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, sw);
    connect(&mut g, sw, tint);
    connect(&mut g, tint, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 3); // grid + radial_sweep + tint
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn field_remap_kernel_matches_the_cpu_within_epsilon() {
    // The REMAPPER: it transforms an existing `falloff`, so it must be FED one — a
    // `field.box` with a soft plateau paints the grid an intermediate ramp, and the
    // remap Quantizes it into bands. Every remap param runs at a non-trivial value so
    // a dropped one cannot hide: Quantize with an odd `steps`, an `inner_offset`
    // plateau, a shifted `[min, max]` range, a `multiplier`, and a partial `strength`
    // (the blend that must land the SAME lerp on both devices). The whole
    // `grid -> box -> remap -> tint` chain runs on the device.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 41.0);
    g.set_param(bx, "height", 41.0);
    g.set_param(bx, "soft", 17.0); // a wide soft ramp, so the remap sees the [0,1] range
    let rm = g.add_node("field.remap");
    g.set_param(rm, "contour", 3.0); // Quantize
    g.set_param(rm, "steps", 5.0);
    g.set_param(rm, "inner_offset", 0.15);
    g.set_param(rm, "min", 0.1);
    g.set_param(rm, "max", 0.9);
    g.set_param(rm, "multiplier", 1.1);
    g.set_param(rm, "strength", 0.8);
    // The probability gate: ~40 % of instances are zeroed by an integer hash of their
    // index. It is BIT-identical CPU↔GPU (integer ops), so both devices must zero the
    // EXACT same instances — a mismatch would diverge by the whole tint, not ε.
    g.set_param(rm, "probability", 0.6);
    g.set_param(rm, "seed", 7.0);
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 0.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, bx);
    connect(&mut g, bx, rm);
    connect(&mut g, rm, tint);
    connect(&mut g, tint, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 4); // grid + box + remap + tint
}

/// A1-gpu, the PLAN half — no GPU adapter needed, so it runs in CI. Before the LUT
/// channel the Curve contour (mode 4) declined its kernel (`applicable`), so a chain
/// using it was NOT fully GPU: the whole `field.remap` node dropped to the CPU. Now
/// the curve bakes to a LUT and every contour mode cooks on the device, so the chain
/// is claimed whole. Restoring the old `applicable` gate makes this `is_fully_gpu`
/// FALSE — the mutation that proves the gate.
#[test]
fn the_curve_contour_is_claimed_for_the_gpu() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 64.0);
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 41.0);
    g.set_param(bx, "height", 41.0);
    g.set_param(bx, "soft", 17.0);
    let rm = g.add_node("field.remap");
    g.set_param(rm, "contour", 4.0); // Curve — the mode that used to fall back
    g.set_text_param(rm, "curve", "c1 0:0:L 0.5:1:L 1:0:L".to_string()); // a tent
    let out = g.add_node("motion.output");
    connect(&mut g, grid, bx);
    connect(&mut g, bx, rm);
    connect(&mut g, rm, out);
    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        plan.is_fully_gpu(),
        "the Curve contour must cook on the device (A1-gpu), never fall back to the CPU"
    );
}

/// A1-gpu, the DEVICE half. A `field.box` paints the grid a soft ramp; a TENT curve
/// (`0 -> 1 -> 0`) remaps it — a shape no SCALAR contour can make, so if the GPU tracks
/// it the LUT is doing its job. The tint is `colour x remapped_falloff`, so the curve's
/// effect lands there. Compared within the LUT's ε — WIDER than the ULP parity of the
/// scalar contours, because a 256-sample table + lerp cuts the tent's peak corner by
/// ~one sample-step (the documented trade). Dropping the LUT (identity ramp) diverges by
/// the whole tent, ~a hundredfold over this bound.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn field_remap_curve_contour_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 41.0);
    g.set_param(bx, "height", 41.0);
    g.set_param(bx, "soft", 17.0);
    let rm = g.add_node("field.remap");
    g.set_param(rm, "contour", 4.0); // Curve
    g.set_text_param(rm, "curve", "c1 0:0:L 0.5:1:L 1:0:L".to_string());
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 0.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, bx);
    connect(&mut g, bx, rm);
    connect(&mut g, rm, tint);
    connect(&mut g, tint, out);

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
        "the Curve chain must cook whole on the device"
    );
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
    assert_eq!(n as usize, cpu.len());
    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));

    // The curve reaches the render only through the tint (colour x remapped falloff).
    const LUT_TOL: f32 = 6e-3;
    let mut max_tint = 0.0f32;
    for (i, (c, gg)) in cpu.iter().zip(&gpu_out).enumerate() {
        for k in 0..4 {
            max_tint = max_tint.max((c.tint[k] - gg.tint[k]).abs());
            assert_close("tint", i, c.tint[k], gg.tint[k], LUT_TOL);
        }
    }
    eprintln!(
        "curve LUT parity: {} instances, max |Δtint| = {max_tint:e}",
        cpu.len()
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn field_combine_kernel_matches_the_cpu_within_epsilon() {
    // The 2-input COMPOSER: two field branches off ONE grid (a fan-out) — an
    // ordinal band and a spatial box — blended by an explicit mode. This exercises
    // the port-qualified readers `read_a_falloff`/`read_b_falloff` AND that the
    // fan-out plans fully-GPU (the grid is cooked once, reused by both branches:
    // 5 dispatching stages, not 6). Max (union) so BOTH branches' masks reach the
    // output at intermediate values — a Min/Multiply where one branch is 0 could
    // hide the other's arithmetic. Params are un-round so nothing hides.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 160.0);
    let ir = g.add_node("field.index_range");
    g.set_param(ir, "start", 0.33);
    g.set_param(ir, "end", 0.66);
    g.set_param(ir, "soft", 0.11);
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 6.7);
    g.set_param(bx, "height", 40.0);
    g.set_param(bx, "soft", 2.3);
    let cmb = g.add_node("field.combine");
    g.set_param(cmb, "mode", 6.0); // Max (union)
    g.set_param(cmb, "strength", 0.85);
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.9);
    g.set_param(tint, "g", 0.31);
    g.set_param(tint, "b", 0.22);
    g.set_param(tint, "a", 0.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, ir);
    connect(&mut g, grid, bx);
    g.connect(Edge {
        from: (ir, 0),
        to: (cmb, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (bx, 0),
        to: (cmb, 1),
        delayed: false,
    })
    .unwrap();
    connect(&mut g, cmb, tint);
    connect(&mut g, tint, out);
    assert_gpu_parity(&gpu, &reg, &g, out, 5); // grid + ir + box + combine + tint
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
    // The CPU boundary. This used to be `motion.cull` "because it has no
    // kernel" — ADR-0136 gave it one and the premise died loudly, as the
    // uncoverable-fixture doc promises. A test-local node can never gain a
    // kernel, so the seam rests on structure again.
    let cull = g.add_node("test.inst_nokernel");
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
fn a_connected_t_is_claimed_since_the_broadcast_reader_exists() {
    // This gate used to pin the OPPOSITE — `RefuseIfPresent` kept a connected
    // `t` on the CPU because a wrong-length field would be silently judged
    // absent. ADR-0136 retired the refusal: the `t` binding is `ReadBroadcast`
    // (the `0/1/n` ladder, with mixed lengths REFUSED at cook time by
    // `BroadcastLengthMismatch`), so the wired ramp is claimable and the plan
    // must claim it — the snow's colour chain depends on exactly this.
    let reg = registry();
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
        plan.stages.iter().any(|s| s.node == cr),
        "a connected `t` must be claimed now that the broadcast reader exists"
    );
    assert!(
        plan.boundaries.is_empty(),
        "the whole chain has kernels — no boundary expected"
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
    //     `count_law` call the same `window()` — so "both paths agree on n"
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

/// **The Rotation and Size variants, numerically** — the payoff of
/// `variant_by_param`, compared against the CPU for every node that ships them.
///
/// The per-node gates above sweep the maths (waveforms, fractal flavours, the
/// integer hash) on the X/Y variant; this sweeps the CHANNEL, which is the axis
/// the variants exist for. Both are needed: the maths gates would pass with the
/// rot/size variants writing the wrong column, and this one would pass with the
/// waveform wrong, because it only ever uses the default one.
///
/// `assert_gpu_parity` compares INSTANCES, and `rot`/`size` both reach them (the
/// lowering folds `rot` into the basis and `size` into the quad), so a variant
/// that wrote its delta to the wrong column shows up as a moved sprite.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_rotation_and_size_variants_match_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for ty in ["motion.noise", "motion.wiggle", "motion.oscillator"] {
        for (channel, label) in [(2.0, "Rotation"), (3.0, "Size")] {
            let mut g = Graph::new();
            let (node, out) = deformer_chain(&mut g, 40.0, ty);
            g.set_param(node, "channel", channel);
            // Non-round, and large enough that a delta landing on the wrong
            // column cannot hide inside ε.
            g.set_param(node, "amplitude", 1.9);
            eprintln!("  {ty} on {label}");
            assert_gpu_parity(&gpu, &reg, &g, out, 2);
        }
    }
}

/// **Two VARIANTS of one node type, in one cook, must not share a pipeline** —
/// the exact sibling of the presence-signature crash above, for the axis
/// `GpuKernel::variant_by_param` introduced.
///
/// `grid → osc(Rot) → osc(Y) → osc(Rot) → output`: one node TYPE, two different
/// variants, so two different modules — and the pipeline cache is keyed by type +
/// signature. A signature that hashed only the presence BITS puts the last two on
/// the same key, the second is dispatched against the first's layout, and wgpu
/// rejects it: **a crash, not a wrong number**.
///
/// ⚠️ **The THREE oscillators are the fixture, not decoration.** Two (`Y` then
/// `Rot`) do not collide: the `Rot` variant finds `rot` ABSENT, so its bits are
/// `0b00` against the `P` variant's `0b01` and the key differs by accident. The
/// leading `osc(Rot)` materialises `rot`, so the trailing one reads it PRESENT and
/// the two variants finally agree on `0b01` while disagreeing on everything else.
/// The mutation removing the binding hash survives the two-node version.
///
/// It takes one `GpuCook` cooking BOTH — a fresh sequencer per graph, which is
/// what every other gate does, never populates the cache twice. That is why the
/// mutation removing the binding hash from the signature survived the whole suite
/// until this existed.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn two_variants_of_one_type_in_one_cook_get_different_pipelines() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 12.0);
    let mut prev = grid;
    for (channel, amp) in [(2.0f32, 13.0f32), (1.0, 1.3), (2.0, 21.0)] {
        let n = g.add_node("motion.oscillator");
        g.set_param(n, "channel", channel);
        g.set_param(n, "amplitude", amp);
        g.set_param(n, "frequency", 0.7);
        connect(&mut g, prev, n);
        prev = n;
    }
    let out = g.add_node("motion.output");
    connect(&mut g, prev, out);
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu(), "every variant is claimed");
    assert_gpu_parity(&gpu, &reg, &g, out, 4);
}

/// **Every channel-switching node claims EVERY channel now** — the payoff of
/// `GpuKernel::variant_by_param`, asserted across the family rather than per
/// node, because they all had the identical `applicable` restriction for the
/// identical reason and a copy-paste that kept one behind would be invisible.
///
/// This gate used to assert the OPPOSITE for `motion.noise` (Rotation/Size
/// recede). That assertion was correct while a static binding set could not
/// switch columns; the engine can switch now, so the fixture flipped rather than
/// being loosened.
#[test]
fn every_channel_switching_node_claims_every_channel() {
    let reg = registry();
    for ty in [
        "motion.noise",
        "motion.wiggle",
        "motion.oscillator",
        "motion.drive",
        "motion.stagger",
        "motion.spring",
    ] {
        for channel in [0.0, 1.0, 2.0, 3.0] {
            let mut g = Graph::new();
            let (node, out) = deformer_chain(&mut g, 8.0, ty);
            g.set_param(node, "channel", channel);
            // ⚠️ VALIDATE, do not merely plan. A type this registry does not
            // carry is unresolvable, `eligible` refuses it, and the plan seams
            // there — so a node missing from `registry()` looks exactly like a
            // node whose variants do not cover the channel. Adding `motion.spring`
            // to this sweep failed that way first, and the failure named the
            // channel rather than the omission.
            g.validate(&reg)
                .unwrap_or_else(|e| panic!("{ty} is not registered here: {e:?}"));
            let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
            assert!(
                plan.boundaries.is_empty(),
                "{ty} channel {channel}: the plan left a seam at {:?} — a \
                 channel-switching node should claim every channel through its \
                 variants",
                plan.boundaries
            );
        }
    }
}

/// **A node that emits a different KIND of stream than it consumes.**
///
/// `value.lfo` takes instances and emits a VALUE stream: one `v` column, and
/// nothing else. Every kernel before it was `INST_VEC2 → INST_VEC2`, where the
/// engine's `out = base + written` is exactly right; here riding the base would
/// hand downstream a VALUE stream still carrying `P`/`Index`/`Count`, which the
/// CPU's does not have. That is not an ε — it is a different shape, and it would
/// surface far away, as some later node reading a column that should be gone.
///
/// So this gate compares the COLUMN SET, not only the numbers. A parity check on
/// `v` alone would stay green with the base wrongly riding through.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_value_node_emits_a_bare_stream_not_the_instance_base() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 12.0);
    let lfo = g.add_node("value.lfo");
    connect(&mut g, grid, lfo);
    // Non-round, and a waveform that is a BRANCH (triangle) so the rounding of
    // `wave` is exercised, not just the default parabolic sine.
    g.set_param(lfo, "wave", 1.0);
    g.set_param(lfo, "period", 0.73);
    g.set_param(lfo, "amplitude", 1.9);
    g.set_param(lfo, "offset", 0.37);
    g.set_param(lfo, "phase", 0.21);
    g.set_param(lfo, "phase_stagger", 0.013);
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, lfo);
    assert!(plan.is_fully_gpu(), "grid → lfo is covered end to end");

    // The CPU's answer, on the canonical path.
    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, lfo, PLAYHEAD).expect("cpu cook");
    let cpu_stream = cpu[0].as_stream();

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

    // THE assertion: the same columns, not merely the same `v`. The grid feeds
    // `P`/`Index`/`Count`, so a base that rode through would show up right here.
    let gpu_cols: Vec<&str> = gc
        .node_columns(lfo)
        .expect("the lfo is staged")
        .iter()
        .map(String::as_str)
        .collect();
    let mut cpu_cols: Vec<&str> = cpu_stream.columns().map(|(n, _)| n.as_str()).collect();
    cpu_cols.sort_unstable();
    assert_eq!(
        gpu_cols, cpu_cols,
        "the GPU stream must carry the SAME columns as the CPU's — a VALUE stream \
         is `v` and nothing else, never the instance base with `v` bolted on"
    );
    assert_eq!(gpu_cols, vec!["v"], "and that column set is exactly `v`");
}

/// **An unconnected `value.lfo` is ONE global oscillation — and the engine has
/// to be TOLD so.**
///
/// This is the sibling the gate above never had. That one connects a grid, so
/// port 0 has a count and the engine's default law ("as wide as port 0") gets
/// the right answer by accident of the fixture. Unconnected, the CPU computes
/// `input(0).count().max(1)` = **1** — one value held across every instance by
/// `motion.drive`'s broadcast rule — while port 0 is empty, so the default law
/// says `0`, a zero-count stage is SKIPPED, and the node emits nothing at all.
///
/// It is not a wrong number: it is the whole `value.*` family being unreachable
/// on the device the moment a consumer of one gets a kernel. Nothing could
/// produce a length-1 VALUE field, so a broadcast reader would have nothing to
/// read — which is exactly how this was found, and why the count law exists.
///
/// The params are chosen so the answer is far from zero: a stage that never ran
/// leaves a zeroed buffer, and a fixture whose right answer is `0.0` cannot tell
/// the two apart ([[reference_topic_fixture_discipline]]).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn an_unconnected_value_node_is_one_global_value_not_zero_of_them() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let lfo = g.add_node("value.lfo");
    // No `connect` — THE point of this gate.
    g.set_param(lfo, "wave", 1.0); // triangle: a branch, not the default sine
    g.set_param(lfo, "period", 0.73);
    g.set_param(lfo, "amplitude", 1.9);
    g.set_param(lfo, "offset", 0.37);
    g.set_param(lfo, "phase", 0.21);
    // `phase_stagger` stays 0: with one element there is nothing to stagger, and
    // a non-zero value here would be a fixture that quietly tests nothing.
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, lfo);
    assert!(plan.is_fully_gpu(), "a lone lfo is covered");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, lfo, PLAYHEAD).expect("cpu cook");
    let cpu_v = match cpu[0].as_stream().get("v") {
        Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
        _ => panic!("the CPU emitted no `v`"),
    };
    assert_eq!(cpu_v.len(), 1, "the CPU's unconnected lfo is one value");
    assert!(
        cpu_v[0].abs() > 0.1,
        "fixture check: the answer must be far from zero ({}), or a stage that \
         never ran would pass this gate with an empty buffer",
        cpu_v[0]
    );

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true); // gate-only: lets `read_column` see `v`
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

    assert_eq!(
        gc.node_count(lfo),
        Some(1),
        "the count law must size the stage at ONE — `Some(0)` is the stage being \
         skipped, which is the bug this gate exists for"
    );
    let gpu_v = gc
        .read_column(&gpu, lfo, "v")
        .expect("the `v` column reads back");
    assert_eq!(gpu_v.len(), 1);
    let d = (gpu_v[0] - cpu_v[0]).abs();
    eprintln!(
        "unconnected lfo: cpu {} gpu {} |dv| {d:e}",
        cpu_v[0], gpu_v[0]
    );
    assert!(
        d < 1e-5,
        "value parity on the global oscillation: |dv| = {d:e}"
    );
}

/// The rest of the bare-emitter family: `motion.luminance` (instances → VALUE,
/// so the base must NOT ride) chained into `value.map_range` (VALUE → VALUE, so
/// it must). One chain proves both halves of the rule the engine now derives
/// from the manifest — and a chain, not two isolated nodes, because the bug this
/// guards against is a wrong stream SHAPE handed to whatever comes next.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_bare_emitters_match_the_cpu_and_keep_their_stream_shape() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 10.0);
    // A tint the luma actually varies over. `motion.tint` is SOLID — every element
    // gets the same colour, so the luma would be constant and this gate would pass
    // with an index slip in the kernel. `motion.color_ramp` maps index → colour,
    // so the field varies and a mispair shows. (The fixture check below is what
    // caught the first draft using a solid tint.)
    let tint = g.add_node("motion.color_ramp");
    g.set_param(tint, "a_r", 0.05);
    g.set_param(tint, "a_g", 0.11);
    g.set_param(tint, "a_b", 0.83);
    g.set_param(tint, "b_r", 0.91);
    g.set_param(tint, "b_g", 0.74);
    g.set_param(tint, "b_b", 0.07);
    let lum = g.add_node("motion.luminance");
    let mr = g.add_node("value.map_range");
    g.set_param(mr, "in_lo", 0.13);
    g.set_param(mr, "in_hi", 0.79);
    g.set_param(mr, "out_lo", -2.3);
    g.set_param(mr, "out_hi", 5.7);
    g.set_param(mr, "clamp", 1.0);
    connect(&mut g, grid, tint);
    connect(&mut g, tint, lum);
    connect(&mut g, lum, mr);
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, mr);
    assert!(
        plan.is_fully_gpu(),
        "grid → tint → luminance → map_range is covered"
    );

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, mr, PLAYHEAD).expect("cpu cook");
    let cpu_stream = cpu[0].as_stream();
    let cpu_v = match cpu_stream.get("v") {
        Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
        _ => panic!("the CPU emitted no `v`"),
    };

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true); // gate-only: lets `read_column` see `v`
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

    // Shape first: both nodes must carry `v` alone. `luminance` gets that by
    // dropping the base; `map_range` by riding one that only had `v`.
    for (node, label) in [(lum, "luminance"), (mr, "map_range")] {
        let cols: Vec<&str> = gc
            .node_columns(node)
            .expect("staged")
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(
            cols,
            vec!["v"],
            "{label} emits a VALUE stream: `v` and nothing else"
        );
    }
    assert_eq!(
        gc.node_count(mr),
        Some(cpu_v.len() as u32),
        "same element count"
    );
    // And the NUMBERS — shape alone would stay green with the luma weights
    // transposed or the map's clamp branch inverted.
    let gpu_v = gc
        .read_column(&gpu, mr, "v")
        .expect("the `v` column reads back");
    assert_eq!(gpu_v.len(), cpu_v.len());
    let worst = gpu_v
        .iter()
        .zip(&cpu_v)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    eprintln!(
        "value parity: {} elements, max |dv| = {worst:e}",
        cpu_v.len()
    );
    assert!(worst < 1e-5, "value parity: max |dv| = {worst:e}");
    assert!(
        cpu_v.iter().any(|v| (v - cpu_v[0]).abs() > 1e-6),
        "fixture check: the field must VARY, or this proves nothing"
    );
}

/// **`motion.look_at` — the first kernel with two CONNECTED stream inputs, and
/// the first that BROADCASTS.**
///
/// The node's whole reason to exist is that its target may be either shape: one
/// global point the entire field turns toward, or a per-element aim. So the gate
/// runs BOTH, from the same graph, with one wire moved — a fixture with only the
/// per-element case would stay green with the broadcast completely broken, since
/// `read(i)` and `read(0)` agree when the field is length N.
///
/// The two lengths come from `value.lfo` itself: unconnected it is ONE value (the
/// count law), connected to the grid it is N. That is not a convenience — it
/// means this gate fails if EITHER the broadcast or the count law regresses.
///
/// The pivot is off-grid and the offset is not a multiple of 90 so a transposed
/// or sign-flipped `atan2` cannot survive: the aim is a real angle in every
/// quadrant.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn look_at_broadcasts_a_single_target_and_reads_a_field_per_element() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for per_element in [false, true] {
        let mut g = Graph::new();
        let grid = grid_node(&mut g, 12.0);
        let tx = g.add_node("value.lfo");
        let ty = g.add_node("value.lfo");
        // Different periods/phases so the two coordinates never coincide — a
        // target on the 45° line would survive an x/y swap.
        g.set_param(tx, "period", 0.61);
        g.set_param(tx, "amplitude", 3.1);
        g.set_param(ty, "period", 0.89);
        g.set_param(ty, "amplitude", 2.3);
        g.set_param(ty, "phase", 0.33);
        if per_element {
            // Connected → a length-N target field, and a stagger so the elements
            // genuinely aim in different directions.
            connect(&mut g, grid, tx);
            connect(&mut g, grid, ty);
            g.set_param(tx, "phase_stagger", 0.021);
            g.set_param(ty, "phase_stagger", 0.017);
        }
        let la = g.add_node("motion.look_at");
        g.set_param(la, "offset", 23.0);
        connect(&mut g, grid, la);
        for (src, port) in [(tx, 1u16), (ty, 2u16)] {
            g.connect(Edge {
                from: (src, 0),
                to: (la, port),
                delayed: false,
            })
            .expect("target port");
        }
        g.validate(&reg).expect("well-typed");

        let label = if per_element {
            "per-element"
        } else {
            "broadcast"
        };
        let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, la);
        assert!(plan.is_fully_gpu(), "{label}: the whole chain is covered");

        let mut cook = Cook::new();
        let cpu = cook.cook(&g, &reg, la, PLAYHEAD).expect("cpu cook");
        let cpu_rot = match cpu[0].as_stream().get("rot") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("{label}: the CPU emitted no `rot`"),
        };

        let mut gc = ph2d_gpu_cook::GpuCook::new();
        gc.retain_streams_for_debug(true);
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

        let gpu_rot = gc
            .read_column(&gpu, la, "rot")
            .expect("the `rot` column reads back");
        assert_eq!(gpu_rot.len(), cpu_rot.len(), "{label}: same element count");
        let worst = gpu_rot
            .iter()
            .zip(&cpu_rot)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        eprintln!(
            "look_at {label}: {} elements, max |drot| = {worst:e} deg",
            cpu_rot.len()
        );
        // Degrees: the Rajan approximation is ~0.09° off true atan2 on BOTH
        // paths identically, so what is compared here is the port, not the model.
        assert!(worst < 1e-3, "{label}: max |drot| = {worst:e} deg");

        // Fixture check: the elements must genuinely aim in DIFFERENT directions,
        // or a kernel that ignored `P` entirely would pass.
        assert!(
            cpu_rot.iter().any(|r| (r - cpu_rot[0]).abs() > 1.0),
            "{label}: fixture check — the field must aim in varied directions"
        );
    }
}

/// A VALUE producer the GPU **cannot** claim, for the fixtures that need a CPU
/// seam to exist at all.
///
/// It is `value.attribute` and not merely some node without a kernel yet, and
/// that distinction is the whole point: covering `value.instance_field` (which
/// these fixtures used first) deleted their seams and turned two gates red for
/// the best possible reason. A seam fixture built on "uncovered so far" erodes
/// every time coverage advances, and the pressure then is to weaken the gate.
///
/// `value.attribute` held this seat while its text param was inexpressible as
/// a static binding; ADR-0136's `StreamOp::Project` covered it anyway (the
/// sequencer resolves the name at cook time) and these fixtures broke loudly,
/// exactly as promised. The only node coverage can NEVER reach is one that
/// does not exist outside this file — so the seam now rests on a test-local
/// kernel-less node whose eval mirrors `value.attribute`'s ladder.
fn uncoverable_value_node(g: &mut Graph, attr: &str) -> NodeId {
    let n = g.add_node("test.value_nokernel");
    g.set_text_param(n, "attr", attr.to_string());
    n
}

/// The test-local kernel-less nodes the seam fixtures rest on (see
/// [`uncoverable_value_node`]): an INST pass-through and a VALUE projector,
/// neither of which any registry outside this file will ever cover.
mod nokernel_fixture {
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

    static INST_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.inst_nokernel"),
        name: "test.inst_nokernel",
        inputs: &[PortSpec {
            name: "in",
            ty: INST,
        }],
        outputs: &[PortSpec {
            name: "out",
            ty: INST,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct InstOp;
    impl NodeOp for InstOp {
        fn manifest(&self) -> &'static NodeManifest {
            &INST_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let out = ctx.input(0).clone();
            ctx.emit(out);
        }
    }

    static VALUE_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.value_nokernel"),
        name: "test.value_nokernel",
        inputs: &[PortSpec {
            name: "in",
            ty: INST,
        }],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct ValueOp;
    impl NodeOp for ValueOp {
        fn manifest(&self) -> &'static NodeManifest {
            &VALUE_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // `value.attribute`'s ladder, scalar arm only — what the two-seam
            // fixtures feed the look_at with (`Index`/`Count`).
            let name = ctx.text_param("attr").unwrap_or("").to_string();
            let out = {
                let input = ctx.input(0);
                let n = input.count();
                let v = match input.get(&name) {
                    Some(Column::Scalar(v)) if v.len() == n => v.clone(),
                    _ => vec![0.0; n],
                };
                Stream::new(n).with("v", Column::Scalar(v))
            };
            ctx.emit(out);
        }
    }

    pub fn register(reg: &mut ph2d_node_registry::NodeRegistry) {
        reg.register(Box::new(InstOp)).unwrap();
        reg.register(Box::new(ValueOp)).unwrap();
    }
}

/// **TWO CPU seams, one march, and the same picture the CPU draws** — slice B,
/// end to end on the product path.
///
/// `motion.look_at`'s two target ports are fed by `value.instance_field`, which
/// has no kernel, so the plan's frontier BRANCHES: two boundaries, with the grid
/// and the look_at still claimed behind them. Until the pump went plural the
/// shell forfeited the GPU for exactly this document.
///
/// It drives the REAL `MotionCookPump` rather than calling `Cook` twice by hand,
/// because the thing being tested is the pump's plural hand-off: the march, the
/// dedupe, and the labelling of each stream with its node. A gate that cooked
/// the two boundaries itself would pass with the pump still singular.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn two_cpu_seams_hand_over_in_one_march_and_match_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 10.0);
    let la = g.add_node("motion.look_at");
    let out = g.add_node("motion.output");
    g.set_param(la, "offset", 17.0);
    connect(&mut g, grid, la);
    connect(&mut g, la, out);
    // Two uncovered VALUE producers, one per target port, with DIFFERENT modes so
    // the two targets never coincide — equal targets would survive the two
    // streams being swapped or one of them being handed over twice.
    let mut fields = Vec::new();
    for (port, attr) in [(1u16, "Index"), (2u16, "Count")] {
        let f = uncoverable_value_node(&mut g, attr);
        connect(&mut g, grid, f);
        g.connect(Edge {
            from: (f, 0),
            to: (la, port),
            delayed: false,
        })
        .expect("target port");
        fields.push(f);
    }
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(
        plan.boundaries,
        vec![(fields[0], 0), (fields[1], 0)],
        "the frontier branches: one seam per uncovered target port"
    );
    assert!(
        plan.stages.iter().any(|s| s.node == la),
        "and the look_at is still claimed BEHIND those seams — otherwise this \
         gate would be proving that two seams cost nothing because nothing ran"
    );

    // The CPU's answer, whole, on the canonical path.
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

    // The product path: ONE march over the whole boundary set.
    let mut pump = ph2d_eval_motion::MotionCookPump::new();
    let nodes: Vec<_> = plan.boundaries.iter().map(|(n, _)| *n).collect();
    pump.advance_or_scrub_to_nodes_scoped(
        &g,
        &reg,
        &nodes,
        0,
        |_| PLAYHEAD,
        &ph2d_nodegraph::cook::TimeScopes::default(),
    );
    let handed: Vec<_> = pump
        .boundary_streams()
        .iter()
        .map(|(n, s)| (*n, s))
        .collect();
    assert_eq!(handed.len(), 2, "both seams handed over in that one march");

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let n = gc
        .cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &handed,
            CookClock::at(PLAYHEAD),
            DEFAULT_UV,
            DEFAULT_SIZE,
        )
        .expect("gpu cook");
    assert_eq!(n as usize, cpu.len(), "same instance count");

    let gpu_out = ph2d_gpu_cook::read_instances(&gpu, gc.instances().expect("cooked"));
    assert_parity(&cpu, &gpu_out);
    // Fixture check: the aim must VARY across the field, or a hand-off that
    // delivered the SAME stream to both ports would pass. `look_at` writes `rot`,
    // which the lowering folds into `basis` — so the basis is where a constant
    // rotation would show up as every element sharing one.
    assert!(
        cpu.iter()
            .any(|i| (i.basis[0] - cpu[0].basis[0]).abs() > 1e-3),
        "fixture check: the elements must aim in varied directions"
    );
}

/// **Does a two-seam hybrid actually PAY?** — the number slice B shipped without.
///
/// The route now sends a plan with N CPU boundaries to the GPU whenever the
/// suffix has one dispatching stage. That rule was inherited from the one-seam
/// case and never re-measured, and the arithmetic is not obviously in its favour:
/// the CPU still cooks the whole prefix (here `grid` plus TWO
/// `value.instance_field`s), then **uploads two streams**, so the GPU only earns
/// back the `look_at` — one `atan2` per element. Upload is bandwidth; `atan2` is
/// arithmetic. Which wins is a measurement, not an opinion.
///
/// **Measured on the RTX (2026-07-18), and the answer is yes — but the obvious
/// reading of it is wrong.**
///
/// ```text
///   elements | pure CPU | hybrid+sync | hybrid CPU-side | prefix
///       1024 |   0.006  |    0.058    |     0.017       | 0.001
///       4096 |   0.022  |    0.060    |     0.020       | 0.002
///       8100 |   0.043  |    0.066    |     0.023       | 0.004
///      16384 |   0.082  |    0.089    |     0.034       | 0.008
///      32761 |   0.174  |    0.131    |     0.052       | 0.019
///      65536 |   0.268  |    0.205    |     0.089       | 0.038
///     524176 |   3.727  |    1.387    |     0.544       | 0.266
///    2002225 |  22.205  |    6.972    |     3.609       | 2.527   (ms)
/// ```
///
/// `hybrid+sync` waits for the device every frame, and read that way the hybrid
/// looks **3× SLOWER below 16k** — which would argue for a size floor on the
/// route. It is the wrong column: **the product never waits.** The shell submits
/// and moves on, so what competes for the frame budget is `hybrid CPU-side`, and
/// by that measure the hybrid is cheaper from ~4k up (dead even there, 1,5× at
/// 8k, **5,9× at 2M**) and costs at most **0,012 ms** more below it — 0,07% of a
/// 60 fps frame.
///
/// So **no threshold was added**: a limit written to dodge 0,012 ms is the
/// "for safety" guess CLAUDE.md §0.0 refuses. The measurement that would justify
/// one is the sync column, and that column is an artefact of this probe.
///
/// **What it does argue for is COVERAGE.** At 2M, 2,527 ms of the hybrid's
/// 3,609 ms CPU-side is the CPU PREFIX — **70% of the cost is the seam's own
/// tax**, paid to cook the uncovered nodes feeding the target ports. The first
/// run of this probe used `value.instance_field` there, and covering that node
/// (same session) deleted the seams outright: the fixture now uses
/// `value.attribute`, which is uncoverable by structure. Every kernel shortens
/// this prefix; enough of them and the plan has no seam left to pay for.
///
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity --release -- --ignored --nocapture two_seam_hybrid_timing
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn two_seam_hybrid_timing() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    eprintln!("  elements |  pure CPU | hybrid+sync | hybrid CPU-side | prefix");
    for rows in [32.0f32, 64.0, 90.0, 128.0, 181.0, 256.0, 724.0, 1415.0] {
        let mut g = Graph::new();
        let grid = grid_node(&mut g, rows);
        let la = g.add_node("motion.look_at");
        let out = g.add_node("motion.output");
        connect(&mut g, grid, la);
        connect(&mut g, la, out);
        for (port, attr) in [(1u16, "Index"), (2u16, "Count")] {
            let f = uncoverable_value_node(&mut g, attr);
            connect(&mut g, grid, f);
            g.connect(Edge {
                from: (f, 0),
                to: (la, port),
                delayed: false,
            })
            .expect("target port");
        }
        g.validate(&reg).expect("well-typed");
        let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
        assert_eq!(plan.boundaries.len(), 2, "the two-seam shape");

        let n_el = (rows * rows) as usize;
        let frames = if n_el > 500_000 { 20 } else { 60 };

        // Pure CPU, steady state (a persistent `Cook` — the memo is the product's).
        let mut cook = Cook::new();
        let mut cpu = Vec::new();
        let mut cpu_ms = f64::MAX;
        for f in 0..frames {
            let t = 0.5 + f as f64 * 1e-3; // move the playhead: no memo hit
            let t0 = std::time::Instant::now();
            ph2d_eval_motion::evaluate_motion_into(
                &mut cook,
                &g,
                &reg,
                out,
                t,
                DEFAULT_UV,
                DEFAULT_SIZE,
                &mut cpu,
            )
            .expect("cpu cook");
            cpu_ms = cpu_ms.min(t0.elapsed().as_secs_f64() * 1e3);
        }

        // The hybrid, on the product path: one march over both boundaries, then
        // the GPU suffix. Warm up first (pipeline compile + pool).
        let mut pump = ph2d_eval_motion::MotionCookPump::new();
        let mut gc = ph2d_gpu_cook::GpuCook::new();
        let nodes: Vec<_> = plan.boundaries.iter().map(|(n, _)| *n).collect();
        let scopes = ph2d_nodegraph::cook::TimeScopes::default();
        let mut hybrid_ms = f64::MAX;
        let mut march_ms = f64::MAX;
        let mut cpu_side_ms = f64::MAX;
        for f in 0..=frames {
            let t = 0.5 + f as f64 * 1e-3;
            let t0 = std::time::Instant::now();
            pump.advance_or_scrub_to_nodes_scoped(&g, &reg, &nodes, f as u64, |_| t, &scopes);
            let handed: Vec<_> = pump
                .boundary_streams()
                .iter()
                .map(|(n, s)| (*n, s))
                .collect();
            let t_march = t0.elapsed().as_secs_f64() * 1e3;
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &handed,
                CookClock::at(t),
                DEFAULT_UV,
                DEFAULT_SIZE,
            )
            .expect("gpu cook");
            // What the frame actually SPENDS: the shell submits and moves on, so
            // the device wait below is the probe's, not the product's.
            let submitted = t0.elapsed().as_secs_f64() * 1e3;
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            let total = t0.elapsed().as_secs_f64() * 1e3;
            if f > 0 {
                // Skip frame 0: it compiles pipelines and grows the pool.
                hybrid_ms = hybrid_ms.min(total);
                march_ms = march_ms.min(t_march);
                cpu_side_ms = cpu_side_ms.min(submitted);
            }
        }
        eprintln!(
            "{n_el:>10} | {cpu_ms:>8.3}ms | {hybrid_ms:>9.3}ms | {cpu_side_ms:>13.3}ms | {march_ms:>6.3}ms",
        );
    }
}

/// **`value.instance_field` — where per-element variation is BORN**, and the node
/// the two-seam timing probe pointed at: 71% of that hybrid's CPU cost was the
/// prefix cooked to feed two of these.
///
/// All three modes in one gate, because they are three different KINDS of answer:
/// `Index` is the raw ordinal (an `f32(i)` cast), `Ramp` divides by `N−1` (the
/// degenerate `N=1` guard), and `Random` is an integer hash — where a `u32` that
/// wrapped differently, or a logical vs arithmetic shift, would give a completely
/// different field rather than an ε.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn instance_field_kernel_matches_the_cpu_in_every_mode() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // The last two rows are HALF-INTEGERS, and they are the point of the sweep.
    // Rust's `f32::round` is half-AWAY-from-zero and WGSL's `round` is half-EVEN,
    // so `mode 0.5` is Ramp on one side and Index on the other (a different field
    // entirely), and `seed 6.5` seeds 7 vs 6. A sweep of 0.0/1.0/2.0 proves
    // nothing about rounding at all: both conventions agree on every integer, and
    // a mutation swapping them SURVIVED such a fixture here
    // ([[feedback_a_green_gate_may_be_green_by_accident]]).
    for (mode, seed, label) in [
        (0.0, 7.4, "Index"),
        (1.0, 7.4, "Ramp"),
        (2.0, 7.4, "Random"),
        (0.5, 7.4, "mode on a half-integer"),
        (2.0, 6.5, "seed on a half-integer"),
    ] {
        let mut g = Graph::new();
        let grid = grid_node(&mut g, 12.0);
        let f = g.add_node("value.instance_field");
        g.set_param(f, "mode", mode);
        g.set_param(f, "seed", seed);
        connect(&mut g, grid, f);
        g.validate(&reg).expect("well-typed");

        let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, f);
        assert!(plan.is_fully_gpu(), "{label}: covered end to end");

        let mut cook = Cook::new();
        let cpu = cook.cook(&g, &reg, f, PLAYHEAD).expect("cpu cook");
        let cpu_v = match cpu[0].as_stream().get("v") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("{label}: the CPU emitted no `v`"),
        };

        let mut gc = ph2d_gpu_cook::GpuCook::new();
        gc.retain_streams_for_debug(true);
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

        let gpu_v = gc.read_column(&gpu, f, "v").expect("`v` reads back");
        assert_eq!(gpu_v.len(), cpu_v.len(), "{label}: same element count");
        let worst = gpu_v
            .iter()
            .zip(&cpu_v)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        eprintln!("instance_field {label}: max |dv| = {worst:e}");
        // Random is an INTEGER hash on both sides, so it must be EXACT — an ε bar
        // there would hide a wrap or a shift that differs.
        let bar = if mode >= 1.5 { 0.0 } else { 1e-6 };
        assert!(worst <= bar, "{label}: max |dv| = {worst:e} (bar {bar:e})");
        assert!(
            cpu_v.iter().any(|v| (v - cpu_v[0]).abs() > 1e-6),
            "{label}: fixture check — the field must VARY, or this proves nothing"
        );
    }
}

/// Did the driven column come out as anything other than all zeros? A row where
/// both paths produce zero agrees perfectly and tests nothing — which is what
/// `rot × Multiply` did before the sweep seeded a base.
fn column_is_nonzero(cpu: &ph2d_nodegraph::attr::Stream, column: &str) -> bool {
    use ph2d_nodegraph::attr::Column;
    match cpu.get(column) {
        Some(Column::Scalar(v)) => v.iter().any(|x| x.abs() > 1e-6),
        Some(Column::Vec2(v)) => v.iter().any(|x| x[0].abs() > 1e-6 || x[1].abs() > 1e-6),
        Some(Column::Vec4(v)) => v.iter().any(|x| x.iter().any(|k| k.abs() > 1e-6)),
        _ => false,
    }
}

/// Compare one column of a staged node against the CPU's stream, whatever its
/// width — the readers are typed, so a gate that needs to follow a PARAM to its
/// column would otherwise have to branch on the type at every call site.
fn compare_column(
    gpu: &ph2d_gpu::GpuContext,
    gc: &ph2d_gpu_cook::GpuCook,
    node: NodeId,
    cpu: &ph2d_nodegraph::attr::Stream,
    column: &str,
) -> f32 {
    use ph2d_nodegraph::attr::Column;
    match cpu.get(column) {
        Some(Column::Scalar(c)) => {
            let g = gc
                .read_column(gpu, node, column)
                .expect("column reads back");
            assert_eq!(g.len(), c.len(), "{column}: element count");
            g.iter()
                .zip(c)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f32::max)
        }
        Some(Column::Vec2(c)) => {
            let g = gc
                .read_column_vec2(gpu, node, column)
                .expect("column reads back");
            assert_eq!(g.len(), c.len(), "{column}: element count");
            g.iter()
                .zip(c)
                .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
                .fold(0.0, f32::max)
        }
        Some(Column::Vec4(c)) => {
            let g = gc
                .read_column_vec4(gpu, node, column)
                .expect("column reads back");
            assert_eq!(g.len(), c.len(), "{column}: element count");
            g.iter()
                .zip(c)
                .map(|(a, b)| (0..4).map(|k| (a[k] - b[k]).abs()).fold(0.0, f32::max))
                .fold(0.0, f32::max)
        }
        _ => panic!("the CPU emitted no `{column}`"),
    }
}

/// **`motion.drive` — the value domain's WRITE side**, and the first chain where
/// a value graph reaches the SCREEN entirely on the device: `grid →
/// value.instance_field → motion.drive → output`, no CPU seam anywhere.
///
/// Sweeps both channels × all three combine modes, and both value LENGTHS — a
/// broadcast (one `value.lfo`, held across the field) and a per-element field.
/// The broadcast half is the one that matters: `read(i)` and `read(0)` agree on a
/// length-N field, so a fixture without the length-1 case would stay green with
/// the broadcast dead.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn drive_kernel_matches_the_cpu_across_channels_modes_and_both_value_lengths() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for channel in [0.0f32, 1.0, 2.0, 3.0, 4.0] {
        for mode in [0.0f32, 1.0, 2.0] {
            for (per_element, masked) in
                [(false, false), (true, false), (false, true), (true, true)]
            {
                let mut g = Graph::new();
                let grid = grid_node(&mut g, 11.0);
                // `masked` inserts a real `falloff` column. The grid emits none, so
                // WITHOUT this every fixture reads the binding's identity (1.0),
                // the blend is a no-op, and deleting the falloff term entirely is
                // invisible — a mutation doing exactly that SURVIVED here
                // ([[reference_topic_fixture_discipline]]).
                let src = if masked {
                    let f = g.add_node("motion.falloff");
                    g.set_param(f, "radius", 3.1);
                    g.set_param(f, "curve", 1.7);
                    connect(&mut g, grid, f);
                    f
                } else {
                    grid
                };
                // Per-element: a length-N field off the grid. Broadcast: a bare
                // `value.lfo`, which the count law sizes at ONE.
                let v = if per_element {
                    let f = g.add_node("value.instance_field");
                    g.set_param(f, "mode", 1.0); // Ramp: varies across the field
                    connect(&mut g, src, f);
                    f
                } else {
                    let l = g.add_node("value.lfo");
                    g.set_param(l, "period", 0.83);
                    g.set_param(l, "amplitude", 2.7);
                    l
                };
                // A SEED drive first, so the channel under test starts from a
                // non-trivial base. Without it `rot` starts at its identity 0 and
                // Multiply yields zero on both sides — agreement that proves
                // nothing, in 1 of every 3 rows.
                let seed = g.add_node("motion.drive");
                g.set_param(seed, "channel", channel);
                g.set_param(seed, "mode", 1.0); // Set
                g.set_param(seed, "scale", 0.9);
                connect(&mut g, src, seed);
                g.connect(Edge {
                    from: (v, 0),
                    to: (seed, 1),
                    delayed: false,
                })
                .expect("seed value port");

                let d = g.add_node("motion.drive");
                g.set_param(d, "channel", channel);
                g.set_param(d, "mode", mode);
                g.set_param(d, "scale", 1.7);
                connect(&mut g, seed, d);
                g.connect(Edge {
                    from: (v, 0),
                    to: (d, 1),
                    delayed: false,
                })
                .expect("value port");
                g.validate(&reg).expect("well-typed");

                let label = format!(
                    "ch{channel} mode{mode} {}{}",
                    if per_element { "field" } else { "broadcast" },
                    if masked { " +falloff" } else { "" }
                );
                let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, d);
                assert!(plan.is_fully_gpu(), "{label}: covered end to end");

                let mut cook = Cook::new();
                let cpu = cook.cook(&g, &reg, d, PLAYHEAD).expect("cpu cook");
                let mut gc = ph2d_gpu_cook::GpuCook::new();
                gc.retain_streams_for_debug(true);
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

                // Column SET too: the drive rewrites one channel and copies the
                // rest, so a kernel that materialised `rot`/`size`/`tint` would
                // hand downstream a different stream shape (which is why the
                // kernel claims only the two channels that write `P`).
                let gpu_cols: Vec<&str> = gc
                    .node_columns(d)
                    .expect("staged")
                    .iter()
                    .map(String::as_str)
                    .collect();
                let mut cpu_cols: Vec<&str> = cpu[0]
                    .as_stream()
                    .columns()
                    .map(|(n, _)| n.as_str())
                    .collect();
                cpu_cols.sort_unstable();
                assert_eq!(gpu_cols, cpu_cols, "{label}: same column SET");

                // Compare the column the CHANNEL writes — the whole point of the
                // variants is that it differs, so a gate hard-wired to `P` would
                // pass on three channels by never looking at their output.
                let col = match channel as i32 {
                    0 | 1 => "P",
                    2 => "rot",
                    4 => "tint",
                    _ => "size",
                };
                let worst = compare_column(&gpu, &gc, d, cpu[0].as_stream(), col);
                eprintln!("drive {label}: col {col}, max |d| = {worst:e}");
                assert!(worst < 1e-5, "{label}: col {col}, max |d| = {worst:e}");
                assert!(
                    column_is_nonzero(cpu[0].as_stream(), col),
                    "{label}: fixture check — `{col}` came out all zeros, so this \
                     row compared nothing to nothing"
                );
            }
        }
    }
}

/// **`value.noise` on the device — the coherent-noise producer, fBm and all.**
/// `grid → value.noise → motion.drive(Y) → output`, no CPU seam. The noise's
/// lattice hash + fade are byte-mirrors of `motion.wiggle` (whose kernel already
/// has a device parity gate), so what THIS test adds is the piece wiggle never
/// exercises: the **fBm octave loop** (`octaves = 3`, so three layers sum and
/// normalize on the GPU) reaching the screen through a value→drive chain.
///
/// Non-round params so a unit slip can't hide behind a tidy number; the integer
/// hash must land bit-exact per lattice cell or the offset diverges by
/// `O(amplitude)`, far outside ε. `is_fully_gpu` PROVES the chain dispatches — a
/// silent CPU fallback would compare CPU to CPU and stay green with the kernel
/// dead.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_noise_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 11.0);
    let vn = g.add_node("value.noise");
    g.set_param(vn, "frequency", 0.23);
    g.set_param(vn, "speed", 0.7);
    g.set_param(vn, "octaves", 3.0); // the fBm loop wiggle never runs
    g.set_param(vn, "roughness", 0.55);
    g.set_param(vn, "amplitude", 1.9);
    g.set_param(vn, "seed", 4.0);
    connect(&mut g, grid, vn); // read the grid for its count
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 1.3);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (vn, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("noise into drive's value port");

    // A value producer emits `v`, not P — so the geometry rides `grid → drive`
    // while `value.noise` feeds drive's value port; the drive node is the sink
    // (mirrors the `drive_kernel_matches...` chain).
    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "value.noise → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.noise → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the noise drove nothing"
    );
}

/// **`value.mix` on the device — the crossfader, with `t` DRIVEN by a port.**
/// `grid → {lfo→a, noise→b, instance_field(Ramp)→t} → mix → drive(Y)`, no CPU
/// seam. The `factor` param is a decoy `0.9`; the connected `t` port must win on
/// BOTH paths, so this is the device proof of the `HAS_t_v` presence choice
/// (`select(factor, port, HAS_t_v)`) — a regression to `false` would read `0.9`
/// on the GPU while the CPU reads the ramp, diverging by `O(amplitude)`, far
/// outside ε. `is_fully_gpu` PROVES the chain dispatches (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_mix_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 11.0);
    // a: a clean LFO; b: coherent noise; t: a per-element ramp (drives the fade).
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.71);
    g.set_param(lfo, "amplitude", 2.3);
    connect(&mut g, grid, lfo);
    let noise = g.add_node("value.noise");
    g.set_param(noise, "frequency", 0.19);
    g.set_param(noise, "amplitude", 2.3);
    g.set_param(noise, "seed", 5.0);
    connect(&mut g, grid, noise);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp: t varies per element in [0,1]
    connect(&mut g, grid, field);
    let mix = g.add_node("value.mix");
    g.set_param(mix, "factor", 0.9); // a decoy — the connected `t` must win
    g.connect(Edge {
        from: (lfo, 0),
        to: (mix, 0),
        delayed: false,
    })
    .expect("a");
    g.connect(Edge {
        from: (noise, 0),
        to: (mix, 1),
        delayed: false,
    })
    .expect("b");
    g.connect(Edge {
        from: (field, 0),
        to: (mix, 2),
        delayed: false,
    })
    .expect("t");
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 1.2);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (mix, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("mixed value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "lfo/noise/mix → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.mix → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the crossfade drove nothing"
    );
}

/// **`value.quantize` on the device — the staircase.** `grid → lfo →
/// value.quantize(Floor) → drive(Y)`, no CPU seam. Floor is the mode where a
/// CPU↔GPU `round` mismatch would show worst (half-to-even vs half-away-from-zero
/// diverge exactly at grid midpoints), and a NON-round `step` (`0.37`) means the
/// snapped levels are irrational-ish — a `/`-then-`×` slip can't hide behind a
/// tidy multiple. `is_fully_gpu` PROVES the chain dispatches (no silent fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_quantize_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 11.0);
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.83);
    g.set_param(lfo, "amplitude", 2.6);
    g.set_param(lfo, "phase_stagger", 0.11); // a per-element travelling wave
    connect(&mut g, grid, lfo);
    let quant = g.add_node("value.quantize");
    g.set_param(quant, "step", 0.37); // non-round grid
    g.set_param(quant, "mode", 1.0); // Floor
    connect(&mut g, lfo, quant);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 1.4);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (quant, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("quantized value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "lfo → quantize → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.quantize → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the staircase drove nothing"
    );
}

/// **`value.gain` runs fully on the GPU and matches the CPU.** A `[0,1]` ramp
/// (instance_field Ramp) through a Gain S-curve at `strength 0.6` — the real
/// division path (`1/a - 2`, the two Schlick rationals), NOT the `strength = 0`
/// neutral identity, which would hide a wrong port behind a passthrough — drives
/// Y; the device result matches the CPU port term for term. `is_fully_gpu` PROVES
/// the chain dispatches (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_gain_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    connect(&mut g, grid, field);
    let gain = g.add_node("value.gain");
    g.set_param(gain, "strength", 0.6); // a real S-curve, not the neutral identity
    g.set_param(gain, "mode", 0.0); // Gain
    connect(&mut g, field, gain);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (gain, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("gained value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "field → gain → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.gain → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the S-curve drove nothing"
    );
}

/// **`value.step` runs fully on the GPU and matches the CPU.** A `[0,1]` ramp
/// (instance_field Ramp) through a Smooth gate at `threshold 0.5, width 0.4` — the
/// real division/Hermite path (`(x-lo)/(hi-lo)`, `3t²−2t³`), NOT the Hard branch
/// (a trivial `select`), which would hide a wrong band behind a comparison —
/// drives Y; the device result matches the CPU port. `is_fully_gpu` PROVES the
/// chain dispatches (no silent CPU fallback). The ramp crosses the band, so the
/// output is neither all-0 nor all-1 (`column_is_nonzero`).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_step_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    connect(&mut g, grid, field);
    let step = g.add_node("value.step");
    g.set_param(step, "threshold", 0.5);
    g.set_param(step, "width", 0.4); // a real smooth band, not the Hard select
    g.set_param(step, "mode", 1.0); // Smooth
    connect(&mut g, field, step);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (step, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("gated value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "field → step → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.step → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the gate drove nothing"
    );
}

/// **`value.normalize` runs fully on the GPU and matches the CPU.** This is the
/// value domain's first `reduce → broadcast → map` — the field's `min` and `max`
/// (two whole-stream reductions) fed into `(v − min)/(max − min)`. The fixture
/// gives the reduction REAL work: a `[0,1]` ramp is stretched to `[−3, 5]` by a
/// `value.map_range` FIRST (so `min = −3`, `max = 5`, not the trivial `0`/`1`),
/// then Range-normalized back and driven to Y. The device tree reduction matches
/// the CPU fold (`Min`/`Max` are bit-exact in any order) and the map matches term
/// for term. `is_fully_gpu` PROVES the reduce pass AND the kernel dispatch on the
/// device (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_normalize_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    connect(&mut g, grid, field);
    let map = g.add_node("value.map_range");
    g.set_param(map, "out_lo", -3.0); // stretch to a non-trivial range so the
    g.set_param(map, "out_hi", 5.0); // reduction has real min/max to discover
    connect(&mut g, field, map);
    let norm = g.add_node("value.normalize");
    g.set_param(norm, "mode", 0.0); // Range → [0,1]
    connect(&mut g, map, norm);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (norm, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("normalized value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "field → map → normalize → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.normalize → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the normalize drove nothing"
    );
}

/// **`value.unary` runs fully on the GPU and matches the CPU.** A ramp stretched
/// to `[1, 5]` through the **Reciprocal** op — the real division path (`1/x`, the
/// guarded one), NOT a trivial `abs`/`negate` — drives Y; the device result
/// matches the CPU port. The range is all-positive so the `x == 0` guard is not
/// hit (the unit tests cover it); the division is what a wrong port would get
/// wrong. `is_fully_gpu` PROVES the chain dispatches (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_unary_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    connect(&mut g, grid, field);
    let map = g.add_node("value.map_range");
    g.set_param(map, "out_lo", 1.0); // [1, 5], all positive: real reciprocal
    g.set_param(map, "out_hi", 5.0);
    connect(&mut g, field, map);
    let unary = g.add_node("value.unary");
    g.set_param(unary, "op", 7.0); // Reciprocal
    connect(&mut g, map, unary);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (unary, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("unary value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "field → map → unary → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.unary → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the op drove nothing"
    );
}

/// **`value.reduce` runs fully on the GPU and matches the CPU.** This is the
/// value domain's `reduce → broadcast`: the field's aggregate written to every
/// element. The fixture uses **Mean** — the mode that exercises the MOST channel:
/// the `Sum` reduction (the ε one) AND the `count = Σ 1.0` reduction (the denominator).
/// A ramp stretched to `[1, 5]` (mean `3`) is reduced and driven to Y; the device
/// tree reductions match the CPU fold within ε and the broadcast writes the same
/// value to all. `is_fully_gpu` PROVES the four reduce passes AND the kernel
/// dispatch on the device (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_reduce_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    connect(&mut g, grid, field);
    let map = g.add_node("value.map_range");
    g.set_param(map, "out_lo", 1.0); // ramp [1, 5]: mean 3, a non-trivial sum
    g.set_param(map, "out_hi", 5.0);
    connect(&mut g, field, map);
    let reduce = g.add_node("value.reduce");
    g.set_param(reduce, "mode", 1.0); // Mean (Sum + count)
    connect(&mut g, map, reduce);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (reduce, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("reduced value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "field → map → reduce → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.reduce → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the reduce drove nothing"
    );
}

/// **`value.smooth` runs fully on the GPU and matches the CPU.** Unlike the other
/// value kernels, element `i` reads its NEIGHBOURS (`v[i−r]…v[i+r]` off `in_v`),
/// so this exercises the neighbour-reading loop and the edge clamp. A JAGGED field
/// — `instance_field` Random — is box-blurred at radius 3 and driven to Y; the
/// window sum runs left to right on both paths, so the device matches the CPU. The
/// Random source gives the smooth real work (a ramp would be near-identity).
/// `is_fully_gpu` PROVES the chain dispatches (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_smooth_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 2.0); // Random: a jagged per-instance field to smooth
    g.set_param(field, "seed", 7.0);
    connect(&mut g, grid, field);
    let smooth = g.add_node("value.smooth");
    g.set_param(smooth, "radius", 3.0); // a real window, reading neighbours
    connect(&mut g, field, smooth);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (smooth, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("smoothed value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "field → smooth → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.smooth → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the smooth drove nothing"
    );
}

/// **`value.pattern` runs fully on the GPU and matches the CPU.** A PRODUCER: it
/// reads the grid for its count and writes `pattern[i mod steps]` from the param
/// slots — the `switch` over the eight values and the index modulo. `steps = 4`
/// with four distinct values repeats across the 24 instances and drives Y; the
/// device selection matches the CPU (a pure param passthrough, exact). `steps` is
/// deliberately < the slot count so the cycle is visible. `is_fully_gpu` PROVES the
/// chain dispatches (no silent CPU fallback).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_pattern_kernel_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 24.0);
    let pattern = g.add_node("value.pattern");
    g.set_param(pattern, "steps", 4.0);
    g.set_param(pattern, "v0", 0.2);
    g.set_param(pattern, "v1", 0.8);
    g.set_param(pattern, "v2", 0.5);
    g.set_param(pattern, "v3", 1.0);
    connect(&mut g, grid, pattern); // read for its count
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive); // geometry into `in`
    g.connect(Edge {
        from: (pattern, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("pattern value into drive");

    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, drive);
    assert!(plan.is_fully_gpu(), "grid → pattern → drive claimed end to end");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, drive, PLAYHEAD).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
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
    let worst = compare_column(&gpu, &gc, drive, cpu[0].as_stream(), "P");
    eprintln!("value.pattern → drive(Y): col P, max |d| = {worst:e}");
    assert!(worst < 1e-4, "col P, max |d| = {worst:e}");
    assert!(
        column_is_nonzero(cpu[0].as_stream(), "P"),
        "fixture check — the pattern drove nothing"
    );
}

/// `motion.orbit` — swings each element around a pivot by an angle the playhead
/// advances. The rotation is ABSOLUTE (the pristine `P` turned by
/// `angle + playhead·speed`), never cumulative, which is what lets the
/// transcendental-free parabolic sine stand in for real trig: its ~0.09% error
/// wobbles the radius sub-pixel and cannot accumulate.
///
/// The pivot is deliberately OFF the grid's centre and the angle is not a
/// multiple of 90°: a pivot at the origin with a right angle would make the
/// rotation a coordinate swap, which a transposed matrix would survive.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn orbit_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (node, out) = deformer_chain(&mut g, 160.0, "motion.orbit");
    g.set_param(node, "pivot_x", 0.83);
    g.set_param(node, "pivot_y", -1.37);
    g.set_param(node, "angle", 23.5);
    g.set_param(node, "speed", 47.0);
    assert_gpu_parity(&gpu, &reg, &g, out, 2);
}

/// `motion.pin_constraint` — writes `inv_mass`, the column an integrator reads to
/// decide how much a force may move each element (0 = nailed down).
///
/// **This one cannot use `assert_gpu_parity`.** That helper compares the LOWERED
/// instances, and `inv_mass` is not lowered — it is sim input, not a render
/// attribute. A parity gate built the usual way would compare two identical
/// pictures and stay green with the kernel doing nothing at all, so this reads the
/// column back and compares it directly.
///
/// `motion.falloff` sits upstream so the weight VARIES across the selection: with
/// a flat falloff every pinned element would get the same `inv_mass` and an index
/// slip inside the range would be invisible.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn pin_constraint_kernel_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = grid_node(&mut g, 12.0);
    let fall = g.add_node("motion.falloff");
    let pin = g.add_node("motion.pin_constraint");
    // A range that starts off zero and ends before the last element, so BOTH
    // edges of the mask are exercised, and a non-round strength.
    g.set_param(pin, "first", 17.0);
    g.set_param(pin, "count", 53.0);
    g.set_param(pin, "strength", 0.71);
    connect(&mut g, grid, fall);
    connect(&mut g, fall, pin);
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, pin);
    assert!(plan.is_fully_gpu(), "grid → falloff → pin is covered");

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, pin, PLAYHEAD).expect("cpu cook");
    let cpu_w = match cpu[0].as_stream().get("inv_mass") {
        Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
        _ => panic!("the CPU wrote no `inv_mass`"),
    };

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true); // gate-only, so `read_column` can see it
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
    let gpu_w = gc
        .read_column(&gpu, pin, "inv_mass")
        .expect("`inv_mass` reads back");

    // The fixture has to CONTAIN the phenomenon: pinned elements and free ones,
    // and a varying weight among the pinned. Otherwise this proves nothing.
    assert!(
        cpu_w.iter().any(|w| *w > 0.99),
        "some elements must be free"
    );
    let pinned: Vec<f32> = cpu_w.iter().copied().filter(|w| *w < 0.99).collect();
    assert!(pinned.len() > 2, "and several must be pinned");
    assert!(
        pinned.iter().any(|w| (w - pinned[0]).abs() > 1e-6),
        "the pin weight must VARY across the selection, or an index slip hides"
    );

    assert_eq!(gpu_w.len(), cpu_w.len());
    let worst = gpu_w
        .iter()
        .zip(&cpu_w)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    eprintln!(
        "inv_mass parity: {} elements, max |dw| = {worst:e}",
        cpu_w.len()
    );
    assert!(worst < 1e-6, "inv_mass parity: max |dw| = {worst:e}");
}

/// `motion.stagger` — offsets a channel by the element's POSITION in the stream
/// (`i/(n−1)`), shaped by an easing curve. One of the few kernels that genuinely
/// needs the engine's own `params.count`: the ramp is a function of the stream's
/// length, not of the element's columns.
///
/// **Every curve family is a BRANCH**, so the fixture walks all eight of them in
/// all three directions — a gate on the default (Linear) alone would prove one
/// twenty-fourth of this kernel, and Linear is precisely the branch that does
/// nothing ([[reference_topic_fixture_discipline]]). Bounce and Circ matter most:
/// Bounce is four piecewise parabolas and Circ is the only `sqrt`.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn stagger_kernel_matches_the_cpu_across_every_easing() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for curve in 0..=7 {
        for dir in 0..=2 {
            let mut g = Graph::new();
            let (node, out) = deformer_chain(&mut g, 40.0, "motion.stagger");
            g.set_param(node, "channel", 1.0);
            g.set_param(node, "min", -0.83);
            g.set_param(node, "max", 2.17);
            g.set_param(node, "ease_curve", curve as f32);
            g.set_param(node, "ease_dir", dir as f32);
            // Reverse on the odd curves, so the ramp flip is exercised too.
            g.set_param(node, "reverse", (curve % 2) as f32);
            assert_gpu_parity(&gpu, &reg, &g, out, 2);
        }
    }

    // A SINGLE-element stream: the ramp is `i/(n−1)`, so `n = 1` is a divide by
    // zero and the CPU guards it with `n <= 1 → 0.0`. The grid above never gets
    // near it, and a mutation that deleted the guard SURVIVED this gate until
    // this case existed ([[reference_topic_fixture_discipline]] — a fixture only
    // proves what it contains, and the degenerate end is where guards live).
    let mut g = Graph::new();
    let (node, out) = deformer_chain(&mut g, 1.0, "motion.stagger");
    g.set_param(node, "channel", 1.0);
    g.set_param(node, "min", -0.83);
    g.set_param(node, "max", 2.17);
    g.set_param(node, "ease_curve", 2.0);
    assert_gpu_parity(&gpu, &reg, &g, out, 2);
}

// ── The value-domain COMBINER and ROUTER ────────────────────────────────────
//
// Both arrive with the engine's **third count law** (`max` over every input
// port), and the law and the kernels land together on purpose: an engine
// mechanism with no consumer was already built and reverted once on this line.

/// A length-1 VALUE field of exactly `v` — an lfo with zero amplitude, so
/// `waveform(..)·0 + offset` is `offset` to the bit on both paths.
///
/// A constant is what these gates need (a routing mistake has to be legible as a
/// jump between named numbers, not as a small numeric drift), and this is the
/// only VALUE producer that can be pinned to one. Each gate asserts the CPU
/// actually produced the constant, so the trick failing would be loud rather
/// than quietly turning the fixture into noise.
fn const_field(g: &mut Graph, v: f32) -> NodeId {
    let n = g.add_node("value.lfo");
    g.set_param(n, "amplitude", 0.0);
    g.set_param(n, "offset", v);
    n
}

/// A length-N VALUE field that varies per element: a sawtooth swept by
/// `phase_stagger`, remapped into `offset ± amplitude`.
fn ramp_field(g: &mut Graph, src: NodeId, amplitude: f32, offset: f32) -> NodeId {
    let n = g.add_node("value.lfo");
    connect(g, src, n);
    g.set_param(n, "wave", 3.0); // sawtooth: 2f − 1, a clean sweep
    g.set_param(n, "period", 1.0);
    g.set_param(n, "amplitude", amplitude);
    g.set_param(n, "offset", offset);
    g.set_param(n, "phase_stagger", 0.017);
    n
}

/// Cook `sink` on both paths and return `(cpu stream, gpu cook)`, having first
/// asserted the plan claims the chain whole.
fn cook_both(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    g: &Graph,
    sink: NodeId,
) -> (ph2d_nodegraph::attr::Stream, ph2d_gpu_cook::GpuCook) {
    g.validate(reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(g, reg, reg, sink);
    assert!(
        plan.is_fully_gpu(),
        "the chain must be claimed whole: {:?}",
        plan.boundaries
    );
    let mut cook = Cook::new();
    let cpu = cook.cook(g, reg, sink, PLAYHEAD).expect("cpu cook");
    let cpu_stream = cpu[0].as_stream().clone();

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
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
    (cpu_stream, gc)
}

/// **`value.math` matches the CPU in every op — and is as wide as its WIDEST
/// input, whichever port that is.**
///
/// The port order is swept, and that sweep is the gate. The engine's default law
/// is *"as wide as port 0"*, so with the length-N field on `a` it gets the right
/// answer for the wrong reason and only the `b` ordering can tell. The node's
/// headline use — `value.instance_field × value.lfo`, a spatial gradient
/// modulated in time — is exactly a length-N against a length-1, and which one
/// the artist happened to wire first is not a fact about the answer.
///
/// The two fields are far from zero and far from each other, so a stage that
/// never ran (a zeroed buffer) and a stage that read the wrong port are both
/// visible rather than hiding inside a plausible number.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_math_matches_the_cpu_in_every_op_and_takes_the_widest_input() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // 0 Add · 1 Subtract · 2 Multiply · 3 Divide · 4 Min · 5 Max — plus 4.4, and
    // plus the HALF-INTEGERS, which are the only values that test `vm_round` at
    // all. Rust's `f32::round` is half-AWAY-from-zero and WGSL's builtin `round`
    // is half-to-EVEN, so they agree on 4.4 and on every integer and disagree
    // precisely at `.5`: at `op = 0.5` one path runs Subtract and the other Add.
    // A sweep of whole numbers leaves that swap invisible
    // ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]).
    for op in [0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 4.4, 0.5, 1.5, 2.5, 4.5] {
        for wide_on_a in [true, false] {
            let mut g = Graph::new();
            let grid = grid_node(&mut g, 11.0);
            let wide = ramp_field(&mut g, grid, 2.5, 7.0); // per element, ≈ 4.5..9.5
            let one = const_field(&mut g, 3.0); // one global value
            let math = g.add_node("value.math");
            g.set_param(math, "op", op);
            let (a, b) = if wide_on_a { (wide, one) } else { (one, wide) };
            g.connect(Edge {
                from: (a, 0),
                to: (math, 0),
                delayed: false,
            })
            .unwrap();
            g.connect(Edge {
                from: (b, 0),
                to: (math, 1),
                delayed: false,
            })
            .unwrap();

            let (cpu, gc) = cook_both(&gpu, &reg, &g, math);
            let cpu_v = match cpu.get("v") {
                Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
                _ => panic!("the CPU emitted no `v`"),
            };
            assert_eq!(
                cpu_v.len(),
                121,
                "op {op} (wide on {}): the output is as wide as the WIDEST input, \
                 not as wide as port 0",
                if wide_on_a { "a" } else { "b" }
            );
            assert_eq!(
                gc.node_count(math),
                Some(121),
                "op {op}: the count law must size the stage at 121 — `Some(1)` is \
                 the default law reading port 0, which is the bug this sweep exists \
                 for"
            );
            assert!(
                cpu_v.iter().any(|v| v.abs() > 0.5),
                "fixture check: op {op} must produce something far from zero, or a \
                 stage that never ran would pass with an empty buffer"
            );
            let d = compare_column(&gpu, &gc, math, &cpu, "v");
            eprintln!(
                "value.math op {op} (wide on {}): max |dv| {d:e}",
                if wide_on_a { "a" } else { "b" }
            );
            assert!(d < 1e-5, "op {op}: value parity |dv| = {d:e}");
        }
    }
}

/// **The divisor threshold is spelled twice, so a gate straddles it.**
///
/// `MIN_DIVISOR` is a Rust constant and the WGSL sees the literal `1e-9`,
/// because a `&'static str` cannot interpolate one. That is the worst kind of
/// number to spell twice: below it the quotient is `0.0` and above it it is
/// `a/b`, so a divisor landing between two spellings takes a different arm on
/// each path and the graph looks otherwise identical
/// ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]).
///
/// `a` is 1.0 and the divisors bracket the threshold by orders of magnitude, so
/// the two arms are `0.0` and something astronomically large — the disagreement,
/// if there is one, cannot hide under any ε.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn dividing_by_a_threshold_divisor_agrees_on_both_sides() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for divisor in [0.0f32, 1e-12, 5e-10, 1e-9, 2e-9, 1e-6, 1.0] {
        let mut g = Graph::new();
        let a = const_field(&mut g, 1.0);
        let b = const_field(&mut g, divisor);
        let math = g.add_node("value.math");
        g.set_param(math, "op", 3.0); // Divide
        for (src, port) in [(a, 0u16), (b, 1u16)] {
            g.connect(Edge {
                from: (src, 0),
                to: (math, port),
                delayed: false,
            })
            .unwrap();
        }
        let (cpu, gc) = cook_both(&gpu, &reg, &g, math);
        let cpu_v = match cpu.get("v") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("no `v`"),
        };
        let gpu_v = gc.read_column(&gpu, math, "v").expect("`v` reads back");
        assert_eq!(cpu_v.len(), 1);
        assert_eq!(gpu_v.len(), 1);
        // The guard's whole promise: never `inf`, never `NaN`, on either path.
        assert!(
            cpu_v[0].is_finite() && gpu_v[0].is_finite(),
            "divisor {divisor}: cpu {} gpu {} — the guard exists so a downstream \
             field never sees inf/NaN",
            cpu_v[0],
            gpu_v[0]
        );
        // Both sides must take the SAME arm. Compared relatively: above the
        // threshold the quotient is up to 1e9, where an absolute ε is meaningless.
        let scale = cpu_v[0].abs().max(1.0);
        let d = (cpu_v[0] - gpu_v[0]).abs() / scale;
        eprintln!("value.math ÷{divisor}: cpu {} gpu {}", cpu_v[0], gpu_v[0]);
        assert!(
            d < 1e-5,
            "divisor {divisor}: the two paths took different arms — cpu {} vs gpu {}",
            cpu_v[0],
            gpu_v[0]
        );
    }
}

/// **`value.switch` routes per element, broadcasts in both directions, and
/// clamps the way the CPU clamps.**
///
/// Two cases, because the broadcast has two directions and only one of them is
/// the common one:
///
/// - a length-N SELECT over length-1 sources — the per-point mux, where the
///   selector is the wide field and the sources are held;
/// - a length-1 SELECT over a length-N source — the whole grid switching
///   together, where the selector is held and the source is wide.
///
/// The sources are 10/20/30/40 so a routing mistake is a *jump between named
/// numbers*, never a drift that an ε could absorb — and the select sweeps past
/// both ends of the range, so the clamp is exercised rather than assumed. The
/// bound is 3 unconditionally on both paths: `eval` builds `N_INPUTS` fields
/// whatever is connected, so `select = 5` with only `in0` wired reads the empty
/// field, not `in0`.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn value_switch_routes_per_element_and_broadcasts_both_ways() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();

    // ── Case 1: a wide SELECT over held sources.
    {
        let mut g = Graph::new();
        let grid = grid_node(&mut g, 11.0);
        // −0.6 .. 3.6: every index, and past BOTH ends of the clamp.
        let select = ramp_field(&mut g, grid, 2.1, 1.5);
        let sw = g.add_node("value.switch");
        g.connect(Edge {
            from: (select, 0),
            to: (sw, 0),
            delayed: false,
        })
        .unwrap();
        for (k, v) in [10.0f32, 20.0, 30.0, 40.0].into_iter().enumerate() {
            let src = const_field(&mut g, v);
            g.connect(Edge {
                from: (src, 0),
                to: (sw, k as u16 + 1),
                delayed: false,
            })
            .unwrap();
        }
        let (cpu, gc) = cook_both(&gpu, &reg, &g, sw);
        let cpu_v = match cpu.get("v") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("no `v`"),
        };
        assert_eq!(cpu_v.len(), 121, "as wide as the selector");
        assert_eq!(gc.node_count(sw), Some(121));
        // Fixture check: the selector must actually SELECT more than one input,
        // or this is a broadcast test wearing a router's name.
        let mut seen: Vec<f32> = cpu_v.clone();
        seen.sort_by(f32::total_cmp);
        seen.dedup();
        assert!(
            seen.len() >= 3,
            "fixture check: the ramp must reach at least three inputs, saw {seen:?}"
        );
        let d = compare_column(&gpu, &gc, sw, &cpu, "v");
        eprintln!("value.switch wide-select: inputs hit {seen:?}, max |dv| {d:e}");
        assert!(d < 1e-5, "wide-select parity |dv| = {d:e}");
    }

    // ── Case 2: a held SELECT over a wide source.
    {
        let mut g = Graph::new();
        let grid = grid_node(&mut g, 11.0);
        let select = const_field(&mut g, 2.0); // → in2, everywhere
        let sw = g.add_node("value.switch");
        g.connect(Edge {
            from: (select, 0),
            to: (sw, 0),
            delayed: false,
        })
        .unwrap();
        let wide = ramp_field(&mut g, grid, 2.5, 7.0);
        for (k, src) in [
            const_field(&mut g, 10.0),
            const_field(&mut g, 20.0),
            wide,
            const_field(&mut g, 40.0),
        ]
        .into_iter()
        .enumerate()
        {
            g.connect(Edge {
                from: (src, 0),
                to: (sw, k as u16 + 1),
                delayed: false,
            })
            .unwrap();
        }
        let (cpu, gc) = cook_both(&gpu, &reg, &g, sw);
        let cpu_v = match cpu.get("v") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("no `v`"),
        };
        assert_eq!(
            cpu_v.len(),
            121,
            "as wide as the widest input — the SOURCE here, not the selector"
        );
        assert_eq!(gc.node_count(sw), Some(121));
        // The held selector must have routed to the WIDE input: if it read
        // `in0` the field would be the constant 10 everywhere.
        let spread = cpu_v.iter().fold(f32::MIN, |m, v| m.max(*v))
            - cpu_v.iter().fold(f32::MAX, |m, v| m.min(*v));
        assert!(
            spread > 1.0,
            "fixture check: the held selector must route to the wide source \
             (spread {spread}), or this gate is comparing two constants"
        );
        let d = compare_column(&gpu, &gc, sw, &cpu, "v");
        eprintln!("value.switch held-select: spread {spread}, max |dv| {d:e}");
        assert!(d < 1e-5, "held-select parity |dv| = {d:e}");
    }
}

/// **The switch's rounding convention, at the only values that can show it.**
///
/// `select` picks a BRANCH, and Rust's `f32::round` is half-AWAY-from-zero while
/// WGSL's builtin `round` is half-to-EVEN. They agree everywhere except `.5`, so
/// a selector that never lands there tests the routing and not the rounding: at
/// `select = 0.5` the CPU reads `in1` and a half-even kernel reads `in0`, and
/// those are two *different inputs*, not two nearby numbers
/// ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]).
///
/// The `.5` values are reachable because [`const_field`] pins a VALUE field to an
/// exact constant — a swept ramp would step over them.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_switch_rounds_a_half_integer_selector_the_way_the_cpu_does() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for select in [0.5f32, 1.5, 2.5, -0.5, 3.5] {
        let mut g = Graph::new();
        let sel = const_field(&mut g, select);
        let sw = g.add_node("value.switch");
        g.connect(Edge {
            from: (sel, 0),
            to: (sw, 0),
            delayed: false,
        })
        .unwrap();
        for (k, v) in [10.0f32, 20.0, 30.0, 40.0].into_iter().enumerate() {
            let src = const_field(&mut g, v);
            g.connect(Edge {
                from: (src, 0),
                to: (sw, k as u16 + 1),
                delayed: false,
            })
            .unwrap();
        }
        let (cpu, gc) = cook_both(&gpu, &reg, &g, sw);
        let cpu_v = match cpu.get("v") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("no `v`"),
        };
        let gpu_v = gc.read_column(&gpu, sw, "v").expect("`v` reads back");
        eprintln!(
            "value.switch select {select}: cpu {} gpu {}",
            cpu_v[0], gpu_v[0]
        );
        // The inputs are 10 apart, so a rounding disagreement is a 10-unit jump.
        assert_eq!(
            cpu_v[0], gpu_v[0],
            "select {select}: the two paths routed to DIFFERENT inputs"
        );
    }
}

/// **`motion.stagger` on Rotation and Size, matching the CPU.**
///
/// Kept apart from the `noise`/`wiggle`/`oscillator` sweep because the stagger's
/// magnitude knobs are `min`/`max` rather than `amplitude` — the sweep would
/// have to special-case it, and a fixture that silently leaves a node at its
/// defaults is a fixture that tests the node being a no-op.
///
/// `min ≠ max` and both far from zero, with a curve that is a BRANCH (Bounce, In
/// Out): the easing table is the bulk of this kernel, and the default linear
/// curve exercises none of it.
///
/// ⚠️ **The HALF-INTEGER curve/direction is not decoration.** `ease_curve` and
/// `ease_dir` are rounded to pick a branch, and Rust's `f32::round` is
/// half-AWAY-from-zero while WGSL's builtin `round` is half-to-EVEN — so they
/// agree on every whole number and part exactly at `.5`, where `6.5` selects
/// Bounce on one path and Back on the other. Replacing `sg_round` with the WGSL
/// builtin **survived** a whole-number-only sweep of this gate
/// ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_stagger_variants_match_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // (curve, dir): Bounce/InOut is the branchiest whole-number pair; the two
    // `.5` pairs are what makes the rounding observable at all.
    let easings = [(7.0f32, 2.0f32), (6.5, 2.0), (7.0, 0.5), (2.5, 1.5)];
    for (channel, label) in [(0.0f32, "X"), (1.0, "Y"), (2.0, "Rotation"), (3.0, "Size")] {
        for reverse in [0.0f32, 1.0] {
            for (curve, dir) in easings {
                let mut g = Graph::new();
                let (node, out) = deformer_chain(&mut g, 40.0, "motion.stagger");
                g.set_param(node, "channel", channel);
                g.set_param(node, "min", -0.7);
                g.set_param(node, "max", 1.9);
                g.set_param(node, "ease_curve", curve);
                g.set_param(node, "ease_dir", dir);
                g.set_param(node, "reverse", reverse);
                eprintln!("  stagger on {label} (reverse {reverse}, ease {curve}/{dir})");
                assert_gpu_parity(&gpu, &reg, &g, out, 2);
            }
        }
    }
}

// ── The bounded tap ─────────────────────────────────────────────────────────

/// **The tap reports what the CPU memo reports** — the gate that lets the graph
/// panel read a GPU-resident frame.
///
/// A GPU cook does not feed the CPU memo, so the panel's readouts, digest, stamps
/// and probe all go blank on exactly the documents worth watching. `GpuCook::tap`
/// samples them back for ~0,075 ms (`bounded_readback_cost_probe`), and this is
/// what makes the samples trustworthy: for the SAME document, every column the
/// tap returns must match the CPU's stream at the strided indices it claims to
/// have read.
///
/// ⚠️ **The stream is large on purpose.** At 48 elements or fewer the stride is 1
/// and the tap degenerates into a prefix copy — which is precisely the bug the
/// strided gather exists to avoid, so a small fixture would gate the one case
/// that cannot fail ([[reference_topic_fixture_discipline]]). 400 elements makes
/// the stride 8.
///
/// ⚠️ **And the COUNT is asserted to come from elsewhere.** The tapped stream
/// carries 48 rows; a panel that counted them would print `48 inst` for a grid of
/// 400. The count is `CookShape`'s, and this gate pins both halves so nobody
/// "simplifies" the readout into asking the tap.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_bounded_tap_reports_what_the_cpu_memo_reports() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (osc, out) = deformer_chain(&mut g, 20.0, "motion.oscillator"); // 400 elements
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 1.7);
    g.set_param(osc, "frequency", 0.9);
    g.validate(&reg).expect("well-typed");

    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu());

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, &reg, osc, PLAYHEAD).expect("cpu cook");
    let cpu_stream = cpu[0].as_stream();
    assert_eq!(cpu_stream.count(), 400, "fixture: the stride must exceed 1");

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

    let samples = ph2d_gpu_cook::tap::TAP_SAMPLES;
    let tapped = gc
        .tap(&gpu, samples)
        .expect("the tap returns the staged nodes");
    let osc_tap = tapped.get(&osc).expect("the oscillator was staged");

    // The count question, and the two different right answers to it.
    assert_eq!(
        osc_tap.count(),
        samples as usize,
        "the tap carries SAMPLES, not elements"
    );
    assert_eq!(
        gc.shape().count(osc),
        Some(400),
        "the exact count comes from CookShape — a panel that counted the tap \
         would print `48 inst` for a grid of 400"
    );

    // Every tapped row must be the CPU's row at the index the stride names.
    let ph2d_nodegraph::attr::Column::Vec2(cpu_p) = cpu_stream.get("P").expect("cpu P") else {
        panic!("P is Vec2");
    };
    let ph2d_nodegraph::attr::Column::Vec2(tap_p) = osc_tap.get("P").expect("tapped P") else {
        panic!("P is Vec2");
    };
    let mut worst = 0.0f32;
    for (i, t) in tap_p.iter().enumerate() {
        let src = (i as u32 * 400) / samples;
        let c = cpu_p[src as usize];
        worst = worst.max((t[0] - c[0]).abs()).max((t[1] - c[1]).abs());
    }
    eprintln!(
        "tap vs cpu: {} samples of 400, worst |dP| {worst:e}",
        tap_p.len()
    );
    assert!(
        worst < 2e-3,
        "the tap must sample the same field: {worst:e}"
    );

    // The stride is REAL: sampling the front 48 of a wave would give a much
    // narrower spread than sampling all 400. Without this the gate would pass on
    // a prefix copy, which is the whole thing the gather exists to prevent.
    let span = |v: &[[f32; 2]]| {
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), p| (l.min(p[1]), h.max(p[1])));
        hi - lo
    };
    let full_span = span(cpu_p);
    let prefix_span = span(&cpu_p[..samples as usize]);
    let tap_span = span(tap_p);
    eprintln!("  span: full {full_span:.4} · cpu prefix {prefix_span:.4} · tapped {tap_span:.4}");
    assert!(
        prefix_span < full_span * 0.9,
        "fixture check: the prefix must be visibly narrower than the whole \
         ({prefix_span} vs {full_span}), or a prefix copy would pass this gate"
    );
    // The bar is the MIDPOINT between the two, not the full span. A 48-point
    // subsample of a wave cannot land on its exact peaks — measured, the tapped
    // span is 6,95 against a full 8,13 — so demanding 90% of the full span would
    // be demanding something false of a correct sampler. What the gate can
    // honestly ask is that the tap look far more like the whole than like the
    // front (prefix 3,67), and that is what a prefix copy would fail.
    let bar = (prefix_span + full_span) * 0.5;
    assert!(
        tap_span > bar,
        "the tap must WALK the stream, not read its front: tapped span \
         {tap_span} vs the {bar} midpoint between prefix {prefix_span} and full \
         {full_span}"
    );
}

/// **A broadcast port at a length the dispatch cannot pair REFUSES the cook**
/// (the mixed-length hole ADR-0127 D3's docs named and nothing closed): a
/// 9-element value field aimed at a 25-element flock is neither per-element
/// (25) nor a global broadcast (1). `column_present` judges it absent, so the
/// kernel would read the identity at EVERY index — while the CPU (`target_at`'s
/// `_` arm) serves rows 0..9 and only falls back past them. Same document, two
/// different fields, and no ε covers a SHAPE.
///
/// The cook now returns [`ph2d_gpu_cook::GpuCookError::BroadcastLengthMismatch`]
/// and the bridge's `.is_ok()` route falls through to the CPU pump — the
/// canonical answer, on both machines.
///
/// MUTATION: delete the `broadcast_length_mismatch` check in `GpuCook::cook`
/// and this cook returns `Ok` (identity-everywhere, silently) — RED here.
#[test]
#[ignore = "needs a GPU adapter"]
fn a_mixed_length_broadcast_port_refuses_the_cook_to_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let flock = grid_node(&mut g, 5.0); // 25 elements — the dispatch length
    let small = grid_node(&mut g, 3.0); // 9 elements — the mismatched field
    let field = g.add_node("value.instance_field");
    connect(&mut g, small, field);
    let la = g.add_node("motion.look_at");
    connect(&mut g, flock, la);
    g.connect(Edge {
        from: (field, 0),
        to: (la, 1), // target_x — a ReadBroadcast port
        delayed: false,
    })
    .expect("target port");
    g.validate(&reg)
        .expect("well-typed — the artist can wire this");

    // The plan CLAIMS it (lengths are a cook-time fact; `applicable` sees only
    // params) — which is exactly why the refusal must live in the cook.
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, la);
    assert!(plan.is_fully_gpu(), "the plan cannot see stream lengths");

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let got = gc.cook(
        &gpu,
        &g,
        &reg,
        &reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
    );
    match got {
        Err(ph2d_gpu_cook::GpuCookError::BroadcastLengthMismatch {
            port, len, count, ..
        }) => {
            assert_eq!((port, len, count), (1, 9, 25), "the offender is named");
        }
        other => panic!(
            "a mixed-length broadcast must refuse to the CPU, got {other:?} \
             (an Ok here is the identity-everywhere divergence, dispatched)"
        ),
    }

    // And the lengths broadcast CAN pair keep cooking: the same wiring with the
    // field fed by the SAME grid (per-element) — the refusal must not overreach.
    let mut g2 = Graph::new();
    let flock2 = grid_node(&mut g2, 5.0);
    let field2 = g2.add_node("value.instance_field");
    connect(&mut g2, flock2, field2);
    let la2 = g2.add_node("motion.look_at");
    connect(&mut g2, flock2, la2);
    g2.connect(Edge {
        from: (field2, 0),
        to: (la2, 1),
        delayed: false,
    })
    .expect("target port");
    let plan2 = ph2d_gpu_cook::plan(&g2, &reg, &reg, la2);
    let mut gc2 = ph2d_gpu_cook::GpuCook::new();
    gc2.cook(
        &gpu,
        &g2,
        &reg,
        &reg,
        &plan2,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
    )
    .expect("a per-element field still cooks");
}
