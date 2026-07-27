#![forbid(unsafe_code)]
//! `value.normalize` — the value-domain FIT-TO-RANGE: bring a field into a
//! standard range using the field's OWN extent, discovered from the data (Motion
//! Nodes M2, the value domain — doc 12/74). It is the first **reducer** of the
//! value domain: where every other value node maps element `i` from element `i`,
//! this one's answer for `i` depends on a number that does not exist until every
//! element has been looked at — the field's min and max — so it is the
//! `reduce → broadcast → map` shape the deformers use, on a `v` column.
//!
//! **Why it matters.** The whole shaper family — `value.gain`, `value.curve`,
//! `value.step` — works on `[0,1]`, and the standing advice was "`map_range` a raw
//! driver first". But `map_range` needs you to KNOW the input range, and a
//! `value.noise`, an `instance_field` Random or a `value.attribute` has an unknown
//! range. `value.normalize` **discovers** it: no typed min/max, no guessing.
//!
//! **The gold standard is fit-to-range by extent** — Houdini's `fit()` with a
//! promoted detail min/max, TouchDesigner's Math CHOP "Range" (from auto), the
//! "Fit" of a grade. It runs `min` and `max` over the whole stream, which are
//! **bit-exact reductions** (associative and exact in any order — no ε, unlike a
//! `Sum`), so the GPU port matches the CPU term for term and the node is
//! **device-resident** (it cooks on the GPU, no CPU fallback).
//!
//! **`mode`** picks the target range:
//! - **Range** — `(v − min) / (max − min)` → `[0,1]`. The auto fit; where each
//!   element sits between the field's low and high.
//! - **MaxAbs** — `v / max(|min|, |max|)` → `[−1,1]`, **sign and zero preserved**.
//!   The right normalize for a BIPOLAR signal (an LFO or noise around `0`): Range
//!   would shift the zero, MaxAbs keeps it, scaling only to fill `[−1,1]`.
//!
//! A **constant** field has no extent (`max == min`, or all-zero for MaxAbs): the
//! degenerate answer is `0` (min maps to the low end; a zero field is already
//! centred). `Pure` (no clock, no state); a **unary** map, length preserved.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12).

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

/// Which standard range to fit into.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// `[0,1]` by the field's `[min, max]` — where each element sits in its range.
    Range,
    /// `[-1,1]` by `max(|min|, |max|)` — sign and zero preserved (bipolar signals).
    MaxAbs,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::MaxAbs,
            _ => Mode::Range,
        }
    }
}

/// Map one element given the field's `min` and `max` (already reduced). The
/// degenerate range (`max ≤ min` for Range, all-zero for MaxAbs) maps to `0` — a
/// constant field has no extent to sit in. Written to mirror the WGSL op for op
/// (subtraction then division; `min`/`max` are bit-exact reductions), so the
/// device matches the host.
fn normalize_one(v: f32, min: f32, max: f32, mode: Mode) -> f32 {
    match mode {
        Mode::Range => {
            let d = max - min;
            if d > 0.0 { (v - min) / d } else { 0.0 }
        }
        Mode::MaxAbs => {
            // max(|min|, |max|): the field's largest magnitude. Derives from the
            // SAME two reductions as Range — `-min` because `min` is the most
            // negative, so `-min` is its magnitude. `m == 0` only for an all-zero
            // field (max ≥ min always), and then the field is already centred.
            let m = max.max(-min);
            if m > 0.0 { v / m } else { 0.0 }
        }
    }
}

/// The static contract of this node type (ADR-0031). The kernel and its
/// reductions are side-metadata (ADR-0126); `NodeManifest` stays the frozen 8
/// fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.normalize"),
    name: "value.normalize",
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

/// The two whole-stream reductions the kernel reads: the field's `min` and `max`
/// over the `v` column (GPU/M5, the deformer channel — `ph2d_nodegraph::reduce_meta`).
/// Both fold the element VERBATIM (`value: "v"`), and both are `Max`/`Min`, so
/// they are **bit-exact in any evaluation order** — there is no product for the
/// device to contract into an FMA, so this stays exact against the CPU. The
/// identity is the `v` binding's own (`0.0`): an absent column reads as a constant
/// `0` field on BOTH paths, and normalizing that is `0`.
static REDUCES: &[ReduceSpec] = &[
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

/// GPU compute kernel (ADR-0126) — the WGSL port of [`normalize_one`] reading the
/// two reductions above, **fully device-resident**. No `applicable` gate — the
/// sequencer never falls back to the CPU (the "maximize GPU" north). VALUE in,
/// VALUE out: the base rides here (a VALUE stream carries only `v`). The
/// degenerate range is guarded by an `if`, so no NaN is even computed.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vn_mode = i32(vn_round(params.mode));\n\
        let vn_min = reduce_min();\n\
        let vn_max = reduce_max();\n\
        let vn_x = read_v(i);\n\
        var vn_o: f32 = 0.0;\n\
        if (vn_mode == 1) {\n\
            let vn_m = max(vn_max, -vn_min);\n\
            if (vn_m > 0.0) { vn_o = vn_x / vn_m; }\n\
        } else {\n\
            let vn_d = vn_max - vn_min;\n\
            if (vn_d > 0.0) { vn_o = (vn_x - vn_min) / vn_d; }\n\
        }\n\
        write_v(i, vn_o);\n",
    wgsl_lib: "\
        fn vn_round(x: f32) -> f32 {\n\
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
    params: &["mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueNormalize;

impl NodeOp for ValueNormalize {
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
        // The reduce → broadcast: the whole field's min and max, the CPU oracle
        // the device tree reduction is reconciled against (Max/Min, bit-exact).
        let min = ReduceOp::Min.cpu(&input);
        let max = ReduceOp::Max.cpu(&input);
        // The map: a unary pass, the field's length preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| normalize_one(v, min, max, mode))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueNormalize))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // GPU/M5: the whole-stream reductions the kernel reads. Side metadata on the
    // registry (ADR-0126) — the frozen node contract is untouched.
    reg.register_reduces(MANIFEST.id, REDUCES);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Normalize",
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
    max: 1.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &["Range", "MaxAbs"],
    },
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// The min/max the map reads are the field's own, via the same reduction the
    /// device runs (the CPU oracle for `Max`/`Min`). A helper for the unit tests.
    fn norm(field: &[f32], mode: Mode) -> Vec<f32> {
        let min = ReduceOp::Min.cpu(field);
        let max = ReduceOp::Max.cpu(field);
        field
            .iter()
            .map(|&v| normalize_one(v, min, max, mode))
            .collect()
    }

    /// **Range fits the field into `[0,1]` by its own extent** — the min lands on
    /// `0`, the max on `1`, and a value halfway sits at `0.5`, whatever the raw
    /// range is.
    #[test]
    fn range_fits_the_field_into_the_unit_band() {
        let out = norm(&[-3.0, 1.0, 5.0], Mode::Range); // min -3, max 5, mid 1
        assert_eq!(out[0], 0.0, "the min lands on 0");
        assert_eq!(out[2], 1.0, "the max lands on 1");
        assert!((out[1] - 0.5).abs() < 1e-6, "the midpoint sits at 0.5");
        // An already-normalised field is the identity.
        assert_eq!(norm(&[0.0, 0.25, 1.0], Mode::Range), vec![0.0, 0.25, 1.0]);
    }

    /// **MaxAbs fits into `[-1,1]` keeping sign and zero** — a bipolar field is
    /// scaled so its largest magnitude hits `±1`, and `0` stays `0` (Range would
    /// shift it).
    #[test]
    fn maxabs_keeps_sign_and_zero() {
        let out = norm(&[-2.0, 0.0, 1.0], Mode::MaxAbs); // maxabs = 2
        assert_eq!(out[0], -1.0, "the most negative hits -1");
        assert_eq!(out[1], 0.0, "zero stays zero");
        assert_eq!(out[2], 0.5, "a half-magnitude sits at 0.5");
        // The largest magnitude can be the positive end.
        assert_eq!(norm(&[-1.0, 4.0], Mode::MaxAbs), vec![-0.25, 1.0]);
    }

    /// **A degenerate field is finite and well-defined, never a divide-by-zero.**
    /// Range has no extent when `max == min` → `0` (the min maps to the low end).
    /// MaxAbs keeps sign, so a constant NONZERO field maps to its sign (`±1`, full
    /// magnitude), and only an all-zero field — which is already centred — maps to
    /// `0`.
    #[test]
    fn a_degenerate_field_is_finite_and_well_defined() {
        // Range: a constant collapses to 0 (no range to sit in).
        assert_eq!(norm(&[7.0, 7.0, 7.0], Mode::Range), vec![0.0, 0.0, 0.0]);
        // MaxAbs: a constant is at full magnitude → its sign.
        assert_eq!(norm(&[7.0, 7.0, 7.0], Mode::MaxAbs), vec![1.0, 1.0, 1.0]);
        assert_eq!(norm(&[-4.0, -4.0], Mode::MaxAbs), vec![-1.0, -1.0]);
        // All-zero: nothing to fit into on either mode → 0, finite (the guard).
        for &m in &[Mode::Range, Mode::MaxAbs] {
            assert_eq!(norm(&[0.0, 0.0], m), vec![0.0, 0.0], "all-zero → 0 ({m:?})");
        }
    }

    /// **The output is finite for any field** — the guards mean no reduction and
    /// no map ever divides by zero, on either mode.
    #[test]
    fn output_is_finite_for_any_field() {
        let fields: &[&[f32]] = &[
            &[1.0],
            &[-5.0, -5.0],
            &[0.0, 100.0, -100.0],
            &[1e-9, 2e-9],
            &[-1.0, 0.0, 1.0],
        ];
        for &m in &[Mode::Range, Mode::MaxAbs] {
            for f in fields {
                for &o in &norm(f, m) {
                    assert!(o.is_finite(), "finite for {f:?} {m:?}");
                }
            }
        }
    }

    /// A value source emitting a fixed field, so `value.normalize` can be driven
    /// through a real cook (the whole-chain proof, reduction included).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.normalize.test.src"),
        name: "value.normalize.test.src",
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

    /// End-to-end through the cook: a `[10, 20, 30, 40]` field (unknown range)
    /// through Range normalize becomes `[0, 1/3, 2/3, 1]`, length preserved (the
    /// unary contract) — the reduction discovered `min = 10`, `max = 40`.
    #[test]
    fn normalises_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueNormalize),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![10.0, 20.0, 30.0, 40.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.normalize.test.src");
        let vn = g.add_node("value.normalize");
        g.set_param(vn, "mode", 0.0); // Range
        g.connect(Edge {
            from: (src, 0),
            to: (vn, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vn, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 4, "unary: length 4 preserved");
                assert_eq!(v[0], 0.0, "min 10 -> 0");
                assert_eq!(v[3], 1.0, "max 40 -> 1");
                assert!((v[1] - 1.0 / 3.0).abs() < 1e-6, "20 -> 1/3");
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
