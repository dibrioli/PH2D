#![forbid(unsafe_code)]
//! `value.slope` — the value-domain DERIVATIVE: the rate of change of a field
//! across the instance order, the discrete `d(value)/d(index)` (Motion Nodes M2,
//! the value domain — doc 12/81). It is the exact SIBLING of `value.smooth`: where
//! smooth averages each element with its neighbours (a low-pass, softening the
//! field), this DIFFERENCES them (a high-pass, finding where the field CHANGES) —
//! zero on the flats, a spike at every edge. The Slope CHOP of TouchDesigner, the
//! `np.gradient` of NumPy, the image gradient made a first-class value operation.
//!
//! **Like `value.smooth`, element `i`'s answer reads its NEIGHBOURS** — `v[i−1]`
//! and `v[i+1]` — not just `v[i]`. The order is meaningful when the instances are
//! laid out in sequence (a row, a grid), the common case for a per-instance
//! driver, and the derivative is *across that order*.
//!
//! **Central in the interior, one-sided at the ends** (the `np.gradient` rule):
//! `out[i] = (v[i+1] − v[i−1]) / span · scale`, where `span` is the actual index
//! distance between the two neighbours read — `2` in the interior (a centred
//! difference), `1` at the boundaries (a forward/backward difference against the
//! edge-clamped index). Dividing by the true span keeps the edge slope HONEST (a
//! plain `/2` everywhere would halve the one-sided ends). A single-element (or
//! empty) field has no slope → `0`.
//!
//! **`scale`** amplifies the derivative (a slope is often small — the change from
//! one instance to the next — and you want to drive something visible with it);
//! negative flips its sign. It is the one knob a derivative node wants; the
//! *magnitude* of the slope (edge strength, direction-agnostic) is a `value.unary`
//! Abs away, so this node stays the SIGNED slope, single-purpose.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state); length
//! preserved. The GPU kernel reads the neighbours off the input buffer, so it is
//! **device-resident** (no CPU fallback) with the existing kernel channel — no
//! reduction, no scan; transcendental-free (a subtraction and a divide).

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

/// The discrete derivative of `field` across index, scaled by `scale`. Central in
/// the interior, one-sided at the ends, divided by the TRUE index span (the
/// `np.gradient` rule) — computed in exactly the WGSL's order (subtract, divide,
/// multiply) so the two paths agree within an FMA. A field of 0 or 1 elements has
/// no slope → all zeros.
fn slope(field: &[f32], scale: f32) -> Vec<f32> {
    let n = field.len();
    if n <= 1 {
        return vec![0.0; n];
    }
    let last = n - 1;
    (0..n)
        .map(|i| {
            let lo = i.saturating_sub(1); // clamp(i − 1, 0, last)
            let hi = (i + 1).min(last); // clamp(i + 1, 0, last)
            let span = (hi - lo) as f32; // 2 interior, 1 at the boundaries
            (field[hi] - field[lo]) / span * scale
        })
        .collect()
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.slope"),
    name: "value.slope",
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
        name: "scale",
        default: 1.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`slope`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to the
/// CPU. VALUE in, VALUE out; the binding is `ReadWrite` because the kernel READS
/// the input (its two neighbours, off `in_v`) and WRITES a fresh `out_v` — separate
/// buffers, so a write never corrupts a neighbour read (the `value.smooth`
/// pattern). `params.count` gives `N` for the edge clamp. A field of `≤ 1` element
/// has no slope and writes `0` (the span guard — a zero span would divide). The
/// arithmetic runs subtract → divide → multiply, matching the CPU order, so the
/// device result is bit-comparable (only an FMA on `· scale` may differ, ε below
/// the parity budget).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vsl_last = i32(params.count) - 1;\n\
        if (vsl_last <= 0) {\n\
            write_v(i, 0.0);\n\
        } else {\n\
            let vsl_lo = clamp(i32(i) - 1, 0, vsl_last);\n\
            let vsl_hi = clamp(i32(i) + 1, 0, vsl_last);\n\
            let vsl_span = f32(vsl_hi - vsl_lo);\n\
            let vsl_d = (read_v(u32(vsl_hi)) - read_v(u32(vsl_lo)))\n\
            \x20   / vsl_span * params.scale;\n\
            write_v(i, vsl_d);\n\
        }\n",
    wgsl_lib: "",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["scale"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueSlope;

impl NodeOp for ValueSlope {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let scale = ctx.param("scale");
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        let out = slope(&input, scale);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueSlope))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Slope",
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
    // Amplifies the derivative (slopes are small); negative flips the sign.
    param: "scale",
    label: "Scale",
    min: -8.0,
    max: 8.0,
    step: 0.05,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **A ramp has a CONSTANT slope** — the derivative of a line is its rate,
    /// the same at every element (the edge-clamped ends match, `np.gradient`
    /// style). A `[0,1,2,3]` ramp has slope `1` everywhere. Falsifiable: a raw
    /// central `/2` without the span fix would report `0.5` at the ends.
    #[test]
    fn a_ramp_has_a_constant_slope() {
        assert_eq!(slope(&[0.0, 1.0, 2.0, 3.0], 1.0), vec![1.0, 1.0, 1.0, 1.0]);
        // A steeper ramp: step 2 → slope 2 everywhere.
        assert_eq!(slope(&[0.0, 2.0, 4.0], 1.0), vec![2.0, 2.0, 2.0]);
        // A falling ramp → a negative slope.
        assert_eq!(slope(&[3.0, 2.0, 1.0], 1.0), vec![-1.0, -1.0, -1.0]);
    }

    /// **A constant field has ZERO slope** — nothing changes, so the derivative is
    /// zero everywhere, at any scale.
    #[test]
    fn a_constant_field_has_zero_slope() {
        for &s in &[1.0, 3.0, -2.0] {
            assert_eq!(slope(&[5.0; 6], s), vec![0.0; 6], "flat -> 0 (scale {s})");
        }
    }

    /// **An edge is a SPIKE in the slope** — the whole point: the flats read `0`
    /// and the transition reads nonzero. A `[0,0,1,1]` step: the flats (i=0, i=3)
    /// are `0`, and the jump spreads over the two central samples (the centred
    /// difference), each `(1−0)/2 = 0.5`. This is edge detection.
    #[test]
    fn an_edge_is_a_spike_in_the_slope() {
        let out = slope(&[0.0, 0.0, 1.0, 1.0], 1.0);
        assert_eq!(out[0], 0.0, "flat before the edge");
        assert_eq!(out[3], 0.0, "flat after the edge");
        assert!(out[1] > 0.0 && out[2] > 0.0, "the edge reads nonzero");
        assert_eq!(out[1], 0.5, "centred difference over the jump");
        assert_eq!(out[2], 0.5, "centred difference over the jump");
    }

    /// **`scale` amplifies and can flip the sign** of the derivative.
    #[test]
    fn scale_amplifies_and_flips_the_slope() {
        assert_eq!(slope(&[0.0, 1.0, 2.0], 3.0), vec![3.0, 3.0, 3.0], "x3");
        assert_eq!(slope(&[0.0, 1.0, 2.0], -1.0), vec![-1.0, -1.0, -1.0], "flipped");
    }

    /// **A single element (or empty) has no slope** → `0`, finite, never a divide
    /// by a zero span. Length is always preserved.
    #[test]
    fn a_degenerate_field_has_no_slope_and_stays_finite() {
        assert_eq!(slope(&[], 1.0), Vec::<f32>::new(), "empty stays empty");
        assert_eq!(slope(&[42.0], 1.0), vec![0.0], "one element -> 0");
        // A general field is finite and length-preserving at any scale.
        let f = vec![-3.0, 100.0, -50.0, 0.0, 8.0];
        let out = slope(&f, 2.5);
        assert_eq!(out.len(), f.len(), "length preserved");
        assert!(out.iter().all(|x| x.is_finite()), "finite");
    }

    /// A value source emitting a fixed field, so `value.slope` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.slope.test.src"),
        name: "value.slope.test.src",
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

    /// End-to-end through the cook: a `[0, 1, 2, 3]` ramp becomes a constant slope
    /// `[1, 1, 1, 1]`, length preserved (the unary contract).
    #[test]
    fn differentiates_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueSlope),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 1.0, 2.0, 3.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.slope.test.src");
        let vsl = g.add_node("value.slope");
        g.connect(Edge {
            from: (src, 0),
            to: (vsl, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vsl, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0, 1.0, 1.0], "the ramp's constant slope"),
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
