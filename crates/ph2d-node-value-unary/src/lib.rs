#![forbid(unsafe_code)]
//! `value.unary` — the value-domain SINGLE-INPUT arithmetic operator: apply one
//! function to every element of a field (Motion Nodes M2, the value domain — doc
//! 12/75). It is the **unary counterpart to `value.math`** (which folds TWO
//! fields with an operation): the same "one node, an operation selector, not a
//! per-op crate explosion" convergence the mature editors reach — TouchDesigner's
//! Math CHOP unary functions, Nuke's Expression, Houdini VEX's `abs`/`sqrt`/`frac`
//! — but for a single input.
//!
//! **`op`** picks the function. All are **transcendental-free** (HR-5) — algebraic
//! or arithmetic, so the GPU port is bit-comparable to the CPU and the node is
//! **device-resident** (no CPU fallback):
//! - **Abs** — `|x|` (fold a bipolar signal to unipolar).
//! - **Negate** — `−x` (flip).
//! - **Sign** — `−1 / 0 / +1` (extract direction; `sign(0) = 0`, not `signum`).
//! - **Floor** — `⌊x⌋` (truncate down to an integer, on ANY scale — distinct from
//!   `value.quantize`, which posterizes `[0,1]` into `N` levels).
//! - **Fract** — `x − ⌊x⌋ ∈ [0,1)` (the repeating sawtooth; the remainder Floor drops).
//! - **Square** — `x²` (an ease-in shape, or a magnitude).
//! - **Sqrt** — `√x`, negatives CLAMPED to `0` (an ease-out shape; correctly
//!   rounded on both paths, so exact — `sqrt` is algebraic, not transcendental).
//! - **Reciprocal** — `1/x`, `x = 0` GUARDED to `0` (never an `inf`/`NaN`).
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state); a
//! **unary** map, length preserved. The input is NOT clamped (arithmetic is
//! meaningful on any scale); the guards on Sqrt/Reciprocal are the only limits.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The single-input operation (the TD Math CHOP unary / Nuke / VEX core).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Op {
    Abs,
    Negate,
    Sign,
    Floor,
    Fract,
    Square,
    Sqrt,
    Reciprocal,
}

impl Op {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Op::Negate,
            2 => Op::Sign,
            3 => Op::Floor,
            4 => Op::Fract,
            5 => Op::Square,
            6 => Op::Sqrt,
            7 => Op::Reciprocal,
            _ => Op::Abs,
        }
    }
}

/// Apply one operation to one value. Written to mirror the WGSL op for op.
/// `Sign` is explicit (`f32::signum` gives `±1` for `±0`, disagreeing with WGSL's
/// `sign(0) = 0`); `Sqrt` clamps negatives; `Reciprocal` guards zero — the two
/// guards are the only limits, and both produce `0`, bit-exact on both paths.
fn unary_one(x: f32, op: Op) -> f32 {
    match op {
        Op::Abs => x.abs(),
        Op::Negate => -x,
        Op::Sign => {
            if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                0.0
            }
        }
        Op::Floor => x.floor(),
        Op::Fract => x - x.floor(),
        Op::Square => x * x,
        Op::Sqrt => x.max(0.0).sqrt(),
        Op::Reciprocal => {
            if x != 0.0 {
                1.0 / x
            } else {
                0.0
            }
        }
    }
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.unary"),
    name: "value.unary",
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
        name: "op",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`unary_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out: the base rides here
/// (a VALUE stream carries only `v`). Every op is ported verbatim, including the
/// explicit `Sign` and the two guards (an `if`, so no `inf`/`NaN` is computed).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vu_op = i32(vu_round(params.op));\n\
        let vu_x = read_v(i);\n\
        var vu_o: f32;\n\
        switch (vu_op) {\n\
            case 1: { vu_o = -vu_x; }\n\
            case 2: { vu_o = select(select(0.0, -1.0, vu_x < 0.0), 1.0, vu_x > 0.0); }\n\
            case 3: { vu_o = floor(vu_x); }\n\
            case 4: { vu_o = vu_x - floor(vu_x); }\n\
            case 5: { vu_o = vu_x * vu_x; }\n\
            case 6: { vu_o = sqrt(max(vu_x, 0.0)); }\n\
            case 7: { if (vu_x != 0.0) { vu_o = 1.0 / vu_x; } else { vu_o = 0.0; } }\n\
            default: { vu_o = abs(vu_x); }\n\
        }\n\
        write_v(i, vu_o);\n",
    wgsl_lib: "\
        fn vu_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["op"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueUnary;

impl NodeOp for ValueUnary {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let op = Op::from_param(ctx.param("op"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input.iter().map(|&x| unary_one(x, op)).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueUnary))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Unary",
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
    param: "op",
    label: "Op",
    min: 0.0,
    max: 7.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &[
            "Abs",
            "Negate",
            "Sign",
            "Floor",
            "Fract",
            "Square",
            "Sqrt",
            "Reciprocal",
        ],
    },
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Every op computes its function** on representative inputs — the sign
    /// family, the rounding pair, and the algebraic transforms.
    #[test]
    fn every_op_computes_its_function() {
        assert_eq!(unary_one(-3.0, Op::Abs), 3.0);
        assert_eq!(unary_one(2.0, Op::Negate), -2.0);
        // Sign: explicit, so 0 (and ±0) map to 0 — NOT signum's ±1.
        assert_eq!(unary_one(5.0, Op::Sign), 1.0);
        assert_eq!(unary_one(-5.0, Op::Sign), -1.0);
        assert_eq!(unary_one(0.0, Op::Sign), 0.0, "sign(0) is 0, not signum's 1");
        assert_eq!(unary_one(-0.0, Op::Sign), 0.0, "sign(-0) is 0");
        assert_eq!(unary_one(2.7, Op::Floor), 2.0);
        assert_eq!(unary_one(-2.3, Op::Floor), -3.0, "floor rounds toward -inf");
        assert!((unary_one(2.7, Op::Fract) - 0.7).abs() < 1e-6);
        assert!((unary_one(-0.3, Op::Fract) - 0.7).abs() < 1e-6, "fract of negative wraps up");
        assert_eq!(unary_one(3.0, Op::Square), 9.0);
        assert_eq!(unary_one(9.0, Op::Sqrt), 3.0);
        assert_eq!(unary_one(4.0, Op::Reciprocal), 0.25);
    }

    /// **The two guards produce `0`, never `inf`/`NaN`** — Sqrt clamps negatives,
    /// Reciprocal guards zero, and every op stays finite for any input.
    #[test]
    fn the_guards_keep_it_finite() {
        assert_eq!(unary_one(-4.0, Op::Sqrt), 0.0, "sqrt of a negative clamps to 0");
        assert_eq!(unary_one(0.0, Op::Reciprocal), 0.0, "1/0 is guarded to 0");
        let ops = [
            Op::Abs,
            Op::Negate,
            Op::Sign,
            Op::Floor,
            Op::Fract,
            Op::Square,
            Op::Sqrt,
            Op::Reciprocal,
        ];
        for op in ops {
            for k in -50..50 {
                let x = k as f32 * 0.37;
                assert!(unary_one(x, op).is_finite(), "finite at x={x} {op:?}");
            }
        }
    }

    /// **Fract is exactly `x − Floor(x)`** — the two ops are the complementary
    /// pair, so their sum reconstructs the input for any value.
    #[test]
    fn floor_plus_fract_reconstructs_the_input() {
        for k in -40..40 {
            let x = k as f32 * 0.29;
            let recon = unary_one(x, Op::Floor) + unary_one(x, Op::Fract);
            assert!((recon - x).abs() < 1e-5, "floor + fract == x at x={x}");
        }
    }

    /// A value source emitting a fixed field, so `value.unary` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.unary.test.src"),
        name: "value.unary.test.src",
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

    /// End-to-end through the cook: a `[-2, 0, 3]` field through Abs becomes
    /// `[2, 0, 3]`, length preserved (the unary contract).
    #[test]
    fn maps_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueUnary),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![-2.0, 0.0, 3.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.unary.test.src");
        let vu = g.add_node("value.unary");
        g.set_param(vu, "op", 0.0); // Abs
        g.connect(Edge {
            from: (src, 0),
            to: (vu, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vu, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v, &vec![2.0, 0.0, 3.0], "abs of [-2, 0, 3]");
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
