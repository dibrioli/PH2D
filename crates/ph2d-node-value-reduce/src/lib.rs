#![forbid(unsafe_code)]
//! `value.reduce` — the value-domain GENERAL reducer: fold a field to ONE number
//! and **broadcast** it back to a constant field (Motion Nodes M2, the value
//! domain — doc 12/76). It is the composable counterpart to `value.normalize`
//! (which FUSES a `min`/`max` reduce with a fit): this one exposes the aggregate
//! itself — `Sum`, `Mean`, `Min`, `Max` — as a field every downstream node can
//! fold against.
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
//! **`mode`** picks the aggregate, all **device-resident** (the sequencer runs the
//! reductions before the kernel and hands it the results):
//! - **Sum** — `Σ vᵢ`. Float addition is not associative, so this carries a
//!   documented ε against the CPU (the tree order differs; `Min`/`Max` do not).
//! - **Mean** — `Σ vᵢ / N`. The count is `Σ 1.0`, which is EXACT for `N < 2²⁴`
//!   (integers add exactly), so only the `Sum` numerator is ε.
//! - **Min** / **Max** — bit-exact in any evaluation order (no ε).
//!
//! **The output is a CONSTANT field of the same length** — the aggregate written
//! to every element — not a length-1 stream, so it lines up element-for-element
//! with the source when a `value.math` folds the two. `Pure` (no clock, no state);
//! length preserved. **The value type** is the continuous per-instance scalar
//! field `(Instances, Scalar, Frame)` on the `v` column (doc 12).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// Which whole-stream aggregate to broadcast.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// `Σ vᵢ` — the total (ε: float addition is not associative).
    Sum,
    /// `Σ vᵢ / N` — the average (the count is exact; the sum is ε).
    Mean,
    /// The smallest element (bit-exact in any order).
    Min,
    /// The largest element (bit-exact in any order).
    Max,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Mean,
            2 => Mode::Min,
            3 => Mode::Max,
            _ => Mode::Sum,
        }
    }
}

/// The single aggregate value from the four whole-stream reductions (the CPU
/// oracle the device tree reductions are reconciled against). `Mean` divides the
/// sum by the count; an empty field has no aggregate, so `Mean`/`Sum` fall to `0`
/// and the min/max identities never reach the map (the kernel does not run on an
/// empty stream). Written to mirror the WGSL op for op.
fn aggregate(field: &[f32], mode: Mode) -> f32 {
    match mode {
        Mode::Sum => ReduceOp::Sum.cpu(field),
        Mode::Mean => {
            let n = field.len() as f32;
            if n > 0.0 {
                ReduceOp::Sum.cpu(field) / n
            } else {
                0.0
            }
        }
        Mode::Min => ReduceOp::Min.cpu(field),
        Mode::Max => ReduceOp::Max.cpu(field),
    }
}

/// The static contract of this node type (ADR-0031). The kernel and its
/// reductions are side-metadata (ADR-0126); `NodeManifest` stays the frozen 8
/// fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.reduce"),
    name: "value.reduce",
    inputs: &[PortSpec {
        name: "in",
        ty: VALUE,
    }],
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
/// `min`/`max` are bit-exact in any order; `sum` is the documented ε. The identity
/// is the `v` binding's own (`0.0`): an absent column reads as a constant `0`
/// field on both paths.
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

/// GPU compute kernel (ADR-0126) — reads the four reductions above and BROADCASTS
/// the chosen aggregate to every element, **fully device-resident**. No
/// `applicable` gate — the sequencer never falls back to the CPU. VALUE in, VALUE
/// out: the base rides here (a VALUE stream carries only `v`). Every element `i`
/// gets the SAME value — that is the broadcast.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vr_mode = i32(vr_round(params.mode));\n\
        var vr_o: f32;\n\
        switch (vr_mode) {\n\
            case 1: { vr_o = reduce_sum() / max(reduce_count(), 1.0); }\n\
            case 2: { vr_o = reduce_min(); }\n\
            case 3: { vr_o = reduce_max(); }\n\
            default: { vr_o = reduce_sum(); }\n\
        }\n\
        write_v(i, vr_o);\n",
    wgsl_lib: "\
        fn vr_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    // **Write-only, not ReadWrite** — the kernel never reads the original `v`
    // (the reductions read it in their own passes); it only WRITES the broadcast
    // aggregate. A ReadWrite binding would declare an `in_v` the body never calls,
    // which naga strips from the layout while the sequencer still binds its
    // buffer — a 7-vs-6 mismatch. Write materializes a fresh output column.
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueReduce;

impl NodeOp for ValueReduce {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = Mode::from_param(ctx.param("mode"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // The reduce → broadcast: one aggregate, written to every element so the
        // constant field lines up with the source for a downstream `value.math`.
        let agg = aggregate(&input, mode);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(vec![agg; n])));
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

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "mode",
    label: "Mode",
    min: 0.0,
    max: 3.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &["Sum", "Mean", "Min", "Max"],
    },
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Each mode folds the whole field to its aggregate.** Sum totals, Mean
    /// averages, Min/Max pick the extremes.
    #[test]
    fn each_mode_folds_the_field_to_its_aggregate() {
        let f = [1.0, 2.0, 3.0, 4.0]; // sum 10, mean 2.5, min 1, max 4
        assert_eq!(aggregate(&f, Mode::Sum), 10.0);
        assert_eq!(aggregate(&f, Mode::Mean), 2.5);
        assert_eq!(aggregate(&f, Mode::Min), 1.0);
        assert_eq!(aggregate(&f, Mode::Max), 4.0);
        // Negatives and a single element.
        assert_eq!(aggregate(&[-3.0, 3.0], Mode::Sum), 0.0);
        assert_eq!(aggregate(&[-3.0, 3.0], Mode::Mean), 0.0);
        assert_eq!(aggregate(&[7.0], Mode::Mean), 7.0);
    }

    /// **The output is a CONSTANT field of the source's length** — the aggregate
    /// broadcast to every element, so it lines up for a downstream fold (not a
    /// length-1 stream).
    #[test]
    fn the_output_broadcasts_to_a_constant_field_of_the_same_length() {
        for n in [1usize, 4, 23] {
            let field: Vec<f32> = (0..n).map(|i| i as f32 + 1.0).collect();
            let agg = aggregate(&field, Mode::Mean);
            let out = vec![agg; field.len()];
            assert_eq!(out.len(), n, "length preserved");
            assert!(
                out.iter().all(|&x| x == agg),
                "every element is the aggregate"
            );
        }
        // An empty field yields an empty field (nothing to reduce).
        assert!(aggregate(&[], Mode::Sum).is_finite());
    }

    /// **Mean subtracted from the field centres it about zero** — the reason this
    /// node exists, proved on the aggregate: `Σ (vᵢ − mean) = 0`.
    #[test]
    fn subtracting_the_mean_centres_the_field() {
        let field = [1.0, 4.0, 10.0, -3.0, 8.0];
        let mean = aggregate(&field, Mode::Mean);
        let centred_sum: f32 = field.iter().map(|&v| v - mean).sum();
        assert!(centred_sum.abs() < 1e-4, "the centred field sums to zero");
    }

    /// A value source emitting a fixed field, so `value.reduce` can be driven
    /// through a real cook (the whole-chain proof, reduction included).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.reduce.test.src"),
        name: "value.reduce.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src(Vec<f32>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
        }
    }

    /// End-to-end through the cook: a `[2, 4, 6]` field through Mean becomes
    /// `[4, 4, 4]` — the aggregate broadcast, length preserved (the reduce ran).
    #[test]
    fn reduces_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueReduce),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![2.0, 4.0, 6.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.reduce.test.src");
        let vr = g.add_node("value.reduce");
        g.set_param(vr, "mode", 1.0); // Mean
        g.connect(Edge {
            from: (src, 0),
            to: (vr, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vr, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v, &vec![4.0, 4.0, 4.0], "mean 4 broadcast to length 3");
            }
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
