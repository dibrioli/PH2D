#![forbid(unsafe_code)]
//! `value.smooth` — the value-domain FILTER: soften a field by averaging each
//! element with its index-neighbours, a box blur over the instance order (Motion
//! Nodes M2, the value domain — doc 12/77). It is the de-noisier: a jagged driver
//! — a `value.noise`, an `instance_field` Random — smoothed into a gradual one.
//! The Filter/Lag CHOP of TouchDesigner, the Smooth of Cavalry.
//!
//! **Unlike every other value node so far, element `i`'s answer reads its
//! NEIGHBOURS** — `v[i−r] … v[i+r]` — not just `v[i]`. It is the `Filter CHOP`
//! shape: a moving average over the ordered field. The order is meaningful when
//! the instances are laid out in sequence (a row, a grid) — the common case for a
//! per-instance driver.
//!
//! **`radius`** is the half-window (`0` = a bit-exact passthrough, the neutral
//! default). `out[i] = mean( v[clamp(i−r, 0, N−1)] … v[clamp(i+r, 0, N−1)] )` —
//! the edges **extend** (a clamped index repeats the boundary value), so the
//! window is always `2r+1` samples and the divisor is fixed. The window is summed
//! **left to right** on BOTH paths, so the parity is **bit-exact** — this is a
//! per-element fixed-order sum, NOT the tree reduction whose order the `reduce`
//! channel documents an ε for.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state);
//! length preserved. The GPU kernel reads the neighbours off the input buffer, so
//! it is **device-resident** (no CPU fallback) with the existing kernel channel —
//! no reduction, no scan.

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

/// Box-blur one field by `radius`. `radius = 0` is a passthrough (bit-exact). The
/// window is summed LEFT TO RIGHT with edge-clamped indices, exactly as the WGSL
/// does, so the two paths agree to the byte — a fixed-order per-element sum, not a
/// tree reduction. The divisor is the fixed window size `2r+1` (edges extend).
fn smooth(field: &[f32], radius: usize) -> Vec<f32> {
    let n = field.len();
    if radius == 0 || n == 0 {
        return field.to_vec();
    }
    let r = radius as isize;
    let last = n as isize - 1;
    (0..n)
        .map(|i| {
            let mut sum = 0.0f32;
            let mut k = i as isize - r;
            let hi = i as isize + r;
            while k <= hi {
                let idx = k.clamp(0, last) as usize;
                sum += field[idx];
                k += 1;
            }
            sum / (2 * radius + 1) as f32
        })
        .collect()
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.smooth"),
    name: "value.smooth",
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
        name: "radius",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`smooth`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU. VALUE in, VALUE out; the binding is `ReadWrite` because the kernel
/// READS the input (its neighbours, off `in_v`) and WRITES a fresh `out_v` — the
/// two are separate buffers, so a write never corrupts a neighbour read. The
/// window sum runs left to right, matching the CPU order exactly.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vs_r = i32(vs_round(params.radius));\n\
        if (vs_r <= 0) {\n\
            write_v(i, read_v(i));\n\
        } else {\n\
            let vs_last = i32(params.count) - 1;\n\
            var vs_sum = 0.0;\n\
            var vs_k = -vs_r;\n\
            loop {\n\
                if (vs_k > vs_r) { break; }\n\
                let vs_idx = clamp(i32(i) + vs_k, 0, vs_last);\n\
                vs_sum = vs_sum + read_v(u32(vs_idx));\n\
                vs_k = vs_k + 1;\n\
            }\n\
            write_v(i, vs_sum / f32(2 * vs_r + 1));\n\
        }\n",
    wgsl_lib: "\
        fn vs_round(x: f32) -> f32 {\n\
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
    params: &["radius"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueSmooth;

impl NodeOp for ValueSmooth {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Radius is a non-negative half-window; round and clamp at 0.
        let radius = ctx.param("radius").round().max(0.0) as usize;
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        let out = smooth(&input, radius);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueSmooth))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Smooth",
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
    // The half-window of the box blur. `0` is a passthrough; higher softens more.
    // The range is a UI hint for the common field size; the cost is `O(radius)`
    // per element (cheap for a value field).
    param: "radius",
    label: "Radius",
    min: 0.0,
    max: 16.0,
    step: 1.0,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Radius 0 is a bit-exact passthrough** — the neutral default; the field is
    /// returned untouched.
    #[test]
    fn radius_zero_is_the_identity() {
        let f = vec![3.0, -1.0, 7.5, 2.0];
        assert_eq!(smooth(&f, 0), f);
    }

    /// **A spike is spread across its window** — a single tall value in a flat
    /// field drops and its neighbours rise. A BOX blur spreads it to a flat
    /// PLATEAU (not a rounded peak — that is a Gaussian), and with zero boundaries
    /// the total is conserved (mass is moved, never created). `[0,0,9,0,0]` at
    /// radius 1 becomes `[0,3,3,3,0]`.
    #[test]
    fn a_spike_is_spread_across_its_window() {
        let f = vec![0.0, 0.0, 9.0, 0.0, 0.0];
        let out = smooth(&f, 1);
        assert!(out[2] < 9.0, "the spike drops");
        assert!(out[1] > 0.0 && out[3] > 0.0, "the neighbours rise");
        assert_eq!(out[1], out[2], "the window becomes a plateau, not a peak");
        assert_eq!(out[2], out[3], "the window becomes a plateau, not a peak");
        let before: f32 = f.iter().sum();
        let after: f32 = out.iter().sum();
        assert!(
            (before - after).abs() < 1e-5,
            "mass conserved (zero boundaries)"
        );
    }

    /// **A constant field is unchanged at any radius** — the average of a window of
    /// equal values is that value, and the edge-extend keeps it constant.
    #[test]
    fn a_constant_field_is_unchanged() {
        let f = vec![4.0; 7];
        for r in [0usize, 1, 3, 20] {
            assert_eq!(smooth(&f, r), f, "constant survives radius {r}");
        }
    }

    /// **The output is finite and length-preserving** for any field and radius,
    /// including a radius larger than the field (edge-extend clamps).
    #[test]
    fn output_is_finite_and_length_preserving() {
        let f = vec![-3.0, 100.0, -50.0, 0.0, 8.0];
        for r in [0usize, 1, 2, 5, 100] {
            let out = smooth(&f, r);
            assert_eq!(out.len(), f.len(), "length preserved at radius {r}");
            assert!(out.iter().all(|x| x.is_finite()), "finite at radius {r}");
        }
    }

    /// A value source emitting a fixed field, so `value.smooth` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.smooth.test.src"),
        name: "value.smooth.test.src",
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

    /// End-to-end through the cook: a `[0, 3, 0]` field through radius 1 becomes
    /// `[1, 1, 1]` (each window edge-extends and averages three values), length
    /// preserved.
    #[test]
    fn smooths_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueSmooth),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 3.0, 0.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.smooth.test.src");
        let vs = g.add_node("value.smooth");
        g.set_param(vs, "radius", 1.0);
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
                // i=0: [0,0,3]/3=1 · i=1: [0,3,0]/3=1 · i=2: [3,0,0]/3=1
                assert_eq!(
                    v,
                    &vec![1.0, 1.0, 1.0],
                    "each edge-extended window averages to 1"
                );
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
