#![forbid(unsafe_code)]
//! **`sim.lifetime`** — death by AGE, inside a simulation zone (Motion Nodes O4, doc 50).
//!
//! ## The death a particle system actually dies of
//!
//! The zone could already kill by PLACE (`motion.falloff` + `motion.cull`: leave the circle and
//! you are gone). But the death every particle system is built on is by **age**: a spark burns
//! out, a flake melts, a puff of smoke thins away — and none of them at a specific coordinate.
//!
//! `sim.step` grows an `age` on every element (it owns the sim's clock, so it owns the ageing).
//! This node reads it, and:
//!
//! - **kills** whatever has outlived its lifetime, and
//! - writes **`life`** — how far through its life each survivor is, `0` at birth, `1` at the end.
//!
//! ## `life` is the point, not a by-product
//!
//! The `life` column is what makes a particle system *look* like one: colour by it (a spark
//! going red → black), shrink by it, fade by it. Feed it to `value.attribute` (doc 50) and it is
//! an ordinary value field that drives anything in the library — the ramp, the scale, a force.
//!
//! A node that only killed would have thrown that number away, and every artist would have had
//! to rebuild it out of an age and a parameter they could not read.
//!
//! ## Variance: no two flakes die together
//!
//! `variance` spreads each element's lifetime around the nominal one by a **hash of its id**
//! (`hash(seed, id, lane)` — stateless, so a scrub reproduces the same deaths; HR-5,
//! transcendental-free). Without it every particle born on the same tick dies on the same tick,
//! and the population blinks instead of breathing.

mod hash;

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, KEEP_FLAG_COL, StreamOp};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The jitter lane for the per-element lifetime (independent of the spawn's own draws).
const LANE_LIFE: u32 = 11;

/// The shortest lifetime the variance may produce, as a fraction of the nominal one. A particle
/// whose lifetime rounded to zero would be born and killed on the same tick — visible only as a
/// flicker, and impossible to debug.
const MIN_LIFE_FRAC: f32 = 0.1;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.lifetime"),
    name: "sim.lifetime",
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
    params: &[
        // Seconds. The age is grown by `sim.step`, so a lifetime outside a zone never advances
        // and nothing ever dies — which is the honest answer: there is no life without a sim.
        ParamSpec {
            name: "life",
            default: 2.0,
        },
        // 0 = every element lives exactly `life`; 1 = anywhere from a tenth of it to twice it.
        ParamSpec {
            name: "variance",
            default: 0.35,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn scalar(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => v.clone(),
        _ => vec![0.0; n],
    }
}

/// This element's own lifetime: the nominal one, spread by a hash of its identity.
fn life_of(id: u32, life: f32, variance: f32, seed: u32) -> f32 {
    if variance <= 0.0 {
        return life;
    }
    // `[-1, 1)` off the id's own lane → a lifetime in `life · [1-v, 1+v]`, floored so it can
    // never round away to nothing.
    let d = hash::rand01(seed, id, LANE_LIFE) * 2.0 - 1.0;
    (life * (1.0 + variance * d)).max(life * MIN_LIFE_FRAC)
}

/// Kill the outlived, and tell the survivors how far through life they are.
fn reap(s: &Stream, life: f32, variance: f32, seed: u32) -> Stream {
    let n = s.count();
    let age = scalar(s, "age", n);
    let ids = scalar(s, "id", n);

    // The survivors, in their original order (identity is order-independent, but the render is
    // not: reshuffling the set every tick would make every downstream index-based node flicker).
    let mut keep: Vec<usize> = Vec::with_capacity(n);
    let mut lives: Vec<f32> = Vec::with_capacity(n);
    for i in 0..n {
        let span = life_of(ids[i] as u32, life, variance, seed);
        if age[i] < span {
            keep.push(i);
            lives.push((age[i] / span).clamp(0.0, 1.0)); // CLAMP-OK: fraction of a life
        }
    }

    let mut out = Stream::new(keep.len());
    for (name, col) in s.columns() {
        if name == "life" {
            continue; // ours to write
        }
        out.set(name.clone(), gather(col, &keep));
    }
    out.set("life", Column::Scalar(lives));
    out
}

fn gather(col: &Column, keep: &[usize]) -> Column {
    fn take<T: Copy>(v: &[T], keep: &[usize]) -> Vec<T> {
        keep.iter().map(|&i| v[i]).collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(take(v, keep)),
        Column::Vec2(v) => Column::Vec2(take(v, keep)),
        Column::Vec3(v) => Column::Vec3(take(v, keep)),
        Column::Vec4(v) => Column::Vec4(take(v, keep)),
    }
}

/// The WGSL mirror of [`life_of`], shared verbatim by the predicate and the
/// epilogue (ADR-0136 — seed and sample must be the same function, here twice
/// over: who DIES and how far through life a survivor is must agree on the
/// span, or an element could die at `life < 1`). The hash is the emitter's
/// avalanche, bit-exact in WGSL (u32 wrapping arithmetic, 24 mantissa bits).
/// The param guards are the CPU `eval`'s own (`max(0)`, `clamp(0,1)`), applied
/// in-kernel because uniforms carry the RAW params.
const LIFE_LIB: &str = "    fn lf_hash3(a: u32, b: u32, lane: u32) -> f32 {
        var h: u32 = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;
        h = h ^ (h >> 16u);
        h = h * 0x7feb352du;
        h = h ^ (h >> 15u);
        h = h * 0x846ca68bu;
        h = h ^ (h >> 16u);
        return f32(h >> 8u) / f32(16777216u);
    }
    fn lf_span(id: f32, life_p: f32, variance_p: f32, seed_p: f32) -> f32 {
        let life = max(life_p, 0.0);
        let variance = clamp(variance_p, 0.0, 1.0);
        if (variance <= 0.0) { return life; }
        let seed = u32(max(seed_p, 0.0));
        let d = lf_hash3(seed, u32(max(id, 0.0)), 11u) * 2.0 - 1.0;
        return max(life * (1.0 + variance * d), life * 0.1);
    }
";

/// The GPU predicate (ADR-0136): survives ⇔ `age < span`, [`reap`]'s own test.
/// Absent `age`/`id` read `0` — the CPU's `scalar()` zero-fill.
const GPU_PREDICATE: GpuKernel = GpuKernel {
    wgsl: "        let lf_sp = lf_span(read_id(i), params.life, params.variance, params.seed);
        write_cp_keep(i, select(0.0, 1.0, read_age(i) < lf_sp));
",
    wgsl_lib: LIFE_LIB,
    bindings: &[
        ColumnBinding {
            column: "age",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: KEEP_FLAG_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["life", "variance", "seed"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The node's own kernel — the EPILOGUE, an ordinary per-element pass over the
/// compacted survivors: `life = clamp(age / span, 0, 1)`, [`reap`]'s survivor
/// arm. Every survivor's span is positive (a zero span died at the predicate),
/// so the division is total here.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "        let lf_sp = lf_span(read_id(i), params.life, params.variance, params.seed);
        write_life(i, clamp(read_age(i) / lf_sp, 0.0, 1.0));
",
    wgsl_lib: LIFE_LIB,
    bindings: &[
        ColumnBinding {
            column: "age",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            // Written fresh for the survivors — the CPU skips copying any
            // pre-existing `life` and writes its own (`ours to write`).
            column: "life",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["life", "variance", "seed"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct SimLifetime;

impl NodeOp for SimLifetime {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let life = ctx.param("life").max(0.0);
        let variance = ctx.param("variance").clamp(0.0, 1.0); // CLAMP-OK: const bounds
        let seed = ctx.param("seed").max(0.0) as u32;
        let out = reap(ctx.input(0), life, variance, seed);
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimLifetime))?;
    // ADR-0136: compact by the predicate, then run the epilogue on the
    // survivors. NOT `register_dense_window` — a reaper breaks the window.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(
        MANIFEST.id,
        StreamOp::Compact {
            port: 0,
            predicate: GPU_PREDICATE,
        },
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Lifetime",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidUp,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "life",
        label: "Life",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "variance",
        label: "Variance",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
