#![forbid(unsafe_code)]
//! `value.quantize` — the value-domain STAIRCASE: snap a value to a grid of
//! spacing `step`, the "stepped / posterized" look (Motion Nodes M2, the value
//! domain — doc 12/71). It is the third value SHAPER, alongside `value.map_range`
//! (linear) and `value.curve` (freeform): where those move a value smoothly, this
//! collapses a continuous field onto discrete levels — the signature motion-
//! graphics move (stop-motion cadence, chunky drives, retro quantization). Every
//! mature tool ships it: TouchDesigner's **Limit CHOP** (Quantize), Blender's
//! **Snap**/increment, After Effects' **Posterize**, Cavalry's **Quantize**.
//!
//! **Grid by SIZE, not by count** (doc 71): `q = round(v / step) · step` snaps `v`
//! to the nearest multiple of `step` — range-agnostic, so it composes (a
//! `value.map_range`/`value.curve` before or after sets the range). `step = 0.25`
//! turns a `[0,1]` ramp into `{0, 0.25, 0.5, 0.75, 1}`; `step = 0` is a passthrough
//! (the identity — safe to drop into a graph before choosing a grid).
//!
//! **`mode`** picks the rounding: **Round** (nearest — a symmetric staircase),
//! **Floor** (snaps DOWN — the sample-and-hold staircase, values never exceed the
//! input), **Ceil** (snaps UP). Round matches Rust's half-away-from-zero via the
//! `vq_round` helper (WGSL `round` is half-to-even), so CPU and GPU agree.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). A **unary** map — length preserved,
//! no broadcast decision (that lives in the consumer). `Pure` (no clock, no state).
//! Transcendental-free (HR-5): `/ × floor ceil` and one guarded compare. The GPU
//! kernel is the WGSL port of the same snap, so the node is **device-resident** —
//! it cooks on the GPU, no CPU fallback.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_curve::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// Below this the grid spacing is treated as zero — the node is a passthrough
/// (the identity), never a divide by (near-)zero.
const MIN_STEP: f32 = 1e-9;

/// The rounding mode: which multiple of `step` a value snaps to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Nearest multiple (a symmetric staircase). Half away from zero, matching
    /// Rust's `f32::round` and the WGSL `vq_round` helper.
    Round,
    /// The multiple at or below `v` (the sample-and-hold staircase — snaps down).
    Floor,
    /// The multiple at or above `v` (snaps up).
    Ceil,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Floor,
            2 => Mode::Ceil,
            _ => Mode::Round,
        }
    }
}

/// Snap one value to the grid: `q = mode(v / step) · step`. A (near-)zero `step`
/// is a passthrough — the identity, never a divide-by-zero `NaN`.
fn quantize_one(v: f32, step: f32, mode: Mode) -> f32 {
    if step.abs() < MIN_STEP {
        return v;
    }
    let n = v / step;
    // `f32::round` is half-away-from-zero (== the WGSL `vq_round` helper); floor /
    // ceil agree between Rust and WGSL exactly.
    let k = match mode {
        Mode::Round => n.round(),
        Mode::Floor => n.floor(),
        Mode::Ceil => n.ceil(),
    };
    k * step
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.quantize"),
    name: "value.quantize",
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
    params: &[
        ParamSpec {
            name: "step",
            default: 0.25,
        },
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`quantize_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out: the base rides here
/// (a VALUE stream carries only `v`, so "base + written `v`" and "a fresh stream
/// with `v`" are the same stream). The `MIN_STEP` guard is ported verbatim.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vq_step = params.step;\n\
        var vq = read_v(i);\n\
        if (abs(vq_step) >= 1e-9) {\n\
        \x20   let vq_n = vq / vq_step;\n\
        \x20   let vq_mode = i32(vq_round(params.mode));\n\
        \x20   var vq_k = vq_round(vq_n);\n\
        \x20   if (vq_mode == 1) { vq_k = floor(vq_n); }\n\
        \x20   else if (vq_mode == 2) { vq_k = ceil(vq_n); }\n\
        \x20   vq = vq_k * vq_step;\n\
        }\n\
        write_v(i, vq);\n",
    wgsl_lib: "\
        fn vq_round(x: f32) -> f32 {\n\
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
    params: &["step", "mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueQuantize;

impl NodeOp for ValueQuantize {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let step = ctx.param("step");
        let mode = Mode::from_param(ctx.param("mode"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input.iter().map(|&v| quantize_one(v, step, mode)).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueQuantize))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Quantize",
            // Utility grey: a value→value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The grid spacing. `0` is a passthrough (the identity); raise it for a
    // coarser staircase. In the input's units — a `[0,1]` ramp with 0.25 gives 5
    // levels.
    ParamUiHint {
        param: "step",
        label: "Step",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Round", "Floor", "Ceil"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **A `[0,1]` field snaps to a `0.25` grid** — the core behaviour. Round sends
    /// each value to its NEAREST multiple, so a smooth ramp becomes a staircase.
    #[test]
    fn round_snaps_to_the_nearest_multiple() {
        let q = |v| quantize_one(v, 0.25, Mode::Round);
        assert_eq!(q(0.0), 0.0);
        assert_eq!(q(0.1), 0.0, "0.1 rounds down to 0");
        assert_eq!(q(0.2), 0.25, "0.2 rounds up to 0.25");
        assert_eq!(q(0.6), 0.5, "0.6 rounds down to 0.5");
        assert_eq!(q(0.9), 1.0, "0.9 rounds up to 1.0");
    }

    /// **Floor vs Ceil vs Round differ** — the whole point of the mode. At `v = 0.6`
    /// on a `0.25` grid: floor holds at 0.5, ceil jumps to 0.75, round nears 0.5.
    #[test]
    fn floor_and_ceil_bracket_the_value() {
        assert_eq!(
            quantize_one(0.6, 0.25, Mode::Floor),
            0.5,
            "floor snaps down"
        );
        assert_eq!(quantize_one(0.6, 0.25, Mode::Ceil), 0.75, "ceil snaps up");
        assert_eq!(quantize_one(0.6, 0.25, Mode::Round), 0.5, "round nears 0.5");
        // Floor never exceeds the input; ceil is never below it.
        for k in 0..100 {
            let v = k as f32 * 0.037;
            assert!(
                quantize_one(v, 0.25, Mode::Floor) <= v + 1e-6,
                "floor ≤ v at {v}"
            );
            assert!(
                quantize_one(v, 0.25, Mode::Ceil) >= v - 1e-6,
                "ceil ≥ v at {v}"
            );
        }
    }

    /// **`step = 0` is a passthrough** — the identity, never a divide-by-zero NaN.
    /// The reason the node is safe to drop into a graph before choosing a grid.
    #[test]
    fn a_zero_step_is_a_passthrough() {
        for k in 0..50 {
            let v = k as f32 * 0.13 - 3.0;
            let out = quantize_one(v, 0.0, Mode::Round);
            assert!(out.is_finite(), "finite at {v}");
            assert_eq!(out, v, "step 0 passes the value through unchanged");
        }
    }

    /// **Negative values snap symmetrically** (round is half-away-from-zero, the
    /// GPU `vq_round` match): `-0.6` on a `0.25` grid rounds to `-0.5`, not `-0.75`.
    #[test]
    fn negatives_round_half_away_from_zero() {
        assert_eq!(quantize_one(-0.6, 0.25, Mode::Round), -0.5);
        assert_eq!(
            quantize_one(-0.125, 0.25, Mode::Round),
            -0.25,
            "0.5 case away from 0"
        );
        assert_eq!(
            quantize_one(-0.6, 0.25, Mode::Floor),
            -0.75,
            "floor snaps toward -inf"
        );
        assert_eq!(
            quantize_one(-0.6, 0.25, Mode::Ceil),
            -0.5,
            "ceil snaps toward +inf"
        );
    }

    /// A value source emitting a fixed field, so `value.quantize` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.quantize.test.src"),
        name: "value.quantize.test.src",
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

    /// End-to-end through the cook: a length-5 ramp `[0, 0.25, 0.5, 0.75, 1]` is
    /// already on the `0.25` grid, so it survives; the OFF-grid `[0.1, 0.4, 0.9]`
    /// snaps to `[0, 0.5, 1]`. The length is preserved (the unary contract).
    #[test]
    fn quantizes_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueQuantize),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.1, 0.4, 0.9]);
        let mut g = Graph::new();
        let src = g.add_node("value.quantize.test.src");
        let vq = g.add_node("value.quantize");
        g.set_param(vq, "step", 0.5); // a coarse grid: {0, 0.5, 1}
        g.connect(Edge {
            from: (src, 0),
            to: (vq, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vq, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 3, "unary: length 3 preserved");
                assert_eq!(v, &vec![0.0, 0.5, 1.0], "snapped to the 0.5 grid");
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
