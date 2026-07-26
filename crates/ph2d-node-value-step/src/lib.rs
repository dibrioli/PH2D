#![forbid(unsafe_code)]
//! `value.step` — the value-domain GATE: turn a continuous field into a `[0,1]`
//! MASK by comparing it to a threshold, with a **hard** cut or a **smooth** band
//! (Motion Nodes M2, the value domain — doc 12/73). Where `value.gain` reshapes a
//! field into another field of the same shape, this one **thresholds** it: it is
//! the comparator that produces the weight a `value.mix` crossfades on or a
//! `value.switch` selects with — the piece the shapers feed and the combiners
//! consume.
//!
//! **The gold standard is the Step / Smoothstep pair** that every procedural tool
//! ships: Houdini VEX `step()`/`smoothstep()`, Unity/Unreal Shader Graph's Step
//! and Smoothstep, TouchDesigner's Logic/Limit CHOP, Cavalry's Compare. Both are
//! **transcendental-free** (HR-5) — Hard is a comparison, Smooth is the Hermite
//! cubic `3t² − 2t³` — so the GPU port is bit-comparable to the CPU and the node
//! is **device-resident** (it cooks on the GPU, no CPU fallback).
//!
//! **`mode`** picks the edge:
//! - **Hard** — a binary gate: `v ≥ threshold → 1`, else `0`. `width` is ignored.
//! - **Smooth** — a smoothstep band of full **`width`** centred on `threshold`:
//!   `0` below `threshold − width/2`, `1` above `threshold + width/2`, `0.5` at
//!   the threshold, a C¹ ramp between. `width = 0` collapses to the hard step.
//!
//! **The output is a `[0,1]` mask; the input is NOT clamped.** A comparison is
//! meaningful on any scale — a `value.step` at `threshold = 2` on an
//! `instance_field` Index picks "the instances past index 2" — so the driver
//! rides in on whatever range it has; only the result is normalised. `Pure` (no
//! clock, no state); a **unary** map, length preserved.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). Feed a `value.map_range` before it
//! to place the threshold on a convenient scale.

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

/// Which edge the gate has.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// A binary cut at the threshold — `width` is ignored.
    Hard,
    /// A smoothstep band of `width`, centred on the threshold.
    Smooth,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Smooth,
            _ => Mode::Hard,
        }
    }
}

/// Smoothstep: the Hermite cubic `3t² − 2t³` over the band `[lo, hi]`, clamped to
/// `[0,1]` outside it. A **degenerate band** (`hi ≤ lo`, i.e. `width = 0`)
/// collapses to a hard step at `lo` — the same answer the Hard mode gives, so the
/// two modes agree exactly at zero width. Transcendental-free.
fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    if hi <= lo {
        return if x >= lo { 1.0 } else { 0.0 };
    }
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Gate one value into a `[0,1]` mask. The input is NOT clamped (a comparison is
/// meaningful on any scale); only the output is normalised. `width` is a **full**
/// band centred on `threshold` and is ignored in Hard mode.
fn step_one(v: f32, threshold: f32, width: f32, mode: Mode) -> f32 {
    match mode {
        Mode::Hard => {
            if v >= threshold {
                1.0
            } else {
                0.0
            }
        }
        Mode::Smooth => {
            let w = width.max(0.0);
            smoothstep(threshold - 0.5 * w, threshold + 0.5 * w, v)
        }
    }
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.step"),
    name: "value.step",
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
            name: "threshold",
            default: 0.5,
        },
        ParamSpec {
            name: "width",
            default: 0.0,
        },
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`step_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out: the base rides here
/// (a VALUE stream carries only `v`). The comparison, the band and the Hermite
/// cubic are ported verbatim.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vs_mode = i32(vs_round(params.mode));\n\
        let vs_th = params.threshold;\n\
        let vs_x = read_v(i);\n\
        var vs_o: f32;\n\
        if (vs_mode == 1) {\n\
            let vs_w = max(params.width, 0.0);\n\
            vs_o = vs_smoothstep(vs_th - 0.5 * vs_w, vs_th + 0.5 * vs_w, vs_x);\n\
        } else {\n\
            vs_o = select(0.0, 1.0, vs_x >= vs_th);\n\
        }\n\
        write_v(i, vs_o);\n",
    wgsl_lib: "\
        fn vs_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn vs_smoothstep(lo: f32, hi: f32, x: f32) -> f32 {\n\
            if (hi <= lo) { return select(0.0, 1.0, x >= lo); }\n\
            let t = clamp((x - lo) / (hi - lo), 0.0, 1.0);\n\
            return t * t * (3.0 - 2.0 * t);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["threshold", "width", "mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueStep;

impl NodeOp for ValueStep {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let threshold = ctx.param("threshold");
        let width = ctx.param("width");
        let mode = Mode::from_param(ctx.param("mode"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| step_one(v, threshold, width, mode))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueStep))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Step",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // Where the gate cuts, on the input's scale. The normalised-driver convention
    // (a `[0,1]` field) makes `0..1` the natural range; map_range a raw driver
    // first to place the threshold where you want it.
    ParamUiHint {
        param: "threshold",
        label: "Threshold",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // The full width of the smooth band (Smooth mode only). `0` = a hard edge.
    ParamUiHint {
        param: "width",
        label: "Width",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Hard", "Smooth"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Hard is a binary gate at the threshold** — strictly below is `0`, at or
    /// above is `1`, exact (no band). `width` is ignored.
    #[test]
    fn hard_is_a_binary_gate_at_the_threshold() {
        for &w in &[0.0, 0.3, 0.9] {
            assert_eq!(step_one(0.49, 0.5, w, Mode::Hard), 0.0, "below is 0 (w={w})");
            assert_eq!(step_one(0.5, 0.5, w, Mode::Hard), 1.0, "at is 1 (w={w})");
            assert_eq!(step_one(0.51, 0.5, w, Mode::Hard), 1.0, "above is 1 (w={w})");
        }
        // The threshold moves the cut, on any scale.
        assert_eq!(step_one(1.9, 2.0, 0.0, Mode::Hard), 0.0, "below index 2");
        assert_eq!(step_one(2.0, 2.0, 0.0, Mode::Hard), 1.0, "at index 2");
    }

    /// **Smooth ramps across the band and pins the ends** — `0` below the band,
    /// `1` above it, `0.5` at the threshold, and strictly increasing between.
    #[test]
    fn smooth_ramps_across_the_band_and_pins_the_ends() {
        let (th, w) = (0.5, 0.4); // band [0.3, 0.7]
        assert_eq!(step_one(0.29, th, w, Mode::Smooth), 0.0, "below the band is 0");
        assert_eq!(step_one(0.71, th, w, Mode::Smooth), 1.0, "above the band is 1");
        assert!((step_one(0.5, th, w, Mode::Smooth) - 0.5).abs() < 1e-6, "0.5 at the threshold");
        let mut prev = -1.0;
        for k in 0..=40 {
            let x = 0.3 + 0.4 * (k as f32 / 40.0);
            let o = step_one(x, th, w, Mode::Smooth);
            assert!(o >= prev, "monotone rising across the band at x={x}");
            prev = o;
        }
    }

    /// **Zero width collapses Smooth onto Hard, bit-exact** — a degenerate band is
    /// the same comparison, so the two modes agree at the seam.
    #[test]
    fn width_zero_smooth_equals_hard() {
        for k in -20..=20 {
            let x = k as f32 * 0.07;
            for &th in &[-0.3, 0.0, 0.5, 1.2] {
                assert_eq!(
                    step_one(x, th, 0.0, Mode::Smooth),
                    step_one(x, th, 0.0, Mode::Hard),
                    "smooth(width 0) == hard at x={x} th={th}"
                );
            }
        }
    }

    /// **The output is a finite `[0,1]` mask** for any input, threshold, width and
    /// mode — the input is unclamped, but the result is always a normalised weight.
    #[test]
    fn output_is_a_finite_mask_in_0_1() {
        for &m in &[Mode::Hard, Mode::Smooth] {
            for &th in &[-1.0, 0.0, 0.5, 3.0] {
                for &w in &[-0.5, 0.0, 0.4, 2.0] {
                    for k in -50..50 {
                        let v = k as f32 * 0.11;
                        let o = step_one(v, th, w, m);
                        assert!(o.is_finite(), "finite at v={v} th={th} w={w} {m:?}");
                        assert!((0.0..=1.0).contains(&o), "in [0,1] at v={v} th={th} w={w} {m:?}");
                    }
                }
            }
        }
    }

    /// A value source emitting a fixed field, so `value.step` can be driven through
    /// a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.step.test.src"),
        name: "value.step.test.src",
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

    /// End-to-end through the cook: a `[0, 0.25, 0.5, 0.75, 1]` ramp through a
    /// hard gate at `0.5` becomes `[0, 0, 1, 1, 1]`, length preserved (the unary
    /// contract).
    #[test]
    fn shapes_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueStep),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.step.test.src");
        let vs = g.add_node("value.step");
        g.set_param(vs, "threshold", 0.5);
        g.set_param(vs, "mode", 0.0); // Hard
        g.connect(Edge {
            from: (src, 0),
            to: (vs, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vs, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v, &vec![0.0, 0.0, 1.0, 1.0, 1.0], "gate at 0.5 (>= is 1)");
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
