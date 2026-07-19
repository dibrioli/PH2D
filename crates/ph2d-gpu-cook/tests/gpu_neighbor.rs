//! **Grid-wiring parity** (ADR-0134, Phase 3) — the sequencer builds a spatial
//! grid before a kernel pass and the body reads neighbours through it.
//!
//! A synthetic node `test.neighbor_disp` displaces each element by the NUMBER of
//! others within `radius`. The GPU path answers that with the grid (3×3 cell
//! sweep, exact-cell dedup, distance filter); the CPU path is the honest
//! all-pairs. The neighbour SET is identical (cell = radius ⇒ the sweep is the
//! within-radius set), so the count is bit-exact and the displacement matches the
//! CPU within ε — the same reconciliation every ported kernel gets, now proving
//! the grid injection (uniform fields + `grid_starts`/`grid_sorted` bindings +
//! `grid_cell_of`/`grid_bucket_of` helpers) lines up with the build.
//!
//! It is the throwaway oracle for the WIRING; boids is the real payload.
//!
//! `#[ignore]`: needs an adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_neighbor --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::{Cook, EvalCtx};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, GridSpec};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_render::RenderInstance;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.0;
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

// ── The synthetic node: displace by the within-radius neighbour count ────────

static NEIGHBOR_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.neighbor_disp"),
    name: "test.neighbor_disp",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "radius",
        default: 1.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

struct NeighborDisp;
impl NodeOp for NeighborDisp {
    fn manifest(&self) -> &'static NodeManifest {
        &NEIGHBOR_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let r2 = ctx.param("radius").powi(2);
        let p: Vec<[f32; 2]> = match ctx.input(0).get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = p.len();
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let mut count = 0u32;
            for j in 0..n {
                if j == i {
                    continue;
                }
                let d = [p[i][0] - p[j][0], p[i][1] - p[j][1]];
                if d[0] * d[0] + d[1] * d[1] <= r2 {
                    count += 1;
                }
            }
            out.push([p[i][0] + count as f32, p[i][1]]);
        }
        // Ride the base (keep Index/Count), replace P — the GPU output does the same.
        let stream = ctx.input(0).clone().with("P", Column::Vec2(out));
        ctx.emit(stream);
    }
}

/// The GPU kernel: the grid answers "who is near me?"; the body counts and
/// displaces exactly as the CPU's all-pairs does.
static NEIGHBOR_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
    let my_p = read_P(i);\n\
    let ci = grid_cell_of(my_p);\n\
    let r2 = params.radius * params.radius;\n\
    var neighbours = 0u;\n\
    for (var dy = -1; dy <= 1; dy = dy + 1) {\n\
        for (var dx = -1; dx <= 1; dx = dx + 1) {\n\
            let c = ci + vec2<i32>(dx, dy);\n\
            let b = grid_bucket_of(c);\n\
            let lo = grid_starts[b];\n\
            let hi = grid_starts[b + 1u];\n\
            for (var s = lo; s < hi; s = s + 1u) {\n\
                let j = grid_sorted[s];\n\
                if (j == i) { continue; }\n\
                let pj = read_P(j);\n\
                let cj = grid_cell_of(pj);\n\
                // Exact-cell dedup: count j only while visiting ITS cell, so a\n\
                // hash collision that puts two cells in one bucket never doubles.\n\
                if (cj.x != c.x || cj.y != c.y) { continue; }\n\
                let d = my_p - pj;\n\
                if (dot(d, d) <= r2) { neighbours = neighbours + 1u; }\n\
            }\n\
        }\n\
    }\n\
    write_P(i, vec2<f32>(my_p.x + f32(neighbours), my_p.y));\n",
    wgsl_lib: "",
    bindings: &[ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["radius"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg.register(Box::new(NeighborDisp)).unwrap();
    reg.register_gpu_kernel(NEIGHBOR_MAN.id, NEIGHBOR_KERNEL);
    reg.register_grid(
        NEIGHBOR_MAN.id,
        GridSpec {
            column: "P",
            port: 0,
            cell_param: "radius",
        },
    );
    reg
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

fn assert_close(what: &str, i: usize, a: f32, b: f32, eps: f32) {
    assert!(
        (a - b).abs() <= eps,
        "instance {i} field {what}: cpu {a} vs gpu {b} (|diff| {})",
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
        }
    }
    eprintln!(
        "neighbour parity: {} instances, max |Δpos| = {max_pos:e}",
        cpu.len()
    );
}

#[test]
#[ignore = "needs a GPU adapter"]
fn the_grid_neighbour_kernel_matches_the_cpu_all_pairs() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_neighbor");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 40.0);
    g.set_param(grid, "cols", 40.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let nb = g.add_node("test.neighbor_disp");
    // Cell = radius; 0.6 spans a few grid steps ⇒ real, non-trivial neighbourhoods.
    g.set_param(nb, "radius", 0.6);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, nb);
    connect(&mut g, nb, out);
    g.validate(&reg).expect("well-typed");

    // Canonical CPU.
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

    // The plan must claim the whole chain and dispatch grid + neighbour (output is
    // pass-through) — else a silent CPU fallback would compare CPU to CPU.
    let plan = plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    assert_eq!(
        plan.dispatching_stages(&reg),
        2,
        "grid + neighbour dispatch"
    );

    let mut gc = GpuCook::new();
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
    let gpu_out = read_instances(&gpu, gc.instances().expect("cooked"));
    assert_parity(&cpu, &gpu_out);
}
