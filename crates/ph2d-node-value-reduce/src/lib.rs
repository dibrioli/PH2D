#![forbid(unsafe_code)]
//! `value.reduce` — the value-domain GENERAL reducer: fold a field to ONE number
//! and **broadcast** it back to a constant field (Motion Nodes M2, the value
//! domain — doc 12/76). It is the composable counterpart to `value.normalize`
//! (which FUSES a `min`/`max` reduce with a fit): this one exposes the aggregate
//! itself — `Sum`, `Mean`, `Min`, `Max`, `Range`, `Variance`, `StdDev`, `Median`
//! — as a field every downstream node can fold against.
//!
//! **Why it matters.** It is the only way to make a value RELATIVE to the whole
//! field: `value.reduce(Mean)` then `value.math(Subtract)` **centres** a field
//! about its own average; `value.reduce(Sum)` then `value.math(Divide)` turns it
//! into a **distribution** (every element as a fraction of the total). Nothing
//! else in the value domain reaches a number that depends on ALL the elements —
//! this is the `reduce → broadcast` half of the deformers' shape, on a `v` column
//! (the `map` is whatever `value.math` you compose downstream). It is Houdini's
//! *promote to detail* / TouchDesigner's Analyze CHOP.
//!
//! **`mode`** picks the aggregate; the law lives in [`stats`], one copy for both
//! paths. Five of the eight are **device-resident** (the sequencer runs the
//! reductions before the kernel and hands it the results):
//! - **Sum** — `Σ vᵢ`. Float addition is not associative, so this carries a
//!   documented ε against the CPU (the tree order differs; `Min`/`Max` do not).
//! - **Mean** — `Σ vᵢ / N`. The count is `Σ 1.0`, which is EXACT for `N < 2²⁴`
//!   (integers add exactly), so only the `Sum` numerator is ε.
//! - **Min** / **Max** / **Range** — bit-exact in any evaluation order (no ε).
//!
//! ⚠️ **Three modes REFUSE the device** (`applicable`), and it is ONE mechanism
//! wearing two faces: **the reductions all run BEFORE the kernel and none can
//! read another's result**. `Median` is a global rank, which has no associative
//! combiner at all; `Variance`/`StdDev` need `Σ(v − μ)²`, whose `μ` does not
//! exist until the first fold is done — and the one-pass algebra that would
//! dodge that was built, MEASURED and thrown away (a **constant** field at
//! magnitude `1e5` reported a standard deviation of **71**; see
//! [`stats::variance`]). Those three become a CPU boundary and the coverage
//! census says so, rather than the graph going quietly wrong. **The device path
//! for a spread still exists by composition** — the five-node chain the folha
//! already measured — and the doc of [`stats::variance`] names it.
//!
//! **Two OPTIONAL ports scope the set** — `mask` (who is counted) and `group`
//! (a segmented reduce, one aggregate per bin). ⚠️ Both are **plan-time
//! refusals** on the device (`ColumnAccess::RefuseIfPresent`): the reduction
//! channel folds ONE column of ONE port through ONE expression, so neither
//! `Σ(v·mask)` (two columns) nor a per-bin fold is expressible with it. Wiring
//! either port hands the node to the CPU — **unwired, nothing changes and the
//! whole family stays on the device**.
//!
//! **The output is a CONSTANT field of the same length** — the aggregate written
//! to every element — not a length-1 stream, so it lines up element-for-element
//! with the source when a `value.math` folds the two. `Pure` (no clock, no state);
//! length preserved. **The value type** is the continuous per-instance scalar
//! field `(Instances, Scalar, Frame)` on the `v` column (doc 12).

mod stats;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use stats::{Mode, reduce_field};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// Read the `v` column of an input port as a plain field (absent ⇒ empty).
fn field_of(ctx: &EvalCtx<'_>, port: usize) -> Vec<f32> {
    match ctx.input(port).get(VALUE_COL) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The static contract of this node type (ADR-0031). The kernel, its reductions
/// and the param side-tables are side-metadata (ADR-0126); `NodeManifest` stays
/// the frozen 8 fields.
///
/// ⚠️ **The two extra ports are APPENDED** — `in` keeps index 0, so every saved
/// graph's edge still lands where it did. A port inserted before it would
/// re-point every existing wire in silence.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.reduce"),
    name: "value.reduce",
    inputs: &[
        PortSpec {
            name: "in",
            ty: VALUE,
        },
        PortSpec {
            name: "mask",
            ty: VALUE,
        },
        PortSpec {
            name: "group",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "mode",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// The four whole-stream reductions the kernel reads (GPU/M5, the deformer channel
/// — `ph2d_nodegraph::reduce_meta`). `sum`/`min`/`max` fold the element VERBATIM;
/// **`count` folds the constant `1.0`**, so `Σ 1.0 = N` — exact for `N < 2²⁴`,
/// which is what makes `Mean`'s denominator match the CPU's `len()` on the byte.
/// `min`/`max` are bit-exact in any order; `sum` is the documented ε.
///
/// ⚠️ **There is no `sumsq` here, and the absence is the wave's finding** — the
/// second moment would make a one-pass variance expressible, and that formula was
/// measured to report a standard deviation of 71 for a CONSTANT field.
///
/// The identity is the `v` binding's own (`0.0`): an absent column reads as a
/// constant `0` field on both paths.
static REDUCES: &[ReduceSpec] = &[
    ReduceSpec {
        name: "sum",
        column: VALUE_COL,
        dim: Dim::Scalar,
        port: 0,
        op: ReduceOp::Sum,
        value: "v",
        params: &[],
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "count",
        column: VALUE_COL,
        dim: Dim::Scalar,
        port: 0,
        op: ReduceOp::Sum,
        // Not `v`: the count is the number of ELEMENTS, so fold `1.0` each. Exact.
        value: "1.0",
        params: &[],
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "min",
        column: VALUE_COL,
        dim: Dim::Scalar,
        port: 0,
        op: ReduceOp::Min,
        value: "v",
        params: &[],
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "max",
        column: VALUE_COL,
        dim: Dim::Scalar,
        port: 0,
        op: ReduceOp::Max,
        value: "v",
        params: &[],
        identity: [0.0; 4],
    },
];

/// **Can the device answer for these params?** — three of the eight aggregates
/// need something the reduction channel cannot give: a global RANK (`Median`) or
/// a second pass that reads the FIRST one's result (`Variance`/`StdDev`). The
/// node recedes to the CPU for those and keeps the device for the other five.
/// Evaluated at plan time against the resolved `mode`.
fn device_can_answer(param: &dyn Fn(&str) -> f32) -> bool {
    !matches!(
        Mode::from_param(param("mode")),
        Mode::Variance | Mode::StdDev | Mode::Median
    )
}

/// GPU compute kernel (ADR-0126) — reads the reductions above and BROADCASTS the
/// chosen aggregate to every element, **fully device-resident** for the seven
/// foldable modes. VALUE in, VALUE out: the base rides here (a VALUE stream
/// carries only `v`). Every element `i` gets the SAME value — that is the
/// broadcast.
const GPU_KERNEL: GpuKernel = GpuKernel {
    // ⚠️ **There is no arm for 5/6/7** — `applicable` proves the sequencer never
    // reaches this body with those modes, and writing an arm that computes them
    // WRONG (the one-pass variance) so the switch "looks complete" is exactly the
    // plausible-and-silent answer this node's refusal exists to prevent. The
    // parity gate asserts the other half: those three modes are NOT claimed.
    wgsl: "\
        let vr_mode = i32(vr_round(params.mode));\n\
        var vr_o: f32;\n\
        switch (vr_mode) {\n\
            case 1: { vr_o = reduce_sum() / max(reduce_count(), 1.0); }\n\
            case 2: { vr_o = reduce_min(); }\n\
            case 3: { vr_o = reduce_max(); }\n\
            case 4: { vr_o = reduce_max() - reduce_min(); }\n\
            default: { vr_o = reduce_sum(); }\n\
        }\n\
        write_v(i, vr_o);\n",
    wgsl_lib: "\
        fn vr_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    // **Write-only on port 0, not ReadWrite** — the kernel never reads the
    // original `v` (the reductions read it in their own passes); it only WRITES
    // the broadcast aggregate. A ReadWrite binding would declare an `in_v` the
    // body never calls, which naga strips from the layout while the sequencer
    // still binds its buffer — a 7-vs-6 mismatch. Write materializes a fresh
    // output column.
    //
    // ⚠️ **Ports 1 and 2 are REFUSALS, not accesses.** A wired `mask` needs
    // `Σ(v·mask)` — two columns in one reduction expression, which the channel
    // cannot express — and a wired `group` needs a segmented fold, which is
    // another machine entirely. Declaring the refusal is what keeps the device
    // from claiming a node it would answer WRONG for; unwired, `input_edge`
    // returns `None` and nothing changes.
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 2,
        },
    ],
    params: &["mode"],
    count_law: None,
    variant_by_param: None,
    applicable: Some(device_can_answer),
};

struct ValueReduce;

impl NodeOp for ValueReduce {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = Mode::from_param(ctx.param("mode"));
        let input = field_of(ctx, 0);
        let mask = field_of(ctx, 1);
        let group = field_of(ctx, 2);
        let n = input.len();
        // The reduce → broadcast: one aggregate per group, written to every
        // element so the constant field lines up with the source for a
        // downstream `value.math`.
        let out = reduce_field(&input, mode, &mask, &group);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueReduce))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // GPU/M5: the whole-stream reductions the kernel reads. Side metadata on the
    // registry (ADR-0126) — the frozen node contract is untouched.
    reg.register_reduces(MANIFEST.id, REDUCES);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Reduce",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// ⚠️ **A ordem dos rótulos É a ordem dos índices** — o `mode` guardado num
/// documento é o índice, então a lista só cresce pelo FIM.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "mode",
    label: "Mode",
    min: 0.0,
    max: 7.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &[
            "Sum", "Mean", "Min", "Max", "Range", "Variance", "Std Dev", "Median",
        ],
    },
}];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "stats_tests.rs"]
mod stats_gates;
